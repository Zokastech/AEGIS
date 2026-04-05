// AEGIS — zokastech.fr — Apache 2.0 / MIT
//! Extended suite: EU / NANP phone (≥ 20 cases).

use aegis_core::config::AnalysisConfig;
use aegis_regex::phone::phone_recognizer;
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

#[test]
fn tp_fr_plus33_spaced() {
    let r = phone_recognizer();
    assert!(!r.analyze("Tel +33 7 65 43 21 09", &cfg()).is_empty());
}

#[test]
fn tp_fr_06_format() {
    let r = phone_recognizer();
    assert!(!r.analyze("Portable 06-11-22-33-44", &cfg()).is_empty());
}

#[test]
fn tp_de_plus49() {
    let r = phone_recognizer();
    assert!(!r.analyze("+49 89 12345678", &cfg()).is_empty());
}

#[test]
fn tp_es_plus34() {
    let r = phone_recognizer();
    assert!(!r.analyze("+34 612 345 678", &cfg()).is_empty());
}

#[test]
fn tp_it_plus39() {
    let r = phone_recognizer();
    assert!(!r.analyze("+39 320 1234567", &cfg()).is_empty());
}

#[test]
fn tp_nl_plus31() {
    let r = phone_recognizer();
    assert!(!r.analyze("+31 6 12345678", &cfg()).is_empty());
}

#[test]
fn tp_pt_plus351() {
    let r = phone_recognizer();
    assert!(!r.analyze("+351 912 345 678", &cfg()).is_empty());
}

#[test]
fn tp_pl_plus48() {
    let r = phone_recognizer();
    assert!(!r.analyze("+48 512 345 678", &cfg()).is_empty());
}

#[test]
fn tp_us_parentheses() {
    let r = phone_recognizer();
    assert!(!r.analyze("Call (212) 555-0199", &cfg()).is_empty());
}

#[test]
fn tp_us_dashes() {
    let r = phone_recognizer();
    assert!(!r.analyze("+1 415-555-2671", &cfg()).is_empty());
}

#[test]
fn tn_too_short_digits() {
    let r = phone_recognizer();
    assert!(r.analyze("+33 12 34", &cfg()).is_empty());
}

#[test]
fn tn_random_number_not_phone() {
    let r = phone_recognizer();
    assert!(r.analyze("order 9876543210 only digits", &cfg()).is_empty());
}

#[test]
fn tn_version_string() {
    let r = phone_recognizer();
    assert!(r.analyze("v1.2.3.4.5 build", &cfg()).is_empty());
}

#[test]
fn edge_dots_instead_of_spaces() {
    let r = phone_recognizer();
    assert!(!r.analyze("+33.6.12.34.56.78", &cfg()).is_empty());
}

#[test]
fn edge_slash_separators_de() {
    let r = phone_recognizer();
    assert!(!r.analyze("0 30 / 12345 67", &cfg()).is_empty());
}

#[test]
fn ml_fr_telephone_word() {
    let r = phone_recognizer();
    assert!(!r.analyze("téléphone +33 1 42 12 34 56", &cfg()).is_empty());
}

#[test]
fn ml_de_telefon() {
    let r = phone_recognizer();
    assert!(!r.analyze("Telefon +49 30 1234567", &cfg()).is_empty());
}

#[test]
fn ml_es_movil() {
    let r = phone_recognizer();
    assert!(!r.analyze("móvil +34 600 111 222", &cfg()).is_empty());
}

#[test]
fn ml_it_cellulare() {
    let r = phone_recognizer();
    assert!(!r.analyze("cellulare +39 333 444 5555", &cfg()).is_empty());
}

#[test]
fn ml_nl_bel() {
    let r = phone_recognizer();
    assert!(!r.analyze("bel +31 20 123 4567", &cfg()).is_empty());
}

#[test]
fn score_context_tel_keyword_vs_bare() {
    let r = phone_recognizer();
    let bare = r.analyze("+33 6 99 88 77 66 end", &cfg_d());
    let ctx = r.analyze("tel +33 6 99 88 77 66 end", &cfg_d());
    assert!(!bare.is_empty() && !ctx.is_empty());
    assert!(ctx[0].score >= bare[0].score);
}

#[test]
fn score_metadata_when_decision() {
    let r = phone_recognizer();
    let v = r.analyze("phone +44 20 7946 0958", &cfg_d());
    assert!(!v.is_empty());
    assert!(v[0].metadata.contains_key("final_score"));
}
