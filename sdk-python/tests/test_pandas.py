# AEGIS — zokastech.fr — Apache 2.0 / MIT

from __future__ import annotations

import pytest

pytest.importorskip("aegis._native")
pytest.importorskip("pandas")
import pandas as pd

from aegis import AegisEngine


def test_analyze_dataframe() -> None:
    df = pd.DataFrame({"col": ["a@b.co", "rien"]})
    with AegisEngine() as engine:
        out = engine.analyze_dataframe(df, columns=["col"])
    assert len(out) == 2
    assert all(isinstance(x, list) for x in out["col"])
