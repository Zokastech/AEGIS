// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! **Level-1** detection engine (regex + validation + Aho–Corasick) for **AEGIS**.
//!
//! On startup, the default recognizer loader registers with `aegis-core` for
//! [`aegis_core::AnalyzerEngineBuilder::with_default_recognizers`].
//!
//! - [`PatternRecognizer`](pattern::PatternRecognizer): regex, optional validator, deny-list, and context.
//! - Pre-built recognizers: email, phone, payment card, IP, URL, date, crypto wallet.
//! - [`MultiPatternScanner`](multi_pattern::MultiPatternScanner): multiple literal patterns in one pass.
//!
//! ## Conventions
//!
//! Constant patterns use [`static_regex::compile`] (`#[track_caller]`) so a bad embedded pattern
//! produces a localized panic message.

#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]

use std::sync::Arc;

pub use aegis_core::recognizer::Recognizer;

mod static_regex;
mod utf8_window;

pub mod context_lexicon;
pub mod credit_card;
pub mod crypto_wallet;
pub mod date;
pub mod defaults;
pub mod email;
pub mod ip_address;
pub mod multi_pattern;
pub mod pattern;
pub mod phone;
pub mod recognizers;
pub mod url_recognizer;
pub mod validation;

pub use credit_card::credit_card_recognizer;
pub use crypto_wallet::crypto_wallet_recognizer;
pub use date::date_recognizer;
pub use defaults::default_regex_recognizers;

fn default_loader_bridge(langs: &[&str]) -> Vec<Arc<dyn Recognizer>> {
    default_regex_recognizers(langs)
}

#[ctor::ctor]
fn _register_default_regex_loader() {
    aegis_core::register_default_regex_loader(default_loader_bridge);
}
pub use email::email_recognizer;
pub use ip_address::{ipv4_recognizer, ipv6_recognizer};
pub use multi_pattern::{LiteralMatch, MultiPatternScanner};
pub use pattern::PatternRecognizer;
pub use phone::phone_recognizer;
pub use recognizers::financial::{
    bic_country_plausible, bic_structure_ok, bic_swift_recognizer, eu_credit_card_recognizer,
    eu_vat_recognizer, financial_recognizers, iban_mod97_valid, iban_recognizer, nir_key_ok,
    nir_match_validate, nir_recognizer, nir_shape_ok, normalize_iban, siren_luhn_ok,
    siren_recognizer, siret_luhn_ok,
    siret_recognizer, IbanRecognizer,
};
pub use recognizers::eu::all_eu_recognizers;
pub use recognizers::national_id::all_eu_national_id_recognizers;
pub use url_recognizer::url_recognizer;
pub use validation::{
    credit_card_network_ok, digits_only, email_rfc5322_pragmatic, is_btc_base58, luhn_valid,
    validate_btc_bech32, validate_btc_p2pkh_p2sh, validate_credit_card_match,
    validate_ethereum_address,
};
