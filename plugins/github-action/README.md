# AEGIS — zokastech.fr — Apache 2.0 / MIT

GitHub Action **aegis-scan** : analyse statique des fuites PII dans un dépôt avec le CLI [AEGIS](https://zokastech.fr).

## Utilisation (`zokastech/aegis`)

L’action vit dans ce monorepo sous `plugins/github-action/`. Référence typique :

```yaml
- uses: zokastech/aegis/plugins/github-action@v1
  with:
    languages: 'fr,en,de'
    score_threshold: '0.7'
    fail_on_pii: true
```

Si le dépôt `zokastech/aegis-scan` est publié seul sur le Marketplace :

```yaml
- uses: zokastech/aegis-scan@v1
  with:
    languages: 'fr,en,de'
    score_threshold: '0.7'
    fail_on_pii: true
```

### Pull request (fichiers modifiés)

Utilisez un checkout profond pour que les SHAs de base / tête soient résolus :

```yaml
on:
  pull_request:

jobs:
  pii:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: zokastech/aegis/plugins/github-action@v1
        with:
          languages: 'fr,en'
          score_threshold: '0.6'
          fail_on_pii: true
          comment_pr: true
          github_token: ${{ secrets.GITHUB_TOKEN }}
```

### Scanner tout le dépôt (ou des dossiers)

```yaml
- uses: zokastech/aegis/plugins/github-action@v1
  with:
    scan_mode: full_repo
    paths: |
      src
      docs
    exclude_patterns: |
      **/testdata/**
      .env.example
    fail_on_pii: false
```

### Entrées

| Entrée | Description |
|--------|-------------|
| `languages` | Langues regex du moteur, séparées par des virgules (défaut `fr,en`) |
| `score_threshold` | Seuil de score (0.0–1.0) |
| `fail_on_pii` | `true` pour faire échouer le job si des PII sont trouvées |
| `config_path` | Fichier YAML/JSON de config moteur (optionnel) |
| `exclude_patterns` | Motifs glob (lignes ou virgules), bash |
| `paths` | Racines à parcourir (lignes), défaut `.` |
| `scan_mode` | `auto`, `pr_files`, `full_repo` |
| `cli_version` | Tag de release (`v0.1.0`) ou `latest` |
| `release_repo` | `owner/repo` des releases binaires (défaut `zokastech/aegis`) |
| `comment_pr` | Commentaire Markdown sur la PR si findings |
| `github_token` | Token (défaut : `GITHUB_TOKEN` du workflow) |

### Sorties

- `pii_found` — chaîne `true` / `false`
- `entity_count` — nombre total d’entités
- `report_path` — `aegis-scan-report.md` à la racine du workspace

### Binaire CLI

L’action tente de télécharger un asset de release nommé de façon compatible avec la plate-forme (`aegis-cli-<os>-<arch>.tar.gz` ou `.zip`). Sinon, elle compile `crates/aegis-cli` avec **cargo** si le checkout contient le dépôt AEGIS.

### Extensions scannées

`.py`, `.js`, `.ts`, `.tsx`, `.jsx`, `.mjs`, `.cjs`, `.java`, `.go`, `.rs`, `.json`, `.yaml`, `.yml`, `.md`, `.txt`, `.csv`, `.sql`, `.env`, `.log`

### Tests locaux

Voir `test/fixtures/` (PII **synthétiques** uniquement) et le workflow `.github/workflows/aegis-scan-action-test.yml`.
