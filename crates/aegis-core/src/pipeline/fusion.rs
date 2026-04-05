// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Multi-level score fusion and overlap resolution.

use crate::entity::{Entity, EntityType};
use std::collections::HashMap;

/// Weighting and resolution for entities from multiple levels.
pub struct ScoreFusion;

impl ScoreFusion {
    /// Combines available scores (weights normalized over present levels).
    pub fn fuse_weighted(level_scores: &[(f64, bool)], weights: &[f64; 3]) -> f64 {
        let mut num = 0.0;
        let mut den = 0.0;
        for ((s, present), w) in level_scores.iter().zip(weights.iter()) {
            if *present {
                num += s * w;
                den += w;
            }
        }
        if den <= f64::EPSILON {
            return 0.0;
        }
        (num / den).clamp(0.0, 1.0)
    }

    /// Intersection-over-union on \[start, end) (byte indices).
    pub fn span_iou(a: (usize, usize), b: (usize, usize)) -> f64 {
        let (a_start, a_end) = a;
        let (b_start, b_end) = b;
        let inter = a_end.min(b_end).saturating_sub(a_start.max(b_start));
        if inter == 0 {
            return 0.0;
        }
        let union = a_end.max(b_end).saturating_sub(a_start.min(b_start));
        if union == 0 {
            return 0.0;
        }
        inter as f64 / union as f64
    }

    /// Groups overlapping candidates (IoU ≥ `iou_min`) with the same [`EntityType`],
    /// fusing scores by weighted average of L1–L3 weights present in the cluster.
    pub fn resolve_overlaps(
        items: Vec<FusedCandidate>,
        iou_min: f64,
        weights: &[f64; 3],
    ) -> Vec<FusedCandidate> {
        let mut items = items;
        items.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| b.score.total_cmp(&a.score))
        });
        let mut clusters: Vec<Vec<FusedCandidate>> = Vec::new();
        'outer: for c in items {
            for cl in &mut clusters {
                let same_type = cl.iter().any(|x| x.entity_type == c.entity_type);
                if !same_type {
                    continue;
                }
                if cl.iter().any(|x| {
                    x.entity_type == c.entity_type
                        && Self::span_iou((x.start, x.end), (c.start, c.end)) >= iou_min
                }) {
                    cl.push(c);
                    continue 'outer;
                }
            }
            clusters.push(vec![c]);
        }
        clusters
            .into_iter()
            .map(|cl| merge_cluster(cl, weights))
            .collect()
    }

    /// Merges adjacent same-type entities (gap ≤ `max_gap`); text must be re-filled afterward.
    pub fn merge_adjacent(mut v: Vec<Entity>, max_gap: usize) -> Vec<Entity> {
        if v.is_empty() {
            return v;
        }
        v.sort_by_key(|e| (e.start, e.end));
        let mut out: Vec<Entity> = Vec::new();
        let mut cur = v[0].clone();
        for e in v.into_iter().skip(1) {
            let gap = e.start.saturating_sub(cur.end);
            if e.entity_type == cur.entity_type && gap <= max_gap {
                cur.end = cur.end.max(e.end);
                cur.score = cur.score.max(e.score);
                cur.metadata.insert("adjacent_merged".into(), "1".into());
            } else {
                out.push(cur);
                cur = e;
            }
        }
        out.push(cur);
        out
    }
}

fn merge_cluster(mut cluster: Vec<FusedCandidate>, weights: &[f64; 3]) -> FusedCandidate {
    cluster.sort_by(|a, b| b.score.total_cmp(&a.score));
    let best = &cluster[0];
    let mut l1 = (0.0f64, false);
    let mut l2 = (0.0f64, false);
    let mut l3 = (0.0f64, false);
    let recognizer = best.recognizer_name.clone();
    for c in &cluster {
        if let Some(&s) = c.sources.get("l1") {
            l1.0 = l1.0.max(s);
            l1.1 = true;
        }
        if let Some(&s) = c.sources.get("l2") {
            l2.0 = l2.0.max(s);
            l2.1 = true;
        }
        if let Some(&s) = c.sources.get("l3") {
            l3.0 = l3.0.max(s);
            l3.1 = true;
        }
    }
    if !l1.1 && !l2.1 && !l3.1 {
        l1 = (best.score, true);
    }
    let fused = ScoreFusion::fuse_weighted(&[l1, l2, l3], weights);
    let start = cluster.iter().map(|c| c.start).min().unwrap_or(best.start);
    let end = cluster.iter().map(|c| c.end).max().unwrap_or(best.end);
    let mut sources = HashMap::new();
    for c in &cluster {
        for (k, v) in &c.sources {
            sources
                .entry(k.clone())
                .and_modify(|x: &mut f64| *x = x.max(*v))
                .or_insert(*v);
        }
    }
    let score = if l2.1 || l3.1 {
        fused
    } else {
        l1.0.max(best.score)
    };
    FusedCandidate {
        entity_type: best.entity_type.clone(),
        start,
        end,
        score,
        recognizer_name: recognizer,
        sources,
    }
}

/// Intermediate candidate (spans + per-level scores).
#[derive(Debug, Clone)]
pub struct FusedCandidate {
    pub entity_type: EntityType,
    pub start: usize,
    pub end: usize,
    pub score: f64,
    pub recognizer_name: String,
    pub sources: HashMap<String, f64>,
}

impl FusedCandidate {
    pub fn from_entity_l1(e: &Entity) -> Self {
        let mut sources = HashMap::new();
        sources.insert("l1".into(), e.score);
        Self {
            entity_type: e.entity_type.clone(),
            start: e.start,
            end: e.end,
            score: e.score,
            recognizer_name: e.recognizer_name.clone(),
            sources,
        }
    }

    pub fn from_entity_tagged(e: &Entity, tag: &str) -> Self {
        let mut sources = HashMap::new();
        sources.insert(tag.into(), e.score);
        Self {
            entity_type: e.entity_type.clone(),
            start: e.start,
            end: e.end,
            score: e.score,
            recognizer_name: e.recognizer_name.clone(),
            sources,
        }
    }

    pub fn into_entity_with_text(self, source: &str) -> Entity {
        let text = source.get(self.start..self.end).unwrap_or("").to_string();
        let mut meta = HashMap::new();
        meta.insert("pipeline".into(), "1".into());
        Entity {
            entity_type: self.entity_type,
            start: self.start,
            end: self.end,
            text,
            score: self.score.clamp(0.0, 1.0),
            recognizer_name: self.recognizer_name,
            metadata: meta,
            decision_trace: None,
        }
    }
}
