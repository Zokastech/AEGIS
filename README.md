# AEGIS

[![CI](https://img.shields.io/github/actions/workflow/status/zokastech/aegis/ci.yml?branch=main&label=CI&logo=github)](https://github.com/zokastech/aegis/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2B%20MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-workspace-orange?logo=rust)](Cargo.toml)
[![Go](https://img.shields.io/badge/Go-1.22-00ADD8?logo=go)](go.work)
[![zokastech](https://img.shields.io/badge/AEGIS-zokastech.fr-1a56db)](https://zokastech.fr)

**Advanced European Guardian for Information Security** — détection et anonymisation de données personnelles (PII), orientée Europe et conformité, alternative open source à Microsoft Presidio.

**AEGIS by [zokastech.fr](https://zokastech.fr)** — https://zokastech.fr

## Arborescence du monorepo (aperçu)

> Généré avec `tree -I 'target|node_modules|.git' -L 4` (voir dépôt pour le détail à jour).

```
.
├── Cargo.toml                 # Workspace Rust
├── go.work                    # Workspace Go (gateway + policy)
├── aegis-gateway/             # API HTTP (Gin)
├── aegis-policy/              # Moteur de politiques YAML (RGPD, etc.)
├── aegis-dashboard/           # Frontend React + TypeScript + Vite
├── crates/                    # aegis-core, aegis-regex, aegis-ner, …
├── sdk-python/, sdk-nodejs/, sdk-java/
├── docker/, deploy/helm/aegis/
├── docs/, tests/, benchmarks/, datasets/
└── …
```

## Statut des composants

| Composant | Rôle | Statut |
|-----------|------|--------|
| `crates/aegis-core` | Types, traits, moteur d’analyse | MVP |
| `crates/aegis-regex` | Recognizers regex | MVP |
| `crates/aegis-ner` | NER / ONNX (pipeline Niveau 3) | Squelette |
| `crates/aegis-anonymize` | Opérateurs d’anonymisation | MVP |
| `crates/aegis-fpe` | Chiffrement préservant le format (FF3-1) | Squelette |
| `crates/aegis-ffi` | FFI C | Squelette |
| `crates/aegis-cli` | CLI `aegis` | MVP |
| `aegis-gateway` | Passerelle REST Go | MVP |
| `aegis-policy` | Politiques conformité (package Go partagé) | MVP |
| `aegis-dashboard` | UI admin React + TS | Squelette |
| `sdk-python` | SDK Python (PyO3 / maturin) | Squelette |
| `sdk-nodejs` (`@aegis-pii/core`) | SDK Node (NAPI-RS prévu) | Squelette |
| `sdk-java` (`fr.zokastech.aegis:aegis-sdk`) | SDK Java (JNI prévu) | Squelette |
| `deploy/helm/aegis` | Chart Kubernetes | Fonctionnel |
| `docker/` + `docker-compose.dev.yml` | Dev (Rust/Go/Vite/Redis/Postgres) & image prod | Fonctionnel |
| `docs/` | Sécurité, conformité, guides | En cours |
| `tests/`, `benchmarks/`, `datasets/` | Intégration, perfs, données synthétiques | À étendre |

## Why AEGIS over Presidio?

1. **Rust performance** — Core engine in Rust for high-throughput scanning and predictable latency on large texts.
2. **Europe-first** — EU recognizers, IBAN/NIR-style patterns, and GDPR-oriented policy packs by default ([zokastech.fr](https://zokastech.fr)).
3. **Secure by default** — Hardened HTTP gateway (headers, rate limits, optional API keys) instead of an unsecured analyzer-only service.
4. **Integrated dashboard** — `aegis-dashboard` targets DPO/compliance workflows (playground, policies, audit) Presidio does not ship.
5. **Contextual SLM / NER** — Three-level pipeline (regex → context → ONNX NER) for better precision on ambiguous entities.

*(Version française : voir « Pourquoi AEGIS plutôt que Presidio ? » ci-dessous.)*

## Pourquoi AEGIS plutôt que Presidio ?

1. **Performance** : cœur analytique en Rust, adapté au traitement de gros volumes.
2. **Europe d’abord** : recognizers et politiques pensés pour les formats UE (IBAN, identifiants, RGPD).
3. **Sécurité par défaut** : API avec en-têtes OWASP, rate limiting, auth par clé optionnelle, politiques déclaratives.
4. **Dashboard intégré** : interface d’admin pour conformité et essais (projet `aegis-dashboard`).
5. **SLM contextuel** : pipeline sur 3 niveaux (regex → contexte → NER ONNX) pour le rappel et le contexte.

## Démarrage rapide

### Rust (CLI)

```bash
cargo build --release
./target/release/aegis analyze "Contact : alice@example.com ou +33 6 12 34 56 78"
./target/release/aegis scan fichier.txt
```

### Go (passerelle HTTP)

```bash
cd aegis-gateway
export AEGIS_POLICY=policies/gdpr-strict.yaml
# optionnel : export AEGIS_API_KEY=secret
go run .
curl -s localhost:8080/v1/health
curl -s localhost:8080/v1/analyze -H "Content-Type: application/json" -d '{"text":"mail test@example.com"}'
```

Le module **`aegis-policy`** est consommé par la gateway (`go.work` à la racine).

### Dashboard (React)

```bash
cd aegis-dashboard
npm install && npm run dev
```

### Développement (une commande)

```bash
cp .env.example .env   # optionnel
make dev              # ou : just dev  |  docker compose -f docker-compose.dev.yml up --build
```

Sont démarrés : **aegis-core** (cargo-watch), **aegis-gateway** (air), **aegis-dashboard** (Vite), **Redis**, **PostgreSQL**. Voir [.env.example](.env.example).

### Kubernetes (Helm)

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

Détails : [deploy/helm/aegis/README.md](deploy/helm/aegis/README.md).

## Documentation

- [Contribution](CONTRIBUTING.md) · [Code de conduite](CODE_OF_CONDUCT.md)
- [Politique de sécurité](SECURITY.md)
- [Conformité RGPD & mesures techniques](docs/en/security/rgpd-compliance.md)
- [Durcissement](docs/en/security/hardening.md)
- [Threat model (STRIDE)](docs/en/security/threat-model.md)
- [Site MkDocs (GitHub Pages)](https://zokastech.github.io/aegis/) — build local : `pip install -r requirements-docs.txt && mkdocs serve` (voir `mkdocs.yml`)
- Prometheus / Grafana déjà installés : [EN](docs/en/monitoring-prometheus-grafana.md) · [FR](docs/fr/monitoring-prometheus-grafana.md)
- [Exemples & tutoriels](examples/README.md) — Python, Node, Go (CGO), notebook Jupyter
- [Landing page](landing/README.md) — `aegis.zokastech.fr` (Vite + React, `cd landing && npm run build`)
- [Guide de marque](assets/brand-guide.md)

## Licence

- **Code AEGIS** (ce dépôt) : double licence **Apache-2.0** *ou* **MIT** au choix — voir [LICENSE](LICENSE) et [NOTICE](NOTICE).
- **Dépendances & images tierces** (Rust, Go, npm, Python training, Prometheus, Grafana, bases de données, etc.) : licences propres — résumé et commandes d’inventaire dans [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

---

*AEGIS by [zokastech.fr](https://zokastech.fr)*
