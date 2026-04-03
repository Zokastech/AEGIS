// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Netherlands: BSN (elfproef).

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// BSN: 9 digits, Σ (9−i)×d[i] ≡ 0 (mod 11), and not a trivial reserved sequence.
pub fn nl_bsn_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 9 {
        return false;
    }
    if d.iter().all(|&x| x == d[0]) {
        return false;
    }
    let sum: i32 = d
        .iter()
        .enumerate()
        .map(|(i, &x)| x as i32 * (9 - i as i32))
        .sum();
    sum % 11 == 0
}

pub fn nl_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "BSN",
        "Burgerservicenummer",
        "sofi-nummer",
        "identiteitsnummer",
        "Dutch citizen service number",
        "NL ID",
    ];
    let rules = vec![IdRule {
        name: "nl_bsn",
        re: Regex::new(r"(?xi)\b(?:BSN|Burgerservicenummer)[\s.:]+(\d{9})\b|\b(\d{9})(?=\s*(?:BSN|Burgerservicenummer))").unwrap(),
        entity: aegis_core::entity::EntityType::NationalId,
        validator: Arc::new(|m| nl_bsn_validate(m)),
        base_score: 0.9,
    }];
    CompositeNationalRecognizer::new("nl_national_identity", rules, vec!["nl", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_bsn() -> String {
        for n in 1_000_000_00u32..1_000_001_00 {
            let s = format!("{n:09}");
            if nl_bsn_validate(&s) {
                return s;
            }
        }
        panic!("bsn");
    }

    #[test]
    fn bsn_synthetic_valid() {
        let s = synth_bsn();
        assert!(nl_bsn_validate(&s));
    }

    #[test]
    fn bsn_invalid() {
        assert!(!nl_bsn_validate("123456789"));
    }

    #[test]
    fn bsn_all_same_rejected() {
        assert!(!nl_bsn_validate("111111111"));
    }
}
