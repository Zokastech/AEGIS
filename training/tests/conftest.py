# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""Pytest loads this automatically: put `training/` on sys.path."""

from __future__ import annotations

import sys
from pathlib import Path

_TRAINING = Path(__file__).resolve().parents[1]
if str(_TRAINING) not in sys.path:
    sys.path.insert(0, str(_TRAINING))


def pytest_configure(config):
    # ONNX Runtime (SWIG) : bruit DeprecationWarning sous Python 3.12+ à l’import.
    for msg in (
        "builtin type SwigPyPacked has no __module__ attribute",
        "builtin type SwigPyObject has no __module__ attribute",
        "builtin type swigvarlink has no __module__ attribute",
    ):
        config.addinivalue_line(
            "filterwarnings",
            f"ignore:{msg}:DeprecationWarning",
        )
