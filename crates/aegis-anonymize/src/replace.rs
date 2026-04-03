// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Replacement with typed tag, optional counter (`<EMAIL_1>`).

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;

pub struct ReplaceOperator;

fn type_tag(entity: &Entity) -> String {
    entity.entity_type.config_key()
}

impl Operator for ReplaceOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let base = config
            .params
            .get("tag")
            .cloned()
            .unwrap_or_else(|| type_tag(entity));
        let numbered = config
            .params
            .get("numbered")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        if !numbered {
            return if base.starts_with('<') && base.ends_with('>') {
                base
            } else {
                format!("<{base}>")
            };
        }
        let serial = config
            .params
            .get("__serial__")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);
        let name = base.trim_matches(|c| c == '<' || c == '>');
        format!("<{name}_{serial}>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    #[test]
    fn typed_numbered() {
        let e = Entity {
            entity_type: EntityType::Email,
            start: 0,
            end: 1,
            text: "".into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        };
        let mut p = HashMap::new();
        p.insert("numbered".into(), "true".into());
        p.insert("__serial__".into(), "2".into());
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Replace,
            params: p,
        };
        assert_eq!(ReplaceOperator.operate(&e, "", &c), "<EMAIL_2>");
    }
}
