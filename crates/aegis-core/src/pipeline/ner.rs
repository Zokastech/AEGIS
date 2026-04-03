// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! NER interface (level 3) — ONNX implementation in `aegis-ner`.

use crate::entity::{Entity, EntityType};
use crate::error::Result;
use std::sync::Arc;

/// NER backend (SLM / ONNX). `aegis-ner` supplies a concrete implementation.
pub trait NerBackend: Send + Sync {
    /// Analyzes a batch of documents (amortizes model latency).
    fn analyze_batch(&self, texts: &[&str], language: Option<&str>) -> Result<Vec<Vec<Entity>>>;
}

/// Mock for tests and benchmarks (negligible latency).
#[derive(Debug, Default, Clone)]
pub struct MockNerBackend {
    /// If empty, returns `[]` for each input.
    pub canned: Vec<Entity>,
}

impl NerBackend for MockNerBackend {
    fn analyze_batch(&self, texts: &[&str], _language: Option<&str>) -> Result<Vec<Vec<Entity>>> {
        Ok(texts
            .iter()
            .map(|t| {
                self.canned
                    .iter()
                    .map(|e| {
                        let end = e.end.min(t.len());
                        let start = e.start.min(end);
                        Entity {
                            entity_type: e.entity_type.clone(),
                            start,
                            end,
                            text: t.get(start..end).unwrap_or("").to_string(),
                            score: e.score,
                            recognizer_name: "mock_ner".into(),
                            metadata: e.metadata.clone(),
                            decision_trace: e.decision_trace.clone(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}

impl MockNerBackend {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Returns a synthetic PERSON entity on the first word when long enough.
    pub fn person_probe() -> Arc<Self> {
        Arc::new(Self { canned: vec![] })
    }
}

/// Entity types where L3 NER is typically useful.
pub fn is_contextual_entity_type(t: &EntityType) -> bool {
    matches!(
        t,
        EntityType::Person | EntityType::Organization | EntityType::Location
    )
}
