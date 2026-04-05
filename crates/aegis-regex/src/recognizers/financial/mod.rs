// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Financial recognizers and French identifiers (SIREN, SIRET, NIR).

mod bic;
pub(crate) mod common;
mod eu_credit_card;
mod fr_ids;
mod iban;
mod vat;

pub use bic::{bic_country_plausible, bic_structure_ok, bic_swift_recognizer};
pub use eu_credit_card::eu_credit_card_recognizer;
pub use fr_ids::{
    nir_key_ok, nir_match_validate, nir_recognizer, nir_shape_ok, siren_luhn_ok, siren_recognizer,
    siret_luhn_ok, siret_recognizer,
};
pub use iban::{iban_mod97_valid, iban_recognizer, normalize_iban, IbanRecognizer};
pub use vat::eu_vat_recognizer;

use aegis_core::recognizer::Recognizer;
use std::sync::Arc;

/// Ensemble des recognizers financiers / FR pour enregistrement dans le moteur.
pub fn financial_recognizers() -> Vec<Arc<dyn Recognizer>> {
    vec![
        Arc::new(iban_recognizer()),
        Arc::new(bic_swift_recognizer()),
        Arc::new(eu_vat_recognizer()),
        Arc::new(eu_credit_card_recognizer()),
        Arc::new(siren_recognizer()),
        Arc::new(siret_recognizer()),
        Arc::new(nir_recognizer()),
    ]
}
