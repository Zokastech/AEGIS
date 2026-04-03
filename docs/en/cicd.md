# AEGIS — zokastech.fr — Apache 2.0 / MIT

# CI/CD

## GitHub Actions

### Main CI

Workflow **`.github/workflows/ci.yml`** builds and tests the Rust workspace, Go modules, and related checks. Push / PR to the default branch triggers it.

### Security

**`.github/workflows/security.yml`** runs dependency and supply-chain oriented jobs (configuration may evolve).

### Release

**`.github/workflows/release.yml`** (on `v*` tags) publishes CLI binaries, container images, SBOM artifacts, and SLSA provenance. See `SECURITY.md` for cosign verification.

### aegis-scan (PII in repos)

The reusable action lives under **`plugins/github-action/`**.

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

Full examples: [`plugins/github-action/README.md`](https://github.com/zokastech/aegis/blob/main/plugins/github-action/README.md).

### Documentation site

Workflow **`.github/workflows/docs.yml`** builds MkDocs and deploys to **GitHub Pages**.

---

## GitLab CI

Include the template from **`plugins/gitlab-ci/.gitlab-ci-aegis.yml`**:

```yaml
include:
  - local: '/plugins/gitlab-ci/.gitlab-ci-aegis.yml'
stages:
  - pii-scan
```

Variables such as `AEGIS_IMAGE`, `AEGIS_FAIL_ON_PII`, `AEGIS_LANGUAGE`, and `AEGIS_SCORE_THRESHOLD` customize behavior. The job emits a GitLab Code Quality report via `jq` + `aegis-to-gitlab-quality.jq`.

---

## Pre-commit

Hooks under **`plugins/pre-commit/`** can run local scans before commit (see `.pre-commit-hooks.yaml` in that directory).

---

## Building images in CI

Use the Dockerfiles in **`docker/`** (`Dockerfile.gateway`, `Dockerfile.core`, …) or your registry’s build pipelines. Pin digests and enable SBOM generation where possible (`scripts/generate-sbom.sh`).
