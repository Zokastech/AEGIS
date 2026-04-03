# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Python examples

### Install

```bash
pip install -r examples/python/requirements.txt
```

Optional (LangChain path in `langchain_rag.py`) :

```bash
pip install langchain-community langchain-core faiss-cpu
```

### Passerelle (moteur Rust réel)

Les scripts appellent l’API HTTP de la gateway (pas de moteur mock côté Python).

- `quickstart.py` : défaut **`https://127.0.0.1:8443`** (TLS local, `verify=False` sauf `AEGIS_TLS_VERIFY=1`).
- Profil dev HTTP : `AEGIS_BASE_URL=http://127.0.0.1:8080`.
- Auth : `AEGIS_API_KEY` (+ optionnel `AEGIS_API_KEY_HEADER`) si la gateway n’est pas en `disable_auth`.

### Fichiers

| Script | Commande |
|--------|----------|
| quickstart | `python quickstart.py` |
| pandas | `AEGIS_DEMO_ROWS=10000 python pandas_pipeline.py` (réduire pour test rapide) |
| FastAPI | `uvicorn fastapi_middleware:app --port 9090` (voir en-tête du fichier) |
| Spark | `SPARK_DEMO_ROWS=1000 python spark_pii_scan.py` |
| LangChain RAG | `python langchain_rag.py` ou `AEGIS_USE_LANGCHAIN=1 python langchain_rag.py` |
| Kafka | `python kafka_consumer.py` (synthétique) ou `python kafka_consumer.py --kafka` |
| dbt hook | `python dbt_hook.py --synthetic` |
