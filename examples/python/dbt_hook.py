#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
dbt-style hook: scan SQL (compiled or inline) for obvious PII patterns
**before** promoting to a warehouse — complements dbt tests (not a dbt-core replacement).

Usage:
  python dbt_hook.py --synthetic
  python dbt_hook.py --compiled-dir path/to/target/run/my_project

dbt integration: run this from Makefile / CI after `dbt compile` or `dbt run`,
or from an `on-run-end` macro that calls an external operator.

Requirements: requests (for --live-aegis, which sends snippets to the gateway).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

import requests

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")

# Synthetic SQL: mix of benign / suspicious (fake data).
SYNTHETIC_SQL = """
-- models/staging/stg_users.sql (synthetic)
SELECT
  id,
  email AS user_email,
  phone
FROM {{ source('raw', 'users') }}
WHERE email LIKE '%@example.com'
  OR phone = '+33 6 12 34 56 78'
"""


def heuristic_hits(sql: str) -> list[str]:
    """Fast, local: email and phone-like patterns."""
    hits: list[str] = []
    if re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", sql):
        hits.append("EMAIL_PATTERN")
    if re.search(r"\+?\d[\d\s.\-]{8,}\d", sql):
        hits.append("PHONE_PATTERN")
    return hits


def aegis_analyze_snippet(text: str) -> None:
    """Optional: send a snippet (e.g. comment) to the AEGIS engine."""
    r = requests.post(f"{BASE}/v1/analyze", json={"text": text[:8000]}, timeout=30)
    r.raise_for_status()
    body = r.json()
    print("  aegis:", json.dumps(body)[:400])


def scan_sql(name: str, sql: str, live: bool) -> bool:
    h = heuristic_hits(sql)
    print(f"== {name} ==")
    if not h:
        print("  no heuristic PII tokens in SQL literals")
        return True
    print("  heuristic flags:", ", ".join(h))
    if live:
        try:
            aegis_analyze_snippet(sql)
        except requests.RequestException as exc:
            print("  AEGIS unreachable:", exc, file=sys.stderr)
            return False
    return True


def load_compiled_files(root: Path) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for path in root.rglob("*.sql"):
        try:
            out.append((str(path), path.read_text(encoding="utf-8", errors="replace")))
        except OSError as exc:
            print("skip", path, exc, file=sys.stderr)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Scan SQL for obvious PII literals")
    ap.add_argument("--synthetic", action="store_true", help="Scan built-in synthetic SQL")
    ap.add_argument("--compiled-dir", type=Path, help="Directory of compiled SQL (dbt target/run/...)")
    ap.add_argument("--live-aegis", action="store_true", help="Also POST snippets to AEGIS")
    args = ap.parse_args()

    files: list[tuple[str, str]] = []
    if args.synthetic:
        files.append(("synthetic/stg_users.sql", SYNTHETIC_SQL))
    if args.compiled_dir:
        files.extend(load_compiled_files(args.compiled_dir))

    if not files:
        ap.print_help()
        print("\nProvide --synthetic and/or --compiled-dir", file=sys.stderr)
        sys.exit(2)

    ok = True
    for name, sql in files:
        if not scan_sql(name, sql, args.live_aegis):
            ok = False

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
