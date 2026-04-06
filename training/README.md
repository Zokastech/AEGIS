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

**Pipeline L3 automatisé (local ou CI)** : après `pip install -r requirements.txt`, depuis `training/` exécuter `bash scripts/run_l3_pipeline.sh` (variables optionnelles `AEGIS_L3_EXAMPLES`, `AEGIS_L3_MAX_STEPS`, `AEGIS_L3_MODEL_NAME`, `AEGIS_MODEL_PRODUCT_NAME`, `AEGIS_MODEL_PRODUCT_VERSION`). Le modèle commercial est enregistré sous le nom **ZOKA-SENTINEL** (défaut) avec version explicite dans `zoka_sentinel_manifest.json` et dans les métadonnées ONNX (`metadata_props`). Sur GitHub : workflow **Helm & NER L3 pipeline** — sur `main`/`master` un smoke CPU produit l’artefact `aegis-ner-l3-onnx.tgz` (ONNX + `tokenizer.json` + manifest pour montage PVC `/opt/aegis/models`) ; **workflow_dispatch** permet d’augmenter exemples/steps. Le chart Helm complet est packagé dans le même workflow (artefact `aegis-helm-chart`) et joint aux **Releases** pour les tags `v*`.

## 1. Jeu de données synthétique

Génère **≥ 50 000** exemples (défaut), IOB2, 11 langues UE, plusieurs domaines (email, formulaire, log, ticket, presse, chat), export Hugging Face `datasets` sur disque :

```bash
python dataset_builder.py --num_examples 50000 --output ./data/eu_pii_synthetic
```

Les **noms de personnes** dans les exemples synthétiques (emails, formulaires, tickets, etc.) suivent **75 % de noms français** et **25 % de noms européens (hors France) ou maghrébins** (écriture latine), voir `PERSON_NAME_FRENCH_FRACTION` et `_pick_person_name` dans `dataset_builder.py`.

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
# Nom / version produit (défaut : ZOKA-SENTINEL, 1.0.0 — ou variables AEGIS_MODEL_PRODUCT_*)
python export_onnx.py ... --product_name ZOKA-SENTINEL --product_version 1.2.0
```

- `model.onnx`, `model_optimized.onnx`, `model_int8.onnx`
- `zoka_sentinel_manifest.json` : `product_name`, `product_version`, horodatage UTC
- Métadonnées ONNX (`product_name`, `product_version`, `producer`)
- `tokenizer_hf/tokenizer.json` chargeable par la crate Rust [**tokenizers**](https://github.com/huggingface/tokenizers)
- `latency_benchmark.txt` : PyTorch vs ONNX vs quantifié
- Moins de bruit en CI : `export_onnx.py` fixe par défaut `ORT_LOG_SEVERITY_LEVEL=3` avant d’importer ONNX Runtime (`AEGIS_EXPORT_ONNX_QUIET_ORT=0` pour tout journaliser) ; optimisations graphe en **`ORT_ENABLE_EXTENDED`** pour un modèle optimisé plus portable que `ORT_ENABLE_ALL` ; avertissements PyTorch « legacy export » / trace masqués autour de `torch.onnx.export` (export TorchScript volontaire, voir code).

Publication Hugging Face du checkpoint **`best_hf`** avec le même nom / version :

```bash
python push_hf_model.py --model_dir ./outputs/ner-xlmr-eu-pii/best_hf --repo_id org/zoka-sentinel-ner \
  --product_name ZOKA-SENTINEL --product_version 1.2.0
```

## 4. Tester les modèles entraînés

Après `train_ner.py` vous disposez d’un dossier **`…/best_hf`** (PyTorch + tokenizer Hugging Face). Après `export_onnx.py`, d’artefacts **ONNX** + **`tokenizer_hf/tokenizer.json`** pour la crate Rust **`aegis_ner`**. Les vérifications ci-dessous sont complémentaires.

### 4.1 Métriques sur un jeu annoté (checkpoint PyTorch)

[`evaluate.py`](evaluate.py) charge le **modèle Hugging Face** depuis `model_dir`, parcourt un split du dataset disque (tokens + `ner_tags` IOB2) et produit un **rapport HTML** : F1 / précision / rappel par type, **F2** (β = 2), matrices de confusion, comparaison optionnelle avec Presidio.

```bash
# Jeu aligné sur le schéma AEGIS (même `LABELS` que à l’entraînement)
python evaluate.py \
  --dataset ./data/eu_pii_synthetic \
  --model_dir ./outputs/ner-xlmr-eu-pii/best_hf \
  --split validation \
  --max_samples 2000 \
  --out_report ./reports/ner_eval.html
```

Paramètres utiles :

| Option | Rôle |
|--------|------|
| `--split` | `train` ou `validation` (défaut : `validation`) |
| `--max_samples` | Plafonne le nombre de phrases évaluées (défaut : 2000) |
| `--with_presidio` | Ajoute une baseline Presidio (nécessite `en_core_web_sm` si analyse EN) |
| `--presidio_language` | Langue passée à Presidio (`en`, `fr`, …) |
| `--presidio_score_threshold` | Seuil de score analyzer Presidio (optionnel) |

Baseline Presidio (approximative, mappage des types → schéma AEGIS) :

```bash
python evaluate.py --with_presidio --out_report ./reports/ner_eval_presidio.html
```

**Benchmark Presidio seul** (précision / rappel / **F1** / **F2**, micro + par type) : [`benchmark_presidio_ner.py`](benchmark_presidio_ner.py) — même alignement token que `evaluate.py`, sortie tableau + JSON pour automatisation.

```bash
python benchmark_presidio_ner.py --dataset ./data/eu_pii_synthetic --split validation \
  --max_samples 1000 --language fr --json-out ./reports/presidio_bench.json
```

| Option (bench) | Rôle |
|----------------|------|
| `--language` | Code spaCy / Presidio (`en`, `fr`, …) — installer le modèle correspondant |
| `--score_threshold` | Seuil analyzer Presidio (optionnel) |
| `--json-out` | Fichier JSON (métriques + métadonnées) |
| `--quiet` | Supprime le tableau (utile avec `--json-out`) |

**Limite** : `evaluate.py` évalue le modèle **PyTorch** ; le bench Presidio n’utilise **pas** ONNX AEGIS. Pour ONNX, voir §4.2 et §4.4.

### 4.2 Latence et exécution ONNX (Python)

1. **Bench intégré** : relancer l’export **sans** `--skip_benchmark` pour comparer PyTorch FP32, ONNX FP32, graphe optimisé ORT et **INT8** sur une phrase fixe, et générer `latency_benchmark.txt` :

   ```bash
   python export_onnx.py --model_dir ./outputs/ner-xlmr-eu-pii/best_hf --out_dir ./exports/onnx_ner
   ```

2. **Contrôle manuel ONNX Runtime** (smoke sur CPU), depuis `training/` après export :

   ```bash
   python -c "
   import numpy as np
   from onnxruntime import InferenceSession
   from transformers import AutoTokenizer
   m = 'exports/onnx_ner/model_int8.onnx'
   tok = AutoTokenizer.from_pretrained('exports/onnx_ner/tokenizer_hf', local_files_only=True)
   s = 'Contact: Marie Dupont, marie@acme.eu'
   enc = tok([s], return_tensors='np', padding='max_length', truncation=True, max_length=128)
   sess = InferenceSession(m, providers=['CPUExecutionProvider'])
   out = sess.run(None, {'input_ids': enc['input_ids'].astype(np.int64),
                         'attention_mask': enc['attention_mask'].astype(np.int64)})[0]
   print('logits shape', out.shape)
   "
   ```

   Si la forme est `(1, 128, num_labels)` (ou équivalent selon longueur), le graphe et les entrées sont cohérents.

### 4.3 Tests automatisés du dépôt

| Commande | But |
|----------|-----|
| `python -m pytest training/tests -q` | Labels, `dataset_builder`, imports (CI **python-training**) — préférer `-m` si `pytest` n’est pas dans le `PATH` |
| `python -m pytest training/tests/test_l3_sensitive_letter_onnx.py training/tests/test_l3_expert_corpus_onnx.py -q` | Après `run_l3_pipeline.sh` : ONNX INT8 vs lettre FR + corpus expert. Les **marqueurs texte** doivent être retrouvés dans les spans à **≥ 95 %** (défaut, `AEGIS_ONNX_MIN_MARKER_PERCENT`) ; **`AEGIS_ONNX_STRICT_MARKERS=1`** impose **100 %**. Les chiffres / IBAN / téléphone restent vérifiés strictement. |
| `bash scripts/run_l3_pipeline.sh` | Synthétique + fusion **`letter_fr_golden.jsonl` + `corpus_expert_composite_fr_golden.jsonl`** (concat) → train → export |
| Workflow **Helm & NER L3 pipeline** (GitHub) | Pipeline + test lettre + artefact `aegis-ner-l3-onnx.tgz` |

Inférence ONNX réutilisable : module **`onnx_ner_infer.py`** (aligné sur `evaluate.py`).

### 4.4 Intégration moteur Rust (niveau 3)

Pour valider le **même** ONNX que en production :

1. Copier **`model_int8.onnx`** (ou `model.onnx`) et **`tokenizer.json`** (depuis `exports/.../tokenizer_hf/tokenizer.json`) dans un répertoire accessible au binaire (ex. volume **`/opt/aegis/models`** en conteneur).
2. La crate Rust **`aegis_ner`** (`crates/aegis-ner`) expose **`NerEngine::new(chemin_onnx, chemin_tokenizer_json, NerConfig::default())`** puis **`predict`** / **`predict_batch`** — voir `cargo doc -p aegis-ner --open` et le crate **`aegis-ner-training`** pour le lien pipeline Python → ONNX.
3. Dans **`aegis-config.yaml`**, activer le pipeline niveau **3** et renseigner **`ner.model_path`** vers le fichier `.onnx` (le tokenizer est attendu **à côté** du modèle ou selon la convention de déploiement documentée pour votre image — alignez les deux chemins avec ceux passés à `NerEngine` dans les intégrations qui chargent ONNX explicitement).

Ensuite, tester avec les flux réels (**API**, **CLI**, dashboard) sur des phrases contenant des PII du domaine visé, en plus des métriques offline du §4.1.

## Schéma d’étiquettes

Aligné entre `dataset_builder.py`, `train_ner.py` et `evaluate.py` : `O`, entités `B-*` / `I-*` pour les spans multi-mots (personne, téléphone, IBAN, etc.), et entités principalement sur un token pour `EMAIL`, `DATE`, `PASSPORT`, `LICENSE_PLATE`.

## Licence

En-tête des fichiers : **Apache 2.0 / MIT** (projet AEGIS — zokastech.fr).
