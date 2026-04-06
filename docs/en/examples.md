# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Examples (repository)

The **`examples/`** directory contains **runnable samples** that show how to call the **AEGIS HTTP gateway** (the real Rust engine behind `/v1/*`) from Python, Node.js, and Jupyter — plus optional paths for **embedded Node** via `sdk-nodejs`.

All HTTP examples assume a **reachable gateway**. Typical local setups:

| Setup | Base URL | TLS notes |
|-------|-----------|-----------|
| `docker compose up` (repo root) | `https://127.0.0.1:8443` | Self-signed cert → disable verification in clients (`AEGIS_TLS_VERIFY=0`, `AEGIS_TLS_SKIP_VERIFY=1`, or equivalent). |
| Dev compose (`docker-compose.dev.yml`) | `http://127.0.0.1:8080` | Plain HTTP for local iteration. |

If the gateway enforces auth, set **`AEGIS_API_KEY`** and optionally **`AEGIS_API_KEY_HEADER`** (default `X-API-Key`).

**Do not** paste real personal data into demo scripts or notebooks; samples use **synthetic** text.

---

## Python (`examples/python/`)

Install dependencies:

```bash
pip install -r examples/python/requirements.txt
```

Optional (for the LangChain path in `langchain_rag.py`):

```bash
pip install langchain-community langchain-core faiss-cpu
```

| Example | What it demonstrates | How to run (typical) |
|---------|----------------------|----------------------|
| **`quickstart.py`** | Minimal **`POST /v1/analyze`** with `analysis_config_json`, then **`POST /v1/anonymize`** with a small `config_json` (mask email, redact phone). | `python examples/python/quickstart.py` — uses `AEGIS_BASE_URL` (default `http://127.0.0.1:8080`). |
| **`pandas_pipeline.py`** | Large tabular text: builds a **synthetic DataFrame**, sends rows in **batches** to **`/v1/analyze/batch`** to avoid huge HTTP bodies. | `AEGIS_DEMO_ROWS=10000 python examples/python/pandas_pipeline.py` — tune `AEGIS_DEMO_ROWS`, `AEGIS_BATCH_SIZE`. |
| **`fastapi_middleware.py`** | **FastAPI** middleware: for JSON POST bodies, calls the gateway to produce a **PII-stripped** copy for logging/audit while returning the original response to the client. | `AEGIS_BASE_URL=http://127.0.0.1:8080 uvicorn fastapi_middleware:app --reload --port 9090` (from `examples/python/`). Test with `curl` to `/echo` (see file header). |
| **`spark_pii_scan.py`** | **PySpark**: each partition sends batches to **`/v1/analyze/batch`** — pattern for data-lake / cluster jobs where executors must reach the gateway URL. | Requires Java + PySpark. `SPARK_DEMO_ROWS=1000 python examples/python/spark_pii_scan.py`. |
| **`kafka_consumer.py`** | **Event stream**: reads JSON messages with a text `payload`, calls **`/v1/analyze`** per message. **Default** uses an in-memory synthetic queue (no broker). **`--kafka`** uses a real cluster (`KAFKA_BOOTSTRAP_SERVERS`, `KAFKA_TOPIC`, `KAFKA_GROUP_ID`). | `python examples/python/kafka_consumer.py` or `python examples/python/kafka_consumer.py --kafka`. |
| **`dbt_hook.py`** | **SQL governance**: scans compiled or inline SQL for obvious PII-like patterns **before** warehouse promotion; optional **`--live-aegis`** sends excerpts to the gateway. Complements dbt tests, **not** a replacement for `dbt-core`. | `python examples/python/dbt_hook.py --synthetic` or `python examples/python/dbt_hook.py --compiled-dir path/to/target/run/...`. |
| **`langchain_rag.py`** | **RAG hygiene**: anonymizes each chunk via **`/v1/anonymize`** before indexing / LLM context. **Mode 1**: minimal pipeline without LangChain. **Mode 2**: `AEGIS_USE_LANGCHAIN=1` with FAISS + optional LangChain stack if packages are installed. | `python examples/python/langchain_rag.py` or `AEGIS_USE_LANGCHAIN=1 python examples/python/langchain_rag.py`. |

More detail in [`examples/python/README.md`](https://github.com/zokastech/aegis/blob/main/examples/python/README.md).

---

## Node.js (`examples/nodejs/`)

Express demo calling **`POST /v1/anonymize`** with axios; sanitizes JSON payloads on a demo route and exposes **`GET /health`** to check gateway reachability.

```bash
cd examples/nodejs
npm install
AEGIS_BASE_URL=http://127.0.0.1:8080 npm start
```

Example client:

```bash
curl -s -X POST http://127.0.0.1:3000/api/note -H "Content-Type: application/json" \
  -d '{"text":"Mail: demo@example.com"}'
```

HTTPS with self-signed gateway:

```bash
AEGIS_BASE_URL=https://127.0.0.1:8443 AEGIS_TLS_SKIP_VERIFY=1 npm start
```

Full notes: [`examples/nodejs/README.md`](https://github.com/zokastech/aegis/blob/main/examples/nodejs/README.md).

---

## Node.js embedded engine (`sdk-nodejs/examples/`)

To call the **Rust N-API addon** from Node **without** going through HTTP, build the package and use the TypeScript examples:

- **`express-middleware.ts`** — Express integration pattern.
- **`fastify-plugin.ts`** — Fastify plugin pattern.
- **`next-api-route.ts`** — Next.js API route pattern.

Build and paths are described in the `sdk-nodejs/` package README in the repository.

---

## Jupyter notebooks (`examples/notebooks/`)

| Notebook | Purpose |
|----------|---------|
| **`aegis_demo.ipynb`** | HTTP calls to the gateway: analyze, anonymize, short **Pandas** excerpt — good first interactive walkthrough. |
| **`train_ner_pii.ipynb`** | **Train a PII NER model**: synthetic data, JSONL under `datasets/training/`, Hugging Face training, optional Hub push, ONNX export (see `training/README.md`). |
| **`train_ner_hf_public.ipynb`** | Same training pipeline using **public Hugging Face datasets** (E3-JSI + optional Ai4Privacy), merge, train, Hub, ONNX. |

**Testing a trained model** (metrics via `evaluate.py`, ONNX bench, ONNX Runtime smoke, CI tests, Rust L3 integration): see **section 4 — “Tester les modèles entraînés”** in [`training/README.md`](https://github.com/zokastech/aegis/blob/main/training/README.md).

Run Jupyter from the repo root or the notebooks folder:

```bash
pip install jupyter requests pandas
jupyter notebook examples/notebooks/aegis_demo.ipynb
```

Default notebook URL is often **`https://127.0.0.1:8443`** with TLS verification off for the dev certificate; use **`http://127.0.0.1:8080`** with dev compose.

Overview: [`examples/notebooks/README.md`](https://github.com/zokastech/aegis/blob/main/examples/notebooks/README.md).

---

## Related documentation

- [Getting started](getting-started.md) — bring up the gateway and first `curl` calls.
- [API Reference](api-reference.md) — full REST surface.
- [Anonymization](anonymization.md) — `config_json` operators.
- [LLM integration](llm-integration.md) — production patterns around models and PII.
