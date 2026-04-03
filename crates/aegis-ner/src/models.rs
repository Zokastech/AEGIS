// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Catalog of recommended NER models (Hugging Face / ONNX Zoo references).

use std::path::{Path, PathBuf};

use crate::error::{NerError, Result};

/// Metadata for one model in the AEGIS zoo.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    /// ONNX URL (or HF page) — for `download_model` when enabled.
    pub url: &'static str,
    /// Approximate bundle size (bytes), 0 if unknown.
    pub size_bytes: u64,
    pub languages: &'static [&'static str],
    pub entities_supported: &'static [&'static str],
    pub accuracy_metrics: &'static str,
    /// Rough parameter count (millions), if known.
    pub params_millions: Option<u32>,
}

/// Zoo entry point (stable ids).
pub struct ModelZoo;

impl ModelZoo {
    pub const XLM_ROBERTA_NER: ModelInfo = ModelInfo {
        id: "xlm-roberta-ner",
        name: "XLM-RoBERTa NER (multilingue)",
        url: "https://huggingface.co/xlm-roberta-large-finetuned-conll03-english",
        size_bytes: 1_100_000_000,
        languages: &["100+ (dont FR, DE, IT, ES, NL, …)"],
        entities_supported: &["PER", "ORG", "LOC", "MISC (selon fine-tuning)"],
        accuracy_metrics: "F1 typique CoNLL-2003 ~0.90+ selon tête et corpus",
        params_millions: Some(270),
    };

    pub const DISTILBERT_NER_EU: ModelInfo = ModelInfo {
        id: "distilbert-ner-eu",
        name: "DistilBERT NER (léger, usage EU)",
        url: "https://huggingface.co/elastic/distilbert-base-uncased-finetuned-conll03-english",
        size_bytes: 260_000_000,
        languages: &["en", "fr", "de", "es", "it (transfert / multilingue possible)"],
        entities_supported: &["PER", "ORG", "LOC", "MISC"],
        accuracy_metrics: "F1 CoNLL ~0.92 (anglais) ; autres langues selon fine-tuning",
        params_millions: Some(66),
    };

    pub const PHI3_MINI_NER: ModelInfo = ModelInfo {
        id: "phi-3-mini-ner",
        name: "Phi-3-mini (SLM — à fine-tuner pour NER token-classification)",
        url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct",
        size_bytes: 7_500_000_000,
        languages: &["multilingue partiel ; dominante EN"],
        entities_supported: &["(dépend du fine-tuning IOB)"],
        accuracy_metrics: "N/A sans fine-tuning NER dédié",
        params_millions: Some(3800),
    };

    /// Lists all catalog models.
    pub fn all() -> &'static [ModelInfo] {
        &[
            Self::XLM_ROBERTA_NER,
            Self::DISTILBERT_NER_EU,
            Self::PHI3_MINI_NER,
        ]
    }

    pub fn by_id(id: &str) -> Result<&'static ModelInfo> {
        Self::all()
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| NerError::UnknownModel(id.into()))
    }
}

/// Convenience alias for `ModelZoo::all()`.
pub fn list_models() -> &'static [ModelInfo] {
    ModelZoo::all()
}

/// Downloads an HTTP resource to `cache_dir / filename`.
///
/// Requires the **`model-download`** feature (`ureq`).
#[cfg(feature = "model-download")]
pub fn download_model(model_id: &str, cache_dir: &Path) -> Result<PathBuf> {
    use std::fs::File;

    let info = ModelZoo::by_id(model_id)?;
    std::fs::create_dir_all(cache_dir).map_err(NerError::from)?;
    let file_name = format!("{}.bin", model_id.replace('/', "_"));
    let dest = cache_dir.join(&file_name);
    tracing::info!(url = info.url, dest = %dest.display(), "downloading NER model");
    let resp = ureq::get(info.url)
        .call()
        .map_err(|e| NerError::Download(e.to_string()))?;
    let mut f = File::create(&dest).map_err(NerError::from)?;
    std::io::copy(&mut resp.into_reader(), &mut f).map_err(|e| NerError::Download(e.to_string()))?;
    Ok(dest)
}

#[cfg(not(feature = "model-download"))]
pub fn download_model(_model_id: &str, _cache_dir: &Path) -> Result<PathBuf> {
    Err(NerError::DownloadDisabled)
}
