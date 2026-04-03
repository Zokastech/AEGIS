// AEGIS — zokastech.fr — Apache 2.0 / MIT

function trimUrl(s: string | undefined): string {
  return String(s ?? "").trim();
}

/** Prometheus UI URL (e.g. http://localhost:9090) — optional, set at build time. */
export function prometheusUiHref(): string | null {
  const b = trimUrl(import.meta.env.VITE_PROMETHEUS_UI_URL);
  return b || null;
}

/** Prometheus "Targets" (scrape) page when the Prometheus UI URL is set. */
export function prometheusTargetsHref(): string | null {
  const b = prometheusUiHref();
  if (!b) return null;
  return `${b.replace(/\/+$/, "")}/targets`;
}

/**
 * Grafana dashboard "AEGIS — Gateway" (provisioned uid: aegis-gateway-overview).
 * `VITE_GRAFANA_UI_URL` = origin only (e.g. http://localhost:3334).
 */
export function grafanaGatewayDashboardHref(): string | null {
  const b = trimUrl(import.meta.env.VITE_GRAFANA_UI_URL);
  if (!b) return null;
  return `${b.replace(/\/+$/, "")}/d/aegis-gateway-overview`;
}
