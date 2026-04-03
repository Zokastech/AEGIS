#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Quickstart: call the AEGIS gateway (real Rust engine) — analyze then anonymize.

The sample text below is **fictional** (demo); responses come from the engine behind the gateway.

Requirements: reachable gateway.
- `docker compose up`: HTTPS https://127.0.0.1:8443 (self-signed cert) — default below + verify=False.
- Dev profile: HTTP http://127.0.0.1:8080 — set AEGIS_BASE_URL=...
- Auth: AEGIS_API_KEY (+ optional AEGIS_API_KEY_HEADER) unless the gateway runs with disable_auth.
AEGIS_TLS_VERIFY=1 for HTTPS with a trusted cert (else verify=False locally).
"""

from __future__ import annotations

import json
import os
import sys

import requests
import urllib3

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")

# Sample text only — do not paste real personal data here.
TEXT = "Contact: jane.doe@zokastech.com or +33 6 12 34 56 78 — démo."


def _tls_kw() -> dict:
    if not BASE.startswith("https://"):
        return {}
    verify = os.environ.get("AEGIS_TLS_VERIFY", "0").lower() in ("1", "true", "yes")
    return {"verify": verify}


def _apply_api_key(session: requests.Session) -> None:
    key = os.environ.get("AEGIS_API_KEY", "").strip()
    if not key:
        return
    header = os.environ.get("AEGIS_API_KEY_HEADER", "X-API-Key").strip() or "X-API-Key"
    session.headers[header] = key


def main() -> None:
    s = requests.Session()
    _apply_api_key(s)
    tls = _tls_kw()

    analysis_cfg = json.dumps(
        {
            "language": "fr",
            "pipeline_level": 2,
            "score_threshold": 0.5,
        }
    )
    r = s.post(
        f"{BASE}/v1/analyze",
        json={"text": TEXT, "analysis_config_json": analysis_cfg},
        timeout=60,
        **tls,
    )
    r.raise_for_status()
    print("analyze:", json.dumps(r.json(), indent=2)[:1200])

    cfg = {
        "operators_by_entity": {
            "EMAIL": {"operator_type": "mask", "params": {"keep_last": "4", "mask_char": "*"}},
            "PHONE": {"operator_type": "redact", "params": {"replacement": "[PHONE]"}},
        }
    }
    r2 = s.post(
        f"{BASE}/v1/anonymize",
        json={"text": TEXT, "config_json": json.dumps(cfg)},
        timeout=60,
        **tls,
    )
    r2.raise_for_status()
    print("anonymize:", json.dumps(r2.json(), indent=2)[:1200])


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as exc:
        print("HTTP error — is the gateway up? Correct AEGIS_BASE_URL / API key?", exc, file=sys.stderr)
        sys.exit(1)
