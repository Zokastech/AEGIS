# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""IOB2 → spans — aucune dépendance lourde (torch / onnxruntime)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, Sequence


@dataclass(frozen=True)
class Span:
    start: int
    end: int
    etype: str


def tags_to_spans(tags: Sequence[str]) -> List[Span]:
    spans: List[Span] = []
    i = 0
    n = len(tags)
    while i < n:
        t = tags[i]
        if t == "O" or not t.startswith("B-"):
            i += 1
            continue
        etype = t[2:]
        j = i + 1
        while j < n and tags[j] == f"I-{etype}":
            j += 1
        spans.append(Span(i, j, etype))
        i = j
    return spans


def refine_iob(tags: List[str]) -> List[str]:
    out = list(tags)
    for i in range(len(out)):
        if out[i].startswith("I-"):
            et = out[i][2:]
            prev = out[i - 1] if i else "O"
            if prev not in (f"B-{et}", f"I-{et}"):
                out[i] = f"B-{et}"
    return out
