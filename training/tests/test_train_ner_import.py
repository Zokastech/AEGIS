# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""Smoke: training script modules import (no full train run)."""

from __future__ import annotations

from dataset_builder import ID2LABEL, LABEL2ID, LABELS


def test_label_maps_consistent():
    assert len(LABELS) == len(LABEL2ID) == len(ID2LABEL)
    for i, name in ID2LABEL.items():
        assert LABEL2ID[name] == i
