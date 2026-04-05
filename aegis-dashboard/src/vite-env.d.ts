// AEGIS — zokastech.fr — Apache 2.0 / MIT

/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE?: string;
  /** Preferred by docker-compose.dev.yml (URL as seen by the browser). */
  readonly VITE_AEGIS_API_URL?: string;
  /**
   * `true` in dev only: shows the "no auth" checkbox on `/login`.
   * Never set in production builds (`import.meta.env.DEV` is false).
   */
  readonly VITE_ENABLE_DEV_LOGIN_BYPASS?: string;
  /**
   * `true`: show a sample YAML snippet on the Policies page (public demo).
   * Omitted by default: no snippet in production builds.
   */
  readonly VITE_SHOW_POLICY_YAML_DEMO?: string;
  /** Prometheus UI (e.g. http://localhost:9090) — links from the dashboard. */
  readonly VITE_PROMETHEUS_UI_URL?: string;
  /** Grafana UI origin only (e.g. http://localhost:3334) — link to AEGIS — Gateway dashboard. */
  readonly VITE_GRAFANA_UI_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
