# Tests d’intégration

**AEGIS by [zokastech.fr](https://zokastech.fr)**

Ce répertoire accueille les tests transverses (Rust `tests/`, Go, charge API, etc.) décrits dans le cahier de prompts.

## Où sont les tests par composant

| Zone | Emplacement |
|------|----------------|
| Rust (workspace) | `crates/*/src/**` (`#[cfg(test)]`), `crates/*/tests/*.rs` ; `cargo test --workspace` |
| CLI | `crates/aegis-cli/src/main.rs` (tests unitaires) |
| Go gateway | `aegis-gateway/**/*_test.go` (policy, REST/gRPC d’intégration, apikey, metrics, bridge, …) |
| Go policy (module autonome) | `aegis-policy/policy/*_test.go` |
| Python NER / données | `training/tests/` (`pytest`) |
| Dashboard / front | pas de suite auto dans ce dépôt (lint : `npm run lint`) |
