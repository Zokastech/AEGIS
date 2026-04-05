// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Fusion des sous-tokens (BPE / WordPiece / SentencePiece) et mapping IOB / IOB2 / BILOU.

use aegis_core::entity::EntityType;
use std::collections::HashMap;

use crate::config::ScoreAggregation;
use crate::prediction::{NerPrediction, SubTokenScore};

/// One predicted label for a token aligned to the text.
#[derive(Debug, Clone)]
pub struct TokenPrediction {
    pub raw_label: String,
    pub score: f64,
    pub start: usize,
    pub end: usize,
    pub token_text: String,
    /// Attention mask (0 = padding — ignored).
    pub attention: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IobPrefix {
    O,
    B,
    I,
    L,
    U,
}

fn parse_label(raw: &str) -> (IobPrefix, String) {
    let raw = raw.trim();
    if raw == "O" || raw.eq_ignore_ascii_case("outside") || raw.eq_ignore_ascii_case("pad") {
        return (IobPrefix::O, String::new());
    }
    if let Some(rest) = raw.strip_prefix("B-") {
        return (IobPrefix::B, rest.to_string());
    }
    if let Some(rest) = raw.strip_prefix("I-") {
        return (IobPrefix::I, rest.to_string());
    }
    if let Some(rest) = raw.strip_prefix("L-") {
        return (IobPrefix::L, rest.to_string());
    }
    if let Some(rest) = raw.strip_prefix("U-") {
        return (IobPrefix::U, rest.to_string());
    }
    if let Some(rest) = raw.strip_prefix("S-") {
        return (IobPrefix::U, rest.to_string());
    }
    (IobPrefix::B, raw.to_string())
}

fn resolve_entity_type(suffix: &str, map: &HashMap<String, EntityType>) -> Option<EntityType> {
    if suffix.is_empty() {
        return None;
    }
    map.get(suffix)
        .cloned()
        .or_else(|| map.get(&suffix.to_uppercase()).cloned())
        .or_else(|| map.get(&suffix.to_lowercase()).cloned())
}

fn aggregate_scores(scores: &[f64], how: ScoreAggregation) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    match how {
        ScoreAggregation::Mean => scores.iter().sum::<f64>() / scores.len() as f64,
        ScoreAggregation::Max => scores.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ScoreAggregation::Min => scores.iter().copied().fold(f64::INFINITY, f64::min),
    }
}

type CurrentSpan = (
    String,
    EntityType,
    Vec<f64>,
    usize,
    usize,
    Vec<SubTokenScore>,
);

fn flush_span(
    current: &mut Option<CurrentSpan>,
    source_text: &str,
    out: &mut Vec<NerPrediction>,
    aggregation: ScoreAggregation,
) {
    if let Some((_key, et, scores, start, end, subs)) = current.take() {
        let score = aggregate_scores(&scores, aggregation);
        let text = source_text.get(start..end).unwrap_or("").to_string();
        out.push(NerPrediction {
            entity_type: et,
            text,
            start,
            end,
            score,
            tokens: subs,
        });
    }
}

/// Merge per-token predictions into entities with aggregated scores.
pub fn merge_token_predictions(
    source_text: &str,
    predictions: &[TokenPrediction],
    label_to_entity: &HashMap<String, EntityType>,
    aggregation: ScoreAggregation,
) -> Vec<NerPrediction> {
    let mut out = Vec::new();
    let mut current: Option<CurrentSpan> = None;

    for p in predictions {
        if p.attention == 0 {
            flush_span(&mut current, source_text, &mut out, aggregation);
            continue;
        }

        let (prefix, suffix) = parse_label(&p.raw_label);
        if matches!(prefix, IobPrefix::O) {
            flush_span(&mut current, source_text, &mut out, aggregation);
            continue;
        }

        let Some(et) = resolve_entity_type(&suffix, label_to_entity) else {
            flush_span(&mut current, source_text, &mut out, aggregation);
            continue;
        };

        let sub = SubTokenScore {
            text: p.token_text.clone(),
            start: p.start,
            end: p.end,
            score: p.score,
            label: p.raw_label.clone(),
        };

        match prefix {
            IobPrefix::B | IobPrefix::U => {
                flush_span(&mut current, source_text, &mut out, aggregation);
                current = Some((suffix.clone(), et, vec![p.score], p.start, p.end, vec![sub]));
                if matches!(prefix, IobPrefix::U) {
                    flush_span(&mut current, source_text, &mut out, aggregation);
                }
            }
            IobPrefix::I | IobPrefix::L => {
                if let Some((
                    ref key,
                    ref c_et,
                    ref mut scores,
                    ref mut start,
                    ref mut end,
                    ref mut subs,
                )) = current
                {
                    if key == &suffix && *c_et == et {
                        scores.push(p.score);
                        *start = (*start).min(p.start);
                        *end = (*end).max(p.end);
                        subs.push(sub);
                        if matches!(prefix, IobPrefix::L) {
                            flush_span(&mut current, source_text, &mut out, aggregation);
                        }
                    } else {
                        flush_span(&mut current, source_text, &mut out, aggregation);
                        current =
                            Some((suffix.clone(), et, vec![p.score], p.start, p.end, vec![sub]));
                        if matches!(prefix, IobPrefix::L) {
                            flush_span(&mut current, source_text, &mut out, aggregation);
                        }
                    }
                } else {
                    current = Some((suffix.clone(), et, vec![p.score], p.start, p.end, vec![sub]));
                    if matches!(prefix, IobPrefix::L) {
                        flush_span(&mut current, source_text, &mut out, aggregation);
                    }
                }
            }
            IobPrefix::O => {}
        }
    }

    flush_span(&mut current, source_text, &mut out, aggregation);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    fn map() -> HashMap<String, EntityType> {
        [
            ("PER".into(), EntityType::Person),
            ("LOC".into(), EntityType::Location),
        ]
        .into_iter()
        .collect()
    }

    fn tok(label: &str, score: f64, start: usize, end: usize, text: &str) -> TokenPrediction {
        TokenPrediction {
            raw_label: label.into(),
            score,
            start,
            end,
            token_text: text.into(),
            attention: 1,
        }
    }

    #[test]
    fn merges_iob2_person() {
        let text = "Jean Dupont vit à Paris";
        // Offsets are byte indices (UTF-8); « à » is 2 bytes, so « Paris » starts at byte 19.
        let preds = vec![
            tok("B-PER", 0.9, 0, 4, "Jean"),
            tok("I-PER", 0.85, 5, 11, "Dupont"),
            tok("O", 0.99, 12, 15, "vit"),
            tok("B-LOC", 0.88, 19, 24, "Paris"),
        ];
        let m = merge_token_predictions(text, &preds, &map(), ScoreAggregation::Mean);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].entity_type, EntityType::Person);
        assert_eq!(m[0].text, "Jean Dupont");
        assert_eq!(m[0].start, 0);
        assert_eq!(m[0].end, 11);
        assert_eq!(m[0].tokens.len(), 2);
        assert_eq!(m[1].entity_type, EntityType::Location);
        assert_eq!(m[1].text, "Paris");
    }

    #[test]
    fn bilou_u_single() {
        let text = "X";
        let preds = vec![tok("U-PER", 0.7, 0, 1, "X")];
        let m = merge_token_predictions(text, &preds, &map(), ScoreAggregation::Max);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].score, 0.7);
    }
}
