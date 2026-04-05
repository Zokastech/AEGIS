# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Exemples (dépôt)

Le répertoire **`examples/`** regroupe des **exemples exécutables** montrant comment appeler la **passerelle HTTP AEGIS** (moteur Rust réel derrière `/v1/*`) depuis Python, Node.js et Jupyter — ainsi que des pistes pour le **moteur embarqué Node** via `sdk-nodejs`.

Tous les exemples HTTP supposent une **passerelle joignable**. Configurations locales typiques :

| Environnement | URL de base | TLS |
|---------------|-------------|-----|
| `docker compose up` (racine du dépôt) | `https://127.0.0.1:8443` | Certificat auto-signé → désactiver la vérification côté client (`AEGIS_TLS_VERIFY=0`, `AEGIS_TLS_SKIP_VERIFY=1`, ou équivalent). |
| Compose dev (`docker-compose.dev.yml`) | `http://127.0.0.1:8080` | HTTP clair pour le développement local. |

Si la passerelle impose une authentification, définissez **`AEGIS_API_KEY`** et éventuellement **`AEGIS_API_KEY_HEADER`** (défaut `X-API-Key`).

**N’insérez pas** de vraies données personnelles dans les scripts ou notebooks de démo ; les textes sont **synthétiques**.

---

## Python (`examples/python/`)

Installation :

```bash
pip install -r examples/python/requirements.txt
```

Optionnel (chemin LangChain dans `langchain_rag.py`) :

```bash
pip install langchain-community langchain-core faiss-cpu
```

| Exemple | Rôle | Lancement (typique) |
|---------|------|---------------------|
| **`quickstart.py`** | Appels minimaux **`POST /v1/analyze`** avec `analysis_config_json`, puis **`POST /v1/anonymize`** avec un petit `config_json` (masquer e-mail, redacter téléphone). | `python examples/python/quickstart.py` — `AEGIS_BASE_URL` (défaut `http://127.0.0.1:8080`). |
| **`pandas_pipeline.py`** | Données tabulaires volumineuses : DataFrame **synthétique**, envoi par **lots** à **`/v1/analyze/batch`**. | `AEGIS_DEMO_ROWS=10000 python examples/python/pandas_pipeline.py` — ajuster `AEGIS_DEMO_ROWS`, `AEGIS_BATCH_SIZE`. |
| **`fastapi_middleware.py`** | **Middleware FastAPI** : pour les POST JSON, appelle la passerelle pour journaliser une copie **sans PII** tout en répondant au client avec le corps d’origine. | `AEGIS_BASE_URL=http://127.0.0.1:8080 uvicorn fastapi_middleware:app --reload --port 9090` (depuis `examples/python/`). Tester avec `curl` sur `/echo` (voir l’en-tête du fichier). |
| **`spark_pii_scan.py`** | **PySpark** : chaque partition envoie des lots à **`/v1/analyze/batch`** — schéma type data lake / cluster (les exécuteurs doivent joindre l’URL de la passerelle). | Java + PySpark requis. `SPARK_DEMO_ROWS=1000 python examples/python/spark_pii_scan.py`. |
| **`kafka_consumer.py`** | **Flux d’événements** : messages JSON avec `payload` textuel, **`/v1/analyze`** par message. **Par défaut** file synthétique en mémoire (sans broker). **`--kafka`** pour un cluster réel (`KAFKA_BOOTSTRAP_SERVERS`, `KAFKA_TOPIC`, `KAFKA_GROUP_ID`). | `python examples/python/kafka_consumer.py` ou `python examples/python/kafka_consumer.py --kafka`. |
| **`dbt_hook.py`** | **Gouvernance SQL** : analyse de SQL compilé ou inline pour motifs type PII **avant** promotion vers l’entrepôt ; option **`--live-aegis`** pour envoyer des extraits à la passerelle. Complète les tests dbt, **ne remplace pas** `dbt-core`. | `python examples/python/dbt_hook.py --synthetic` ou `python examples/python/dbt_hook.py --compiled-dir chemin/vers/target/run/...`. |
| **`langchain_rag.py`** | **RAG « propre »** : anonymisation de chaque extrait via **`/v1/anonymize`** avant indexation / contexte LLM. **Mode 1** : pipeline minimal sans LangChain. **Mode 2** : `AEGIS_USE_LANGCHAIN=1` avec FAISS + stack LangChain si installée. | `python examples/python/langchain_rag.py` ou `AEGIS_USE_LANGCHAIN=1 python examples/python/langchain_rag.py`. |

Détails : [`examples/python/README.md`](https://github.com/zokastech/aegis/blob/main/examples/python/README.md).

---

## Node.js (`examples/nodejs/`)

Démo **Express** qui appelle **`POST /v1/anonymize`** (axios) ; assainit des charges JSON sur une route de démo et expose **`GET /health`** pour vérifier la passerelle.

```bash
cd examples/nodejs
npm install
AEGIS_BASE_URL=http://127.0.0.1:8080 npm start
```

Exemple client :

```bash
curl -s -X POST http://127.0.0.1:3000/api/note -H "Content-Type: application/json" \
  -d '{"text":"Mail: demo@example.com"}'
```

Passerelle HTTPS auto-signée :

```bash
AEGIS_BASE_URL=https://127.0.0.1:8443 AEGIS_TLS_SKIP_VERIFY=1 npm start
```

Voir [`examples/nodejs/README.md`](https://github.com/zokastech/aegis/blob/main/examples/nodejs/README.md).

---

## Moteur embarqué Node (`sdk-nodejs/examples/`)

Pour utiliser l’**addon Rust N-API** depuis Node **sans** HTTP, construire le paquet et s’appuyer sur les exemples TypeScript :

- **`express-middleware.ts`**
- **`fastify-plugin.ts`**
- **`next-api-route.ts`**

Le README du paquet `sdk-nodejs/` dans le dépôt décrit le build et les chemins.

---

## Notebooks Jupyter (`examples/notebooks/`)

| Notebook | Objectif |
|----------|----------|
| **`aegis_demo.ipynb`** | Appels HTTP : analyse, anonymisation, extrait **Pandas** — bon premier pas interactif. |
| **`train_ner_pii.ipynb`** | **Entraîner un modèle NER PII** : données synthétiques, JSONL sous `datasets/training/`, entraînement Hugging Face, push Hub optionnel, export ONNX (voir `training/README.md`). |
| **`train_ner_hf_public.ipynb`** | Même pipeline avec **jeux Hugging Face publics** (E3-JSI + Ai4Privacy optionnel), fusion, entraînement, Hub, ONNX. |

Lancer Jupyter depuis la racine du dépôt ou le dossier des notebooks :

```bash
pip install jupyter requests pandas
jupyter notebook examples/notebooks/aegis_demo.ipynb
```

L’URL par défaut du notebook est souvent **`https://127.0.0.1:8443`** avec vérification TLS désactivée pour le certificat de dev ; utiliser **`http://127.0.0.1:8080`** avec le compose de développement.

Vue d’ensemble : [`examples/notebooks/README.md`](https://github.com/zokastech/aegis/blob/main/examples/notebooks/README.md).

---

## Documentation associée

- [Démarrage](getting-started.md) — lancer la passerelle et premiers `curl`.
- [Référence API](api-reference.md) — surface REST complète.
- [Anonymisation](anonymization.md) — opérateurs `config_json`.
- [Intégration LLM](llm-integration.md) — usages production autour des modèles et du PII.
