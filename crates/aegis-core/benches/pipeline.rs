// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::pipeline::{
    ContextScorer, DetectionPipeline, MockNerBackend, NerBackend, PipelineConfig, PipelineLevels,
};
use aegis_core::Result;
use aegis_regex::default_regex_recognizers;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

fn repeat_block(target: usize) -> String {
    let block = "Contact: alice@company.co.uk tel +33 6 12 34 56 78 \
        card 4532015112830366 ip 192.168.0.1 url https://zokastech.fr/x \
        date 2024-06-01 patient M. Dupont à Paris. \n";
    let mut s = String::with_capacity(target);
    while s.len() < target {
        s.push_str(block);
    }
    s.truncate(target);
    s
}

fn pipeline_l1(l1: Vec<Arc<dyn aegis_core::Recognizer>>) -> DetectionPipeline {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1Only;
    cfg.record_decision_trace = false;
    cfg.analysis.score_threshold = 0.35;
    DetectionPipeline::new(cfg, l1, ContextScorer::default_eu(), None)
}

fn pipeline_l1_l2(l1: Vec<Arc<dyn aegis_core::Recognizer>>) -> DetectionPipeline {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1L2;
    cfg.record_decision_trace = false;
    cfg.analysis.score_threshold = 0.35;
    DetectionPipeline::new(cfg, l1, ContextScorer::default_eu(), None)
}

fn pipeline_full(
    l1: Vec<Arc<dyn aegis_core::Recognizer>>,
    ner: Arc<dyn NerBackend>,
) -> DetectionPipeline {
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1L2L3;
    cfg.record_decision_trace = false;
    cfg.analysis.score_threshold = 0.35;
    cfg.ner_invocation_score_threshold = 0.99;
    DetectionPipeline::new(cfg, l1, ContextScorer::default_eu(), Some(ner))
}

fn bench_sizes(c: &mut Criterion) {
    let l1 = default_regex_recognizers(&[]);
    let sizes = [1024usize, 10_240, 102_400, 1_048_576];

    let mut g = c.benchmark_group("pipeline_n1_only");
    for sz in sizes {
        let text = repeat_block(sz);
        let p = pipeline_l1(l1.clone());
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(p.analyze(black_box(t.as_str())).unwrap());
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("pipeline_n1_n2");
    for sz in sizes {
        let text = repeat_block(sz);
        let p = pipeline_l1_l2(l1.clone());
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(p.analyze(black_box(t.as_str())).unwrap());
            });
        });
    }
    g.finish();

    let ner: Arc<dyn NerBackend> = Arc::new(MockNerBackend {
        canned: vec![aegis_core::Entity {
            entity_type: aegis_core::EntityType::Person,
            start: 0,
            end: 5,
            text: "alice".into(),
            score: 0.82,
            recognizer_name: "mock".into(),
            metadata: Default::default(),
            decision_trace: None,
        }],
    });
    let mut g = c.benchmark_group("pipeline_full_mock_ner");
    for sz in sizes {
        let text = repeat_block(sz);
        let p = pipeline_full(l1.clone(), ner.clone());
        g.bench_with_input(BenchmarkId::from_parameter(sz), &text, |b, t| {
            b.iter(|| {
                black_box(p.analyze(black_box(t.as_str())).unwrap());
            });
        });
    }
    g.finish();
}

fn bench_batch_100x10kb(c: &mut Criterion) {
    let l1 = default_regex_recognizers(&[]);
    let mut cfg = PipelineConfig::default();
    cfg.levels = PipelineLevels::L1L2;
    cfg.analysis.score_threshold = 0.35;
    let p = DetectionPipeline::new(cfg, l1, ContextScorer::default_eu(), None);
    let chunk = repeat_block(10_240);
    let batch: Vec<String> = (0..100).map(|_| chunk.clone()).collect();
    let refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();

    c.bench_function("pipeline_batch_100_docs_10kb_l1l2", |b| {
        b.iter(|| {
            let v: Result<Vec<_>> = p.analyze_batch(black_box(&refs));
            black_box(v.unwrap().len());
        });
    });
}

criterion_group!(benches, bench_sizes, bench_batch_100x10kb);
criterion_main!(benches);
