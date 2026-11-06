// AEGIS — zokastech.fr — Apache 2.0 / MIT
import { describe, expect, it } from "vitest";
import { alertsFromMetrics, assertPrometheusMetricsBody, entitiesToChartData, parsePrometheusText } from "@/lib/prometheus";

const SAMPLE = `# HELP aegis_analyze_requests_total x
# TYPE aegis_analyze_requests_total counter
aegis_analyze_requests_total{endpoint="/v1/analyze",method="POST",status="200"} 7
aegis_analyze_requests_total{endpoint="/v1/anonymize",method="POST",status="200"} 3
`;

describe("prometheus", () => {
  it("rejette une page HTML ou un corps sans série aegis_*", () => {
    expect(() => assertPrometheusMetricsBody("<!DOCTYPE html><html></html>")).toThrow();
    expect(() => assertPrometheusMetricsBody("# HELP x\nfoo 1\n")).toThrow();
    expect(() => assertPrometheusMetricsBody(SAMPLE)).not.toThrow();
  });

  it("parse les compteurs sans planter sur des lignes malformées", () => {
    const m = parsePrometheusText(SAMPLE + "garbage line {{{\naegis_x{a=\"b\"} NaN\n");
    expect(m.analyzeRequestTotal).toBeGreaterThanOrEqual(7);
    expect(Array.isArray(alertsFromMetrics(m))).toBe(true);
  });

  it("entitiesToChartData", () => {
    const d = entitiesToChartData(new Map([["EMAIL", 2], ["PHONE", 5]]));
    expect(d.map((x) => x.name).sort()).toEqual(["EMAIL", "PHONE"]);
  });
});
