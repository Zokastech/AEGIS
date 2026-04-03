# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Publie sur le Hugging Face Hub un dossier checkpoint produit par train_ner.py
(ex. …/outputs/…/best_hf : config, poids, tokenizer).
"""

from __future__ import annotations

import argparse
import os

from huggingface_hub import HfApi, create_repo


def main() -> None:
    p = argparse.ArgumentParser(
        description="Publier un checkpoint NER (best_hf) sur le Hub Hugging Face (token-classification)."
    )
    p.add_argument(
        "--model_dir",
        type=str,
        required=True,
        help="Dossier best_hf (config.json, tokenizer, poids PyTorch / safetensors).",
    )
    p.add_argument(
        "--repo_id",
        type=str,
        required=True,
        help="Identifiant du dépôt modèle, ex. mon-org/aegis-ner-eu-pii.",
    )
    p.add_argument("--private", action="store_true", help="Créer ou utiliser un dépôt privé.")
    p.add_argument(
        "--token",
        type=str,
        default=None,
        help="Jeton HF (sinon variable HF_TOKEN ou cache `huggingface-cli login`).",
    )
    p.add_argument(
        "--commit_message",
        type=str,
        default="Upload AEGIS NER token-classification checkpoint",
    )
    args = p.parse_args()

    model_dir = os.path.abspath(args.model_dir)
    if not os.path.isdir(model_dir):
        raise SystemExit(f"Dossier introuvable : {model_dir}")
    cfg = os.path.join(model_dir, "config.json")
    if not os.path.isfile(cfg):
        raise SystemExit(
            f"config.json manquant dans {model_dir} — indiquez le répertoire best_hf après train_ner.py."
        )

    token = args.token if args.token is not None else os.environ.get("HF_TOKEN")
    api = HfApi(token=token)
    create_repo(
        repo_id=args.repo_id,
        private=args.private,
        exist_ok=True,
        repo_type="model",
        token=token,
    )
    api.upload_folder(
        folder_path=model_dir,
        repo_id=args.repo_id,
        repo_type="model",
        commit_message=args.commit_message,
        token=token,
        ignore_patterns=[".DS_Store", "__pycache__/**", ".git/**"],
    )
    print(f"Publié : https://huggingface.co/{args.repo_id}")


if __name__ == "__main__":
    main()
