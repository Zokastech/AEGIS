// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! National ID recognizers (formats and checksums per country).

pub mod be;
pub mod composite;
pub mod de;
pub mod es;
pub mod eu_common;
pub mod fr;
pub mod it;
pub mod nl;
pub mod pl;
pub mod pt;

pub use eu_common::all_eu_national_id_recognizers;
