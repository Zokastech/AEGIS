// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Stable derived pseudonym (BLAKE3) to hide values while keeping a join identifier.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;
use blake3::Hasher;

pub struct PseudonymizeOperator;

impl Operator for PseudonymizeOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let salt = config.params.get("salt").map(|s| s.as_str()).unwrap_or("aegis_ps");
        let mut h = Hasher::new();
        h.update(salt.as_bytes());
        h.update(entity.entity_type.config_key().as_bytes());
        h.update(&[0]);
        h.update(entity.text.as_bytes());
        h.update(&entity.start.to_le_bytes());
        h.update(&entity.end.to_le_bytes());
        let hx = hex::encode(&h.finalize().as_bytes()[..10]);
        let label = config
            .params
            .get("label")
            .cloned()
            .unwrap_or_else(|| entity.entity_type.config_key());
        format!("⟨{label}:{hx}⟩")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    #[test]
    fn deterministic() {
        let e = Entity {
            entity_type: EntityType::Person,
            start: 2,
            end: 12,
            text: "Marie Martin".into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        };
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Pseudonymize,
            params: HashMap::new(),
        };
        let a = PseudonymizeOperator.operate(&e, "", &c);
        let b = PseudonymizeOperator.operate(&e, "", &c);
        assert_eq!(a, b);
        assert!(a.starts_with("⟨PERSON:"));
    }
}
