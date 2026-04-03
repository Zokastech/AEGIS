// AEGIS — zokastech.fr — Apache 2.0 / MIT
/** Minimal parser for Prometheus exposition text (AEGIS counters / gauges). */

/**
 * Ensures the body looks like an AEGIS Prometheus scrape (not SPA HTML or JSON error payload).
 */
export function assertPrometheusMetricsBody(text: string): void {
  const head = text.trimStart();
  if (head.startsWith("<!DOCTYPE") || head.startsWith("<!doctype") || /^<html[\s>]/i.test(head)) {
    throw new Error(
      "metrics: HTML body — /metrics URL does not point at the gateway (see VITE_AEGIS_API_URL)."
    );
  }
  // At least one `aegis_*` metric line (not only # HELP / # TYPE).
  if (!/\baegis_[a-z0-9_]+(?:\{|(\s+[-+eE0-9.]+))/.test(text)) {
    throw new Error(
      "metrics: no aegis_* series found (wrong URL, proxy, or non-Prometheus response)."
    );
  }
}

export interface ParsedMetrics {
  /** Sum of all `aegis_analyze_requests_total` (all endpoints). */
  analyzeRequestTotal: number;
  /** Single analyze requests: REST `/v1/analyze` or gRPC `grpc.Analyze`. */
  httpAnalyzeCount: number;
  /** Batch requests: `/v1/analyze/batch` or `grpc.AnalyzeBatch`. */
  httpAnalyzeBatchCount: number;
  /** Anonymize requests: `/v1/anonymize` or `grpc.Anonymize`. */
  httpAnonymizeCount: number;
  anonymizeOpsTotal: number;
  falsePositiveTotal: number;
  activeConnections: number;
  deanonymizeTotal: number;
  /** Sum of `aegis_analyze_duration_seconds_sum` (seconds). */
  analyzeDurationSum: number;
  /** Sum of `aegis_analyze_duration_seconds_count`. */
  analyzeDurationCount: number;
  entitiesByType: Map<string, number>;
  /** Gauge component_ready{component}. */
  componentReady: Map<string, number>;
}

function parseLabels(raw: string): Record<string, string> {
  const out: Record<string, string> = {};
  if (!raw.trim()) return out;
  const re = /(\w+)="((?:\\.|[^"\\])*)"/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(raw)) !== null) {
    out[m[1]] = m[2].replace(/\\"/g, '"');
  }
  return out;
}

function parseLine(line: string): { name: string; labels: Record<string, string>; value: number } | null {
  const t = line.trim();
  if (!t || t.startsWith("#")) return null;
  const i = t.indexOf("{");
  if (i === -1) {
    // `name value` or `name value timestamp` (Prometheus / OpenMetrics exposition).
    const parts = t.split(/\s+/).filter(Boolean);
    if (parts.length < 2) return null;
    const name = parts[0];
    const value = Number(parts[1]);
    if (!name.startsWith("aegis_") || Number.isNaN(value)) return null;
    return { name, labels: {}, value };
  }
  const j = t.indexOf("}", i);
  if (j < 0) return null;
  const name = t.slice(0, i).trim();
  const labels = parseLabels(t.slice(i + 1, j));
  const rest = t.slice(j + 1).trim();
  const value = Number(rest.split(/\s+/)[0]);
  if (!name.startsWith("aegis_") || Number.isNaN(value)) return null;
  return { name, labels, value };
}

export function parsePrometheusText(text: string): ParsedMetrics {
  const out: ParsedMetrics = {
    analyzeRequestTotal: 0,
    httpAnalyzeCount: 0,
    httpAnalyzeBatchCount: 0,
    httpAnonymizeCount: 0,
    anonymizeOpsTotal: 0,
    falsePositiveTotal: 0,
    activeConnections: 0,
    deanonymizeTotal: 0,
    analyzeDurationSum: 0,
    analyzeDurationCount: 0,
    entitiesByType: new Map(),
    componentReady: new Map(),
  };

  for (const line of text.split("\n")) {
    const p = parseLine(line);
    if (!p) continue;

    if (p.name === "aegis_analyze_requests_total") {
      out.analyzeRequestTotal += p.value;
      const ep = p.labels.endpoint ?? "";
      if (ep === "/v1/anonymize" || ep === "grpc.Anonymize") out.httpAnonymizeCount += p.value;
      else if (ep === "/v1/analyze/batch" || ep === "grpc.AnalyzeBatch") out.httpAnalyzeBatchCount += p.value;
      else if (ep === "/v1/analyze" || ep === "grpc.Analyze") out.httpAnalyzeCount += p.value;
    } else if (p.name === "aegis_entities_detected_total") {
      const et = (p.labels.entity_type ?? "unknown").toUpperCase();
      out.entitiesByType.set(et, (out.entitiesByType.get(et) ?? 0) + p.value);
    } else if (p.name === "aegis_anonymize_operations_total") {
      out.anonymizeOpsTotal += p.value;
    } else if (p.name === "aegis_false_positive_reports_total") {
      out.falsePositiveTotal += p.value;
    } else if (p.name === "aegis_active_connections") {
      out.activeConnections = p.value;
    } else if (p.name === "aegis_deanonymize_operations_total") {
      out.deanonymizeTotal += p.value;
    } else if (p.name === "aegis_analyze_duration_seconds_sum") {
      out.analyzeDurationSum += p.value;
    } else if (p.name === "aegis_analyze_duration_seconds_count") {
      out.analyzeDurationCount += p.value;
    } else if (p.name === "aegis_component_ready") {
      const c = p.labels.component ?? "unknown";
      out.componentReady.set(c, p.value);
    }
  }

  return out;
}

export function entitiesToChartData(m: Map<string, number>): { name: string; count: number }[] {
  return [...m.entries()]
    .map(([name, count]) => ({ name, count: Math.round(count) }))
    .sort((a, b) => b.count - a.count);
}

export function topEntitiesForDonut(
  m: Map<string, number>,
  colors: string[],
  topN = 5
): { name: string; value: number; fill: string }[] {
  const rows = entitiesToChartData(m).slice(0, topN);
  return rows.map((e, i) => ({ name: e.name, value: e.count, fill: colors[i % colors.length] }));
}

export type AlertItem = {
  id: string;
  severity: "high" | "medium" | "low";
  titleKey: string;
  titleParams?: Record<string, string | number>;
  at: number;
};

export function alertsFromMetrics(m: ParsedMetrics): AlertItem[] {
  const alerts: AlertItem[] = [];
  let id = 0;
  const ts = () => Date.now();

  for (const [comp, v] of m.componentReady) {
    if (v < 0.5) {
      alerts.push({
        id: `c-${++id}`,
        severity: comp === "ner" ? "medium" : "high",
        titleKey: "dashboard.alerts.componentNotReady",
        titleParams: { comp },
        at: ts(),
      });
    }
  }
  if (m.falsePositiveTotal > 0) {
    alerts.push({
      id: `fp-${++id}`,
      severity: m.falsePositiveTotal >= 10 ? "high" : "low",
      titleKey: "dashboard.alerts.falsePositives",
      titleParams: { count: Math.round(m.falsePositiveTotal) },
      at: ts(),
    });
  }
  if (m.deanonymizeTotal > 0) {
    alerts.push({
      id: `dn-${++id}`,
      severity: "high",
      titleKey: "dashboard.alerts.deanonymizeOps",
      titleParams: { count: Math.round(m.deanonymizeTotal) },
      at: ts(),
    });
  }
  if (m.activeConnections > 50) {
    alerts.push({
      id: `load-${++id}`,
      severity: "medium",
      titleKey: "dashboard.alerts.highLoad",
      titleParams: { count: Math.round(m.activeConnections) },
      at: ts(),
    });
  }
  return alerts;
}
