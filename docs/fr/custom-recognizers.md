# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Recognizers personnalisés

Les détecteurs personnalisés s’écrivent en **Rust** contre le trait `Recognizer` de `aegis-core` ([`Recognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/recognizer.rs)).

## Étape 1 — Implémenter le trait

```rust
use aegis_core::entity::{Entity, EntityType};
use aegis_core::config::AnalysisConfig;
use aegis_core::Recognizer;

struct AcmeInvoiceRecognizer;

impl Recognizer for AcmeInvoiceRecognizer {
    fn name(&self) -> &str {
        "acme_invoice_id"
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![EntityType::Custom("INVOICE_ID".into())]
    }

    fn supported_languages(&self) -> Vec<&str> {
        vec!["*"]
    }

    fn analyze(&self, text: &str, config: &AnalysisConfig) -> Vec<Entity> {
        // Parcourir `text`, émettre Entity { start, end, text, score, recognizer_name, metadata }
        vec![]
    }
}
```

## Étape 2 — Préférer `PatternRecognizer` pour les règles regex

Pour regex + validateur optionnel + bonus contexte, réutiliser [`PatternRecognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-regex/src/pattern.rs) :

```rust
use aegis_regex::PatternRecognizer;
use aegis_core::entity::EntityType;
use regex::Regex;

fn invoice_recognizer() -> PatternRecognizer {
    let re = Regex::new(r"\bINV-\d{6}\b").unwrap();
    PatternRecognizer::new(
        "acme_invoice_id",
        re,
        EntityType::Custom("INVOICE_ID".into()),
        vec!["en", "fr"],
        0.9,
    )
}
```

## Étape 3 — Enregistrer dans le builder du moteur

```rust
use aegis_core::AnalyzerEngineBuilder;
use std::sync::Arc;

let engine = AnalyzerEngineBuilder::new()
    .with_default_recognizers(&["en", "fr"])
    .with_recognizer(Arc::new(invoice_recognizer()))
    .with_engine_yaml_str(include_str!("../aegis-config.yaml"))?
    .build()?;
```

L’ordre compte : les défauts se chargent d’abord ; votre recognizer est ajouté sauf construction manuelle du registre.

## Étape 4 — Câbler `EntityType::Custom`

Les YAML de politique et les cartes d’anonymisation doivent référencer la même clé, ex. `CUSTOM:INVOICE_ID` là où `EntityType::config_key` s’applique.

## Étape 5 — Tester

Ajouter des tests unitaires dans votre crate :

```rust
let ents = recognizer.analyze("See INV-000042 for details", &AnalysisConfig::default());
assert_eq!(ents.len(), 1);
```

## Passerelle / HTTP

La passerelle utilise le moteur **compilé** (FFI). Livrer un nouveau recognizer impose de **reconstruire** l’image `aegis-core` / passerelle — pas de chargeur de plugins dynamique pour l’instant.

---

## Voir aussi

- [Recognizers](recognizers.md) — catalogue intégré
- [Multilingue](multilanguage.md) — filtres de langue
