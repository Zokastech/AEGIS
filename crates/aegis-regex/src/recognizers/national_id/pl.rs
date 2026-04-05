// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Pologne : PESEL, NIP.

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// PESEL : 11 chiffres, d[10] = (10 − (Σ w[i]×d[i] mod 10)) mod 10, w = 1,3,7,9,1,3,7,9,1,3.
pub fn pl_pesel_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 {
        return false;
    }
    let w = [1u32, 3, 7, 9, 1, 3, 7, 9, 1, 3];
    let sum: u32 = (0..10).map(|i| d[i] * w[i]).sum();
    let c = (10 - (sum % 10)) % 10;
    c == d[10]
}

/// NIP : 10 chiffres, d[9] = (Σ d[i]×w[i]) mod 11 avec w = 6,5,7,2,3,4,5,6,7 (si 10 → invalide en pratique, ici 0).
pub fn pl_nip_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 10 {
        return false;
    }
    let w = [6u32, 5, 7, 2, 3, 4, 5, 6, 7];
    let sum: u32 = (0..9).map(|i| d[i] * w[i]).sum();
    let r = sum % 11;
    if r == 10 {
        return false;
    }
    r == d[9]
}

pub fn pl_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "PESEL",
        "NIP",
        "numer PESEL",
        "Polish national",
        "identyfikator podatkowy",
    ];
    let rules = vec![
        IdRule {
            name: "pl_pesel",
            re: Regex::new(r"(?xi)\b(?:PESEL)[\s.:]+(\d{11})\b|\b(\d{11})(?=\s*PESEL)").unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(pl_pesel_validate),
            base_score: 0.9,
        },
        IdRule {
            name: "pl_nip",
            re: Regex::new(r"(?xi)\b(?:NIP)[\s.:]+(\d{10})\b|\b(\d{10})(?=\s*NIP)").unwrap(),
            entity: aegis_core::entity::EntityType::TaxId,
            validator: Arc::new(pl_nip_validate),
            base_score: 0.89,
        },
    ];
    CompositeNationalRecognizer::new("pl_national_identity", rules, vec!["pl", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_pesel() -> String {
        for body in 0u64..500_000 {
            let b = format!("{:010}", body);
            let d: Vec<u32> = b.chars().filter_map(|c| c.to_digit(10)).collect();
            let w = [1u32, 3, 7, 9, 1, 3, 7, 9, 1, 3];
            let sum: u32 = (0..10).map(|i| d[i] * w[i]).sum();
            let c = (10 - (sum % 10)) % 10;
            let s = format!("{b}{c}");
            if pl_pesel_validate(&s) {
                return s;
            }
        }
        panic!("pesel");
    }

    #[test]
    fn pesel_synthetic() {
        let s = synth_pesel();
        assert!(pl_pesel_validate(&s));
    }

    #[test]
    fn nip_synthetic() {
        for n in 100_000_000u64..100_000_200 {
            let b = format!("{n:09}");
            let d: Vec<u32> = b.chars().filter_map(|c| c.to_digit(10)).collect();
            let w = [6u32, 5, 7, 2, 3, 4, 5, 6, 7];
            let sum: u32 = (0..9).map(|i| d[i] * w[i]).sum();
            let r = sum % 11;
            if r == 10 {
                continue;
            }
            let s = format!("{b}{r}");
            assert!(pl_nip_validate(&s), "{s}");
            return;
        }
        panic!("nip");
    }
}
