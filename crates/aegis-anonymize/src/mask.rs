// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Partial masking: visible prefix/suffix, mask character, optional per-word mode.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;

pub struct MaskOperator;

fn mask_token(s: &str, prefix: usize, suffix: usize, ch: char) -> String {
    if s.is_empty() {
        return String::new();
    }
    if s.len() <= prefix + suffix {
        return ch.to_string().repeat(s.len());
    }
    let mut out = String::new();
    out.push_str(&s[..prefix]);
    out.extend(std::iter::repeat(ch).take(s.len().saturating_sub(prefix + suffix)));
    out.push_str(&s[s.len() - suffix..]);
    out
}

impl Operator for MaskOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let s = &entity.text;
        let prefix: usize = config
            .params
            .get("visible_prefix")
            .and_then(|x| x.parse().ok())
            .unwrap_or(1);
        let suffix: usize = config
            .params
            .get("visible_suffix")
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        let ch = config
            .params
            .get("mask_char")
            .or_else(|| config.params.get("char"))
            .and_then(|c| c.chars().next())
            .unwrap_or('*');

        let per_word = config
            .params
            .get("per_word")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        if !per_word {
            return mask_token(s, prefix, suffix, ch);
        }

        s.split_whitespace()
            .map(|w| mask_token(w, prefix, suffix, ch))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    fn ent(text: &str) -> Entity {
        Entity {
            entity_type: EntityType::Person,
            start: 0,
            end: text.len(),
            text: text.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    #[test]
    fn jean_dupont_per_word() {
        let mut p = HashMap::new();
        p.insert("per_word".into(), "true".into());
        p.insert("visible_prefix".into(), "1".into());
        p.insert("visible_suffix".into(), "0".into());
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Mask,
            params: p,
        };
        let out = MaskOperator.operate(&ent("Jean Dupont"), "", &c);
        assert_eq!(out, "J*** D*****");
    }

    #[test]
    fn iban_edges() {
        let mut p = HashMap::new();
        p.insert("visible_prefix".into(), "4".into());
        p.insert("visible_suffix".into(), "4".into());
        let c = OperatorConfig {
            operator_type: aegis_core::OperatorType::Mask,
            params: p,
        };
        let iban = "FR7630006000011234567890189";
        let out = MaskOperator.operate(&ent(iban), "", &c);
        assert_eq!(out, "FR76**********************0189");
    }
}
