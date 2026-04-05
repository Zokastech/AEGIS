// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Full redaction: empty string or configurable placeholder.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;

pub struct RedactOperator;

impl Operator for RedactOperator {
    fn operate(&self, _entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        if config
            .params
            .get("use_empty")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
        {
            return String::new();
        }
        config
            .params
            .get("placeholder")
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty() && *s != "__EMPTY__")
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    fn entity() -> Entity {
        Entity {
            entity_type: EntityType::Email,
            start: 0,
            end: 5,
            text: "a@b.c".into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    #[test]
    fn empty_default() {
        let mut p = HashMap::new();
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Redact,
            params: p.clone(),
        };
        assert_eq!(RedactOperator.operate(&entity(), "x", &c), "");
        p.insert("placeholder".into(), "[X]".into());
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Redact,
            params: p,
        };
        assert_eq!(RedactOperator.operate(&entity(), "x", &c), "[X]");
    }
}
