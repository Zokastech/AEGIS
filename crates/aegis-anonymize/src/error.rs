// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Erreurs d’anonymisation ([`AnonymizeError`]).

use thiserror::Error;

/// Errors returned by [`crate::AnonymizerEngine`] and related operators.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AnonymizeError {
    #[error("invalid key material: {0}")]
    InvalidKey(String),

    #[error("decryption failed: {0}")]
    DecryptFailed(String),

    #[error("missing key for id {0}")]
    MissingKeyId(String),

    #[error("final span missing on transformation record")]
    MissingFinalSpan,

    #[error("replacement mismatch at span {start}..{end}")]
    ReplacementMismatch { start: usize, end: usize },

    #[error("unsupported reverse for operator {0}")]
    UnsupportedReverse(String),
}

pub type Result<T> = std::result::Result<T, AnonymizeError>;
