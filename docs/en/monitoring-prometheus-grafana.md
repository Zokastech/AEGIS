# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Prometheus & Grafana (existing stack)

Use this guide when **Prometheus and Grafana are already deployed** in your organization (managed service, shared platform, or self‑hosted). You only need to **scrape** the AEGIS gateway and **point Grafana** at your Prometheus (or import the dashboard JSON).

## What the gateway exposes

| Endpoint | Content | Notes |
|----------|---------|--------|
| `GET /metrics` | Prometheus text exposition (`aegis_*` metrics) | **Plain text** (gzip is disabled for this path so scrapers parse it reliably). |
| `GET /livez`, `GET /readyz` | HTTP probes | Optional; not scraped by default. |

See also [API reference](api-reference.md) and [Deployment](deployment.md) (observability).

### Authentication on `/metrics`

- **Development** (`development.disable_auth` / insecure dev compose): scrape **without** credentials.
- **Production** (gateway security enabled): scrapes must satisfy **`metrics:view`** (RBAC). Prometheus must send the same credential style as the API, e.g. **`X-API-Key`** or **`Authorization: Bearer <JWT>`**.

Use a secret file mounted into Prometheus (never commit tokens).

---

## 1. Configure Prometheus scrape

Add a `scrape_configs` job (names and addresses are examples — adjust to your network/DNS).

### Gateway in HTTP (e.g. internal mesh, `AEGIS_INSECURE_HTTP=1`)

```yaml
scrape_configs:
  - job_name: aegis-gateway
    scrape_interval: 15s
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ["aegis-gateway.your-namespace.svc:8080"]
        labels:
          service: gateway
          environment: production
```

### Gateway in HTTPS (default production-style)

Example aligned with the repository file [`docker/monitoring/prometheus.yml`](https://github.com/zokastech/aegis/blob/main/docker/monitoring/prometheus.yml):

```yaml
scrape_configs:
  - job_name: aegis-gateway
    scrape_interval: 15s
    metrics_path: /metrics
    scheme: https
    tls_config:
      insecure_skip_verify: true   # replace with proper CA when possible
    static_configs:
      - targets: ["aegis-gateway.example.com:8443"]
        labels:
          service: gateway
```

### Scrape with API key (production RBAC)

Prometheus 2.26+ supports custom authorization headers, e.g. file-based API key:

```yaml
  - job_name: aegis-gateway
    metrics_path: /metrics
    scheme: https
    authorization:
      type: Bearer
      credentials_file: /etc/prometheus/secrets/aegis-api-key.txt
    tls_config:
      ca_file: /etc/prometheus/tls/your-ca.pem
    static_configs:
      - targets: ["aegis-gateway.internal:8080"]
```

If your gateway expects **`X-API-Key`** instead of Bearer, use a **reverse proxy** in front of `/metrics` that maps a path or adds the header, or use Prometheus **`metric_relabel_configs`** / external **oauth2** block where applicable — many teams use a small nginx sidecar that injects `X-API-Key` from a file.

!!! tip "Job label and Grafana"
    The bundled Grafana dashboard filters on `job="aegis-gateway"`. Keep `job_name: aegis-gateway` **or** change the dashboard queries to match your `job` label.

Reload Prometheus (`/-/reload` if lifecycle API enabled, or restart).

---

## 2. Verify the target

1. Prometheus UI → **Status → Targets**: state should be **UP** for `aegis-gateway`.
2. **Graph** → try: `aegis_active_connections` or `aegis_analyze_requests_total`.
3. If you see parse errors mentioning `\x1f`, the response was gzip — use a current gateway build (gzip is skipped for `/metrics`) or disable compression on that path at your proxy.

---

## 3. Grafana (existing instance)

### Datasource

1. **Connections → Data sources → Add data source → Prometheus**.
2. Set **URL** to your Prometheus base (e.g. `http://prometheus:9090` inside the cluster, or your hosted URL).
3. Save & **Test**.

### Dashboard

1. **Dashboards → New → Import**.
2. Upload **`docker/monitoring/grafana/dashboards/aegis-gateway-overview.json`** from the AEGIS repository (or paste JSON).
3. Select the Prometheus datasource you created.

The panels assume `job="aegis-gateway"`. After import, if your scrape job name differs, use **Explore** to run e.g. `sum by (endpoint) (rate(aegis_analyze_requests_total[5m]))` and edit panel queries accordingly.

---

## 4. Kubernetes (optional)

If you use **Prometheus Operator**, add a `ServiceMonitor` (or PodMonitor) that selects the gateway Service and sets `path: /metrics`, `scheme`, and TLS as needed. Mount scrape secrets via `basicAuth` / `bearerTokenSecret` per operator docs.

---

## 5. Relation to the repository compose files

| File | Use case |
|------|-----------|
| `docker-compose.dev.yml` | Local dev: Prometheus + Grafana + scrape HTTP (`docker/monitoring/prometheus.dev.yml`). |
| `docker-compose.yml` + `docker-compose.monitoring.yml` | Full stack with TLS scrape example in `docker/monitoring/prometheus.yml`. |

This page is for **bringing your own** Prometheus/Grafana; you do not need those compose services.

---

## See also

- [THIRD_PARTY_LICENSES.md](https://github.com/zokastech/aegis/blob/main/THIRD_PARTY_LICENSES.md) (Prometheus / Grafana licenses)
- [Security overview](security/index.md)
- [Hardening](security/hardening.md)
