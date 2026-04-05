# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Traitement par lots (fichier volumineux) avec fenêtre de lignes."""

from __future__ import annotations

from pathlib import Path
from typing import Iterable, List

from aegis import AegisEngine


def iter_lines(path: Path, chunk_size: int = 500) -> Iterable[List[str]]:
    batch: List[str] = []
    with path.open(encoding="utf-8", errors="replace") as f:
        for line in f:
            batch.append(line.rstrip("\n"))
            if len(batch) >= chunk_size:
                yield batch
                batch = []
        if batch:
            yield batch


def main() -> None:
    # Remplacez par un vrai fichier log / export CSV texte
    demo = Path(__file__).resolve().parent / "_demo_batch.txt"
    demo.write_text("a@b.co\nrien\ncontact c@d.org\n", encoding="utf-8")
    with AegisEngine() as engine:
        for chunk in iter_lines(demo, chunk_size=2):
            results = engine.analyze_batch(chunk)
            for line, ents in zip(chunk, results):
                if ents:
                    print(line[:60], "->", len(ents), "entité(s)")


if __name__ == "__main__":
    main()
