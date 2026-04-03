#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Agrège les sorties Criterion (target/criterion/**/new/estimates.json),
optionnellement hyperfine + rapport Presidio datasets/, et génère
benchmarks/reports/performance_report.html + copie docs/performance/report.html
"""

from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRITERION = ROOT / "target" / "criterion"
OUT_DIR = ROOT / "benchmarks" / "reports"
DOCS_PERF = ROOT / "docs" / "performance"
HYPERFINE = OUT_DIR / "hyperfine_aegis.json"
PRESIDIO_HTML = ROOT / "datasets" / "reports" / "benchmark_report.html"


def load_mean_ns(estimates_path: Path) -> float | None:
    """Criterion 0.5 : `point_estimate` du temps est en nanosecondes."""
    try:
        data = json.loads(estimates_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    mean = data.get("mean") or {}
    pe = mean.get("point_estimate")
    return float(pe) if pe is not None else None


def collect_criterion_rows() -> list[tuple[str, str, float]]:
    rows: list[tuple[str, str, float]] = []
    if not CRITERION.is_dir():
        return rows
    for est in CRITERION.rglob("new/estimates.json"):
        rel = est.relative_to(CRITERION)
        parts = rel.parts
        if len(parts) >= 3:
            group = parts[0]
            case = "/".join(parts[1:-2])
        else:
            group, case = str(rel), ""
        ns = load_mean_ns(est)
        if ns is not None:
            rows.append((group, case, ns))
    rows.sort(key=lambda x: (x[0], x[1]))
    return rows


def ns_to_human(ns: float) -> str:
    if ns < 1_000:
        return f"{ns:.1f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    return f"{ns / 1_000_000_000:.3f} s"


def pipeline_latency_chart(rows: list[tuple[str, str, float]], png_path: Path) -> None:
    try:
        import matplotlib

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        return
    # pipeline_engine_* groups vs size parameter in case name
    pat = re.compile(r"(\d+)$")
    series: dict[str, list[tuple[int, float]]] = {}
    for group, case, ns in rows:
        if "pipeline" not in group.lower():
            continue
        m = pat.search(case.replace("/", "_"))
        if not m:
            continue
        sz = int(m.group(1))
        series.setdefault(group, []).append((sz, ns / 1_000_000.0))  # ms
    if not series:
        return
    plt.figure(figsize=(10, 5))
    for g, pts in sorted(series.items()):
        pts.sort(key=lambda x: x[0])
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        plt.plot(xs, ys, marker="o", label=g[:40])
    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("Taille texte (octets)")
    plt.ylabel("Latence médiane (ms)")
    plt.title("AEGIS — pipelines vs taille de texte (Criterion)")
    plt.legend(fontsize=7, loc="upper left")
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.savefig(png_path, dpi=120)
    plt.close()


def html_escape(s: str) -> str:
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def build_html(rows: list[tuple[str, str, float]]) -> str:
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    table_rows = "".join(
        f"<tr><td>{html_escape(g)}</td><td><code>{html_escape(c)}</code></td>"
        f"<td>{ns_to_human(ns)}</td><td>{ns:.2e}</td></tr>"
        for g, c, ns in rows[:500]
    )
    hf_block = ""
    if HYPERFINE.is_file():
        try:
            hf = json.loads(HYPERFINE.read_text(encoding="utf-8"))
            res = hf.get("results", [])
            if res:
                t = res[0].get("mean", 0)
                hf_block = f"<p><strong>CLI (hyperfine)</strong> : moyenne ≈ {t*1000:.3f} ms par exécution.</p>"
        except (json.JSONDecodeError, OSError):
            hf_block = "<p>hyperfine : JSON invalide.</p>"
    presidio_block = ""
    if PRESIDIO_HTML.is_file():
        presidio_block = (
            f'<p>Rapport qualité / latence Presidio vs AEGIS (datasets) : '
            f'<a href="../../datasets/reports/benchmark_report.html">benchmark_report.html</a></p>'
        )
    chart_rel = "pipeline_latency.png"
    chart_tag = ""
    if (OUT_DIR / chart_rel).is_file():
        chart_tag = f'<p><img src="{chart_rel}" alt="Latence pipeline" style="max-width:100%"/></p>'

    return f"""<!DOCTYPE html>
<html lang="fr"><head><meta charset="utf-8"/>
<title>AEGIS — rapport de performance</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; max-width: 1200px; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #ccc; padding: 6px 8px; text-align: left; }}
th {{ background: #f4f4f4; }}
code {{ font-size: 0.9em; }}
</style></head><body>
<h1>AEGIS — rapport de performance (zokastech.fr)</h1>
<p>Généré : {html_escape(now)} — source : Criterion (<code>target/criterion</code>).</p>
{hf_block}
{presidio_block}
{chart_tag}
<h2>Latences Criterion (échantillon)</h2>
<table><thead><tr><th>Groupe</th><th>Cas</th><th>Moyenne</th><th>ns (brut)</th></tr></thead>
<tbody>{table_rows}</tbody></table>
<p><em>Exécutez <code>make bench</code> depuis la racine du dépôt pour régénérer.</em></p>
</body></html>"""


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    DOCS_PERF.mkdir(parents=True, exist_ok=True)
    rows = collect_criterion_rows()
    chart_path = OUT_DIR / "pipeline_latency.png"
    pipeline_latency_chart(rows, chart_path)
    html = build_html(rows)
    main_report = OUT_DIR / "performance_report.html"
    main_report.write_text(html, encoding="utf-8")
    (DOCS_PERF / "report.html").write_text(html, encoding="utf-8")
    if chart_path.is_file():
        import shutil

        shutil.copy(chart_path, DOCS_PERF / "pipeline_latency.png")
    print(f"Écrit : {main_report}")
    print(f"Copié : {DOCS_PERF / 'report.html'}")


if __name__ == "__main__":
    main()
