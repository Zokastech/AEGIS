// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{phone_negative_context, phone_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use aegis_core::entity::EntityType;
use std::sync::Arc;

/// Normalise pour compter les chiffres (E.164 / formats locaux).
fn plausible_phone(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let masked = s.contains('*') || s.contains('•');
    // FR mobile 06/07 partially masked, e.g. "07 ** ** 45 91"
    if masked && (s.trim_start().starts_with("06") || s.trim_start().starts_with("07")) {
        return (6..=10).contains(&digits.len());
    }
    if s.contains('+') {
        return (8..=15).contains(&digits.len());
    }
    // NANP regex accepts 10 contiguous digits (and may include a leading `\s*`); require real
    // grouping — not just padding spaces (e.g. " 9876543210" from `\b\s*` before the run).
    if digits.len() == 10 && !s.trim_start().starts_with('0') {
        let t = s.trim();
        let has_grouping = t.contains('(')
            || t.contains(')')
            || t.contains('-')
            || t.contains('.')
            || t.contains('/')
            || t.as_bytes().windows(3).any(|w| {
                w[0].is_ascii_digit() && w[1].is_ascii_whitespace() && w[2].is_ascii_digit()
            });
        if !has_grouping {
            return false;
        }
    }
    (8..=14).contains(&digits.len())
}

/// International numbers (simplified E.164) and common EU / US formats.
pub fn phone_recognizer() -> PatternRecognizer {
    // Single line: `(?x)` + `#` comments silently broke alternation branches in the past.
    let re = compile(concat!(
        r"\+\s*(?:33|49|39|34|31|32|351|48|352|353|44|41|43|45|46|47|358|30|420|36|40|372|371|370|356|386|421|385)\s*",
        r"(?:[\s.\-/]*\d){7,14}\d|",
        r"(?:^|\b)\s*(?:\+1[\s.\-]+)?(?:\([0-9]{3}\)|[0-9]{3})[\s.\-]?[0-9]{3}[\s.\-]?[0-9]{4}\b|",
        r"\b0[1-9](?:[\s.\-/]\d{2}){4}\b|",
        r"\b0[67](?:[\s.\-/](?:\d{2}|\*{2}|[•xX]{2})){4}\b|",
        r"\b0[\s.\-/]*\d{2,4}(?:[\s.\-/]+\d{2,8}){1,8}\b",
    ));
    let pos: Vec<&str> = phone_positive_context();
    let neg: Vec<&str> = phone_negative_context();
    PatternRecognizer::new(
        "phone_e164_eu",
        re,
        EntityType::Phone,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.78,
    )
    .with_validator(Arc::new(plausible_phone))
    .with_min_score(0.4)
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
    fn fr_international() {
        let r = phone_recognizer();
        let v = r.analyze("Tel +33 6 12 34 56 78", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn de_international() {
        let r = phone_recognizer();
        let v = r.analyze("Call +49 30 12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn us_nanp() {
        let r = phone_recognizer();
        let v = r.analyze("(415) 555-0123", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn fr_local() {
        let r = phone_recognizer();
        let v = r.analyze("01 23 45 67 89", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn too_few_digits_rejected() {
        let r = phone_recognizer();
        assert!(r.analyze("+33 12 34", &cfg()).is_empty());
    }

    #[test]
    fn context_words_multilingual() {
        let r = phone_recognizer();
        assert!(!r.analyze("telefono +34 612 345 678", &cfg()).is_empty());
        assert!(!r.analyze("Telefon +49 89 123456", &cfg()).is_empty());
    }

    #[test]
    fn fr_mobile_partially_masked() {
        let r = phone_recognizer();
        let v = r.analyze("contact Samira (soeur) 07 ** ** 45 91 fin", &cfg());
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].text.contains("45 91"));
    }
}
