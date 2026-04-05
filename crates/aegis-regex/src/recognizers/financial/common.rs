// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Context words (10 languages: EN, FR, DE, ES, IT, NL, PT, PL, RO, SV) and window scoring.

use crate::utf8_window::byte_window_utf8;
use aho_corasick::AhoCorasick;
use aegis_core::config::AnalysisConfig;

pub fn build_ac(patterns: &[&str]) -> Option<AhoCorasick> {
    if patterns.is_empty() {
        return None;
    }
    let pats: Vec<Vec<u8>> = patterns.iter().map(|w| w.as_bytes().to_vec()).collect();
    AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(pats)
        .ok()
}

pub fn context_window_bytes(config: &AnalysisConfig) -> usize {
    config.context_window_size.max(1).saturating_mul(12)
}

/// Adjust a base score with boosts / penalties (same logic as [`crate::pattern::PatternRecognizer`]).
pub fn adjust_score_in_context(
    mut score: f64,
    text: &str,
    start: usize,
    end: usize,
    config: &AnalysisConfig,
    boost_ac: Option<&AhoCorasick>,
    boost_delta: f64,
    penalty_ac: Option<&AhoCorasick>,
    penalty_delta: f64,
    invalidate_ac: Option<&AhoCorasick>,
    invalidate_penalty: f64,
    min_score: f64,
    score_cap: f64,
) -> Option<f64> {
    let window = context_window_bytes(config);
    let lo = start.saturating_sub(window);
    let hi = (end + window).min(text.len());
    let ctx_bytes = byte_window_utf8(text, lo, hi).as_bytes();

    if let Some(ac) = boost_ac {
        if ac.find_iter(ctx_bytes).next().is_some() {
            score = (score + boost_delta).min(score_cap);
        }
    }
    if let Some(ac) = penalty_ac {
        for _ in ac.find_iter(ctx_bytes) {
            score = (score - penalty_delta).max(min_score);
        }
    }
    if let Some(ac) = invalidate_ac {
        for _ in ac.find_iter(ctx_bytes) {
            score = (score - invalidate_penalty).max(min_score);
        }
    }
    Some(score.min(score_cap))
}

/// Contexte positif financier — 10 langues (EN, FR, DE, ES, IT, NL, PT, PL, RO, SV).
pub fn iban_context_words() -> Vec<&'static str> {
    vec![
        "IBAN",
        "bank account",
        "bank transfer",
        "wire transfer",
        "compte bancaire",
        "coordonnées bancaires",
        "RIB",
        "relevé d'identité bancaire",
        "Bankverbindung",
        "Kontonummer",
        "Bankkonto",
        "Überweisung",
        "cuenta bancaria",
        "cuenta corriente",
        "IBAN bancario",
        "domiciliación",
        "conto corrente",
        "coordinate bancarie",
        "bonifico",
        "bankrekening",
        "rekeningnummer",
        "overschrijving",
        "conta bancária",
        "NIB",
        "transferência",
        "numer konta",
        "przelew",
        "rachunek bankowy",
        "cont bancar",
        "cont curent",
        "transfer bancar",
        "bankkonto",
        "bankgiro",
        "kontonummer",
        "plusgiro",
    ]
}

pub fn bic_swift_context_words() -> Vec<&'static str> {
    vec![
        "BIC",
        "SWIFT",
        "SWIFT code",
        "bank identifier",
        "code banque",
        "identifiant banque",
        "Bankleitzahl",
        "Bankcode",
        "código SWIFT",
        "código BIC",
        "codice SWIFT",
        "codice BIC",
        "bankcode",
        "BIC code",
        "código bancário",
        "identificador bancário",
        "kod SWIFT",
        "identyfikator banku",
        "cod bancar",
        "cod SWIFT",
        "bankidentifiering",
        "bankkod",
    ]
}

pub fn vat_context_words() -> Vec<&'static str> {
    vec![
        "VAT",
        "VAT number",
        "TVA",
        "numéro de TVA",
        "MwSt",
        "USt-IdNr",
        "Umsatzsteuer",
        "IVA",
        "NIF-IVA",
        "partita IVA",
        "P.IVA",
        "BTW",
        "btw-nummer",
        "NIF",
        "número de IVA",
        "NIP",
        "numer VAT",
        "TVA intracommunautaire",
        "CIF",
        "CUI",
        "moms",
        "momssats",
        "VAT ID",
        "TIN",
    ]
}

pub fn eu_card_context_words() -> Vec<&'static str> {
    vec![
        "Visa",
        "Mastercard",
        "MasterCard",
        "American Express",
        "AmEx",
        "Carte Bleue",
        "carte bancaire",
        "CB",
        "Bancontact",
        "Maestro",
        "CartaSi",
        "carta di credito",
        "tarjeta bancaria",
        "bank card",
        "debitcard",
        "credit card",
        "Kreditkarte",
        "EC-Karte",
        "Girocard",
        "betaalpas",
        "pas kredytowy",
        "card bancar",
        "betalningskort",
        "betalningsmedel",
    ]
}

pub fn fr_id_context_words() -> Vec<&'static str> {
    vec![
        "SIREN",
        "SIRET",
        "NIR",
        "numéro de sécurité sociale",
        "sécurité sociale",
        "INSEE",
        "identifiant entreprise",
        "RCS",
        "RM",
        "entreprise",
        "company number",
        "firmennummer",
        "Unternehmens-ID",
        "CIF empresa",
        "partita IVA azienda",
        "KVK",
        "NIP firmy",
        "social security number",
        "personnummer",
        "număr de identificare",
    ]
}
