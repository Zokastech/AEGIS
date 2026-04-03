// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! [`Recognizer`] trait for detection modules.

use crate::config::AnalysisConfig;
use crate::entity::{Entity, EntityType};
use std::sync::Arc;

/// Contract for any detection module (regex, NER, heuristics).
///
/// ```
/// # use aegis_core::{AnalysisConfig, EntityType, Recognizer, Entity};
/// # use std::sync::Arc;
/// struct Stub;
/// impl Recognizer for Stub {
///     fn name(&self) -> &str { "stub" }
///     fn supported_entities(&self) -> Vec<EntityType> { vec![EntityType::Person] }
///     fn supported_languages(&self) -> Vec<&str> { vec!["*"] }
///     fn analyze(&self, _text: &str, _cfg: &AnalysisConfig) -> Vec<Entity> { vec![] }
/// }
/// let r: Arc<dyn Recognizer> = Arc::new(Stub);
/// assert_eq!(r.min_score(), 0.5);
/// ```
pub trait Recognizer: Send + Sync {
    fn name(&self) -> &str;
    fn supported_entities(&self) -> Vec<EntityType>;
    fn supported_languages(&self) -> Vec<&str>;
    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity>;
    fn min_score(&self) -> f64 {
        0.5
    }
}

/// Delegation through [`Arc`] to share a recognizer across threads.
///
/// ```
/// # use aegis_core::{AnalysisConfig, EntityType, Recognizer, Entity};
/// # use std::sync::Arc;
/// struct Stub;
/// impl Recognizer for Stub {
///     fn name(&self) -> &str { "stub" }
///     fn supported_entities(&self) -> Vec<EntityType> { vec![] }
///     fn supported_languages(&self) -> Vec<&str> { vec![] }
///     fn analyze(&self, _t: &str, _c: &AnalysisConfig) -> Vec<Entity> { vec![] }
/// }
/// let a = Arc::new(Stub);
/// let r: Arc<dyn Recognizer> = a.clone();
/// assert_eq!(r.name(), "stub");
/// ```
impl<T: Recognizer + ?Sized> Recognizer for Arc<T> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn supported_entities(&self) -> Vec<EntityType> {
        (**self).supported_entities()
    }
    fn supported_languages(&self) -> Vec<&str> {
        (**self).supported_languages()
    }
    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        (**self).analyze(text, config)
    }
    fn min_score(&self) -> f64 {
        (**self).min_score()
    }
}
