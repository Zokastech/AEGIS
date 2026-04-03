# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Configuration (`aegis-config.yaml`)

The engine loads **`AegisEngineConfig`** from YAML (path often passed as `--config` / mounted as `/etc/aegis/aegis.yaml` in containers). This file controls recognizers, pipeline depth, NER, context scoring, and default analysis parameters.

!!! tip "File name"
    The repository example is named `aegis-config.yaml`; inside Docker images the same content may be mounted as `aegis.yaml`. The schema is identical.

---

## Top-level keys (overview)

| Key | Purpose |
|-----|---------|
| `recognizers` | Enable/disable default regex pack and per-recognizer disable list |
| `entity_thresholds` | Minimum score per entity type after the pipeline |
| `pipeline_level` | Shorthand: `1`, `2`, or `3` |
| `pipeline` | Fine-grained multi-level pipeline tuning |
| `context_scorer` | Level-2 contextual boosts, combinations, quasi-identifiers |
| `ner` | ONNX model path and runtime options |
| `analysis` | Default `AnalysisConfig` for all passes |

---

## `recognizers`

### `recognizers.default_regex`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Load the built-in regex recognizer pack |
| `languages` | string[] | `["en","fr"]` | Filter recognizers by declared language support (`*` = all) |

Recognizers whose `supported_languages` does not intersect the list are **omitted**.

### `recognizers.disabled`

List of recognizer **names** (case-insensitive) to skip after loading, e.g.:

```yaml
recognizers:
  default_regex:
    enabled: true
    languages: [en, fr, de, es, it, nl, pt, pl]
  disabled:
    - phone_e164_eu
```

---

## `entity_thresholds`

Map of **entity type key** → minimum **final score** (after fusion). Keys follow `EntityType::config_key` (e.g. `EMAIL`, `CREDIT_CARD`, `PERSON`).

```yaml
entity_thresholds:
  EMAIL: 0.82
  CREDIT_CARD: 0.72
  PERSON: 0.55
```

---

## `pipeline_level`

Integer **`1`**, **`2`**, or **`3`**:

| Value | Effect |
|-------|--------|
| 1 | Regex / level-1 only |
| 2 | Level 1 + contextual scorer |
| 3 | Levels 1 + 2 + ONNX NER (if model configured) |

Merged into the detailed `pipeline.levels` enum: `l1_only`, `l1_l2`, `l1_l2_l3`.

---

## `pipeline`

| Field | Type | Default (engine) | Description |
|-------|------|------------------|-------------|
| `levels` | enum | `l1_l2_l3` | `l1_only` / `l1_l2` / `l1_l2_l3` |
| `output_score_threshold` | float | `0.5` | Global cutoff on merged scores |
| `ner_invocation_score_threshold` | float | `0.75` | When to consider invoking NER |
| `short_circuit_l1_score` | float | `0.95` | Strong L1 score skips deeper levels for that span |
| `weight_level1` | float | `0.45` | Fusion weight for L1 |
| `weight_level2` | float | `0.30` | Fusion weight for L2 |
| `weight_level3` | float | `0.25` | Fusion weight for L3 / NER |
| `timeout_level1_ms` | u64 | `2` | Time budget for L1 (0 = unlimited) |
| `timeout_level2_ms` | u64 | `8` | Time budget for L2 |
| `timeout_level3_ms` | u64 | `60` | Time budget for NER |
| `adjacent_merge_gap_chars` | usize | `1` | Merge gap for adjacent same-type spans |
| `overlap_iou_min` | float | `0.35` | Minimum IoU to treat overlaps as same entity |
| `record_decision_trace` | bool | `false` | Extra trace (avoid in prod if verbose) |
| `analysis` | object | defaults | Nested `AnalysisConfig` for pipeline-internal passes |

Nested `pipeline.analysis` fields mirror the root `analysis` block (`language`, `score_threshold`, `return_decision_process`, `context_window_size`).

---

## `context_scorer`

Drives **level 2**. The schema supports both a **legacy** style (`context_window_chars` + `languages`) and a **modern** rule set (`rules`, `tokens_before` / `tokens_after`, `scorer` block). See `crates/aegis-core/src/context/rules.rs` for the full struct tree.

Common fields:

| Field | Description |
|-------|-------------|
| `context_window_chars` | Character window for legacy PERSON-focused context |
| `languages` | Map locale code → `person_boost`, `person_penalty`, `boost_delta`, `penalty_delta` word lists |
| `combinations` | Rules that boost scores when multiple entity types appear within `within_chars` |
| `quasi_identifiers` | Extended quasi-ID combinations, `base_risk_score`, `cap_risk_at` |
| `tokens_before` / `tokens_after` | Token window when using modern rules |
| `rules` | Typed context rules (advanced) |

Example (from repository sample):

```yaml
context_scorer:
  context_window_chars: 96
  languages:
    fr:
      person_boost: [patient, client, M., Mme, monsieur]
      person_penalty: [ville de, pays, région]
      boost_delta: 0.08
      penalty_delta: 0.12
  combinations:
    - within_chars: 120
      require_entity_types: [DATE, LOCATION]
      alert_score_boost: 0.15
      note: quasi_id_date_location
```

---

## `ner`

| Field | Type | Description |
|-------|------|-------------|
| `model_path` | string | Filesystem path to the `.onnx` model inside the container/host |
| `device` | string | e.g. `cpu` |
| `batch_size` | usize | Inference batch size |
| `thread_pool_size` | usize (optional) | Dedicated thread pool for ORT |

```yaml
ner:
  model_path: models/ner-mini.onnx
  device: cpu
  batch_size: 8
  thread_pool_size: 2
```

---

## `analysis` (root)

Default [`AnalysisConfig`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/config.rs):

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `language` | string? | `None` | BCP-47 style hint (`fr`, `en`, …) |
| `score_threshold` | float | `0.5` | Minimum score to emit an entity in this pass (Playground label: *Confidence threshold* — see [Dashboard — Playground](dashboard-playground.md)) |
| `pipeline_level` | int? | `None` | Per-request override: `1` = L1 only, `2` = L1+L2, `3` = L1+L2+L3 (e.g. `analysis_config_json` on `POST /v1/analyze`) |
| `entities_to_analyze` | list? | `None` | Restrict to specific entity types |
| `return_decision_process` | bool | `false` | When `true`, the engine records pipeline steps and attaches `decision_trace` on each entity in the JSON result |
| `context_window_size` | usize | `5` | Context window for analysis |

---

## Anonymization operators (separate concern)

Operator maps per entity type live in **`AegisConfig`** (`operators_by_entity`) — often supplied via API `config_json` or a complementary YAML for anonymization profiles. See [Anonymization](anonymization.md).

---

## Hot reload via API

Admins can `PUT /v1/config` with a YAML **fragment** that merges into the running engine (gateway → FFI). Use with care in production (audit + RBAC).

---

## Reference implementation

- YAML structs: `crates/aegis-core/src/config/engine_yaml.rs`, `crates/aegis-core/src/context/rules.rs`, `crates/aegis-core/src/pipeline/config.rs`
- Example file: `aegis-config.yaml` at repository root
