// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! **Synthetic** PII data (FR / DE / ES / IT) — no real individuals.

use aegis_anonymize::{AnonymizationConfig, AnonymizerEngine, HashOperator, RedactOperator};
use aegis_core::anonymizer::{Operator, OperatorConfig, OperatorType};
use aegis_core::entity::{Entity, EntityType};
use std::collections::HashMap;

/// 64 hex digits → 32 octets (AES-256).
const KEY32: &str = "abcdef00112233445566778899aabbccddeeff00112233445566778899aabb00";

fn synth_entities_fr(text: &str) -> Vec<Entity> {
    let name = "Claire Martin";
    let phone = "+33 6 12 34 56 78";
    let ns = text.find(name).expect("name");
    let ne = ns + name.len();
    let ps = text.find(phone).expect("phone");
    let pe = ps + phone.len();
    vec![
        Entity {
            entity_type: EntityType::Person,
            start: ns,
            end: ne,
            text: name.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        },
        Entity {
            entity_type: EntityType::Phone,
            start: ps,
            end: pe,
            text: phone.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        },
    ]
}

#[test]
fn fr_mask_phone_replace_person() {
    let text = "Bonjour Claire Martin, joignable au +33 6 12 34 56 78.";
    let engine = AnonymizerEngine::new();
    let mut cfg = AnonymizationConfig::default();
    cfg.operators_by_entity.insert(
        "PERSON".into(),
        OperatorConfig {
            operator_type: OperatorType::Replace,
            params: [("numbered".into(), "true".into())].into_iter().collect(),
        },
    );
    cfg.operators_by_entity.insert(
        "PHONE".into(),
        OperatorConfig {
            operator_type: OperatorType::Mask,
            params: [
                ("visible_prefix".into(), "4".into()),
                ("visible_suffix".into(), "2".into()),
            ]
            .into_iter()
            .collect(),
        },
    );
    let out = engine.anonymize(text, &synth_entities_fr(text), &cfg);
    assert!(out.text.contains("<PERSON_1>"));
    assert!(out.text.contains("+33 "));
    assert!(!out.text.contains("Claire Martin"));
}

#[test]
fn de_hash_email() {
    let e = Entity {
        entity_type: EntityType::Email,
        start: 10,
        end: 42,
        text: "max.mustermann.synth@beispiel.test".into(),
        score: 1.0,
        recognizer_name: "t".into(),
        metadata: HashMap::new(),
        decision_trace: None,
    };
    let mut p = HashMap::new();
    p.insert("algorithm".into(), "sha512".into());
    p.insert("salt".into(), "de-test-salt".into());
    p.insert("truncate".into(), "24".into());
    let cfg = OperatorConfig {
        operator_type: OperatorType::Hash,
        params: p,
    };
    let h = HashOperator.operate(&e, "", &cfg);
    assert!(h.starts_with("sha512:"));
    assert_eq!(h.len(), "sha512:".len() + 24);
}

#[test]
fn es_redact_national_id_placeholder() {
    let e = Entity {
        entity_type: EntityType::NationalId,
        start: 16,
        end: 25,
        text: "99999999R".into(),
        score: 1.0,
        recognizer_name: "t".into(),
        metadata: HashMap::new(),
        decision_trace: None,
    };
    let mut p = HashMap::new();
    p.insert("placeholder".into(), "[ID]".into());
    let cfg = OperatorConfig {
        operator_type: OperatorType::Redact,
        params: p,
    };
    let r = RedactOperator.operate(&e, "", &cfg);
    assert_eq!(r, "[ID]");
}

#[test]
fn it_encrypt_address_roundtrip() {
    let text = "Via Roma 1, 00100 Roma — signor Luigi Bianchi";
    let entities = vec![Entity {
        entity_type: EntityType::Address,
        start: 0,
        end: 22,
        text: "Via Roma 1, 00100 Roma".into(),
        score: 1.0,
        recognizer_name: "t".into(),
        metadata: HashMap::new(),
        decision_trace: None,
    }];
    let engine = AnonymizerEngine::new();
    let mut cfg = AnonymizationConfig::default();
    let mut p = HashMap::new();
    p.insert("key_hex".into(), KEY32.into());
    cfg.operators_by_entity.insert(
        "ADDRESS".into(),
        OperatorConfig {
            operator_type: OperatorType::Encrypt,
            params: p,
        },
    );
    let out = engine.anonymize(text, &entities, &cfg);
    let key = hex::decode(KEY32).unwrap();
    let back = AnonymizerEngine::deanonymize(&out, &key).unwrap();
    assert_eq!(back, text);
}

#[test]
fn key_rotation_map() {
    let engine = AnonymizerEngine::new();
    let text = "secret X";
    let entities = vec![Entity {
        entity_type: EntityType::TaxId,
        start: 7,
        end: 8,
        text: "X".into(),
        score: 1.0,
        recognizer_name: "t".into(),
        metadata: HashMap::new(),
        decision_trace: None,
    }];
    let mut cfg = AnonymizationConfig::default();
    let mut p = HashMap::new();
    p.insert("key_hex".into(), KEY32.into());
    p.insert("key_id".into(), "v2".into());
    cfg.operators_by_entity.insert(
        "TAX_ID".into(),
        OperatorConfig {
            operator_type: OperatorType::Encrypt,
            params: p,
        },
    );
    let out = engine.anonymize(text, &entities, &cfg);
    let mut keys = HashMap::new();
    keys.insert("v2".into(), hex::decode(KEY32).unwrap());
    let back = AnonymizerEngine::deanonymize_with_key_map(&out, &keys).unwrap();
    assert_eq!(back, text);
}
