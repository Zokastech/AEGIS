// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Context analysis engine (level 2): token window, YAML rules, lemmas, quasi-identifiers.

mod lemma;
mod quasi_id;
mod rules;
mod scorer;

pub use lemma::LemmaAnalyzer;
pub use quasi_id::{QuasiIdentifierDetector, QuasiIdentifierReport};
pub use rules::{
    CombinationRule, ContextRule, ContextScorerConfig, LanguageContextRules, QuasiComboRuleYaml,
    QuasiIdentifierYamlSection, ScorerWindowSettings, ScorerYamlBlock,
};
pub use scorer::ContextScorer;
