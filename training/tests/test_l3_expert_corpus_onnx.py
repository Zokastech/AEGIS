# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Régression L3 — corpus expert composite (multi-domaines : identité, médical, juridique,
financier, numérique, relations).

Fichier source : `datasets/training/l3_regression/corpus_expert_composite_fr.txt`
Jeu d’or : `corpus_expert_composite_fr_golden.jsonl` (fusionné dans `run_l3_pipeline.sh`).

Les étiquettes « or » pour IP, montants, références dossier sont pédagogiques ; voir
`datasets/training/README.md` (l3_regression).
"""

from __future__ import annotations

import os
import re
from pathlib import Path

import pytest

_TRAINING = Path(__file__).resolve().parents[1]
_REPO = _TRAINING.parent
_CORPUS = _REPO / "datasets/training/l3_regression/corpus_expert_composite_fr.txt"
_DEFAULT_ONNX = _TRAINING / "exports/ci_onnx/model_int8.onnx"
_DEFAULT_TOK = _TRAINING / "exports/ci_onnx/tokenizer_hf"


def _digits(s: str) -> str:
    return re.sub(r"\D", "", s)


@pytest.fixture(scope="module")
def onnx_session():
    pytest.importorskip("onnxruntime")
    from onnxruntime import InferenceSession

    onnx_p = Path(os.environ.get("AEGIS_ONNX_MODEL", str(_DEFAULT_ONNX)))
    if not onnx_p.is_file():
        pytest.skip(f"Modèle ONNX absent : {onnx_p} (lancer scripts/run_l3_pipeline.sh)")
    return InferenceSession(str(onnx_p), providers=["CPUExecutionProvider"])


@pytest.fixture(scope="module")
def ner_tokenizer():
    pytest.importorskip("transformers")
    from transformers import AutoTokenizer

    tok_dir = Path(os.environ.get("AEGIS_ONNX_TOKENIZER", str(_DEFAULT_TOK)))
    if not (tok_dir / "tokenizer.json").is_file():
        pytest.skip(f"Tokenizer absent : {tok_dir}")
    return AutoTokenizer.from_pretrained(str(tok_dir), local_files_only=True)


@pytest.fixture(scope="module")
def expert_lines():
    assert _CORPUS.is_file(), f"Fixture corpus expert manquante : {_CORPUS}"
    return _CORPUS.read_text(encoding="utf-8").splitlines()


def test_l3_onnx_covers_expert_composite_corpus(onnx_session, ner_tokenizer, expert_lines):
    from onnx_ner_infer import collect_entity_texts_onnx_lines

    spans = collect_entity_texts_onnx_lines(
        onnx_session,
        ner_tokenizer,
        expert_lines,
        max_length=int(os.environ.get("AEGIS_ONNX_MAX_LENGTH", "256")),
    )
    blob = " ".join(spans).lower()
    blob_nospace = re.sub(r"\s+", "", blob)
    digits_blob = _digits(blob)

    substr_markers = [
        "yacine",
        "yassin",
        "salah",
        "bensaleh",
        "ben-saleh",
        "tourcoing",
        "tunis",
        "03-11-1991",
        "11/03/91",
        "chr-lil-00239871",
        "metformine",
        "sertraline",
        "harmonie",
        "delphine",
        "caron",
        "lille",
        "tj-lille",
        "14/06/2024",
        "psstfrppxxx",
        "mastercard",
        "5326",
        "9087",
        "dataxpert",
        "y.bensalah91@gmail.com",
        "protonmail.com",
        "ybs-data.io",
        "user_ybs91",
        "yac!ne_dev",
        "nadia",
        "leila",
        "lila",
        "adam",
        "eu-rgpd-test-009x",
        "98fr76x12345",
        "bsyac91110359",
        "12ab34567",
        "hm-778291-az",
        "caron-def-91-ybs",
        "550e8400",
    ]
    for m in substr_markers:
        ok = m in blob or m in blob_nospace
        assert ok, f"Marqueur texte manquant (corpus expert) : {m!r}\nspans={spans!r}"

    assert "0744912388" in digits_blob or "33744912388" in digits_blob, (
        f"Téléphone mobile manquant.\ndigits={digits_blob!r}\nspans={spans!r}"
    )
    for d in (
        "191115998765432",
        "1420041010050500013",
        "113",
        "51158203112",
        "18522010133",
        "192168145",
        "4850",
        "446655440000",
    ):
        assert d in digits_blob, f"Marqueur numérique manquant : {d!r}\ndigits={digits_blob!r}\nspans={spans!r}"

    assert (
        "fr1420041010050500013m02606" in blob_nospace
        or "fr142004101005050001302606" in blob_nospace
        or "1420041010050500013" in blob_nospace
    ), f"IBAN FR14 … peu détecté.\nblob_nospace={blob_nospace[:280]!r}…\nspans={spans!r}"
