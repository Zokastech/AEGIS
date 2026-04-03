# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Plateforme Zokastech — configuration et lancement

Cette page décrit comment **configurer et lancer** les éléments présents dans ce dépôt sous **`zokastech/`** (backend Node + Docker Compose), et leur lien avec **AEGIS** et le **site ZokasTech**.

## Documentation sur zokastech.fr

Le même contenu MkDocs que ce site est intégré à l’application web ZokasTech sous **`/aegis/docs`**, avec l’identité visuelle ZokasTech. La **source Markdown** est ce monorepo (`docs/en`, `docs/fr`, …).

Le **site vitrine** (Vue.js) est dans un dépôt séparé ([`zok-zokastech-front`](https://gitlab.zokas.tech/zok-zokastech-front) — voir [`zokastech/README.md`](https://github.com/zokastech/aegis/blob/main/zokastech/README.md) pour installation, `.env`, Firebase, et `npm run dev` / `npm run build`).

---

## API backend (`zokastech/backend`)

Le dossier **`zokastech/backend`** est un service **Node.js** léger (santé, reformulation IA, signature PDF de stage). Ce n’est **pas** la passerelle AEGIS ; pour AEGIS, voir [Démarrage](getting-started.md) et [Déploiement](deployment.md).

### Prérequis

- Node.js (LTS) **ou** Docker / Docker Compose

### Configuration

1. Copier le modèle d’environnement :

   ```bash
   cd zokastech/backend
   cp .env.example .env
   ```

2. Éditer **`.env`**. Variables typiques :

   | Variable | Rôle |
   |----------|------|
   | `PORT` | Port HTTP (défaut `9100`) |
   | `ALLOWED_ORIGIN` | Origine CORS du frontend |
   | `VERIFY_BASE_URL` | URL de base des pages de vérification de signature |
   | `STAGE_PAGE_PASSWORD` | Mot de passe du module PDF stage |
   | `OLLAMA_BASE_URL` / `OLLAMA_MODEL` | LLM local optionnel pour la reformulation |
   | `PG_*` | Connexion PostgreSQL (voir Docker Compose ci-dessous) |
   | `JWT_SECRET` | Secret pour les jetons émis |

   Liste complète : [`.env.example`](https://github.com/zokastech/aegis/blob/main/zokastech/backend/.env.example).

### Lancement (local, sans Docker)

```bash
cd zokastech/backend
npm install
npm run dev
```

L’API écoute sur **`http://localhost:9100`** (ou `PORT`).

### Lancement avec Docker Compose

Depuis **`zokastech/backend`** :

```bash
docker compose up -d --build
```

| Service | URL / port |
|---------|------------|
| API | `http://localhost:9100` |
| PostgreSQL | `localhost:5433` (base `zokastech`, utilisateur `postgres` / mot de passe `postgres` en dev) |

Arrêt :

```bash
docker compose down
```

### Points d’API (résumé)

- `GET /api/health`
- `POST /api/ai/reformulate` — corps JSON : `text`, `fieldLabel`, `tone`, …
- `POST /api/stage/fill-sign` — multipart PDF + payload formulaire
- `POST /api/stage/verify` — multipart PDF signé

Détails : [`zokastech/backend/README.md`](https://github.com/zokastech/aegis/blob/main/zokastech/backend/README.md).

---

## CI/CD (GitLab)

Le fichier **`zokastech/.gitlab-ci.yml`** décrit des pipelines pour la partie Zokastech sous GitLab. Adaptez images, registres et étapes à votre environnement.

---

## Voir aussi

- [Démarrage](getting-started.md) — moteur AEGIS, passerelle, dashboard
- [Déploiement](deployment.md) — Docker Compose / Helm pour AEGIS
- [Fournisseurs cloud (Terraform)](cloud-providers.md) — AWS, GCP, Azure, OVHcloud
