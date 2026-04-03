#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Lance le benchmark datasets/benchmark_vs_presidio.py (même dataset, métriques alignées).
Utilise la même machine ; sortie HTML sous datasets/reports/ (référencée par le rapport perf).
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DATASETS = ROOT / "datasets"
SCRIPT = DATASETS / "benchmark_vs_presidio.py"
AEGIS = ROOT / "target" / "release" / "aegis"
OUT = DATASETS / "reports" / "benchmark_report.html"


def main() -> int:
    if os.environ.get("SKIP_PRESIDIO"):
        print("SKIP_PRESIDIO=1 — comparaison Presidio ignorée.")
        return 0
    if not SCRIPT.is_file():
        print("Script datasets manquant.")
        return 0
    if not AEGIS.is_file():
        print("Compilez : cargo build -p aegis-cli --release")
        return 0
    OUT.parent.mkdir(parents=True, exist_ok=True)
    limit = os.environ.get("BENCH_PRESIDIO_LIMIT", "300")
    cmd = [
        sys.executable,
        str(SCRIPT),
        "--aegis-bin",
        str(AEGIS),
        "--output",
        str(OUT),
        "--limit",
        limit,
    ]
    if os.environ.get("SKIP_PRESIDIO_ENGINE"):
        cmd.append("--skip-presidio")
    print("Exécution :", " ".join(cmd))
    subprocess.run(cmd, cwd=str(DATASETS), check=False)
    print(f"Rapport : {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
