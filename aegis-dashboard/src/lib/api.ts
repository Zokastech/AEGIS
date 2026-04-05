// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { assertPrometheusMetricsBody } from "@/lib/prometheus";

/** Gateway base URL (no trailing slash). Dev compose: VITE_AEGIS_API_URL. */
function apiBase(): string {
  const raw =
    import.meta.env.VITE_API_BASE ||
    import.meta.env.VITE_AEGIS_API_URL ||
    "";
  return String(raw).replace(/\/+$/, "");
}

/** True when an absolute gateway base URL is set at build time (required outside `vite dev` + proxy). */
export function isApiBaseConfigured(): boolean {
  return Boolean(apiBase());
}

/**
 * In `vite dev`, requests to `http://localhost:8080/...` from origin `:5173` are **cross-origin**:
 * the browser requires CORS headers on the response. If the gateway runs in prod mode or without
 * `Access-Control-Allow-Origin` for that origin, fetch fails (even when the server returns 200).
 * The Vite proxy (`vite.config.ts` → `/v1`, `/metrics`) only applies to **relative** paths on `:5173`.
 * So in dev, when the configured base already points at the local gateway, we drop the base and keep a relative path.
 */
function devShouldUseViteProxy(base: string): boolean {
  if (!import.meta.env.DEV || !base) return false;
  try {
    const href = base.includes("://") ? base : `http://${base}`;
    const u = new URL(href);
    return u.hostname === "localhost" || u.hostname === "127.0.0.1";
  } catch {
    return false;
  }
}

/**
 * Typical JWT: base64url header often starts with eyJ ({"alg":...}).
 * Do not rely on "three segments" alone: many API keys contain dots and would be sent as Bearer
 * while the gateway expects X-API-Key.
 */
function looksLikeJwt(value: string): boolean {
  const v = value.trim();
  if (!v.startsWith("eyJ")) return false;
  const p = v.split(".");
  return p.length === 3 && p.every((s) => s.length > 0);
}

export function apiUrl(path: string): string {
  if (path.startsWith("http")) return path;
  const b = apiBase();
  const p = path.startsWith("/") ? path : `/${path}`;
  if (!b) return p;
  if (devShouldUseViteProxy(b)) return p;
  return `${b}${p}`;
}

export async function apiFetch(path: string, init: RequestInit = {}, credential?: string | null): Promise<Response> {
  const headers = new Headers(init.headers);
  const c = credential?.trim();
  if (c) {
    if (looksLikeJwt(c)) headers.set("Authorization", `Bearer ${c}`);
    else headers.set("X-API-Key", c);
  }
  if (!headers.has("Content-Type") && init.body && typeof init.body === "string") {
    headers.set("Content-Type", "application/json");
  }
  return fetch(apiUrl(path), { ...init, headers });
}

export async function analyzeText(
  text: string,
  opts: { analysisConfigJson?: string; policy?: string },
  token: string | null
): Promise<unknown> {
  const res = await apiFetch(
    "/v1/analyze",
    {
      method: "POST",
      body: JSON.stringify({
        text,
        analysis_config_json: opts.analysisConfigJson ?? "",
        policy: opts.policy ?? "",
      }),
    },
    token
  );
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function anonymizeText(
  text: string,
  configJson: string,
  opts: { policy?: string; subject_id?: string },
  token: string | null
): Promise<unknown> {
  const res = await apiFetch(
    "/v1/anonymize",
    {
      method: "POST",
      body: JSON.stringify({
        text,
        config_json: configJson,
        policy: opts.policy ?? "",
        subject_id: opts.subject_id ?? "",
      }),
    },
    token
  );
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function listRecognizers(token: string | null): Promise<{ recognizers: { name: string; kind: string; enabled: boolean }[] }> {
  const res = await apiFetch("/v1/recognizers", {}, token);
  if (!res.ok) throw new Error("recognizers");
  return res.json();
}

export async function listPolicies(token: string | null): Promise<{ policies: string[] }> {
  const res = await apiFetch("/v1/policies", {}, token);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

/** GET /v1/policy/dpia?policy=&format=markdown — response body (Markdown). */
export async function fetchPolicyDpiaMarkdown(policyName: string, token: string | null): Promise<string> {
  const q = new URLSearchParams({ policy: policyName, format: "markdown" });
  const res = await apiFetch(`/v1/policy/dpia?${q}`, { method: "GET" }, token);
  if (!res.ok) throw new Error(await res.text());
  return res.text();
}

export type AuditEntryRow = {
  id: string;
  at: string;
  user: string;
  action: string;
  path: string;
  success: boolean;
  method?: string;
  requestId?: string;
  statusCode?: number;
};

/** GET /v1/audit/export — JSONL (one audit entry per line). */
export async function fetchAuditExportLines(token: string | null): Promise<AuditEntryRow[]> {
  const res = await apiFetch("/v1/audit/export", { method: "GET" }, token);
  if (!res.ok) throw new Error(await res.text());
  const text = await res.text();
  const rows: AuditEntryRow[] = [];
  let i = 0;
  for (const line of text.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    try {
      const o = JSON.parse(t) as {
        ts?: string;
        actor?: string;
        action?: string;
        endpoint?: string;
        method?: string;
        request_id?: string;
        success?: boolean;
        status_code?: number;
      };
      rows.push({
        id: `audit-${i++}`,
        at: o.ts ?? "",
        user: o.actor ?? "",
        action: o.action ?? "",
        path: o.endpoint ?? "",
        success: Boolean(o.success),
        method: o.method,
        requestId: o.request_id,
        statusCode: o.status_code,
      });
    } catch {
      /* ignore malformed line */
    }
  }
  return rows;
}

/** Raw Prometheus text (`GET /metrics`) — same auth headers as the rest of the dashboard when needed. */
export async function fetchMetricsText(credential: string | null): Promise<string> {
  const res = await apiFetch("/metrics", { method: "GET" }, credential);
  if (!res.ok) {
    const hint = res.status === 403 ? " (metrics:view permission required in secured prod?)" : "";
    throw new Error(`metrics: HTTP ${res.status}${hint}`);
  }
  const text = await res.text();
  const ct = (res.headers.get("content-type") ?? "").toLowerCase();
  if (ct.includes("text/html") && !/\baegis_[a-z0-9_]+(?:\{|\s)/.test(text)) {
    throw new Error(
      "metrics: HTML response — /metrics is not reaching the gateway. Set VITE_AEGIS_API_URL (or VITE_API_BASE) at build time, or proxy /metrics to the gateway."
    );
  }
  assertPrometheusMetricsBody(text);
  return text;
}
