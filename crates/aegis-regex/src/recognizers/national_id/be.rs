// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Belgium: national register number (11 digits, mod-97 key on first 9).

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// Rijksregisternummer: `97 − (N % 97)` on the first 9 digits (integer N), compared to the last 2.
pub fn be_national_number_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 {
        return false;
    }
    let n: u64 = d[..9].iter().fold(0u64, |a, &x| a * 10 + x as u64);
    let k = 97 - (n % 97);
    let key = if k == 97 { 97u32 } else { k as u32 };
    let g = d[9] * 10 + d[10];
    g == key
}

pub fn be_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "numéro national",
        "rijksregisternummer",
        "registre national",
        "BSN België",
        "Belgian national number",
        "NN",
    ];
    let rules = vec![IdRule {
        name: "be_nrn",
        re: Regex::new(
            r"(?xi)\b(?:rijksregisternummer|numéro\s*national|NN)[\s.:]+(\d{11})\b|\b(\d{11})(?=\s*(?:rijksregisternummer|national))",
        )
        .unwrap(),
        entity: aegis_core::entity::EntityType::NationalId,
        validator: Arc::new(be_national_number_validate),
        base_score: 0.91,
    }];
    CompositeNationalRecognizer::new("be_national_identity", rules, vec!["nl", "fr", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_nrn() -> String {
        for n in 1_000_000_00u64..1_000_000_500 {
            let base = format!("{n:09}");
            let v: u64 = base.parse().unwrap();
            let k = 97 - (v % 97);
            let key = if k == 97 { 97u32 } else { k as u32 };
            let s = format!("{base}{key:02}");
            if be_national_number_validate(&s) {
                return s;
            }
        }
        panic!("nrn");
    }

    #[test]
    fn nrn_synthetic() {
        let s = synth_nrn();
        assert!(be_national_number_validate(&s));
    }

    #[test]
    fn nrn_bad_check() {
        let s = synth_nrn();
        let mut v: Vec<char> = s.chars().collect();
        let x = v.pop().unwrap();
        v.push(if x == '0' { '1' } else { '0' });
        assert!(!be_national_number_validate(&v.iter().collect::<String>()));
    }
}
