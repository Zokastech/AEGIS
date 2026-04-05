# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Custom recognizers

Custom detectors are written in **Rust** against the `aegis-core` trait [`Recognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/recognizer.rs).

## Step 1 — Implement the trait

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
        // Scan `text`, emit Entity { start, end, text, score, recognizer_name, metadata }
        vec![]
    }
}
```

## Step 2 — Prefer `PatternRecognizer` for regex-heavy rules

For regex + optional validator + context boosts, reuse [`PatternRecognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-regex/src/pattern.rs):

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

## Step 3 — Register in the engine builder

```rust
use aegis_core::AnalyzerEngineBuilder;
use std::sync::Arc;

let engine = AnalyzerEngineBuilder::new()
    .with_default_recognizers(&["en", "fr"])
    .with_recognizer(Arc::new(invoice_recognizer()))
    .with_engine_yaml_str(include_str!("../aegis-config.yaml"))?
    .build()?;
```

Order matters: defaults load first; your recognizer is appended unless you build the registry manually.

## Step 4 — Wire `EntityType::Custom`

Policy YAML and anonymization maps must reference the same key, e.g. `CUSTOM:INVOICE_ID` in config keys where `EntityType::config_key` applies.

## Step 5 — Test

Add unit tests in your crate:

```rust
let ents = recognizer.analyze("See INV-000042 for details", &AnalysisConfig::default());
assert_eq!(ents.len(), 1);
```

## Gateway / HTTP

The gateway uses the **compiled** engine (FFI). Shipping a new recognizer requires **rebuilding** the `aegis-core` / gateway image — there is no dynamic plugin loader yet.

---

## Related

- [Recognizers](recognizers.md) — built-in catalog
- [Multi-language](multilanguage.md) — language filters
