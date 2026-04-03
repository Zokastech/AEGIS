#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Kafka consumer: read JSON messages (text field `payload`), send each text
to /v1/analyze and log entity counts — handy for event buses carrying PII.

Modes:
  (default)  Without --kafka: in-memory synthetic stream (no broker).
  --kafka    Real Kafka: KAFKA_BOOTSTRAP_SERVERS, KAFKA_TOPIC, KAFKA_GROUP_ID

Requirements: pip install kafka-python requests
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from typing import Any

import requests

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")


def analyze_one(text: str) -> dict[str, Any]:
    """Call the gateway; in demo mode without gateway, fall back to heuristic counts."""
    try:
        r = requests.post(f"{BASE}/v1/analyze", json={"text": text}, timeout=30)
        r.raise_for_status()
        out = r.json().get("result", r.json())
        if isinstance(out, str):
            out = json.loads(out)
        return out if isinstance(out, dict) else {}
    except requests.RequestException:
        # Allows `python kafka_consumer.py` without Docker (indicative only).
        fake = 1 if ("@" in text or any(c.isdigit() for c in text)) else 0
        return {"entities": [None] * fake}


def count_entities(result: dict[str, Any]) -> int:
    ents = result.get("entities", [])
    return len(ents) if isinstance(ents, list) else 0


def synthetic_messages() -> list[bytes]:
    """Messages JSON UTF-8 comme un producteur Kafka les enverrait."""
    payloads = [
        {"event": "signup", "payload": "User bob@example.com registered."},
        {"event": "call", "payload": "Callback +33 7 98 76 54 32 requested."},
        {"event": "note", "payload": "No PII in this line."},
    ]
    return [json.dumps(p).encode("utf-8") for p in payloads]


def run_demo() -> None:
    print("Demo mode (no Kafka broker)")
    for raw in synthetic_messages():
        data = json.loads(raw.decode())
        text = data.get("payload", "")
        n = count_entities(analyze_one(text))
        print(f"topic=demo key={data.get('event')} entity_hits={n}")


def run_kafka(bootstrap: str, topic: str, group: str) -> None:
    from kafka import KafkaConsumer

    consumer = KafkaConsumer(
        topic,
        bootstrap_servers=bootstrap.split(","),
        group_id=group,
        enable_auto_commit=True,
        auto_offset_reset="earliest",
        value_deserializer=lambda b: b.decode("utf-8", errors="replace"),
    )
    print(f"Listening {topic} @ {bootstrap} …")
    for msg in consumer:
        try:
            data = json.loads(msg.value)
            text = data.get("payload", msg.value)
            n = count_entities(analyze_one(text))
            print(
                f"partition={msg.partition} offset={msg.offset} entity_hits={n} sample={text[:80]!r}"
            )
        except Exception as exc:  # noqa: BLE001
            print("skip message:", exc, file=sys.stderr)
        time.sleep(0.01)


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument(
        "--kafka",
        action="store_true",
        help="Use a real Kafka cluster (otherwise synthetic in-process messages)",
    )
    args = p.parse_args()

    if args.kafka:
        bootstrap = os.environ.get("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092")
        topic = os.environ.get("KAFKA_TOPIC", "aegis.demo")
        group = os.environ.get("KAFKA_GROUP_ID", "aegis-pii-scanner")
        run_kafka(bootstrap, topic, group)
        return

    run_demo()


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as exc:
        print("AEGIS HTTP error:", exc, file=sys.stderr)
        sys.exit(1)
