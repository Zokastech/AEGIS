// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Adaptateur [`Recognizer`] (aegis-core) autour de [`crate::engine::NerEngine`].

use std::collections::HashMap;
use std::sync::Arc;

use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::Recognizer;

use crate::engine::NerEngine;

/// ONNX NER wrapper for the AEGIS detection pipeline.
pub struct NerRecognizer {
    engine: Arc<NerEngine>,
    name: String,
    min_score: f64,
    languages: Vec<String>,
}

impl NerRecognizer {
    pub fn new(engine: Arc<NerEngine>, name: impl Into<String>) -> Self {
        Self {
            engine,
            name: name.into(),
            min_score: 0.5,
            languages: vec!["*".to_string()],
        }
    }

    pub fn with_min_score(mut self, v: f64) -> Self {
        self.min_score = v;
        self
    }

    pub fn with_languages(mut self, langs: Vec<String>) -> Self {
        self.languages = langs;
        self
    }

    pub fn engine(&self) -> &Arc<NerEngine> {
        &self.engine
    }
}

impl Recognizer for NerRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        let mut v: Vec<EntityType> = self
            .engine
            .config()
            .label_to_entity
            .values()
            .cloned()
            .collect();
        v.sort_by_key(|a| a.config_key());
        v.dedup_by(|a, b| a.config_key() == b.config_key());
        v
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.languages.iter().map(|s| s.as_str()).collect()
    }

    fn min_score(&self) -> f64 {
        self.min_score
    }

    fn analyze(&self, text: &str, cfg: &AnalysisConfig) -> Vec<Entity> {
        let preds = match self.engine.predict(text) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, recognizer = %self.name, "NER ONNX indisponible ou erreur d’inférence");
                return vec![];
            }
        };

        let threshold = cfg.score_threshold.max(self.min_score);
        let allow = cfg.entities_to_analyze.as_ref();

        preds
            .into_iter()
            .filter(|p| p.score >= threshold && allow.is_none_or(|a| a.contains(&p.entity_type)))
            .map(|p| {
                let end = p.end.min(text.len());
                let start = p.start.min(end);
                Entity {
                    entity_type: p.entity_type,
                    start,
                    end,
                    text: text.get(start..end).unwrap_or("").to_string(),
                    score: p.score,
                    recognizer_name: self.name.clone(),
                    metadata: HashMap::new(),
                    decision_trace: None,
                }
            })
            .collect()
    }
}
