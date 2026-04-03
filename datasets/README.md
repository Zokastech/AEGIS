# AEGIS — zokastech.fr — Apache 2.0 / MIT

Jeux de données et benchmark **PII** pour comparer **AEGIS** (CLI / gateway) et **Microsoft Presidio**.

## Fichiers

| Élément | Description |
|---------|-------------|
| [`training/`](training/) | Seeds JSONL + doc pour **fine-tuning NER** (`training/train_ner.py`, notebook `examples/notebooks/train_ner_pii.ipynb`). |
| `generate_dataset.py` | Génère 10k textes synthétiques multilingues annotés (JSONL). |
| `generated/` | Sortie du générateur (créé au premier run). |
| `false_positives/cases.jsonl` | Cas attendus **sans** PII (pièges pour détecteurs naïfs). |
| `recall_test/cases.jsonl` | PII difficiles (URLs, Base64, stack traces, etc.). |
| `benchmark_vs_presidio.py` | Métriques + rapport HTML. |

## Commandes

```bash
# Générer le corpus principal (10 000 lignes)
python generate_dataset.py --output generated/synthetic_pii.jsonl --n 10000 --seed 42

# Modèles Spacy utiles pour Presidio (optionnel, par langue)
python -m spacy download en_core_web_sm
python -m spacy download fr_core_news_sm

# Benchmark (depuis la racine du dépôt)
make benchmark
```

Le benchmark invoque **`aegis scan`** sur un répertoire de fichiers `.txt` (binaire `target/release/aegis` après `cargo build -p aegis-cli --release`). Variable **`AEGIS_CLI`** pour un chemin explicite. Limite d’échantillon : `make -C datasets benchmark BENCH_LIMIT=2000`. Presidio est optionnel (modèles Spacy) ; en cas d’échec d’import, le rapport reste **AEGIS seul**.

## Licence

Apache-2.0 et MIT.
