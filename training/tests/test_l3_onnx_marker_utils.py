# AEGIS — zokastech.fr — Apache 2.0 / MIT
from __future__ import annotations

from l3_onnx_marker_utils import count_marker_hits, marker_hit, min_hits_required


def test_min_hits_required_ceil():
    assert min_hits_required(39, 95) == 38
    assert min_hits_required(17, 95) == 17
    assert min_hits_required(20, 95) == 19
    assert min_hits_required(100, 95) == 95
    assert min_hits_required(10, 100) == 10


def test_marker_hit_synonym_tuple():
    blob = "foo data solutions bar".lower()
    assert marker_hit(("km data solutions", "data solutions"), blob, "")


def test_count_marker_hits():
    blob = "a b c".lower()
    hits, miss = count_marker_hits(["a", "x", "b"], blob, "")
    assert hits == 2
    assert miss == ["x"]
