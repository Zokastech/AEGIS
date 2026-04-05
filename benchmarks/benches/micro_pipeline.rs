// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Pipelines : L1, L1+L2 via [`AnalyzerEngine`], L1+L2+L3 avec NER mock via [`DetectionPipeline`].

use aegis_benchmarks::{corpus_n_bytes, SIZES};
use aegis_core::engine::{AnalyzerEngineBuilder, PipelineLevel};
use aegis_core::entity::{Entity, EntityType};
use aegis_core::pipeline::{
    ContextScorer, DetectionPipeline, MockNerBackend, NerBackend, PipelineConfig, PipelineLevels,
};
use aegis_regex as _;
use aegis_regex::default_regex_recognizers;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

fn engine_l1() -> aegis_core::engine::AnalyzerEngine {
    AnalyzerEngineBuilder::new()
        .with_default_recognizers(&["en", "fr"])
        .with_pipeline_level(PipelineLevel::One)
        .build()
        .expect("l1 engine")
}

fn engine_l12() -> aegis_core::engine::AnalyzerEngine {
    AnalyzerEngineBuilder::new()
        .with_default_recognizers(&["en", "fr"])
        .with_pipeline_level(PipelineLevel::Two)
        .build()
        .expect("l12 engine")
}

fn pipeline_l123_mock() -> DetectionPipeline {
    let l1 = default_regex_recognizers(&[]);
    let ner: Arc<dyn NerBackend> = Arc::new(MockNerBackend {
        canned: vec![Entity {
            entity_type: EntityType::Person,
            start: 0,
            end: 5,
            text: "alice".into(),
            score: 0.82,
            recognizer_name: "mock".into(),
            metadata: Default::default(),
            decision_trace: None,
        }],
    });
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1L2L3;
    cfg.record_decision_trace = false;
    cfg.analysis.score_threshold = 0.35;
    cfg.ner_invocation_score_threshold = 0.99;
    DetectionPipeline::new(cfg, l1, ContextScorer::default_eu(), Some(ner))
}

fn bench_engine_l1(c: &mut Criterion) {
    let eng = engine_l1();
    let mut g = c.benchmark_group("pipeline_engine_level1");
    for &sz in &SIZES {
        let text = corpus_n_bytes(sz);
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(eng.analyze(black_box(t.as_str()), None).unwrap());
            });
        });
    }
    g.finish();
}

fn bench_engine_l12(c: &mut Criterion) {
    let eng = engine_l12();
    let mut g = c.benchmark_group("pipeline_engine_level1_2");
    for &sz in &SIZES {
        let text = corpus_n_bytes(sz);
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(eng.analyze(black_box(t.as_str()), None).unwrap());
            });
        });
    }
    g.finish();
}

fn bench_pipeline_l123(c: &mut Criterion) {
    let p = pipeline_l123_mock();
    let mut g = c.benchmark_group("pipeline_level1_2_3_mock_ner");
    for &sz in &SIZES {
        let text = corpus_n_bytes(sz);
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(p.analyze(black_box(t.as_str())).unwrap());
            });
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_engine_l1,
    bench_engine_l12,
    bench_pipeline_l123
);
criterion_main!(benches);
