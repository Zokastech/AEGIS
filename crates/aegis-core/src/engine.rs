// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! [`AnalyzerEngine`] — main entry point (Presidio-style three-level pipeline).

use crate::config::{AegisConfig, AegisEngineConfig, AnalysisConfig};
use crate::entity::{
    AnalysisResult, Entity, EntityType, JsonEntityDecisionTrace, JsonTraceStep,
};
use crate::error::{AegisError, Result};
use crate::context::ContextScorer;
use crate::pipeline::{
    DecisionTrace, DetectionPipeline, MockNerBackend, NerBackend, PipelineConfig, PipelineLevels,
    TraceStep,
};
use crate::recognizer::Recognizer;
use crate::registry::RecognizerRegistry;
use metrics::{counter, histogram};
use rayon::ThreadPoolBuilder;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Function type registered by `aegis-regex` at load time (avoids a dependency cycle).
pub type DefaultRegexLoaderFn = fn(&[&str]) -> Vec<Arc<dyn Recognizer>>;

static DEFAULT_REGEX_LOADER: RwLock<Option<DefaultRegexLoaderFn>> = RwLock::new(None);

/// Registers the default regex recognizer loader (called from `aegis-regex` via `ctor`).
pub fn register_default_regex_loader(f: DefaultRegexLoaderFn) {
    let mut g = DEFAULT_REGEX_LOADER
        .write()
        .unwrap_or_else(|e| e.into_inner());
    *g = Some(f);
}

fn try_default_regex_loaders(langs: &[&str]) -> Option<Vec<Arc<dyn Recognizer>>> {
    let g = DEFAULT_REGEX_LOADER.read().ok()?;
    g.as_ref().map(|f| f(langs))
}

/// Pipeline level (1 = regex, 2 = +context, 3 = +NER).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineLevel {
    One = 1,
    Two = 2,
    Three = 3,
}

impl From<PipelineLevel> for PipelineLevels {
    fn from(v: PipelineLevel) -> Self {
        match v {
            PipelineLevel::One => PipelineLevels::L1Only,
            PipelineLevel::Two => PipelineLevels::L1L2,
            PipelineLevel::Three => PipelineLevels::L1L2L3,
        }
    }
}

/// **AEGIS** engine entry: registry + three-level pipeline + metrics.
pub struct AnalyzerEngine {
    registry: RecognizerRegistry,
    pipeline_config: PipelineConfig,
    context: ContextScorer,
    default_analysis: AnalysisConfig,
    aegis: Option<AegisConfig>,
    ner_model_path: Option<String>,
    ner_lazy: Mutex<Option<Arc<dyn NerBackend>>>,
    ner_pool: Option<Arc<rayon::ThreadPool>>,
    entity_thresholds: HashMap<EntityType, f64>,
    disabled_recognizers: HashSet<String>,
}

impl AnalyzerEngine {
    /// Compatibility default: L1+L2+L3 without a NER model until a model path is set.
    pub fn new(registry: RecognizerRegistry) -> Self {
        Self {
            registry,
            pipeline_config: PipelineConfig::default(),
            context: ContextScorer::default_eu(),
            default_analysis: AnalysisConfig::default(),
            aegis: None,
            ner_model_path: None,
            ner_lazy: Mutex::new(None),
            ner_pool: None,
            entity_thresholds: HashMap::new(),
            disabled_recognizers: HashSet::new(),
        }
    }

    pub fn from_aegis_config(cfg: &AegisConfig, registry: RecognizerRegistry) -> Self {
        let mut e = Self::new(registry);
        e.default_analysis = cfg.analysis.clone();
        e.aegis = Some(cfg.clone());
        e
    }

    pub fn with_analysis_config(mut self, c: AnalysisConfig) -> Self {
        self.default_analysis = c;
        self
    }

    fn filter_recognizers(&self) -> Vec<Arc<dyn Recognizer>> {
        self.registry
            .all()
            .into_iter()
            .filter(|r| {
                !self
                    .disabled_recognizers
                    .contains(&r.name().to_lowercase())
            })
            .collect()
    }

    fn ner_backend_for_levels(&self, levels: PipelineLevels) -> Result<Option<Arc<dyn NerBackend>>> {
        if !levels.l3() {
            return Ok(None);
        }
        let path = match &self.ner_model_path {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        let mut g = self
            .ner_lazy
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if g.is_none() {
            *g = Some(load_ner_lazy(path)?);
        }
        Ok(g.clone())
    }

    fn make_pipeline(&self, analysis: AnalysisConfig) -> Result<DetectionPipeline> {
        let mut pc = self.pipeline_config.clone();
        if let Some(lvl) = analysis.pipeline_level {
            pc.levels = match lvl {
                1 => PipelineLevels::L1Only,
                2 => PipelineLevels::L1L2,
                _ => PipelineLevels::L1L2L3,
            };
        }
        if analysis.return_decision_process {
            pc.record_decision_trace = true;
        }
        pc.analysis = analysis;
        let ner = self.ner_backend_for_levels(pc.levels)?;
        Ok(
            DetectionPipeline::new(pc, self.filter_recognizers(), self.context.clone(), ner)
                .with_ner_pool(self.ner_pool.clone()),
        )
    }

    fn spans_overlap(a: (usize, usize), b: (usize, usize)) -> bool {
        a.0 < b.1 && a.1 > b.0
    }

    fn trace_step_detail(step: &TraceStep) -> Option<String> {
        if let Some(n) = step.note.as_deref() {
            let t = n.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        if !step.context_word_hits.is_empty() {
            let s = step.context_word_hits.join(", ");
            if !s.is_empty() {
                return Some(s);
            }
        }
        None
    }

    /// Attaches L1/L2/L3 steps to final entities (overlapping spans + same type).
    fn attach_pipeline_traces(entities: &mut [Entity], trace: &DecisionTrace) {
        for ent in entities.iter_mut() {
            let mut steps_out: Vec<JsonTraceStep> = Vec::new();
            let mut max_lvl: u8 = 0;
            for st in &trace.steps {
                if st.entity_type != ent.entity_type {
                    continue;
                }
                if !Self::spans_overlap((st.span_start, st.span_end), (ent.start, ent.end)) {
                    continue;
                }
                max_lvl = max_lvl.max(st.level);
                let passed = st.action != "skipped_short_circuit";
                steps_out.push(JsonTraceStep {
                    name: format!("L{}:{}", st.level, st.action),
                    passed,
                    detail: Self::trace_step_detail(st),
                });
            }
            if !steps_out.is_empty() {
                ent.decision_trace = Some(JsonEntityDecisionTrace {
                    steps: steps_out,
                    pipeline_level: Some(max_lvl),
                });
            }
        }
    }

    pub fn get_recognizers(&self) -> &RecognizerRegistry {
        &self.registry
    }

    pub fn get_supported_entities(&self) -> Vec<EntityType> {
        let mut set = HashSet::new();
        for r in self.filter_recognizers() {
            for e in r.supported_entities() {
                set.insert(e);
            }
        }
        set.into_iter().collect()
    }

    pub fn analyze(&self, text: &str, config: Option<AnalysisConfig>) -> Result<AnalysisResult> {
        let t0 = Instant::now();
        counter!("aegis_analyze_started_total").increment(1);
        let cfg = config.unwrap_or_else(|| self.default_analysis.clone());
        let pipeline = self.make_pipeline(cfg)?;
        let want_trace = pipeline.config().record_decision_trace;
        let mut out = if want_trace {
            match pipeline.analyze_detailed(text) {
                Ok(detailed) => {
                    let mut analysis = detailed.analysis;
                    if let Some(tr) = detailed.trace {
                        Self::attach_pipeline_traces(&mut analysis.entities, &tr);
                    }
                    analysis
                }
                Err(e) => {
                    counter!("aegis_analyze_errors_total").increment(1);
                    return Err(e);
                }
            }
        } else {
            match pipeline.analyze(text) {
                Ok(o) => o,
                Err(e) => {
                    counter!("aegis_analyze_errors_total").increment(1);
                    return Err(e);
                }
            }
        };
        self.apply_entity_thresholds(&mut out.entities);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        histogram!("aegis_analyze_latency_ms").record(ms);
        counter!("aegis_analyze_ok_total").increment(1);
        counter!("aegis_detections_total").increment(out.entities.len() as u64);
        Ok(out)
    }

    pub fn analyze_batch(
        &self,
        texts: &[&str],
        config: Option<AnalysisConfig>,
    ) -> Result<Vec<AnalysisResult>> {
        texts
            .iter()
            .map(|t| self.analyze(t, config.clone()))
            .collect()
    }

    fn apply_entity_thresholds(&self, entities: &mut Vec<Entity>) {
        if self.entity_thresholds.is_empty() {
            return;
        }
        entities.retain(|e| {
            self.entity_thresholds
                .get(&e.entity_type)
                .map(|th| e.score >= *th)
                .unwrap_or(true)
        });
    }

    pub fn pipeline_config(&self) -> &PipelineConfig {
        &self.pipeline_config
    }

    pub fn aegis_config(&self) -> Option<&AegisConfig> {
        self.aegis.as_ref()
    }
}

fn load_ner_lazy(path: &str) -> Result<Arc<dyn NerBackend>> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AegisError::ModelError(format!(
            "chemin modèle NER introuvable: {path}"
        )));
    }
    tracing::info!(target: "aegis_core::engine", path = %path, "chargement NER paresseux (stub ONNX — brancher aegis-ner)");
    Ok(Arc::new(MockNerBackend::default()) as Arc<dyn NerBackend>)
}

/// Fluent builder for [`AnalyzerEngine`].
pub struct AnalyzerEngineBuilder {
    registry: RecognizerRegistry,
    pipeline_config: PipelineConfig,
    context: ContextScorer,
    default_analysis: AnalysisConfig,
    aegis: Option<AegisConfig>,
    ner_model_path: Option<String>,
    ner_thread_pool_size: Option<usize>,
    entity_thresholds: HashMap<EntityType, f64>,
    disabled_recognizers: HashSet<String>,
    pending_default_regex_langs: Option<Vec<String>>,
}

impl AnalyzerEngineBuilder {
    pub fn new() -> Self {
        Self {
            registry: RecognizerRegistry::new(),
            pipeline_config: PipelineConfig::default(),
            context: ContextScorer::default_eu(),
            default_analysis: AnalysisConfig::default(),
            aegis: None,
            ner_model_path: None,
            ner_thread_pool_size: None,
            entity_thresholds: HashMap::new(),
            disabled_recognizers: HashSet::new(),
            pending_default_regex_langs: None,
        }
    }

    /// Loads default regex recognizers when `aegis-regex` is linked (`ctor` registration).
    pub fn with_default_recognizers(mut self, languages: &[&str]) -> Self {
        self.pending_default_regex_langs = Some(languages.iter().map(|s| (*s).to_string()).collect());
        self
    }

    pub fn with_recognizer(self, r: Arc<dyn Recognizer>) -> Self {
        self.registry.add(r);
        self
    }

    pub fn with_recognizer_box(self, r: Box<dyn Recognizer>) -> Self {
        self.with_recognizer(Arc::from(r))
    }

    pub fn with_ner_model(mut self, model_path: &str) -> Self {
        self.ner_model_path = Some(model_path.to_string());
        self
    }

    pub fn with_config(mut self, config: AegisConfig) -> Self {
        self.default_analysis = config.analysis.clone();
        self.aegis = Some(config);
        self
    }

    pub fn with_pipeline_level(mut self, level: PipelineLevel) -> Self {
        self.pipeline_config.levels = level.into();
        self
    }

    /// Applies an already-deserialized engine config (YAML or JSON).
    pub fn merge_engine_config(&mut self, ec: &AegisEngineConfig) -> Result<()> {
        self.disabled_recognizers = ec.disabled_set();
        self.entity_thresholds = ec.entity_threshold_map()?;
        self.pipeline_config = ec.merged_pipeline_config();
        if let Some(ref cs) = ec.context_scorer {
            self.context = ContextScorer::new(cs.clone());
        }
        if let Some(ref n) = ec.ner {
            if let Some(ref p) = n.model_path {
                self.ner_model_path = Some(p.clone());
            }
            self.ner_thread_pool_size = n.thread_pool_size;
        }
        if let Some(a) = ec.analysis.clone() {
            self.default_analysis = a;
        }
        if ec.recognizers.default_regex.enabled {
            let langs: Vec<&str> = ec
                .recognizers
                .default_regex
                .languages
                .iter()
                .map(|s| s.as_str())
                .collect();
            self.pending_default_regex_langs = Some(langs.iter().map(|s| (*s).to_string()).collect());
        }
        Ok(())
    }

    /// Merges engine YAML config (`aegis-config.yaml`).
    pub fn with_engine_yaml_str(mut self, yaml: &str) -> Result<Self> {
        let ec = AegisEngineConfig::from_yaml_str(yaml)?;
        self.merge_engine_config(&ec)?;
        Ok(self)
    }

    /// Merges engine JSON config (same schema as YAML).
    pub fn with_engine_json_str(mut self, json: &str) -> Result<Self> {
        let ec = AegisEngineConfig::from_json_str(json)?;
        self.merge_engine_config(&ec)?;
        Ok(self)
    }

    pub fn build(self) -> Result<AnalyzerEngine> {
        if let Some(ref langs) = self.pending_default_regex_langs {
            let sl: Vec<&str> = langs.iter().map(|s| s.as_str()).collect();
            if let Some(v) = try_default_regex_loaders(&sl) {
                for r in v {
                    self.registry.add(r);
                }
            } else {
                tracing::warn!(
                    target: "aegis_core::engine",
                    "with_default_recognizers: aegis-regex non enregistré — liez le crate `aegis-regex`"
                );
            }
        }
        let registry = self.registry;
        if registry.all().is_empty() {
            return Err(AegisError::ConfigError(
                "aucun recognizer enregistré".into(),
            ));
        }
        let ner_pool = self
            .ner_thread_pool_size
            .filter(|&n| n > 0)
            .and_then(|n| ThreadPoolBuilder::new().num_threads(n).build().ok())
            .map(Arc::new);
        Ok(AnalyzerEngine {
            registry,
            pipeline_config: self.pipeline_config,
            context: self.context,
            default_analysis: self.default_analysis,
            aegis: self.aegis,
            ner_model_path: self.ner_model_path,
            ner_lazy: Mutex::new(None),
            ner_pool,
            entity_thresholds: self.entity_thresholds,
            disabled_recognizers: self.disabled_recognizers,
        })
    }
}

impl Default for AnalyzerEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
