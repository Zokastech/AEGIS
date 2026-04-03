# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Dashboard — Confidence threshold (Playground)

The **Playground** in `aegis-dashboard` lets you experiment with detection. The **Confidence threshold** control is tightly linked to the engine’s `score_threshold` and to a **preview** that updates as you move the slider.

---

## What “above / below the threshold” means

The hint in the UI:

- **Above the threshold** → entity **kept**
- **Below the threshold** → entity **ignored**

refers to the **final confidence score** of each detected span (after the pipeline: L1 recognizers, optional L2 context, optional L3 NER, then fusion). For every entity, the engine checks:

`entity.score >= score_threshold`

- If **true**, the entity is included in the JSON returned by `POST /v1/analyze`.
- If **false**, that entity is **dropped** and does not appear in the response.

The Playground sends this value as **`score_threshold`** inside `analysis_config_json` (same object is also used when you run **Anonymize**, so analyze and anonymize stay aligned).

Scores are always in the range **[0, 1]**.

---

## The line: “At 0.75 : 1 detected · 0 ignored”

This line is a **live summary based on the last Analyze result**, not a hidden second request:

1. You click **Analyze** → the gateway returns entities, each with a `score`.
2. The slider shows a cutoff, e.g. **0.75**.
3. The UI counts:
   - **Detected** = number of entities from that result with `score ≥ 0.75` (they would still be returned if you ran Analyze again with threshold 0.75).
   - **Ignored** = the rest (`score < 0.75`); they would be **filtered out** by the engine at that threshold.

**Example:** one email with score **0.92** and threshold **0.75** → **1 detected · 0 ignored**, because `0.92 ≥ 0.75`.

Changing the slider **only** updates this preview (and the chart) until you click **Analyze** again. To apply a new cutoff to the actual API call, run **Analyze** (or **Anonymize**) after moving the slider.

---

## Chart under the slider

The curves show, for many possible thresholds on the horizontal axis, how many entities from the **last** analysis would count as **detected** vs **ignored**. Use it to compare cutoffs without calling the API for every value.

---

## How this relates to `aegis-config.yaml`

- **`analysis.score_threshold`** (and the Playground field of the same name) is the **per-request / default** cutoff for emitting entities in that pass.
- **`entity_thresholds`** and **`pipeline.output_score_threshold`** are **additional** server-side rules; see [Configuration](configuration.md).

For API field names and JSON shape, see [API Reference](api-reference.md).
