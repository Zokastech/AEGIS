// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Italy: codice fiscale (16), CIE, partita IVA.

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// Odd/even position tables (indices 0–35 = 0–9 then A–Z), per codice fiscale spec.
const CF_ODD: [u32; 36] = [
    1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6,
    8, 12, 14, 16, 10, 22, 25, 24, 23,
];

const CF_EVEN: [u32; 36] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 23, 24, 25,
];

fn cf_char_index(c: u8) -> Option<usize> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as usize),
        b'A'..=b'Z' => Some((c - b'A') as usize + 10),
        b'a'..=b'z' => Some((c - b'a') as usize + 10),
        _ => None,
    }
}

/// Sum of contributions from the first 15 chars (16th letter check).
pub fn it_codice_fiscale_checksum_sum_15(s: &str) -> Option<u32> {
    let u = s.to_ascii_uppercase();
    if u.len() != 15 {
        return None;
    }
    let b = u.as_bytes();
    let mut sum = 0u32;
    for i in 0..15 {
        let idx = cf_char_index(b[i])?;
        if idx >= 36 {
            return None;
        }
        sum += if (i + 1) % 2 == 1 {
            CF_ODD[idx]
        } else {
            CF_EVEN[idx]
        };
    }
    Some(sum)
}

/// Codice fiscale: 16 alphanumeric chars; 16th = `A` + (sum mod 26).
pub fn it_codice_fiscale_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    if u.len() != 16 || !u.bytes().all(|x| x.is_ascii_alphanumeric()) {
        return false;
    }
    let Some(sum) = it_codice_fiscale_checksum_sum_15(&u[..15]) else {
        return false;
    };
    let expected = char::from_u32(u32::from(b'A') + (sum % 26)).unwrap();
    u.chars().nth(15) == Some(expected)
}

/// CIE: `LL#####LL` (shape only, no unified public checksum).
pub fn it_cie_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    Regex::new(r"^[A-Z]{2}\d{5}[A-Z]{2}$").unwrap().is_match(&u)
}

/// Partita IVA: 11 digits, last digit checks the first 10 (alternating weights 1/2, reduce >9).
pub fn it_partita_iva_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 {
        return false;
    }
    let mut sum = 0u32;
    for i in 0..10 {
        let mut p = d[i] * if i % 2 == 0 { 1 } else { 2 };
        if p > 9 {
            p = (p / 10) + (p % 10);
        }
        sum += p;
    }
    let c = (10 - (sum % 10)) % 10;
    c == d[10]
}

pub fn it_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "codice fiscale",
        "Codice Fiscale",
        "CIE",
        "carta d'identità",
        "Partita IVA",
        "P.IVA",
        "Italian tax",
    ];
    let rules = vec![
        IdRule {
            name: "it_codice_fiscale",
            re: Regex::new(
                r"(?xi)\b(?:codice\s*fiscale|CF)[\s.:]+([A-Za-z0-9]{16})\b|\b([A-Za-z0-9]{16})(?=\s*codice\s*fiscale)",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(it_codice_fiscale_validate),
            base_score: 0.92,
        },
        IdRule {
            name: "it_cie",
            re: Regex::new(r"(?xi)\b(?:CIE|carta\s*d'identit[aà]\s*elettronica)[\s.:]+([A-Za-z]{2}\d{5}[A-Za-z]{2})\b").unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(it_cie_validate),
            base_score: 0.85,
        },
        IdRule {
            name: "it_partita_iva",
            re: Regex::new(r"(?xi)\b(?:Partita\s*IVA|P\.?\s*IVA)[\s.:]+(\d{11})\b|\b(\d{11})(?=\s*P\.?\s*IVA)").unwrap(),
            entity: aegis_core::entity::EntityType::TaxId,
            validator: Arc::new(it_partita_iva_validate),
            base_score: 0.9,
        },
    ];
    CompositeNationalRecognizer::new("it_national_identity", rules, vec!["it", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_cf() -> String {
        let body = "MRNMTT80A01F205";
        assert_eq!(body.len(), 15);
        let sum = it_codice_fiscale_checksum_sum_15(body).unwrap();
        let ch = char::from_u32(u32::from(b'A') + (sum % 26)).unwrap();
        format!("{body}{ch}")
    }

    fn synth_iva() -> String {
        for n in 0u64..500_000 {
            let b = format!("{:010}", n);
            let d: Vec<u32> = b.chars().filter_map(|c| c.to_digit(10)).collect();
            let mut sum = 0u32;
            for i in 0..10 {
                let mut p = d[i] * if i % 2 == 0 { 1 } else { 2 };
                if p > 9 {
                    p = (p / 10) + (p % 10);
                }
                sum += p;
            }
            let c = (10 - (sum % 10)) % 10;
            let s = format!("{b}{c}");
            if it_partita_iva_validate(&s) {
                return s;
            }
        }
        panic!("iva");
    }

    #[test]
    fn cf_synthetic_roundtrip() {
        let cf = synth_cf();
        assert_eq!(cf.len(), 16);
        assert!(it_codice_fiscale_validate(&cf));
    }

    #[test]
    fn cf_bad_last_char() {
        let cf = synth_cf();
        let mut c: Vec<char> = cf.chars().collect();
        let last = c.pop().unwrap();
        c.push(if last == 'A' { 'B' } else { 'A' });
        assert!(!it_codice_fiscale_validate(&c.iter().collect::<String>()));
    }

    #[test]
    fn cie_synthetic() {
        assert!(it_cie_validate("CA00000AA"));
        assert!(!it_cie_validate("C000000AA"));
    }

    #[test]
    fn iva_synthetic() {
        let s = synth_iva();
        assert!(it_partita_iva_validate(&s));
    }
}
