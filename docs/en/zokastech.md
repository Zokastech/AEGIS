# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Zokastech platform — configuration & launch

This page describes how to **configure and run** the pieces that ship in this repository under **`zokastech/`** (Node backend + Docker Compose), and how they relate to **AEGIS** and the **ZokasTech** website.

## Documentation on zokastech.fr

The same MkDocs content as this site is embedded in the ZokasTech web app at **`/aegis/docs`**, with ZokasTech branding. The **source of truth** for Markdown is this monorepo (`docs/en`, `docs/fr`, …).

The **public marketing site** (Vue.js) lives in a separate repository ([`zok-zokastech-front`](https://gitlab.zokas.tech/zok-zokastech-front) — see [`zokastech/README.md`](https://github.com/zokastech/aegis/blob/main/zokastech/README.md) in this repo for install, `.env`, Firebase, and `npm run dev` / `npm run build`).

---

## Backend API (`zokastech/backend`)

The **`zokastech/backend`** folder is a small **Node.js** service (health, AI reformulation hook, PDF stage signing). It is **not** the AEGIS gateway; use [Getting started](getting-started.md) and [Deployment](deployment.md) for AEGIS itself.

### Prerequisites

- Node.js (LTS) **or** Docker / Docker Compose

### Configuration

1. Copy the environment template:

   ```bash
   cd zokastech/backend
   cp .env.example .env
   ```

2. Edit **`.env`**. Typical variables:

   | Variable | Purpose |
   |----------|---------|
   | `PORT` | HTTP port (default `9100`) |
   | `ALLOWED_ORIGIN` | CORS origin for the frontend |
   | `VERIFY_BASE_URL` | Base URL for signature verification pages |
   | `STAGE_PAGE_PASSWORD` | Password for stage PDF module |
   | `OLLAMA_BASE_URL` / `OLLAMA_MODEL` | Optional local LLM for reformulation |
   | `PG_*` | PostgreSQL connection (see Docker Compose below) |
   | `JWT_SECRET` | Secret for issued tokens |

   Full list: [`.env.example`](https://github.com/zokastech/aegis/blob/main/zokastech/backend/.env.example).

### Launch (local, without Docker)

```bash
cd zokastech/backend
npm install
npm run dev
```

API listens on **`http://localhost:9100`** (or `PORT`).

### Launch with Docker Compose

From **`zokastech/backend`**:

```bash
docker compose up -d --build
```

| Service | URL / port |
|---------|------------|
| API | `http://localhost:9100` |
| PostgreSQL | `localhost:5433` (db `zokastech`, user `postgres` / password `postgres` in dev) |

Stop:

```bash
docker compose down
```

### API endpoints (summary)

- `GET /api/health`
- `POST /api/ai/reformulate` — JSON body: `text`, `fieldLabel`, `tone`, …
- `POST /api/stage/fill-sign` — multipart PDF + form payload
- `POST /api/stage/verify` — multipart signed PDF

Details: [`zokastech/backend/README.md`](https://github.com/zokastech/aegis/blob/main/zokastech/backend/README.md).

---

## CI/CD (GitLab)

The **`zokastech/.gitlab-ci.yml`** file defines pipelines for the Zokastech subtree when used in GitLab. Adjust image names, registries, and stages to match your environment.

---

## Related

- [Getting started](getting-started.md) — AEGIS engine, gateway, dashboard
- [Deployment](deployment.md) — Docker Compose / Helm for AEGIS
- [Cloud providers (Terraform)](cloud-providers.md) — AWS, GCP, Azure, OVHcloud
