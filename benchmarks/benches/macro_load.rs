// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Macro : débit (docs/s estimé), 100 analyses concurrentes, montée en charge 1–16 threads.

use aegis_benchmarks::corpus_n_bytes;
use aegis_core::engine::{AnalyzerEngineBuilder, PipelineLevel};
use aegis_regex as _;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::{Arc, Barrier};

fn engine_l12() -> Arc<aegis_core::engine::AnalyzerEngine> {
    Arc::new(
        AnalyzerEngineBuilder::new()
            .with_default_recognizers(&["en", "fr"])
            .with_pipeline_level(PipelineLevel::Two)
            .build()
            .expect("engine"),
    )
}

/// Débit : traiter beaucoup de documents 1 KiB en série (métrique inverse du temps total).
fn bench_throughput_serial(c: &mut Criterion) {
    let eng = engine_l12();
    let doc = corpus_n_bytes(1024);
    const N_DOC: usize = 400;
    c.bench_function("macro_throughput_serial_400x1kb", |b| {
        b.iter(|| {
            for _ in 0..N_DOC {
                black_box(eng.analyze(black_box(doc.as_str()), None).unwrap());
            }
        });
    });
}

/// 100 threads : chacun une analyse 1 KiB après barrière (pic de contention).
fn bench_concurrent_100(c: &mut Criterion) {
    let eng = engine_l12();
    let doc = Arc::new(corpus_n_bytes(1024));
    const N: usize = 100;
    c.bench_function("macro_concurrent_100x_analyze_1kb_barrier", |b| {
        b.iter(|| {
            let barrier = Arc::new(Barrier::new(N));
            std::thread::scope(|s| {
                for _ in 0..N {
                    let b = barrier.clone();
                    let e = Arc::clone(&eng);
                    let t = Arc::clone(&doc);
                    s.spawn(move || {
                        b.wait();
                        black_box(e.analyze(t.as_str(), None).unwrap());
                    });
                }
            });
        });
    });
}

/// Répartition d’un budget fixe d’analyses sur N threads (scope).
fn bench_thread_scaling(c: &mut Criterion) {
    let eng = engine_l12();
    let doc = Arc::new(corpus_n_bytes(2048));
    const TOTAL: usize = 256;
    let mut g = c.benchmark_group("macro_thread_scaling_256_analyze_2kb");
    for n_threads in [1usize, 2, 4, 8, 16] {
        g.bench_with_input(
            BenchmarkId::from_parameter(n_threads),
            &n_threads,
            |b, &n_threads| {
                b.iter(|| {
                    let chunk = TOTAL / n_threads;
                    std::thread::scope(|s| {
                        for _ in 0..n_threads {
                            let e = Arc::clone(&eng);
                            let t = Arc::clone(&doc);
                            s.spawn(move || {
                                for _ in 0..chunk {
                                    black_box(e.analyze(t.as_str(), None).unwrap());
                                }
                            });
                        }
                    });
                });
            },
        );
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_throughput_serial,
    bench_concurrent_100,
    bench_thread_scaling
);
criterion_main!(benches);
