# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Fusionne plusieurs DatasetDict sauvegardés sur disque (mêmes splits train/validation,
mêmes colonnes) pour un seul dossier consommable par train_ner.py.
"""

from __future__ import annotations

import argparse
import os
from typing import List

from ensure_hf_datasets import load_datasets

_ds = load_datasets()
DatasetDict = _ds.DatasetDict
concatenate_datasets = _ds.concatenate_datasets
load_from_disk = _ds.load_from_disk


def main() -> None:
    p = argparse.ArgumentParser(description="Fusionner des répertoires HuggingFace DatasetDict.")
    p.add_argument("inputs", nargs="+", help="Répertoires contenant train/ validation/")
    p.add_argument("--output", type=str, required=True)
    args = p.parse_args()

    trains: List = []
    vals: List = []
    for path in args.inputs:
        d: DatasetDict = load_from_disk(path)
        if "train" not in d or "validation" not in d:
            raise ValueError(f"{path}: attendu splits 'train' et 'validation'")
        trains.append(d["train"])
        vals.append(d["validation"])

    out = DatasetDict(
        train=concatenate_datasets(trains),
        validation=concatenate_datasets(vals),
    )
    os.makedirs(os.path.dirname(os.path.abspath(args.output)) or ".", exist_ok=True)
    out.save_to_disk(args.output)
    print(f"Fusionné {len(args.inputs)} jeux → {args.output}")
    print(out)


if __name__ == "__main__":
    main()
