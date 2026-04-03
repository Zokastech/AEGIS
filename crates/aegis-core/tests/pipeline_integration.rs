// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::pipeline::{
    ContextScorer, DetectionPipeline, MockNerBackend, NerBackend, PipelineConfig, PipelineLevels,
};
use aegis_regex::default_regex_recognizers;
use std::sync::Arc;

#[test]
fn pipeline_l1_finds_email() {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1Only;
    cfg.analysis.score_threshold = 0.3;
    let p = DetectionPipeline::new(
        cfg,
        default_regex_recognizers(&["fr"]),
        ContextScorer::default_eu(),
        None,
    );
    let out = p.analyze("mail: x@y.co ok").unwrap();
    assert!(
        out.entities
            .iter()
            .any(|e| e.entity_type == aegis_core::EntityType::Email),
        "{:?}",
        out.entities
    );
}

#[test]
fn pipeline_l2_runs_on_text() {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1L2;
    cfg.analysis.score_threshold = 0.2;
    cfg.analysis.language = Some("fr".into());
    let p = DetectionPipeline::new(
        cfg,
        default_regex_recognizers(&["fr"]),
        ContextScorer::default_eu(),
        None,
    );
    let out = p
        .analyze("patient Jean Dupont né à Paris email a@b.co")
        .unwrap();
    assert!(
        out.entities
            .iter()
            .any(|e| e.entity_type == aegis_core::EntityType::Email),
        "{:?}",
        out.entities
    );
}

#[test]
fn pipeline_trace_when_enabled() {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1Only;
    cfg.record_decision_trace = true;
    cfg.analysis.score_threshold = 0.3;
    let p = DetectionPipeline::new(
        cfg,
        default_regex_recognizers(&["en"]),
        ContextScorer::default_eu(),
        None,
    );
    let out = p.analyze_detailed("a@b.co").unwrap();
    assert!(out.trace.is_some());
}

#[test]
fn mock_ner_batch() {
    let ner: Arc<dyn NerBackend> = Arc::new(MockNerBackend::default());
    let v = ner.analyze_batch(&["hello", "world"], None).unwrap();
    assert_eq!(v.len(), 2);
}
