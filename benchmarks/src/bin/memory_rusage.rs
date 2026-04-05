// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Estimation RSS max après charge (Unix `getrusage`). Usage :
//! `cargo run -p aegis-benchmarks --release --bin aegis-memory-rusage -- 2000`

fn main() {
    #[cfg(unix)]
    {
        use aegis_core::engine::{AnalyzerEngineBuilder, PipelineLevel};
        use aegis_regex as _;
        use libc::{getrusage, rusage, RUSAGE_SELF};
        use std::env;

        let n: usize = env::args()
            .nth(1)
            .and_then(|a| a.parse().ok())
            .unwrap_or(500);
        let text = aegis_benchmarks::corpus_n_bytes(10_240);
        let engine = AnalyzerEngineBuilder::new()
            .with_default_recognizers(&["en", "fr"])
            .with_pipeline_level(PipelineLevel::Two)
            .build()
            .expect("engine");

        let mut before: rusage = unsafe { std::mem::zeroed() };
        unsafe {
            getrusage(RUSAGE_SELF, &mut before);
        }
        for _ in 0..n {
            let _ = engine.analyze(&text, None).expect("analyze");
        }
        let mut after: rusage = unsafe { std::mem::zeroed() };
        unsafe {
            getrusage(RUSAGE_SELF, &mut after);
        }
        let rss = after.ru_maxrss;
        #[cfg(target_os = "macos")]
        let rss_kb = rss / 1024;
        #[cfg(not(target_os = "macos"))]
        let rss_kb = rss;
        println!("iterations={n} max_rss_kb≈{rss_kb} (plateforme: macOS=bytes/1024, Linux=ko)");
    }
    #[cfg(not(unix))]
    {
        println!("aegis-memory-rusage: Unix uniquement (getrusage).");
    }
}
