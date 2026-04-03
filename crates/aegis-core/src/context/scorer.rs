// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! [`ContextScorer`]: token window, typed rules, lemmas, quasi-identifiers.

use crate::context::lemma::LemmaAnalyzer;
use crate::context::quasi_id::{QuasiIdentifierDetector, QuasiIdentifierReport};
use crate::context::rules::ContextScorerConfig;
use crate::entity::{Entity, EntityType};
use crate::error::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct CompiledRule {
    entity_type: EntityType,
    boost_multi: Vec<String>,
    boost_single: Vec<String>,
    penalty_multi: Vec<String>,
    penalty_single: Vec<String>,
    boost_amount: f64,
    penalty_amount: f64,
}

#[derive(Debug, Clone)]
struct LegacyPerson {
    boost_by_lang: HashMap<String, HashSet<String>>,
    penalty_by_lang: HashMap<String, HashSet<String>>,
    default_boost: HashSet<String>,
    default_penalty: HashSet<String>,
    char_window: usize,
}

/// Level-2 heuristics: tokenized context, YAML rules, light lemmatization.
#[derive(Debug, Clone)]
pub struct ContextScorer {
    cfg: ContextScorerConfig,
    compiled: Vec<CompiledRule>,
    legacy_person: Option<LegacyPerson>,
    quasi: QuasiIdentifierDetector,
}

impl ContextScorer {
    pub fn new(cfg: ContextScorerConfig) -> Self {
        let compiled = compile_rules(&cfg);
        let legacy_person = if cfg.uses_legacy_person_only() {
            Some(compile_legacy_person(&cfg))
        } else {
            None
        };
        let quasi = QuasiIdentifierDetector::from_config(&cfg);
        Self {
            cfg,
            compiled,
            legacy_person,
            quasi,
        }
    }

    pub fn from_yaml_str(s: &str) -> Result<Self> {
        Ok(Self::new(ContextScorerConfig::from_yaml_str(s)?))
    }

    pub fn default_eu() -> Self {
        let yaml = include_str!("context-rules.yaml");
        Self::from_yaml_str(yaml).expect("embedded context-rules.yaml")
    }

    pub fn config(&self) -> &ContextScorerConfig {
        &self.cfg
    }

    /// Quasi-identifier report (document risk).
    pub fn quasi_report(&self, text: &str, entities: &[Entity]) -> QuasiIdentifierReport {
        self.quasi.assess_document(text, entities)
    }

    /// Adjusts score plus list of matched words / rules.
    pub fn adjust_entity_score(
        &self,
        text: &str,
        entity: &Entity,
        lang_hint: Option<&str>,
    ) -> (f64, Vec<String>) {
        let lang = lang_hint
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_else(|| "fr".into());

        if let Some(rule) = self
            .compiled
            .iter()
            .find(|r| r.entity_type == entity.entity_type)
        {
            return self.adjust_modern(text, entity, &lang, rule);
        }

        if entity.entity_type == EntityType::Person {
            if let Some(ref leg) = self.legacy_person {
                return self.adjust_legacy_person(text, entity, &lang, leg);
            }
        }

        (entity.score, vec![])
    }

    fn adjust_modern(
        &self,
        text: &str,
        entity: &Entity,
        lang: &str,
        rule: &CompiledRule,
    ) -> (f64, Vec<String>) {
        let tokens = tokenize_spans(text);
        let Some((first, last)) = entity_token_range(&tokens, entity) else {
            return (entity.score, vec![]);
        };
        let before = self.cfg.tokens_before.max(1);
        let after = self.cfg.tokens_after.max(1);
        let lo = first.saturating_sub(before);
        let hi = (last + 1 + after).min(tokens.len());
        let context_slice = &tokens[lo..hi];
        let ctx_lower: String = context_slice
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        let mut delta = 0.0f64;
        let mut hits = Vec::new();

        for ph in &rule.boost_multi {
            let p = ph.to_lowercase();
            if ctx_lower.contains(&p) {
                hits.push(format!("boost:{p}"));
                delta += rule.boost_amount;
            }
        }
        for w in &rule.boost_single {
            let wl = w.to_lowercase();
            for tok in context_slice {
                if LemmaAnalyzer::lemma_matches(&tok.text, &wl, lang) {
                    hits.push(format!("boost:{w}"));
                    delta += rule.boost_amount;
                    break;
                }
            }
        }

        for ph in &rule.penalty_multi {
            let p = ph.to_lowercase();
            if ctx_lower.contains(&p) {
                hits.push(format!("penalty:{p}"));
                delta -= rule.penalty_amount;
            }
        }
        for w in &rule.penalty_single {
            let wl = w.to_lowercase();
            for tok in context_slice {
                if LemmaAnalyzer::lemma_matches(&tok.text, &wl, lang) {
                    hits.push(format!("penalty:{w}"));
                    delta -= rule.penalty_amount;
                    break;
                }
            }
        }

        let new_score = (entity.score + delta).clamp(0.0, 1.0);
        (new_score, hits)
    }

    fn adjust_legacy_person(
        &self,
        text: &str,
        entity: &Entity,
        lang: &str,
        leg: &LegacyPerson,
    ) -> (f64, Vec<String>) {
        let win = leg.char_window.max(8);
        let lo = entity.start.saturating_sub(win);
        let hi = (entity.end + win).min(text.len());
        let slice = text.get(lo..hi).unwrap_or("");
        let lower = slice.to_lowercase();

        let (boost_set, pen_set, boost_d, pen_d) = self.legacy_sets(lang, leg);

        let mut hits = Vec::new();
        let mut delta = 0.0f64;
        for w in boost_set {
            if lower.contains(w.as_str()) {
                hits.push(format!("boost:{w}"));
                delta += boost_d;
            }
        }
        for w in pen_set {
            if lower.contains(w.as_str()) {
                hits.push(format!("penalty:{w}"));
                delta -= pen_d;
            }
        }
        let new_score = (entity.score + delta).clamp(0.0, 1.0);
        (new_score, hits)
    }

    fn legacy_sets<'a>(
        &'a self,
        lang: &str,
        leg: &'a LegacyPerson,
    ) -> (
        &'a HashSet<String>,
        &'a HashSet<String>,
        f64,
        f64,
    ) {
        let rules = self
            .cfg
            .languages
            .get(lang)
            .or_else(|| self.cfg.languages.get("en"));
        let bs = leg
            .boost_by_lang
            .get(lang)
            .or_else(|| leg.boost_by_lang.get("en"))
            .unwrap_or(&leg.default_boost);
        let ps = leg
            .penalty_by_lang
            .get(lang)
            .or_else(|| leg.penalty_by_lang.get("en"))
            .unwrap_or(&leg.default_penalty);
        if let Some(r) = rules {
            (
                bs,
                ps,
                r.effective_boost_delta(),
                r.penalty_delta,
            )
        } else {
            (bs, ps, 0.08, 0.12)
        }
    }

    /// Combination boosts (entity indices) — legacy + `quasi_identifiers` section.
    pub fn combination_boosts(&self, entities: &[Entity], text: &str) -> Vec<(usize, f64, String)> {
        self.quasi.entity_boosts(entities, text)
    }
}

#[derive(Debug, Clone)]
struct Tok {
    text: String,
    start: usize,
    end: usize,
}

fn tokenize_spans(text: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut in_word = false;
    let mut start = 0usize;
    for (i, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if in_word {
                if let Some(slice) = text.get(start..i) {
                    out.push(Tok {
                        text: slice.to_string(),
                        start,
                        end: i,
                    });
                }
                in_word = false;
            }
        } else if !in_word {
            start = i;
            in_word = true;
        }
    }
    if in_word {
        let end = text.len();
        if let Some(slice) = text.get(start..end) {
            out.push(Tok {
                text: slice.to_string(),
                start,
                end,
            });
        }
    }
    out
}

fn entity_token_range(tokens: &[Tok], entity: &Entity) -> Option<(usize, usize)> {
    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        let overlaps = t.start < entity.end && t.end > entity.start;
        if overlaps {
            first.get_or_insert(i);
            last = Some(i);
        }
    }
    Some((first?, last?))
}

fn split_multi_single(words: &[String]) -> (Vec<String>, Vec<String>) {
    let mut multi = Vec::new();
    let mut single = Vec::new();
    for w in words {
        if w.split_whitespace().nth(1).is_some() {
            multi.push(w.trim().to_string());
        } else if !w.trim().is_empty() {
            single.push(w.trim().to_string());
        }
    }
    (multi, single)
}

fn compile_rules(cfg: &ContextScorerConfig) -> Vec<CompiledRule> {
    let mut out = Vec::new();
    for r in &cfg.rules {
        let Ok(et) = EntityType::from_config_key(r.entity_type.trim()) else {
            continue;
        };
        let (bm, bs) = split_multi_single(&r.boost_words);
        let (pm, ps) = split_multi_single(&r.penalty_words);
        let bm: Vec<String> = bm.iter().map(|s| s.to_lowercase()).collect();
        let pm: Vec<String> = pm.iter().map(|s| s.to_lowercase()).collect();
        out.push(CompiledRule {
            entity_type: et,
            boost_multi: bm,
            boost_single: bs,
            penalty_multi: pm,
            penalty_single: ps,
            boost_amount: r.boost_amount,
            penalty_amount: r.penalty_amount,
        });
    }
    out
}

fn compile_legacy_person(cfg: &ContextScorerConfig) -> LegacyPerson {
    let mut boost_by_lang: HashMap<String, HashSet<String>> = HashMap::new();
    let mut penalty_by_lang: HashMap<String, HashSet<String>> = HashMap::new();
    let mut default_boost = HashSet::new();
    let mut default_penalty = HashSet::new();
    for (lang, rules) in &cfg.languages {
        let l = lang.to_ascii_lowercase();
        boost_by_lang.insert(
            l.clone(),
            rules
                .person_boost
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<HashSet<String>>(),
        );
        penalty_by_lang.insert(
            l,
            rules
                .person_penalty
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<HashSet<String>>(),
        );
    }
    if let Some(en) = boost_by_lang.get("en") {
        default_boost = en.clone();
    } else if let Some(f) = boost_by_lang.values().next() {
        default_boost = f.clone();
    }
    if let Some(en) = penalty_by_lang.get("en") {
        default_penalty = en.clone();
    } else if let Some(f) = penalty_by_lang.values().next() {
        default_penalty = f.clone();
    }
    let char_window = if cfg.context_window_chars > 0 {
        cfg.context_window_chars
    } else {
        80
    };
    LegacyPerson {
        boost_by_lang,
        penalty_by_lang,
        default_boost,
        default_penalty,
        char_window,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn person(start: usize, end: usize, score: f64) -> Entity {
        Entity {
            entity_type: EntityType::Person,
            start,
            end,
            text: String::new(),
            score,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    fn iban_e(start: usize, end: usize, score: f64) -> Entity {
        Entity {
            entity_type: EntityType::Iban,
            start,
            end,
            text: String::new(),
            score,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    fn bank_acct(start: usize, end: usize, score: f64) -> Entity {
        Entity {
            entity_type: EntityType::BankAccount,
            start,
            end,
            text: String::new(),
            score,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    #[test]
    fn patient_boosts_person() {
        let s = ContextScorer::default_eu();
        let text = "Le patient Jean Dupont est admis.";
        let jean = person(text.find("Jean").unwrap(), text.find("Dupont").unwrap() + "Dupont".len(), 0.7);
        let (sc, hits) = s.adjust_entity_score(text, &jean, Some("fr"));
        assert!(sc > jean.score, "score {sc} hits {hits:?}");
        assert!(hits.iter().any(|h| h.contains("patient")));
    }

    #[test]
    fn ville_de_penalizes_paris_as_person() {
        let s = ContextScorer::default_eu();
        let text = "La ville de Paris organise le forum.";
        let paris = person(
            text.find("Paris").unwrap(),
            text.find("Paris").unwrap() + "Paris".len(),
            0.85,
        );
        let (sc, hits) = s.adjust_entity_score(text, &paris, Some("fr"));
        assert!(sc < paris.score, "score {sc} hits {hits:?}");
        assert!(hits.iter().any(|h| h.contains("ville de") || h.contains("penalty")));
    }

    #[test]
    fn virement_boosts_iban() {
        let s = ContextScorer::default_eu();
        let text = "Virement IBAN FR7630006000011234567890189 effectué.";
        let iban_start = text.find("FR76").unwrap();
        let iban_end = iban_start + "FR7630006000011234567890189".len();
        let ib = iban_e(iban_start, iban_end, 0.75);
        let (sc, hits) = s.adjust_entity_score(text, &ib, Some("fr"));
        assert!(sc > ib.score, "score {sc} hits {hits:?}");
        assert!(hits.iter().any(|h| h.contains("virement")));
    }

    #[test]
    fn identifiant_interne_boosts_bank_account() {
        let s = ContextScorer::default_eu();
        let text = "Le client a également fourni un identifiant interne : usr_kelm_7721";
        let start = text.find("usr_kelm").unwrap();
        let end = start + "usr_kelm_7721".len();
        let e = bank_acct(start, end, 0.52);
        let (sc, hits) = s.adjust_entity_score(text, &e, Some("fr"));
        assert!(sc > e.score, "score {sc} hits {hits:?}");
        assert!(hits.iter().any(|h| h.contains("identifiant interne")));
    }

    #[test]
    fn ancien_login_boosts_bank_account() {
        let s = ContextScorer::default_eu();
        let text = "Et un ancien login : karim1987!";
        let start = text.find("karim1987!").unwrap();
        let end = start + "karim1987!".len();
        let e = bank_acct(start, end, 0.5);
        let (sc, hits) = s.adjust_entity_score(text, &e, Some("fr"));
        assert!(sc > e.score, "score {sc} hits {hits:?}");
        assert!(hits.iter().any(|h| h.contains("ancien login")));
    }

    #[test]
    fn quasi_report_postal_dob_gender() {
        let s = ContextScorer::default_eu();
        let text = "Né le 12/05/1980, demeurant 12 rue X 75001 Paris. Genre : M.";
        let entities = vec![
            Entity {
                entity_type: EntityType::Date,
                start: text.find("12/05/1980").unwrap(),
                end: text.find("12/05/1980").unwrap() + "12/05/1980".len(),
                text: String::new(),
                score: 0.9,
                recognizer_name: "t".into(),
                metadata: HashMap::new(),
                decision_trace: None,
            },
            Entity {
                entity_type: EntityType::Address,
                start: text.find("12 rue").unwrap(),
                end: text.find("Paris").unwrap() + "Paris".len(),
                text: String::new(),
                score: 0.85,
                recognizer_name: "t".into(),
                metadata: HashMap::new(),
                decision_trace: None,
            },
        ];
        let r = s.quasi_report(text, &entities);
        assert!(r.risk_score > 0.0);
        assert!(r.alerts.iter().any(|a| a.contains("postal_dob_gender")));
    }
}
