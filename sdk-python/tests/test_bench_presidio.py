# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Benchmarks comparatifs AEGIS vs Presidio (optionnel, `pip install presidio-analyzer`)."""

from __future__ import annotations

import time

import pytest

pytest.importorskip("aegis._native")
from aegis import AegisEngine

pytest.importorskip("presidio_analyzer", reason="presidio-analyzer non installé")
from presidio_analyzer import AnalyzerEngine as PresidioAnalyzer  # type: ignore[import-not-found]

SAMPLES = [
    "Contact jane.doe@acme.fr ou +33 6 12 34 56 78",
    "IBAN FR76 3000 6000 0112 3456 7890 189",
    "Pas de PII dans cette phrase.",
    "Email: admin@example.org et https://zokastech.fr/page",
]


@pytest.mark.presidio
def test_both_engines_run_on_same_corpus() -> None:
    aegis = AegisEngine()
    presidio = PresidioAnalyzer()
    for text in SAMPLES:
        ae = aegis.analyze(text, score_threshold=0.25)
        pe = presidio.analyze(text=text, language="en")
        assert isinstance(ae, list) and isinstance(pe, list)


@pytest.mark.presidio
def test_wall_clock_comparison() -> None:
    """Même textes, métrique : temps total (ordre de grandeur)."""
    aegis = AegisEngine()
    presidio = PresidioAnalyzer()
    t0 = time.perf_counter()
    for t in SAMPLES:
        aegis.analyze(t, score_threshold=0.25)
    dt_a = time.perf_counter() - t0
    t1 = time.perf_counter()
    for t in SAMPLES:
        presidio.analyze(text=t, language="en")
    dt_p = time.perf_counter() - t1
    assert dt_a < 120.0 and dt_p < 120.0
