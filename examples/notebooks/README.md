# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Notebooks

- [`aegis_demo.ipynb`](aegis_demo.ipynb) — appels HTTP vers la passerelle (**moteur Rust réel**) : analyse (`analysis_config_json`), anonymisation, extrait Pandas.
- [`train_ner_pii.ipynb`](train_ner_pii.ipynb) — pipeline **NER PII** : données synthétiques, import JSONL (`datasets/training/`), entraînement Hugging Face, publication optionnelle sur le **Hugging Face Hub** (`push_hf_model.py`), export ONNX (voir `training/README.md`).
- [`train_ner_hf_public.ipynb`](train_ner_hf_public.ipynb) — même pipeline avec **jeux Hugging Face publics** (E3-JSI + option Ai4Privacy), fusion, entraînement, Hub, export ONNX.
- [`train_ner_colab.ipynb`](train_ner_colab.ipynb) — même pipeline pour **Google Colab** : clone du dépôt, GPU + `--fp16`, fusion JSONL optionnelle, export ONNX, copie optionnelle vers **Google Drive**.

Lancer Jupyter depuis la racine du dépôt ou ce dossier :

```bash
pip install jupyter requests pandas
jupyter notebook examples/notebooks/aegis_demo.ipynb
```

Avec `docker compose up` (racine), la passerelle est en **HTTPS** sur `https://127.0.0.1:8443` (défaut du notebook, TLS non vérifié pour le cert auto-signé). Avec `docker-compose.dev.yml`, utilisez `AEGIS_BASE_URL=http://127.0.0.1:8080`.
