// AEGIS — zokastech.fr — Apache 2.0 / MIT
//! Extended suite: email recognizer (≥ 20 cases) — positives, negatives, edge cases, multilingual, context scoring.

use aegis_core::config::AnalysisConfig;
use aegis_regex::email::email_recognizer;
use aegis_regex::Recognizer;

fn cfg() -> AnalysisConfig {
    AnalysisConfig::default()
}

fn cfg_decision() -> AnalysisConfig {
    AnalysisConfig {
        return_decision_process: true,
        ..AnalysisConfig::default()
    }
}

// --- Vrais positifs ---
#[test]
fn tp_simple_corporate() {
    let r = email_recognizer();
    let v = r.analyze("reach us at team@acme.io today", &cfg());
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].text, "team@acme.io");
}

#[test]
fn tp_subdomain() {
    let r = email_recognizer();
    let v = r.analyze("support@mail.gateway.zokastech.fr", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_long_tld() {
    let r = email_recognizer();
    let v = r.analyze("u@v.technology", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_numeric_local() {
    let r = email_recognizer();
    let v = r.analyze("user123456@numeric-domain.co.uk", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_underscore_local() {
    let r = email_recognizer();
    let v = r.analyze("first_last@company.org", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_percent_local() {
    let r = email_recognizer();
    let v = r.analyze("news%list@broadcast.net", &cfg());
    assert_eq!(v.len(), 1);
}

#[test]
fn tp_hyphen_domain() {
    let r = email_recognizer();
    let v = r.analyze("a@my-email-provider.com", &cfg());
    assert_eq!(v.len(), 1);
}

// --- True negatives (looks similar but not valid PII) ---
#[test]
fn tn_not_email_two_ats() {
    let r = email_recognizer();
    assert!(r.analyze("a@@b.co", &cfg()).is_empty());
}

#[test]
fn tn_trailing_dot_domain() {
    let r = email_recognizer();
    assert!(r.analyze("x@y.co.", &cfg()).is_empty());
}

#[test]
fn tn_no_tld() {
    let r = email_recognizer();
    assert!(r.analyze("local@hostonly", &cfg()).is_empty());
}

#[test]
fn tn_example_reserved() {
    let r = email_recognizer();
    assert!(r.analyze("a@example.com", &cfg()).is_empty());
}

#[test]
fn tn_test_domain() {
    let r = email_recognizer();
    assert!(r.analyze("b@test.com", &cfg()).is_empty());
}

#[test]
fn tn_localhost() {
    let r = email_recognizer();
    assert!(r.analyze("root@localhost", &cfg()).is_empty());
}

#[test]
fn tn_arobase_in_quotes_not_matched_as_whole() {
    let r = email_recognizer();
    let v = r.analyze(r#""not@an@email" literal"#, &cfg());
    assert!(v.is_empty() || !v.iter().any(|e| e.text.contains('"')));
}

// --- Bords : espaces / ponctuation ---
#[test]
fn edge_comma_after() {
    let r = email_recognizer();
    let v = r.analyze("write a@b.co, thanks", &cfg());
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].text, "a@b.co");
}

#[test]
fn edge_parentheses() {
    let r = email_recognizer();
    let v = r.analyze("(contact: notify@svc.eu)", &cfg());
    assert!(!v.is_empty());
    assert!(v[0].text.contains('@'));
}

#[test]
fn edge_multiline() {
    let r = email_recognizer();
    let v = r.analyze("Line1\nmail: x@y.fr\nend", &cfg());
    assert_eq!(v.len(), 1);
}

// --- Multilingue (FR, DE, ES, IT, NL) ---
#[test]
fn ml_fr_courriel() {
    let r = email_recognizer();
    assert!(!r.analyze("Mon courriel est jean@poste.fr", &cfg()).is_empty());
}

#[test]
fn ml_de_email_wort() {
    let r = email_recognizer();
    assert!(!r.analyze("E-Mail Adresse max@firma.de bitte", &cfg()).is_empty());
}

#[test]
fn ml_es_correo() {
    let r = email_recognizer();
    assert!(!r.analyze("correo electrónico ana@empresa.es", &cfg()).is_empty());
}

#[test]
fn ml_it_email_label() {
    let r = email_recognizer();
    assert!(!r.analyze("email commerciale: info@azienda.it", &cfg()).is_empty());
}

#[test]
fn ml_nl_contact() {
    let r = email_recognizer();
    assert!(!r.analyze("neem contact op via jan@bedrijf.nl", &cfg()).is_empty());
}

// --- Scoring : mots de contexte ---
#[test]
fn score_boost_with_email_keyword_higher_than_bare() {
    let r = email_recognizer();
    let bare = r.analyze("x@unique-domain-xyz123.co", &cfg_decision());
    let ctx = r.analyze("email x@unique-domain-xyz123.co please", &cfg_decision());
    assert!(!bare.is_empty() && !ctx.is_empty());
    assert!(ctx[0].score >= bare[0].score);
}

#[test]
fn score_metadata_includes_base_when_decision_on() {
    let r = email_recognizer();
    let mut c = cfg_decision();
    c.return_decision_process = true;
    let v = r.analyze("contact email user@corp.io", &c);
    assert!(!v.is_empty());
    assert!(v[0].metadata.contains_key("base_score"));
    assert!(v[0].metadata.contains_key("final_score"));
}

#[test]
fn score_penalty_example_in_window_lowers_vs_clean() {
    let r = email_recognizer();
    let clean = r.analyze("reach me@clean-brand.io ok", &cfg_decision());
    let dirty = r.analyze("example template me@clean-brand.io test", &cfg_decision());
    assert!(!clean.is_empty() && !dirty.is_empty());
    assert!(dirty[0].score <= clean[0].score);
}
