// AEGIS — zokastech.fr — Apache 2.0 / MIT

mod sarif;

use aegis_anonymize::{AnonymizationConfig, AnonymizerEngine};
use aegis_core::anonymizer::{OperatorConfig, OperatorType};
use aegis_core::config::AnalysisConfig;
use aegis_core::engine::{AnalyzerEngine, AnalyzerEngineBuilder, PipelineLevel};
use aegis_core::entity::AnalysisResult;
use clap::{Parser, Subcommand, ValueEnum};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use walkdir::WalkDir;

use aegis_regex as _;

const DEFAULT_AEGIS_YAML: &str = r#"# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Default engine configuration — tune for your environment.

recognizers:
  default_regex:
    enabled: true
    languages: [en, fr]
  disabled: []

pipeline_level: 2

analysis:
  language: fr
  score_threshold: 0.5
"#;

const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Parser)]
#[command(
    name = "aegis",
    version,
    about = "AEGIS — PII detection and anonymization (zokastech.fr)"
)]
struct Cli {
    #[command(flatten)]
    global: GlobalOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
struct GlobalOpts {
    /// Regex recognizer languages (comma-separated), e.g. fr,en,de
    #[arg(long = "language", global = true, default_value = "fr,en")]
    language: String,

    /// Minimum detection score threshold (0.0–1.0)
    #[arg(long, global = true, default_value_t = 0.5)]
    score_threshold: f64,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Engine config file (YAML or JSON)
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Disable color output (see also NO_COLOR)
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Json,
    Table,
    Sarif,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a file, directory (recursive), or stdin when no paths given
    Scan {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Analyze text from the command line
    Analyze {
        #[arg(required = true)]
        text: String,
    },
    /// Anonymize a file to an output path
    Anonymize {
        path: PathBuf,
        #[arg(short = 'o', long = "output")]
        output: PathBuf,
    },
    /// Manage engine configuration file
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Benchmark analysis performance on a file
    Benchmark {
        path: PathBuf,
        #[arg(long, default_value_t = 10)]
        iterations: u32,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Write a minimal aegis.yaml
    Init {
        #[arg(default_value = "aegis.yaml")]
        output: PathBuf,
        #[arg(long)]
        force: bool,
    },
    /// Validate an engine configuration file
    Validate {
        #[arg(default_value = "aegis.yaml")]
        path: PathBuf,
    },
}

#[derive(Serialize)]
struct ScanRecord {
    path: String,
    #[serde(flatten)]
    analysis: AnalysisResult,
}

fn parse_languages(csv: &str) -> Vec<String> {
    csv.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn scan_engine(
    languages_csv: &str,
    config_path: Option<&Path>,
) -> Result<AnalyzerEngine, Box<dyn std::error::Error>> {
    let langs = parse_languages(languages_csv);
    let sl: Vec<&str> = if langs.is_empty() {
        vec!["fr", "en"]
    } else {
        langs.iter().map(|s| s.as_str()).collect()
    };
    let mut b = AnalyzerEngineBuilder::new()
        .with_default_recognizers(&sl)
        .with_pipeline_level(PipelineLevel::Two);
    if let Some(p) = config_path {
        let content = fs::read_to_string(p)?;
        let lower = p.to_string_lossy().to_lowercase();
        b = if lower.ends_with(".json") {
            b.with_engine_json_str(&content)?
        } else {
            b.with_engine_yaml_str(&content)?
        };
    }
    Ok(b.build()?)
}

fn analysis_config(global: &GlobalOpts) -> AnalysisConfig {
    let mut cfg = AnalysisConfig::default();
    cfg.score_threshold = global.score_threshold;
    let langs = parse_languages(&global.language);
    if let Some(first) = langs.first() {
        cfg.language = Some(first.clone());
    }
    cfg
}

fn use_color(global: &GlobalOpts) -> bool {
    !global.no_color && std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn print_table(path_label: &str, res: &AnalysisResult, color: bool) {
    if res.entities.is_empty() {
        let msg = format!("{} — aucune PII détectée", path_label);
        if color {
            println!("{}", msg.green());
        } else {
            println!("{msg}");
        }
        return;
    }
    let title = format!("{} — {} entité(s)", path_label, res.entities.len());
    if color {
        println!("{}", title.red().bold());
    } else {
        println!("{title}");
    }
    println!("{:-<80}", "");
    println!(
        "{:<14} {:>6} {:>6} {:>7}  {}",
        "type", "début", "fin", "score", "extrait"
    );
    for e in &res.entities {
        let snippet: String = e.text.chars().take(48).collect();
        println!(
            "{:<14} {:>6} {:>6} {:>7.3}  {}",
            e.entity_type.config_key(),
            e.start,
            e.end,
            e.score,
            snippet.replace('\n', " ")
        );
    }
}

fn collect_scan_targets(
    paths: &[PathBuf],
    cwd: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    if paths.is_empty() {
        return Ok(out);
    }
    for p in paths {
        let fp = if p.is_absolute() {
            p.clone()
        } else {
            cwd.join(p)
        };
        let meta = fs::metadata(&fp)?;
        if meta.is_dir() {
            for entry in WalkDir::new(&fp).into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() {
                    continue;
                }
                let f = entry.path().to_path_buf();
                if fs::metadata(&f)?.len() > MAX_FILE_BYTES {
                    continue;
                }
                out.push(f);
            }
        } else if meta.len() <= MAX_FILE_BYTES {
            out.push(fp);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn file_uri_for(path: &Path) -> String {
    match fs::canonicalize(path) {
        Ok(abs) => format!("file://{}", abs.display()),
        Err(_) => format!("file://{}", path.display()),
    }
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let color = use_color(&cli.global);
    let cfg = analysis_config(&cli.global);
    let config_ref = cli.global.config.as_deref();

    match &cli.command {
        Commands::Scan { paths } => {
            let eng = scan_engine(&cli.global.language, config_ref)?;
            let cwd = std::env::current_dir()?;
            if paths.is_empty() {
                let mut text = String::new();
                io::stdin().read_to_string(&mut text)?;
                let res = eng.analyze(&text, Some(cfg.clone()))?;
                match cli.global.format {
                    OutputFormat::Json => {
                        let rec = ScanRecord {
                            path: "-".into(),
                            analysis: res.clone(),
                        };
                        println!("{}", serde_json::to_string_pretty(&vec![rec])?);
                    }
                    OutputFormat::Table => print_table("- (stdin)", &res, color),
                    OutputFormat::Sarif => {
                        let doc = sarif::build_document(
                            env!("CARGO_PKG_VERSION"),
                            &[("file:///dev/stdin".into(), text.clone(), res.clone())],
                        );
                        println!("{}", serde_json::to_string_pretty(&doc)?);
                    }
                }
                let code = if res.entities.is_empty() { 0 } else { 1 };
                return Ok(code);
            }

            let targets = collect_scan_targets(paths, &cwd)?;
            let mut batch: Vec<(PathBuf, String, AnalysisResult)> = Vec::new();
            for fp in &targets {
                let text = match fs::read_to_string(fp) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                let res = eng.analyze(&text, Some(cfg.clone()))?;
                batch.push((fp.clone(), text, res));
            }

            match cli.global.format {
                OutputFormat::Json => {
                    let records: Vec<ScanRecord> = batch
                        .iter()
                        .map(|(fp, _, r)| ScanRecord {
                            path: fp
                                .strip_prefix(&cwd)
                                .unwrap_or(fp.as_path())
                                .display()
                                .to_string(),
                            analysis: r.clone(),
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&records)?);
                }
                OutputFormat::Table => {
                    for (fp, _, r) in &batch {
                        let label = fp.strip_prefix(&cwd).unwrap_or(fp).display().to_string();
                        print_table(&label, r, color);
                        println!();
                    }
                }
                OutputFormat::Sarif => {
                    let sarif_batch: Vec<(String, String, AnalysisResult)> = batch
                        .iter()
                        .map(|(fp, t, r)| (file_uri_for(fp), t.clone(), r.clone()))
                        .collect();
                    let doc = sarif::build_document(env!("CARGO_PKG_VERSION"), &sarif_batch);
                    println!("{}", serde_json::to_string_pretty(&doc)?);
                }
            }

            let total: usize = batch.iter().map(|(_, _, r)| r.entities.len()).sum();
            Ok(if total == 0 { 0 } else { 1 })
        }

        Commands::Analyze { text } => {
            let eng = scan_engine(&cli.global.language, config_ref)?;
            let res = eng.analyze(text, Some(cfg))?;
            match cli.global.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&res)?),
                OutputFormat::Table => print_table("(analyze)", &res, color),
                OutputFormat::Sarif => {
                    let doc = sarif::build_document(
                        env!("CARGO_PKG_VERSION"),
                        &[("file:///inline".into(), text.clone(), res.clone())],
                    );
                    println!("{}", serde_json::to_string_pretty(&doc)?);
                }
            }
            Ok(if res.entities.is_empty() { 0 } else { 1 })
        }

        Commands::Anonymize { path, output } => {
            let eng = scan_engine(&cli.global.language, config_ref)?;
            let text = fs::read_to_string(path)?;
            let res = eng.analyze(&text, Some(cfg))?;
            let mut per_type = HashMap::new();
            per_type.insert(
                "EMAIL".into(),
                OperatorConfig {
                    operator_type: OperatorType::Mask,
                    params: [
                        ("visible_prefix".into(), "2".into()),
                        ("visible_suffix".into(), "0".into()),
                    ]
                    .into_iter()
                    .collect(),
                },
            );
            let anon_cfg = AnonymizationConfig {
                operators_by_entity: per_type,
                default_operator: None,
            };
            let anon = AnonymizerEngine::new().anonymize(&text, &res.entities, &anon_cfg);
            fs::write(output, &anon.text)?;
            let msg = format!(
                "Écrit : {} ({} transformations)",
                output.display(),
                anon.transformations.len()
            );
            if color {
                eprintln!("{}", msg.cyan());
            } else {
                eprintln!("{msg}");
            }
            Ok(0)
        }

        Commands::Config { cmd } => match cmd {
            ConfigCmd::Init { output, force } => {
                if output.exists() && !*force {
                    return Err(format!(
                        "{} existe déjà — utilisez --force pour écraser",
                        output.display()
                    )
                    .into());
                }
                fs::write(output, DEFAULT_AEGIS_YAML)?;
                let msg = format!("Créé : {}", output.display());
                if color {
                    println!("{}", msg.green());
                } else {
                    println!("{msg}");
                }
                Ok(0)
            }
            ConfigCmd::Validate { path } => {
                let content = fs::read_to_string(path)?;
                let mut b = AnalyzerEngineBuilder::new();
                let lower = path.to_string_lossy().to_lowercase();
                b = if lower.ends_with(".json") {
                    b.with_engine_json_str(&content)?
                } else {
                    b.with_engine_yaml_str(&content)?
                };
                b.build()?;
                let msg = format!("OK — configuration valide : {}", path.display());
                if color {
                    println!("{}", msg.green());
                } else {
                    println!("{msg}");
                }
                Ok(0)
            }
        },

        Commands::Benchmark { path, iterations } => {
            let eng = scan_engine(&cli.global.language, config_ref)?;
            let text = fs::read_to_string(path)?;
            let cfg_b = cfg.clone();
            let n = (*iterations).max(1);
            let t0 = Instant::now();
            for _ in 0..n {
                let _ = eng.analyze(&text, Some(cfg_b.clone()))?;
            }
            let elapsed = t0.elapsed();
            let ms_avg = elapsed.as_secs_f64() * 1000.0 / f64::from(n);
            let summary = format!(
                "Benchmark {} — {} itérations en {:?} (~{ms_avg:.3} ms/it, {} caractères)",
                path.display(),
                n,
                elapsed,
                text.len()
            );
            if color {
                println!("{}", summary.cyan());
            } else {
                println!("{summary}");
            }
            Ok(0)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(1) => ExitCode::from(1),
        Ok(_) => ExitCode::from(3),
        Err(e) => {
            let _ = writeln!(io::stderr(), "aegis: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_languages_splits_trims_and_drops_empty() {
        assert_eq!(
            parse_languages("fr, en, de "),
            vec!["fr".to_string(), "en".to_string(), "de".to_string()]
        );
        assert!(parse_languages("  , , ").is_empty());
        assert_eq!(parse_languages("en"), vec!["en".to_string()]);
    }

    #[test]
    fn analysis_config_picks_first_language_and_threshold() {
        let g = GlobalOpts {
            language: "fr,en".into(),
            score_threshold: 0.72,
            format: OutputFormat::Json,
            config: None,
            no_color: true,
        };
        let c = analysis_config(&g);
        assert_eq!(c.language.as_deref(), Some("fr"));
        assert!((c.score_threshold - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn file_uri_for_nonexistent_still_prefixes_file_scheme() {
        let p = Path::new("no_such_file_for_cli_test.txt");
        let u = file_uri_for(p);
        assert!(u.starts_with("file://"));
        assert!(u.contains("no_such_file_for_cli_test"));
    }
}
