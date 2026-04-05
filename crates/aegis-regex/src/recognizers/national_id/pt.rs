// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Portugal: NIF (mod 11), Cartão de Cidadão number (9 digits, same checksum family).

use super::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

/// NIF / PT tax number: 9 digits; d[8] from Σ d[i]×(9−i) mod 11.
pub fn pt_nif_validate(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 9 {
        return false;
    }
    let mut t = 0u32;
    for i in 0..8 {
        t += d[i] * (9 - i as u32);
    }
    let r = t % 11;
    let c = if r < 2 { 0 } else { 11 - r };
    c == d[8]
}

/// Number printed on Cartão de Cidadão (9 digits) — same check as NIF for the numeric variant.
pub fn pt_cartao_cidadao_numeric_validate(s: &str) -> bool {
    pt_nif_validate(s)
}

pub fn pt_national_id_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "NIF",
        "número de identificação fiscal",
        "Cartão de Cidadão",
        "cartão de cidadão",
        "CC",
        "Portuguese tax",
    ];
    let rules = vec![
        IdRule {
            name: "pt_nif",
            re: Regex::new(
                r"(?xi)\b(?:NIF|n[uú]mero\s*de\s*identifica[çc][aã]o\s*fiscal)[\s.:]+(\d{9})\b|\b(\d{9})\s+NIF\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::TaxId,
            validator: Arc::new(pt_nif_validate),
            base_score: 0.9,
        },
        IdRule {
            name: "pt_cartao_cidadao",
            re: Regex::new(
                r"(?xi)\b(?:Cart[aã]o\s*de\s*Cidad[aã]o|CC)[\s.:]+(\d{9})\b|\b(\d{9})\s+(?:Cart[aã]o\s*de\s*Cidad[aã]o|CC)\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::NationalId,
            validator: Arc::new(pt_cartao_cidadao_numeric_validate),
            base_score: 0.88,
        },
    ];
    CompositeNationalRecognizer::new("pt_national_identity", rules, vec!["pt", "en"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_nif() -> String {
        for n in 0u64..100_000_000 {
            let b = format!("{n:08}");
            let d: Vec<u32> = b.chars().filter_map(|c| c.to_digit(10)).collect();
            let mut t = 0u32;
            for i in 0..8 {
                t += d[i] * (9 - i as u32);
            }
            let r = t % 11;
            let c = if r < 2 { 0 } else { 11 - r };
            let s = format!("{b}{c}");
            if pt_nif_validate(&s) {
                return s;
            }
        }
        panic!("nif");
    }

    #[test]
    fn nif_synthetic() {
        let s = synth_nif();
        assert!(pt_nif_validate(&s));
    }

    #[test]
    fn nif_invalid_check() {
        let s = synth_nif();
        let mut c: Vec<char> = s.chars().collect();
        let last = c.pop().unwrap();
        c.push(if last == '0' { '1' } else { '0' });
        assert!(!pt_nif_validate(&c.iter().collect::<String>()));
    }
}
