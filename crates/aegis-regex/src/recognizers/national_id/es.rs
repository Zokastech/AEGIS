// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Spain: DNI/NIF, NIE, social security number (format + DNI/NIE letter check).

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

const DNI_TABLE: &[u8; 23] = b"TRWAGMYFPDXBNJZSQVHLCKE";

fn dni_letter_from_number(n: u32) -> char {
    DNI_TABLE[(n % 23) as usize] as char
}

/// DNI: 8 digits + control letter.
pub fn es_dni_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let re = Regex::new(r"^(\d{8})([A-Z])$").unwrap();
    let Some(cap) = re.captures(&u) else {
        return false;
    };
    let n: u32 = cap[1].parse().unwrap_or(999);
    let l = cap[2].chars().next().unwrap();
    l == dni_letter_from_number(n)
}

/// NIE: X/Y/Z + 7 digits + letter (X=0, Y=1, Z=2 for the checksum).
pub fn es_nie_validate(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let re = Regex::new(r"^([XYZ])(\d{7})([A-Z])$").unwrap();
    let Some(cap) = re.captures(&u) else {
        return false;
    };
    let prefix = match cap[1].chars().next().unwrap() {
        'X' => 0u32,
        'Y' => 1,
        'Z' => 2,
        _ => return false,
    };
    let n: u32 = cap[2].parse().unwrap_or(9999999);
    let full = prefix * 10_000_000 + n;
    let l = cap[3].chars().next().unwrap();
    l == dni_letter_from_number(full)
}

/// Spanish social security number (12 digits): key `97 - (N % 97)` on the number from the first 10 digits.
pub fn es_seguridad_social_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 12 {
        return false;
    }
    let n: u64 = d[..10].iter().fold(0u64, |acc, &x| acc * 10 + x as u64);
    let k = 97 - (n % 97);
    let key = if k == 97 { 97u32 } else { k as u32 };
    let g = d[10] * 10 + d[11];
    g == key
}

pub fn es_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "DNI",
        "NIF",
        "NIE",
        "documento nacional",
        "seguridad social",
        "número de afiliación",
        "Spanish ID",
        "tarjeta sanitaria",
    ];
    let rules = vec![
        IdRule {
            name: "es_dni",
            re: Regex::new(r"(?xi)\b(?:DNI|NIF)[\s.:]+(\d{8}[A-Za-z])\b|\b(\d{8}[A-Za-z])(?=\s*(?:DNI|NIF))").unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(es_dni_validate),
            base_score: 0.9,
        },
        IdRule {
            name: "es_nie",
            re: Regex::new(r"(?xi)\b(?:NIE)[\s.:]+([XYZxyz]\d{7}[A-Za-z])\b|\b([XYZxyz]\d{7}[A-Za-z])(?=\s*NIE)").unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(es_nie_validate),
            base_score: 0.89,
        },
        IdRule {
            name: "es_seguridad_social",
            re: Regex::new(
                r"(?xi)\b(?:seguridad\s*social|n[uú]mero\s*de\s*la\s*seguridad\s*social|NAF)[\s.:]+(\d{12})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::Ssn,
            validator: Arc::new(es_seguridad_social_validate),
            base_score: 0.86,
        },
    ];
    CompositeNationalRecognizer::new("es_national_identity", rules, vec!["es", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_dni() -> String {
        for n in 10_000_000..10_000_100 {
            let l = dni_letter_from_number(n);
            let s = format!("{n:08}{l}");
            if es_dni_validate(&s) {
                return s;
            }
        }
        panic!("dni");
    }

    #[test]
    fn dni_synthetic() {
        let s = synth_dni();
        assert!(es_dni_validate(&s));
    }

    #[test]
    fn dni_bad_letter() {
        let s = synth_dni();
        let mut c: Vec<char> = s.chars().collect();
        let last = c.pop().unwrap();
        c.push(if last == 'A' { 'B' } else { 'A' });
        assert!(!es_dni_validate(&c.iter().collect::<String>()));
    }

    #[test]
    fn nie_synthetic() {
        let n = 1234567u32;
        let full = n;
        let l = dni_letter_from_number(full);
        let s = format!("X{n:07}{l}");
        assert!(es_nie_validate(&s));
    }

    #[test]
    fn ss_synthetic() {
        if let Some(body) = (0u64..1_000_000).next() {
            let base = format!("{:010}", body);
            let n: u64 = base.parse().unwrap();
            let k = 97 - (n % 97);
            let key = if k == 97 { 97u32 } else { k as u32 };
            let s = format!("{base}{key:02}");
            assert!(es_seguridad_social_validate(&s), "{s}");
            return;
        }
        panic!("ss");
    }
}
