#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Pandas pipeline: build a synthetic CSV (~100k rows), send texts in batches to /v1/analyze/batch.

Batches limit HTTP body size and keep load on the engine reasonable.
Requirements: AEGIS gateway + `pip install pandas requests`.
"""

from __future__ import annotations

import json
import os
import sys
import time

import pandas as pd
import requests

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")
ROWS = int(os.environ.get("AEGIS_DEMO_ROWS", "100000"))
BATCH = int(os.environ.get("AEGIS_BATCH_SIZE", "80"))


def synth_frame(n: int) -> pd.DataFrame:
    """Build n rows with fake emails / phones (repetitive but realistic patterns)."""
    rng = range(n)
    return pd.DataFrame(
        {
            "id": rng,
            "note": [
                f"user_{i}@customer{i % 1000}.example.com called +33 6 {i % 10:01d} {i % 100:02d} {i % 100:02d} {i % 100:02d} {i % 100:02d}"
                for i in rng
            ],
        }
    )


def analyze_batch(session: requests.Session, texts: list[str]) -> list[dict]:
    """Call /v1/analyze/batch (one page)."""
    r = session.post(
        f"{BASE}/v1/analyze/batch",
        json={"texts": texts, "page": 1, "page_size": len(texts)},
        timeout=120,
    )
    r.raise_for_status()
    data = r.json()
    return list(data.get("items", []))


def main() -> None:
    print(f"Building synthetic DataFrame ({ROWS} rows)…")
    df = synth_frame(ROWS)

    # Optional: write a CSV for local inspection
    out_csv = os.environ.get("AEGIS_CSV_OUT")
    if out_csv:
        df.to_csv(out_csv, index=False)
        print("Wrote", out_csv)

    session = requests.Session()
    t0 = time.perf_counter()
    total_entities = 0
    processed = 0

    for start in range(0, len(df), BATCH):
        chunk = df["note"].iloc[start : start + BATCH].tolist()
        items = analyze_batch(session, chunk)
        for raw in items:
            obj = json.loads(raw) if isinstance(raw, str) else raw
            if not isinstance(obj, dict):
                continue
            inner = obj.get("result", obj)
            if isinstance(inner, str):
                inner = json.loads(inner)
            ents = inner.get("entities") if isinstance(inner, dict) else None
            if ents is None:
                ents = obj.get("entities", [])
            total_entities += len(ents) if isinstance(ents, list) else 0
        processed += len(chunk)
        if processed % (BATCH * 50) == 0:
            print(f"  … {processed} rows sent")

    elapsed = time.perf_counter() - t0
    print(f"Done: {processed} rows in {elapsed:.1f}s (~{processed / elapsed:.0f} rows/s)")
    print(f"Approx. entity hits aggregated (heuristic): {total_entities}")


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as exc:
        print("HTTP error:", exc, file=sys.stderr)
        sys.exit(1)
