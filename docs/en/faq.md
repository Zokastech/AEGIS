# AEGIS — zokastech.fr — Apache 2.0 / MIT

# FAQ

## Is AEGIS a drop-in replacement for Microsoft Presidio?

Conceptually yes for **analyze + anonymize** flows, but APIs and YAML differ. See [Migration from Presidio](migration-presidio.md).

## Does AEGIS call external services by default?

No. Detection runs **on your infrastructure**. Optional components (downloading an ONNX model URL, LLM proxy upstream) are **explicitly configured**.

## How accurate is detection?

Depends on text language, domain, and pipeline level. Regex is fast but can miss paraphrases; NER improves recall at CPU cost. Always run **domain evaluation** ([Evaluation](evaluation.md)).

## Where is personal data stored?

By default, **transient in memory** for analyze/anonymize. Persistent storage (Postgres, Redis, audit volume) appears when you enable gateway features — scope your DPIA accordingly.

## How do I disable NER?

Set `pipeline_level: 1` or `2`, or omit / unload `ner.model_path`.

## Can I use only the Rust library without HTTP?

Yes — depend on `aegis-core` + `aegis-regex` (+ `aegis-ner` if needed) from the workspace and build with `AnalyzerEngineBuilder`.

## How do I report a security issue?

See [`SECURITY.md`](https://github.com/zokastech/aegis/blob/main/SECURITY.md) — **do not** file public issues for undisclosed vulnerabilities.

## Licenses?

Dual **Apache 2.0** and **MIT** — see repository `LICENSE`.
