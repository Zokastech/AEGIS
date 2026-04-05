# AEGIS — Pipeline fine-tuning NER PII (zokastech.fr)

Pipeline Python pour générer un jeu synthétique multilingue (UE), entraîner **XLM-RoBERTa-base** en classification de tokens, **publier le checkpoint sur le Hugging Face Hub** (optionnel), exporter **ONNX** (dont INT8) et produire un **rapport d’évaluation** (F1 / P / R par type, **F2** comme métrique principale, matrices de confusion, comparaison optionnelle **Presidio**).

Objectif cible sur données synthétiques alignées sur le schéma : viser **F2 > 0,95** au niveau entité ; en production, la généralisation dépendra du domaine et d’un jeu réel annoté.

## Prérequis

- Python 3.10+
- GPU recommandé pour l’entraînement (`--fp16` si CUDA)

## Installation

```bash
cd training
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python -m spacy download en_core_web_sm  # si vous utilisez --with_presidio
```

**Note (monorepo)** : à la racine du dépôt, le dossier `datasets/` n’est pas le paquet Hugging Face. Les scripts sous `training/` passent par `ensure_hf_datasets.py` pour importer le bon module ; en Jupyter, gardez `sys.path` avec `training/` en tête comme dans le notebook.

**CI / image Docker (niveau 3)** : `requirements-ci.txt` couvre les tests légers (sans PyTorch). Image : `docker build -f docker/Dockerfile.training -t aegis-training-ner .` depuis la racine du monorepo. Côté Rust, le crate `crates/aegis-ner-training` documente la chaîne **training → ONNX → `aegis_ner::NerEngine`**.

**Pipeline L3 automatisé (local ou CI)** : après `pip install -r requirements.txt`, depuis `training/` exécuter `bash scripts/run_l3_pipeline.sh` (variables optionnelles `AEGIS_L3_EXAMPLES`, `AEGIS_L3_MAX_STEPS`, `AEGIS_L3_MODEL_NAME`). Sur GitHub : workflow **Helm & NER L3 pipeline** — sur `main`/`master` un smoke CPU produit l’artefact `aegis-ner-l3-onnx.tgz` (ONNX + `tokenizer.json` pour montage PVC `/opt/aegis/models`) ; **workflow_dispatch** permet d’augmenter exemples/steps. Le chart Helm complet est packagé dans le même workflow (artefact `aegis-helm-chart`) et joint aux **Releases** pour les tags `v*`.

## 1. Jeu de données synthétique

Génère **≥ 50 000** exemples (défaut), IOB2, 11 langues UE, plusieurs domaines (email, formulaire, log, ticket, presse, chat), export Hugging Face `datasets` sur disque :

```bash
python dataset_builder.py --num_examples 50000 --output ./data/eu_pii_synthetic
```

**Jeux complémentaires (JSONL)** : placez des fichiers `tokens` + `ner_tags` (labels = `LABELS` dans `dataset_builder.py`) puis :

```bash
python jsonl_to_hf_dataset.py ../datasets/training/ner_custom/samples.jsonl --output ./data/from_jsonl
python merge_hf_datasets.py ./data/eu_pii_synthetic ./data/from_jsonl --output ./data/merged
```

Notebook pas à pas : [`examples/notebooks/train_ner_pii.ipynb`](../examples/notebooks/train_ner_pii.ipynb). Détails des dossiers d’exemple : [`datasets/training/README.md`](../datasets/training/README.md).

**Jeux Hugging Face publics** (téléchargement + conversion IOB2 AEGIS) :

- [E3-JSI/synthetic-multi-pii-ner-v1](https://huggingface.co/datasets/E3-JSI/synthetic-multi-pii-ner-v1) — PII synthétique multilingue (~3k phrases) ; types annotés mappés vers notre schéma (une partie des lignes est ignorée si les spans produisent des étiquettes hors `LABELS`).
- [ai4privacy/pii-masking-300k](https://huggingface.co/datasets/ai4privacy/pii-masking-300k) — grand corpus (usage académique / licence à vérifier pour le commercial) ; utilise `source_text` + `span_labels`.

```bash
python import_hf_external_pii.py e3jsi --output ./data/hf_e3jsi_synthetic
# L’intégralité du train HF : --max_samples 0 (défaut 0 = tout le split)
python import_hf_external_pii.py ai4privacy --output ./data/hf_ai4privacy_train --max_samples 50000   # ou 0 pour ~178k train (très lourd)
python merge_hf_datasets.py ./data/eu_pii_synthetic ./data/hf_e3jsi_synthetic ./data/hf_ai4privacy_train --output ./data/merged_with_hf
```

Notebook pas à pas : [`examples/notebooks/train_ner_hf_public.ipynb`](../examples/notebooks/train_ner_hf_public.ipynb).

## 2. Entraînement

```bash
python train_ner.py --dataset ./data/eu_pii_synthetic --output_dir ./outputs/ner-xlmr-eu-pii --fp16
```

Sur macOS (**surtout M1/M2 16 Go**), l’OOM MPS peut survenir au **backward** (`loss.backward`) ou chez **AdamW** (`exp_avg_sq`). Si le message affiche **« other allocations » ~15–18 Go**, la RAM unifiée est saturée par le reste du système : **`--cpu`** est en général la solution fiable (fermer navigateur / Docker / autres apps aide aussi). Pistes MPS : **`--adafactor`**, `--gradient_checkpointing`, **`--max_seq_length 256`**, batch **1** + `--gradient_accumulation_steps 8`. Le notebook HF public active **Adafactor** sur MPS par défaut (`AEGIS_TRAIN_ADAFACTOR=0` pour AdamW). Variables d’env : `AEGIS_TRAIN_CPU`, `AEGIS_TRAIN_BATCH`, `AEGIS_TRAIN_GRAD_ACCUM`, `AEGIS_MAX_SEQ_LENGTH`, `AEGIS_TRAIN_ADAFACTOR`. **Dernier recours MPS** (avant de lancer Python) : `AEGIS_RELAX_MPS_MEMORY_CAP=1` — le script fixe `PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0` *avant* l’import de torch ; ou `AEGIS_MPS_HIGH_WATERMARK_RATIO=0.5` pour un plafond moins strict. Risque : **swap / gel** si la machine manque de RAM physique.

Checkpoints par époque sous `output_dir` ; meilleur modèle (F1 seqeval) recopié dans `output_dir/best_hf`.

## 3. Export ONNX + tokenizer (Rust `tokenizers`)

```bash
python export_onnx.py --model_dir ./outputs/ner-xlmr-eu-pii/best_hf --out_dir ./exports/onnx_ner
```

- `model.onnx`, `model_optimized.onnx`, `model_int8.onnx`
- `tokenizer_hf/tokenizer.json` chargeable par la crate Rust [**tokenizers**](https://github.com/huggingface/tokenizers)
- `latency_benchmark.txt` : PyTorch vs ONNX vs quantifié

## 5. Évaluation et rapport HTML

```bash
python evaluate.py --dataset ./data/eu_pii_synthetic --model_dir ./outputs/ner-xlmr-eu-pii/best_hf --out_report ./reports/ner_eval.html
```

Avec baseline Presidio (approximative, mappage des types Presidio → schéma AEGIS) :

```bash
python evaluate.py --with_presidio --out_report ./reports/ner_eval_presidio.html
```

## Schéma d’étiquettes

Aligné entre `dataset_builder.py`, `train_ner.py` et `evaluate.py` : `O`, entités `B-*` / `I-*` pour les spans multi-mots (personne, téléphone, IBAN, etc.), et entités principalement sur un token pour `EMAIL`, `DATE`, `PASSPORT`, `LICENSE_PLATE`.

## Licence

En-tête des fichiers : **Apache 2.0 / MIT** (projet AEGIS — zokastech.fr).
