# AEGIS — zokastech.fr — Apache 2.0 / MIT

# CI/CD

## GitHub Actions

### CI principale

Le workflow **`.github/workflows/ci.yml`** construit et teste le workspace Rust, les modules Go et contrôles associés. Déclenché sur push / PR vers la branche par défaut.

### Sécurité

**`.github/workflows/security.yml`** exécute des jobs orientés dépendances et chaîne d’approvisionnement (évolution possible).

### Release

**`.github/workflows/release.yml`** (tags `v*`) publie binaires CLI, images conteneur, artefacts SBOM et provenance SLSA. Voir `SECURITY.md` pour la vérification cosign.

### aegis-scan (PII dans les dépôts)

L’action réutilisable se trouve sous **`plugins/github-action/`**.

```yaml
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: zokastech/aegis/plugins/github-action@main
        with:
          languages: 'fr,en,de'
          score_threshold: '0.7'
          fail_on_pii: true
```

Exemples complets : [`plugins/github-action/README.md`](https://github.com/zokastech/aegis/blob/main/plugins/github-action/README.md).

### Site de documentation

Le workflow **`.github/workflows/docs.yml`** construit MkDocs et déploie sur **GitHub Pages**.

---

## GitLab CI

Inclure le modèle depuis **`plugins/gitlab-ci/.gitlab-ci-aegis.yml`** :

```yaml
include:
  - local: '/plugins/gitlab-ci/.gitlab-ci-aegis.yml'
stages:
  - pii-scan
```

Des variables comme `AEGIS_IMAGE`, `AEGIS_FAIL_ON_PII`, `AEGIS_LANGUAGE` et `AEGIS_SCORE_THRESHOLD` personnalisent le comportement. Le job émet un rapport Code Quality GitLab via `jq` + `aegis-to-gitlab-quality.jq`.

---

## Pre-commit

Les hooks sous **`plugins/pre-commit/`** peuvent lancer des scans locaux avant commit (voir `.pre-commit-hooks.yaml` dans ce répertoire).

---

## Construction d’images en CI

Utiliser les Dockerfiles dans **`docker/`** (`Dockerfile.gateway`, `Dockerfile.core`, …) ou les pipelines de build de votre registre. Épingler les digests et activer la génération SBOM si possible (`scripts/generate-sbom.sh`).
