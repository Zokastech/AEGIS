# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Tests de non-régression sécurité du SDK Python (pyo3 ≥ 0.29)."""

from __future__ import annotations

import pytest

pytest.importorskip("aegis._native", reason="maturin develop requis")
from aegis import AegisEngine  # noqa: E402

KEY_HEX = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
PII = "contact jean.dupont@example.fr ou +33 6 12 34 56 78, IBAN FR7630006000011234567890189"


@pytest.mark.parametrize(
    "operator",
    [
        {"operator_type": "redact", "params": {}},
        {"operator_type": "replace", "params": {"new_value": "<EMAIL>"}},
        {"operator_type": "hash", "params": {"salt": "s3cr3t"}},
        {"operator_type": "encrypt", "params": {"key_hex": KEY_HEX}},
        {"operator_type": "pseudonymize", "params": {"salt": "s3cr3t"}},
    ],
)
def test_anonymize_never_leaks_original_email(operator: dict) -> None:
    with AegisEngine() as engine:
        r = engine.anonymize(PII, operators={"EMAIL": operator})
    assert "jean.dupont@example.fr" not in r.text


def test_encrypt_with_invalid_key_does_not_leak_plaintext() -> None:
    with AegisEngine() as engine:
        r = engine.anonymize(PII, operators={"EMAIL": {"operator_type": "encrypt", "params": {"key_hex": "zz"}}})
    assert "jean.dupont@example.fr" not in r.text


def test_encrypt_is_non_deterministic() -> None:
    op = {"EMAIL": {"operator_type": "encrypt", "params": {"key_hex": KEY_HEX}}}
    with AegisEngine() as engine:
        a = engine.anonymize(PII, operators=op).text
        b = engine.anonymize(PII, operators=op).text
    assert a != b, "même nonce AES-GCM réutilisé"


def test_invalid_operator_payload_raises() -> None:
    with AegisEngine() as engine:
        with pytest.raises(Exception):
            engine.anonymize(PII, operators={"EMAIL": object()})  # type: ignore[dict-item]


def test_missing_config_file_raises() -> None:
    with pytest.raises(Exception):
        AegisEngine(config_path="/nonexistent/aegis.yaml")


def test_large_and_unicode_inputs_do_not_crash() -> None:
    big = ("Élodie ✉ a@b.co 🚀 " * 5000)[:100_000]
    with AegisEngine() as engine:
        ents = engine.analyze(big, score_threshold=0.5)
        assert isinstance(ents, list)
        assert engine.analyze("", score_threshold=0.5) == []


def test_closed_engine_rejects_all_calls() -> None:
    eng = AegisEngine()
    eng.close()
    for call in (lambda: eng.analyze("x"), lambda: eng.anonymize("x"), lambda: eng.analyze_batch(["x"])):
        with pytest.raises(RuntimeError):
            call()
