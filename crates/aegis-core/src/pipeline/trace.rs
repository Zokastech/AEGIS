// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Structured pipeline decision trace (audit / debug).

use crate::entity::EntityType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log of decision steps for one document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub steps: Vec<TraceStep>,
}

impl DecisionTrace {
    pub fn push(&mut self, step: TraceStep) {
        self.steps.push(step);
    }
}

/// One atomic decision (level, scores, context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub level: u8,
    pub action: String,
    pub span_start: usize,
    pub span_end: usize,
    pub entity_type: EntityType,
    pub scores_by_source: HashMap<String, f64>,
    #[serde(default)]
    pub context_word_hits: Vec<String>,
    #[serde(default)]
    pub short_circuit: bool,
    #[serde(default)]
    pub note: Option<String>,
}
