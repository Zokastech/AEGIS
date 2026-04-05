// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::config::AnalysisConfig;
use aegis_regex::default_regex_recognizers;
use aegis_regex::Recognizer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

fn bench_all_recognizers_10kb(c: &mut Criterion) {
    let text = sample_10kb();
    let recs = default_regex_recognizers(&[]);
    let cfg = AnalysisConfig::default();
    c.bench_function("regex_all_recognizers_10kb", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for r in &recs {
                total = total.wrapping_add(black_box(r.analyze(black_box(text.as_str()), &cfg)).len());
            }
            black_box(total)
        });
    });
}

criterion_group!(benches, bench_all_recognizers_10kb);
criterion_main!(benches);
