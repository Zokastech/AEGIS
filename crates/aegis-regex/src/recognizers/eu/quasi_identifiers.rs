// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Quasi-identifier combinations (local window) and re-identification risk score.
//!
//! Inspired by Sweeney’s k-anonymity principle: a few combined dimensions often suffice
//! to single out an individual in a dataset.

use super::eu_address::eu_postal_format_ok;
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::OnceLock;

fn postal_finder() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?xi)
            \b\d{5}\b
            |
            \b\d{4}\s*[A-Za-z]{2}\b
            |
            \b[A-Za-z]{1,2}\d[A-Za-z0-9]?\s*\d[A-Za-z]{2}\b
            |
            \b\d{2}-\d{3}\b
            |
            \b\d{3}\s?\d{2}\b
            ",
        )
        .expect("postal finder")
    })
}

fn date_finder() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?xi)
            \b\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}\b
            |
            \b(?:né|née|born|geboren|geb\.|DOB|date\s+of\s+birth|date\s+de\s+naissance)\b
            [^\n]{0,24}\b\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}\b
            |
            \b(?:19|20)\d{2}\b(?:\s*(?:year|ans|Jahr|años))?
            ",
        )
        .expect("date finder")
    })
}

fn gender_finder() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?xi)
            \b(?:male|female|masculin|féminin|feminin|homme|femme|mann|frau|mujer|hombre)\b
            |
            \bgenre\s*[:/]\s*(?:M|F|H|X)\b
            |
            \bsex\s*[:/]\s*(?:M|F)\b
            ",
        )
        .expect("gender finder")
    })
}

fn profession_combo() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?xi)
            \b(?:ingénieur|engineer|médecin|physician|avocat|lawyer|enseignant|teacher|nurse|infirmier|developer|consultant)\b
            [\s\S]{0,140}?
            \b(?:chez|at|@|bei|bij)\s+([A-Z0-9][A-Za-z0-9&\.\s-]{2,42})
            [\s\S]{0,120}?
            \b([A-Z][a-zà-ÿ]+(?:\s+[A-Z][a-zà-ÿ]+){0,2})\b
            ",
        )
        .expect("profession combo")
    })
}

/// Heuristic score in ~\[0.35, 0.58\] from the number of dimensions seen in the window.
fn risk_score(components: u8) -> f64 {
    match components {
        3 => 0.55,
        4.. => 0.58,
        _ => 0.38,
    }
}

pub struct QuasiIdentifierRecognizer {
    window: usize,
    min_score: f64,
}

impl Default for QuasiIdentifierRecognizer {
    fn default() -> Self {
        Self {
            window: 140,
            min_score: 0.32,
        }
    }
}

impl Recognizer for QuasiIdentifierRecognizer {
    fn name(&self) -> &str {
        "eu_quasi_identifiers"
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![EntityType::Custom("QUASI_IDENTIFIER".into())]
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["*"]
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut out = Vec::new();
        let w = self.window;

        for m in postal_finder().find_iter(text) {
            let pc = m.as_str();
            if !eu_postal_format_ok(pc) {
                continue;
            }
            let lo = m.start().saturating_sub(w);
            let hi = min(m.end() + w, text.len());
            let Some(win) = text.get(lo..hi) else {
                continue;
            };
            let has_d = date_finder().is_match(win);
            let has_g = gender_finder().is_match(win);
            if !has_d || !has_g {
                continue;
            }
            let ds = date_finder()
                .find(win)
                .map(|x| lo + x.start()..lo + x.end());
            let gs = gender_finder()
                .find(win)
                .map(|x| lo + x.start()..lo + x.end());
            let mut starts = vec![m.start()];
            let mut ends = vec![m.end()];
            if let Some(r) = ds {
                starts.push(r.start);
                ends.push(r.end);
            }
            if let Some(r) = gs {
                starts.push(r.start);
                ends.push(r.end);
            }
            let s = *starts.iter().min().unwrap();
            let e = *ends.iter().max().unwrap();
            let sc = risk_score(3).max(self.min_score);
            let mut metadata = HashMap::new();
            metadata.insert("level".into(), "regex".into());
            metadata.insert("quasi_pattern".into(), "postal_dob_gender".into());
            metadata.insert("reidentification_risk".into(), "elevated".into());
            metadata.insert("human_review_recommended".into(), "true".into());
            metadata.insert("sweeney_reference".into(), "k_anonymity".into());
            if config.return_decision_process {
                metadata.insert("components".into(), "3".into());
            }
            out.push(Entity {
                entity_type: EntityType::Custom("QUASI_IDENTIFIER".into()),
                start: s,
                end: e,
                text: text.get(s..e).unwrap_or_default().to_string(),
                score: sc,
                recognizer_name: self.name().to_string(),
                metadata,
                decision_trace: None,
            });
        }

        for cap in profession_combo().captures_iter(text) {
            let full = cap.get(0).unwrap();
            let emp = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let city = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            if emp.len() < 3 || city.len() < 3 {
                continue;
            }
            let s = full.start();
            let e = full.end();
            let sc = risk_score(3);
            let mut metadata = HashMap::new();
            metadata.insert("level".into(), "regex".into());
            metadata.insert("quasi_pattern".into(), "profession_employer_city".into());
            metadata.insert("reidentification_risk".into(), "elevated".into());
            metadata.insert("human_review_recommended".into(), "true".into());
            out.push(Entity {
                entity_type: EntityType::Custom("QUASI_IDENTIFIER".into()),
                start: s,
                end: e,
                text: text.get(s..e).unwrap_or_default().to_string(),
                score: sc.max(self.min_score),
                recognizer_name: self.name().to_string(),
                metadata,
                decision_trace: None,
            });
        }

        out.sort_by_key(|e| e.start);
        dedupe_overlapping(out)
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }
}

fn dedupe_overlapping(mut v: Vec<Entity>) -> Vec<Entity> {
    v.sort_by_key(|e| (e.start, e.end));
    let mut keep: Vec<Entity> = Vec::new();
    for e in v {
        if let Some(last) = keep.last_mut() {
            if e.start < last.end && e.score > last.score {
                *last = e;
                continue;
            }
            if e.start < last.end {
                continue;
            }
        }
        keep.push(e);
    }
    keep
}

pub fn quasi_identifier_recognizer() -> QuasiIdentifierRecognizer {
    QuasiIdentifierRecognizer::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Recognizer;
    #[test]
    fn triple_in_window() {
        let r = QuasiIdentifierRecognizer::default();
        let cfg = AnalysisConfig::default();
        let text = "Patient: genre: M, né le 14/05/1980, résidence 75001 Paris";
        let v = r.analyze(text, &cfg);
        assert!(!v.is_empty());
    }
}
