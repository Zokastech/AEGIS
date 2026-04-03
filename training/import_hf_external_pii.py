# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Download public Hugging Face datasets and convert them to AEGIS IOB2
(tokens + ner_tags), then export a DatasetDict (train/validation) for
train_ner.py and merge_hf_datasets.py.

Supported sources:
  - E3-JSI/synthetic-multi-pii-ner-v1  (entities + raw text, multilingual types)
  - ai4privacy/pii-masking-300k        (character span_labels + source_text)

External types are mapped to dataset_builder.LABELS; everything else is kept as
context "O" via (text, "O") segments.
"""

from __future__ import annotations

import argparse
import json
import os
import re
from typing import Any, Dict, List, Optional, Sequence, Tuple

from dataset_builder import LABEL2ID, LABELS, tokens_tags_from_chunks
from ensure_hf_datasets import load_datasets

_ds = load_datasets()
ClassLabel = _ds.ClassLabel
Dataset = _ds.Dataset
DatasetDict = _ds.DatasetDict
Sequence = _ds.Sequence
load_dataset = _ds.load_dataset

# ---------------------------------------------------------------------------
# E3-JSI: types (often Slovenian / multilingual) → AEGIS entity name (no B-/I-)
# ---------------------------------------------------------------------------

_E3JSI_TYPE_KEYWORDS: List[Tuple[str, str]] = [
    # Person
    ("osebno ime", "PERSON"),
    ("ime in priimek", "PERSON"),
    ("ime", "PERSON"),
    ("priimek", "PERSON"),
    ("pacient", "PERSON"),
    ("stranka", "PERSON"),
    ("zdravnik", "PERSON"),
    ("obtoženec", "PERSON"),
    # Contact
    ("email", "EMAIL"),
    ("elektronski", "EMAIL"),
    ("telefon", "PHONE"),
    ("številka", "PHONE"),
    ("mobil", "PHONE"),
    ("faks", "PHONE"),
    # Finance / ID
    ("iban", "IBAN"),
    ("številka računa", "IBAN"),
    ("kartice", "CREDIT_CARD"),
    ("kredit", "CREDIT_CARD"),
    ("cvv", "CREDIT_CARD"),
    ("social", "SSN"),
    ("davčna", "TAX_ID"),
    ("matična", "NATIONAL_ID"),
    ("osebna izkaznica", "NATIONAL_ID"),
    ("potni list", "PASSPORT"),
    ("passport", "PASSPORT"),
    # Address / place
    ("naslov", "ADDRESS"),
    ("ulica", "ADDRESS"),
    ("poštna", "ADDRESS"),
    ("mesto", "LOCATION"),
    ("kraj", "LOCATION"),
    ("lokacija", "LOCATION"),
    ("država", "LOCATION"),
    # Orgs
    ("organizacija", "ORGANIZATION"),
    ("banka", "ORGANIZATION"),
    ("podjetje", "ORGANIZATION"),
    ("ustanova", "ORGANIZATION"),
    ("bolnišnica", "ORGANIZATION"),
    ("sodišče", "ORGANIZATION"),
    ("datum", "DATE"),
    ("rojstva", "DATE"),
    ("registrsk", "LICENSE_PLATE"),
    ("tablic", "LICENSE_PLATE"),
    ("zdravstven", "MEDICAL_RECORD"),
    ("zdravstvenega zavarovanja", "MEDICAL_RECORD"),
    ("cnpj", "TAX_ID"),
    ("znesek", "O"),
    ("poklic", "O"),
    ("simptom", "O"),
    ("bolezen", "O"),
    ("zakon", "O"),
    ("dokument", "O"),
    ("storitev", "O"),
    ("čas", "DATE"),
    ("delovni čas", "O"),
]


def _e3jsi_map_type(raw: str) -> Optional[str]:
    t = raw.strip().lower()
    if not t:
        return None
    for needle, kind in _E3JSI_TYPE_KEYWORDS:
        if needle in t:
            if kind == "O":
                return None
            return kind
    return None


def _e3jsi_pick_kind(types: Sequence[str]) -> Optional[str]:
    best: Optional[str] = None
    for tp in types:
        k = _e3jsi_map_type(tp)
        if k is not None:
            best = k
            if k in ("PERSON", "EMAIL", "PHONE", "IBAN", "ADDRESS"):
                break
    return best


def _alloc_entity_span(text: str, entity_str: str, occupied: List[Tuple[int, int]]) -> Optional[Tuple[int, int]]:
    if not entity_str:
        return None
    start = 0
    elen = len(entity_str)
    while True:
        i = text.find(entity_str, start)
        if i < 0:
            return None
        e = i + elen
        if not any(max(i, os) < min(e, oe) for os, oe in occupied):
            return i, e
        start = i + 1


def _merge_non_overlapping(spans: List[Tuple[int, int, str]]) -> List[Tuple[int, int, str]]:
    """Keep spans sorted by start; on overlap, keep the longer span."""
    spans = sorted(spans, key=lambda x: (x[0], -(x[1] - x[0])))
    out: List[Tuple[int, int, str]] = []
    for s, e, k in spans:
        if s >= e:
            continue
        if not out:
            out.append((s, e, k))
            continue
        ps, pe, _ = out[-1]
        if s < pe:
            if e - s > pe - ps:
                out[-1] = (s, e, k)
            continue
        out.append((s, e, k))
    return out


def _sanitize_ner_tags(tags: List[str]) -> List[str]:
    """Replace I-TYPE tags missing from the AEGIS schema (e.g. I-MEDICAL_RECORD) with O."""
    fixed: List[str] = []
    for t in tags:
        if t in LABEL2ID:
            fixed.append(t)
            continue
        if t.startswith("I-"):
            base = t[2:]
            btag, itag = f"B-{base}", f"I-{base}"
            if btag in LABEL2ID and itag not in LABEL2ID:
                fixed.append("O")
                continue
        fixed.append("O")
    return fixed


def _text_and_spans_to_row(text: str, spans: List[Tuple[int, int, str]], lang: str, domain: str) -> Optional[Dict[str, Any]]:
    spans = _merge_non_overlapping(spans)
    if not spans:
        return None
    chunks: List[Tuple[str, str]] = []
    pos = 0
    for s, e, kind in sorted(spans, key=lambda x: x[0]):
        if s > pos:
            chunks.append((text[pos:s], "O"))
        chunks.append((text[s:e], kind))
        pos = e
    if pos < len(text):
        chunks.append((text[pos:], "O"))
    tokens: List[str] = []
    tags: List[str] = []
    for piece, kind in chunks:
        if not piece:
            continue
        t, g = tokens_tags_from_chunks([(piece, kind)])
        tokens.extend(t)
        tags.extend(g)
    if not tokens:
        return None
    tags = _sanitize_ner_tags(tags)
    if all(t == "O" for t in tags):
        return None
    for g in tags:
        if g not in LABEL2ID:
            return None
    return {"tokens": tokens, "ner_tags": tags, "lang": lang, "domain": domain}


def convert_e3jsi_example(ex: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    text = ex.get("text") or ""
    if not text.strip():
        return None
    entities = ex.get("entities") or []
    lang = str(ex.get("language") or ex.get("lang") or "e3jsi").lower()[:32]
    domain = str(ex.get("domain") or "e3jsi_synthetic").lower()[:48]
    spans: List[Tuple[int, int, str]] = []
    occupied: List[Tuple[int, int]] = []
    for ent in entities:
        if not isinstance(ent, dict):
            continue
        es = ent.get("entity")
        if not es or not isinstance(es, str):
            continue
        types = ent.get("types") or []
        if isinstance(types, str):
            types = [types]
        kind = _e3jsi_pick_kind([str(x) for x in types])
        if kind is None:
            continue
        found = _alloc_entity_span(text, es, occupied)
        if found is None:
            continue
        s, e = found
        spans.append((s, e, kind))
        occupied.append((s, e))
    if not spans:
        return None
    return _text_and_spans_to_row(text, spans, lang, domain)


# ---------------------------------------------------------------------------
# ai4privacy : span_labels (liste [start, end, label]) → chunks
# ---------------------------------------------------------------------------

_AI4PRIVACY_MAP = {
    "USERNAME": "PERSON",
    "LASTNAME1": "PERSON",
    "LASTNAME2": "PERSON",
    "LASTNAME3": "PERSON",
    "GIVENNAME1": "PERSON",
    "GIVENNAME2": "PERSON",
    "TITLE": "O",
    "EMAIL": "EMAIL",
    "TEL": "PHONE",
    "SOCIALNUMBER": "SSN",
    "PASSPORT": "PASSPORT",
    "DRIVERLICENSE": "NATIONAL_ID",
    "IDCARD": "NATIONAL_ID",
    "CREDITCARD": "CREDIT_CARD",
    "IBAN": "IBAN",
    "BOD": "DATE",
    "DATE": "DATE",
    "TIME": "DATE",
    "STREET": "ADDRESS",
    "CITY": "LOCATION",
    "STATE": "LOCATION",
    "POSTCODE": "ADDRESS",
    "BUILDING": "ADDRESS",
    "SECADDRESS": "ADDRESS",
    "COUNTRY": "LOCATION",
    "IP": "O",
    "PASS": "O",
    "SEX": "O",
    "CVV": "O",
    "CVC": "O",
    "ZIPCODE": "ADDRESS",
    "ACCOUNTNUMBER": "IBAN",
    "DEBITCARDNUMBER": "CREDIT_CARD",
    "MASKEDSEQUENCE": "O",
    "JSON": "O",
    "NUMERICAL": "O",
    "AMOUNT": "O",
    "ORGANIZATION": "ORGANIZATION",
    "COMPANYNAME": "ORGANIZATION",
    "JOBPOSITION": "O",
    "USERNAME_SOCIAL": "PERSON",
}


def _parse_span_labels(raw: Any) -> List[Tuple[int, int, str]]:
    if raw is None:
        return []
    if isinstance(raw, str):
        raw = raw.strip()
        if not raw:
            return []
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return []
    else:
        data = raw
    if not isinstance(data, list):
        return []
    out: List[Tuple[int, int, str]] = []
    for item in data:
        if not isinstance(item, (list, tuple)) or len(item) != 3:
            continue
        a, b, lab = item
        try:
            s, e = int(a), int(b)
        except (TypeError, ValueError):
            continue
        if s < 0 or e <= s:
            continue
        out.append((s, e, str(lab).strip().upper()))
    return out


def convert_ai4privacy_example(ex: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    text = ex.get("source_text") or ""
    if not text:
        return None
    spans_raw = _parse_span_labels(ex.get("span_labels"))
    lang = str(ex.get("language") or "ai4privacy").lower().replace(" ", "_")[:32]
    domain = "ai4privacy_pii_masking"
    spans: List[Tuple[int, int, str]] = []
    for s, e, lab in spans_raw:
        kind = _AI4PRIVACY_MAP.get(lab)
        if kind is None or kind == "O":
            continue
        spans.append((s, e, kind))
    if not spans:
        return None
    return _text_and_spans_to_row(text, spans, lang, domain)


def _rows_to_datasetdict(rows: List[Dict[str, Any]], val_ratio: float, seed: int) -> DatasetDict:
    if not rows:
        raise ValueError("Aucun exemple converti — vérifier le mapping ou augmenter max_samples.")
    ds = Dataset.from_list(rows)
    ds = ds.cast_column("ner_tags", Sequence(ClassLabel(names=LABELS)))
    split = ds.train_test_split(test_size=val_ratio, seed=seed)
    return DatasetDict(train=split["train"], validation=split["test"])


def cmd_e3jsi(args: argparse.Namespace) -> None:
    print("Chargement E3-JSI/synthetic-multi-pii-ner-v1 …")
    split_name = args.split or "train"
    hf = load_dataset("E3-JSI/synthetic-multi-pii-ner-v1", split=split_name)
    rows: List[Dict[str, Any]] = []
    max_n = args.max_samples or len(hf)
    for i in range(min(max_n, len(hf))):
        row = convert_e3jsi_example(hf[i])
        if row:
            rows.append(row)
        if (i + 1) % 500 == 0:
            print(f"  … {i + 1} lignes HF, {len(rows)} exemples retenus")
    print(f"Retenu {len(rows)} / {min(max_n, len(hf))} exemples (split={split_name}).")
    dd = _rows_to_datasetdict(rows, args.val_ratio, args.seed)
    os.makedirs(os.path.dirname(os.path.abspath(args.output)) or ".", exist_ok=True)
    dd.save_to_disk(args.output)
    print(f"OK → {args.output}")
    print(dd)


def cmd_ai4privacy(args: argparse.Namespace) -> None:
    print("Chargement ai4privacy/pii-masking-300k (peut être long la 1ère fois) …")
    split_name = args.split or "train"
    hf = load_dataset(
        "ai4privacy/pii-masking-300k",
        split=split_name,
        trust_remote_code=args.trust_remote_code,
    )
    n = len(hf)
    max_n = args.max_samples or n
    max_n = min(max_n, n)
    rows: List[Dict[str, Any]] = []
    step = max(1, max_n // 20)
    for i in range(max_n):
        row = convert_ai4privacy_example(hf[i])
        if row:
            rows.append(row)
        if (i + 1) % step == 0:
            print(f"  … {i + 1} lignes HF, {len(rows)} exemples retenus")
    print(f"Retenu {len(rows)} / {max_n} exemples (split={split_name}).")
    dd = _rows_to_datasetdict(rows, args.val_ratio, args.seed)
    os.makedirs(os.path.dirname(os.path.abspath(args.output)) or ".", exist_ok=True)
    dd.save_to_disk(args.output)
    print(f"OK → {args.output}")
    print(dd)


def main() -> None:
    p = argparse.ArgumentParser(description="HF public PII → DatasetDict AEGIS (IOB2).")
    sub = p.add_subparsers(dest="cmd", required=True)

    p_e = sub.add_parser("e3jsi", help="E3-JSI/synthetic-multi-pii-ner-v1")
    p_e.add_argument("--output", type=str, required=True, help="Répertoire save_to_disk")
    p_e.add_argument("--split", type=str, default="train", help="Split HF (souvent train)")
    p_e.add_argument("--max_samples", type=int, default=0, help="0 = tout le split")
    p_e.add_argument("--val_ratio", type=float, default=0.1)
    p_e.add_argument("--seed", type=int, default=42)
    p_e.set_defaults(func=cmd_e3jsi)

    p_a = sub.add_parser("ai4privacy", help="ai4privacy/pii-masking-300k")
    p_a.add_argument("--output", type=str, required=True)
    p_a.add_argument("--split", type=str, default="train")
    p_a.add_argument("--max_samples", type=int, default=0, help="0 = tout le split (très volumineux)")
    p_a.add_argument("--val_ratio", type=float, default=0.05)
    p_a.add_argument("--seed", type=int, default=42)
    p_a.add_argument(
        "--trust_remote_code",
        action="store_true",
        help="Si le dataset exige trust_remote_code au chargement.",
    )
    p_a.set_defaults(func=cmd_ai4privacy)

    args = p.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
