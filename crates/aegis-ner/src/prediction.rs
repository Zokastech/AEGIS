// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Structured NER predictions (merged entity + sub-tokens).

use aegis_core::entity::EntityType;
use serde::{Deserialize, Serialize};

/// One sub-token with its score and character offsets in the source text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubTokenScore {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub score: f64,
    /// Raw model label for this sub-token (e.g. `I-PER`).
    pub label: String,
}

/// Merged entity after IOB/BILOU post-processing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NerPrediction {
    pub entity_type: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub score: f64,
    pub tokens: Vec<SubTokenScore>,
}
