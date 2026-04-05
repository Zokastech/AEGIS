// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Extension PyO3 : `Entity`, `AnonymizedResult`, `AegisEngine` (bindings vers aegis-core / aegis-anonymize).

use aegis_anonymize::{AnonymizationConfig, AnonymizedResult as CoreAnon, AnonymizerEngine};
use aegis_core::config::AnalysisConfig;
use aegis_core::engine::AnalyzerEngineBuilder;
use aegis_core::entity::{AnalysisResult as CoreAnalysis, Entity as CoreEntity, EntityType};
use aegis_core::error::AegisError;
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

// Enregistre le chargeur regex par défaut (ctor dans aegis-regex).
use aegis_regex as _;

fn aegis_to_py(e: AegisError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

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

/// Occurrence PII détectée (exposée à Python).
#[pyclass(name = "Entity", module = "aegis._native")]
#[derive(Clone)]
pub struct Entity {
    #[pyo3(get)]
    pub entity_type: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub score: f64,
    #[pyo3(get)]
    pub recognizer_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

impl From<&CoreEntity> for Entity {
    fn from(e: &CoreEntity) -> Self {
        Self {
            entity_type: e.entity_type.config_key(),
            start: e.start,
            end: e.end,
            text: e.text.clone(),
            score: e.score,
            recognizer_name: e.recognizer_name.clone(),
            metadata: e.metadata.clone(),
        }
    }
}

#[pymethods]
impl Entity {
    #[new]
    #[pyo3(signature = (entity_type, start, end, text, score=1.0, recognizer_name="custom", metadata=None))]
    fn py_new(
        entity_type: String,
        start: usize,
        end: usize,
        text: String,
        score: f64,
        recognizer_name: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            entity_type,
            start,
            end,
            text,
            score,
            recognizer_name: recognizer_name.to_string(),
            metadata: metadata.unwrap_or_default(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Entity(type={:?}, start={}, end={}, score={:.3})",
            self.entity_type, self.start, self.end, self.score
        )
    }
}

/// Enregistrement de transformation (anonymisation).
#[pyclass(name = "TransformationRecord", module = "aegis._native")]
#[derive(Clone)]
pub struct TransformationRecord {
    #[pyo3(get)]
    pub entity_start: usize,
    #[pyo3(get)]
    pub entity_end: usize,
    #[pyo3(get)]
    pub original_text: String,
    #[pyo3(get)]
    pub replacement: String,
    #[pyo3(get)]
    pub operator: String,
    #[pyo3(get)]
    pub entity_type: String,
}

impl From<&aegis_anonymize::TransformationRecord> for TransformationRecord {
    fn from(t: &aegis_anonymize::TransformationRecord) -> Self {
        Self {
            entity_start: t.entity_start,
            entity_end: t.entity_end,
            original_text: t.original_text.clone(),
            replacement: t.replacement.clone(),
            operator: t.operator.clone(),
            entity_type: t.entity_type.clone(),
        }
    }
}

/// Résultat d’anonymisation.
#[pyclass(name = "AnonymizedResult", module = "aegis._native")]
#[derive(Clone)]
pub struct AnonymizedResult {
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub transformations: Vec<TransformationRecord>,
    #[pyo3(get)]
    pub key_ids_used: Vec<String>,
    #[pyo3(get)]
    pub mapping_hints: HashMap<String, String>,
}

#[pymethods]
impl AnonymizedResult {
    fn __repr__(&self) -> String {
        format!(
            "AnonymizedResult(text_len={}, n_transforms={})",
            self.text.len(),
            self.transformations.len()
        )
    }
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

/// Résultat d’analyse complet (métadonnées moteur).
#[pyclass(name = "AnalysisResult", module = "aegis._native")]
pub struct AnalysisResult {
    #[pyo3(get)]
    pub entities: Vec<Entity>,
    #[pyo3(get)]
    pub processing_time_ms: u64,
    #[pyo3(get)]
    pub language_detected: Option<String>,
    #[pyo3(get)]
    pub text_length: usize,
}

impl From<CoreAnalysis> for AnalysisResult {
    fn from(r: CoreAnalysis) -> Self {
        let entities = r.entities.iter().map(Entity::from).collect();
        Self {
            entities,
            processing_time_ms: r.processing_time_ms,
            language_detected: r.language_detected,
            text_length: r.text_length,
        }
    }
}

struct EngineInner {
    analyzer: aegis_core::AnalyzerEngine,
    anonymizer: AnonymizerEngine,
}

/// Moteur AEGIS (analyse + anonymisation).
#[pyclass(name = "AegisEngine", module = "aegis._native")]
pub struct AegisEngine {
    inner: Mutex<Option<EngineInner>>,
}

#[pymethods]
impl AegisEngine {
    #[new]
    #[pyo3(signature = (config_path=None, languages=None))]
    fn new(config_path: Option<String>, languages: Option<Vec<String>>) -> PyResult<Self> {
        let langs: Vec<String> = languages.unwrap_or_else(|| vec!["en".into(), "fr".into()]);
        let sl: Vec<&str> = langs.iter().map(|s| s.as_str()).collect();

        let built = if let Some(p) = config_path {
            let path = Path::new(&p);
            let content = fs::read_to_string(path).map_err(|e| {
                PyIOError::new_err(format!("config_path {p}: {e}"))
            })?;
            let lower = p.to_lowercase();
            let mut b = AnalyzerEngineBuilder::new().with_default_recognizers(&sl);
            b = if lower.ends_with(".json") {
                b.with_engine_json_str(&content).map_err(aegis_to_py)?
            } else {
                b.with_engine_yaml_str(&content).map_err(aegis_to_py)?
            };
            b.build().map_err(aegis_to_py)?
        } else {
            AnalyzerEngineBuilder::new()
                .with_default_recognizers(&sl)
                .build()
                .map_err(aegis_to_py)?
        };

        Ok(Self {
            inner: Mutex::new(Some(EngineInner {
                analyzer: built,
                anonymizer: AnonymizerEngine::new(),
            })),
        })
    }

    /// Analyse un texte ; retourne la liste des entités détectées.
    #[pyo3(signature = (text, language=None, entities=None, score_threshold=0.5))]
    fn analyze(
        &self,
        text: &str,
        language: Option<String>,
        entities: Option<Vec<String>>,
        score_threshold: f64,
    ) -> PyResult<Vec<Entity>> {
        let mut cfg = AnalysisConfig::default();
        cfg.score_threshold = score_threshold;
        cfg.language = language;
        if let Some(names) = entities {
            let mut v = Vec::with_capacity(names.len());
            for n in names {
                v.push(EntityType::from_config_key(&n).map_err(aegis_to_py)?);
            }
            cfg.entities_to_analyze = Some(v);
        }
        let g = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("moteur verrouillé (poison)"))?;
        let inner = g
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("moteur fermé (close)"))?;
        let res = inner
            .analyzer
            .analyze(text, Some(cfg))
            .map_err(aegis_to_py)?;
        Ok(res.entities.iter().map(Entity::from).collect())
    }

    /// Analyse avec résultat complet (temps, langue, etc.).
    #[pyo3(signature = (text, language=None, entities=None, score_threshold=0.5))]
    fn analyze_full(
        &self,
        text: &str,
        language: Option<String>,
        entities: Option<Vec<String>>,
        score_threshold: f64,
    ) -> PyResult<AnalysisResult> {
        let mut cfg = AnalysisConfig::default();
        cfg.score_threshold = score_threshold;
        cfg.language = language;
        if let Some(names) = entities {
            let mut v = Vec::with_capacity(names.len());
            for n in names {
                v.push(EntityType::from_config_key(&n).map_err(aegis_to_py)?);
            }
            cfg.entities_to_analyze = Some(v);
        }
        let g = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("moteur verrouillé (poison)"))?;
        let inner = g
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("moteur fermé (close)"))?;
        let res = inner
            .analyzer
            .analyze(text, Some(cfg))
            .map_err(aegis_to_py)?;
        Ok(AnalysisResult::from(res))
    }

    /// Anonymise le texte. `operators` est un dict `ENTITY_TYPE -> {operator_type, params}` ou JSON complet FFI.
    #[pyo3(signature = (text, operators=None))]
    fn anonymize(&self, py: Python<'_>, text: &str, operators: Option<Py<PyAny>>) -> PyResult<AnonymizedResult> {
        let cfg_json = operators_to_config_json(py, operators)?;
        let cfg: FfiAnonymizeConfig = if cfg_json.trim().is_empty() || cfg_json.trim() == "{}" {
            FfiAnonymizeConfig::default()
        } else {
            serde_json::from_str(&cfg_json)
                .map_err(|e| PyValueError::new_err(format!("operators JSON invalide: {e}")))?
        };

        let g = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("moteur verrouillé (poison)"))?;
        let inner = g
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("moteur fermé (close)"))?;

        let analyzed = inner
            .analyzer
            .analyze(text, Some(cfg.analysis))
            .map_err(aegis_to_py)?;
        let anon_cfg = AnonymizationConfig {
            operators_by_entity: cfg.operators_by_entity,
            default_operator: cfg.default_operator,
        };
        let out = inner
            .anonymizer
            .anonymize(text, &analyzed.entities, &anon_cfg);
        Ok(AnonymizedResult::from(out))
    }

    /// Analyse un lot de textes (une liste d’entités par texte).
    fn analyze_batch(&self, texts: Vec<String>) -> PyResult<Vec<Vec<Entity>>> {
        let g = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("moteur verrouillé (poison)"))?;
        let inner = g
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("moteur fermé (close)"))?;
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            let res = inner.analyzer.analyze(&t, None).map_err(aegis_to_py)?;
            out.push(res.entities.iter().map(Entity::from).collect());
        }
        Ok(out)
    }

    /// Libère le moteur nativement (appelé par le context manager Python).
    fn close(&mut self) -> PyResult<()> {
        let mut g = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("moteur verrouillé (poison)"))?;
        *g = None;
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        mut slf: PyRefMut<'_, Self>,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<()> {
        slf.close()
    }
}

fn operators_to_config_json(py: Python<'_>, operators: Option<Py<PyAny>>) -> PyResult<String> {
    let Some(obj) = operators else {
        return Ok("{}".to_string());
    };
    let bound = obj.bind(py);
    let json = py.import("json")?;
    let dumps = json.getattr("dumps")?;
    let s: String = dumps.call1((bound,))?.extract()?;
    if s.contains("operators_by_entity") || s.contains("analysis") || s.contains("default_operator") {
        Ok(s)
    } else {
        Ok(format!(r#"{{"operators_by_entity":{s}}}"#))
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Entity>()?;
    m.add_class::<TransformationRecord>()?;
    m.add_class::<AnonymizedResult>()?;
    m.add_class::<AnalysisResult>()?;
    m.add_class::<AegisEngine>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
