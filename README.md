# AEGIS

[![CI](https://img.shields.io/github/actions/workflow/status/zokastech/aegis/ci.yml?branch=main&label=CI&logo=github)](https://github.com/zokastech/aegis/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2B%20MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-workspace-orange?logo=rust)](Cargo.toml)
[![Go](https://img.shields.io/badge/Go-1.22-00ADD8?logo=go)](go.work)
[![zokastech](https://img.shields.io/badge/AEGIS-zokastech.fr-1a56db)](https://zokastech.fr)

**Advanced European Guardian for Information Security** — open-source PII detection and anonymization for text, with an EU and compliance focus; an open alternative to Microsoft Presidio.

**AEGIS by [zokastech.fr](https://zokastech.fr)** — https://zokastech.fr

## Monorepo layout (overview)

> Generated with `tree -I 'target|node_modules|.git' -L 4` (see the repo for the up-to-date tree).

```
.
├── Cargo.toml                 # Rust workspace
├── go.work                    # Go workspace (gateway + policy)
├── aegis-gateway/             # HTTP API (Gin)
├── aegis-policy/              # YAML policy engine (GDPR-oriented packs, etc.)
├── aegis-dashboard/           # React + TypeScript + Vite frontend
├── crates/                    # aegis-core, aegis-regex, aegis-ner, …
├── sdk-python/, sdk-nodejs/, sdk-java/
├── docker/, deploy/helm/aegis/
├── docs/, tests/, benchmarks/, datasets/
└── …
```

## Component status

| Component | Role | Status |
|-----------|------|--------|
| `crates/aegis-core` | Types, traits, analysis engine | MVP |
| `crates/aegis-regex` | Regex recognizers | MVP |
| `crates/aegis-ner` | NER / ONNX (level-3 pipeline) | Skeleton |
| `crates/aegis-anonymize` | Anonymization operators | MVP |
| `crates/aegis-fpe` | Format-preserving encryption (FF3-1) | Skeleton |
| `crates/aegis-ffi` | C FFI | Skeleton |
| `crates/aegis-cli` | `aegis` CLI | MVP |
| `aegis-gateway` | Go REST gateway | MVP |
| `aegis-policy` | Compliance policies (shared Go package) | MVP |
| `aegis-dashboard` | Admin UI (React + TS) | Skeleton |
| `sdk-python` | Python SDK (PyO3 / maturin) | Skeleton |
| `sdk-nodejs` (`@aegis-pii/core`) | Node SDK (NAPI-RS planned) | Skeleton |
| `sdk-java` (`fr.zokastech.aegis:aegis-sdk`) | Java SDK (JNI planned) | Skeleton |
| `deploy/helm/aegis` | Kubernetes chart | Working |
| `docker/` + `docker-compose.dev.yml` | Dev (Rust/Go/Vite/Redis/Postgres) & prod image | Working |
| `docs/` | Security, compliance, guides | In progress |
| `tests/`, `benchmarks/`, `datasets/` | Integration, perf, synthetic data | To extend |

## Why AEGIS over Presidio?

1. **Rust performance** — Core engine in Rust for high-throughput scanning and predictable latency on large texts.
2. **Europe-first** — EU recognizers, IBAN/NIR-style patterns, and GDPR-oriented policy packs by default ([zokastech.fr](https://zokastech.fr)).
3. **Secure by default** — Hardened HTTP gateway (headers, rate limits, optional API keys) instead of an unsecured analyzer-only service.
4. **Integrated dashboard** — `aegis-dashboard` targets DPO/compliance workflows (playground, policies, audit) Presidio does not ship.
5. **Contextual SLM / NER** — Three-level pipeline (regex → context → ONNX NER) for better precision on ambiguous entities.

*French documentation: [MkDocs / Français](https://zokastech.github.io/aegis/fr/).*

## Quick start

### Rust (CLI)

```bash
cargo build --release
./target/release/aegis analyze "Contact : alice@example.com ou +33 6 12 34 56 78"
./target/release/aegis scan fichier.txt
```

### Go (HTTP gateway)

```bash
cd aegis-gateway
export AEGIS_POLICY=policies/gdpr-strict.yaml
# optional: export AEGIS_API_KEY=secret
go run ./cmd/aegis-gateway
curl -s localhost:8080/v1/health
curl -s localhost:8080/v1/analyze -H "Content-Type: application/json" -d '{"text":"mail test@example.com"}'
```

The **`aegis-policy`** module is consumed by the gateway (see root `go.work`).

### Dashboard (React)

```bash
cd aegis-dashboard
npm install && npm run dev
```

### One-command development

```bash
cp .env.example .env   # optional
make dev              # or: just dev  |  docker compose -f docker-compose.dev.yml up --build
```

This starts **aegis-core** (cargo-watch), **aegis-gateway** (air), **aegis-dashboard** (Vite), **Redis**, and **PostgreSQL**. See [.env.example](.env.example).

### Kubernetes (Helm)

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

Details: [deploy/helm/aegis/README.md](deploy/helm/aegis/README.md).

## Documentation

- [Contributing](CONTRIBUTING.md) · [Code of conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [GDPR compliance & technical measures](docs/en/security/rgpd-compliance.md)
- [Hardening](docs/en/security/hardening.md)
- [Threat model (STRIDE)](docs/en/security/threat-model.md)
- [MkDocs site (GitHub Pages)](https://zokastech.github.io/aegis/) — local build: `pip install -r requirements-docs.txt && mkdocs serve` (see `mkdocs.yml`)
- Prometheus / Grafana: [EN](docs/en/monitoring-prometheus-grafana.md) · [FR](docs/fr/monitoring-prometheus-grafana.md)
- [Examples & tutorials](examples/README.md) — Python, Node, Go (CGO), Jupyter notebook
- [Landing page](landing/README.md) — `aegis.zokastech.fr` (Vite + React, `cd landing && npm run build`)
- [Brand guide](assets/brand-guide.md)

## License

- **AEGIS code** (this repository): dual-licensed under **Apache-2.0** *or* **MIT** at your option — see [LICENSE](LICENSE) and [NOTICE](NOTICE).
- **Third-party dependencies & images** (Rust, Go, npm, Python training, Prometheus, Grafana, databases, etc.): their own licenses — summary and inventory commands in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

---

*AEGIS by [zokastech.fr](https://zokastech.fr)*
