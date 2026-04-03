# AEGIS — zokastech.fr — Apache 2.0 / MIT

# AEGIS documentation

**AEGIS** (Advanced European Guardian for Information Security) is an open-source engine for **detecting and anonymizing personal data (PII)** with an EU-first posture, developed by **[zokastech.fr](https://zokastech.fr)**.

## What you can do with AEGIS

- Run **regex + contextual scoring + optional ONNX NER** over free text.
- **Anonymize** spans using redact, mask, hash, replace, encrypt, FPE, or pseudonymization.
- Expose capabilities through a **hardened HTTP gateway** (Go), **CLI** (Rust), or **SDKs** (Python, Node, Java — maturity varies; see the repository).

## Where to start

| I want to… | Read |
|------------|------|
| Understand **why AEGIS** vs Presidio, Macie, and similar tools | [Why AEGIS — competitive landscape](why-aegis.md) |
| Try AEGIS in 5 minutes | [Getting Started](getting-started.md) |
| Run Python, Node, or notebook samples | [Examples](examples.md) |
| Understand the 3-level pipeline | [Architecture](architecture.md) |
| Call the REST API | [API Reference](api-reference.md) |
| Tune `aegis-config.yaml` | [Configuration](configuration.md) |
| Use the dashboard Playground (confidence threshold) | [Dashboard — Playground](dashboard-playground.md) |
| See built-in detectors | [Recognizers](recognizers.md) |
| Deploy in production | [Deployment](deployment.md) + [Security overview](security/index.md) |
| Configure Zokastech backend / site context | [Zokastech platform](zokastech.md) |
| Deploy AEGIS on AWS, GCP, Azure, OVH | [Cloud providers](cloud-providers.md) |

## On the ZokasTech website

The full documentation is also published at **[zokastech.fr/aegis/docs](https://zokastech.fr/aegis/docs)** inside the ZokasTech application, with the same content (English source) and the ZokasTech visual identity.

## Languages

This site is available in **English** (default), **French**, **German**, **Spanish**, and **Italian** via the language selector. Pages without a translation **fall back to English** automatically.

## License

The project is distributed under **Apache 2.0** and **MIT** (dual-licensed); see the repository `LICENSE` file.

## Brand

Visual identity tokens for Zokastech-aligned UIs: [Brand guidelines](brand-guidelines.md).
