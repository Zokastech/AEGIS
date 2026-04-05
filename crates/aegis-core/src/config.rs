// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Analysis configuration and per-entity operator policy.
//!
//! ```
//! use aegis_core::config::AnalysisConfig;
//!
//! let c = AnalysisConfig::default();
//! assert_eq!(c.score_threshold, 0.5);
//! ```

pub mod engine_yaml;

pub use engine_yaml::{AegisEngineConfig, DefaultRegexYaml, NerYaml, RecognizersYaml};

use crate::anonymizer::OperatorConfig;
use crate::entity::EntityType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_analysis_score_threshold() -> f64 {
    0.5
}

fn default_context_window_size() -> usize {
    5
}

/// Parameters for one analysis pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default = "default_analysis_score_threshold")]
    pub score_threshold: f64,
    /// Per-request pipeline override: `1` = L1 only, `2` = L1+L2, `3` = L1+L2+L3 (FFI / API).
    #[serde(default)]
    pub pipeline_level: Option<u8>,
    #[serde(default)]
    pub entities_to_analyze: Option<Vec<EntityType>>,
    #[serde(default)]
    pub return_decision_process: bool,
    #[serde(default = "default_context_window_size")]
    pub context_window_size: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            language: None,
            score_threshold: 0.5,
            pipeline_level: None,
            entities_to_analyze: None,
            return_decision_process: false,
            context_window_size: 5,
        }
    }
}

mod operators_by_entity_serde {
    use super::OperatorConfig;
    use crate::entity::EntityType;
    use serde::de::Error as DeError;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::HashMap;
    use std::result::Result;

    pub fn serialize<S>(
        map: &HashMap<EntityType, OperatorConfig>,
        ser: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut m = ser.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            m.serialize_entry(&k.config_key(), v)?;
        }
        m.end()
    }

    pub fn deserialize<'de, D>(de: D) -> Result<HashMap<EntityType, OperatorConfig>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: HashMap<String, OperatorConfig> = HashMap::deserialize(de)?;
        let mut out = HashMap::new();
        for (key, v) in raw {
            let et = EntityType::from_config_key(&key).map_err(DeError::custom)?;
            out.insert(et, v);
        }
        Ok(out)
    }
}

/// Global configuration loadable from YAML or TOML.
///
/// `operators_by_entity` keys follow [`EntityType::config_key`] (e.g. `EMAIL`, `NATIONAL_ID`).
///
/// ```
/// use aegis_core::AegisConfig;
///
/// let yaml = r#"
/// analysis:
///   score_threshold: 0.7
/// operators_by_entity:
///   EMAIL:
///     operator_type: mask
///     params: {}
/// "#;
/// let cfg = AegisConfig::from_yaml_str(yaml).unwrap();
/// assert_eq!(cfg.analysis.score_threshold, 0.7);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AegisConfig {
    pub analysis: AnalysisConfig,
    #[serde(default)]
    #[serde(with = "operators_by_entity_serde")]
    pub operators_by_entity: HashMap<EntityType, OperatorConfig>,
}

impl AegisConfig {
    /// Deserializes from a JSON string (FFI / SDK interop).
    pub fn from_json_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Deserializes from a YAML string.
    pub fn from_yaml_str(s: &str) -> Result<Self> {
        let v = serde_yaml::from_str(s)?;
        Ok(v)
    }

    /// Deserializes from a TOML string.
    ///
    /// ```
    /// use aegis_core::AegisConfig;
    ///
    /// let toml = r#"
    /// [analysis]
    /// score_threshold = 0.6
    ///
    /// [operators_by_entity.EMAIL]
    /// operator_type = "redact"
    /// "#;
    /// let cfg = AegisConfig::from_toml_str(toml).unwrap();
    /// assert_eq!(cfg.analysis.score_threshold, 0.6);
    /// ```
    pub fn from_toml_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }

    /// Serialize to YAML.
    pub fn to_yaml_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Serialize to TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        Ok(toml::to_string(self)?)
    }
}
