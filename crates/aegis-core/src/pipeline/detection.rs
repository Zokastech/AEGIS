// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Orchestration des niveaux 1 (regex), 2 (contexte), 3 (NER).

use crate::config::AnalysisConfig;
use crate::context::ContextScorer;
use crate::entity::{AnalysisResult, Entity, EntityType};
use crate::error::Result;
use crate::pipeline::config::PipelineConfig;
use crate::pipeline::fusion::{FusedCandidate, ScoreFusion};
use crate::pipeline::ner::{is_contextual_entity_type, NerBackend};
use crate::pipeline::trace::{DecisionTrace, TraceStep};
use crate::recognizer::Recognizer;
use rayon::prelude::*;
use rayon::ThreadPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// How level 3 (NER) related to this document — used to enrich per-entity JSON traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L3TraceAttachment {
    /// Pipeline stops at L1 or L2.
    NotApplicable,
    /// L3 enabled but no [`NerBackend`] (e.g. missing model path).
    NoBackend,
    /// Backend present but `analyze_batch` was not called (invocation gate).
    SkippedInvocation,
    /// NER batch ran (may still yield zero spans).
    Executed,
}

/// Pipeline analysis result (output + optional trace).
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub analysis: AnalysisResult,
    pub trace: Option<DecisionTrace>,
    pub l3_trace_attachment: L3TraceAttachment,
}

impl PipelineOutput {
    pub fn into_analysis_result(self) -> AnalysisResult {
        self.analysis
    }
}

struct WorkingEntity {
    entity: Entity,
    l1_score: f64,
    l2_score: Option<f64>,
    short_circuit: bool,
}

/// Pipeline multi-niveaux **AEGIS** (zokastech.fr).
pub struct DetectionPipeline {
    config: PipelineConfig,
    level1: Vec<Arc<dyn Recognizer>>,
    context: ContextScorer,
    ner: Option<Arc<dyn NerBackend>>,
    /// Thread pool for NER calls (optional).
    ner_pool: Option<Arc<ThreadPool>>,
}

impl DetectionPipeline {
    pub fn new(
        config: PipelineConfig,
        level1: Vec<Arc<dyn Recognizer>>,
        context: ContextScorer,
        ner: Option<Arc<dyn NerBackend>>,
    ) -> Self {
        Self {
            config,
            level1,
            context,
            ner,
            ner_pool: None,
        }
    }

    pub fn with_ner_pool(mut self, pool: Option<Arc<ThreadPool>>) -> Self {
        self.ner_pool = pool;
        self
    }

    /// Analyse un document ; retourne uniquement [`AnalysisResult`] (sans trace).
    pub fn analyze(&self, text: &str) -> Result<AnalysisResult> {
        self.analyze_detailed(text).map(|o| o.analysis)
    }

    /// Analyze with decision trace when `record_decision_trace` is enabled.
    pub fn analyze_detailed(&self, text: &str) -> Result<PipelineOutput> {
        let t0 = Instant::now();
        let mut trace = if self.config.record_decision_trace {
            Some(DecisionTrace::default())
        } else {
            None
        };
        let mut l3_trace_attachment = L3TraceAttachment::NotApplicable;
        let lang = self.config.analysis.language.as_deref().or(Some("en"));

        let weights = [
            self.config.weight_level1,
            self.config.weight_level2,
            self.config.weight_level3,
        ];

        // ——— Niveau 1 ———
        let t_l1 = Instant::now();
        let mut l1_entities: Vec<Entity> = if self.config.levels.l1() && !self.level1.is_empty() {
            let cfg = self.config.analysis.clone();
            let chunks: Vec<Vec<Entity>> = self
                .level1
                .par_iter()
                .map(|r| r.analyze(text, &cfg))
                .collect();
            chunks.into_iter().flatten().collect()
        } else {
            Vec::new()
        };

        if self.config.timeout_level1_ms > 0
            && t_l1.elapsed().as_millis() as u64 > self.config.timeout_level1_ms
        {
            tracing::warn!(target: "aegis_core::pipeline", "L1 timeout budget exceeded");
            if let Some(ref mut tr) = trace {
                tr.push(TraceStep {
                    level: 1,
                    action: "timeout".into(),
                    span_start: 0,
                    span_end: 0,
                    entity_type: EntityType::Custom("TIMEOUT".into()),
                    scores_by_source: HashMap::new(),
                    context_word_hits: vec![],
                    short_circuit: false,
                    note: Some("l1_budget".into()),
                });
            }
        }

        l1_entities.sort_by_key(|e| (e.start, e.end));
        l1_entities.retain(|e| e.score >= self.config.analysis.score_threshold);
        if let Some(ref filter) = self.config.analysis.entities_to_analyze {
            l1_entities.retain(|e| filter.contains(&e.entity_type));
        }

        let mut work: Vec<WorkingEntity> = l1_entities
            .into_iter()
            .map(|e| {
                let sc = e.score >= self.config.short_circuit_l1_score;
                let l1 = e.score;
                WorkingEntity {
                    entity: e,
                    l1_score: l1,
                    l2_score: None,
                    short_circuit: sc,
                }
            })
            .collect();

        // ——— Niveau 2 ———
        if self.config.levels.l2() {
            let t_l2 = Instant::now();
            for w in &mut work {
                if w.short_circuit {
                    if let Some(ref mut tr) = trace {
                        tr.push(TraceStep {
                            level: 2,
                            action: "skipped_short_circuit".into(),
                            span_start: w.entity.start,
                            span_end: w.entity.end,
                            entity_type: w.entity.entity_type.clone(),
                            scores_by_source: [("l1".into(), w.l1_score)].into_iter().collect(),
                            context_word_hits: vec![],
                            short_circuit: true,
                            note: None,
                        });
                    }
                    continue;
                }
                let (new_score, hits) = self.context.adjust_entity_score(text, &w.entity, lang);
                w.l2_score = Some(new_score);
                w.entity.score = new_score;
                if let Some(ref mut tr) = trace {
                    let mut src = HashMap::new();
                    src.insert("l1".into(), w.l1_score);
                    src.insert("l2".into(), new_score);
                    tr.push(TraceStep {
                        level: 2,
                        action: "context_adjust".into(),
                        span_start: w.entity.start,
                        span_end: w.entity.end,
                        entity_type: w.entity.entity_type.clone(),
                        scores_by_source: src,
                        context_word_hits: hits,
                        short_circuit: false,
                        note: None,
                    });
                }
            }

            // second pass : boosts de combinaison (indices stables)
            let ents: Vec<Entity> = work.iter().map(|w| w.entity.clone()).collect();
            for (idx, delta, note) in self.context.combination_boosts(&ents, text) {
                if let Some(w) = work.get_mut(idx) {
                    if !w.short_circuit {
                        w.entity.score = (w.entity.score + delta).clamp(0.0, 1.0);
                        w.l2_score = Some(w.entity.score);
                        if let Some(ref mut tr) = trace {
                            tr.push(TraceStep {
                                level: 2,
                                action: "quasi_identifier".into(),
                                span_start: w.entity.start,
                                span_end: w.entity.end,
                                entity_type: w.entity.entity_type.clone(),
                                scores_by_source: [("combo".into(), delta)].into_iter().collect(),
                                context_word_hits: vec![],
                                short_circuit: false,
                                note: Some(note),
                            });
                        }
                    }
                }
            }

            if self.config.timeout_level2_ms > 0
                && t_l2.elapsed().as_millis() as u64 > self.config.timeout_level2_ms
            {
                tracing::warn!(target: "aegis_core::pipeline", "L2 timeout budget exceeded");
            }
        }

        // ——— Niveau 3 ———
        let mut l3_entities: Vec<Entity> = Vec::new();
        if self.config.levels.l3() {
            match &self.ner {
                None => {
                    l3_trace_attachment = L3TraceAttachment::NoBackend;
                }
                Some(ner) => {
                    let run = work.iter().any(|w| {
                        !w.short_circuit
                            && (is_contextual_entity_type(&w.entity.entity_type)
                                || w.entity.score < self.config.ner_invocation_score_threshold)
                    });
                    if run {
                        l3_trace_attachment = L3TraceAttachment::Executed;
                        let t_l3 = Instant::now();
                        let batch = if let Some(pool) = &self.ner_pool {
                            pool.install(|| ner.analyze_batch(&[text], lang))?
                        } else {
                            ner.analyze_batch(&[text], lang)?
                        };
                        l3_entities = batch.into_iter().next().unwrap_or_default();
                        if let Some(ref mut tr) = trace {
                            for e in &l3_entities {
                                tr.push(TraceStep {
                                    level: 3,
                                    action: "ner_hit".into(),
                                    span_start: e.start,
                                    span_end: e.end,
                                    entity_type: e.entity_type.clone(),
                                    scores_by_source: [("l3".into(), e.score)]
                                        .into_iter()
                                        .collect(),
                                    context_word_hits: vec![],
                                    short_circuit: false,
                                    note: None,
                                });
                            }
                        }
                        if self.config.timeout_level3_ms > 0
                            && t_l3.elapsed().as_millis() as u64 > self.config.timeout_level3_ms
                        {
                            tracing::warn!(target: "aegis_core::pipeline", "L3 timeout budget exceeded");
                        }
                    } else {
                        l3_trace_attachment = L3TraceAttachment::SkippedInvocation;
                    }
                }
            }
        }

        // ——— Fusion ———
        let mut candidates: Vec<FusedCandidate> = Vec::new();
        for w in &work {
            let mut sources = HashMap::new();
            sources.insert("l1".into(), w.l1_score);
            if let Some(s2) = w.l2_score {
                sources.insert("l2".into(), s2);
            } else if !w.short_circuit && self.config.levels.l2() {
                sources.insert("l2".into(), w.entity.score);
            }
            let fused_l12 = ScoreFusion::fuse_weighted(
                &[
                    (w.l1_score, true),
                    (
                        w.l2_score.unwrap_or(w.l1_score),
                        self.config.levels.l2() && !w.short_circuit,
                    ),
                    (0.0, false),
                ],
                &weights,
            );
            let score = if w.short_circuit {
                w.l1_score
            } else if self.config.levels.l2() {
                fused_l12
            } else {
                w.l1_score
            };
            candidates.push(FusedCandidate {
                entity_type: w.entity.entity_type.clone(),
                start: w.entity.start,
                end: w.entity.end,
                score,
                recognizer_name: w.entity.recognizer_name.clone(),
                sources,
            });
        }
        for e in &l3_entities {
            candidates.push(FusedCandidate::from_entity_tagged(e, "l3"));
        }

        let merged =
            ScoreFusion::resolve_overlaps(candidates, self.config.overlap_iou_min, &weights);
        let mut entities: Vec<Entity> = merged
            .into_iter()
            .map(|c| c.into_entity_with_text(text))
            .collect();
        entities = ScoreFusion::merge_adjacent(entities, self.config.adjacent_merge_gap_chars);
        for e in &mut entities {
            if e.end <= text.len() {
                e.text = text[e.start..e.end].to_string();
            }
        }
        entities.retain(|e| e.score >= self.config.output_score_threshold);

        let elapsed = t0.elapsed().as_millis() as u64;
        Ok(PipelineOutput {
            analysis: AnalysisResult {
                entities,
                processing_time_ms: elapsed,
                language_detected: self.config.analysis.language.clone(),
                text_length: text.len(),
            },
            trace,
            l3_trace_attachment,
        })
    }

    /// Parallel analysis of multiple documents (Rayon).
    pub fn analyze_batch(&self, texts: &[&str]) -> Result<Vec<AnalysisResult>> {
        texts.par_iter().map(|t| self.analyze(t)).collect()
    }

    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    pub fn with_analysis_config(mut self, analysis: AnalysisConfig) -> Self {
        self.config.analysis = analysis;
        self
    }
}
