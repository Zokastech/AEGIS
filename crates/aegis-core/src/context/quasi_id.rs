// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Quasi-identifiers: entity combinations + keywords, document re-identification risk score.

use crate::context::rules::{CombinationRule, ContextScorerConfig, QuasiComboRuleYaml};
use crate::entity::{Entity, EntityType};

/// Re-identification risk report for a document.
#[derive(Debug, Clone, Default)]
pub struct QuasiIdentifierReport {
    /// Aggregated score \[0, cap\].
    pub risk_score: f64,
    /// Alert messages (rule ids or notes).
    pub alerts: Vec<String>,
}

/// Configurable detector (YAML [`ContextScorerConfig::quasi_identifiers`] + `combinations`).
#[derive(Debug, Clone)]
pub struct QuasiIdentifierDetector {
    classic: Vec<CombinationRule>,
    extended: Vec<QuasiComboRuleYaml>,
    base_risk: f64,
    cap: f64,
}

impl QuasiIdentifierDetector {
    pub fn from_config(cfg: &ContextScorerConfig) -> Self {
        Self {
            classic: cfg.combinations.clone(),
            extended: cfg.quasi_identifiers.combinations.clone(),
            base_risk: cfg.quasi_identifiers.base_risk_score,
            cap: cfg.quasi_identifiers.cap_risk_at,
        }
    }

    /// Evaluates the document: risk score + text alerts.
    pub fn assess_document(&self, text: &str, entities: &[Entity]) -> QuasiIdentifierReport {
        let lower = text.to_lowercase();
        let mut risk = self.base_risk;
        let mut alerts = Vec::new();

        for rule in &self.classic {
            if let Some(msg) = self.combo_satisfied_message(rule, entities, &lower) {
                risk += (rule.alert_score_boost * 0.5).min(0.25);
                alerts.push(msg);
            }
        }

        for rule in &self.extended {
            if let Some(msg) = self.extended_satisfied_message(rule, entities, &lower) {
                risk += rule.risk_increment.max(0.0);
                alerts.push(msg);
            }
        }

        risk = risk.min(self.cap).max(0.0);
        QuasiIdentifierReport {
            risk_score: risk,
            alerts,
        }
    }

    /// Boosts to apply to entities (indices) for the L2 pipeline.
    pub fn entity_boosts(&self, entities: &[Entity], text: &str) -> Vec<(usize, f64, String)> {
        let lower = text.to_lowercase();
        let mut out = Vec::new();
        for rule in &self.classic {
            if let Some(indices) = self.combo_entity_indices(rule, entities) {
                for &i in &indices {
                    out.push((i, rule.alert_score_boost, rule.note.clone()));
                }
            }
        }
        for rule in &self.extended {
            if let Some(indices) = self.extended_entity_indices(rule, entities, &lower) {
                let note = if rule.note.is_empty() {
                    rule.id.clone()
                } else {
                    format!("{}: {}", rule.id, rule.note)
                };
                for &i in &indices {
                    out.push((i, rule.alert_score_boost, note.clone()));
                }
            }
        }
        out
    }

    fn combo_satisfied_message(
        &self,
        rule: &CombinationRule,
        entities: &[Entity],
        _text_lower: &str,
    ) -> Option<String> {
        self.combo_entity_indices(rule, entities).map(|_| {
            if rule.note.is_empty() {
                "quasi_identifier_combo".to_string()
            } else {
                rule.note.clone()
            }
        })
    }

    fn combo_entity_indices(
        &self,
        rule: &CombinationRule,
        entities: &[Entity],
    ) -> Option<Vec<usize>> {
        if rule.require_entity_types.is_empty() {
            return None;
        }
        let combos = product_entity_indices(entities, &rule.require_entity_types)?;
        combos
            .into_iter()
            .find(|combo| span_width(combo, entities) <= rule.within_chars)
    }

    fn extended_satisfied_message(
        &self,
        rule: &QuasiComboRuleYaml,
        entities: &[Entity],
        text_lower: &str,
    ) -> Option<String> {
        self.extended_entity_indices(rule, entities, text_lower)
            .map(|_| {
                if rule.note.is_empty() {
                    rule.id.clone()
                } else {
                    format!("{}: {}", rule.id, rule.note)
                }
            })
    }

    fn extended_entity_indices(
        &self,
        rule: &QuasiComboRuleYaml,
        entities: &[Entity],
        text_lower: &str,
    ) -> Option<Vec<usize>> {
        if rule.require_entity_types.is_empty() {
            return None;
        }
        if !rule.require_keywords_any.is_empty() {
            let any = rule.require_keywords_any.iter().any(|k| {
                let k = k.to_lowercase();
                text_lower.contains(k.as_str())
            });
            if !any {
                return None;
            }
        }
        let combos = product_entity_indices(entities, &rule.require_entity_types)?;
        combos
            .into_iter()
            .find(|combo| span_width(combo, entities) <= rule.within_chars)
    }
}

fn span_width(indices: &[usize], entities: &[Entity]) -> usize {
    let mut mn = usize::MAX;
    let mut mx = 0usize;
    for &i in indices {
        if let Some(e) = entities.get(i) {
            mn = mn.min(e.start);
            mx = mx.max(e.end);
        }
    }
    mx.saturating_sub(mn)
}

fn product_entity_indices(entities: &[Entity], types: &[EntityType]) -> Option<Vec<Vec<usize>>> {
    let mut lists: Vec<Vec<usize>> = Vec::new();
    for t in types {
        let v: Vec<usize> = entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.entity_type == *t)
            .map(|(i, _)| i)
            .collect();
        if v.is_empty() {
            return None;
        }
        lists.push(v);
    }
    let mut combos: Vec<Vec<usize>> = Vec::new();
    fn product_rec(lists: &[Vec<usize>], depth: usize, cur: Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if depth == lists.len() {
            out.push(cur);
            return;
        }
        for &x in &lists[depth] {
            let mut c = cur.clone();
            c.push(x);
            product_rec(lists, depth + 1, c, out);
        }
    }
    product_rec(&lists, 0, Vec::new(), &mut combos);
    Some(combos)
}

#[cfg(test)]
impl QuasiIdentifierDetector {
    fn test_new(classic: Vec<CombinationRule>, extended: Vec<QuasiComboRuleYaml>) -> Self {
        Self {
            classic,
            extended,
            base_risk: 0.0,
            cap: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn e(tp: EntityType, start: usize, end: usize) -> Entity {
        Entity {
            entity_type: tp,
            start,
            end,
            text: String::new(),
            score: 0.9,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    #[test]
    fn date_location_combo() {
        let rule = CombinationRule {
            within_chars: 50,
            require_entity_types: vec![EntityType::Date, EntityType::Location],
            alert_score_boost: 0.1,
            note: "d+l".into(),
        };
        let entities = vec![e(EntityType::Date, 0, 10), e(EntityType::Location, 30, 40)];
        let q = QuasiIdentifierDetector::test_new(vec![rule], vec![]);
        let b = q.entity_boosts(&entities, "x");
        assert_eq!(b.len(), 2);
    }
}
