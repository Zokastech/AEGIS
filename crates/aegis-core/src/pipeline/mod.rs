// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Multi-level detection pipeline (regex → context → NER).
//!
//! **Level-1** recognizers are injected (`Vec<Arc<dyn Recognizer>>`) — typically
//! from `aegis-regex` — to avoid a circular dependency with `aegis-core`.
//!
//! ## Batch SLM / ONNX
//!
//! To amortize model latency, use [`NerBackend::analyze_batch`] in your
//! `aegis-ner` implementation and call it on a batch of documents after running
//! L1+L2 (e.g. in parallel with Rayon) in the application.

mod config;
mod detection;
mod fusion;
mod ner;
mod trace;

pub use crate::context::{
    CombinationRule, ContextScorer, ContextScorerConfig, LanguageContextRules,
};
pub use config::{PipelineConfig, PipelineLevels};
pub use detection::{DetectionPipeline, L3TraceAttachment, PipelineOutput};
pub use fusion::{FusedCandidate, ScoreFusion};
pub use ner::{is_contextual_entity_type, MockNerBackend, NerBackend};
pub use trace::{DecisionTrace, TraceStep};
