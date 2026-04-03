// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Anonymization operators: types and [`Operator`] trait.

use crate::entity::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anonymization operator kind.
///
/// ```
/// use aegis_core::OperatorType;
/// let t = OperatorType::Mask;
/// assert!(matches!(t, OperatorType::Mask));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorType {
    Redact,
    Replace,
    Mask,
    Hash,
    Encrypt,
    #[serde(rename = "fpe", alias = "FPE")]
    Fpe,
    Pseudonymize,
    Custom(String),
}

/// Operator runtime parameters (loadable from config).
///
/// ```
/// use aegis_core::{OperatorConfig, OperatorType};
/// use std::collections::HashMap;
///
/// let c = OperatorConfig {
///     operator_type: OperatorType::Redact,
///     params: HashMap::new(),
/// };
/// assert!(matches!(c.operator_type, OperatorType::Redact));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorConfig {
    pub operator_type: OperatorType,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

/// Operator applied to an entity in the text.
///
/// ```
/// # use aegis_core::entity::{Entity, EntityType};
/// # use aegis_core::{Operator, OperatorConfig};
/// # use std::collections::HashMap;
/// struct Echo;
/// impl Operator for Echo {
///     fn operate(&self, _e: &Entity, text: &str, _c: &OperatorConfig) -> String {
///         text.to_string()
///     }
/// }
/// let op = Echo;
/// let entity = Entity {
///     entity_type: EntityType::Email,
///     start: 0,
///     end: 3,
///     text: "ab".into(),
///     score: 1.0,
///     recognizer_name: "t".into(),
///     metadata: HashMap::new(),
///     decision_trace: None,
/// };
/// assert_eq!(op.operate(&entity, "hello", &OperatorConfig {
///     operator_type: aegis_core::OperatorType::Redact,
///     params: HashMap::new(),
/// }), "hello");
/// ```
pub trait Operator: Send + Sync {
    fn operate(&self, entity: &Entity, text: &str, config: &OperatorConfig) -> String;
}
