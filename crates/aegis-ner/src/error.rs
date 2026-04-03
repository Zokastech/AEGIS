// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Erreurs du module NER ONNX ([`NerError`]).

use thiserror::Error;

/// Erreurs du module NER ONNX.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NerError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("ONNX Runtime: {0}")]
    Onnx(String),
    #[error("tokenizer: {0}")]
    Tokenizer(String),
    #[error("configuration NER: {0}")]
    Config(String),
    #[error("forme tenseur inattendue: {0}")]
    Shape(String),
    #[error("modèle inconnu dans le zoo: {0}")]
    UnknownModel(String),
    #[error("téléchargement désactivé: activez la feature `model-download`")]
    DownloadDisabled,
    #[error("téléchargement HTTP: {0}")]
    Download(String),
    #[error("mutex session ONNX empoisonné")]
    SessionLock,
}

impl From<ort::Error> for NerError {
    fn from(e: ort::Error) -> Self {
        NerError::Onnx(e.to_string())
    }
}

impl From<tokenizers::Error> for NerError {
    fn from(e: tokenizers::Error) -> Self {
        NerError::Tokenizer(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NerError>;
