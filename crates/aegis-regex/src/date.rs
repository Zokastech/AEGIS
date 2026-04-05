// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{date_negative_context, date_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use aegis_core::entity::EntityType;
use regex::Regex;
use std::sync::Arc;

fn plausible_calendar_date(s: &str) -> bool {
    let t = s.trim();
    static ISO: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let iso = ISO.get_or_init(|| compile(r"^(\d{4})-(\d{2})-(\d{2})"));
    if let Some(cap) = iso.captures(t) {
        let m: u32 = cap[2].parse().unwrap_or(0);
        let d: u32 = cap[3].parse().unwrap_or(0);
        return m >= 1 && m <= 12 && d >= 1 && d <= 31;
    }
    let nums: Vec<u32> = t
        .split(|c: char| c == '/' || c == '.' || c == '-')
        .filter_map(|p| p.parse().ok())
        .collect();
    if nums.len() >= 3 {
        let (a, b, c) = (nums[0], nums[1], nums[2]);
        // rejette les triplets type « version » 1.2.3
        if a < 13 && b < 13 && c < 100 {
            return false;
        }
        let (day, month) = if c > 31 {
            (a.min(31), b.min(12))
        } else if a > 12 {
            (a.min(31), b.min(12))
        } else {
            (a.min(31), b.min(12))
        };
        return (1..=31).contains(&day) && (1..=12).contains(&month);
    }
    true
}

/// European dates (separators `/` `.` `-`) and ISO 8601 (date + simple date-time).
pub fn date_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?xi)
        \b\d{4}-\d{2}-\d{2}(?:[T\s]\d{2}:\d{2}(?::\d{2})?(?:Z|[+-]\d{2}:\d{2})?)?\b
        | \b\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}\b
        ",
    );
    let pos: Vec<&str> = date_positive_context();
    let neg: Vec<&str> = date_negative_context();
    PatternRecognizer::new(
        "date_eu_iso",
        re,
        EntityType::Date,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.65,
    )
    .with_validator(Arc::new(|s| plausible_calendar_date(s)))
    .with_min_score(0.35)
    .with_context_boost_words(&pos, 0.06)
    .with_context_penalty_words(&neg, 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recognizer;
    use aegis_core::config::AnalysisConfig;

    fn cfg() -> AnalysisConfig {
        AnalysisConfig::default()
    }

    #[test]
    fn iso_date() {
        let r = date_recognizer();
        let v = r.analyze("created 2024-03-15 end", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn eu_slash() {
        let r = date_recognizer();
        let v = r.analyze("le 31/12/2025 fin", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn eu_dot() {
        let r = date_recognizer();
        let v = r.analyze("Datum 15.03.2024", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn reject_version_triplet() {
        let r = date_recognizer();
        assert!(r.analyze("version 1.2.3.4 build", &cfg()).is_empty());
    }

    #[test]
    fn multilingual_context() {
        let r = date_recognizer();
        assert!(!r.analyze("fecha 12/05/2024", &cfg()).is_empty());
        assert!(!r.analyze("data di nascita 01-01-1990", &cfg()).is_empty());
    }
}
