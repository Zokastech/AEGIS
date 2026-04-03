# Rapports de performance AEGIS

Ce répertoire accueille la **documentation** des mesures de performance (zokastech.fr).

Après `make bench` à la racine du dépôt :

- [`report.html`](report.html) — synthèse Criterion, lien vers le benchmark qualité/latence Presidio (`datasets/reports/`), graphique pipeline si `matplotlib` est disponible.
- `pipeline_latency.png` — copie du graphique latence vs taille de texte (généré avec les derniers résultats).

Pour régénérer : voir [`benchmarks/README.md`](../../benchmarks/README.md).
