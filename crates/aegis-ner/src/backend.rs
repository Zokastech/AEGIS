// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Backend pipeline L3 : [`aegis_core::pipeline::NerBackend`] autour de [`crate::engine::NerEngine`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use aegis_core::entity::Entity;
use aegis_core::error::{AegisError, Result};
use aegis_core::pipeline::NerBackend;

use crate::config::{NerConfig, NerDevice};
use crate::engine::NerEngine;

/// Résout `tokenizer.json` à côté du `.onnx` ou chemin explicite (`ner.tokenizer_path`).
pub fn resolve_tokenizer_path(
    model_path: &str,
    explicit: Option<&str>,
) -> std::result::Result<String, AegisError> {
    if let Some(p) = explicit {
        if Path::new(p).is_file() {
            return Ok(p.to_string());
        }
        return Err(AegisError::ModelError(format!(
            "fichier tokenizer introuvable: {p}"
        )));
    }
    let mp = Path::new(model_path);
    let parent = mp.parent().unwrap_or_else(|| Path::new("."));
    let direct = parent.join("tokenizer.json");
    if direct.is_file() {
        return Ok(direct.to_string_lossy().into());
    }
    let nested = parent.join("tokenizer_hf").join("tokenizer.json");
    if nested.is_file() {
        return Ok(nested.to_string_lossy().into());
    }
    Err(AegisError::ModelError(format!(
        "tokenizer.json introuvable (placer à côté de {model_path} ou définir ner.tokenizer_path dans engine_init_json)"
    )))
}

fn ner_device_from_str(s: &str) -> NerDevice {
    match s.trim().to_lowercase().as_str() {
        "cuda" | "gpu" => NerDevice::Cuda { device_id: 0 },
        "tensorrt" => NerDevice::TensorRt,
        "coreml" => NerDevice::CoreML,
        _ => NerDevice::Cpu,
    }
}

/// Enregistré par `aegis-ffi` via [`aegis_core::engine::try_register_onnx_ner_loader`].
pub fn onnx_ner_backend_loader(
    model_path: &str,
    tokenizer_path: Option<&str>,
    hf_config_json_path: Option<&str>,
    batch_size: usize,
    device_str: &str,
) -> Result<Arc<dyn NerBackend>> {
    let tok = resolve_tokenizer_path(model_path, tokenizer_path)?;
    let mut cfg = NerConfig {
        batch_size: batch_size.max(1),
        device: ner_device_from_str(device_str),
        max_sequence_length: 512,
        ..NerConfig::default()
    };
    if let Some(h) = hf_config_json_path {
        cfg.hf_config_json_path = Some(h.to_string());
    }
    let engine =
        NerEngine::new(model_path, &tok, cfg).map_err(|e| AegisError::ModelError(e.to_string()))?;
    Ok(Arc::new(OnnxNerBackend {
        engine: Arc::new(engine),
    }))
}

/// Inférence ONNX pour le niveau 3 du pipeline.
pub struct OnnxNerBackend {
    engine: Arc<NerEngine>,
}

impl NerBackend for OnnxNerBackend {
    fn is_onnx_backend(&self) -> bool {
        true
    }

    fn analyze_batch(&self, texts: &[&str], _language: Option<&str>) -> Result<Vec<Vec<Entity>>> {
        let preds = self
            .engine
            .predict_batch(texts)
            .map_err(|e| AegisError::ModelError(e.to_string()))?;
        let mut out = Vec::with_capacity(preds.len());
        for (doc, text) in preds.iter().zip(texts.iter()) {
            let row: Vec<Entity> = doc
                .iter()
                .map(|p| {
                    let end = p.end.min(text.len());
                    let start = p.start.min(end);
                    Entity {
                        entity_type: p.entity_type.clone(),
                        start,
                        end,
                        text: text.get(start..end).unwrap_or("").to_string(),
                        score: p.score,
                        recognizer_name: "onnx_ner".into(),
                        metadata: HashMap::new(),
                        decision_trace: None,
                    }
                })
                .collect();
            out.push(row);
        }
        Ok(out)
    }
}
