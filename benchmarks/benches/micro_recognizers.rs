// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Chaque recognizer regex par défaut, seul, sur un corpus de 1 KiB.

use aegis_benchmarks::corpus_n_bytes;
use aegis_core::config::AnalysisConfig;
use aegis_core::recognizer::Recognizer;
use aegis_regex::default_regex_recognizers;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_each_recognizer_1kb(c: &mut Criterion) {
    let text = corpus_n_bytes(1024);
    let cfg = AnalysisConfig {
        score_threshold: 0.35,
        ..AnalysisConfig::default()
    };
    let mut g = c.benchmark_group("recognizer_1kb_each");
    for r in default_regex_recognizers(&[]) {
        let name = r.name().to_string();
        g.bench_with_input(BenchmarkId::new("analyze", &name), &name, |b, _| {
            b.iter(|| {
                black_box(r.analyze(black_box(text.as_str()), black_box(&cfg)));
            });
        });
    }
    g.finish();
}

criterion_group!(benches, bench_each_recognizer_1kb);
criterion_main!(benches);
