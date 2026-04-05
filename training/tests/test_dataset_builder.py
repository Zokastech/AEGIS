# AEGIS — zokastech.fr — Apache 2.0 / MIT
from __future__ import annotations

from dataset_builder import LABELS, build_dataset


def test_labels_include_o_and_bio_style():
    assert "O" in LABELS
    assert any(x.startswith("B-") for x in LABELS)


def test_build_dataset_small_split_sums_to_n():
    n = 20
    ds = build_dataset(n, seed=1)
    assert len(ds["train"]) + len(ds["validation"]) == n
    row = ds["train"][0]
    assert "tokens" in row and "ner_tags" in row
    assert len(row["tokens"]) == len(row["ner_tags"])
