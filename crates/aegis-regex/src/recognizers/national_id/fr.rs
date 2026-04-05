// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! France: NIR, CNI (12 digits), passport, driving licence.

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// NIR: key `97 - (N % 97)` on the first 13 digits (N = integer from those digits).
pub fn fr_nir_validate(s: &str) -> bool {
    let d = digits_only(s);
    if d.len() != 15 {
        return false;
    }
    let n: u64 = match d[..13].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let k = 97 - (n % 97);
    let key = if k == 97 { 97u32 } else { k as u32 };
    d[13..].parse::<u32>().map(|g| g == key).unwrap_or(false)
}

/// CNI (historical 12-digit format): key = Σ d[i]×(i+1) for i=0..10, mod 10 (documented check scheme for synthetic tests).
pub fn fr_cni_12_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 12 {
        return false;
    }
    let sum: u32 = (0..11).map(|i| d[i] * (i as u32 + 1)).sum();
    d[11] == sum % 10
}

/// French passport: 2 letters + 7 digits; no unified public checksum — structural check + not all identical digits.
pub fn fr_passport_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let b = u.as_bytes();
    if u.len() != 9 {
        return false;
    }
    if !(b[0].is_ascii_alphabetic() && b[1].is_ascii_alphabetic()) {
        return false;
    }
    if !u[2..].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    u[2..].chars().any(|c| c != u.chars().nth(2).unwrap())
}

/// Licence (common post-reform format): 2 digits + 2 letters + 6 digits + 2 letters + 2 digits (14 chars).
pub fn fr_driver_license_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let re = Regex::new(r"^\d{2}[A-Z]{2}\d{6}[A-Z]{2}\d{2}$").unwrap();
    if !re.is_match(&u) {
        return false;
    }
    let dept: u32 = u[0..2].parse().unwrap_or(99);
    (1..=95).contains(&dept) || dept == 99
}

pub fn fr_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "numéro de sécurité sociale",
        "NIR",
        "sécurité sociale",
        "INSEE",
        "carte d'identité",
        "carte nationale",
        "CNI",
        "passeport",
        "permis de conduire",
        "permis",
        "national identity",
        "French ID",
    ];
    let rules = vec![
        IdRule {
            name: "fr_nir",
            re: Regex::new(
                r"(?xi)
                \b
                (?:NIR|n°\s*ss|numéro\s*de\s*sécurité\s*sociale|insee)[\s.:]+
                ([12]\d{2}[\s]?\d{2}[\s]?\d{2}[\s]?\d{3}[\s]?\d{3}[\s]?\d{2})
                \b
                |
                \b
                ([12]\d{14})
                \b
                ",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(fr_nir_validate),
            base_score: 0.9,
        },
        IdRule {
            name: "fr_cni_12",
            re: Regex::new(
                r"(?xi)
                \b
                (?:carte\s*d'identité|carte\s*nationale|CNI)[\s.:]+
                (\d{12})
                \b
                |
                \b
                (\d{12})
                (?=\s*(?:carte|CNI|identité))
                \b
                ",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(|m| {
                let d = digits_only(m);
                d.len() == 12 && fr_cni_12_validate(&d)
            }),
            base_score: 0.86,
        },
        IdRule {
            name: "fr_passport",
            re: Regex::new(
                r"(?xi)
                \b
                (?:passeport|passport)[\s.:]+
                ([A-Za-z]{2}\d{7})
                \b
                |
                \b
                ([A-Za-z]{2}\d{7})
                (?=\s*passeport)
                \b
                ",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::Passport,
            validator: Arc::new(|m| {
                let u: String = m.chars().filter(|c| !c.is_whitespace()).collect();
                fr_passport_validate(&u)
            }),
            base_score: 0.84,
        },
        IdRule {
            name: "fr_driver_license",
            re: Regex::new(
                r"(?xi)
                \b
                (?:permis\s*de\s*conduire|permis)[\s.:]+
                (\d{2}[A-Za-z]{2}\d{6}[A-Za-z]{2}\d{2})
                \b
                |
                \b
                (\d{2}[A-Za-z]{2}\d{6}[A-Za-z]{2}\d{2})
                (?=\s*permis)
                \b
                ",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::DriverLicense,
            validator: Arc::new(fr_driver_license_validate),
            base_score: 0.83,
        },
    ];
    CompositeNationalRecognizer::new("fr_national_identity", rules, vec!["fr", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recognizer;
    use aegis_core::config::AnalysisConfig;

    fn synth_nir() -> String {
        let base = "1860244221149";
        let n: u64 = base.parse().unwrap();
        let k = 97 - (n % 97);
        format!("{base}{k:02}")
    }

    fn synth_cni() -> String {
        let body: Vec<u32> = (0..11).map(|i| ((i * 7 + 3) % 10) as u32).collect();
        let sum: u32 = body
            .iter()
            .enumerate()
            .map(|(i, &d)| d * (i as u32 + 1))
            .sum();
        let check = sum % 10;
        let mut s: String = body
            .iter()
            .map(|d| char::from_digit(*d, 10).unwrap())
            .collect();
        s.push(char::from_digit(check, 10).unwrap());
        s
    }

    #[test]
    fn nir_synthetic_valid() {
        let n = synth_nir();
        assert!(fr_nir_validate(&n));
    }

    #[test]
    fn nir_synthetic_invalid_key() {
        let n = synth_nir();
        let mut c: Vec<char> = n.chars().collect();
        let last = c.pop().unwrap();
        let wrong = if last == '0' { '1' } else { '0' };
        c.push(wrong);
        assert!(!fr_nir_validate(&c.iter().collect::<String>()));
    }

    #[test]
    fn cni_synthetic_valid() {
        let c = synth_cni();
        assert_eq!(c.len(), 12);
        assert!(fr_cni_12_validate(&c));
    }

    #[test]
    fn cni_synthetic_invalid() {
        let mut c = synth_cni();
        unsafe_replace_last_char(&mut c);
        assert!(!fr_cni_12_validate(&c));
    }

    fn unsafe_replace_last_char(s: &mut String) {
        let b = s.as_bytes();
        let last = *b.last().unwrap();
        let w = if last == b'0' { '1' } else { '0' };
        s.pop();
        s.push(w);
    }

    #[test]
    fn passport_synthetic() {
        assert!(fr_passport_validate("AB1234567"));
        assert!(!fr_passport_validate("AB1111111"));
        assert!(!fr_passport_validate("A12345678"));
    }

    #[test]
    fn driver_license_synthetic() {
        assert!(fr_driver_license_validate("75AB123456CD01"));
        assert!(!fr_driver_license_validate("00AB123456CD01"));
    }

    #[test]
    fn recognizer_finds_nir() {
        let r = fr_national_id_recognizer();
        let n = synth_nir();
        let v = r.analyze(&format!("NIR {n}"), &AnalysisConfig::default());
        assert!(!v.is_empty());
    }

    #[test]
    fn recognizer_finds_cni() {
        let r = fr_national_id_recognizer();
        let c = synth_cni();
        let v = r.analyze(&format!("carte d'identité {c}"), &AnalysisConfig::default());
        assert!(!v.is_empty());
    }
}
