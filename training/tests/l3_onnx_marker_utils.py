# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""Seuils de couverture des marqueurs pour les tests ONNX L3 (régression corpus / lettre)."""

from __future__ import annotations

import os
from typing import List, Tuple, Union

# Marqueur = chaîne ou tuple de synonymes (un seul suffit pour compter un succès).
Marker = Union[str, Tuple[str, ...]]

DEFAULT_MIN_MARKER_PERCENT = 95


def effective_min_marker_percent() -> int:
    """Pourcentage minimum de marqueurs textuels à retrouver dans les spans (0–100)."""
    if os.environ.get("AEGIS_ONNX_STRICT_MARKERS", "").lower() in ("1", "true", "yes"):
        return 100
    raw = os.environ.get("AEGIS_ONNX_MIN_MARKER_PERCENT", str(DEFAULT_MIN_MARKER_PERCENT)).strip()
    try:
        p = int(raw)
    except ValueError:
        p = DEFAULT_MIN_MARKER_PERCENT
    return max(0, min(100, p))


def min_hits_required(num_markers: int, percent: int) -> int:
    """Nombre minimum de succès pour atteindre au moins ``percent`` % (arrondi supérieur)."""
    if num_markers <= 0:
        return 0
    if percent >= 100:
        return num_markers
    if percent <= 0:
        return 0
    return (num_markers * percent + 99) // 100


def marker_hit(m: Marker, blob: str, blob_nospace: str) -> bool:
    if isinstance(m, str):
        return m in blob or m in blob_nospace
    return any(marker_hit(s, blob, blob_nospace) for s in m)


def count_marker_hits(markers: List[Marker], blob: str, blob_nospace: str) -> Tuple[int, List[str]]:
    """Retourne (nombre de succès, libellés des marqueurs manquants pour le message d’erreur)."""
    missing: List[str] = []
    hits = 0
    for m in markers:
        if marker_hit(m, blob, blob_nospace):
            hits += 1
        else:
            label = m if isinstance(m, str) else " | ".join(m)
            missing.append(label)
    return hits, missing
