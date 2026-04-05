// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Payment cards with European BIN focus (Visa, MC, AmEx, CB, Bancontact, Maestro, Carta Si) + Luhn.

use super::common::eu_card_context_words;
use crate::pattern::PatternRecognizer;
use crate::validation::{digits_only, luhn_valid};
use aegis_core::entity::EntityType;
use regex::Regex;
use std::sync::Arc;

/// BIN prefixes (4 or 6 digits) commonly issued in Europe / requested networks.
fn eu_or_major_bin(d: &str) -> bool {
    if d.len() < 4 {
        return false;
    }
    let p4: u32 = d.get(0..4).and_then(|x| x.parse().ok()).unwrap_or(0);
    let p6: u32 = d.get(0..6).and_then(|x| x.parse().ok()).unwrap_or(0);
    // Bancontact / BCMC (prioritaire)
    if matches!(p4, 6703 | 4871 | 4870) {
        return true;
    }
    // Visa — EU / domestic sub-ranges (excludes generic 4111x test bins)
    if d.starts_with('4') {
        return eu_visa_bin(p4, p6);
    }
    // MasterCard / Maestro / Carta Si
    if d.starts_with('5') || d.starts_with('2') {
        return eu_mc_maestro_cartasi(p4, p6);
    }
    // AmEx 34, 37
    if d.len() >= 2 {
        let t2: u32 = d.get(0..2).and_then(|x| x.parse().ok()).unwrap_or(0);
        if matches!(t2, 34 | 37) && d.len() == 15 {
            return true;
        }
    }
    false
}

fn eu_visa_bin(p4: u32, p6: u32) -> bool {
    (416500..=416599).contains(&p6)
        || (497300..=497999).contains(&p6)
        || (450800..=450899).contains(&p6)
        || matches!(p4, 4035 | 4165 | 4177 | 4508)
        || matches!(p4 / 100, 4974..=4977)
}

fn eu_mc_maestro_cartasi(p4: u32, p6: u32) -> bool {
    let mc51_55 = (5100..=5599).contains(&p4);
    let mc2 = (2221..=2720).contains(&p4);
    let maestro = (50..=69).contains(&(p4 / 100));
    let carta_si = (500000..=509999).contains(&p6) || (530000..=539999).contains(&p6);
    mc51_55 || mc2 || maestro || carta_si
}

fn eu_credit_card_ok(s: &str) -> bool {
    let d = digits_only(s);
    if !(13..=19).contains(&d.len()) {
        return false;
    }
    if !luhn_valid(&d) {
        return false;
    }
    eu_or_major_bin(&d)
}

pub fn eu_credit_card_recognizer() -> PatternRecognizer {
    let re = Regex::new(
        r"(?x)
        \b(?:\d[\s-]*?){12,18}\d\b
        ",
    )
    .expect("eu credit card regex");
    let ctx = eu_card_context_words();
    PatternRecognizer::new(
        "eu_credit_card_bins",
        re,
        EntityType::CreditCard,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"],
        0.78,
    )
    .with_validator(Arc::new(|m| eu_credit_card_ok(m)))
    .with_min_score(0.48)
    .with_context_boost_words(&ctx, 0.08)
    .with_context_penalty_words(&["test card", "fake", "example"], 0.14)
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
    fn luhn_visa_eu_bin_detected() {
        let r = eu_credit_card_recognizer();
        let mut hit = false;
        for cd in 0..10 {
            let s = format!("497301234567890{cd}");
            if luhn_valid(&s) && eu_credit_card_ok(&s) {
                let v = r.analyze(&format!("Carte Bleue {s}"), &cfg());
                assert!(!v.is_empty(), "{s}");
                hit = true;
                break;
            }
        }
        assert!(
            hit,
            "aucune variante Luhn valide pour le BIN 497301234567890x"
        );
    }

    #[test]
    fn bancontact_bin_6703_with_luhn() {
        let r = eu_credit_card_recognizer();
        let mut found = false;
        for xx in 0u32..100 {
            let s = format!("6703123456789{:02}", xx);
            if luhn_valid(&s) && eu_credit_card_ok(&s) {
                assert!(!r.analyze(&format!("Bancontact {s}"), &cfg()).is_empty());
                found = true;
                break;
            }
        }
        assert!(
            found,
            "aucune variante Luhn pour préfixe Bancontact 6703123456789xx"
        );
    }

    #[test]
    fn rejects_non_luhn() {
        let r = eu_credit_card_recognizer();
        assert!(r.analyze("4973012345678905", &cfg()).is_empty());
    }

    #[test]
    fn rejects_generic_test_visa_4111() {
        let r = eu_credit_card_recognizer();
        assert!(r.analyze("4111111111111111", &cfg()).is_empty());
    }

    #[test]
    fn mc_51_luhn_valid() {
        let r = eu_credit_card_recognizer();
        let v = r.analyze("Mastercard 5555555555554444", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn amex_15() {
        let r = eu_credit_card_recognizer();
        let v = r.analyze("AmEx 378282246310005", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn spaced_number() {
        let r = eu_credit_card_recognizer();
        let v = r.analyze("4973 0123 4567 8908", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn eu_bin_helper_visa_4973xx() {
        assert!(eu_or_major_bin("4973012345678908"));
    }

    #[test]
    fn eu_bin_helper_mc_55() {
        assert!(eu_or_major_bin("5555555555554444"));
    }

    #[test]
    fn eu_bin_helper_amex() {
        assert!(eu_or_major_bin("378282246310005"));
    }

    #[test]
    fn validator_requires_luhn() {
        assert!(!eu_credit_card_ok("4973012345678905"));
    }

    #[test]
    fn validator_accepts_known_good() {
        assert!(eu_credit_card_ok("4973012345678908"));
    }

    #[test]
    fn context_cartes_bancaire_fr() {
        let r = eu_credit_card_recognizer();
        let v = r.analyze("carte bancaire 4973012345678908", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn min_len_13() {
        // 13 digits, invalid Luhn (prefix alone must not pass).
        assert!(!eu_credit_card_ok("4973012345677"));
    }

    #[test]
    fn max_len_19() {
        let d = "4".repeat(18) + "0";
        assert!(!eu_credit_card_ok(&d) || d.len() > 19);
    }

    #[test]
    fn recognizer_name() {
        assert_eq!(eu_credit_card_recognizer().name(), "eu_credit_card_bins");
    }

    #[test]
    fn langs_10() {
        assert_eq!(eu_credit_card_recognizer().supported_languages().len(), 10);
    }

    #[test]
    fn maestro_like_56_bin() {
        assert!(eu_mc_maestro_cartasi(5600, 560000));
    }

    #[test]
    fn digits_only_helper() {
        assert_eq!(digits_only("49 73 0123"), "49730123");
    }

    #[test]
    fn luhn_known_good() {
        assert!(luhn_valid("79927398713"));
    }

    #[test]
    fn cartasi_range() {
        assert!(eu_mc_maestro_cartasi(5300, 530012));
    }

    #[test]
    fn visa_4974xx_block() {
        assert!(eu_visa_bin(4974, 497412));
    }

    #[test]
    fn visa_4177_cb() {
        assert!(eu_visa_bin(4177, 417700));
    }

    #[test]
    fn reject_random_4_series_non_eu_hint() {
        let r = eu_credit_card_recognizer();
        assert!(r.analyze("4119119119119119", &cfg()).is_empty());
    }

    #[test]
    fn eu_credit_card_ok_with_dashes() {
        assert!(eu_credit_card_ok("4973-0123-4567-8908"));
    }

    #[test]
    fn entity_type_credit_card() {
        let t = eu_credit_card_recognizer().supported_entities();
        assert!(matches!(t[0], EntityType::CreditCard));
    }
}
