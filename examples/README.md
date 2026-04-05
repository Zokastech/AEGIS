# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Examples & tutorials

Runnable samples for **AEGIS** ([zokastech.fr](https://zokastech.fr)). Most Python scripts use the **HTTP gateway** (`/v1/analyze`, `/v1/anonymize`) so you only need a running stack or port-forward.

## Prérequis communs

1. **Passerelle AEGIS** (recommandé) : depuis la racine du dépôt  
   `docker compose up -d aegis-gateway`  
   (ou stack complète). Par défaut les scripts utilisent `AEGIS_BASE_URL=http://127.0.0.1:8080` — adaptez le port si votre `docker-compose` mappe `8443:8080` (utilisez alors `http://127.0.0.1:8443`).

2. **Python 3.10+** pour les exemples `python/` :
   ```bash
   pip install -r examples/python/requirements.txt
   ```

3. **Node.js 18+** pour `nodejs/` :
   ```bash
   cd examples/nodejs && npm install
   ```

4. **Go 1.23+** + **Rust** pour `go/` (bibliothèque FFI) — voir `examples/go/README.md`.

## Index

| Exemple | Description |
|---------|-------------|
| [python/quickstart.py](python/quickstart.py) | Analyse + anonymisation en quelques lignes (HTTP) |
| [python/pandas_pipeline.py](python/pandas_pipeline.py) | CSV synthétique 100k lignes, traitement par lots |
| [python/fastapi_middleware.py](python/fastapi_middleware.py) | Middleware FastAPI pour journaux / corps anonymisés |
| [python/spark_pii_scan.py](python/spark_pii_scan.py) | UDF Spark via appels HTTP par partition |
| [python/langchain_rag.py](python/langchain_rag.py) | RAG avec anonymisation amont (LangChain optionnel) |
| [nodejs/express_middleware.js](nodejs/express_middleware.js) | Middleware Express |
| [go/](go/) | CGO contre `libaegis_ffi` |
| [python/kafka_consumer.py](python/kafka_consumer.py) | Consommateur Kafka ou mode `--demo` |
| [python/dbt_hook.py](python/dbt_hook.py) | Scan PII sur artefacts dbt / SQL synthétique |
| [notebooks/aegis_demo.ipynb](notebooks/aegis_demo.ipynb) | Notebook de démonstration |

## Données

Toutes les données sont **synthétiques** (e-mails et numéros factices). Ne collez pas de vraies PII dans les issues publiques.

## Licence

Apache 2.0 / MIT — aligné sur le dépôt principal.
