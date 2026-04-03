# Contributing to AEGIS

**AEGIS by [zokastech.fr](https://zokastech.fr)** — https://zokastech.fr

Thank you for helping improve **AEGIS**, the open-source European PII detection and anonymization platform.

## Principles

- Prefer small, focused pull requests with clear motivation.
- Match existing style: Rust `rustfmt` / `clippy`, Go `gofmt`, TypeScript `prettier` where configured.
- **Never commit real PII**; use synthetic or generated test data only.
- Security-sensitive changes: report privately first (see [SECURITY.md](SECURITY.md)).

## Monorepo layout

| Area | Path |
|------|------|
| Rust workspace | `crates/aegis-*`, root `Cargo.toml` |
| Go modules | `aegis-gateway`, `aegis-policy`, root `go.work` |
| Dashboard | `aegis-dashboard` |
| SDKs | `sdk-python`, `sdk-nodejs`, `sdk-java` |
| Containers | `docker/` |
| Helm | `deploy/helm/aegis` |
| Integration tests / benches / data | `tests/`, `benchmarks/`, `datasets/` |

## Development

1. Fork and clone the repository.
2. **Stack complète (Docker)** : `cp .env.example .env` puis `make dev` (ou `just dev`). Voir `docker-compose.dev.yml` à la racine.
3. **Rust:** `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
4. **Go:** `cd aegis-policy && go test ./...` puis `cd ../aegis-gateway && go test ./...` (ou `go.work` à la racine).
5. **Dev Container (VS Code)** : ouvrir le dossier dans un container (Rust + Go + Node + Docker-from-Docker).

## Licenses & third-party code

By contributing, you agree your work is licensed under the project’s terms (**Apache-2.0 OR MIT**, same as [LICENSE](LICENSE)). Prefer **permissive** dependencies (MIT, Apache-2.0, BSD, ISC) compatible with both licenses. For an overview of stacks and how to regenerate attribution lists, see [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md); extend it when you add a new class of tooling (e.g. a CI image, a training stack, or a packaged service).

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
