// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Formats de plaques d’immatriculation courants (UE + UK) — heuristiques, pas un registre officiel.

use crate::recognizers::national_id::composite::{CompositeNationalRecognizer, IdRule};
use regex::Regex;
use std::sync::Arc;

fn norm_plate(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn fr(s: &str) -> bool {
    Regex::new(r"^[A-HJ-NP-TV-Z]{2}\d{3}[A-HJ-NP-TV-Z]{2}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn de(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    Regex::new(r"^[A-ZÄÖÜ]{1,3}[\s\-]+[A-ZÄÖÜ]{1,2}[\s\-]+\d{1,4}[A-Z]{0,2}$")
        .unwrap()
        .is_match(u.trim())
}

fn it(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2}\d{3}[A-Z]{2}$").unwrap().is_match(&norm_plate(s))
}

fn es(s: &str) -> bool {
    Regex::new(r"^\d{4}[A-Z]{3}$").unwrap().is_match(&norm_plate(s))
}

fn nl(s: &str) -> bool {
    let n = norm_plate(s);
    Regex::new(r"^(\d{1,3}[A-Z]{1,3}\d{1,3}|\d{2}[A-Z]{2}\d{2}|[A-Z]{2}\d{2}\d{2})$")
        .unwrap()
        .is_match(&n)
}

fn be(s: &str) -> bool {
    Regex::new(r"^\d[A-Z]{3}\d{3}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn pt(s: &str) -> bool {
    let n = norm_plate(s);
    Regex::new(r"^([A-Z]{2}\d{4}|\d{2}[A-Z]{2}\d{2}|\d{2}\d{2}[A-Z]{2})$")
        .unwrap()
        .is_match(&n)
}

fn pl(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2,3}\d{4,5}[A-Z]?$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn at(s: &str) -> bool {
    let u = s.trim().to_ascii_uppercase();
    Regex::new(r"^[A-Z]{1,3}\s+[A-Z]{1,2}\s+\d{1,6}$")
        .unwrap()
        .is_match(&u)
}

fn se(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{2,3}[A-Z]$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn gb(s: &str) -> bool {
    let n = norm_plate(s);
    Regex::new(r"^[A-Z]{2}\d{2}[A-Z]{3}$").unwrap().is_match(&n)
}

fn ie(s: &str) -> bool {
    Regex::new(r"^\d{2}[A-Z]{1,2}\d{1,6}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn dk(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2}\d{5}$").unwrap().is_match(&norm_plate(s))
}

fn fi(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{3}$").unwrap().is_match(&norm_plate(s))
}

fn gr(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{4}$").unwrap().is_match(&norm_plate(s))
}

fn cz(s: &str) -> bool {
    Regex::new(r"^\d[A-Z]\d\d{4}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn ro(s: &str) -> bool {
    Regex::new(r"^[A-Z]{1,2}\d{2,3}[A-Z]{3}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn hu(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{3}$").unwrap().is_match(&norm_plate(s))
}

fn sk(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2}\d{3}[A-Z]{2}$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn hr(s: &str) -> bool {
    let u = s.trim().to_ascii_uppercase();
    Regex::new(r"^[A-Z]{2}\s*\d{3,4}\s*[A-Z]{2}$")
        .unwrap()
        .is_match(&u)
}

fn si(s: &str) -> bool {
    let n = norm_plate(s);
    Regex::new(r"^[A-Z]{1,2}\d{1,2}[A-Z]{2}$").unwrap().is_match(&n)
}

fn ee(s: &str) -> bool {
    Regex::new(r"^\d{3}[A-Z]{2}\d$")
        .unwrap()
        .is_match(&norm_plate(s))
}

fn lv(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2}\d{4}$").unwrap().is_match(&norm_plate(s))
}

fn lt(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{3}$").unwrap().is_match(&norm_plate(s))
}

fn cy(s: &str) -> bool {
    Regex::new(r"^[A-Z]{3}\d{3}$").unwrap().is_match(&norm_plate(s))
}

fn lu(s: &str) -> bool {
    Regex::new(r"^[A-Z]{2}\d{4}$").unwrap().is_match(&norm_plate(s))
}

fn mt(s: &str) -> bool {
    let n = norm_plate(s);
    Regex::new(r"^[A-Z]{3}\d{2,4}$").unwrap().is_match(&n)
}

/// Recognizer composite : motifs par pays + validation de forme.
pub fn eu_license_plate_recognizer() -> CompositeNationalRecognizer {
    let ctx = [
        "plaque",
        "plaque d'immatriculation",
        "immatriculation",
        "numéro de plaque",
        "Kennzeichen",
        "Nummernschild",
        "targa",
        "targhe",
        "matrícula",
        "kenteken",
        "nummerplaat",
        "registration plate",
        "license plate",
        "numéro d'immatriculation",
        "SPZ",
        "tablica rejestracyjna",
        "matrícula automóvel",
    ];
    let rules = vec![
        IdRule {
            name: "plate_fr",
            re: Regex::new(
                r"(?xi)\b(?:FR|FRA)[\s\-]*([A-HJ-NP-TV-Z]{2}[\s\-]?\d{3}[\s\-]?[A-HJ-NP-TV-Z]{2})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| fr(m)),
            base_score: 0.78,
        },
        IdRule {
            name: "plate_de",
            re: Regex::new(
                r"(?xi)\bDE[\s\-]*([A-ZÄÖÜ]{1,3}[\s\-]+[A-ZÄÖÜ]{1,2}[\s\-]+\d{1,4}[A-Z]{0,2})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| de(m)),
            base_score: 0.76,
        },
        IdRule {
            name: "plate_it",
            re: Regex::new(r"(?xi)\bIT[\s\-]*([A-Z]{2}\s*\d{3}\s*[A-Z]{2})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| it(m)),
            base_score: 0.78,
        },
        IdRule {
            name: "plate_es",
            re: Regex::new(r"(?xi)\bES[\s\-]*(\d{4}\s*[A-Z]{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| es(m)),
            base_score: 0.77,
        },
        IdRule {
            name: "plate_nl",
            re: Regex::new(
                r"(?xi)\b(?:NL)[\s\-]*(\d{1,3}[\s\-]?[A-Z]{1,3}[\s\-]?\d{1,3}|\d{2}[\s\-]?[A-Z]{2}[\s\-]?\d{2})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| nl(m)),
            base_score: 0.74,
        },
        IdRule {
            name: "plate_be",
            re: Regex::new(r"(?xi)\bBE[\s\-]*(\d[\s\-]?[A-Z]{3}[\s\-]?\d{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| be(m)),
            base_score: 0.75,
        },
        IdRule {
            name: "plate_pt",
            re: Regex::new(
                r"(?xi)\bPT[\s\-]*([A-Z]{2}[\s\-]?\d{2}[\s\-]?\d{2}|\d{2}[\s\-]?[A-Z]{2}[\s\-]?\d{2})\b",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| pt(m)),
            base_score: 0.73,
        },
        IdRule {
            name: "plate_pl",
            re: Regex::new(r"(?xi)\b(?:PL)[\s\-]*([A-Z]{2,3}\s?\d{4,5}[A-Z]?)\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| pl(m)),
            base_score: 0.74,
        },
        IdRule {
            name: "plate_at",
            re: Regex::new(r"(?xi)\bAT[\s\-]*([A-Z]{1,3}\s+[A-Z]{1,2}\s+\d{1,6})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| at(m)),
            base_score: 0.75,
        },
        IdRule {
            name: "plate_se",
            re: Regex::new(r"(?xi)\bSE[\s\-]*([A-Z]{3}\s?\d{2,3}\s?[A-Z])\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| se(m)),
            base_score: 0.74,
        },
        IdRule {
            name: "plate_gb",
            re: Regex::new(r"(?xi)\b(?:GB|UK)[\s\-]*([A-Z]{2}\s?\d{2}\s?[A-Z]{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| gb(m)),
            base_score: 0.77,
        },
        IdRule {
            name: "plate_ie",
            re: Regex::new(r"(?xi)\b(?:IRL|IE)[\s\-]*(\d{2}[\s\-]?[A-Z]{1,2}[\s\-]?\d{1,6})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| ie(m)),
            base_score: 0.72,
        },
        IdRule {
            name: "plate_dk",
            re: Regex::new(r"(?xi)\b(?:DK)[\s\-]*([A-Z]{2}\s?\d{5})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| dk(m)),
            base_score: 0.74,
        },
        IdRule {
            name: "plate_fi",
            re: Regex::new(r"(?xi)\b(?:FIN|FI)[\s\-]*([A-Z]{3}[\s\-]?\d{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| fi(m)),
            base_score: 0.73,
        },
        IdRule {
            name: "plate_gr",
            re: Regex::new(r"(?xi)\b(?:GR)[\s\-]*([A-Z]{3}[\s\-]?\d{4})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| gr(m)),
            base_score: 0.72,
        },
        IdRule {
            name: "plate_cz",
            re: Regex::new(r"(?xi)\b(?:CZ)[\s\-]*(\d[A-Z]\d\s?\d{4})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| cz(m)),
            base_score: 0.71,
        },
        IdRule {
            name: "plate_ro",
            re: Regex::new(r"(?xi)\b(?:RO)[\s\-]*([A-Z]{1,2}\s?\d{2,3}\s?[A-Z]{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| ro(m)),
            base_score: 0.71,
        },
        IdRule {
            name: "plate_hu",
            re: Regex::new(r"(?xi)\bHU[\s\-]*([A-Z]{3}[\s\-]?\d{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| hu(m)),
            base_score: 0.72,
        },
        IdRule {
            name: "plate_sk",
            re: Regex::new(r"(?xi)\b(?:SK)[\s\-]*([A-Z]{2}\d{3}[A-Z]{2})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| sk(m)),
            base_score: 0.73,
        },
        IdRule {
            name: "plate_hr",
            re: Regex::new(r"(?xi)\b(?:HR)[\s\-]*([A-Z]{2}\s*\d{3,4}\s*[A-Z]{2})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| hr(m)),
            base_score: 0.72,
        },
        IdRule {
            name: "plate_si",
            re: Regex::new(r"(?xi)\b(?:SLO|SI)[\s\-]*([A-Z]{1,2}\s?\d{1,2}\s?[A-Z]{2})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| si(m)),
            base_score: 0.7,
        },
        IdRule {
            name: "plate_ee",
            re: Regex::new(r"(?xi)\b(?:EST|EE)[\s\-]*(\d{3}\s?[A-Z]{2}\d)\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| ee(m)),
            base_score: 0.71,
        },
        IdRule {
            name: "plate_lv",
            re: Regex::new(r"(?xi)\b(?:LV)[\s\-]*([A-Z]{2}[\s\-]?\d{4})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| lv(m)),
            base_score: 0.71,
        },
        IdRule {
            name: "plate_lt",
            re: Regex::new(r"(?xi)\b(?:LT)[\s\-]*([A-Z]{3}\s?\d{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| lt(m)),
            base_score: 0.71,
        },
        IdRule {
            name: "plate_cy",
            re: Regex::new(r"(?xi)\b(?:CY)[\s\-]*([A-Z]{3}\s?\d{3})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| cy(m)),
            base_score: 0.7,
        },
        IdRule {
            name: "plate_lu",
            re: Regex::new(r"(?xi)\bLU[\s\-]*([A-Z]{2}\d{4})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| lu(m)),
            base_score: 0.72,
        },
        IdRule {
            name: "plate_mt",
            re: Regex::new(r"(?xi)\bMT[\s\-]*([A-Z]{3}\d{2,4})\b").unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| mt(m)),
            base_score: 0.7,
        },
        IdRule {
            name: "plate_fr_bare",
            re: Regex::new(
                r"(?xi)(?<![A-Z0-9])([A-HJ-NP-TV-Z]{2}[\s\-]\d{3}[\s\-][A-HJ-NP-TV-Z]{2})(?![A-Z0-9])",
            )
            .unwrap(),
            entity: aegis_core::entity::EntityType::VehiclePlate,
            validator: Arc::new(|m| fr(m)),
            base_score: 0.55,
        },
    ];
    CompositeNationalRecognizer::new("eu_license_plates", rules, vec!["*"], &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validators_shape() {
        assert!(fr("AB-123-CD"));
        assert!(!fr("AB-12-CD"));
        assert!(it("AB123CD"));
        assert!(es("1234 ABC"));
        assert!(gb("AB12CDE"));
    }
}
