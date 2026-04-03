# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Charge un ou plusieurs fichiers JSONL (une ligne = un exemple) au format :
  {"tokens": ["Contact", ":", "a@b.com"], "ner_tags": ["O", "O", "B-EMAIL"]}

Les étiquettes doivent appartenir au schéma IOB2 de `dataset_builder.LABELS`.
Sortie : répertoire compatible `train_ner.py --dataset`.
"""

from __future__ import annotations

import argparse
import json
import os
from typing import Any, Dict, List

from dataset_builder import LABEL2ID, LABELS
from ensure_hf_datasets import load_datasets

_ds = load_datasets()
ClassLabel = _ds.ClassLabel
Dataset = _ds.Dataset
DatasetDict = _ds.DatasetDict
Sequence = _ds.Sequence


def _row_from_obj(obj: Dict[str, Any]) -> Dict[str, Any]:
    if "tokens" not in obj or "ner_tags" not in obj:
        raise ValueError("Chaque ligne doit contenir 'tokens' (liste) et 'ner_tags' (liste IOB2).")
    tokens = obj["tokens"]
    tags = obj["ner_tags"]
    if len(tokens) != len(tags):
        raise ValueError("tokens et ner_tags doivent avoir la même longueur.")
    for t in tags:
        if t not in LABEL2ID:
            raise ValueError(f"Label inconnu: {t!r} — doit être dans le schéma AEGIS ({len(LABELS)} labels).")
    return {
        "tokens": tokens,
        "ner_tags": tags,
        "lang": obj.get("lang", "custom"),
        "domain": obj.get("domain", "manual_jsonl"),
    }


def load_jsonl_paths(paths: List[str]) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for p in paths:
        with open(p, encoding="utf-8") as f:
            for i, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    rows.append(_row_from_obj(json.loads(line)))
                except (json.JSONDecodeError, ValueError) as e:
                    raise ValueError(f"{p}:{i}: {e}") from e
    return rows


def build_datasetdict(rows: List[Dict[str, Any]], val_ratio: float, seed: int) -> DatasetDict:
    if not rows:
        raise ValueError("Aucun exemple chargé.")
    ds = Dataset.from_list(rows)
    ds = ds.cast_column("ner_tags", Sequence(ClassLabel(names=LABELS)))
    split = ds.train_test_split(test_size=val_ratio, seed=seed)
    return DatasetDict(train=split["train"], validation=split["test"])


def main() -> None:
    p = argparse.ArgumentParser(description="JSONL → HuggingFace DatasetDict (NER IOB2 AEGIS).")
    p.add_argument("jsonl", nargs="+", help="Fichier(s) .jsonl")
    p.add_argument("--output", type=str, required=True, help="Répertoire sortie (save_to_disk)")
    p.add_argument("--val_ratio", type=float, default=0.1, help="Part validation (0–1)")
    p.add_argument("--seed", type=int, default=42)
    args = p.parse_args()

    rows = load_jsonl_paths(args.jsonl)
    dd = build_datasetdict(rows, args.val_ratio, args.seed)
    os.makedirs(os.path.dirname(os.path.abspath(args.output)) or ".", exist_ok=True)
    dd.save_to_disk(args.output)
    print(f"OK — {len(rows)} exemples → {args.output}")
    print(dd)


if __name__ == "__main__":
    main()
