// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::config::AnalysisConfig;
use aegis_core::recognizer::Recognizer;
use aegis_regex::recognizers::financial::{
    bic_swift_recognizer, eu_credit_card_recognizer, eu_vat_recognizer, iban_recognizer,
    nir_recognizer, siren_recognizer, siret_recognizer,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn corpus_financial() -> String {
    r#"
    IBAN FR76 3000 6000 0112 3456 7890 189
    BIC BNPAFRPPXXX
    TVA FR29443169758 DE123456789 IT12345678901
    Carte Bleue 4973012345678908
    SWIFT DEUTDEFF500
    EL123456789 NL123456789B01
    "#
    .repeat(48)
}

fn bench_financial_recognizers(c: &mut Criterion) {
    let text = corpus_financial();
    let cfg = AnalysisConfig::default();
    let iban = iban_recognizer();
    let bic = bic_swift_recognizer();
    let vat = eu_vat_recognizer();
    let card = eu_credit_card_recognizer();
    let siren = siren_recognizer();
    let siret = siret_recognizer();
    let nir = nir_recognizer();

    c.bench_function("financial_iban", |b| {
        b.iter(|| black_box(iban.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_bic", |b| {
        b.iter(|| black_box(bic.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_vat", |b| {
        b.iter(|| black_box(vat.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_eu_card", |b| {
        b.iter(|| black_box(card.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_siren", |b| {
        b.iter(|| black_box(siren.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_siret", |b| {
        b.iter(|| black_box(siret.analyze(black_box(text.as_str()), &cfg)))
    });
    c.bench_function("financial_nir", |b| {
        b.iter(|| black_box(nir.analyze(black_box(text.as_str()), &cfg)))
    });
    let all: Vec<Box<dyn Recognizer>> = vec![
        Box::new(iban_recognizer()),
        Box::new(bic_swift_recognizer()),
        Box::new(eu_vat_recognizer()),
        Box::new(eu_credit_card_recognizer()),
        Box::new(siren_recognizer()),
        Box::new(siret_recognizer()),
        Box::new(nir_recognizer()),
    ];
    c.bench_function("financial_all_seven", |b| {
        b.iter(|| {
            let mut n = 0usize;
            for r in &all {
                n = n.wrapping_add(black_box(r.analyze(black_box(text.as_str()), &cfg)).len());
            }
            black_box(n)
        });
    });
}

criterion_group!(benches, bench_financial_recognizers);
criterion_main!(benches);
