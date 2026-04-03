# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Démarrage

## Prérequis

- **Docker** (recommandé pour la stack complète) *ou*
- Chaîne **Rust** (`stable`) pour la CLI et le moteur *ou*
- **Python 3.10+** avec le SDK Python (build via `maturin` depuis `sdk-python/` — voir l’état du dépôt).

## Installation avec Docker (recommandé)

1. Cloner le dépôt et copier le modèle d’environnement :

```bash
git clone https://github.com/zokastech/aegis.git
cd aegis
cp .env.example .env
# Éditer .env : mots de passe forts et NER_ONNX_URL optionnel
```

2. Démarrer la stack (passerelle, cœur, Postgres, Redis, dashboard optionnel) :

```bash
docker compose up -d --build
```

3. La passerelle écoute par défaut sur **`8443` → conteneur `8080`** (`GATEWAY_HTTPS_PORT` dans `.env`). Utiliser HTTPS en production ; en local, les images peuvent être en HTTP derrière une terminaison TLS.

## CLI Rust depuis les sources

```bash
git clone https://github.com/zokastech/aegis.git
cd aegis
cargo build --release -p aegis-cli
./target/release/aegis --help
```

Des binaires précompilés peuvent être publiés sur [GitHub Releases](https://github.com/zokastech/aegis/releases).

## SDK Python (sources)

Le crate `sdk-python/` se construit avec **maturin** une fois l’extension PyO3 branchée :

```bash
cd sdk-python
pip install "maturin>=1.4,<2"
maturin develop --release
```

Tant que la liaison n’est pas complète, préférer l’**API HTTP** ou la **CLI**.

---

## Quickstart en cinq minutes

### 1) Santé

```bash
curl -sS "http://localhost:8080/v1/health"   # adapter hôte/port à votre passerelle
```

Si la passerelle a **RBAC / clés API**, ajouter : `-H "X-API-Key: VOTRE_CLE"`.

### 2) Analyser du texte

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze" \
  -H "Content-Type: application/json" \
  -d '{"text":"Contact: alice@example.com ou +33 6 12 34 56 78"}'
```

### 3) Lister les recognizers

```bash
curl -sS "http://localhost:8080/v1/recognizers"
```

### 4) Anonymiser (exemple)

```bash
curl -sS -X POST "http://localhost:8080/v1/anonymize" \
  -H "Content-Type: application/json" \
  -d '{"text":"Email: bob@company.test","config_json":"{}"}'
```

Utiliser un `config_json` réel aligné sur [Anonymisation](anonymization.md) une fois les opérateurs définis par type d’entité.

### 5) OpenAPI

```bash
curl -sS "http://localhost:8080/v1/openapi.yaml" | head
```

---

## Étapes suivantes

- [Exemples](examples.md) — Python, Node.js, notebooks et échantillons `sdk-nodejs` expliqués
- [Architecture](architecture.md) — pipeline à 3 niveaux
- [Configuration](configuration.md) — `aegis-config.yaml`
- [Référence API](api-reference.md) — toutes les routes REST
- [Déploiement](deployment.md) — Compose, Kubernetes, pointeurs cloud
