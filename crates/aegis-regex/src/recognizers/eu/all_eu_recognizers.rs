// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Entry point: all EU-pack recognizers (extensions + national IDs).

use super::{eu_address, eu_health, eu_phone, gdpr_sensitive, license_plate, quasi_identifiers};
use crate::recognizers::national_id;
use aegis_core::recognizer::Recognizer;
use std::sync::Arc;

/// Returns all EU-pack recognizers: plates, phones (`phonenumber` validation), addresses,
/// health, GDPR art. 9 indicators, quasi-identifiers, then [`national_id::all_eu_national_id_recognizers`].
///
/// `languages` is passed to the national-ID filter (empty = all countries).
pub fn all_eu_recognizers(languages: &[&str]) -> Vec<Arc<dyn Recognizer>> {
    let mut v: Vec<Arc<dyn Recognizer>> = vec![
        Arc::new(license_plate::eu_license_plate_recognizer()),
        Arc::new(eu_phone::eu_extended_phone_recognizer()),
        Arc::new(eu_address::eu_address_recognizer()),
        Arc::new(eu_health::eu_health_recognizer()),
        Arc::new(gdpr_sensitive::gdpr_art9_sensitive_recognizer()),
        Arc::new(quasi_identifiers::quasi_identifier_recognizer()),
    ];
    v.extend(national_id::all_eu_national_id_recognizers(languages));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_includes_national_ids() {
        let v = all_eu_recognizers(&[]);
        assert!(v.len() >= 14);
    }

    #[test]
    fn filters_national_by_lang() {
        let all = all_eu_recognizers(&[]).len();
        let fr = all_eu_recognizers(&["fr"]).len();
        assert!(fr < all);
    }
}
