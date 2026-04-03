// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! [`PatternRecognizer`]: regex + Aho–Corasick automata (deny-list, context, invalidation).

use crate::utf8_window::byte_window_utf8;
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

type ValidatorFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

fn build_ac(patterns: &[&str]) -> Option<AhoCorasick> {
    if patterns.is_empty() {
        return None;
    }
    let pats: Vec<Vec<u8>> = patterns.iter().map(|w| w.as_bytes().to_vec()).collect();
    AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(pats)
        .ok()
}

/// Generic recognizer: [`regex::Regex`] pattern, optional validator, Aho–Corasick lists.
///
/// - **Deny-list**: substrings in the **match** → entity rejected.
/// - **Invalidation**: patterns in the **context window** → penalty (floor [`PatternRecognizer::with_min_score`]).
/// - **Positive / negative context**: bonus or penalty when a word appears in the window around the entity.
pub struct PatternRecognizer {
    name: String,
    pattern: Regex,
    entity_type: EntityType,
    languages: Vec<String>,
    base_score: f64,
    min_score: f64,
    score_cap: f64,
    validator: Option<ValidatorFn>,
    /// Patterns present **in the captured text** → reject.
    deny_in_match_ac: Option<AhoCorasick>,
    /// Patterns in the window → penalty per hit.
    invalidate_ac: Option<AhoCorasick>,
    invalidate_penalty: f64,
    /// “Positive” context words (all EU languages merged into one automaton).
    context_boost_ac: Option<AhoCorasick>,
    context_boost: f64,
    /// “Negative” context words (e.g. known false positives).
    context_penalty_ac: Option<AhoCorasick>,
    context_penalty: f64,
}

impl PatternRecognizer {
    pub fn new(
        name: impl Into<String>,
        pattern: Regex,
        entity_type: EntityType,
        languages: Vec<&str>,
        base_score: f64,
    ) -> Self {
        Self {
            name: name.into(),
            pattern,
            entity_type,
            languages: languages.iter().map(|s| (*s).to_string()).collect(),
            base_score: base_score.clamp(0.0, 1.0),
            min_score: 0.5,
            score_cap: 1.0,
            validator: None,
            deny_in_match_ac: None,
            invalidate_ac: None,
            invalidate_penalty: 0.15,
            context_boost_ac: None,
            context_boost: 0.08,
            context_penalty_ac: None,
            context_penalty: 0.12,
        }
    }

    pub fn with_validator(mut self, v: ValidatorFn) -> Self {
        self.validator = Some(v);
        self
    }

    /// Forbidden substrings **inside** the regex match (e.g. fake domains).
    pub fn with_deny_substrings(mut self, words: &[&str]) -> Self {
        self.deny_in_match_ac = build_ac(words);
        self
    }

    /// Penalty per hit in the context window (invalidation).
    pub fn with_invalidate_words(mut self, words: &[&str]) -> Self {
        self.invalidate_ac = build_ac(words);
        self
    }

    pub fn with_invalidate_penalty(mut self, p: f64) -> Self {
        self.invalidate_penalty = p;
        self
    }

    /// Bonus when at least one word appears in the window (e.g. “email”, “courriel”…).
    pub fn with_context_boost_words(mut self, words: &[&str], delta: f64) -> Self {
        self.context_boost_ac = build_ac(words);
        self.context_boost = delta;
        self
    }

    /// Penalty per occurrence when a word appears in the window (e.g. “example”, “test”…).
    pub fn with_context_penalty_words(mut self, words: &[&str], delta: f64) -> Self {
        self.context_penalty_ac = build_ac(words);
        self.context_penalty = delta;
        self
    }

    pub fn with_min_score(mut self, s: f64) -> Self {
        self.min_score = s.clamp(0.0, 1.0);
        self
    }

    pub fn with_score_cap(mut self, c: f64) -> Self {
        self.score_cap = c.clamp(0.0, 1.0);
        self
    }

    fn context_window_bytes(config: &AnalysisConfig) -> usize {
        config.context_window_size.max(1).saturating_mul(12)
    }

    /// Computes final score or `None` if the match is rejected (deny-in-match).
    fn score_for_match(&self, text: &str, start: usize, end: usize, config: &AnalysisConfig) -> Option<f64> {
        if let Some(ref ac) = self.deny_in_match_ac {
            let slice = text.get(start..end)?.as_bytes();
            if ac.find_iter(slice).next().is_some() {
                return None;
            }
        }

        let window = Self::context_window_bytes(config);
        let lo = start.saturating_sub(window);
        let hi = (end + window).min(text.len());
        let ctx_bytes = byte_window_utf8(text, lo, hi).as_bytes();

        let mut score = self.base_score;

        if let Some(ref ac) = self.context_boost_ac {
            if ac.find_iter(ctx_bytes).next().is_some() {
                score = (score + self.context_boost).min(self.score_cap);
            }
        }
        if let Some(ref ac) = self.context_penalty_ac {
            for _ in ac.find_iter(ctx_bytes) {
                score = (score - self.context_penalty).max(self.min_score);
            }
        }
        if let Some(ref ac) = self.invalidate_ac {
            for _ in ac.find_iter(ctx_bytes) {
                score = (score - self.invalidate_penalty).max(self.min_score);
            }
        }

        Some(score.min(self.score_cap))
    }
}

impl Recognizer for PatternRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![self.entity_type.clone()]
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.languages.iter().map(|s| s.as_str()).collect()
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        for m in self.pattern.find_iter(text) {
            let slice = m.as_str();
            if let Some(ref v) = self.validator {
                if !v(slice) {
                    continue;
                }
            }
            let Some(score) = self.score_for_match(text, m.start(), m.end(), config) else {
                continue;
            };
            if score < self.min_score {
                continue;
            }
            let mut metadata = HashMap::new();
            metadata.insert("level".into(), "regex".into());
            if config.return_decision_process {
                metadata.insert("base_score".into(), format!("{:.4}", self.base_score));
                metadata.insert("final_score".into(), format!("{:.4}", score));
            }
            out.push(Entity {
                entity_type: self.entity_type.clone(),
                start: m.start(),
                end: m.end(),
                text: slice.to_string(),
                score,
                recognizer_name: self.name.clone(),
                metadata,
                decision_trace: None,
            });
        }
        out
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recognizer;
    use aegis_core::entity::EntityType;

    fn sample_email_re() -> Regex {
        Regex::new(r"(?i)\b[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}\b").unwrap()
    }

    #[test]
    fn deny_in_match_rejects() {
        let r = PatternRecognizer::new(
            "t",
            sample_email_re(),
            EntityType::Email,
            vec!["*"],
            0.9,
        )
        .with_deny_substrings(&["example.com", "test.invalid"]);
        let text = "contact user@example.com now";
        let cfg = AnalysisConfig::default();
        let v = r.analyze(text, &cfg);
        assert!(v.is_empty());
    }

    #[test]
    fn context_boost_increases_score() {
        let r = PatternRecognizer::new(
            "t",
            sample_email_re(),
            EntityType::Email,
            vec!["*"],
            0.5,
        )
        .with_min_score(0.3)
        .with_context_boost_words(&["email"], 0.25);
        let text = "my email is a@b.co";
        let cfg = AnalysisConfig::default();
        let v = r.analyze(text, &cfg);
        assert_eq!(v.len(), 1);
        assert!(v[0].score > 0.5);
    }

    #[test]
    fn context_penalty_reduces_score_below_min() {
        let r = PatternRecognizer::new(
            "t",
            sample_email_re(),
            EntityType::Email,
            vec!["*"],
            0.95,
        )
        .with_min_score(0.5)
        .with_context_penalty_words(&["example"], 0.5);
        let text = "see example a@b.co for info";
        let cfg = AnalysisConfig::default();
        assert!(r.analyze(text, &cfg).is_empty());
    }

    #[test]
    fn validator_filters() {
        let r = PatternRecognizer::new(
            "t",
            Regex::new(r"\d+").unwrap(),
            EntityType::TaxId,
            vec!["*"],
            0.9,
        )
        .with_validator(Arc::new(|s| s.len() == 3));
        let cfg = AnalysisConfig::default();
        let v = r.analyze("12 999 4", &cfg);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].text, "999");
    }
}
