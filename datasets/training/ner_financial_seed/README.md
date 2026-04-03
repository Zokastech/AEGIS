# Seed finance / conformité

Exemples **synthétiques** orientés **IBAN, carte, TVA** pour enrichir un jeu destiné à un modèle « financial compliance » ou à fusionner avec le corpus EU générique.

Même format JSONL que [`../ner_custom/`](../ner_custom/). Après conversion + fusion :

```bash
cd training
python jsonl_to_hf_dataset.py ../datasets/training/ner_financial_seed/samples.jsonl --output ./data/fin_seed --val_ratio 0.15
python merge_hf_datasets.py ./data/eu_pii_synthetic ./data/fin_seed --output ./data/eu_plus_fin
python train_ner.py --dataset ./data/eu_plus_fin --output_dir ./outputs/ner-eu-fin --fp16
```

Pour un **deuxième modèle** (checkpoint séparé), pointez `--output_dir` vers un autre dossier et, après `export_onnx.py`, montez un autre fichier `.onnx` dans le conteneur moteur (`ner.model_path`).
