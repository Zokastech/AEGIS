// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! EU postal-address heuristics: number, street, postal code, locality.

use crate::pattern::PatternRecognizer;
use aegis_core::entity::EntityType;
use regex::Regex;
use std::sync::{Arc, OnceLock};

/// Street number prefix: number, bis/ter/quater, optional letter (e.g. 12 bis, 3A).
const HOUSE_PREFIX: &str = r#"(\d{1,4})(?:\s+(?:bis|ter|quater))?[A-Za-z]?"#;

fn address_multiline_body() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| {
        format!(
            r#"
            \b
            {HOUSE_PREFIX}
            \s+
            ([^\n]{{4,120}})
            \s*\r?\n\s*
            (\d{{5}})\s+
            ([A-Za-zÀ-ÿ][A-Za-zÀ-ÿ\-'. ]{{1,45}})
            (?:\r?\n\s*France\b)?
            "#,
            HOUSE_PREFIX = HOUSE_PREFIX
        )
    })
    .as_str()
}

fn address_single_line_body() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| {
        format!(
            r#"
            \b
            {HOUSE_PREFIX}
            \s+
            ([\w'\.\-,\s]{{3,100}}?)
            (?:\s*,\s*(?:apt|appt|appart\.?|porte)\s*[A-Za-z0-9.\s/-]{{1,24}})?
            \s*,?\s+
            (
                \d{{5}}
                |
                \d{{4}}\s*[A-Za-z]{{2}}
                |
                [A-Za-z]{{1,2}}\d[A-Za-z0-9]?\s*\d[A-Za-z]{{2}}
                |
                \d{{2}}-\d{{3}}
                |
                \d{{3}}\s?\d{{2}}
            )
            \s+
            ([A-Za-zÀ-ÿ][A-Za-zÀ-ÿ\-'\.\s]{{1,45}})
            \b
            "#,
            HOUSE_PREFIX = HOUSE_PREFIX
        )
    })
    .as_str()
}

fn address_combined_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let ml = address_multiline_body();
        let sl = address_single_line_body();
        Regex::new(&format!(r"(?xi)(?:{ml})|(?:{sl})")).expect("eu address combined")
    })
}

/// Valide le bloc code postal selon des formats nationaux courants (UE + UK).
pub fn eu_postal_format_ok(pc: &str) -> bool {
    let u = pc.trim().to_ascii_uppercase().replace(' ', "");
    let u_spaced = pc.trim().to_ascii_uppercase();
    static D5: OnceLock<Regex> = OnceLock::new();
    static NL: OnceLock<Regex> = OnceLock::new();
    static UK: OnceLock<Regex> = OnceLock::new();
    static PL: OnceLock<Regex> = OnceLock::new();
    static CZ: OnceLock<Regex> = OnceLock::new();
    D5.get_or_init(|| Regex::new(r"^\d{5}$").unwrap());
    NL.get_or_init(|| Regex::new(r"^\d{4}[A-Z]{2}$").unwrap());
    UK.get_or_init(|| Regex::new(r"^[A-Z]{1,2}\d[A-Z0-9]?\d[A-Z]{2}$").unwrap());
    PL.get_or_init(|| Regex::new(r"^\d{2}-\d{3}$").unwrap());
    CZ.get_or_init(|| Regex::new(r"^\d{3}\s?\d{2}$").unwrap());
    D5.get().unwrap().is_match(&u)
        || NL.get().unwrap().is_match(&u)
        || UK.get().unwrap().is_match(&u)
        || PL.get().unwrap().is_match(&u)
        || CZ.get().unwrap().is_match(&u_spaced)
}

fn valid_address_caps(c: &regex::Captures<'_>) -> bool {
    let street = match c.get(2) {
        Some(m) => m.as_str().trim(),
        None => return false,
    };
    let pc = match c.get(3) {
        Some(m) => m.as_str().trim(),
        None => return false,
    };
    street.len() >= 3 && eu_postal_format_ok(pc)
}

fn address_ml_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(r"(?xi)(?:{})", address_multiline_body())).expect("eu address ml")
    })
}

fn address_sl_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(r"(?xi)(?:{})", address_single_line_body())).expect("eu address sl")
    })
}

fn validate_full_line(s: &str) -> bool {
    if let Some(c) = address_ml_regex().captures(s) {
        if valid_address_caps(&c) {
            return true;
        }
    }
    if let Some(c) = address_sl_regex().captures(s) {
        if valid_address_caps(&c) {
            return true;
        }
    }
    false
}

pub fn eu_address_recognizer() -> PatternRecognizer {
    let ctx = [
        "adresse",
        "Adresse",
        "address",
        "Anschrift",
        "indirizzo",
        "dirección",
        "direccion",
        "adres",
        "morada",
        "rue",
        "avenue",
        "boulevard",
        "Straße",
        "Strasse",
        "via",
        "viale",
        "calle",
        "plaats",
        "woonplaats",
        "CAP",
        "code postal",
        "postcode",
        "ZIP",
        "PLZ",
    ];
    PatternRecognizer::new(
        "eu_postal_address",
        address_combined_regex().clone(),
        EntityType::Address,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.62,
    )
    .with_validator(Arc::new(validate_full_line))
    .with_min_score(0.35)
    .with_context_boost_words(&ctx, 0.08)
    .with_context_penalty_words(&["example.com", "test", "dummy"], 0.12)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recognizer;
    use aegis_core::config::AnalysisConfig;

    #[test]
    fn postal_formats() {
        assert!(eu_postal_format_ok("75001"));
        assert!(eu_postal_format_ok("10115"));
        assert!(eu_postal_format_ok("1234 AB"));
        assert!(eu_postal_format_ok("SW1A1AA"));
        assert!(eu_postal_format_ok("00-950"));
        assert!(!eu_postal_format_ok("12"));
    }

    #[test]
    fn line_fr() {
        assert!(validate_full_line("12 rue de Rivoli 75001 Paris"));
    }

    #[test]
    fn fr_multiline_bis_apt_roubaix() {
        let r = eu_address_recognizer();
        let t = "12 bis rue des Fleurs, apt 3B\n59100 Roubaix";
        let v = r.analyze(t, &AnalysisConfig::default());
        assert!(!v.is_empty(), "{v:?}");
    }

    #[test]
    fn fr_multiline_avenue_lille() {
        let r = eu_address_recognizer();
        let t = "45 avenue J. Jaurès\n59000 Lille";
        let v = r.analyze(t, &AnalysisConfig::default());
        assert!(!v.is_empty(), "{v:?}");
    }

    #[test]
    fn fr_multiline_with_france_line() {
        let r = eu_address_recognizer();
        let t = "8 rue Pasteur\n75013 Paris\nFrance";
        let v = r.analyze(t, &AnalysisConfig::default());
        assert!(!v.is_empty(), "{v:?}");
    }
}
