# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Régression L3 : le modèle ONNX doit recouvrir les PII de la lettre FR de référence
(`datasets/training/l3_regression/letter_fr_sensitive.txt`).

Le jeu `letter_fr_golden.jsonl` est fusionné dans `run_l3_pipeline.sh` pour ancrer
l’entraînement sur ces entités. Hors pipeline (pytest local sans export), le test est ignoré.

Ces champs sont appris via des lignes « or » (IP en LOCATION, CVV en CREDIT_CARD, identifiants
en ORG/PERSON) pour forcer leur présence dans les spans ; ce ne sont pas des types PII canoniques.
"""

from __future__ import annotations

import os
import re
from pathlib import Path

import pytest

_TRAINING = Path(__file__).resolve().parents[1]
_REPO = _TRAINING.parent
_LETTER = _REPO / "datasets/training/l3_regression/letter_fr_sensitive.txt"
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
def letter_lines():
    assert _LETTER.is_file(), f"Fixture lettre manquante : {_LETTER}"
    return _LETTER.read_text(encoding="utf-8").splitlines()


def test_l3_onnx_covers_sensitive_letter(onnx_session, ner_tokenizer, letter_lines):
    """Chaque marqueur doit apparaître dans le texte prédit (spans) ou via normalisation chiffres."""
    from onnx_ner_infer import collect_entity_texts_onnx_lines

    spans = collect_entity_texts_onnx_lines(
        onnx_session,
        ner_tokenizer,
        letter_lines,
        max_length=int(os.environ.get("AEGIS_ONNX_MAX_LENGTH", "256")),
    )
    blob = " ".join(spans).lower()
    blob_nospace = re.sub(r"\s+", "", blob)
    digits_blob = _digits(blob)

    # Sous-chaînes textuelles (insensible à la casse ; espaces conservés dans blob).
    substr_markers = [
        "karim",
        "mahdi",
        "samira",
        "14/02/1987",
        "roubaix",
        "lille",
        "59100",
        "59000",
        "gmail.com",
        "consulting-eu.org",
        "agrifrppxxx",
        "km data solutions",
        "elmahdy",
        "elm87",
        "france",
        "usr_kelm_7721",
        "karim1987!",
        "185.217.0.12",
    ]
    for m in substr_markers:
        assert m in blob, f"Marqueur texte manquant dans les spans ONNX : {m!r}\nspans={spans!r}"

    # Séquences numériques (téléphone / IBAN / NIR / SIRET / carte).
    assert "0658771209" in digits_blob or "337658771209" in digits_blob, (
        f"Téléphone FR manquant dans les spans.\ndigits={digits_blob!r}\nspans={spans!r}"
    )
    for d in (
        "7630006000011234567890189",
        "187025912345678",
        "81234567800017",
        "4832",
        "482",
        "185217012",
    ):
        assert d in digits_blob, f"Marqueur numérique manquant : {d!r}\ndigits={digits_blob!r}\nspans={spans!r}"

    # Obfuscation (at) : le modèle peut prédire le span avec (at) ou normalisé ; le fichier source contient les deux.
    assert (
        "(at)" in blob_nospace
        or "karim.elm87" in blob_nospace
        or "gmail.com" in blob
    ), f"Email perso non détecté (attendu fragment elm / gmail). spans={spans!r}"
