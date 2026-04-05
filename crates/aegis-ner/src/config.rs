// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! NER inference configuration (device, threads, batch, labels).

use aegis_core::entity::EntityType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::NerError;

/// How to aggregate confidence scores across merged sub-tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScoreAggregation {
    #[default]
    Mean,
    Max,
    Min,
}

/// Device / ONNX Runtime execution providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NerDevice {
    #[default]
    Cpu,
    /// NVIDIA GPU (CUDA EP, falls back to CPU if unavailable).
    Cuda {
        #[serde(default)]
        device_id: i32,
    },
    TensorRt,
    /// Apple Silicon / macOS / iOS (CoreML EP).
    CoreML,
}

/// Runtime and post-processing settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NerConfig {
    #[serde(default)]
    pub device: NerDevice,
    /// ONNX Runtime intra-op threads.
    #[serde(default = "default_intra_threads")]
    pub intra_threads: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_max_sequence_length")]
    pub max_sequence_length: usize,
    /// Score aggregation across merged sub-tokens.
    #[serde(default)]
    pub score_aggregation: ScoreAggregation,
    /// Map raw model labels → [`EntityType`] (e.g. `"PER"` → `Person` after stripping IOB prefix).
    #[serde(default)]
    pub label_to_entity: HashMap<String, EntityType>,
    /// `id → label` when not loaded from `hf_config_json_path`.
    #[serde(default)]
    pub id2label: Option<HashMap<u32, String>>,
    /// Optional path to Hugging Face `config.json` (`id2label`).
    #[serde(default)]
    pub hf_config_json_path: Option<String>,
    /// Input tensor name for `input_ids`.
    #[serde(default = "default_input_ids")]
    pub input_ids_input_name: String,
    /// Tensor name for `attention_mask`.
    #[serde(default = "default_attention_mask")]
    pub attention_mask_input_name: String,
    /// `token_type_ids` when required by the model (BERT), else `None`.
    #[serde(default)]
    pub token_type_ids_input_name: Option<String>,
    /// Output tensor name (often `logits`).
    #[serde(default = "default_logits")]
    pub logits_output_name: String,
    /// Skip first and last tokens (often [CLS] / [SEP] or `<s>` / `</s>`).
    #[serde(default = "default_true")]
    pub skip_edge_special_tokens: bool,
}

fn default_intra_threads() -> usize {
    1
}

fn default_batch_size() -> usize {
    8
}

fn default_max_sequence_length() -> usize {
    128
}

fn default_input_ids() -> String {
    "input_ids".into()
}

fn default_attention_mask() -> String {
    "attention_mask".into()
}

fn default_logits() -> String {
    "logits".into()
}

fn default_true() -> bool {
    true
}

impl Default for NerConfig {
    fn default() -> Self {
        Self {
            device: NerDevice::default(),
            intra_threads: default_intra_threads(),
            batch_size: default_batch_size(),
            max_sequence_length: default_max_sequence_length(),
            score_aggregation: ScoreAggregation::default(),
            label_to_entity: default_label_map(),
            id2label: None,
            hf_config_json_path: None,
            input_ids_input_name: default_input_ids(),
            attention_mask_input_name: default_attention_mask(),
            token_type_ids_input_name: None,
            logits_output_name: default_logits(),
            skip_edge_special_tokens: default_true(),
        }
    }
}

/// Default CoNLL-style labels (9 classes + O).
pub fn default_id2label_map() -> HashMap<u32, String> {
    [
        (0u32, "O".into()),
        (1, "B-PER".into()),
        (2, "I-PER".into()),
        (3, "B-ORG".into()),
        (4, "I-ORG".into()),
        (5, "B-LOC".into()),
        (6, "I-LOC".into()),
        (7, "B-MISC".into()),
        (8, "I-MISC".into()),
    ]
    .into_iter()
    .collect()
}

fn default_label_map() -> HashMap<String, EntityType> {
    [
        ("PER".into(), EntityType::Person),
        ("PERSON".into(), EntityType::Person),
        ("ORG".into(), EntityType::Organization),
        ("ORGANIZATION".into(), EntityType::Organization),
        ("LOC".into(), EntityType::Location),
        ("LOCATION".into(), EntityType::Location),
        ("GPE".into(), EntityType::Location),
        ("MISC".into(), EntityType::Custom("NER_MISC".into())),
    ]
    .into_iter()
    .collect()
}

pub(crate) fn parse_hf_id2label_json(raw: &str) -> crate::error::Result<HashMap<u32, String>> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| NerError::Config(e.to_string()))?;
    let obj = v
        .get("id2label")
        .and_then(|x| x.as_object())
        .ok_or_else(|| NerError::Config("missing id2label field".into()))?;
    let mut m = HashMap::new();
    for (k, val) in obj {
        let id: u32 = k
            .parse()
            .map_err(|_| NerError::Config(format!("id2label key: {k}")))?;
        let label = val
            .as_str()
            .ok_or_else(|| NerError::Config("id2label value is not a string".into()))?
            .to_string();
        m.insert(id, label);
    }
    Ok(m)
}

impl NerConfig {
    /// Charge `id2label` depuis un `config.json` Hugging Face.
    pub fn load_hf_id2label(path: impl AsRef<Path>) -> crate::error::Result<HashMap<u32, String>> {
        let p = path.as_ref();
        let raw = std::fs::read_to_string(p)?;
        parse_hf_id2label_json(&raw)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_hf_id2label() {
        let j = r#"{"id2label": {"0": "O", "1": "B-PER"}}"#;
        let m = super::parse_hf_id2label_json(j).unwrap();
        assert_eq!(m.get(&1).map(String::as_str), Some("B-PER"));
    }
}
