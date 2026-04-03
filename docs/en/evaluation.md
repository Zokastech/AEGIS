# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Evaluation & benchmarks

## Metrics

Typical PII-detection metrics apply:

- **Precision / recall / F1** per entity type (requires labeled data)
- **Latency** p50/p95/p99 per pipeline level (L1 vs L3)
- **Throughput** tokens or documents per second

The engine exposes **Prometheus** histograms/counters — scrape `/metrics` from `aegis-gateway`.

## Benchmarks (repository)

- Rust **Criterion** crates under `benchmarks/` / workspace crate `aegis-benchmarks` (see repository `Makefile` and `benchmarks/README.md`).
- Optional comparison scripts (e.g. Presidio) live in `datasets/` — read each script’s README before running (may require external services).

## Synthetic datasets

`datasets/` contains **synthetic** or test-oriented material — **not** production PHI/PII. Use only compliant data for your jurisdiction.

## HTML performance report

Running benchmark pipelines may copy a report to `docs/performance/report.html` (see `benchmarks/scripts/generate_report.py`). This path is **excluded** from the MkDocs site build but remains in the repo for developers.

## Comparisons

When publishing comparisons to other engines:

- Disclose **version**, **hardware**, and **configuration** (especially `pipeline_level` and model path).
- Report **failure modes** (false negatives) — critical for GDPR fairness discussions.
