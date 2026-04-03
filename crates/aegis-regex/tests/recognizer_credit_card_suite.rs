// AEGIS — zokastech.fr — Apache 2.0 / MIT
//! Extended suite: payment card (Luhn) — ≥ 20 cases.

use aegis_core::config::AnalysisConfig;
use aegis_regex::credit_card::credit_card_recognizer;
use aegis_regex::Recognizer;

fn cfg() -> AnalysisConfig {
    AnalysisConfig::default()
}

fn cfg_d() -> AnalysisConfig {
    AnalysisConfig {
        return_decision_process: true,
        ..AnalysisConfig::default()
    }
}

// Luhn-valid Visa test PAN (industry reference)
const VISA_TEST: &str = "4532015112830366";
const MC_TEST: &str = "5425233430109903";

#[test]
fn tp_visa_compact() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("pay {VISA_TEST} now"), &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_visa_spaced_groups() {
    let r = credit_card_recognizer();
    let v = r.analyze("card 4532 0151 1283 0366 ok", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_visa_dashed() {
    let r = credit_card_recognizer();
    let v = r.analyze("4532-0151-1283-0366", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_mc_compact() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("MC {MC_TEST}"), &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tn_invalid_luhn_digit_flip() {
    let r = credit_card_recognizer();
    assert!(r.analyze("4532015112830367", &cfg()).is_empty());
}

#[test]
fn tn_short_digit_run() {
    let r = credit_card_recognizer();
    assert!(r.analyze("id 123456789012", &cfg()).is_empty());
}

#[test]
fn tn_11_digits_only() {
    let r = credit_card_recognizer();
    assert!(r.analyze("12345678901", &cfg()).is_empty());
}

#[test]
fn tn_isbn_like_without_luhn() {
    let r = credit_card_recognizer();
    assert!(r.analyze("ISBN 978-2-123456-78-9", &cfg()).is_empty());
}

#[test]
fn edge_spaces_between_digits() {
    let r = credit_card_recognizer();
    let v = r.analyze("4 5 3 2 0 1 5 1 1 2 8 3 0 3 6 6", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn edge_trailing_comma() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("{VISA_TEST},"), &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn ml_fr_carte_bancaire() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("carte bancaire {VISA_TEST}"), &cfg());
    assert!(!v.is_empty());
}

#[test]
fn ml_de_kreditkarte() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("Kreditkarte {VISA_TEST} bitte"), &cfg());
    assert!(!v.is_empty());
}

#[test]
fn ml_es_tarjeta() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("tarjeta {VISA_TEST}"), &cfg());
    assert!(!v.is_empty());
}

#[test]
fn ml_it_carta() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("carta {VISA_TEST}"), &cfg());
    assert!(!v.is_empty());
}

#[test]
fn ml_nl_kaart() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("betaalkaart {VISA_TEST}"), &cfg());
    assert!(!v.is_empty());
}

#[test]
fn score_card_context_vs_random_digits() {
    let r = credit_card_recognizer();
    let ctx = r.analyze(&format!("payment card {VISA_TEST}"), &cfg_d());
    let rnd = r.analyze(&format!("sku {VISA_TEST} warehouse"), &cfg_d());
    assert!(!ctx.is_empty() && !rnd.is_empty());
    assert!(ctx[0].score >= rnd[0].score);
}

#[test]
fn score_penalty_sample_in_window() {
    let r = credit_card_recognizer();
    let clean = r.analyze(&format!("charge {VISA_TEST}"), &cfg_d());
    let dirty = r.analyze(&format!("sample data {VISA_TEST} test"), &cfg_d());
    assert!(!clean.is_empty() && !dirty.is_empty());
    assert!(dirty[0].score <= clean[0].score);
}

#[test]
fn tp_amex_style_15() {
    let r = credit_card_recognizer();
    // 378282246310005 — Luhn valide AmEx test
    let v = r.analyze("378282246310005", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tn_all_zeros() {
    let r = credit_card_recognizer();
    assert!(r.analyze("0000000000000000", &cfg()).is_empty());
}

#[test]
fn tn_all_nines_invalid() {
    let r = credit_card_recognizer();
    assert!(r.analyze("9999999999999999", &cfg()).is_empty());
}

#[test]
fn metadata_scores_when_decision() {
    let r = credit_card_recognizer();
    let v = r.analyze(&format!("visa {VISA_TEST}"), &cfg_d());
    assert!(v[0].metadata.contains_key("base_score"));
}
