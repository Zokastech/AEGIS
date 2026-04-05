// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! EU recognizers: license plates, phones (libphonenumber), addresses, health, GDPR Art. 9, quasi-identifiers.

pub mod all_eu_recognizers;
pub mod eu_address;
pub mod eu_health;
pub mod eu_phone;
pub mod gdpr_sensitive;
pub mod license_plate;
pub mod quasi_identifiers;

pub use all_eu_recognizers::all_eu_recognizers;
pub use gdpr_sensitive::GdprArt9Recognizer;
pub use eu_phone::EuExtendedPhoneRecognizer;
