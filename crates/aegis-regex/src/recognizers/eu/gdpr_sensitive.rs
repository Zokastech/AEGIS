// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Linguistic indicators for GDPR Art. 9 sensitive categories — low scores, human review recommended.

use crate::recognizers::national_id::composite::{CompositeNationalRecognizer, IdRule};
use aegis_core::config::AnalysisConfig;
use aegis_core::entity::{Entity, EntityType};
use aegis_core::recognizer::Recognizer;
use regex::Regex;
use std::sync::Arc;

fn always(_: &str) -> bool {
    true
}

/// Wrapper that flags hits as requiring human review.
pub struct GdprArt9Recognizer {
    inner: CompositeNationalRecognizer,
}

impl Recognizer for GdprArt9Recognizer {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        self.inner.supported_entities()
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.inner.supported_languages()
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        let mut v = self.inner.analyze(text, config);
        for e in &mut v {
            e.metadata
                .insert("human_review_recommended".into(), "true".into());
            e.metadata.insert("gdpr_article".into(), "9".into());
        }
        v
    }

    fn min_score(&self) -> f64 {
        self.inner.min_score()
    }
}

/// Keywords by broad category (neutral / legal phrasing).
pub fn gdpr_art9_sensitive_recognizer() -> GdprArt9Recognizer {
    let ctx = [
        "données sensibles",
        "sensitive data",
        "besondere Kategorien",
        "categoría especial",
        "categorie particolari",
        "bijzondere persoonsgegevens",
        "art. 9",
        "article 9",
        "Artikel 9",
    ];
    let rules = vec![
        IdRule {
            name: "gdpr_racial_ethnic",
            re: Regex::new(
                r"(?xi)\b(
                    racial\s+or\s+ethnic\s+origin
                    |origine\s+(?:racial|ethnique)
                    |ethnische\s+Herkunft
                    |origen\s+(?:racial|étnico|etnico)
                    |origine\s+etnica
                    |ras\s+of\s+etnische\s+afkomst
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.45,
        },
        IdRule {
            name: "gdpr_political",
            re: Regex::new(
                r"(?xi)\b(
                    political\s+opinions?
                    |opinions?\s+politiques
                    |politische\s+Meinung(?:en)?
                    |opini[oó]n\s+pol[ií]tica
                    |opinioni\s+politiche
                    |politieke\s+opvattingen
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.42,
        },
        IdRule {
            name: "gdpr_religious_philosophical",
            re: Regex::new(
                r"(?xi)\b(
                    religious\s+or\s+philosophical\s+beliefs?
                    |convictions?\s+religieuses
                    |Weltanschauung
                    |creencias?\s+religiosas
                    |cred(?:o|enze)\s+religios[ae]
                    |levensbeschouwing
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.42,
        },
        IdRule {
            name: "gdpr_biometric",
            re: Regex::new(
                r"(?xi)\b(
                    biometric\s+data
                    |données?\s+biométriques
                    |Biometr(?:ie|ische\s+Daten)
                    |datos?\s+biométricos
                    |dati\s+biometrici
                    |biometrische\s+gegevens
                    |template\s+biométrique
                    |fingerprint\s+template
                    |facial\s+recognition\s+template
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.48,
        },
        IdRule {
            name: "gdpr_sexual_orientation",
            re: Regex::new(
                r"(?xi)\b(
                    sexual\s+orientation
                    |orientation\s+sexuelle
                    |sexuelle\s+Orientierung
                    |orientaci[oó]n\s+sexual
                    |orientamento\s+sessuale
                    |seksuele\s+geaardheid
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.4,
        },
        IdRule {
            name: "gdpr_trade_union",
            re: Regex::new(
                r"(?xi)\b(
                    trade\s+union\s+membership
                    |appartenance\s+syndicale
                    |Gewerkschaftszugehörigkeit
                    |afiliaci[oó]n\s+sindical
                    |appartenenza\s+sindacale
                )\b",
            )
            .unwrap(),
            entity: EntityType::Custom("GDPR_ART9".into()),
            validator: Arc::new(always),
            base_score: 0.41,
        },
    ];
    let inner = CompositeNationalRecognizer::new("gdpr_art9_keywords", rules, vec!["*"], &ctx)
        .with_min_score(0.28);
    GdprArt9Recognizer { inner }
}
