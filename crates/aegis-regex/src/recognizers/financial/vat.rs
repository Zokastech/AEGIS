// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Intra-community VAT numbers (EU formats + targeted checks).

use super::common::{adjust_score_in_context, build_ac, vat_context_words};
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

struct VatRule {
    name: &'static str,
    re: Regex,
    validator: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

/// French VAT key check (2 chars after FR + 9 SIREN digits).
fn fr_vat_ok(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let Some(body) = u.strip_prefix("FR") else {
        return false;
    };
    if body.len() != 11 {
        return false;
    }
    let key = &body[..2];
    let num = &body[2..];
    if !num.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let sireno: u64 = num.parse().unwrap_or(0);
    let c = (12 + 3 * (sireno % 97)) % 97;
    let expected = format!("{:02}", c);
    key == expected
}

/// Espagne : 1 alnum + 7 chiffres + 1 alnum (structure minimale).
fn es_vat_ok(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let Some(rest) = u.strip_prefix("ES") else {
        return false;
    };
    rest.len() == 9 && rest[1..8].chars().all(|c| c.is_ascii_digit())
}

fn nl_vat_ok(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    let Some(rest) = u.strip_prefix("NL") else {
        return false;
    };
    if rest.len() != 12 {
        return false;
    }
    if !rest[0..9].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    rest.as_bytes().get(9) == Some(&b'B')
}

pub struct EuVatRecognizer {
    rules: Vec<VatRule>,
    boost_ac: Option<AhoCorasick>,
    penalty_ac: Option<AhoCorasick>,
    min_score: f64,
    score_cap: f64,
}

impl EuVatRecognizer {
    pub fn new() -> Self {
        let rules = vec![
            VatRule {
                name: "vat_fr",
                re: Regex::new(r"(?i)\bFR[A-HJ-NP-Z0-9]{2}\d{9}\b").unwrap(),
                validator: Some(Arc::new(fr_vat_ok)),
            },
            VatRule {
                name: "vat_de",
                re: Regex::new(r"(?i)\bDE\d{9}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_it",
                re: Regex::new(r"(?i)\bIT\d{11}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_es",
                re: Regex::new(r"(?i)\bES[A-Z0-9]\d{7}[A-Z0-9]\b").unwrap(),
                validator: Some(Arc::new(es_vat_ok)),
            },
            VatRule {
                name: "vat_nl",
                re: Regex::new(r"(?i)\bNL\d{9}B\d{2}\b").unwrap(),
                validator: Some(Arc::new(nl_vat_ok)),
            },
            VatRule {
                name: "vat_be",
                re: Regex::new(r"(?i)\bBE0\d{9}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_at",
                re: Regex::new(r"(?i)\bATU\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_pt",
                re: Regex::new(r"(?i)\bPT\d{9}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_pl",
                re: Regex::new(r"(?i)\bPL(\d{10}|\d{9})\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_se",
                re: Regex::new(r"(?i)\bSE\d{12}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_dk",
                re: Regex::new(r"(?i)\bDK\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_fi",
                re: Regex::new(r"(?i)\bFI\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_hu",
                re: Regex::new(r"(?i)\bHU\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_cz",
                re: Regex::new(r"(?i)\bCZ\d{8,10}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_sk",
                re: Regex::new(r"(?i)\bSK\d{10}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_si",
                re: Regex::new(r"(?i)\bSI\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_ro",
                re: Regex::new(r"(?i)\bRO\d{2,10}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_bg",
                re: Regex::new(r"(?i)\bBG\d{9,10}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_hr",
                re: Regex::new(r"(?i)\bHR\d{11}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_gr",
                re: Regex::new(r"(?i)\bEL\d{9}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_ie",
                re: Regex::new(r"(?i)\bIE[0-9][A-Z0-9][0-9]{5}[A-Z]\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_lt",
                re: Regex::new(r"(?i)\bLT(\d{9}|\d{12})\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_lv",
                re: Regex::new(r"(?i)\bLV\d{11}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_ee",
                re: Regex::new(r"(?i)\bEE\d{9}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_mt",
                re: Regex::new(r"(?i)\bMT\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_cy",
                re: Regex::new(r"(?i)\bCY\d{8}[A-Z]\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_lu",
                re: Regex::new(r"(?i)\bLU\d{8}\b").unwrap(),
                validator: None,
            },
            VatRule {
                name: "vat_xi",
                re: Regex::new(r"(?i)\bXI\d{9}\b").unwrap(),
                validator: None,
            },
        ];
        let ctx = vat_context_words();
        Self {
            rules,
            boost_ac: build_ac(&ctx),
            penalty_ac: build_ac(&["example vat", "test tin", "dummy"]),
            min_score: 0.4,
            score_cap: 1.0,
        }
    }

    fn base_score(matched: &str, rule: &VatRule) -> Option<f64> {
        let u = matched.to_ascii_uppercase();
        if let Some(ref v) = rule.validator {
            if v(&u) {
                return Some(0.92);
            }
            return Some(0.48);
        }
        Some(0.82)
    }
}

impl Default for EuVatRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Recognizer for EuVatRecognizer {
    fn name(&self) -> &str {
        "eu_vat_intracom"
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![EntityType::TaxId]
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"]
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        for rule in &self.rules {
            for m in rule.re.find_iter(text) {
                let slice = m.as_str();
                let u = slice.to_ascii_uppercase();
                let Some(base) = Self::base_score(&u, rule) else {
                    continue;
                };
                let Some(score) = adjust_score_in_context(
                    base,
                    text,
                    m.start(),
                    m.end(),
                    config,
                    self.boost_ac.as_ref(),
                    0.06,
                    self.penalty_ac.as_ref(),
                    0.12,
                    None,
                    0.12,
                    self.min_score,
                    self.score_cap,
                ) else {
                    continue;
                };
                if score < self.min_score {
                    continue;
                }
                let mut metadata = HashMap::new();
                metadata.insert("level".into(), "regex".into());
                metadata.insert("vat_rule".into(), rule.name.to_string());
                out.push(Entity {
                    entity_type: EntityType::TaxId,
                    start: m.start(),
                    end: m.end(),
                    text: slice.to_string(),
                    score,
                    recognizer_name: self.name().to_string(),
                    metadata,
                    decision_trace: None,
                });
            }
        }
        out
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}

pub fn eu_vat_recognizer() -> EuVatRecognizer {
    EuVatRecognizer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Recognizer;
    fn cfg() -> AnalysisConfig {
        AnalysisConfig::default()
    }

    #[test]
    fn detect_de_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("USt-IdNr DE123456789", &cfg());
        assert!(!v.is_empty());
        assert!(v.iter().any(|e| e.text.contains("DE123456789")));
    }

    #[test]
    fn detect_fr_vat_format() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("TVA FR29443169758", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_it_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("P.IVA IT12345678901", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_es_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("IVA ESX1234567X", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_nl_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("BTW NL123456789B01", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_be_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("TVA BE0123456789", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_at_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("UID ATU12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_pt_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("NIF PT123456789", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_pl_vat_10() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("NIP PL1234567890", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_se_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("moms SE123456789012", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_dk_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("DK12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_fi_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("FI12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_hu_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("HU12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_cz_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("CZ12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_si_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("SI12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_sk_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("SK1234567890", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_ro_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("RO12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_gr_el_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("EL123456789", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_ie_vat_shape() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("IE1A12345J", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_lt_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("LT123456789", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_ee_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("EE123456789", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_mt_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("MT12345678", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn detect_cy_vat() {
        let r = EuVatRecognizer::new();
        let v = r.analyze("CY12345678X", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn fr_checksum_valid_sample() {
        assert!(fr_vat_ok("FR29443169758"));
    }

    #[test]
    fn fr_checksum_invalid() {
        assert!(!fr_vat_ok("FR99443169758"));
    }

    #[test]
    fn es_vat_pattern_len() {
        let r = EuVatRecognizer::new();
        assert!(r.analyze("ESB1234567B", &cfg()).len() >= 1);
    }

    #[test]
    fn no_match_us_ein() {
        let r = EuVatRecognizer::new();
        assert!(r.analyze("EIN 12-3456789", &cfg()).is_empty());
    }

    #[test]
    fn supported_langs_count() {
        assert_eq!(EuVatRecognizer::new().supported_languages().len(), 10);
    }
}
