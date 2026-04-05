// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! BIC / SWIFT — 8 or 11 characters, SWIFT structure.

use super::common::bic_swift_context_words;
use crate::pattern::PatternRecognizer;
use aegis_core::entity::EntityType;
use regex::Regex;
use std::sync::Arc;

/// Common ISO 3166-1 alpha-2 country codes to filter random English words (Europe + major partners).
pub fn bic_country_plausible(s: &str) -> bool {
    let u: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let u = u.to_ascii_uppercase();
    if u.len() < 6 {
        return false;
    }
    let cc = &u[4..6];
    matches!(
        cc,
        "AD" | "AE" | "AT" | "AU" | "BA" | "BE" | "BG" | "BH" | "BR" | "BY" | "CH" | "CN"
            | "CY" | "CZ" | "DE" | "DK" | "EE" | "EG" | "ES" | "FI" | "FO" | "FR" | "GB" | "GE"
            | "GI" | "GL" | "GR" | "GT" | "HK" | "HR" | "HU" | "IE" | "IL" | "IN" | "IQ" | "IS"
            | "IT" | "JO" | "JP" | "KW" | "KZ" | "LB" | "LI" | "LT" | "LU" | "LV" | "MC" | "MD"
            | "ME" | "MK" | "MT" | "MU" | "MX" | "NL" | "NO" | "NZ" | "OM" | "PL" | "PS" | "PT"
            | "QA" | "RO" | "RS" | "SA" | "SE" | "SG" | "SI" | "SK" | "SM" | "TR" | "UA" | "US"
            | "VA" | "VG" | "XK"
    )
}

/// Valide la structure : 4 lettres banque, 2 lettres pays, 2 alnum localisation, 3 alnum branche optionnels.
pub fn bic_structure_ok(s: &str) -> bool {
    let u: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let u = u.to_ascii_uppercase();
    if !(u.len() == 8 || u.len() == 11) {
        return false;
    }
    let b = u.as_bytes();
    for i in 0..4 {
        if !b[i].is_ascii_alphabetic() {
            return false;
        }
    }
    for i in 4..6 {
        if !b[i].is_ascii_alphabetic() {
            return false;
        }
    }
    for i in 6..8 {
        if !b[i].is_ascii_alphanumeric() {
            return false;
        }
    }
    if u.len() == 11 {
        for i in 8..11 {
            if !b[i].is_ascii_alphanumeric() {
                return false;
            }
        }
    }
    true
}

pub fn bic_swift_recognizer() -> PatternRecognizer {
    let re = Regex::new(
        r"(?xi)
        \b
        ([A-Za-z]{4}[A-Za-z]{2}[A-Za-z0-9]{2}(?:[A-Za-z0-9]{3})?)
        \b
        ",
    )
    .expect("bic regex");
    let ctx = bic_swift_context_words();
    PatternRecognizer::new(
        "bic_swift",
        re,
        EntityType::BankAccount,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"],
        0.88,
    )
    .with_validator(Arc::new(|m| bic_structure_ok(m) && bic_country_plausible(m)))
    .with_min_score(0.42)
    .with_context_boost_words(&ctx, 0.07)
    .with_context_penalty_words(&["example", "testbank", "XXXX", "dummy"], 0.15)
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
    fn structure_ok_8() {
        assert!(bic_structure_ok("DEUTDEFF"));
    }

    #[test]
    fn structure_ok_11() {
        assert!(bic_structure_ok("DEUTDEFF500"));
    }

    #[test]
    fn structure_rejects_digit_in_bank() {
        assert!(!bic_structure_ok("DEU1DEFF"));
    }

    #[test]
    fn structure_rejects_short() {
        assert!(!bic_structure_ok("DEUTDE"));
    }

    #[test]
    fn structure_rejects_10() {
        assert!(!bic_structure_ok("DEUTDEFF50"));
    }

    #[test]
    fn structure_country_must_be_alpha() {
        assert!(!bic_structure_ok("DEUT12FF"));
    }

    #[test]
    fn bnpafrpp() {
        let r = bic_swift_recognizer();
        let v = r.analyze("BIC BNPAFRPP context", &cfg());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].text, "BNPAFRPP");
    }

    #[test]
    fn swift_deutsche_11() {
        let r = bic_swift_recognizer();
        let v = r.analyze("SWIFT DEUTDEFF500", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn spaced_swift_contiguous_code_only() {
        let r = bic_swift_recognizer();
        let v = r.analyze("code SWIFT: DEUTDEFF500", &cfg());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].text, "DEUTDEFF500");
    }

    #[test]
    fn recognize_with_label() {
        let r = bic_swift_recognizer();
        let v = r.analyze("BIC: NBBEBEBB", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn ingbnl2a() {
        assert!(bic_structure_ok("INGBNL2A"));
    }

    #[test]
    fn unicredit_it() {
        assert!(bic_structure_ok("UNCRITMM"));
    }

    #[test]
    fn invalid_location_non_alnum() {
        assert!(!bic_structure_ok("DEUTDE@F"));
    }

    #[test]
    fn lowercase_normalized_in_validator() {
        assert!(bic_structure_ok("deutdeff"));
    }

    #[test]
    fn branch_xxx_main_office() {
        assert!(bic_structure_ok("BNPAFRPPXXX"));
    }

    #[test]
    fn portuguese_bank_sample() {
        assert!(bic_structure_ok("BCOMPTPL"));
    }

    #[test]
    fn polish_bank_sample() {
        assert!(bic_structure_ok("BPKOPLPW"));
    }

    #[test]
    fn romanian_bank_sample() {
        assert!(bic_structure_ok("RNCBROBU"));
    }

    #[test]
    fn swedish_bank_sample() {
        assert!(bic_structure_ok("HANDSESS"));
    }

    #[test]
    fn false_positive_random_word() {
        let r = bic_swift_recognizer();
        assert!(r.analyze("ABCDEFGH not a bic", &cfg()).is_empty());
    }

    #[test]
    fn context_boost_swift_keyword() {
        let r = bic_swift_recognizer();
        let v = r.analyze("SWIFT DEUTDEFF", &cfg());
        assert!(!v.is_empty());
        assert!(v[0].score > 0.88);
    }

    #[test]
    fn recognizer_name() {
        let r = bic_swift_recognizer();
        assert_eq!(r.name(), "bic_swift");
    }

    #[test]
    fn supported_langs_10() {
        let r = bic_swift_recognizer();
        assert_eq!(r.supported_languages().len(), 10);
    }

    #[test]
    fn entity_bank_account() {
        let r = bic_swift_recognizer();
        assert!(matches!(
            r.supported_entities()[0],
            EntityType::BankAccount
        ));
    }
}
