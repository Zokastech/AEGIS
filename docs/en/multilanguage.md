# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Multi-language support

AEGIS separates **(a)** which recognizers load, **(b)** analysis language hints, and **(c)** contextual lexicons.

## 1. Filter built-in recognizers

In `aegis-config.yaml`:

```yaml
recognizers:
  default_regex:
    enabled: true
    languages: [en, fr, de, es, it, nl, pt, pl]
```

Only recognizers whose `supported_languages()` contains `*` or one of these codes stay registered.

## 2. Analysis language hint

```yaml
analysis:
  language: fr
```

Also overridable per request via `analysis_config_json` on `POST /v1/analyze`.

## 3. Context scorer lexicons

Add or extend `context_scorer.languages`:

```yaml
context_scorer:
  languages:
    de:
      person_boost: [Patient, Herr, Frau]
      person_penalty: [Stadt, Land]
      boost_delta: 0.08
      penalty_delta: 0.12
```

## 4. Adding a **new** language to built-in recognizers

For regex recognizers declared in `aegis-regex`, extend the language vector in the `PatternRecognizer::new(...)` call (or equivalent) and rebuild.

For context words, add a new key under `context_scorer.languages`.

## 5. EU national ID pack

`all_eu_recognizers(&["fr"])` filters country-specific recognizers. Pass the locales you need when wiring Rust.

---

## Pointers

- Recognizer sources: `crates/aegis-regex/src/**/*.rs`
- Context config: `crates/aegis-core/src/context/rules.rs`
