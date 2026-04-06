# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Inférence NER token-classification via ONNX Runtime + tokenizer Hugging Face,
alignée sur `evaluate.predict_tags_per_word` (alignement sous-mots → mots).
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

import numpy as np

from dataset_builder import ID2LABEL
from ner_span_iob import refine_iob, tags_to_spans


def predict_tags_per_word_onnx(
    session: InferenceSession,
    tokenizer: Any,
    tokens: List[str],
    id2label: Optional[Dict[int, str]] = None,
    max_length: int = 256,
) -> List[str]:
    """Retourne une étiquette IOB2 par mot (même convention que l’entraînement)."""
    id2label = id2label or ID2LABEL
    enc = tokenizer(
        tokens,
        is_split_into_words=True,
        return_tensors="np",
        truncation=True,
        max_length=max_length,
        padding="max_length",
    )
    input_ids = enc["input_ids"].astype(np.int64)
    attention_mask = enc["attention_mask"].astype(np.int64)
    logits = session.run(
        None,
        {"input_ids": input_ids, "attention_mask": attention_mask},
    )[0]
    pred = logits[0].argmax(axis=-1)
    word_ids: List[Optional[int]] = enc.word_ids(0)
    out_tags: List[Optional[str]] = [None] * len(tokens)
    for idx, wid in enumerate(word_ids):
        if wid is None or wid >= len(out_tags):
            continue
        if out_tags[wid] is None:
            lab = id2label[int(pred[idx])]
            out_tags[wid] = lab
    return [t if t is not None else "O" for t in out_tags]


def span_texts_from_tokens_and_tags(tokens: List[str], tags: List[str]) -> List[str]:
    """Texte des spans entité (IOB2) pour concaténation mot à mot."""
    refined = refine_iob(tags)
    spans = tags_to_spans(refined)
    parts: List[str] = []
    for sp in spans:
        parts.append(" ".join(tokens[sp.start : sp.end]))
    return parts


def collect_entity_texts_onnx_lines(
    session: InferenceSession,
    tokenizer: Any,
    lines: List[str],
    id2label: Optional[Dict[int, str]] = None,
    max_length: int = 256,
) -> List[str]:
    """Pour chaque ligne non vide : tokenisation whitespace, prédiction, spans concaténés."""
    collected: List[str] = []
    for line in lines:
        s = line.strip()
        if not s:
            continue
        toks = s.split()
        if not toks:
            continue
        tags = predict_tags_per_word_onnx(session, tokenizer, toks, id2label, max_length)
        collected.extend(span_texts_from_tokens_and_tags(toks, tags))
    return collected
