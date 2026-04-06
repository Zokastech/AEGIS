# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Évaluation NER : F1 / P / R par type, F2 (β=2), matrices de confusion, rapport HTML.
Comparaison optionnelle avec Presidio sur le même jeu.
"""

from __future__ import annotations

import argparse
import base64
import io
import os
from typing import Any, Dict, List, Optional, Sequence, Set, Tuple

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import torch
from ensure_hf_datasets import load_datasets
from sklearn.preprocessing import LabelEncoder

load_from_disk = load_datasets().load_from_disk
from transformers import AutoModelForTokenClassification

from dataset_builder import ID2LABEL, LABELS
from hf_tokenizer_utils import load_autotokenizer_pretrained
from ner_span_iob import Span, refine_iob, tags_to_spans

try:
    from presidio_analyzer import AnalyzerEngine
except ImportError:  # pragma: no cover
    AnalyzerEngine = None  # type: ignore[misc, assignment]


BETA_MAIN = 2.0


def f_beta(precision: float, recall: float, beta: float = BETA_MAIN) -> float:
    if precision + recall == 0:
        return 0.0
    b2 = beta * beta
    return (1 + b2) * precision * recall / (b2 * precision + recall)


def per_entity_prf2(
    y_true_spans: List[List[Span]],
    y_pred_spans: List[List[Span]],
    entity_types: List[str],
) -> Tuple[Dict[str, Dict[str, float]], float]:
    per: Dict[str, Dict[str, float]] = {}
    total_tp = total_fp = total_fn = 0
    for et in entity_types:
        tp = fp = fn = 0
        for gold, pred in zip(y_true_spans, y_pred_spans):
            gs = {s for s in gold if s.etype == et}
            ps = {s for s in pred if s.etype == et}
            tp += len(gs & ps)
            fp += len(ps - gs)
            fn += len(gs - ps)
        total_tp += tp
        total_fp += fp
        total_fn += fn
        p = tp / (tp + fp) if (tp + fp) else 0.0
        r = tp / (tp + fn) if (tp + fn) else 0.0
        per[et] = {
            "precision": p,
            "recall": r,
            "f1": (2 * p * r / (p + r)) if (p + r) else 0.0,
            "f2": f_beta(p, r, BETA_MAIN),
            "support": tp + fn,
        }
    p_micro = total_tp / (total_tp + total_fp) if (total_tp + total_fp) else 0.0
    r_micro = total_tp / (total_tp + total_fn) if (total_tp + total_fn) else 0.0
    f2_micro = f_beta(p_micro, r_micro, BETA_MAIN)
    return per, f2_micro


def micro_span_entity_scores(
    y_true_spans: List[List[Span]],
    y_pred_spans: List[List[Span]],
    entity_types: List[str],
) -> Dict[str, float]:
    """Métriques **micro** (agrégation globale des spans) : P, R, F1, F2 + comptes."""
    tp = fp = fn = 0
    for et in entity_types:
        for gold, pred in zip(y_true_spans, y_pred_spans):
            gs = {s for s in gold if s.etype == et}
            ps = {s for s in pred if s.etype == et}
            tp += len(gs & ps)
            fp += len(ps - gs)
            fn += len(gs - ps)
    p = tp / (tp + fp) if (tp + fp) else 0.0
    r = tp / (tp + fn) if (tp + fn) else 0.0
    f1 = (2 * p * r / (p + r)) if (p + r) else 0.0
    f2 = f_beta(p, r, BETA_MAIN)
    return {
        "tp": float(tp),
        "fp": float(fp),
        "fn": float(fn),
        "precision": p,
        "recall": r,
        "f1": f1,
        "f2": f2,
    }


def confusion_entity_level(
    y_true_spans: List[List[Span]],
    y_pred_spans: List[List[Span]],
    types: List[str],
) -> np.ndarray:
    le = LabelEncoder()
    le.fit(types + ["O", "MISMATCH"])
    size = len(le.classes_)
    cm = np.zeros((size, size), dtype=int)
    for gold, pred in zip(y_true_spans, y_pred_spans):
        gmap: Dict[Tuple[int, int], str] = {(s.start, s.end): s.etype for s in gold}
        pmap: Dict[Tuple[int, int], str] = {(s.start, s.end): s.etype for s in pred}
        keys: Set[Tuple[int, int]] = set(gmap) | set(pmap)
        for k in keys:
            gt = gmap.get(k, "O")
            pt = pmap.get(k, "O")
            if gt not in le.classes_:
                gt = "MISMATCH"
            if pt not in le.classes_:
                pt = "MISMATCH"
            i = int(le.transform([gt])[0])
            j = int(le.transform([pt])[0])
            cm[i, j] += 1
    return cm, le


def predict_tags_per_word(
    model: torch.nn.Module,
    tokenizer: Any,
    tokens: List[str],
    max_length: int = 512,
) -> List[str]:
    model.eval()
    enc = tokenizer(
        tokens,
        is_split_into_words=True,
        return_tensors="pt",
        truncation=True,
        max_length=max_length,
        padding="max_length",
    )
    word_ids = enc.word_ids(0)
    with torch.no_grad():
        logits = model(**enc).logits[0].cpu().numpy()
    pred = logits.argmax(-1)
    tags: List[Optional[str]] = [None] * len(tokens)
    for idx, wid in enumerate(word_ids):
        if wid is None or wid >= len(tags):
            continue
        if tags[wid] is None:
            tags[wid] = ID2LABEL[int(pred[idx])]
    return [t if t is not None else "O" for t in tags]


def refine_iob(tags: List[str]) -> List[str]:
    out = list(tags)
    for i in range(len(out)):
        if out[i].startswith("I-"):
            et = out[i][2:]
            prev = out[i - 1] if i else "O"
            if prev not in (f"B-{et}", f"I-{et}"):
                out[i] = f"B-{et}"
    return out


def presidio_to_tags(
    text: str,
    words: List[str],
    analyzer: Any,
    language: str = "en",
    score_threshold: Optional[float] = None,
) -> List[str]:
    if analyzer is None:
        return ["O"] * len(words)
    kw: Dict[str, Any] = dict(text=text, language=language, entities=[])
    if score_threshold is not None:
        kw["score_threshold"] = score_threshold
    results = analyzer.analyze(**kw)
    char_spans: List[Tuple[int, int, str]] = []
    for r in results:
        char_spans.append((r.start, r.end, map_presidio_entity(r.entity_type)))
    tags = ["O"] * len(words)
    if not words:
        return tags
    offsets: List[Tuple[int, int]] = []
    pos = 0
    for w in words:
        start = text.find(w, pos)
        if start < 0:
            start = pos
        end = start + len(w)
        offsets.append((start, end))
        pos = end + 1
    for i, (ws, we) in enumerate(offsets):
        for cs, ce, et in char_spans:
            if not (we <= cs or ws >= ce):
                tags[i] = f"B-{et}" if tags[i] == "O" else tags[i]
    return tags


def map_presidio_entity(presidio_type: str) -> str:
    m = {
        "PERSON": "PERSON",
        "EMAIL_ADDRESS": "EMAIL",
        "PHONE_NUMBER": "PHONE",
        "IBAN_CODE": "IBAN",
        "CREDIT_CARD": "CREDIT_CARD",
        "US_SSN": "SSN",
        "US_PASSPORT": "PASSPORT",
        "LOCATION": "LOCATION",
        "ORGANIZATION": "ORGANIZATION",
        "DATE_TIME": "DATE",
        "NRP": "NATIONAL_ID",
        "MEDICAL_LICENSE": "MEDICAL_RECORD",
    }
    return m.get(presidio_type, "NATIONAL_ID")


def render_html(
    per_model: Dict[str, Dict[str, float]],
    f2_micro: float,
    cm_img_b64: str,
    per_presidio: Optional[Dict[str, Dict[str, float]]],
    f2_presidio: Optional[float],
) -> str:
    rows = "".join(
        f"<tr><td>{k}</td><td>{v['precision']:.4f}</td><td>{v['recall']:.4f}</td>"
        f"<td>{v['f1']:.4f}</td><td>{v['f2']:.4f}</td><td>{int(v['support'])}</td></tr>"
        for k, v in sorted(per_model.items())
    )
    presidio_block = ""
    if per_presidio is not None and f2_presidio is not None:
        pr = "".join(
            f"<tr><td>{k}</td><td>{v['precision']:.4f}</td><td>{v['recall']:.4f}</td>"
            f"<td>{v['f1']:.4f}</td><td>{v['f2']:.4f}</td><td>{int(v['support'])}</td></tr>"
            for k, v in sorted(per_presidio.items())
        )
        presidio_block = f"""
        <h2>Presidio (baseline, même texte — alignement approximatif)</h2>
        <p>F2 micro (entités) : <b>{f2_presidio:.4f}</b></p>
        <table border="1" cellpadding="4"><tr><th>Type</th><th>P</th><th>R</th><th>F1</th><th>F2</th><th>Support</th></tr>{pr}</table>
        """
    return f"""<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>AEGIS NER — rapport</title></head>
<body>
<h1>AEGIS — NER PII EU — évaluation</h1>
<p>Métrique principale : <b>F2</b> (β={BETA_MAIN}) au niveau entité (micro) : <b>{f2_micro:.4f}</b></p>
<table border="1" cellpadding="4">
<tr><th>Type</th><th>Precision</th><th>Recall</th><th>F1</th><th>F2</th><th>Support</th></tr>
{rows}
</table>
<h2>Matrice de confusion (positions d’entités gold vs prédit)</h2>
<img src="data:image/png;base64,{cm_img_b64}" alt="confusion"/>
{presidio_block}
</body></html>"""


def main() -> None:
    parser = argparse.ArgumentParser(description="Evaluate NER + HTML report.")
    parser.add_argument("--dataset", type=str, default="./data/eu_pii_synthetic")
    parser.add_argument("--model_dir", type=str, default="./outputs/ner-xlmr-eu-pii/best_hf")
    parser.add_argument("--split", type=str, default="validation")
    parser.add_argument("--max_samples", type=int, default=2000)
    parser.add_argument("--out_report", type=str, default="./reports/ner_eval.html")
    parser.add_argument("--with_presidio", action="store_true")
    parser.add_argument(
        "--presidio_language",
        type=str,
        default="en",
        help="Code langue Presidio / spaCy (ex. en, fr) — installer le modèle spaCy correspondant.",
    )
    parser.add_argument(
        "--presidio_score_threshold",
        type=float,
        default=None,
        help="Seuil optionnel Presidio (score analyzer).",
    )
    args = parser.parse_args()

    os.makedirs(os.path.dirname(args.out_report) or ".", exist_ok=True)

    ds = load_from_disk(args.dataset)[args.split]
    n = min(len(ds), args.max_samples)
    ds = ds.select(range(n))

    tokenizer = load_autotokenizer_pretrained(args.model_dir, use_fast=True)
    model = AutoModelForTokenClassification.from_pretrained(args.model_dir)
    model.eval()

    entity_types = sorted({lab[2:] for lab in LABELS if lab.startswith("B-")})

    y_true_spans: List[List[Span]] = []
    y_pred_spans: List[List[Span]] = []

    analyzer = AnalyzerEngine() if args.with_presidio and AnalyzerEngine is not None else None
    if args.with_presidio and AnalyzerEngine is None:
        print("Presidio non installé — ignorer --with_presidio.")

    y_presidio_spans: List[List[Span]] = []

    for row in ds:
        tokens: List[str] = row["tokens"]
        tags_raw = row["ner_tags"]
        if tags_raw and not isinstance(tags_raw[0], str):
            gold_tags = [ID2LABEL[int(x)] for x in tags_raw]
        else:
            gold_tags = list(tags_raw)
        text = " ".join(tokens)

        word_tags_pred = refine_iob(predict_tags_per_word(model, tokenizer, tokens))
        y_true_spans.append(tags_to_spans(gold_tags))
        y_pred_spans.append(tags_to_spans(word_tags_pred))

        if analyzer is not None:
            pt = presidio_to_tags(
                text,
                tokens,
                analyzer,
                language=args.presidio_language,
                score_threshold=args.presidio_score_threshold,
            )
            pt = refine_iob(pt)
            y_presidio_spans.append(tags_to_spans(pt))

    per, f2_micro = per_entity_prf2(y_true_spans, y_pred_spans, entity_types)

    cm, le = confusion_entity_level(y_true_spans, y_pred_spans, list(entity_types))
    _, ax = plt.subplots(figsize=(10, 8))
    im = ax.imshow(cm, interpolation="nearest", cmap=plt.cm.Blues)
    ax.figure.colorbar(im, ax=ax)
    ax.set_xticks(np.arange(cm.shape[1]))
    ax.set_yticks(np.arange(cm.shape[0]))
    ax.set_xticklabels(le.classes_, rotation=45, ha="right")
    ax.set_yticklabels(le.classes_)
    ax.set_ylabel("Gold")
    ax.set_xlabel("Predicted")
    buf = io.BytesIO()
    plt.tight_layout()
    plt.savefig(buf, format="png", dpi=120)
    plt.close()
    cm_b64 = base64.b64encode(buf.getvalue()).decode("ascii")

    per_pr: Optional[Dict[str, Dict[str, float]]] = None
    f2_pr: Optional[float] = None
    if analyzer is not None and y_presidio_spans:
        per_pr, f2_pr = per_entity_prf2(y_true_spans, y_presidio_spans, entity_types)

    html = render_html(per, f2_micro, cm_b64, per_pr, f2_pr)
    with open(args.out_report, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"F2 micro (entités) — modèle : {f2_micro:.4f}")
    if f2_pr is not None:
        print(f"F2 micro (entités) — Presidio (baseline, même gold) : {f2_pr:.4f}")
        print(f"Écart F2 (modèle − Presidio) : {f2_micro - f2_pr:+.4f}")
    print(f"Rapport : {args.out_report}")


if __name__ == "__main__":
    main()
