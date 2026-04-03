# Jeu custom (seed)

| Fichier | Contenu |
|---------|---------|
| [`samples.jsonl`](samples.jsonl) | Amorce courte (email, IBAN, adresse, …). |
| [`fr_dossier_client_seed.jsonl`](fr_dossier_client_seed.jsonl) | **Synthétique** : courrier FR type dossier client (NIR « serait », adresses multilignes, emails obfusqués `(at)`, téléphones, IBAN, carte masquée, SIRET « possible », téléphone partiellement masqué, etc.). |

Pour entraîner sur `samples.jsonl` seul :

```bash
cd training
python jsonl_to_hf_dataset.py ../datasets/training/ner_custom/samples.jsonl --output ./data/custom_seed --val_ratio 0.2
python train_ner.py --dataset ./data/custom_seed --output_dir ./outputs/ner-custom --num_train_epochs 2 --per_device_train_batch_size 8 --cpu
```

Pour inclure le jeu **dossier client** (concaténation JSONL puis conversion) :

```bash
cd training
cat ../datasets/training/ner_custom/samples.jsonl ../datasets/training/ner_custom/fr_dossier_client_seed.jsonl > ../datasets/training/ner_custom/merged_custom.jsonl
python jsonl_to_hf_dataset.py ../datasets/training/ner_custom/merged_custom.jsonl --output ./data/custom_merged --val_ratio 0.2
python train_ner.py --dataset ./data/custom_merged --output_dir ./outputs/ner-custom --num_train_epochs 2 --per_device_train_batch_size 8 --cpu
```

Ajoutez vos propres lignes dans `samples.jsonl` (ou de nouveaux `.jsonl`) puis fusionnez avec un jeu synthétique large via `merge_hf_datasets.py` pour éviter le sur-apprentissage sur quelques phrases.
