# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""
Extension côté Python : détections complémentaires (regex) fusionnées avec AEGIS.

Les recognizers natifs restent dans Rust (YAML / binaire). Ici on illustre un
pipeline hybride pour des codes métiers non couverts par le moteur.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import List

from aegis import AegisEngine, Entity


@dataclass
class RegexHit:
    start: int
    end: int
    text: str


def find_internal_codes(text: str, pattern: str = r"\bINT-[0-9]{4,8}\b") -> List[RegexHit]:
    return [RegexHit(m.start(), m.end(), m.group()) for m in re.finditer(pattern, text)]


def merge_with_aegis(text: str, engine: AegisEngine) -> List[Entity]:
    rust_entities = engine.analyze(text)
    extra = find_internal_codes(text)
    merged: List[Entity] = list(rust_entities)
    for h in extra:
        merged.append(
            Entity(
                "CUSTOM:INTERNAL_REF",
                h.start,
                h.end,
                h.text,
                1.0,
                "python_regex",
                {},
            )
        )
    merged.sort(key=lambda e: e.start)
    return merged


def main() -> None:
    text = "Ticket INT-12345 assigné à support@corp.eu"
    with AegisEngine() as engine:
        ents = merge_with_aegis(text, engine)
    for e in ents:
        print(e.entity_type, e.text)


if __name__ == "__main__":
    main()
