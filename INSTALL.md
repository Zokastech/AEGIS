# Installation d’AEGIS v0.2.0

**AEGIS by [zokastech.fr](https://zokastech.fr)** — détection et anonymisation de données personnelles (RGPD).

Depuis la v0.2.0, AEGIS est **sécurisé par défaut** : l’authentification est active, TLS est activé et
la passerelle refuse de démarrer avec une configuration dangereuse. Ce guide couvre :

1. [Prérequis](#1-prérequis)
2. [Docker Compose (recommandé)](#2-installation-docker-compose-recommandée)
3. [Kubernetes / Helm](#3-kubernetes--helm)
4. [CLI `aegis`](#4-cli-aegis)
5. [SDK Python, Node.js, Java, Rust](#5-sdk)
6. [Proxy LLM](#6-proxy-llm)
7. [Développement local (sans auth)](#7-développement-local)
8. [Durcissement production — checklist](#8-checklist-production)
9. [Dépannage](#9-dépannage)

---

## 1. Prérequis

| Usage | Outils |
|---|---|
| Stack complète | Docker 24+ et Docker Compose v2 |
| Kubernetes | Kubernetes ≥ 1.25, Helm 3.12+ |
| CLI depuis les sources | Rust stable ≥ 1.85 |
| Passerelle depuis les sources | Go 1.27.3 (+ Rust pour le moteur FFI) |
| SDK | Python ≥ 3.9 · Node.js ≥ 18 (24 recommandé) · Java 17 |

Générez dès maintenant deux secrets forts :

```bash
openssl rand -hex 32      # AEGIS_API_KEY_PEPPER (pepper des clés API)
openssl rand -base64 24   # POSTGRES_PASSWORD
```

> Le pepper protège les empreintes des clés API. Conservez-le dans un gestionnaire de secrets :
> le perdre invalide toutes les clés, le divulguer affaiblit leur stockage.

---

## 2. Installation Docker Compose (recommandée)

### 2.1 Configuration

```bash
git clone https://github.com/Zokastech/AEGIS.git && cd AEGIS
git checkout v0.2.0
cp .env.example .env
chmod 600 .env
```

Éditez `.env` et renseignez au minimum :

```dotenv
POSTGRES_PASSWORD=<mot de passe fort>
AEGIS_API_KEY_PEPPER=<sortie de openssl rand -hex 32>
```

`docker compose` refuse de démarrer si l’une de ces deux variables est vide.

### 2.2 Démarrage

```bash
docker compose up -d --build
docker compose ps
```

| Service | Port hôte (par défaut) | Variable |
|---|---|---|
| Passerelle HTTPS (REST) | `8443` | `GATEWAY_HTTPS_PORT` |
| Dashboard | `3000` | `DASHBOARD_PORT` |
| gRPC | non exposé (réseau interne) | — |

Au premier démarrage la passerelle génère un certificat auto-signé dans le volume `aegis_audit`
(`/var/lib/aegis/tls/`). Pour la production, montez votre certificat (voir §8).

### 2.3 Créer les clés API

Les clés sont stockées **hashées** dans `/var/lib/aegis/api_keys.yaml` (volume persistant).
Le secret en clair est affiché **une seule fois** :

```bash
# Clé d’intégration applicative (analyse + anonymisation)
docker compose exec aegis-gateway aegis-keygen \
  -file /var/lib/aegis/api_keys.yaml -role operator -id app-backend

# Deux clés admin distinctes (la dé-anonymisation exige une double approbation)
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role admin -id admin-alice
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role admin -id admin-bob

# Lister / révoquer / faire tourner
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -list
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -revoke app-backend
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -rotate app-backend -id app-backend-2

# Les clés sont lues au démarrage : redémarrer la passerelle après une modification
docker compose restart aegis-gateway
```

Rôles : `viewer` (analyse), `operator` (+ anonymisation), `auditor` (audit, métriques, DPIA),
`admin` (tout, dont config et dé-anonymisation avec 2 clés).

### 2.4 Vérification

```bash
KEY='aegis_…'   # secret affiché par aegis-keygen
curl -sk https://localhost:8443/livez -o /dev/null -w '%{http_code}\n'          # 204
curl -sk https://localhost:8443/v1/analyze -d '{"text":"x@y.fr"}' \
     -H 'Content-Type: application/json' -o /dev/null -w '%{http_code}\n'       # 401 (sans clé)
curl -sk https://localhost:8443/v1/analyze \
     -H "X-API-Key: $KEY" -H 'Content-Type: application/json' \
     -d '{"text":"Contactez jean.dupont@example.fr"}'                           # 200 + entités
```

Dashboard : <http://localhost:3000> → saisir la clé API sur l’écran de connexion.

### 2.5 Modèle NER (optionnel, niveau L3)

Voir les variables `ZOKA_SENTINEL_*` / `NER_ONNX_URL` dans `.env.example` : le service `onnx-init`
télécharge le modèle dans le volume `aegis_models` avant le démarrage du moteur.

### 2.6 Mise à jour depuis 0.1.0

```bash
git fetch --tags && git checkout v0.2.0
# 1) ajouter POSTGRES_PASSWORD et AEGIS_API_KEY_PEPPER dans .env
# 2) reconstruire et redémarrer
docker compose up -d --build
# 3) créer les clés API (§2.3) — l’image 0.1.0 tournait SANS authentification
```

Lisez la table de migration de [RELEASE_NOTES.md](RELEASE_NOTES.md#migration-depuis-010).

---

## 3. Kubernetes / Helm

Le chart publie la passerelle `ghcr.io/zokastech/aegis-gateway` (tag = `appVersion`, `0.2.0`).
Les images `aegis-core` et `aegis-dashboard` ne sont pas publiées par la release : construisez-les
(`docker/Dockerfile.core`, `docker/Dockerfile.dashboard`) et poussez-les dans votre registre, ou désactivez-les.

```bash
kubectl create namespace aegis
kubectl label namespace aegis pod-security.kubernetes.io/enforce=restricted

# 1) Pepper
kubectl -n aegis create secret generic aegis-pepper \
  --from-literal=AEGIS_API_KEY_PEPPER="$(openssl rand -hex 32)"

# 2) Fichier de clés généré hors cluster (même pepper !)
AEGIS_API_KEY_PEPPER=… go run ./aegis-gateway/cmd/aegis-keygen -file api_keys.yaml -role admin -id admin-alice
kubectl -n aegis create secret generic aegis-api-keys --from-file=api_keys.yaml
```

`values-prod.yaml` :

```yaml
gateway:
  # Le pepper est fourni via --set-string (voir commande helm ci-dessous) → Secret <release>-gateway-env.
  extraVolumes:
    - name: api-keys
      secret:
        secretName: aegis-api-keys
        defaultMode: 0400
  extraVolumeMounts:
    - name: api-keys
      mountPath: /var/lib/aegis/api_keys.yaml
      subPath: api_keys.yaml
      readOnly: true
core:
  image:
    repository: registry.example.org/aegis-core
dashboard:
  image:
    repository: registry.example.org/aegis-dashboard
```

```bash
helm upgrade --install aegis deploy/helm/aegis -n aegis -f values-prod.yaml \
  --set-string gateway.secretEnv.AEGIS_API_KEY_PEPPER="$(kubectl -n aegis get secret aegis-pepper -o jsonpath='{.data.AEGIS_API_KEY_PEPPER}' | base64 -d)"
kubectl -n aegis rollout status deploy/aegis-gateway
```

Recommandé : activer `networkpolicy`, l’ingress TLS (cert-manager) et monter un certificat serveur
signé par votre PKI ; limiter l’accès au port gRPC aux seuls clients autorisés.

---

## 4. CLI `aegis`

### Binaire signé (Linux amd64 / arm64)

```bash
V=v0.2.0; ARCH=linux-amd64
curl -LO https://github.com/Zokastech/AEGIS/releases/download/$V/aegis-cli-$V-$ARCH.tar.gz
curl -LO https://github.com/Zokastech/AEGIS/releases/download/$V/aegis-cli-$V-$ARCH.tar.gz.cosign.bundle
export COSIGN_CERT_IDENTITY_REGEX='https://github.com/Zokastech/AEGIS/.*'
export COSIGN_CERT_OIDC_ISSUER_REGEX='https://token.actions.githubusercontent.com'
./scripts/cosign-verify-aegis-cli.sh aegis-cli-$V-$ARCH.tar.gz aegis-cli-$V-$ARCH.tar.gz.cosign.bundle
tar -xzf aegis-cli-$V-$ARCH.tar.gz && sudo install -m 0755 aegis /usr/local/bin/aegis
aegis --version   # aegis 0.2.0
```

### Depuis les sources

```bash
cargo install --path crates/aegis-cli --locked
# ou : cargo install aegis-cli --version 0.2.0 --locked
```

### Utilisation

```bash
aegis analyze "Appelez Marie au +33 6 12 34 56 78"
aegis scan ./exports --format sarif > aegis.sarif
aegis anonymize clients.csv --output clients.anon.csv
```

---

## 5. SDK

### Python (`aegis-pii`)

```bash
pip install aegis-pii==0.2.0
```

```python
from aegis import AegisEngine
with AegisEngine() as engine:
    print(engine.analyze("mail : jean.dupont@example.fr"))
    print(engine.anonymize("mail : jean.dupont@example.fr",
          operators={"EMAIL": {"operator_type": "redact", "params": {}}}).text)
```

Depuis les sources : `cd sdk-python && pip install maturin && maturin develop --release`.

### Node.js (`@aegis-pii/core`)

```bash
npm install @aegis-pii/core@0.2.0
```

```js
import { AegisEngine } from '@aegis-pii/core'
const engine = new AegisEngine()
console.log(await engine.analyze('mail : jean.dupont@example.fr', { scoreThreshold: 0.5 }))
```

Depuis les sources : `cd sdk-nodejs && npm ci && npm run build && npm test` (Rust requis).

### Java (JNI)

```bash
cd sdk-java/aegis-jni-native && cargo build --release
cd .. && mvn -B install -Daegis.native.dir=$PWD/aegis-jni-native/target/release
```

Dépendance Maven : `fr.zokastech.aegis:aegis-sdk:0.2.0` (+ `aegis-spring-boot-starter`).

### Rust

```toml
[dependencies]
aegis-core = "0.2.0"
aegis-regex = "0.2.0"
aegis-anonymize = "0.2.0"
```

---

## 6. Proxy LLM

Proxy transparent devant OpenAI / Anthropic qui analyse, bloque ou anonymise les prompts.

```bash
cp aegis-llm-proxy/config.example.yaml llm-proxy.yaml
docker build -t aegis-llm-proxy:0.2.0 aegis-llm-proxy
# Clé API AEGIS dédiée au proxy (rôle operator : analyse + anonymisation)
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role operator -id llm-proxy
docker compose restart aegis-gateway
docker compose cp aegis-gateway:/var/lib/aegis/tls/server.crt ./aegis-gateway-ca.pem

docker run --rm -p 8088:8080 --network aegis-backend \
  -v $PWD/llm-proxy.yaml:/etc/aegis/llm-proxy.yaml:ro \
  -v $PWD/aegis-gateway-ca.pem:/etc/aegis/gateway-ca.pem:ro \
  -e AEGIS_UPSTREAM_URL=https://api.openai.com -e AEGIS_LLM_MODE=anonymize \
  -e AEGIS_ENGINE_URL=https://aegis-gateway:8080 -e AEGIS_ENGINE_API_KEY='aegis_…' \
  aegis-llm-proxy:0.2.0
```

Dans `llm-proxy.yaml`, définissez `engine.ca_file: /etc/aegis/gateway-ca.pem` (la vérification TLS
n’est jamais désactivée).

| Option | Défaut | Effet |
|---|---|---|
| `mode` | `anonymize` | `transparent` · `anonymize` · `block` · `alert` (toute autre valeur est refusée) |
| `fail_open` | `false` | `block` : moteur indisponible ⇒ 503. `true` ⇒ requête transmise sans analyse |
| `max_body_bytes` | `10485760` | Taille max. requête et réponse amont |
| `engine.api_key_env` | `AEGIS_ENGINE_API_KEY` | Variable contenant la clé API envoyée à la passerelle |
| `engine.ca_file` | — | CA (PEM) à approuver pour le certificat de la passerelle |

En mode `anonymize`, les requêtes `"stream": true` sont anonymisées mais les pseudonymes ne sont pas
restaurés dans la réponse diffusée. Le proxy n’authentifie pas ses clients : exposez-le uniquement
sur un réseau interne.

---

## 7. Développement local

```bash
make dev        # ou : docker compose -f docker-compose.dev.yml up --build
```

Le compose de développement force `AEGIS_INSECURE_HTTP=1`, `AEGIS_SECURITY_PRODUCTION=false` et
`AEGIS_SECURITY_DEVELOPMENT_DISABLE_AUTH=true`. Depuis 0.2.0, `disable_auth` est **refusé** si
`production` vaut `true` : la passerelle s’arrête au démarrage.

Tests :

```bash
cargo test --workspace                                   # Rust
(cd aegis-gateway && go test -race ./...)                # passerelle
(cd aegis-llm-proxy && GOWORK=off go test -race ./...)   # proxy LLM
(cd sdk-python && maturin develop && pytest -q tests)    # SDK Python
(cd sdk-nodejs && npm ci && npm run build && npm test)   # SDK Node.js
(cd aegis-dashboard && npm ci && npm test && npm run lint)
```

---

## 8. Checklist production

- [ ] `production: true`, `development.disable_auth: false` (défaut de l’image).
- [ ] Certificat serveur de votre PKI monté sur `/var/lib/aegis/tls/server.crt|key` (ou `security.tls.cert_file/key_file`).
- [ ] mTLS pour les services internes : `mtls.require: true` + `tls.client_ca_file` + `rbac.mtls_cn_roles`.
- [ ] `AEGIS_API_KEY_PEPPER` dans un gestionnaire de secrets ; **jamais** dans `api_keys.yaml` ni dans git.
- [ ] Au moins **deux** clés admin distinctes, détenues par deux personnes (double approbation).
- [ ] Rotation périodique des clés : `aegis-keygen -rotate`.
- [ ] `security.trusted_proxies` = CIDR exacts de vos reverse-proxies (sinon laisser vide).
- [ ] Port gRPC `9090` non exposé publiquement (NetworkPolicy / pare-feu).
- [ ] JWT/OIDC : `hmac_secret` ≥ 32 caractères, `issuer` et `audience` renseignés, jetons avec `exp`.
- [ ] `alerts.webhook_url` vers votre SIEM / astreinte pour chaque dé-anonymisation.
- [ ] Journal d’audit sur volume persistant, export SIEM (`/v1/audit/export?format=cef|leef`), intégrité vérifiable.
- [ ] Images épinglées par digest et vérifiées avec `cosign verify`.

---

## 9. Dépannage

| Symptôme | Cause / solution |
|---|---|
| `required variable POSTGRES_PASSWORD is missing a value` | Renseigner `.env` (§2.1). |
| `development.disable_auth=true est interdit quand production=true` | Ajouter `AEGIS_SECURITY_PRODUCTION=false` (dev uniquement). |
| `mtls.require=true impose tls.client_ca_file` | Fournir la CA client ou désactiver mTLS. |
| `jwt.hmac_secret doit contenir au moins 32 caractères` | Allonger le secret. |
| Toutes les requêtes en `401` | Pas de clé API créée, mauvais pepper, ou passerelle non redémarrée après `aegis-keygen`. |
| gRPC `Unauthenticated` | Ajouter la métadonnée `x-api-key` (ou `authorization: Bearer …`). |
| `PUT /v1/config` → 400 « ne peut pas être modifiée à chaud » | Clé protégée (`security`, listeners, clés admin) : modifier le fichier et redémarrer. |
| Rate limiting global derrière un reverse-proxy | Déclarer le proxy dans `security.trusted_proxies`. |
| Proxy LLM : `503 analyse PII indisponible` | Moteur AEGIS injoignable en mode `block` (comportement voulu) ; vérifier `AEGIS_ENGINE_URL`. |
| `aegis-keygen: apikey: pepper requis` | Exporter `AEGIS_API_KEY_PEPPER` (identique à celui de la passerelle). |

Support sécurité : voir [SECURITY.md](SECURITY.md).
