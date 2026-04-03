#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
PII scan with PySpark: each partition calls the AEGIS gateway (/v1/analyze/batch)
on row batches — fits a data lake already loaded as a DataFrame.

Requirements:
  - Java + PySpark (`pip install pyspark`)
  - AEGIS gateway reachable from executors (same network / internal URL in cluster).

Env:
  AEGIS_BASE_URL   (default http://127.0.0.1:8080)
  SPARK_DEMO_ROWS  (default 5000; use 100000 for local stress)
"""

from __future__ import annotations

import json
import os
import sys
from typing import Iterator

import requests
from pyspark.sql import Row, SparkSession
from pyspark.sql import functions as F
from pyspark.sql.types import IntegerType, StringType, StructField, StructType

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")
ROWS = int(os.environ.get("SPARK_DEMO_ROWS", "5000"))
PARTITION_BATCH = int(os.environ.get("AEGIS_PARTITION_BATCH", "50"))


def _analyze_batch_http(texts: list[str]) -> list[int]:
    """Return entity count per row (heuristic)."""
    if not texts:
        return []
    r = requests.post(
        f"{BASE}/v1/analyze/batch",
        json={"texts": texts, "page": 1, "page_size": min(100, len(texts))},
        timeout=120,
    )
    r.raise_for_status()
    items = r.json().get("items", [])
    counts: list[int] = []
    for raw in items:
        if isinstance(raw, (bytes, str)):
            obj = json.loads(raw) if isinstance(raw, str) else json.loads(raw.decode())
        else:
            obj = raw
        inner = obj.get("result", obj)
        if isinstance(inner, str):
            inner = json.loads(inner)
        ents = inner.get("entities", []) if isinstance(inner, dict) else []
        counts.append(len(ents) if isinstance(ents, list) else 0)
    return counts


def scan_partition(rows: Iterator) -> Iterator:
    """
    Spark Row iterator → HTTP call per sub-batch.
    In cluster, use the internal Kubernetes service URL (ClusterIP), not localhost.
    """
    buf: list[str] = []
    keys: list[int] = []
    for row in rows:
        keys.append(row["id"])
        buf.append(row["note"])
        if len(buf) >= PARTITION_BATCH:
            counts = _analyze_batch_http(buf)
            for i, k in enumerate(keys):
                yield (k, counts[i] if i < len(counts) else 0)
            buf, keys = [], []
    if buf:
        counts = _analyze_batch_http(buf)
        for i, k in enumerate(keys):
            yield (k, counts[i] if i < len(counts) else 0)


def main() -> None:
    spark = SparkSession.builder.appName("aegis-pii-scan").master("local[*]").getOrCreate()
    spark.sparkContext.setLogLevel("WARN")

    # Distributed synthetic data
    data = [
        (i, f"user_{i}@demo.example.org +33 6 {i % 10:01d} 00 00 00 0{i % 9}")
        for i in range(ROWS)
    ]
    schema = StructType(
        [
            StructField("id", IntegerType(), False),
            StructField("note", StringType(), False),
        ]
    )
    df = spark.createDataFrame(data, schema)
    df = df.repartition(max(4, spark.sparkContext.defaultParallelism))

    def partition_fn(iterator: Iterator[Row]) -> Iterator:
        return scan_partition(iterator)

    counts_rdd = df.rdd.mapPartitions(partition_fn)
    out_schema = StructType(
        [
            StructField("id", IntegerType(), False),
            StructField("pii_hits", IntegerType(), False),
        ]
    )
    result = spark.createDataFrame(counts_rdd, schema=out_schema)
    total = result.agg(F.sum("pii_hits")).collect()[0][0] or 0
    print(f"Rows scanned: {ROWS}, approximate entity hit sum: {total}")
    result.show(5)


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as exc:
        print("HTTP error (executors must reach AEGIS_BASE_URL):", exc, file=sys.stderr)
        sys.exit(1)
