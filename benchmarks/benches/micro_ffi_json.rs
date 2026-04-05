// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Surcoût FFI (`aegis_analyze`) vs appel Rust direct ; sérialisation JSON des résultats.

use aegis_benchmarks::corpus_n_bytes;
use aegis_core::engine::{AnalyzerEngineBuilder, PipelineLevel};
use aegis_ffi::{aegis_analyze, aegis_free, aegis_free_string, aegis_init};
use aegis_regex as _;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::to_string;
use std::ffi::{CStr, CString};
use std::ptr;

fn build_engine() -> aegis_core::engine::AnalyzerEngine {
    AnalyzerEngineBuilder::new()
        .with_default_recognizers(&["en", "fr"])
        .with_pipeline_level(PipelineLevel::Two)
        .build()
        .expect("engine")
}

fn bench_ffi_vs_native(c: &mut Criterion) {
    let text = corpus_n_bytes(1024);
    let rust_eng = build_engine();

    let mut g = c.benchmark_group("analyze_1kb_rust_vs_ffi");
    g.bench_function("rust_native_analyze", |b| {
        b.iter(|| {
            black_box(rust_eng.analyze(black_box(text.as_str()), None).unwrap());
        });
    });

    unsafe {
        let handle = aegis_init(ptr::null());
        assert!(!handle.is_null(), "aegis_init");
        let cfg_json = CString::new("{}").unwrap();
        let text_c = CString::new(text.as_str()).unwrap();
        g.bench_function("ffi_aegis_analyze", |b| {
            b.iter(|| {
                let p = aegis_analyze(handle, text_c.as_ptr(), cfg_json.as_ptr());
                assert!(!p.is_null(), "{}", err_msg());
                aegis_free_string(p);
            });
        });
        aegis_free(handle);
    }
    g.finish();
}

unsafe fn err_msg() -> String {
    let p = aegis_ffi::aegis_last_error();
    if p.is_null() {
        return String::new();
    }
    CStr::from_ptr(p).to_string_lossy().into_owned()
}

fn bench_json_serialize(c: &mut Criterion) {
    let text = corpus_n_bytes(4096);
    let eng = build_engine();
    let res = eng.analyze(&text, None).unwrap();
    c.bench_function("serde_json_to_string_analysis_result", |b| {
        b.iter(|| {
            black_box(to_string(black_box(&res)).unwrap());
        });
    });
}

criterion_group!(benches, bench_ffi_vs_native, bench_json_serialize);
criterion_main!(benches);
