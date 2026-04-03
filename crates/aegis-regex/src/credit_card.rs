// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{card_negative_context, card_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use crate::validation::validate_credit_card_match;
use aegis_core::entity::EntityType;
use std::sync::Arc;

/// Visa, MasterCard, AmEx and other common networks + Luhn validation.
pub fn credit_card_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?x)
        \b(?:\d[\s-]*?){12,18}\d\b
        ",
    );
    let pos: Vec<&str> = card_positive_context();
    let neg: Vec<&str> = card_negative_context();
    PatternRecognizer::new(
        "credit_card_luhn",
        re,
        EntityType::CreditCard,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.72,
    )
    .with_validator(Arc::new(|s| validate_credit_card_match(s)))
    .with_min_score(0.45)
    .with_context_boost_words(&pos, 0.08)
    .with_context_penalty_words(&neg, 0.12)
}

fn valid_brand_masked_last4(s: &str) -> bool {
    let has_mask = s.contains('*') || s.contains('•');
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    has_mask && digits.len() == 4
}

/// **Masked** card (`****` segments + last 4 digits), e.g. emails "Visa **** … 4832".
/// No Luhn (incomplete PAN) — slightly lower score.
pub fn masked_credit_card_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?xi)
        \b
        (?:visa|mastercard|master\s*card|\bmc\b|amex|american\s*express)
        \s+
        (?:\*{2,4}|[•x]{2,4})
        (?:\s+[\s*•x._-]*(?:\*{2,4}|[•x]{2,4})){2}
        \s+
        \d{4}
        \b
        ",
    );
    let pos: Vec<&str> = card_positive_context();
    let neg: Vec<&str> = card_negative_context();
    PatternRecognizer::new(
        "credit_card_masked_last4",
        re,
        EntityType::CreditCard,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.62,
    )
    .with_validator(Arc::new(|s| valid_brand_masked_last4(s)))
    .with_min_score(0.42)
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
    fn visa_valid_luhn() {
        let r = credit_card_recognizer();
        let v = r.analyze("pay 4532015112830366 now", &cfg());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn invalid_luhn_rejected() {
        let r = credit_card_recognizer();
        assert!(r.analyze("4532015112830367", &cfg()).is_empty());
    }

    #[test]
    fn spaced_number() {
        let r = credit_card_recognizer();
        let v = r.analyze("4532 0151 1283 0366", &cfg());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn false_positive_short_digit_run() {
        let r = credit_card_recognizer();
        assert!(r.analyze("order id 12345678901234", &cfg()).is_empty());
    }

    #[test]
    fn context_cartes_bancaire() {
        let r = credit_card_recognizer();
        let fr = r.analyze("carte bancaire 4532015112830366", &cfg());
        assert!(!fr.is_empty());
    }

    #[test]
    fn visa_masked_last_four() {
        let r = masked_credit_card_recognizer();
        let v = r.analyze(
            "Visa **** **** **** 4832 (expiration 09/27)",
            &cfg(),
        );
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].text.contains("4832"));
    }

    #[test]
    fn masked_requires_brand_and_four_digits() {
        let r = masked_credit_card_recognizer();
        assert!(r
            .analyze("**** **** **** 4832", &cfg())
            .is_empty());
    }
}
