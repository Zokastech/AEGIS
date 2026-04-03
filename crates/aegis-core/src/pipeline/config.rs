// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Multi-level pipeline configuration.

use crate::config::AnalysisConfig;
use serde::{Deserialize, Serialize};

/// Enabled detection levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PipelineLevels {
    /// Regex / rigid recognizers only.
    #[default]
    L1Only,
    /// L1 + contextual heuristics.
    L1L2,
    /// L1 + L2 + NER (SLM / ONNX).
    L1L2L3,
}

impl PipelineLevels {
    pub fn l1(&self) -> bool {
        true
    }
    pub fn l2(&self) -> bool {
        matches!(self, Self::L1L2 | Self::L1L2L3)
    }
    pub fn l3(&self) -> bool {
        matches!(self, Self::L1L2L3)
    }
}

/// Full configuration (YAML / serialization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub levels: PipelineLevels,
    /// Global output threshold (after merge).
    pub output_score_threshold: f64,
    /// Below this score, NER may run (alongside contextual types).
    pub ner_invocation_score_threshold: f64,
    /// L1 score above which L2/L3 are skipped for **this** entity (short-circuit).
    pub short_circuit_l1_score: f64,
    pub weight_level1: f64,
    pub weight_level2: f64,
    pub weight_level3: f64,
    /// Soft time budget per level (ms); 0 = unlimited.
    pub timeout_level1_ms: u64,
    pub timeout_level2_ms: u64,
    pub timeout_level3_ms: u64,
    /// Max gap (chars) to merge adjacent same-type entities.
    pub adjacent_merge_gap_chars: usize,
    /// Minimum IoU to treat two spans as the same multi-level entity.
    pub overlap_iou_min: f64,
    #[serde(default)]
    pub record_decision_trace: bool,
    #[serde(default)]
    pub analysis: AnalysisConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            levels: PipelineLevels::L1L2L3,
            output_score_threshold: 0.5,
            ner_invocation_score_threshold: 0.75,
            short_circuit_l1_score: 0.95,
            weight_level1: 0.45,
            weight_level2: 0.30,
            weight_level3: 0.25,
            timeout_level1_ms: 2,
            timeout_level2_ms: 8,
            timeout_level3_ms: 60,
            adjacent_merge_gap_chars: 1,
            overlap_iou_min: 0.35,
            record_decision_trace: false,
            analysis: AnalysisConfig::default(),
        }
    }
}

impl PipelineConfig {
    pub fn from_yaml_str(s: &str) -> crate::error::Result<Self> {
        Ok(serde_yaml::from_str(s)?)
    }
}
