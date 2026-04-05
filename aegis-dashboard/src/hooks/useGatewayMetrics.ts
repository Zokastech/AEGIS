// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { gatewayMetricsQueryOptions } from "@/queries/gateway-query-options";
import type { ParsedMetrics } from "@/lib/prometheus";
import { useAuthCredential } from "@/hooks/useAuthCredential";

export type TrendPoint = { label: string; scans: number; anonymized: number };

const MAX_TREND = 36;

/**
 * Prometheus metrics + local series of deltas between polls (derived effect, outside queryFn).
 */
export function useGatewayMetrics(pollMs: number) {
  const credential = useAuthCredential();
  const prev = useRef<{ a: number; n: number } | null>(null);
  const [trend, setTrend] = useState<TrendPoint[]>([]);

  const q = useQuery(gatewayMetricsQueryOptions(pollMs, credential));

  useEffect(() => {
    const m = q.data;
    if (!m) return;
    const a = m.httpAnalyzeCount + m.httpAnalyzeBatchCount;
    const n = m.httpAnonymizeCount;
    if (prev.current !== null) {
      const da = Math.max(0, a - prev.current.a);
      const dn = Math.max(0, n - prev.current.n);
      const label = new Date().toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
      setTrend((rows) => [...rows.slice(-(MAX_TREND - 1)), { label, scans: da, anonymized: dn }]);
    }
    prev.current = { a, n };
  }, [q.data]);

  return { ...q, trend };
}

export function meanAnalyzeLatencyMs(m: ParsedMetrics): number | null {
  if (m.analyzeDurationCount <= 0) return null;
  return (m.analyzeDurationSum / m.analyzeDurationCount) * 1000;
}

export function fpRatePercent(m: ParsedMetrics): number | null {
  const rest = m.httpAnalyzeCount + m.httpAnalyzeBatchCount;
  const denom = rest > 0 ? rest : m.analyzeRequestTotal;
  if (denom <= 0) return m.falsePositiveTotal > 0 ? null : 0;
  return (m.falsePositiveTotal / denom) * 100;
}
