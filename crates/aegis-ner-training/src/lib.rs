// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! # Pipeline NER niveau 3 (AEGIS)
//!
//! | Niveau | Composant |
//! |--------|-----------|
//! | L1 / L2 | Reconnaisseurs regex / règles ([`aegis_regex`], [`aegis_core`]) |
//! | **L3** | Modèle token-classification via ONNX ([`aegis_ner::NerEngine`]) |
//!
//! ## Chaîne amont (Python)
//!
//! Le dépôt contient le dossier **`training/`** : jeu synthétique UE, fine-tuning XLM-RoBERTa,
//! export ONNX + `tokenizer.json` pour la crate Rust [`aegis_ner`].
//!
//! ```text
//! dataset_builder / train_ner → export_onnx → model.onnx + tokenizer.json → NerEngine::new
//! ```
//!
//! Voir `training/README.md` à la racine du monorepo.
//!
//! [`aegis_regex`]: https://docs.rs/aegis-regex
//! [`aegis_core`]: https://docs.rs/aegis-core

#![warn(rust_2018_idioms)]

pub use aegis_core;
pub use aegis_ner;

pub use aegis_ner::{NerConfig, NerEngine, NerRecognizer};

/// Étapes documentées du pipeline détection PII (pas d’exécution Python ici).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NerPipelineLevel {
    /// Regex + heuristiques.
    L1L2,
    /// Inférence ONNX (`aegis_ner`).
    L3Onnx,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level3_links_to_ner_engine_type() {
        let _ = NerPipelineLevel::L3Onnx;
        assert!(NerConfig::default().intra_threads >= 1);
    }
}
