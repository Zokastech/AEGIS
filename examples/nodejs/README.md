# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Node.js examples

Les scripts appellent la **passerelle HTTP** (`/v1/anonymize`, etc.) : c’est le **même moteur Rust** que la gateway embarque (pas de mock local).

### Prérequis

- Node.js 18+
- Passerelle joignable : `AEGIS_BASE_URL`
  - `docker compose up` (racine) : souvent **`https://127.0.0.1:8443`** (TLS auto-signé).
  - Profil dev HTTP : **`http://127.0.0.1:8080`** (voir `docker-compose.dev.yml` / variables du projet).
- Si la gateway exige une clé : `AEGIS_API_KEY` (en-tête `X-API-Key` par défaut, surcharge possible avec `AEGIS_API_KEY_HEADER`).

### Installation

```bash
cd examples/nodejs
npm install
```

### Express → gateway

```bash
AEGIS_BASE_URL=http://127.0.0.1:8080 npm start
# ou HTTPS (cert auto-signé) :
# AEGIS_BASE_URL=https://127.0.0.1:8443 AEGIS_TLS_SKIP_VERIFY=1 npm start
# (équivalent : NODE_TLS_REJECT_UNAUTHORIZED=0)
# avec clé API :
# AEGIS_API_KEY=your-key AEGIS_BASE_URL=... npm start
```

Autre terminal :

```bash
curl -s -X POST http://127.0.0.1:3000/api/note -H "Content-Type: application/json" \
  -d '{"text":"Mail: demo@example.com"}'
```

La réponse JSON contient `text_sanitized` produit par le moteur. `GET /health` vérifie la joignabilité de la gateway.

### Moteur embarqué en Node (sans HTTP)

Pour lier directement le **addon NAPI** Rust dans un service Node, utilisez le package `sdk-nodejs` (`@aegis-pii/core`) : `npm run build` dans `sdk-nodejs/`, puis les exemples sous `sdk-nodejs/examples/`.
