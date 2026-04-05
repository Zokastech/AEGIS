# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""Pytest loads this automatically: put `training/` on sys.path."""

from __future__ import annotations

import sys
from pathlib import Path

_TRAINING = Path(__file__).resolve().parents[1]
if str(_TRAINING) not in sys.path:
    sys.path.insert(0, str(_TRAINING))
