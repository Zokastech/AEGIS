// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::config::AnalysisConfig;
use aegis_regex::default_regex_recognizers;
use aegis_regex::Recognizer;
use std::time::Instant;

fn sample_10kb() -> String {
    let mut s = String::with_capacity(10_240);
    let block = "Contact: alice@company.co.uk tel +33 6 12 34 56 78 \
        card 4532015112830366 ip 192.168.0.1 url https://zokastech.fr/a \
        date 2024-06-01 eth 0x742d35Cc6634C0532925a3b844Bc454e4438f44e \
        btc bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq \n";
    while s.len() < 10_240 {
        s.push_str(block);
    }
    s.truncate(10_240);
    s
}

/// Product goal: under 1 ms for 10 KiB with all recognizers (machine-dependent).
#[test]
#[ignore = "benchmark manuel : cargo test -p aegis-regex --test perf_smoke -- --ignored"]
fn all_regex_recognizers_10kb_under_1ms() {
    let text = sample_10kb();
    let recs = default_regex_recognizers(&[]);
    let cfg = AnalysisConfig::default();
    let t0 = Instant::now();
    let mut n = 0usize;
    for r in &recs {
        n += r.analyze(&text, &cfg).len();
    }
    let elapsed = t0.elapsed();
    assert!(n > 0, "expected some hits");
    assert!(
        elapsed.as_secs_f64() * 1000.0 < 1.0,
        "took {:?} (target < 1ms)",
        elapsed
    );
}
