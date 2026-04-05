// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! IBAN — country patterns SEPA / wider Europe, MOD-97 (ISO 13616).

use super::common::{adjust_score_in_context, build_ac, iban_context_words};
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;

/// Longueur totale IBAN (sans espaces) par code pays ISO (registre SWIFT / SEPA, focus Europe).
fn iban_length_for_country(cc: &str) -> Option<usize> {
    let c = cc.to_ascii_uppercase();
    match c.as_str() {
        "AD" => Some(24),
        "AT" => Some(20),
        "BE" => Some(16),
        "BG" => Some(22),
        "CH" => Some(21),
        "CY" => Some(28),
        "CZ" => Some(24),
        "DE" => Some(22),
        "DK" => Some(18),
        "EE" => Some(20),
        "ES" => Some(24),
        "FI" => Some(18),
        "FO" => Some(18),
        "FR" => Some(27),
        "GB" => Some(22),
        "GI" => Some(23),
        "GL" => Some(18),
        "GR" => Some(27),
        "HR" => Some(21),
        "HU" => Some(28),
        "IE" => Some(22),
        "IS" => Some(26),
        "IT" => Some(27),
        "LI" => Some(21),
        "LT" => Some(20),
        "LU" => Some(20),
        "LV" => Some(21),
        "MC" => Some(27),
        "MD" => Some(24),
        "ME" => Some(22),
        "MK" => Some(19),
        "MT" => Some(31),
        "NL" => Some(18),
        "NO" => Some(15),
        "PL" => Some(28),
        "PT" => Some(25),
        "RO" => Some(24),
        "RS" => Some(22),
        "SE" => Some(24),
        "SI" => Some(19),
        "SK" => Some(24),
        "SM" => Some(27),
        "TR" => Some(26),
        "VA" => Some(22),
        "XK" => Some(20),
        _ => None,
    }
}

/// Normalise : uniquement alphanum ASCII majuscules.
pub fn normalize_iban(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// MOD-97-10 (ISO 13616) on normalized string (no spaces).
pub fn iban_mod97_valid(normalized: &str) -> bool {
    if normalized.len() < 8 {
        return false;
    }
    let rear = &normalized[4..];
    let front = &normalized[..4];
    let mut rearranged = String::with_capacity(normalized.len());
    rearranged.push_str(rear);
    rearranged.push_str(front);

    let mut rem: u64 = 0;
    for ch in rearranged.chars() {
        match ch {
            '0'..='9' => {
                let d = (ch as u8 - b'0') as u64;
                rem = (rem * 10 + d) % 97;
            }
            'A'..='Z' => {
                let v = (ch as u8 - b'A') as u64 + 10;
                rem = (rem * 100 + v) % 97;
            }
            _ => {}
        }
    }
    rem == 1
}

/// Plausible format (2 letters + 2 digits + alnum) and length consistent with country when known.
pub fn iban_format_plausible(normalized: &str) -> bool {
    let bytes = normalized.as_bytes();
    if normalized.len() < 15 || normalized.len() > 34 {
        return false;
    }
    if !(bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_alphabetic()) {
        return false;
    }
    if !(bytes[2].is_ascii_digit() && bytes[3].is_ascii_digit()) {
        return false;
    }
    if !normalized[4..].chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    let cc = &normalized[..2];
    if let Some(exp) = iban_length_for_country(cc) {
        normalized.len() == exp
    } else {
        true
    }
}

pub struct IbanRecognizer {
    pattern: Regex,
    boost_ac: Option<AhoCorasick>,
    penalty_ac: Option<AhoCorasick>,
    min_score: f64,
    score_cap: f64,
    boost_delta: f64,
    penalty_delta: f64,
}

impl IbanRecognizer {
    pub fn new() -> Self {
        // Under `(?x)`, a literal space inside `[ \t.-]` was ignored → no match on ASCII spaces.
        // `(?:\s|[.\-])*` covers spaces, dots and dashes. No trailing `\b`: trim until a valid IBAN.
        let pattern = Regex::new(
            r"(?xi)
            \b
            ([A-Za-z]{2}\d{2}(?:(?:\s|[.\-])*[A-Za-z0-9]){11,35})
            ",
        )
        .expect("iban regex");
        let words = iban_context_words();
        let boost_ac = build_ac(&words);
        let penalty_ac = build_ac(&["example.com", "test.invalid", "fake-iban", "dummy"]);
        Self {
            pattern,
            boost_ac,
            penalty_ac,
            min_score: 0.35,
            score_cap: 1.0,
            boost_delta: 0.05,
            penalty_delta: 0.12,
        }
    }

    fn score_for_normalized(&self, normalized: &str) -> Option<f64> {
        if !iban_format_plausible(normalized) {
            return None;
        }
        let mod_ok = iban_mod97_valid(normalized);
        let base = if mod_ok { 0.95 } else { 0.4 };
        Some(base)
    }
}

impl Default for IbanRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Recognizer for IbanRecognizer {
    fn name(&self) -> &str {
        "iban_iso13616"
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![EntityType::Iban]
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl", "ro", "sv"]
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        let mut search_at = 0usize;
        while search_at < text.len() {
            let Some(m) = self.pattern.find_at(text, search_at) else {
                break;
            };
            let start = m.start();
            let mut end = m.end();
            let mut normalized_opt = None;
            let mut base_opt = None;
            while end > start + 14 {
                if let Some(slice) = text.get(start..end) {
                    let n = normalize_iban(slice);
                    if let Some(b) = self.score_for_normalized(&n) {
                        normalized_opt = Some(n);
                        base_opt = Some(b);
                        break;
                    }
                }
                end -= 1;
                while end > start && !text.is_char_boundary(end) {
                    end -= 1;
                }
            }
            let (normalized, base) = match (normalized_opt, base_opt) {
                (Some(n), Some(b)) => (n, b),
                _ => {
                    search_at = start.saturating_add(1);
                    continue;
                }
            };
            let Some(score) = adjust_score_in_context(
                base,
                text,
                start,
                end,
                config,
                self.boost_ac.as_ref(),
                self.boost_delta,
                self.penalty_ac.as_ref(),
                self.penalty_delta,
                None,
                0.15,
                self.min_score,
                self.score_cap,
            ) else {
                search_at = end.max(start + 1);
                continue;
            };
            if score < self.min_score {
                search_at = end.max(start + 1);
                continue;
            }
            let mut metadata = HashMap::new();
            metadata.insert("level".into(), "regex".into());
            metadata.insert("iban_normalized".into(), normalized.clone());
            metadata.insert(
                "checksum_ok".into(),
                iban_mod97_valid(&normalized).to_string(),
            );
            if config.return_decision_process {
                metadata.insert("base_score".into(), format!("{:.4}", base));
            }
            let raw = text.get(start..end).unwrap_or("").to_string();
            out.push(Entity {
                entity_type: EntityType::Iban,
                start,
                end,
                text: raw,
                score,
                recognizer_name: self.name().to_string(),
                metadata,
                decision_trace: None,
            });
            search_at = end.max(start + 1);
        }
        out
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}

pub fn iban_recognizer() -> IbanRecognizer {
    IbanRecognizer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Recognizer;
    fn cfg() -> AnalysisConfig {
        AnalysisConfig::default()
    }

    #[test]
    fn mod97_fr_known() {
        let n = normalize_iban("FR7630006000011234567890189");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_de_known() {
        let n = normalize_iban("DE89370400440532013000");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_es_known() {
        let n = normalize_iban("ES9121000418450200051332");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_it_known() {
        let n = normalize_iban("IT60X0542811101000000123456");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_nl_known() {
        let n = normalize_iban("NL91ABNA0417164300");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_be_known() {
        let n = normalize_iban("BE68539007547034");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_gb_known() {
        let n = normalize_iban("GB82WEST12345698765432");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_pl_known() {
        let n = normalize_iban("PL61109010140000071219812874");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_pt_known() {
        let n = normalize_iban("PT50000201231234567890154");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_se_known() {
        let n = normalize_iban("SE4550000000058398257466");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_ch_known() {
        let n = normalize_iban("CH9300762011623852957");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn mod97_invalid_checksum() {
        let n = normalize_iban("FR7630006000011234567890190");
        assert!(!iban_mod97_valid(&n));
    }

    #[test]
    fn spaced_fr_detected_high_score() {
        let r = IbanRecognizer::new();
        let v = r.analyze("IBAN FR76 3000 6000 0112 3456 7890 189 ok", &cfg());
        assert_eq!(v.len(), 1);
        assert!(v[0].score >= 0.9);
    }

    #[test]
    fn wrong_length_fr_rejected() {
        let r = IbanRecognizer::new();
        let v = r.analyze("FR76300060000112345678901", &cfg());
        assert!(v.is_empty());
    }

    #[test]
    fn bad_checksum_low_score_still_detected() {
        let r = IbanRecognizer::new();
        let v = r.analyze("RIB FR7630006000011234567890190", &cfg());
        assert_eq!(v.len(), 1);
        assert!((0.35..0.5).contains(&v[0].score));
    }

    #[test]
    fn invalid_checksum_base_score_04() {
        let r = IbanRecognizer::new();
        let v = r.analyze("RIB FR7630006000011234567890190", &cfg());
        assert_eq!(v.len(), 1);
        assert!((0.38..0.48).contains(&v[0].score));
    }

    #[test]
    fn lu_iban() {
        let n = normalize_iban("LU280019400644750000");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn at_iban() {
        let n = normalize_iban("AT611904300234573201");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn ee_iban_sample() {
        let n = normalize_iban("EE382200221020145685");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn cy_iban_sample() {
        let n = normalize_iban("CY17002001280000001200527600");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn bg_iban_sample() {
        let n = normalize_iban("BG80BNBG96611020345678");
        assert!(iban_mod97_valid(&n));
    }

    #[test]
    fn normalize_strips_spaces() {
        assert_eq!(
            normalize_iban("GB82 WEST 1234 5698 7654 32"),
            "GB82WEST12345698765432"
        );
    }

    #[test]
    fn metadata_checksum_flag() {
        let r = IbanRecognizer::new();
        let mut c = cfg();
        c.return_decision_process = true;
        let v = r.analyze("IBAN FR7630006000011234567890189", &c);
        assert_eq!(v[0].metadata.get("checksum_ok"), Some(&"true".into()));
    }

    /// Typical "contact" line (em dash, example.com email): IBAN must still be extracted.
    #[test]
    fn iban_in_busy_fr_contact_line() {
        let r = IbanRecognizer::new();
        let s = "Contact : Jean Dupont — tél. +33 6 12 34 56 78 — marie.durand@example.com — IBAN FR7630006000011234567890189 — NIR 1 85 05 75 806 043 75";
        let v = r.analyze(s, &cfg());
        assert_eq!(v.len(), 1);
        assert!(v[0].text.contains("FR76"));
    }
}
