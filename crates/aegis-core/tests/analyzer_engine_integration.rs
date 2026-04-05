// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::config::AegisEngineConfig;
use aegis_core::engine::{AnalyzerEngineBuilder, PipelineLevel};
use aegis_core::entity::EntityType;
use aegis_regex::pattern::PatternRecognizer;
use regex::Regex;
use std::sync::Arc;

fn iban_stub() -> PatternRecognizer {
    let re = Regex::new(r"\b[A-Z]{2}\d{2}[A-Z0-9]{10,30}\b").unwrap();
    PatternRecognizer::new("iban_stub", re, EntityType::Iban, vec!["en", "fr"], 0.85)
        .with_min_score(0.4)
}

fn ssn_stub() -> PatternRecognizer {
    let re = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    PatternRecognizer::new("ssn_stub", re, EntityType::Ssn, vec!["en"], 0.8).with_min_score(0.4)
}

fn plate_stub() -> PatternRecognizer {
    let re = Regex::new(r"\b[A-Z]{2}-\d{3}-[A-Z]{2}\b").unwrap();
    PatternRecognizer::new("plate_fr", re, EntityType::VehiclePlate, vec!["fr"], 0.7)
        .with_min_score(0.35)
}

#[test]
fn engine_detects_many_entity_types() {
    let eng = AnalyzerEngineBuilder::new()
        .with_default_recognizers(&["en", "fr", "de", "es", "it", "nl", "pt", "pl"])
        .with_recognizer(Arc::new(iban_stub()))
        .with_recognizer(Arc::new(ssn_stub()))
        .with_recognizer(Arc::new(plate_stub()))
        .with_pipeline_level(PipelineLevel::Two)
        .build()
        .expect("engine");

    let text = r#"
        Contact: alice@company.co.uk +33 6 12 34 56 78 card 4532015112830366
        ip 192.168.1.1 see https://zokastech.fr/x date 2024-12-01
        btc bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq eth 0x742d35Cc6634C0532925a3b844Bc454e4438f44e
        IBAN FR7630006000011234567890189 SSN 078-05-1120 plate AB-123-CD
    "#;

    let mut cfg = aegis_core::config::AnalysisConfig::default();
    cfg.score_threshold = 0.25;
    let res = eng.analyze(text, Some(cfg)).expect("analyze");

    let types: std::collections::HashSet<_> =
        res.entities.iter().map(|e| e.entity_type.clone()).collect();

    for expected in [
        EntityType::Email,
        EntityType::Phone,
        EntityType::CreditCard,
        EntityType::IpAddress,
        EntityType::Url,
        EntityType::Date,
        EntityType::CryptoWallet,
        EntityType::Iban,
        EntityType::Ssn,
        EntityType::VehiclePlate,
    ] {
        assert!(
            types.contains(&expected),
            "missing {:?}, have {:?}",
            expected,
            types
        );
    }
}

#[test]
fn concurrent_analyze_100() {
    let eng = Arc::new(
        AnalyzerEngineBuilder::new()
            .with_default_recognizers(&["en", "fr"])
            .with_pipeline_level(PipelineLevel::One)
            .build()
            .unwrap(),
    );
    let text = "email x@y.co please";
    std::thread::scope(|s| {
        for _ in 0..100 {
            let e = eng.clone();
            s.spawn(move || {
                let _ = e.analyze(text, None).unwrap();
            });
        }
    });
}

#[test]
fn yaml_load_modify_reload() {
    let yaml1 = r#"
pipeline_level: 1
recognizers:
  default_regex:
    enabled: true
    languages: [en]
analysis:
  score_threshold: 0.4
"#;
    let mut cfg = AegisEngineConfig::from_yaml_str(yaml1).unwrap();
    assert_eq!(cfg.pipeline_level, Some(1));

    cfg.pipeline_level = Some(2);
    cfg.analysis = Some(aegis_core::config::AnalysisConfig {
        score_threshold: 0.55,
        ..Default::default()
    });
    let dumped = serde_json::to_string(&cfg).unwrap();
    let cfg2: AegisEngineConfig = serde_json::from_str(&dumped).unwrap();
    assert_eq!(cfg2.pipeline_level, Some(2));
    assert_eq!(cfg2.analysis.as_ref().unwrap().score_threshold, 0.55);
}

/// `pipeline_level` dans le JSON d’analyse (Playground / API) doit surcharger le niveau YAML du moteur.
#[test]
fn per_request_pipeline_level_two_runs_context_trace_not_level_one() {
    let eng = AnalyzerEngineBuilder::new()
        .with_default_recognizers(&["fr"])
        .with_pipeline_level(PipelineLevel::Three)
        .build()
        .expect("engine");

    let text = "Virement IBAN FR7630006000011234567890189 effectué.";

    let mut ac_l2 = aegis_core::config::AnalysisConfig::default();
    ac_l2.pipeline_level = Some(2);
    ac_l2.return_decision_process = true;
    ac_l2.score_threshold = 0.2;
    ac_l2.language = Some("fr".into());

    let iban_l2 = eng
        .analyze(text, Some(ac_l2.clone()))
        .expect("l2")
        .entities
        .into_iter()
        .find(|e| e.entity_type == EntityType::Iban)
        .expect("iban l2");
    let steps_l2 = iban_l2
        .decision_trace
        .as_ref()
        .map(|t| t.steps.as_slice())
        .unwrap_or(&[]);
    assert!(
        steps_l2.iter().any(|s| s.name.contains("L2:")),
        "avec pipeline_level=2 on attend des étapes L2 dans la trace, obtenu {:?}",
        steps_l2
    );

    let mut ac_l1 = ac_l2;
    ac_l1.pipeline_level = Some(1);

    let iban_l1 = eng
        .analyze(text, Some(ac_l1))
        .expect("l1")
        .entities
        .into_iter()
        .find(|e| e.entity_type == EntityType::Iban)
        .expect("iban l1");
    let steps_l1 = iban_l1
        .decision_trace
        .as_ref()
        .map(|t| t.steps.as_slice())
        .unwrap_or(&[]);
    assert!(
        !steps_l1.iter().any(|s| s.name.contains("L2:")),
        "avec pipeline_level=1 il ne doit pas y avoir d’étapes L2, obtenu {:?}",
        steps_l1
    );
}
