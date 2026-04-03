#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Compare les métriques PII (Precision, Recall, F1, F2, latences, throughput)
entre AEGIS (CLI scan) et Microsoft Presidio, sur les JSONL du dossier datasets/.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

# ---------------------------------------------------------------------------
# Alignement des types Presidio → libellés AEGIS (config_key)
# ---------------------------------------------------------------------------
PRESIDIO_TO_AEGIS: dict[str, str] = {
    "EMAIL_ADDRESS": "EMAIL",
    "PHONE_NUMBER": "PHONE",
    "CREDIT_CARD": "CREDIT_CARD",
    "IBAN_CODE": "IBAN",
    "IP_ADDRESS": "IP_ADDRESS",
    "PERSON": "PERSON",
    "LOCATION": "LOCATION",
    "GPE": "LOCATION",
    "ORGANIZATION": "ORGANIZATION",
    "ORG": "ORGANIZATION",
    "DATE_TIME": "DATE",
    "DATE": "DATE",
    "URL": "URL",
    "NRP": "PERSON",
    "US_DRIVER_LICENSE": "DRIVER_LICENSE",
    "US_PASSPORT": "PASSPORT",
    "US_SSN": "SSN",
    "UK_NHS": "MEDICAL_RECORD",
    "MEDICAL_LICENSE": "MEDICAL_RECORD",
    "CRYPTO": "CRYPTO_WALLET",
    "UK_NINO": "NATIONAL_ID",
}


def normalize_aegis_entity_type(raw: Any) -> str:
    if raw is None:
        return "UNKNOWN"
    if isinstance(raw, str):
        return raw.upper().replace(" ", "_")
    if isinstance(raw, dict):
        if len(raw) == 1:
            k = next(iter(raw.keys()))
            return str(k).upper()
    return str(raw).upper()


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def iou_span(a: tuple[int, int], b: tuple[int, int]) -> float:
    s1, e1 = a
    s2, e2 = b
    inter = max(0, min(e1, e2) - max(s1, s2))
    if inter <= 0:
        return 0.0
    union = max(e1, e2) - min(s1, s2)
    return inter / union if union > 0 else 0.0


def types_match(gold_t: str, pred_t: str, strict: bool) -> bool:
    if gold_t == pred_t:
        return True
    if strict:
        return False
    aliases = {
        ("DATE", "DATE_TIME"),
        ("LOCATION", "GPE"),
        ("PERSON", "NRP"),
    }
    for x, y in aliases:
        if (gold_t == x and pred_t == y) or (gold_t == y and pred_t == x):
            return True
    return gold_t == pred_t


@dataclass
class MatchResult:
    tp: int
    fp: int
    fn: int


def match_entities(
    gold: list[dict[str, Any]],
    pred: list[dict[str, Any]],
    *,
    iou_min: float = 0.5,
    strict_types: bool = False,
) -> MatchResult:
    """Appariement glouton sur IoU + type."""
    used_pred: set[int] = set()
    tp = 0
    for g in gold:
        gs, ge = int(g["start"]), int(g["end"])
        gt = str(g["entity_type"]).upper()
        best_j = -1
        best_iou = 0.0
        for j, p in enumerate(pred):
            if j in used_pred:
                continue
            ps, pe = int(p["start"]), int(p["end"])
            pt = str(p["entity_type"]).upper()
            ov = iou_span((gs, ge), (ps, pe))
            if ov >= iou_min and types_match(gt, pt, strict_types) and ov > best_iou:
                best_iou = ov
                best_j = j
        if best_j >= 0:
            tp += 1
            used_pred.add(best_j)
    fp = len(pred) - len(used_pred)
    fn = len(gold) - tp
    return MatchResult(tp=tp, fp=fp, fn=fn)


def aggregate_metrics(results: list[MatchResult]) -> dict[str, float]:
    tp = sum(r.tp for r in results)
    fp = sum(r.fp for r in results)
    fn = sum(r.fn for r in results)
    prec = tp / (tp + fp) if (tp + fp) else 0.0
    rec = tp / (tp + fn) if (tp + fn) else 0.0
    f1 = 2 * prec * rec / (prec + rec) if (prec + rec) else 0.0
    b = 2.0
    b2 = b * b
    f2 = (1 + b2) * prec * rec / (b2 * prec + rec) if (b2 * prec + rec) else 0.0
    return {"precision": prec, "recall": rec, "f1": f1, "f2": f2, "tp": tp, "fp": fp, "fn": fn}


def percentile(xs: list[float], p: float) -> float:
    if not xs:
        return 0.0
    xs = sorted(xs)
    k = (len(xs) - 1) * p / 100.0
    f = int(k)
    c = min(f + 1, len(xs) - 1)
    return xs[f] + (xs[c] - xs[f]) * (k - f)


def find_aegis_binary(cli: str | None) -> str:
    if cli and Path(cli).is_file():
        return str(Path(cli).resolve())
    env = os.environ.get("AEGIS_CLI", "")
    if env and Path(env).is_file():
        return str(Path(env).resolve())
    root = Path(__file__).resolve().parent.parent
    release = root / "target" / "release" / "aegis"
    debug = root / "target" / "debug" / "aegis"
    if release.is_file():
        return str(release)
    if debug.is_file():
        return str(debug)
    return "aegis"


def run_aegis_batch(
    aegis_bin: str,
    work_dir: Path,
    records: list[dict[str, Any]],
    languages: str,
    score_threshold: float,
) -> tuple[dict[str, list[dict[str, Any]]], list[float], float]:
    """Écrit un fichier .txt par id, exécute `aegis scan work_dir`, parse JSON."""
    for r in records:
        rid = r["id"]
        (work_dir / f"{rid}.txt").write_text(r["text"], encoding="utf-8")
    cmd = [
        aegis_bin,
        "scan",
        str(work_dir),
        "--format",
        "json",
        "--language",
        languages,
        "--score-threshold",
        str(score_threshold),
    ]
    t0 = time.perf_counter()
    proc = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=600,
    )
    elapsed = time.perf_counter() - t0
    if proc.returncode != 0:
        raise RuntimeError(f"aegis scan failed: {proc.stderr[:2000]}")
    data = json.loads(proc.stdout)
    by_id: dict[str, list[dict[str, Any]]] = {}
    per_doc_ms: list[float] = []
    for item in data:
        path = item.get("path", "")
        rid = Path(path).stem
        analysis = item.get("analysis", {})
        ents_raw = analysis.get("entities", [])
        ents: list[dict[str, Any]] = []
        for e in ents_raw:
            ents.append(
                {
                    "entity_type": normalize_aegis_entity_type(e.get("entity_type")),
                    "start": int(e["start"]),
                    "end": int(e["end"]),
                    "text": e.get("text", ""),
                }
            )
        by_id[rid] = ents
        per_doc_ms.append(float(analysis.get("processing_time_ms", 0)))
    n = max(len(records), 1)
    amort = (elapsed * 1000) / n
    if not per_doc_ms or statistics.mean(per_doc_ms) < 1e-6:
        per_doc_ms = [amort] * len(records)
    else:
        while len(per_doc_ms) < len(records):
            per_doc_ms.append(amort)
    return by_id, per_doc_ms, elapsed


def presidio_entities(text: str, language: str) -> list[dict[str, Any]]:
    from presidio_analyzer import AnalyzerEngine

    if not hasattr(presidio_entities, "_engine"):
        presidio_entities._engine = AnalyzerEngine()  # type: ignore[attr-defined]
    eng = presidio_entities._engine  # type: ignore[attr-defined]
    lang = language if language in ("en", "fr", "de", "es", "it", "nl", "pl", "pt") else "en"
    try:
        results = eng.analyze(text=text, language=lang)
    except Exception:
        results = eng.analyze(text=text, language="en")
    out: list[dict[str, Any]] = []
    for r in results:
        et = PRESIDIO_TO_AEGIS.get(r.entity_type, r.entity_type.upper())
        out.append({"entity_type": et, "start": r.start, "end": r.end, "text": text[r.start : r.end]})
    return out


def eval_labeled(
    records: list[dict[str, Any]],
    predict: Callable[[dict[str, Any]], list[dict[str, Any]]],
) -> tuple[list[MatchResult], list[float]]:
    results: list[MatchResult] = []
    times: list[float] = []
    for r in records:
        gold = r.get("entities") or []
        t0 = time.perf_counter()
        pred = predict(r)
        times.append(time.perf_counter() - t0)
        results.append(match_entities(gold, pred))
    return results, times


def eval_false_positives(
    records: list[dict[str, Any]],
    predict: Callable[[dict[str, Any]], list[dict[str, Any]]],
) -> tuple[float, list[float]]:
    """Fraction de documents avec au moins une détection (à minimiser)."""
    flagged = 0
    times: list[float] = []
    for r in records:
        t0 = time.perf_counter()
        pred = predict(r)
        times.append(time.perf_counter() - t0)
        if len(pred) > 0:
            flagged += 1
    rate = flagged / len(records) if records else 0.0
    return rate, times


def write_html_report(
    path: Path,
    sections: dict[str, Any],
    chart_pngs: dict[str, bytes],
) -> None:
    parts = [
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>AEGIS vs Presidio — benchmark</title>",
        "<style>body{font-family:system-ui,sans-serif;margin:2rem;} table{border-collapse:collapse;} td,th{border:1px solid #ccc;padding:6px;} img{max-width:100%;}</style>",
        "</head><body>",
        "<h1>AEGIS — zokastech.fr — Benchmark PII</h1>",
    ]
    for name, metrics in sections.items():
        parts.append(f"<h2>{name}</h2>")
        if isinstance(metrics, dict) and "precision" in metrics:
            parts.append("<table><tr><th>Métrique</th><th>Valeur</th></tr>")
            for k, v in metrics.items():
                if isinstance(v, float):
                    parts.append(f"<tr><td>{k}</td><td>{v:.4f}</td></tr>")
                else:
                    parts.append(f"<tr><td>{k}</td><td>{v}</td></tr>")
            parts.append("</table>")
        elif isinstance(metrics, dict) and "false_positive_doc_rate" in metrics:
            parts.append("<table>")
            for k, v in metrics.items():
                parts.append(f"<tr><td>{k}</td><td>{v}</td></tr>")
            parts.append("</table>")
        else:
            parts.append(f"<pre>{json.dumps(metrics, indent=2, ensure_ascii=False)}</pre>")
    for title, png in chart_pngs.items():
        b64 = base64.b64encode(png).decode("ascii")
        parts.append(f"<h3>{title}</h3><img src='data:image/png;base64,{b64}' alt='{title}'/>")
    parts.append("</body></html>")
    path.write_text("\n".join(parts), encoding="utf-8")


def make_bar_chart(labels: list[str], aegis_vals: list[float], presidio_vals: list[float], title: str) -> bytes:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    x = range(len(labels))
    w = 0.35
    fig, ax = plt.subplots(figsize=(8, 4))
    ax.bar([i - w / 2 for i in x], aegis_vals, width=w, label="AEGIS")
    ax.bar([i + w / 2 for i in x], presidio_vals, width=w, label="Presidio")
    ax.set_xticks(list(x))
    ax.set_xticklabels(labels)
    ax.set_ylim(0, 1.05)
    ax.set_title(title)
    ax.legend()
    fig.tight_layout()
    buf = __import__("io").BytesIO()
    fig.savefig(buf, format="png", dpi=120)
    plt.close(fig)
    return buf.getvalue()


def main() -> int:
    ap = argparse.ArgumentParser(description="Benchmark AEGIS vs Presidio")
    ap.add_argument("--aegis-bin", default=None, help="Chemin binaire aegis (sinon target/release ou AEGIS_CLI)")
    ap.add_argument(
        "--datasets-dir",
        type=Path,
        default=Path(__file__).parent,
        help="Répertoire contenant generated/, false_positives/, recall_test/",
    )
    ap.add_argument("--output", type=Path, default=Path("reports/benchmark_report.html"))
    ap.add_argument("--limit", type=int, default=500, help="Max lignes par jeu (0 = tout)")
    ap.add_argument("--language-regex", default="en,fr,de,es,it,nl,pl,pt,ro,sv", help="Langues recognizers AEGIS")
    ap.add_argument("--score-threshold", type=float, default=0.35)
    ap.add_argument("--skip-presidio", action="store_true")
    args = ap.parse_args()

    ds = args.datasets_dir
    paths = {
        "synthetic": ds / "generated" / "synthetic_pii.jsonl",
        "recall": ds / "recall_test" / "cases.jsonl",
        "false_positives": ds / "false_positives" / "cases.jsonl",
    }

    if not paths["synthetic"].is_file():
        print(f"Note: {paths['synthetic']} absent — exécuter: python generate_dataset.py", file=sys.stderr)

    use_presidio = not args.skip_presidio
    if use_presidio:
        try:
            from presidio_analyzer import AnalyzerEngine as _PresidioEngine  # noqa: F401

            _ = _PresidioEngine
        except Exception as exc:
            print(f"Presidio indisponible ({exc}) — rapport AEGIS seul.", file=sys.stderr)
            use_presidio = False

    aegis_bin = find_aegis_binary(args.aegis_bin)
    if not shutil.which(aegis_bin) and not Path(aegis_bin).is_file():
        print(f"AEGIS binary not found: {aegis_bin}. Build with: cargo build -p aegis-cli --release", file=sys.stderr)
        return 1

    sections: dict[str, Any] = {}
    chart_pngs: dict[str, bytes] = {}

    # --- Synthetic ---
    if paths["synthetic"].is_file():
        syn = load_jsonl(paths["synthetic"])
        if args.limit > 0:
            syn = syn[: args.limit]
        tmp = Path(tempfile.mkdtemp(prefix="aegis-bench-"))
        try:
            by_id, batch_ms, wall_s = run_aegis_batch(
                aegis_bin, tmp, syn, args.language_regex, args.score_threshold
            )

            def pred_aegis(r: dict[str, Any]) -> list[dict[str, Any]]:
                return by_id.get(r["id"], [])

            mr_aegis, _ = eval_labeled(syn, pred_aegis)
            metrics_aegis = aggregate_metrics(mr_aegis)
            metrics_aegis["latency_p50_ms"] = percentile(batch_ms, 50)
            metrics_aegis["latency_p95_ms"] = percentile(batch_ms, 95)
            metrics_aegis["latency_p99_ms"] = percentile(batch_ms, 99)
            metrics_aegis["throughput_docs_per_s"] = len(syn) / max(wall_s, 1e-9)

            sections["synthetic — AEGIS"] = metrics_aegis

            if use_presidio:
                mr_pre, times_pre = eval_labeled(
                    syn, lambda r: presidio_entities(r["text"], r.get("language", "en"))
                )
                metrics_pre = aggregate_metrics(mr_pre)
                metrics_pre["latency_p50_ms"] = percentile(times_pre, 50)
                metrics_pre["latency_p95_ms"] = percentile(times_pre, 95)
                metrics_pre["latency_p99_ms"] = percentile(times_pre, 99)
                metrics_pre["throughput_docs_per_s"] = len(syn) / max(sum(times_pre), 1e-9)
                sections["synthetic — Presidio"] = metrics_pre
                chart_pngs["Synthétique P/R/F1"] = make_bar_chart(
                    ["Precision", "Recall", "F1", "F2"],
                    [
                        metrics_aegis["precision"],
                        metrics_aegis["recall"],
                        metrics_aegis["f1"],
                        metrics_aegis["f2"],
                    ],
                    [
                        metrics_pre["precision"],
                        metrics_pre["recall"],
                        metrics_pre["f1"],
                        metrics_pre["f2"],
                    ],
                    "Jeu synthétique (échantillon)",
                )
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    # --- Recall ---
    if paths["recall"].is_file():
        rec = load_jsonl(paths["recall"])
        if args.limit:
            rec = rec[: max(1, min(args.limit, 200))]
        tmp = Path(tempfile.mkdtemp(prefix="aegis-bench-rc-"))
        try:
            by_id, batch_ms, _ = run_aegis_batch(
                aegis_bin, tmp, rec, args.language_regex, args.score_threshold
            )

            def pred_aegis_r(r: dict[str, Any]) -> list[dict[str, Any]]:
                return by_id.get(r["id"], [])

            mr_aegis, _ = eval_labeled(rec, pred_aegis_r)
            sections["recall_test — AEGIS"] = aggregate_metrics(mr_aegis)
            if use_presidio:
                mr_pre, _ = eval_labeled(
                    rec, lambda r: presidio_entities(r["text"], r.get("language", "en"))
                )
                sections["recall_test — Presidio"] = aggregate_metrics(mr_pre)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    # --- False positives ---
    if paths["false_positives"].is_file():
        fp = load_jsonl(paths["false_positives"])
        if args.limit > 0:
            fp = fp[: max(1, min(args.limit, 100))]
        tmp = Path(tempfile.mkdtemp(prefix="aegis-bench-fp-"))
        try:
            by_id, _, _ = run_aegis_batch(
                aegis_bin, tmp, fp, args.language_regex, args.score_threshold
            )

            def pred_aegis_f(r: dict[str, Any]) -> list[dict[str, Any]]:
                return by_id.get(r["id"], [])

            rate_aegis, _ = eval_false_positives(fp, pred_aegis_f)
            sections["false_positives — AEGIS"] = {
                "false_positive_doc_rate": f"{rate_aegis:.2%}",
                "goal": "0% — aucune détection attendue",
            }
            if use_presidio:
                rate_pre, _ = eval_false_positives(
                    fp, lambda r: presidio_entities(r["text"], r.get("language", "en"))
                )
                sections["false_positives — Presidio"] = {
                    "false_positive_doc_rate": f"{rate_pre:.2%}",
                    "goal": "0% — aucune détection attendue",
                }
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    write_html_report(args.output, sections, chart_pngs)
    print(f"Report written to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
