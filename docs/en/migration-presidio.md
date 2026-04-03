# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Migration from Microsoft Presidio

This guide maps common Presidio concepts to AEGIS. For **positioning and competitive context**, see [Why AEGIS](why-aegis.md). Validate with your **DPO** and **test data** before switching production traffic.

## High-level mapping

| Presidio | AEGIS |
|----------|-------|
| `AnalyzerEngine` | `AnalyzerEngine` (`aegis-core`) |
| Recognizers / recognizer registry | `Recognizer` trait + `RecognizerRegistry` |
| `AnonymizerEngine` | `AnonymizerEngine` (`aegis-anonymize`) |
| REST analyzer service | `aegis-gateway` `/v1/analyze` |
| YAML recognizer config | `aegis-config.yaml` + `recognizers` / `pipeline` |
| Decision process | `return_decision_process`, `record_decision_trace` (use sparingly) |

## Step 1 — Inventory Presidio artifacts

List enabled recognizers, language packs, deny-lists, and anonymization operators. Capture sample payloads from `analyze` and `anonymize` APIs.

## Step 2 — Map operators

| Presidio style | AEGIS `operator_type` |
|----------------|----------------------|
| Replace / redact | `replace`, `redact` |
| Mask | `mask` |
| Hash | `hash` |
| Encrypt / FPE | `encrypt`, `fpe` |

Confirm **reversible** flows use compatible key management (`encrypt`/`fpe` params).

## Step 3 — Recreate policy intent

Port Presidio **analyzer** thresholds to `entity_thresholds` and `pipeline` weights. Port **context** words into `context_scorer.languages`.

## Step 4 — API migration

Replace Presidio HTTP calls with AEGIS **`/v1/analyze`** and **`/v1/anonymize`**. Field names differ — see [API Reference](api-reference.md).

## Step 5 — Evaluation

Run side-by-side benchmarks on **sanitized** datasets: compare counts, spans, and false positives. See [Evaluation](evaluation.md).

## Step 6 — Rollout

- Blue/green or canary the gateway.
- Monitor latency (NER adds tail latency).
- Keep Presidio rollback until metrics stabilize.

---

AEGIS is **not** API-compatible with Presidio without an adapter layer — plan middleware if you need identical URL paths.
