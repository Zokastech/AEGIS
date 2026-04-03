# aegis-pii

**AEGIS — zokastech.fr** — SDK Python (PyO3) pour détecter et anonymiser les données personnelles, aligné sur le moteur Rust du monorepo AEGIS.

## Installation

```bash
pip install aegis-pii
```

Depuis la racine du dépôt (développement) :

```bash
cd sdk-python
pip install maturin
maturin develop --release
```

## Quickstart (3 lignes)

```python
from aegis import AegisEngine

with AegisEngine() as engine:
    print(engine.analyze("Contact: jane@acme.fr"))
```

## Fonctionnalités

- **Analyse** : `analyze`, `analyze_full`, `analyze_batch`
- **Anonymisation** : `anonymize` avec opérateurs par type (`redact`, `mask`, `replace`, …)
- **Configuration** : `AegisEngine(config_path="aegis-config.yaml")` ou JSON moteur
- **Langues** : `AegisEngine(languages=["en", "fr", "de"])`
- **Pandas** (extra) : `pip install 'aegis-pii[pandas]'` puis `engine.analyze_dataframe(df, columns=[...])`
- **LangChain** (extra) : `AegisPIIGuard` dans `aegis.langchain_tool`

## Build wheels (CI / maturin)

```bash
maturin build --release -i python3.11
```

Cibles typiques : `manylinux_2_28`, `macosx` (x86_64 + arm64), `windows`. Voir [maturin](https://www.maturin.rs/distribution.html) pour la publication PyPI.

## Exemples

Le dossier `examples/` contient :

- `quickstart.py`
- `pandas_integration.py`
- `langchain_guard.py`
- `batch_processing.py`
- `custom_recognizer.py`

## Tests

```bash
pip install -e '.[dev]'
pytest tests/ -q
```

Benchmark comparatif Presidio (optionnel) :

```bash
pip install presidio-analyzer pytest-benchmark
pytest tests/test_bench_presidio.py -m presidio --benchmark-only
```

## Licence

Apache-2.0 **OR** MIT — voir le dépôt AEGIS.
