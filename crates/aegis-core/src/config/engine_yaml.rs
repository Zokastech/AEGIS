// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! YAML schema for `aegis-config.yaml` (engine + pipeline + NER).

use crate::context::ContextScorerConfig;
use crate::entity::EntityType;
use crate::error::Result;
use crate::pipeline::{PipelineConfig, PipelineLevels};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Full engine configuration (`aegis-config.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AegisEngineConfig {
    #[serde(default)]
    pub recognizers: RecognizersYaml,
    /// Minimum score threshold per entity type (post-pipeline).
    #[serde(default)]
    pub entity_thresholds: HashMap<String, f64>,
    /// Pipeline level `1` | `2` | `3` (cumulative).
    #[serde(default)]
    pub pipeline_level: Option<u8>,
    /// Detailed block (merged after `pipeline_level`).
    #[serde(default)]
    pub pipeline: Option<PipelineConfig>,
    #[serde(default)]
    pub context_scorer: Option<ContextScorerConfig>,
    #[serde(default)]
    pub ner: Option<NerYaml>,
    #[serde(default)]
    pub analysis: Option<crate::config::AnalysisConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecognizersYaml {
    #[serde(default)]
    pub default_regex: DefaultRegexYaml,
    #[serde(default)]
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultRegexYaml {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_langs")]
    pub languages: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_langs() -> Vec<String> {
    vec!["en".into(), "fr".into()]
}

impl Default for DefaultRegexYaml {
    fn default() -> Self {
        Self {
            enabled: true,
            languages: default_langs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NerYaml {
    pub model_path: Option<String>,
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default = "default_batch")]
    pub batch_size: usize,
    #[serde(default)]
    pub thread_pool_size: Option<usize>,
}

fn default_device() -> String {
    "cpu".into()
}

fn default_batch() -> usize {
    8
}

impl Default for NerYaml {
    fn default() -> Self {
        Self {
            model_path: None,
            device: default_device(),
            batch_size: default_batch(),
            thread_pool_size: None,
        }
    }
}

impl AegisEngineConfig {
    pub fn from_yaml_str(s: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(s)?)
    }

    pub fn from_json_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn disabled_set(&self) -> HashSet<String> {
        self.recognizers
            .disabled
            .iter()
            .map(|x| x.to_lowercase())
            .collect()
    }

    pub fn entity_threshold_map(&self) -> Result<HashMap<EntityType, f64>> {
        let mut m = HashMap::new();
        for (k, v) in &self.entity_thresholds {
            let et = EntityType::from_config_key(k)?;
            m.insert(et, *v);
        }
        Ok(m)
    }

    /// Construit un [`PipelineConfig`] en appliquant `pipeline_level` puis le bloc `pipeline`.
    pub fn merged_pipeline_config(&self) -> PipelineConfig {
        let mut base = self.pipeline.clone().unwrap_or_default();
        if let Some(lvl) = self.pipeline_level {
            base.levels = match lvl {
                1 => PipelineLevels::L1Only,
                2 => PipelineLevels::L1L2,
                _ => PipelineLevels::L1L2L3,
            };
        }
        if let Some(ref a) = self.analysis {
            base.analysis = a.clone();
        }
        base
    }
}
