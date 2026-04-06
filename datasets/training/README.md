# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Données pour l’entraînement NER (niveau 3)

Ce répertoire regroupe des **jeux complémentaires** pour affiner ou spécialiser le modèle **token classification** décrit dans [`training/README.md`](../../training/README.md). Le moteur AEGIS attend un modèle ONNX + tokenizer alignés sur ce pipeline (XLM-RoBERTa ou autre `--model_name`).

### Schéma d’étiquettes (IOB2)

Les labels autorisés sont ceux de `training/dataset_builder.py` (`LABELS`) : `O`, `B-PERSON`, `I-PERSON`, `B-EMAIL`, … — **ne pas inventer de nouveaux types** sans mettre à jour le code Rust / la config (`num_labels`, `ID2LABEL`).

### Formats

| Format | Usage |
|--------|--------|
| **JSONL** | Une ligne JSON par phrase : `tokens` (liste de mots) + `ner_tags` (liste de labels IOB2 de même longueur). Champs optionnels : `lang`, `domain`. |
| **HuggingFace `save_to_disk`** | Produit par `dataset_builder.py` ou `training/jsonl_to_hf_dataset.py` ; consommé par `train_ner.py --dataset`. |

### Dossiers d’exemple

| Dossier | Rôle |
|---------|------|
| [`ner_custom/`](ner_custom/) | Phrases annotées (`samples.jsonl`) + jeu **synthétique** long `fr_dossier_client_seed.jsonl` (mail FR dossier client, PII variées). |
| [`ner_financial_seed/`](ner_financial_seed/) | Amorce **finance / conformité** (IBAN, TVA, cartes) pour un second jeu ou un fine-tuning séparé. |
| [`l3_regression/`](l3_regression/) | **Régressions ONNX** : lettre sensible (`letter_fr_sensitive.txt` + `letter_fr_golden.jsonl`) et **corpus expert composite** (`corpus_expert_composite_fr.txt` + `corpus_expert_composite_fr_golden.jsonl`, multi-domaines CRM/SIH/juridique/finance/numérique). Les deux JSONL sont concaténés par `training/scripts/run_l3_pipeline.sh`. Étiquettes **pédagogiques** possibles (IP → `LOCATION`, montants → `TAX_ID`, etc.) — ne pas les confondre avec une taxonomie légale. |

### Chaîne outils

```bash
cd training
# JSONL → DatasetDict sur disque
python jsonl_to_hf_dataset.py ../datasets/training/ner_custom/samples.jsonl --output ./data/from_custom

# Fusion synthétique + custom
python merge_hf_datasets.py ./data/eu_pii_synthetic ./data/from_custom --output ./data/merged

python train_ner.py --dataset ./data/merged --output_dir ./outputs/ner-merged --cpu
```

### Notebook

Voir [`examples/notebooks/train_ner_pii.ipynb`](../../examples/notebooks/train_ner_pii.ipynb) pour un flux interactif (génération synthétique, import JSONL, entraînement, export ONNX optionnel).

### Données sensibles

N’importez **pas** de PII réelles dans le dépôt. Utilisez des **données synthétiques** ou des jeux anonymisés conformes à votre politique interne.
