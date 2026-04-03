// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Anonymisation : un appel par famille d’opérateur (texte + entités fixes).

use aegis_anonymize::{AnonymizationConfig, AnonymizerEngine};
use aegis_core::anonymizer::{OperatorConfig, OperatorType};
use aegis_core::entity::{Entity, EntityType};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

const SAMPLE: &str = "Email alice@corp.test and phone +33 6 11 22 33 44 then card 4532015112830366";

fn sample_entities() -> Vec<Entity> {
    vec![
        Entity {
            entity_type: EntityType::Email,
            start: 6,
            end: 21,
            text: "alice@corp.test".into(),
            score: 0.9,
            recognizer_name: "b".into(),
            metadata: HashMap::new(),
        },
        Entity {
            entity_type: EntityType::Phone,
            start: 32,
            end: 51,
            text: "+33 6 11 22 33 44".into(),
            score: 0.85,
            recognizer_name: "b".into(),
            metadata: HashMap::new(),
        },
        Entity {
            entity_type: EntityType::CreditCard,
            start: 62,
            end: 78,
            text: "4532015112830366".into(),
            score: 0.88,
            recognizer_name: "b".into(),
            metadata: HashMap::new(),
        },
    ]
}

fn op_config(t: OperatorType, extra: &[(&str, &str)]) -> OperatorConfig {
    let mut params: HashMap<String, String> = extra
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    if matches!(t, OperatorType::Encrypt | OperatorType::Fpe) {
        params.entry("key_hex".into()).or_insert_with(|| {
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()
        });
        params
            .entry("key_id".into())
            .or_insert_with(|| "bench".into());
    }
    OperatorConfig {
        operator_type: t,
        params,
    }
}

fn bench_operators(c: &mut Criterion) {
    let engine = AnonymizerEngine::new();
    let ents = sample_entities();
    let cases: &[(&str, OperatorType, &[(&str, &str)])] = &[
        ("redact", OperatorType::Redact, &[]),
        ("replace", OperatorType::Replace, &[("tag", "PII")]),
        ("mask", OperatorType::Mask, &[]),
        ("hash", OperatorType::Hash, &[]),
        ("encrypt", OperatorType::Encrypt, &[("aad", "bench")]),
        ("fpe", OperatorType::Fpe, &[]),
        (
            "pseudonymize",
            OperatorType::Pseudonymize,
            &[("salt", "bench-salt")],
        ),
    ];

    let mut g = c.benchmark_group("anonymize_operator");
    for (label, ot, extra) in cases.iter() {
        let mut cfg = AnonymizationConfig::default();
        cfg.default_operator = Some(op_config((*ot).clone(), *extra));
        g.bench_with_input(BenchmarkId::new("anonymize", *label), &cfg, |b, cfg| {
            b.iter(|| {
                black_box(engine.anonymize(
                    black_box(SAMPLE),
                    black_box(ents.as_slice()),
                    black_box(cfg),
                ));
            });
        });
    }
    g.finish();
}

criterion_group!(benches, bench_operators);
criterion_main!(benches);
