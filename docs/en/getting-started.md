# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Getting started

## Prerequisites

- **Docker** (recommended for the full stack) *or*
- **Rust** toolchain (`stable`) for the CLI and engine *or*
- **Python 3.10+** when using the Python SDK (build via `maturin` from `sdk-python/` — see repository status).

## Install with Docker (recommended)

1. Clone the repository and copy the environment template:

```bash
git clone https://github.com/zokastech/aegis.git
cd aegis
cp .env.example .env
# Edit `.env` (from `.env.example`): strong passwords; for local **ZOKA-SENTINEL** NER see `ZOKA_SENTINEL_BUNDLE_URL` or `ZOKA_SENTINEL_FETCH_LATEST` + `ZOKA_SENTINEL_REPO` in `.env.example`.
```

2. Start the stack (gateway, core, Postgres, Redis, optional dashboard):

```bash
docker compose up -d --build
```

3. The gateway listens on **`8443` → container `8080`** by default (`GATEWAY_HTTPS_PORT` in `.env`). Use HTTPS in production; local images may use plain HTTP behind TLS termination.

## Install the Rust CLI from source

```bash
git clone https://github.com/zokastech/aegis.git
cd aegis
cargo build --release -p aegis-cli
./target/release/aegis --help
```

Prebuilt binaries may be attached to [GitHub Releases](https://github.com/zokastech/aegis/releases) when published.

## Python SDK (from source)

The `sdk-python/` crate is built with **maturin** once the PyO3 extension is wired in the repo:

```bash
cd sdk-python
pip install "maturin>=1.4,<2"
maturin develop --release
```

Until the binding is complete, prefer the **HTTP API** or **CLI**.

---

## Five-minute quickstart

### 1) Health

```bash
curl -sS "http://localhost:8080/v1/health"   # adjust host/port to your gateway
```

If the gateway runs with **RBAC / API keys**, add the header: `-H "X-API-Key: YOUR_KEY"`.

### 2) Analyze text

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze" \
  -H "Content-Type: application/json" \
  -d '{"text":"Contact: alice@example.com or +33 6 12 34 56 78"}'
```

### 3) List recognizers

```bash
curl -sS "http://localhost:8080/v1/recognizers"
```

### 4) Anonymize (example)

```bash
curl -sS -X POST "http://localhost:8080/v1/anonymize" \
  -H "Content-Type: application/json" \
  -d '{"text":"Email: bob@company.test","config_json":"{}"}'
```

Use a real `config_json` matching [`Anonymization`](anonymization.md) once you define operators per entity type.

### 5) OpenAPI

```bash
curl -sS "http://localhost:8080/v1/openapi.yaml" | head
```

---

## Next steps

- [Examples](examples.md) — Python, Node.js, notebooks, and `sdk-nodejs` samples explained
- [Architecture](architecture.md) — 3-level pipeline
- [Configuration](configuration.md) — `aegis-config.yaml`
- [API Reference](api-reference.md) — all REST routes
- [Deployment](deployment.md) — Compose, Kubernetes, cloud pointers
