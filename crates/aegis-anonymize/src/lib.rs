// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Anonymization operators: mask, replace, hash, encrypt, FPE, pseudonym, engine.
//!
//! Public errors [`AnonymizeError`] / [`SyntheticError`] are [`non_exhaustive`] for semver-friendly extension.

#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]

mod encrypt;
mod engine;
mod error;
pub mod ff3_1;
mod fpe_op;
mod hash_op;
mod mask;
mod pseudonymize;
mod redact;
mod replace;
pub mod synthetic;
mod types;

pub use engine::AnonymizerEngine;
pub use error::{AnonymizeError, Result};
pub use types::{
    AnonymizationConfig, AnonymizedResult, ReverseMetadata, TransformationRecord,
};

pub use encrypt::EncryptOperator;
pub use fpe_op::FpeOperator;
pub use hash_op::HashOperator;
pub use mask::MaskOperator;
pub use pseudonymize::PseudonymizeOperator;
pub use redact::RedactOperator;
pub use replace::ReplaceOperator;
pub use synthetic::{
    generate_synthetic, seed_for_entity, subject_seed, synthetic_iban, synthetic_person_parts,
    CountryProfile, NationalIdKind, SyntheticDataGenerator, SyntheticError,
};
