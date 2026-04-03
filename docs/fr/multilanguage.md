# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Support multilingue

AEGIS sépare **(a)** quels recognizers sont chargés, **(b)** les indications de langue d’analyse, et **(c)** les lexiques contextuels.

## 1. Filtrer les recognizers intégrés

Dans `aegis-config.yaml` :

```yaml
recognizers:
  default_regex:
    enabled: true
    languages: [en, fr, de, es, it, nl, pt, pl]
```

Seuls restent enregistrés les recognizers dont `supported_languages()` contient `*` ou l’un de ces codes.

## 2. Indication de langue d’analyse

```yaml
analysis:
  language: fr
```

Également surchargeable par requête via `analysis_config_json` sur `POST /v1/analyze`.

## 3. Lexiques du scoreur contextuel

Ajouter ou étendre `context_scorer.languages` :

```yaml
context_scorer:
  languages:
    de:
      person_boost: [Patient, Herr, Frau]
      person_penalty: [Stadt, Land]
      boost_delta: 0.08
      penalty_delta: 0.12
```

## 4. Ajouter une **nouvelle** langue aux recognizers intégrés

Pour les recognizers regex déclarés dans `aegis-regex`, étendre le vecteur de langues dans l’appel `PatternRecognizer::new(...)` (ou équivalent) et recompiler.

Pour les mots de contexte, ajouter une clé sous `context_scorer.languages`.

## 5. Pack d’identifiants nationaux UE

`all_eu_recognizers(&["fr"])` filtre les recognizers par pays. Passer les locales nécessaires lors du câblage Rust.

---

## Pistes

- Sources recognizers : `crates/aegis-regex/src/**/*.rs`
- Config contexte : `crates/aegis-core/src/context/rules.rs`
