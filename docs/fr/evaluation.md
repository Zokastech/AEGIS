# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Évaluation et benchmarks

## Métriques

Les métriques classiques de détection PII s’appliquent :

- **Précision / rappel / F1** par type d’entité (données étiquetées nécessaires)
- **Latence** p50/p95/p99 par niveau de pipeline (L1 vs L3)
- **Débit** (tokens ou documents par seconde)

Le moteur expose des **compteurs / histogrammes Prometheus** — scraper `/metrics` sur `aegis-gateway`.

## Benchmarks (dépôt)

- Crates **Criterion** Rust sous `benchmarks/` / crate workspace `aegis-benchmarks` (voir `Makefile` du dépôt et `benchmarks/README.md`).
- Des scripts de comparaison optionnels (ex. Presidio) se trouvent dans `datasets/` — lire le README de chaque script avant exécution (services externes possibles).

## Jeux de données synthétiques

`datasets/` contient du matériel **synthétique** ou orienté tests — **pas** de PHI/PII de production. N’utiliser que des données conformes à votre juridiction.

## Rapport HTML de performance

L’exécution des pipelines de benchmark peut copier un rapport vers `docs/performance/report.html` (voir `benchmarks/scripts/generate_report.py`). Ce chemin est **exclu** du build MkDocs mais reste dans le dépôt pour les développeurs.

## Comparaisons

Lors de publications comparatives avec d’autres moteurs :

- Indiquer **version**, **matériel** et **configuration** (notamment `pipeline_level` et chemin du modèle).
- Mentionner les **modes d’échec** (faux négatifs) — essentiel pour les discussions RGPD / équité.
