// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Salted hashing: SHA-256 (default), SHA-512, BLAKE3; optional truncation.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;
use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256, Sha512};

pub struct HashOperator;

fn hash_bytes(alg: &str, salt: &[u8], payload: &[u8]) -> Vec<u8> {
    match alg {
        "sha512" => {
            let mut h = Sha512::new();
            h.update(salt);
            h.update(payload);
            h.finalize().to_vec()
        }
        "blake3" => {
            let mut h = Blake3Hasher::new();
            h.update(salt);
            h.update(payload);
            h.finalize().as_bytes().to_vec()
        }
        _ => {
            let mut h = Sha256::new();
            h.update(salt);
            h.update(payload);
            h.finalize().to_vec()
        }
    }
}

impl Operator for HashOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let alg = config
            .params
            .get("algorithm")
            .map(|s| s.as_str())
            .unwrap_or("sha256");

        let global_salt = config
            .params
            .get("salt")
            .map(|s| s.as_bytes())
            .unwrap_or(b"");
        let per_entity = config
            .params
            .get("salt_scope")
            .map(|s| s == "entity" || s == "per_entity")
            .unwrap_or(false);

        let entity_salt = if per_entity {
            let mut s = String::new();
            s.push_str(&entity.entity_type.config_key());
            s.push(':');
            s.push_str(&entity.text);
            s.into_bytes()
        } else {
            Vec::new()
        };

        let mut salt_buf = Vec::new();
        salt_buf.extend_from_slice(global_salt);
        salt_buf.extend_from_slice(&entity_salt);

        let full = hash_bytes(alg, &salt_buf, entity.text.as_bytes());
        let hex_full = hex::encode(&full);
        let truncate: usize = config
            .params
            .get("truncate")
            .and_then(|x| x.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(16)
            .min(hex_full.len());
        format!("{}:{}", alg, &hex_full[..truncate])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    #[test]
    fn blake3_and_truncate() {
        let e = Entity {
            entity_type: EntityType::Email,
            start: 0,
            end: 10,
            text: "synthetic@test.invalid".into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        };
        let mut p = HashMap::new();
        p.insert("algorithm".into(), "blake3".into());
        p.insert("salt".into(), "pepper".into());
        p.insert("truncate".into(), "20".into());
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Hash,
            params: p,
        };
        let out = HashOperator.operate(&e, "", &c);
        assert!(out.starts_with("blake3:"));
        assert_eq!(out.len(), "blake3:".len() + 20);
    }
}
