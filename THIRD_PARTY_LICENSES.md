# Licences des dépendances et outils tiers — AEGIS

Ce document complète le fichier [LICENSE](LICENSE) (code **AEGIS** : **Apache-2.0 OR MIT**) et le fichier [NOTICE](NOTICE). Il ne remplace pas les textes de licence des tiers ; en cas de redistribution commerciale ou d’audit juridique, générez un inventaire à jour (SBOM) et validez avec votre conseil.

## 1. Code source AEGIS (ce dépôt)

| Zone | Fichiers / manifestes | Licence déclarée |
|------|------------------------|------------------|
| Workspace Rust | `Cargo.toml`, `crates/*` | Apache-2.0 OR MIT (`license` workspace) |
| Passerelle & policy Go | `aegis-gateway/`, `aegis-policy/` | Alignée sur la racine (voir `LICENSE`) |
| Proxy LLM Go | `aegis-llm-proxy/` | Idem |
| Dashboard | `aegis-dashboard/` | Idem (`package.json` → `license`) |
| Landing | `landing/` | Idem |

## 2. Dépendances Rust (crates.io)

Résolues via `Cargo.lock`. L’écosystème Rust utilise surtout **MIT**, **Apache-2.0**, **BSD-3-Clause**, **ISC**, **Unicode-DFS-2016**, etc.

**Inventaire à jour (recommandé)** :

```bash
cargo install cargo-license cargo-deny  # une fois
cargo license --workspace
# Optionnel : politique de licences
cargo deny check licenses
```

## 3. Dépendances Go

| Module | Manifeste | Exemples de familles de licences* |
|--------|-----------|-----------------------------------|
| Passerelle | `aegis-gateway/go.mod` | MIT (Echo, Zerolog…), Apache-2.0 (gRPC, Prometheus client…), BSD-3-Clause |
| Policy | `aegis-policy/go.mod` | MIT / BSD (transitif via `yaml.v3`) |
| LLM proxy | `aegis-llm-proxy/go.mod` | Selon modules réellement `require` |

\*Indicatif ; les versions exactes sont dans `go.sum`.

**Inventaire automatique (optionnel)** :

```bash
go install github.com/google/go-licenses@latest
cd aegis-gateway && go-licenses report ./... 2>/dev/null
```

## 4. Dépendances JavaScript / TypeScript (npm)

| Projet | Manifeste | Profil typique |
|--------|-----------|----------------|
| Dashboard | `aegis-dashboard/package.json` | React **MIT**, Vite **MIT**, Radix **MIT**, TanStack **MIT**, Recharts **MIT**, Tailwind **MIT**, ESLint **MIT**, Storybook **MIT** |
| Landing | `landing/package.json` | React, Vite — **MIT** |
| Exemple Node | `examples/nodejs/package.json` | Express, axios — **MIT** |

**Inventaire à jour** :

```bash
cd aegis-dashboard && npx --yes license-checker --production --summary
cd landing && npx --yes license-checker --production --summary
```

Si une dépendance **copyleft** (ex. GPL) apparaît, elle doit être traitée explicitement (souvent absente des deps « production » ci-dessus).

## 5. Python (entraînement / évaluation / notebooks)

Principalement sous `training/` (`requirements.txt`, scripts, notebooks). Usage typique : **hors binaire embarqué** de la passerelle ; toutefois les **wheels** installées ont leurs propres licences.

| Famille | Exemples | Licences courantes |
|---------|----------|-------------------|
| ML / données | `torch`, `transformers`, `datasets`, `onnxruntime`, `onnx` | BSD-style, Apache-2.0 |
| Science des données | `numpy`, `pandas`, `scikit-learn`, `scipy` | BSD |
| Optionnel Presidio | `presidio-analyzer`, `spacy` | Apache-2.0, MIT |

**Inventaire** :

```bash
python3 -m venv .venv && . .venv/bin/activate
pip install -r training/requirements.txt
pip-licenses --summary  # pip install pip-licenses
```

## 6. Conteneurs et images tierces (Docker Compose)

| Service / image | Fichiers | Licence indicative |
|-----------------|----------|-------------------|
| Prometheus | `docker-compose.dev.yml`, `docker-compose.monitoring.yml` | **Apache-2.0** |
| Grafana | idem | **Apache-2.0** (AGPL pour certaines *plugins* non inclus par défaut ; image OSS standard) |
| Alertmanager | `docker-compose.monitoring.yml` | **Apache-2.0** |
| PostgreSQL | `docker-compose*.yml` | **PostgreSQL License** (libre, style permissif) |
| Redis | idem | **BSD-3-Clause** |
| Bases d’images build (Debian, Alpine, Rust, Go) | `docker/Dockerfile*` | Selon image amont |

Les binaires **AEGIS** construits dans ces images restent soumis à **Apache-2.0 OR MIT** pour le code du dépôt ; les couches de base (OS, libc) suivent les licences des distributions.

## 7. Outils de build et CI

Compilateurs Rust/Go, Node, `mkdocs`, Terraform/Helm pour `deploy/` : licences propres à chaque outil (généralement MIT/BSD/Apache). Ils ne sont en principe **pas** redistribués dans l’artefact final utilisateur sauf inclusion explicite.

## 8. Synthèse de conformité

1. **Code AEGIS** : double licence au choix du réutilisateur — voir [LICENSE](LICENSE).  
2. **Dépendances** : chaque bibliothèque conserve sa licence ; les obligations (copyright, NOTICE, etc.) s’appliquent selon **votre** mode de distribution (source seule, binaire, image OCI).  
3. **Prometheus / Grafana** : conformez-vous aux notices Apache-2.0 si vous redistribuez leurs binaires ou des dérivés.  
4. **PyTorch / Hugging Face / etc.** : lire les fichiers `LICENSE` / `NOTICE` dans les paquets installés pour l’entraînement ou l’export ONNX.

Pour toute évolution majeure des stacks (nouveau SDK, nouveau service Docker), mettre à jour ce fichier et régénérer les rapports (`cargo license`, `license-checker`, `pip-licenses`, `go-licenses`).
