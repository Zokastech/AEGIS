// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! YAML schema: contextual rules, `languages` inheritance (PERSON-only legacy), combinations.

use crate::entity::EntityType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context window in tokens (level 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorerWindowSettings {
    #[serde(default = "default_window_tokens")]
    pub tokens_before: usize,
    #[serde(default = "default_window_tokens")]
    pub tokens_after: usize,
}

fn default_window_tokens() -> usize {
    5
}

impl Default for ScorerWindowSettings {
    fn default() -> Self {
        Self {
            tokens_before: default_window_tokens(),
            tokens_after: default_window_tokens(),
        }
    }
}

/// Rule: boost / penalty words for a given entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRule {
    /// Config key: `PERSON`, `IBAN`, `MEDICAL_RECORD`, …
    pub entity_type: String,
    #[serde(default)]
    pub boost_words: Vec<String>,
    #[serde(default)]
    pub penalty_words: Vec<String>,
    #[serde(default = "default_boost_amount")]
    pub boost_amount: f64,
    #[serde(default = "default_penalty_amount")]
    pub penalty_amount: f64,
    #[serde(default)]
    pub note: String,
}

fn default_boost_amount() -> f64 {
    0.12
}

fn default_penalty_amount() -> f64 {
    0.15
}

/// Optional nested `scorer:` block in YAML (otherwise flat fields).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScorerYamlBlock {
    #[serde(default = "default_window_tokens")]
    pub tokens_before: usize,
    #[serde(default = "default_window_tokens")]
    pub tokens_after: usize,
}

/// Legacy per-language format (PERSON + deltas only).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageContextRules {
    #[serde(default)]
    pub person_boost: Vec<String>,
    #[serde(default)]
    pub person_penalty: Vec<String>,
    #[serde(default)]
    pub boost_delta: f64,
    #[serde(default = "default_penalty_delta")]
    pub penalty_delta: f64,
}

fn default_penalty_delta() -> f64 {
    0.12
}

impl LanguageContextRules {
    pub(crate) fn effective_boost_delta(&self) -> f64 {
        if self.boost_delta > 0.0 {
            self.boost_delta
        } else {
            0.08
        }
    }
}

/// Nearby entity combination (quasi-identifiers) — legacy engine format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinationRule {
    pub within_chars: usize,
    pub require_entity_types: Vec<EntityType>,
    /// Score added to matching entities.
    pub alert_score_boost: f64,
    #[serde(default)]
    pub note: String,
}

/// Context engine YAML config (dedicated file or `context_scorer:` in `aegis-config`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextScorerConfig {
    /// Window override via nested `scorer:` block.
    #[serde(default)]
    pub scorer: Option<ScorerYamlBlock>,
    #[serde(default = "default_window_tokens")]
    pub tokens_before: usize,
    #[serde(default = "default_window_tokens")]
    pub tokens_after: usize,
    /// Legacy mode: window in characters (no `rules`, `languages` only).
    #[serde(default)]
    pub context_window_chars: usize,
    #[serde(default)]
    pub rules: Vec<ContextRule>,
    #[serde(default)]
    pub languages: HashMap<String, LanguageContextRules>,
    #[serde(default)]
    pub combinations: Vec<CombinationRule>,
    #[serde(default)]
    pub quasi_identifiers: QuasiIdentifierYamlSection,
}

impl Default for ContextScorerConfig {
    fn default() -> Self {
        Self {
            scorer: None,
            tokens_before: default_window_tokens(),
            tokens_after: default_window_tokens(),
            context_window_chars: 0,
            rules: Vec::new(),
            languages: HashMap::new(),
            combinations: Vec::new(),
            quasi_identifiers: QuasiIdentifierYamlSection::default(),
        }
    }
}

impl ContextScorerConfig {
    pub fn from_yaml_str(s: &str) -> Result<Self> {
        let mut c: Self = serde_yaml::from_str(s)?;
        c.apply_scorer_block();
        Ok(c)
    }

    fn apply_scorer_block(&mut self) {
        if let Some(ref s) = self.scorer {
            self.tokens_before = s.tokens_before;
            self.tokens_after = s.tokens_after;
        }
    }

    /// Whether modern format (typed rules + tokens) is active.
    pub fn uses_modern_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    /// Whether legacy PERSON / language / character mode is used alone.
    pub fn uses_legacy_person_only(&self) -> bool {
        self.rules.is_empty() && !self.languages.is_empty()
    }
}

/// YAML section `quasi_identifiers:` — extended combinations + document risk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuasiIdentifierYamlSection {
    #[serde(default)]
    pub combinations: Vec<QuasiComboRuleYaml>,
    /// Base score before combinations (often 0).
    #[serde(default)]
    pub base_risk_score: f64,
    #[serde(default = "default_risk_cap")]
    pub cap_risk_at: f64,
}

fn default_risk_cap() -> f64 {
    1.0
}

/// Quasi-identifier rule: entities + optional keywords + risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuasiComboRuleYaml {
    pub id: String,
    #[serde(default)]
    pub note: String,
    pub within_chars: usize,
    #[serde(default)]
    pub require_entity_types: Vec<EntityType>,
    /// At least one of these substrings must appear in the text (lowercased).
    #[serde(default)]
    pub require_keywords_any: Vec<String>,
    pub alert_score_boost: f64,
    #[serde(default)]
    pub risk_increment: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_flat_rules() {
        let y = r#"
tokens_before: 3
tokens_after: 4
rules:
  - entity_type: PERSON
    boost_words: ["patient"]
    penalty_words: ["ville de"]
    boost_amount: 0.1
    penalty_amount: 0.2
"#;
        let c = ContextScorerConfig::from_yaml_str(y).unwrap();
        assert_eq!(c.tokens_before, 3);
        assert_eq!(c.rules.len(), 1);
    }
}
