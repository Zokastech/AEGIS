// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! EU health: Carte Vitale (Luhn heuristic on 10 digits in context), EHIC, record identifiers.

use crate::recognizers::national_id::composite::{CompositeNationalRecognizer, IdRule};
use crate::validation::luhn_valid;
use regex::Regex;
use std::sync::Arc;

fn vitale_ten_digits(s: &str) -> bool {
    let d: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    d.len() == 10 && luhn_valid(&d)
}

fn ehic_token(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    u.len() >= 12
        && u.len() <= 22
        && u.chars().take(2).all(|c| c.is_ascii_alphabetic())
        && u.chars().skip(2).all(|c| c.is_ascii_alphanumeric())
}

pub fn eu_health_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "Carte Vitale",
        "carte vitale",
        "CV",
        "vitale",
        "EHIC",
        "CEAM",
        "TSE",
        "European Health Insurance",
        "carte européenne",
        "Krankenversicherung",
        "Krankenkasse",
        "mutuelle",
        "assurance maladie",
        "dossier médical",
        "dossier medical",
        "patient",
        "MRN",
        "NIR patient",
        "hôpital",
        "ospedale",
        "hospital",
    ];
    let rules = vec![
        IdRule {
            name: "fr_carte_vitale_ctx",
            re: Regex::new(
                r"(?xi)\b(?:carte\s*vitale|CV\s*vitale|numéro\s*vitale)[\s.:]+(\d{10})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::MedicalRecord,
            validator: Arc::new(vitale_ten_digits),
            base_score: 0.8,
        },
        IdRule {
            name: "ehic_iso_token",
            re: Regex::new(
                r"(?xi)\b(?:EHIC|CEAM|TSE|european\s*health\s*insurance)[\s.:#]+([A-Z]{2}[\dA-Z]{10,20})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::MedicalRecord,
            validator: Arc::new(ehic_token),
            base_score: 0.72,
        },
        IdRule {
            name: "medical_record_id",
            re: Regex::new(
                r"(?xi)\b(?:MRN|dossier\s*(?:patient|médical|medical)|patient\s*id|numéro\s*de\s*dossier)[\s.:]+([A-Z0-9]{4,16}(?:[\-/][A-Z0-9]{2,8})?)\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::MedicalRecord,
            validator: Arc::new(|m| {
                let t = m.trim();
                t.len() >= 4 && t.len() <= 24 && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '/')
            }),
            base_score: 0.68,
        },
    ];
    CompositeNationalRecognizer::new("eu_health_ids", rules, vec!["*"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vitale_luhn_synthetic() {
        // 7992739871 is classic Luhn-valid test vector (not real data).
        assert!(vitale_ten_digits("7992739871"));
        assert!(!vitale_ten_digits("7992739872"));
    }
}
