// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! EU phones: validation via `phonenumber` (libphonenumber), international or national format.

use crate::recognizers::financial::common::{adjust_score_in_context, build_ac};
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use phonenumber::country;
use phonenumber::metadata::DATABASE;
use phonenumber::{self, Type};
use regex::Regex;
use std::collections::HashMap;

/// EU member states (CLDR) — resolved country code must be one of them.
const EU_MEMBER: &[country::Id] = &[
    country::AT,
    country::BE,
    country::BG,
    country::HR,
    country::CY,
    country::CZ,
    country::DK,
    country::EE,
    country::FI,
    country::FR,
    country::DE,
    country::GR,
    country::HU,
    country::IE,
    country::IT,
    country::LV,
    country::LT,
    country::LU,
    country::MT,
    country::NL,
    country::PL,
    country::PT,
    country::RO,
    country::SK,
    country::SI,
    country::ES,
    country::SE,
];

fn is_eu_member(id: country::Id) -> bool {
    EU_MEMBER.contains(&id)
}

/// Parse and validate; returns line type (fixed/mobile/…) when metadata allows.
pub fn parse_valid_eu_phone(s: &str) -> Option<(phonenumber::PhoneNumber, Type)> {
    let t = s.trim();
    if !phonenumber::is_viable(t) {
        return None;
    }
    let normalized = if let Some(rest) = t.strip_prefix("00") {
        format!("+{}", rest)
    } else {
        t.to_string()
    };
    let pn = if normalized.starts_with('+') {
        phonenumber::parse(None, &normalized).ok()?
    } else {
        EU_MEMBER
            .iter()
            .copied()
            .find_map(|id| phonenumber::parse(Some(id), &normalized).ok())?
    };
    if !pn.is_valid() {
        return None;
    }
    let cid = pn.country().id()?;
    if !is_eu_member(cid) {
        return None;
    }
    let nt = pn.number_type(&DATABASE);
    Some((pn, nt))
}

fn line_type_label(t: Type) -> &'static str {
    match t {
        Type::FixedLine => "fixed_line",
        Type::Mobile => "mobile",
        Type::FixedLineOrMobile => "fixed_or_mobile",
        Type::Voip => "voip",
        Type::PersonalNumber => "personal",
        Type::Pager => "pager",
        Type::Uan => "uan",
        Type::TollFree => "toll_free",
        Type::PremiumRate => "premium_rate",
        Type::SharedCost => "shared_cost",
        Type::Emergency => "emergency",
        Type::Voicemail => "voicemail",
        Type::ShortCode => "short_code",
        Type::StandardRate => "standard_rate",
        Type::Carrier => "carrier",
        Type::NoInternational => "no_international",
        Type::Unknown => "unknown",
    }
}

const BOOST: &[&str] = &[
    "téléphone",
    "telephone",
    "tel",
    "Tel.",
    "mobile",
    "portable",
    "Telefon",
    "Handy",
    "telefono",
    "cellulare",
    "teléfono",
    "móvil",
    "telefoon",
    "gsm",
    "WhatsApp",
    "WhatsApp Business",
    "call",
    "appeler",
    "numéro",
    "numero",
];

/// Dedicated recognizer: regex candidates + `phonenumber` validation.
pub struct EuExtendedPhoneRecognizer {
    pattern: Regex,
    boost_ac: Option<aho_corasick::AhoCorasick>,
    penalty_ac: Option<aho_corasick::AhoCorasick>,
    min_score: f64,
    base_score: f64,
    score_cap: f64,
}

impl EuExtendedPhoneRecognizer {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"(?xi)
            \+\s*\d{1,3}[\s.\-/()]*(?:\d[\s.\-/()]*){5,14}\d
            |
            \b0\d(?:[\s.\-/()]*\d){7,12}\d\b
            ",
        )
        .expect("eu phone regex");
        Self {
            pattern,
            boost_ac: build_ac(BOOST),
            penalty_ac: build_ac(&["example", "test", "dummy", "sample", "555-"]),
            min_score: 0.38,
            base_score: 0.82,
            score_cap: 1.0,
        }
    }
}

impl Default for EuExtendedPhoneRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Recognizer for EuExtendedPhoneRecognizer {
    fn name(&self) -> &str {
        "eu_phone_phonenumber"
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![EntityType::Phone]
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["*"]
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        for m in self.pattern.find_iter(text) {
            let slice = m.as_str();
            if m.start() > 0 {
                let prev = text.as_bytes()[m.start() - 1];
                if prev.is_ascii_digit() || prev == b'+' {
                    continue;
                }
            }
            if m.end() < text.len() && text.as_bytes()[m.end()].is_ascii_digit() {
                continue;
            }
            let Some((pn, nt)) = parse_valid_eu_phone(slice) else {
                continue;
            };
            let mut score: f64 = match adjust_score_in_context(
                self.base_score,
                text,
                m.start(),
                m.end(),
                config,
                self.boost_ac.as_ref(),
                0.06,
                self.penalty_ac.as_ref(),
                0.1,
                None,
                0.1,
                self.min_score,
                self.score_cap,
            ) {
                Some(s) => s,
                None => continue,
            };
            score = score.max(self.min_score);
            if score < self.min_score {
                continue;
            }
            let mut metadata = HashMap::new();
            metadata.insert("level".into(), "regex".into());
            metadata.insert("phone_line_type".into(), line_type_label(nt).into());
            if let Some(region) = pn.country().id() {
                metadata.insert("phone_region".into(), format!("{region:?}"));
            }
            if config.return_decision_process {
                metadata.insert("phonenumber_valid".into(), "true".into());
            }
            out.push(Entity {
                entity_type: EntityType::Phone,
                start: m.start(),
                end: m.end(),
                text: slice.to_string(),
                score,
                recognizer_name: self.name().to_string(),
                metadata,
                decision_trace: None,
            });
        }
        out
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}

pub fn eu_extended_phone_recognizer() -> EuExtendedPhoneRecognizer {
    EuExtendedPhoneRecognizer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_be_mobile_e164() {
        // Test number consistent with libphonenumber fixtures (valid format).
        let r = parse_valid_eu_phone("+32 474 09 11 50");
        assert!(r.is_some());
        let (_, t) = r.unwrap();
        assert_eq!(t, Type::Mobile);
    }

    #[test]
    fn rejects_us_nanp() {
        assert!(parse_valid_eu_phone("+1 415 555 0199").is_none());
    }

    #[test]
    fn rejects_gibberish() {
        assert!(parse_valid_eu_phone("+33 1 2").is_none());
    }
}
