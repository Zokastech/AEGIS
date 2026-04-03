# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Contributing

Thank you for helping improve **AEGIS** ([zokastech.fr](https://zokastech.fr)).

The repository also ships a root [`CONTRIBUTING.md`](https://github.com/zokastech/aegis/blob/main/CONTRIBUTING.md) — keep both aligned when process changes.

## Workflow

1. **Fork** and create a feature branch from `main`.
2. Run **tests** locally (`cargo test`, `go test ./...` in changed modules).
3. Keep commits focused; follow existing **style** (rustfmt, gofmt).
4. Open a **Pull Request** with a clear description, linked issues, and upgrade notes if behavior changes.

## Security issues

Do **not** open a public PR for undisclosed vulnerabilities — email per [`SECURITY.md`](https://github.com/zokastech/aegis/blob/main/SECURITY.md).

## Legal

By contributing, you agree your contributions are licensed under the project’s **Apache 2.0** and **MIT** dual license terms.

## Docs

- Update **MkDocs** pages under `docs/en/` when you change user-visible behavior.
- Mirror substantive changes under `docs/fr/` (or other `docs/<locale>/`) when you maintain translations.
- Run `mkdocs serve` locally after `pip install -r requirements-docs.txt`.

## Recognizers & languages

New regex recognizers should include **unit tests** and declare accurate `supported_languages`.

## Questions

Use **GitHub Discussions** (if enabled) or issues for design questions; keep issues actionable.
