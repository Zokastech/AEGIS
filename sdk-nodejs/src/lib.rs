// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Bindings NAPI-RS : entités, résultats, [`AegisEngine`] (aegis-core / aegis-anonymize).

use aegis_anonymize::{AnonymizationConfig, AnonymizedResult as CoreAnon, AnonymizerEngine};
use aegis_core::config::AnalysisConfig;
use aegis_core::engine::AnalyzerEngineBuilder;
use aegis_core::entity::{AnalysisResult as CoreAnalysis, Entity as CoreEntity, EntityType};
use aegis_core::error::AegisError;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use aegis_regex as _;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FfiAnonymizeConfig {
    #[serde(default)]
    analysis: AnalysisConfig,
    #[serde(default)]
    operators_by_entity: HashMap<String, aegis_core::anonymizer::OperatorConfig>,
    #[serde(default)]
    default_operator: Option<aegis_core::anonymizer::OperatorConfig>,
}

fn aegis_to_napi(e: AegisError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

#[napi(object)]
#[derive(Clone)]
pub struct Entity {
    pub entity_type: String,
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub score: f64,
    pub recognizer_name: String,
    pub metadata: HashMap<String, String>,
}

impl From<&CoreEntity> for Entity {
    fn from(e: &CoreEntity) -> Self {
        Self {
            entity_type: e.entity_type.config_key(),
            start: e.start.min(u32::MAX as usize) as u32,
            end: e.end.min(u32::MAX as usize) as u32,
            text: e.text.clone(),
            score: e.score,
            recognizer_name: e.recognizer_name.clone(),
            metadata: e.metadata.clone(),
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct TransformationRecord {
    pub entity_start: u32,
    pub entity_end: u32,
    pub original_text: String,
    pub replacement: String,
    pub operator: String,
    pub entity_type: String,
}

impl From<&aegis_anonymize::TransformationRecord> for TransformationRecord {
    fn from(t: &aegis_anonymize::TransformationRecord) -> Self {
        Self {
            entity_start: t.entity_start.min(u32::MAX as usize) as u32,
            entity_end: t.entity_end.min(u32::MAX as usize) as u32,
            original_text: t.original_text.clone(),
            replacement: t.replacement.clone(),
            operator: t.operator.clone(),
            entity_type: t.entity_type.clone(),
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct AnonymizedResult {
    pub text: String,
    pub transformations: Vec<TransformationRecord>,
    pub key_ids_used: Vec<String>,
    pub mapping_hints: HashMap<String, String>,
}

impl From<CoreAnon> for AnonymizedResult {
    fn from(a: CoreAnon) -> Self {
        let transformations = a.transformations.iter().map(TransformationRecord::from).collect();
        Self {
            text: a.text,
            transformations,
            key_ids_used: a.key_ids_used,
            mapping_hints: a.mapping_hints,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct AnalysisResult {
    pub entities: Vec<Entity>,
    pub processing_time_ms: u32,
    pub language_detected: Option<String>,
    pub text_length: u32,
}

impl From<CoreAnalysis> for AnalysisResult {
    fn from(r: CoreAnalysis) -> Self {
        let entities = r.entities.iter().map(Entity::from).collect();
        Self {
            entities,
            processing_time_ms: r.processing_time_ms.min(u32::MAX as u64) as u32,
            language_detected: r.language_detected,
            text_length: r.text_length.min(u32::MAX as usize) as u32,
        }
    }
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct AnalyzeOptions {
    pub language: Option<String>,
    pub entities: Option<Vec<String>>,
    pub score_threshold: Option<f64>,
}

struct EngineInner {
    analyzer: aegis_core::AnalyzerEngine,
    anonymizer: AnonymizerEngine,
}

/// Moteur AEGIS (analyse + anonymisation), exposé à Node.js.
#[napi(js_name = "NativeAegisEngine")]
pub struct AegisEngine {
    inner: Arc<Mutex<Option<EngineInner>>>,
}

#[napi]
impl AegisEngine {
    /// Crée un moteur. `configPath` : fichier YAML ou JSON moteur ; `languages` : ex. `["en","fr"]`.
    #[napi(constructor)]
    pub fn new(config_path: Option<String>, languages: Option<Vec<String>>) -> Result<Self> {
        let langs: Vec<String> = languages.unwrap_or_else(|| vec!["en".into(), "fr".into()]);
        let sl: Vec<&str> = langs.iter().map(|s| s.as_str()).collect();

        let built = if let Some(p) = config_path {
            let path = Path::new(&p);
            let content = fs::read_to_string(path).map_err(|e| {
                napi::Error::from_reason(format!("config_path {p}: {e}"))
            })?;
            let lower = p.to_string_lossy().to_lowercase();
            let mut b = AnalyzerEngineBuilder::new().with_default_recognizers(&sl);
            b = if lower.ends_with(".json") {
                b.with_engine_json_str(&content).map_err(aegis_to_napi)?
            } else {
                b.with_engine_yaml_str(&content).map_err(aegis_to_napi)?
            };
            b.build().map_err(aegis_to_napi)?
        } else {
            AnalyzerEngineBuilder::new()
                .with_default_recognizers(&sl)
                .build()
                .map_err(aegis_to_napi)?
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(EngineInner {
                analyzer: built,
                anonymizer: AnonymizerEngine::new(),
            }))),
        })
    }

    fn with_inner<R, F: FnOnce(&EngineInner) -> Result<R>>(inner: &Arc<Mutex<Option<EngineInner>>>, f: F) -> Result<R> {
        let g = inner
            .lock()
            .map_err(|_| napi::Error::from_reason("engine mutex poisoned"))?;
        let e = g
            .as_ref()
            .ok_or_else(|| napi::Error::from_reason("engine closed"))?;
        f(e)
    }

    #[napi]
    pub async fn analyze(
        &self,
        text: String,
        options: Option<AnalyzeOptions>,
    ) -> Result<Vec<Entity>> {
        let inner = Arc::clone(&self.inner);
        let options = options.unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            let opts = options;
            let mut cfg = AnalysisConfig::default();
            cfg.score_threshold = opts.score_threshold.unwrap_or(0.5);
            cfg.language = opts.language;
            if let Some(names) = opts.entities {
                let mut v = Vec::with_capacity(names.len());
                for n in names {
                    v.push(EntityType::from_config_key(&n).map_err(aegis_to_napi)?);
                }
                cfg.entities_to_analyze = Some(v);
            }
            Self::with_inner(&inner, |eng| {
                let res = eng
                    .analyzer
                    .analyze(&text, Some(cfg))
                    .map_err(aegis_to_napi)?;
                Ok(res.entities.iter().map(Entity::from).collect())
            })
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("join analyze: {e}")))?
    }

    #[napi]
    pub async fn analyze_full(
        &self,
        text: String,
        options: Option<AnalyzeOptions>,
    ) -> Result<AnalysisResult> {
        let inner = Arc::clone(&self.inner);
        let options = options.unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            let opts = options;
            let mut cfg = AnalysisConfig::default();
            cfg.score_threshold = opts.score_threshold.unwrap_or(0.5);
            cfg.language = opts.language;
            if let Some(names) = opts.entities {
                let mut v = Vec::with_capacity(names.len());
                for n in names {
                    v.push(EntityType::from_config_key(&n).map_err(aegis_to_napi)?);
                }
                cfg.entities_to_analyze = Some(v);
            }
            Self::with_inner(&inner, |eng| {
                let res = eng
                    .analyzer
                    .analyze(&text, Some(cfg))
                    .map_err(aegis_to_napi)?;
                Ok(AnalysisResult::from(res))
            })
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("join analyze_full: {e}")))?
    }

    #[napi]
    pub async fn anonymize(
        &self,
        text: String,
        operators_json: Option<String>,
    ) -> Result<AnonymizedResult> {
        let inner = Arc::clone(&self.inner);
        let json = operators_json.unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            let cfg: FfiAnonymizeConfig = if json.trim().is_empty() || json.trim() == "{}" {
                FfiAnonymizeConfig::default()
            } else {
                let mut raw: serde_json::Value =
                    serde_json::from_str(&json).map_err(|e| {
                        napi::Error::from_reason(format!("operators JSON invalide: {e}"))
                    })?;
                if raw.is_object()
                    && !raw
                        .as_object()
                        .unwrap()
                        .contains_key("operators_by_entity")
                    && !raw.as_object().unwrap().contains_key("analysis")
                    && !raw.as_object().unwrap().contains_key("default_operator")
                {
                    raw = serde_json::json!({ "operators_by_entity": raw });
                }
                serde_json::from_value(raw).map_err(|e| {
                    napi::Error::from_reason(format!("operators JSON invalide: {e}"))
                })?
            };

            Self::with_inner(&inner, |eng| {
                let analyzed = eng
                    .analyzer
                    .analyze(&text, Some(cfg.analysis))
                    .map_err(aegis_to_napi)?;
                let anon_cfg = AnonymizationConfig {
                    operators_by_entity: cfg.operators_by_entity,
                    default_operator: cfg.default_operator,
                };
                let out = eng
                    .anonymizer
                    .anonymize(&text, &analyzed.entities, &anon_cfg);
                Ok(AnonymizedResult::from(out))
            })
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("join anonymize: {e}")))?
    }

    #[napi]
    pub async fn analyze_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<Entity>>> {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || {
            Self::with_inner(&inner, |eng| {
                let mut out = Vec::with_capacity(texts.len());
                for t in &texts {
                    let res = eng.analyzer.analyze(t, None).map_err(aegis_to_napi)?;
                    out.push(res.entities.iter().map(Entity::from).collect());
                }
                Ok(out)
            })
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("join analyze_batch: {e}")))?
    }

    #[napi]
    pub fn close(&self) -> Result<()> {
        let mut g = self
            .inner
            .lock()
            .map_err(|_| napi::Error::from_reason("engine mutex poisoned"))?;
        *g = None;
        Ok(())
    }
}

/// Version du binaire natif (alignée sur `Cargo.toml`).
#[napi(js_name = "nativeAddonVersion")]
pub fn native_addon_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
