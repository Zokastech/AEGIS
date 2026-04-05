// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Germany: Personalausweis (VDV / check series), tax ID, passport, social insurance number.

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// Steuer-Identifikationsnummer (11 digits) — BZSt algorithm (product / 10, double, etc.).
pub fn de_steuer_id_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 || d[0] == 0 {
        return false;
    }
    let mut p = 10u32;
    for i in 0..10 {
        let sum = (d[i] + p) % 10;
        let s = if sum == 0 { 10 } else { sum };
        p = (s * 2) % 11;
    }
    let c = (11 - p) % 10;
    c == d[10]
}

fn de_pa_char_value(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as u32),
        b'A'..=b'Z' => Some((c - b'A') as u32 + 10),
        b'a'..=b'z' => Some((c - b'a') as u32 + 10),
        _ => None,
    }
}

fn de_pa_weighted_sum_digit(n: u32, w: u32) -> u32 {
    let p = n * w;
    (p / 10) + (p % 10)
}

/// Personalausweis: 9 alphanumeric chars; 9th is check digit (weights 7,3,1 — VDV / card-read logic).
pub fn de_personalausweis_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    if u.len() != 9 || !u.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return false;
    }
    let w = [7u32, 3, 1, 7, 3, 1, 7, 3];
    let mut sum = 0u32;
    for i in 0..8 {
        let v = match de_pa_char_value(u.as_bytes()[i]) {
            Some(x) => x,
            None => return false,
        };
        if v < 10 {
            sum += de_pa_weighted_sum_digit(v, w[i]);
        } else {
            sum += de_pa_weighted_sum_digit(v / 10, w[i]);
            sum += de_pa_weighted_sum_digit(v % 10, w[i]);
        }
    }
    let check = (10 - (sum % 10)) % 10;
    u.as_bytes()[8].is_ascii_digit() && (u.as_bytes()[8] - b'0') as u32 == check
}

/// German passport: one allowed letter + 8 digits (often C + 8 digits).
pub fn de_reisepass_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let re = Regex::new(r"^[CFGHJKLMNPRTVWXYZ]\d{8}$").unwrap();
    re.is_match(&u)
}

/// German social insurance number: 12 digits + one letter A–Y (not **S**) + 3 digits; check on 15 digits (weights 2,1,2,5,7,…).
pub fn de_sozialversicherung_validate(s: &str) -> bool {
    let u = s
        .to_ascii_uppercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let re = Regex::new(r"^(\d{12})([A-Y])(\d{3})$").unwrap();
    let Some(cap) = re.captures(&u) else {
        return false;
    };
    if cap[2].starts_with('S') {
        return false;
    }
    let digits: Vec<u32> = cap[1]
        .chars()
        .chain(cap[3].chars())
        .filter_map(|c| c.to_digit(10))
        .collect();
    if digits.len() != 15 {
        return false;
    }
    let w = [2u32, 1, 2, 5, 7, 1, 2, 1, 2, 5, 7, 1, 2, 1, 2];
    let mut sum = 0u32;
    for (i, &d) in digits.iter().enumerate() {
        let mut p = d * w[i];
        if p > 9 {
            p = (p / 10) + (p % 10);
        }
        sum += p;
    }
    sum.is_multiple_of(10)
}

pub fn de_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "Personalausweis",
        "Ausweis",
        "Steuer-ID",
        "Steueridentifikationsnummer",
        "Reisepass",
        "Pass",
        "Sozialversicherungsnummer",
        "SV-Nummer",
        "Rentenversicherungsnummer",
        "German ID",
        "tax number",
    ];
    let rules = vec![
        IdRule {
            name: "de_steuer_id",
            re: Regex::new(r"(?xi)\b(?:Steuer[- ]?ID|Steueridentifikationsnummer)[\s.:]+(\d{11})\b|\b(\d{11})(?=\s*(?:Steuer|tax))").unwrap(),
            entity: aegis_core::entity::EntityType::TaxId,
            validator: Arc::new(de_steuer_id_validate),
            base_score: 0.91,
        },
        IdRule {
            name: "de_personalausweis",
            re: Regex::new(r"(?xi)\b(?:Personalausweis|Ausweis)[\s.:]+([A-Za-z0-9]{9})\b|\b([A-Za-z0-9]{9})(?=\s*Ausweis)").unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(de_personalausweis_validate),
            base_score: 0.88,
        },
        IdRule {
            name: "de_reisepass",
            re: Regex::new(r"(?xi)\b(?:Reisepass|Pass|Passport)[\s.:]+([CFGHJKLMNPRTVWXYZcfghjklmnprtvwxyz]\d{8})\b").unwrap(),
            entity: aegis_core::entity::EntityType::Passport,
            validator: Arc::new(de_reisepass_validate),
            base_score: 0.85,
        },
        IdRule {
            name: "de_sozialversicherung",
            re: Regex::new(
                r"(?xi)\b(?:Sozialversicherungsnummer|SV[- ]?Nummer|Rentenversicherung)[\s.:]+(\d{12}[A-Za-z]\d{3})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::Ssn,
            validator: Arc::new(de_sozialversicherung_validate),
            base_score: 0.87,
        },
    ];
    CompositeNationalRecognizer::new("de_national_identity", rules, vec!["de", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_steuer_id() -> String {
        for base in 10_000_000_00u64..10_000_001_00 {
            let s = format!("{:011}", base);
            if de_steuer_id_validate(&s) {
                return s;
            }
        }
        panic!("synth steuer");
    }

    fn synth_personalausweis() -> String {
        for a in b'A'..=b'Z' {
            for n in 0u32..1_000_000 {
                let body = format!("{}{n:07}", a as char);
                let mut sum = 0u32;
                let w = [7u32, 3, 1, 7, 3, 1, 7, 3];
                for i in 0..8 {
                    let v = de_pa_char_value(body.as_bytes()[i]).unwrap();
                    if v < 10 {
                        sum += de_pa_weighted_sum_digit(v, w[i]);
                    } else {
                        sum += de_pa_weighted_sum_digit(v / 10, w[i]);
                        sum += de_pa_weighted_sum_digit(v % 10, w[i]);
                    }
                }
                let check = (10 - (sum % 10)) % 10;
                let cand = format!("{body}{check}");
                if de_personalausweis_validate(&cand) {
                    return cand;
                }
            }
        }
        panic!("synth pa");
    }

    #[test]
    fn steuer_synthetic_roundtrip() {
        let s = synth_steuer_id();
        assert!(de_steuer_id_validate(&s));
    }

    #[test]
    fn steuer_invalid_last() {
        let s = synth_steuer_id();
        let mut c: Vec<char> = s.chars().collect();
        let last = c.pop().unwrap();
        c.push(if last == '0' { '1' } else { '0' });
        assert!(!de_steuer_id_validate(&c.iter().collect::<String>()));
    }

    #[test]
    fn pa_synthetic() {
        let p = synth_personalausweis();
        assert!(de_personalausweis_validate(&p));
    }

    #[test]
    fn reisepass_synthetic() {
        assert!(de_reisepass_validate("C12345678"));
        assert!(!de_reisepass_validate("1C2345678"));
    }

    #[test]
    fn sozial_synthetic_valid() {
        for n in 0u64..2_000_000 {
            let body = format!("{:012}", n);
            for t in 0..1000u32 {
                let s = format!("{body}B{t:03}");
                if de_sozialversicherung_validate(&s) {
                    assert!(de_sozialversicherung_validate(&s));
                    return;
                }
            }
        }
        panic!("aucun numéro synthétique SV valide trouvé");
    }
}
