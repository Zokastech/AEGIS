// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Core AEGIS engine concepts: entities, recognizers, configuration.
//!
//! ## Conventions (Rust idioms)
//!
//! - Errors use [`Result`] and `?` on [`AegisError`] ([try
//!   operator](https://doc.rust-lang.org/reference/expressions/operator-expr.html)).
//! - [`AegisError`] is [`non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
//!   so new variants can be added without breaking minor compatibility for callers.

#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]

pub mod anonymizer;
pub mod config;
pub mod context;
pub mod engine;
pub mod entity;
pub mod error;
pub mod ffi;
pub mod pipeline;
pub mod recognizer;
pub mod registry;

pub use anonymizer::{Operator, OperatorConfig, OperatorType};
pub use config::{AegisConfig, AnalysisConfig};
pub use context::{
    CombinationRule, ContextRule, ContextScorer, ContextScorerConfig, LanguageContextRules,
    LemmaAnalyzer, QuasiComboRuleYaml, QuasiIdentifierDetector, QuasiIdentifierReport,
    QuasiIdentifierYamlSection, ScorerWindowSettings, ScorerYamlBlock,
};
pub use engine::{
    register_default_regex_loader, AnalyzerEngine, AnalyzerEngineBuilder, DefaultRegexLoaderFn,
    PipelineLevel,
};
pub use entity::{AnalysisResult, Entity, EntityType};
pub use error::{AegisError, Result};
pub use ffi::{
    engine_analyze_json_c, engine_create_boxed, ffi_last_error_ptr, ffi_set_last_error,
    ffi_string_free,
};
pub use pipeline::{
    is_contextual_entity_type, DecisionTrace, DetectionPipeline, FusedCandidate, L3TraceAttachment,
    MockNerBackend, NerBackend, PipelineConfig, PipelineLevels, PipelineOutput, ScoreFusion,
    TraceStep,
};
pub use recognizer::Recognizer;
pub use registry::{RecognizerRegistry, RecognizerRegistryBuilder};
