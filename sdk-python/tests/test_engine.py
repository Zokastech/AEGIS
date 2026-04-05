# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Tests unitaires du SDK (nécessite l'extension compilée : maturin develop)."""

from __future__ import annotations

import pytest

pytest.importorskip("aegis._native", reason="maturin develop requis")
from aegis import AegisEngine, Entity  # noqa: E402


def test_analyze_finds_email() -> None:
    with AegisEngine() as engine:
        ents = engine.analyze("write to x@y.co please", score_threshold=0.2)
    assert isinstance(ents, list)
    types = {e.entity_type for e in ents}
    assert "EMAIL" in types


def test_analyze_batch() -> None:
    with AegisEngine() as engine:
        out = engine.analyze_batch(["a@b.co", "no pii here"])
    assert len(out) == 2
    assert any(e.entity_type == "EMAIL" for e in out[0])


def test_anonymize_redact() -> None:
    text = "mail x@y.co end"
    with AegisEngine() as engine:
        r = engine.anonymize(
            text,
            operators={"EMAIL": {"operator_type": "redact", "params": {}}},
        )
    assert "x@y.co" not in r.text
    assert len(r.text) > 0


def test_context_manager_closes() -> None:
    eng = AegisEngine()
    with eng:
        eng.analyze("test")
    with pytest.raises(RuntimeError, match="fermé"):
        eng.analyze("x")


def test_entity_python_constructor() -> None:
    e = Entity(
        "EMAIL",
        0,
        5,
        "a@b.c",
        0.99,
        "unit",
        {},
    )
    assert e.entity_type == "EMAIL"


def test_analyze_full() -> None:
    with AegisEngine() as engine:
        r = engine.analyze_full("z@w.fr", score_threshold=0.2)
    assert r.text_length > 0
    assert isinstance(r.entities, list)
