#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Benchmark **Microsoft Presidio** sur un jeu NER token-level (Hugging Face `datasets` sur disque),
avec scoring automatique : précision / rappel / **F1** / **F2** (micro + par type d’entité).

Alignement gold ↔ Presidio : mêmes conventions que `evaluate.py` (spans au niveau **mot**,
correspondance exacte start/end après projection char → tokens).

Prérequis :
  pip install presidio-analyzer spacy
  python -m spacy download en_core_web_sm   # si --language en
  python -m spacy download fr_core_news_sm  # si --language fr

Exemple :
  cd training
  python benchmark_presidio_ner.py --dataset ./data/eu_pii_synthetic --split validation \\
    --max_samples 500 --language fr --json-out ./reports/presidio_scores.json
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List

from dataset_builder import ID2LABEL, LABELS
from ensure_hf_datasets import load_datasets
from evaluate import (
    AnalyzerEngine,
    micro_span_entity_scores,
    per_entity_prf2,
    presidio_to_tags,
    refine_iob,
    tags_to_spans,
)

load_from_disk = load_datasets().load_from_disk


def _print_table(per: Dict[str, Dict[str, float]], micro: Dict[str, float]) -> None:
    print("\n=== Presidio — micro (tous types, spans) ===")
    print(
        f"  TP={int(micro['tp'])}  FP={int(micro['fp'])}  FN={int(micro['fn'])}\n"
        f"  Precision={micro['precision']:.4f}  Recall={micro['recall']:.4f}  "
        f"F1={micro['f1']:.4f}  F2={micro['f2']:.4f}"
    )
    print("\n=== Presidio — par type (entity) ===")
    print(f"{'type':<18} {'P':>8} {'R':>8} {'F1':>8} {'F2':>8} {'support':>8}")
    for et in sorted(per.keys()):
        row = per[et]
        print(
            f"{et:<18} {row['precision']:>8.4f} {row['recall']:>8.4f} "
            f"{row['f1']:>8.4f} {row['f2']:>8.4f} {int(row['support']):>8}"
        )
    print()


def main() -> None:
    p = argparse.ArgumentParser(
        description="Benchmark Presidio (P/R/F1/F2) sur dataset IOB2 disque."
    )
    p.add_argument("--dataset", type=str, default="./data/eu_pii_synthetic")
    p.add_argument("--split", type=str, default="validation")
    p.add_argument("--max_samples", type=int, default=2000)
    p.add_argument(
        "--language",
        type=str,
        default="en",
        help="Langue Presidio (en, fr, …) — modèle spaCy requis.",
    )
    p.add_argument(
        "--score_threshold",
        type=float,
        default=None,
        help="Seuil score Presidio (défaut : comportement moteur Presidio).",
    )
    p.add_argument(
        "--json-out",
        type=str,
        default="",
        help="Écriture des métriques + métadonnées en JSON (dossier créé si besoin).",
    )
    p.add_argument("--quiet", action="store_true", help="Pas de tableau, uniquement JSON si fourni.")
    args = p.parse_args()

    if AnalyzerEngine is None:
        print(
            "Erreur : installez presidio-analyzer (et spacy + modèle langue).\n"
            "  pip install presidio-analyzer spacy\n"
            "  python -m spacy download en_core_web_sm",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        analyzer = AnalyzerEngine()
    except Exception as exc:  # pragma: no cover
        print(f"Erreur initialisation Presidio : {exc}", file=sys.stderr)
        sys.exit(1)

    ds_path = Path(args.dataset)
    if not ds_path.is_dir():
        print(f"Dataset introuvable : {ds_path}", file=sys.stderr)
        sys.exit(1)

    ds = load_from_disk(str(ds_path))[args.split]
    n = min(len(ds), args.max_samples)
    ds = ds.select(range(n))

    entity_types = sorted({lab[2:] for lab in LABELS if lab.startswith("B-")})

    y_true_spans = []
    y_presidio_spans = []

    for row in ds:
        tokens: List[str] = row["tokens"]
        tags_raw = row["ner_tags"]
        if tags_raw and not isinstance(tags_raw[0], str):
            gold_tags = [ID2LABEL[int(x)] for x in tags_raw]
        else:
            gold_tags = list(tags_raw)
        text = " ".join(tokens)

        pt = presidio_to_tags(
            text,
            tokens,
            analyzer,
            language=args.language,
            score_threshold=args.score_threshold,
        )
        pt = refine_iob(pt)
        y_true_spans.append(tags_to_spans(gold_tags))
        y_presidio_spans.append(tags_to_spans(pt))

    per, f2_micro = per_entity_prf2(y_true_spans, y_presidio_spans, entity_types)
    micro = micro_span_entity_scores(y_true_spans, y_presidio_spans, entity_types)
    if abs(micro["f2"] - f2_micro) > 1e-6:
        print(f"Avertissement : F2 micro incohérent {micro['f2']} vs {f2_micro}", file=sys.stderr)

    if not args.quiet:
        _print_table(per, micro)

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "tool": "presidio_analyzer",
            "dataset": str(ds_path.resolve()),
            "split": args.split,
            "samples": n,
            "language": args.language,
            "score_threshold": args.score_threshold,
            "micro": {k: micro[k] for k in ("tp", "fp", "fn", "precision", "recall", "f1", "f2")},
            "per_entity": per,
            "generated_at": datetime.now(timezone.utc).isoformat(),
        }
        out_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
        if not args.quiet:
            print(f"JSON : {out_path.resolve()}")

    if args.quiet and not args.json_out:
        print(json.dumps({"micro": micro, "per_entity": per}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
