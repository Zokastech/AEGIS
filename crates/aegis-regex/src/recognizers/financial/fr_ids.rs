// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! French business identifiers and NIR: SIREN, SIRET, national social security number.

use super::common::fr_id_context_words;
use crate::pattern::PatternRecognizer;
use crate::validation::luhn_valid;
use aegis_core::entity::EntityType;
use regex::Regex;
use std::sync::{Arc, OnceLock};

fn digits_compact(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
fn siren_with_check_digit(base8: &str) -> String {
    assert_eq!(base8.len(), 8);
    for check in 0..10u8 {
        let s = format!("{base8}{check}");
        if luhn_valid(&s) {
            return s;
        }
    }
    panic!("pas de chiffre de contrôle Luhn pour {base8}");
}

#[cfg(test)]
fn siret_luhn_valid_for_siren(siren: &str) -> String {
    assert_eq!(siren.len(), 9);
    for nic in 0u32..100_000 {
        let s = format!("{siren}{nic:05}");
        if luhn_valid(&s) {
            return s;
        }
    }
    panic!("aucun NIC Luhn pour SIREN {siren}");
}

/// SIREN: 9 digits, Luhn check digit.
pub fn siren_luhn_ok(s: &str) -> bool {
    let d = digits_compact(s);
    d.len() == 9 && luhn_valid(&d)
}

/// SIRET: 14 digits (SIREN + NIC), Luhn on the full number.
pub fn siret_luhn_ok(s: &str) -> bool {
    let d = digits_compact(s);
    d.len() == 14 && luhn_valid(&d)
}

/// NIR (social security no.): 15 digits, key `97 - (N % 97)` on the first 13.
pub fn nir_key_ok(s: &str) -> bool {
    let d = digits_compact(s);
    if d.len() != 15 {
        return false;
    }
    let n: u64 = match d[..13].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let key_computed = 97 - (n % 97);
    let key_expected = if key_computed == 97 {
        97u32
    } else {
        key_computed as u32
    };
    let key_given: u32 = d[13..].parse().unwrap_or(999);
    key_given == key_expected
}

/// Plausible NIR shape (15 digits, sex, month, department) without key verification.
pub fn nir_shape_ok(s: &str) -> bool {
    let d = digits_compact(s);
    if d.len() != 15 {
        return false;
    }
    if !matches!(d.as_bytes().first(), Some(b'1' | b'2')) {
        return false;
    }
    let mm: u32 = d[3..5].parse().unwrap_or(99);
    if !((1..=12).contains(&mm) || (20..=99).contains(&mm)) {
        return false;
    }
    let dept: u32 = d[5..7].parse().unwrap_or(0);
    dept != 0
}

fn nir_uncertain_phrase_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?xi)
            \b(?:serait|seraient|possible|probable|probablement|environ|approximativement|
                aurait|auraient|supposé|non[\s-]+confirmé|non[\s-]+vérifié|invérifiable)\b",
        )
        .expect("nir uncertain phrase")
    })
}

/// If the fragment has uncertainty phrasing ("would be", "possible", …), accept a NIR with correct
/// shape but no INSEE key; otherwise require [`nir_key_ok`].
pub fn nir_match_validate(m: &str) -> bool {
    let d = digits_compact(m);
    if d.len() != 15 {
        return false;
    }
    if nir_uncertain_phrase_re().is_match(m) {
        nir_shape_ok(m)
    } else {
        nir_key_ok(m)
    }
}

pub fn siren_recognizer() -> PatternRecognizer {
    let re = Regex::new(
        r"(?xi)
        \b
        (?:SIREN|n°\s*siren)\s*[.:]?\s*
        (\d{3}[\s.]?\d{3}[\s.]?\d{3})
        \b
        ",
    )
    .expect("siren regex");
    let ctx = fr_id_context_words();
    PatternRecognizer::new(
        "fr_siren",
        re,
        EntityType::TaxId,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"],
        0.9,
    )
    .with_validator(Arc::new(|m| {
        let d = digits_compact(m);
        d.len() == 9 && siren_luhn_ok(&d)
    }))
    .with_min_score(0.45)
    .with_context_boost_words(&ctx, 0.07)
}

pub fn siret_recognizer() -> PatternRecognizer {
    let re = Regex::new(
        r"(?xi)
        \b
        (?:SIRET|siret)
        (?:\s+\w+){0,6}
        \s*[.:]\s*
        (\d{3}[\s.]?\d{3}[\s.]?\d{3}[\s.]?\d{5})
        \b
        |
        \b
        (?:SIRET|siret)(?:\s|[.:])+
        (\d{3}[\s.]?\d{3}[\s.]?\d{3}[\s.]?\d{5})
        \b
        |
        \b
        (\d{14})
        \b
        ",
    )
    .expect("siret regex");
    let ctx = fr_id_context_words();
    PatternRecognizer::new(
        "fr_siret",
        re,
        EntityType::TaxId,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"],
        0.92,
    )
    .with_validator(Arc::new(siret_luhn_ok))
    .with_min_score(0.45)
    .with_context_boost_words(&ctx, 0.07)
}

pub fn nir_recognizer() -> PatternRecognizer {
    let re = Regex::new(
        r"(?xi)
        \b
        (?:NIR|n°\s*ss|numéro\s*de\s*sécurité\s*sociale|insee)
        (?:\s+[[:alpha:]][\w'-]*){0,14}
        \s*[.:]?\s*
        ([12]\s*\d{2}\s*\d{2}\s*\d{2}\s*\d{3}\s*\d{3}\s*\d{2})
        \b
        |
        \b
        ([12]\d{14})
        \b
        ",
    )
    .expect("nir regex");
    let ctx = fr_id_context_words();
    PatternRecognizer::new(
        "fr_nir",
        re,
        EntityType::NationalId,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"],
        0.88,
    )
    .with_validator(Arc::new(nir_match_validate))
    .with_min_score(0.48)
    .with_context_boost_words(&ctx, 0.07)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Recognizer;
    #[test]
    fn siren_luhn_valid_constructed() {
        let s = siren_with_check_digit("55203253");
        assert!(siren_luhn_ok(&s));
    }

    #[test]
    fn siren_invalid_luhn() {
        assert!(!siren_luhn_ok("552032539"));
    }

    #[test]
    fn siret_luhn_valid_constructed() {
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        assert_eq!(siret.len(), 14);
        assert!(siret_luhn_ok(&siret));
    }

    #[test]
    fn siret_invalid_flip_last() {
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        let mut bad = siret.chars().collect::<Vec<_>>();
        let last = bad.pop().unwrap();
        let wrong = if last == '0' { '1' } else { '0' };
        bad.push(wrong);
        assert!(!siret_luhn_ok(&bad.iter().collect::<String>()));
    }

    #[test]
    fn nir_key_constructed() {
        let base = "1850760050101";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        let full = format!("{base}{k:02}");
        assert_eq!(full.len(), 15);
        assert!(nir_key_ok(&full));
    }

    #[test]
    fn nir_wrong_key() {
        assert!(!nir_key_ok("185076005010199"));
    }

    #[test]
    fn nir_too_short() {
        assert!(!nir_key_ok("1850760050101"));
    }

    #[test]
    fn digits_compact_spaces() {
        assert_eq!(digits_compact("732 829 320"), "732829320");
    }

    #[test]
    fn siren_recognizer_finds_labeled() {
        use aegis_core::config::AnalysisConfig;
        let s = siren_with_check_digit("73282932");
        let r = siren_recognizer();
        let v = r.analyze(&format!("SIREN {s}"), &AnalysisConfig::default());
        assert!(!v.is_empty());
    }

    #[test]
    fn siret_recognizer_14_plain() {
        use aegis_core::config::AnalysisConfig;
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        let r = siret_recognizer();
        let v = r.analyze(&siret, &AnalysisConfig::default());
        assert!(!v.is_empty());
    }

    #[test]
    fn nir_recognizer_with_label() {
        use aegis_core::config::AnalysisConfig;
        let r = nir_recognizer();
        let base = "1850760050101";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        let num = format!("{base}{k:02}");
        let v = r.analyze(&format!("NIR {num}"), &AnalysisConfig::default());
        assert!(!v.is_empty());
    }

    #[test]
    fn nir_spaced_fr_format_valid_key() {
        use aegis_core::config::AnalysisConfig;
        let r = nir_recognizer();
        // 1 85 05 75 806 043 + key 75 (first 13 digits → INSEE key)
        let s = "NIR 1 85 05 75 806 043 75";
        assert!(nir_key_ok(s));
        let v = r.analyze(s, &AnalysisConfig::default());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn nir_numero_securite_serait_shape_without_valid_key() {
        use aegis_core::config::AnalysisConfig;
        let r = nir_recognizer();
        let s = "Son numéro de sécurité sociale serait 1 87 02 59 123 456 78";
        assert!(!nir_key_ok(s));
        assert!(nir_shape_ok(s));
        let v = r.analyze(s, &AnalysisConfig::default());
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn nir_invalid_key_rejected_when_no_uncertain_phrase() {
        use aegis_core::config::AnalysisConfig;
        let r = nir_recognizer();
        let s = "NIR 1 87 02 59 123 456 78";
        assert!(r.analyze(s, &AnalysisConfig::default()).is_empty());
    }

    #[test]
    fn siren_rejects_bad_checksum() {
        use aegis_core::config::AnalysisConfig;
        let r = siren_recognizer();
        assert!(r
            .analyze("SIREN 732 829 329", &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siret_spaced() {
        use aegis_core::config::AnalysisConfig;
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        let spaced = format!(
            "{} {} {} {}",
            &siret[0..3],
            &siret[3..6],
            &siret[6..9],
            &siret[9..14]
        );
        let r = siret_recognizer();
        let v = r.analyze(&format!("SIRET {spaced}"), &AnalysisConfig::default());
        assert!(!v.is_empty());
    }

    #[test]
    fn siret_possible_colon_natural_language() {
        use aegis_core::config::AnalysisConfig;
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        let spaced = format!(
            "{} {} {} {}",
            &siret[0..3],
            &siret[3..6],
            &siret[6..9],
            &siret[9..14]
        );
        let r = siret_recognizer();
        let phrase = format!(r#"« KM Data » (SIRET possible : {spaced} — non confirmé)"#);
        let v = r.analyze(&phrase, &AnalysisConfig::default());
        assert!(!v.is_empty(), "expected SIRET in {phrase}");
    }

    #[test]
    fn nir_15_compact() {
        use aegis_core::config::AnalysisConfig;
        let r = nir_recognizer();
        let base = "2850510040054";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        let num = format!("{base}{k:02}");
        if num.len() == 15 {
            let v = r.analyze(&num, &AnalysisConfig::default());
            assert!(!v.is_empty() || !nir_key_ok(&num));
        }
    }

    #[test]
    fn luhn_edge_single() {
        assert!(!luhn_valid("1"));
    }

    #[test]
    fn siren_all_zeros_invalid() {
        assert!(!siren_luhn_ok("000000000"));
    }

    #[test]
    fn siret_wrong_length() {
        assert!(!siret_luhn_ok("7328293200007"));
    }

    #[test]
    fn nir_starts_with_3_rejected() {
        assert!(!nir_key_ok("385076005010147"));
    }

    #[test]
    fn recognizer_names() {
        assert_eq!(siren_recognizer().name(), "fr_siren");
        assert_eq!(siret_recognizer().name(), "fr_siret");
        assert_eq!(nir_recognizer().name(), "fr_nir");
    }

    #[test]
    fn langs_10_each() {
        assert_eq!(siren_recognizer().supported_languages().len(), 10);
        assert_eq!(siret_recognizer().supported_languages().len(), 10);
        assert_eq!(nir_recognizer().supported_languages().len(), 10);
    }

    #[test]
    fn siren_entity_tax_id() {
        assert!(matches!(
            siren_recognizer().supported_entities()[0],
            EntityType::TaxId
        ));
    }

    #[test]
    fn nir_entity_national_id() {
        assert!(matches!(
            nir_recognizer().supported_entities()[0],
            EntityType::NationalId
        ));
    }

    #[test]
    fn siret_entity_tax_id() {
        assert!(matches!(
            siret_recognizer().supported_entities()[0],
            EntityType::TaxId
        ));
    }

    #[test]
    fn compact_siret_dots() {
        let siren = siren_with_check_digit("73282932");
        let siret = siret_luhn_valid_for_siren(&siren);
        let dotted = format!(
            "{}.{}.{}.{}",
            &siret[0..3],
            &siret[3..6],
            &siret[6..9],
            &siret[9..14]
        );
        assert!(siret_luhn_ok(&dotted));
    }

    #[test]
    fn nir_math_consistency() {
        for base in ["1000000000001", "2000000000002"] {
            let n: u64 = base.parse().unwrap();
            let k = 97 - (n % 97);
            let full = format!("{base}{k:02}");
            assert!(nir_key_ok(&full), "{full}");
        }
    }

    // SIREN coverage (dedicated recognizer)
    #[test]
    fn siren_label_n_degree() {
        use aegis_core::config::AnalysisConfig;
        let s = siren_with_check_digit("41106160");
        let r = siren_recognizer();
        assert!(!r
            .analyze(&format!("n° siren: {s}"), &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siren_dots_in_number() {
        use aegis_core::config::AnalysisConfig;
        let s = siren_with_check_digit("44306184");
        let dotted = format!("{}.{}.{}", &s[0..3], &s[3..6], &s[6..9]);
        let r = siren_recognizer();
        assert!(!r
            .analyze(&format!("SIREN {dotted}"), &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siren_no_label_no_match() {
        use aegis_core::config::AnalysisConfig;
        let s = siren_with_check_digit("55203253");
        let r = siren_recognizer();
        assert!(r.analyze(&s, &AnalysisConfig::default()).is_empty());
    }

    #[test]
    fn siren_case_insensitive_label() {
        use aegis_core::config::AnalysisConfig;
        let s = siren_with_check_digit("30967458");
        let r = siren_recognizer();
        assert!(!r
            .analyze(&format!("siren {s}"), &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siren_min_score_trait() {
        assert!(siren_recognizer().min_score() <= 0.5);
    }

    #[test]
    fn siren_two_on_line() {
        use aegis_core::config::AnalysisConfig;
        let a = siren_with_check_digit("10000000");
        let b = siren_with_check_digit("20000000");
        let r = siren_recognizer();
        let hits = r.analyze(
            &format!("SIREN {a} ou SIREN {b}"),
            &AnalysisConfig::default(),
        );
        assert!(hits.len() >= 2);
    }

    // Couverture SIRET
    #[test]
    fn siret_plain_14_must_pass_luhn() {
        use aegis_core::config::AnalysisConfig;
        let siren = siren_with_check_digit("12345678");
        let st = siret_luhn_valid_for_siren(&siren);
        assert!(
            siret_recognizer()
                .analyze(&st, &AnalysisConfig::default())
                .len()
                == 1
        );
    }

    #[test]
    fn siret_not_match_13_digits() {
        use aegis_core::config::AnalysisConfig;
        assert!(siret_recognizer()
            .analyze("1234567890123", &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siret_label_uppercase() {
        use aegis_core::config::AnalysisConfig;
        let st = siret_luhn_valid_for_siren(&siren_with_check_digit("99887766"));
        assert!(!siret_recognizer()
            .analyze(&format!("SIRET {st}"), &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn siret_min_score() {
        assert!(siret_recognizer().min_score() < 0.5);
    }

    #[test]
    fn siret_metadata_level_regex() {
        use aegis_core::config::AnalysisConfig;
        let mut c = AnalysisConfig::default();
        c.return_decision_process = true;
        let st = siret_luhn_valid_for_siren(&siren_with_check_digit("88776655"));
        let e = &siret_recognizer().analyze(&st, &c)[0];
        assert_eq!(e.metadata.get("level"), Some(&"regex".into()));
    }

    // Couverture NIR
    #[test]
    fn nir_compact_15() {
        use aegis_core::config::AnalysisConfig;
        let base = "1860244221149";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        let num = format!("{base}{k:02}");
        assert!(!nir_recognizer()
            .analyze(&num, &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn nir_label_insee() {
        use aegis_core::config::AnalysisConfig;
        let base = "1860244221149";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        let num = format!("{base}{k:02}");
        assert!(!nir_recognizer()
            .analyze(&format!("INSEE {num}"), &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn nir_wrong_length_14() {
        use aegis_core::config::AnalysisConfig;
        assert!(nir_recognizer()
            .analyze("18602442211490", &AnalysisConfig::default())
            .is_empty());
    }

    #[test]
    fn nir_min_score() {
        assert!(nir_recognizer().min_score() < 0.5);
    }

    #[test]
    fn nir_supported_entity() {
        assert!(matches!(
            nir_recognizer().supported_entities()[0],
            EntityType::NationalId
        ));
    }
}
