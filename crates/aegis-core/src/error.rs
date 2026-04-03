// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! AEGIS core errors ([`AegisError`]).

use thiserror::Error;

/// Errors surfaced by **AEGIS** engine crates (zokastech.fr).
///
/// Marked [`non_exhaustive`]: in `match` outside this crate, add a `_` arm to stay
/// compatible with future variants.
///
/// ```
/// use aegis_core::AegisError;
/// let e = AegisError::ConfigError(String::from("test"));
/// assert!(e.to_string().contains("test"));
/// ```
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AegisError {
    #[error("recognizer error: {0}")]
    RecognizerError(String),

    #[error("anonymization error: {0}")]
    AnonymizationError(String),

    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("model error: {0}")]
    ModelError(String),

    #[error("registry error: {0}")]
    RegistryError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("yaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("toml decode error: {0}")]
    TomlDecode(#[from] toml::de::Error),

    #[error("toml encode error: {0}")]
    TomlEncode(#[from] toml::ser::Error),

    #[error("pipeline timeout at level {0}")]
    PipelineTimeout(u8),

    #[error("pipeline NER: {0}")]
    PipelineNer(String),
}

pub type Result<T> = std::result::Result<T, AegisError>;
