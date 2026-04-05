// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Composite recognizer: multiple regex patterns + per-rule validators.

use crate::recognizers::financial::common::{adjust_score_in_context, build_ac};
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

pub struct IdRule {
    pub name: &'static str,
    pub re: Regex,
    pub entity: EntityType,
    pub validator: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    pub base_score: f64,
}

pub struct CompositeNationalRecognizer {
    id: &'static str,
    rules: Vec<IdRule>,
    languages: Vec<&'static str>,
    boost_ac: Option<AhoCorasick>,
    penalty_ac: Option<AhoCorasick>,
    min_score: f64,
    score_cap: f64,
}

impl CompositeNationalRecognizer {
    pub fn new(
        id: &'static str,
        rules: Vec<IdRule>,
        languages: Vec<&'static str>,
        context_words: &[&str],
    ) -> Self {
        Self {
            id,
            rules,
            languages,
            boost_ac: build_ac(context_words),
            penalty_ac: build_ac(&["example", "test", "dummy", "sample"]),
            min_score: 0.42,
            score_cap: 1.0,
        }
    }

    /// Minimum final score (after context) required to emit an entity.
    pub fn with_min_score(mut self, s: f64) -> Self {
        self.min_score = s.clamp(0.0, 1.0);
        self
    }
}

impl Recognizer for CompositeNationalRecognizer {
    fn name(&self) -> &str {
        self.id
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        let mut v: Vec<EntityType> = self.rules.iter().map(|r| r.entity.clone()).collect();
        v.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        v.dedup_by(|a, b| a == b);
        v
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.languages.clone()
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        for rule in &self.rules {
            for cap in rule.re.captures_iter(text) {
                let id_cap = (1..=4)
                    .find_map(|i| cap.get(i))
                    .or_else(|| cap.get(0))
                    .expect("capture 0");
                let slice = id_cap.as_str();
                if !(rule.validator)(slice) {
                    continue;
                }
                let Some(score) = adjust_score_in_context(
                    rule.base_score,
                    text,
                    id_cap.start(),
                    id_cap.end(),
                    config,
                    self.boost_ac.as_ref(),
                    0.06,
                    self.penalty_ac.as_ref(),
                    0.1,
                    None,
                    0.1,
                    self.min_score,
                    self.score_cap,
                ) else {
                    continue;
                };
                if score < self.min_score {
                    continue;
                }
                let mut metadata = HashMap::new();
                metadata.insert("level".into(), "regex".into());
                metadata.insert("rule".into(), rule.name.into());
                if config.return_decision_process {
                    metadata.insert("country_rule".into(), rule.name.into());
                }
                out.push(Entity {
                    entity_type: rule.entity.clone(),
                    start: id_cap.start(),
                    end: id_cap.end(),
                    text: slice.to_string(),
                    score,
                    recognizer_name: self.id.to_string(),
                    metadata,
                    decision_trace: None,
                });
            }
        }
        out
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}
