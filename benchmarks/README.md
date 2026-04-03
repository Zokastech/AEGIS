# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Benchmarks performance

Crate workspace **`aegis-benchmarks`** : micro-benchmarks (Criterion), macro (débit, concurrence, threads), comparaison FFI, sérialisation JSON.

### Commandes

```bash
# Tout (depuis la racine du dépôt) : Criterion + RSS + rapport HTML + hyperfine + Presidio (optionnel)
make bench

# Criterion seul (plus rapide)
cargo bench -p aegis-benchmarks

# Banc individuel
cargo bench -p aegis-benchmarks --bench micro_pipeline

# Mémoire (Unix `getrusage`, après N analyses)
cargo run -p aegis-benchmarks --release --bin aegis-memory-rusage -- 2000
```

### Variables d’environnement

| Variable | Effet |
|----------|--------|
| `SKIP_PRESIDIO=1` | Ne lance pas le script Python Presidio (plus rapide). |
| `SKIP_PRESIDIO_ENGINE=1` | Passe `--skip-presidio` au benchmark datasets (AEGIS seul). |
| `BENCH_PRESIDIO_LIMIT` | Limite de lignes JSONL (défaut `300`). |
| `AEGIS_BIN` | Chemin CLI pour `hyperfine_cli.sh`. |

### Sorties

- **Criterion** : `target/criterion/`
- **Rapport HTML** : `benchmarks/reports/performance_report.html` (copie : `docs/performance/report.html`)
- **hyperfine** : `benchmarks/reports/hyperfine_aegis.json` (si `hyperfine` installé)
- **Presidio vs AEGIS** : `datasets/reports/benchmark_report.html` (via `datasets/benchmark_vs_presidio.py`)

### Dépendances Python (rapport)

```bash
pip install -r benchmarks/scripts/requirements-bench.txt
```
