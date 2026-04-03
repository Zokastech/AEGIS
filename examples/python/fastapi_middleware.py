#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Middleware FastAPI : pour chaque POST avec corps JSON, envoie une copie textuelle à la passerelle AEGIS
(moteur Rust réel) pour journaliser une version **sans PII**.

Lancer :
  AEGIS_BASE_URL=http://127.0.0.1:8080 uvicorn fastapi_middleware:app --reload --port 9090

Avec clé API :
  AEGIS_API_KEY=... AEGIS_BASE_URL=... uvicorn ...

Tester :
  curl -s -X POST http://127.0.0.1:9090/echo -H "Content-Type: application/json" \\
    -d '{"message":"Reach alice@example.com at +33123456789"}'
"""

from __future__ import annotations

import json
import logging
import os
from typing import Callable

import requests
from fastapi import FastAPI, Request
from starlette.middleware.base import BaseHTTPMiddleware

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")
LOG = logging.getLogger("aegis.demo")
logging.basicConfig(level=logging.INFO)

ANON_CFG = json.dumps(
    {
        "operators_by_entity": {
            "EMAIL": {"operator_type": "mask", "params": {"keep_last": "3", "mask_char": "*"}},
            "PHONE": {"operator_type": "redact", "params": {"replacement": "[PHONE]"}},
        },
        "default_operator": {"operator_type": "replace", "params": {}},
    }
)


def anonymize_for_log(text: str) -> str:
    if not text:
        return text
    r = _SESSION.post(
        f"{BASE}/v1/anonymize",
        json={"text": text, "config_json": ANON_CFG},
        timeout=15,
    )
    r.raise_for_status()
    payload = r.json()
    raw = payload.get("result", payload)
    if isinstance(raw, str):
        raw = json.loads(raw)
    anon = raw.get("anonymized") or raw
    if isinstance(anon, dict):
        return anon.get("text", json.dumps(anon)[:2000])
    return str(anon)


class AegisLogSanitizerMiddleware(BaseHTTPMiddleware):
    """Re-read request body, log an anonymized copy, then re-inject the body for the route."""

    async def dispatch(self, request: Request, call_next: Callable):
        if request.method == "POST":
            body = await request.body()

            async def receive():
                return {"type": "http.request", "body": body, "more_body": False}

            request = Request(request.scope, receive)
            try:
                txt = body.decode("utf-8", errors="replace")
                safe = anonymize_for_log(txt)
                LOG.info("request_body_sanitized=%s", safe[:800])
            except Exception as exc:  # noqa: BLE001
                LOG.warning("aegis sanitizer skipped: %s", exc)

        return await call_next(request)


app = FastAPI(title="AEGIS FastAPI middleware demo")
app.add_middleware(AegisLogSanitizerMiddleware)


@app.post("/echo")
async def echo(payload: dict) -> dict:
    return payload
