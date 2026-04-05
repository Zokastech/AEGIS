// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! NER through **ONNX Runtime** (`ort`) and Hugging Face **tokenizers** for the AEGIS pipeline.
//!
//! - [`NerEngine`]: loads a token-classification model (BERT, DistilBERT, XLM-RoBERTa exported ONNX).
//! - [`NerRecognizer`]: implements [`aegis_core::Recognizer`].
//! - [`postprocess`]: merges IOB / IOB2 / BILOU and maps to [`aegis_core::entity::EntityType`].
//!
//! ## Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use aegis_ner::{NerConfig, NerEngine, NerRecognizer};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cfg = NerConfig::default();
//! let engine = Arc::new(NerEngine::new("model.onnx", "tokenizer.json", cfg)?);
//! let _recognizer = NerRecognizer::new(engine, "onnx_ner");
//! # Ok(())
//! # }
//! ```
//!
//! ## Idiomes
//!
//! Erreurs via [`NerError`] ([`non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute))
//! et propagation avec `?`.

#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]

pub mod config;
pub mod engine;
pub mod error;
pub mod models;
pub mod postprocess;
pub mod prediction;
pub mod recognizer_adapter;

pub use aegis_core;
pub use config::{
    default_id2label_map, NerConfig, NerDevice, ScoreAggregation,
};
pub use engine::NerEngine;
pub use error::{NerError, Result};
pub use models::{download_model, list_models, ModelInfo, ModelZoo};
pub use postprocess::{merge_token_predictions, TokenPrediction};
pub use prediction::{NerPrediction, SubTokenScore};
pub use recognizer_adapter::NerRecognizer;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_non_empty() {
        assert!(!super::VERSION.is_empty());
    }
}
