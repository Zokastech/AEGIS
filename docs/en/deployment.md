# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Deployment

For **Zokastech** backend (`zokastech/backend`) configuration and launch, see [Zokastech platform](zokastech.md). For **per-cloud Terraform** (AWS, GCP, Azure, OVHcloud), see [Cloud providers](cloud-providers.md).

## Docker Compose (development / small prod)

1. `cp .env.example .env` and set secrets.
2. `docker compose up -d --build`

Services (typical):

| Service | Role |
|---------|------|
| `aegis-core` | Rust engine + optional ONNX volume |
| `aegis-gateway` | HTTP API |
| `aegis-dashboard` | Web UI |
| `postgres` | Metadata / policy / audit (per integration) |
| `redis` | Cache / queues |
| `onnx-init` | Optional job: **ZOKA-SENTINEL** bundle (`.tgz` → `ner.onnx` + `tokenizer.json`) or `NER_ONNX_URL` — see `.env.example` |

Networks **`frontend`** and **`backend`** separate the dashboard from internal services — mirror this with security groups in cloud deployments.

See [`docker-compose.yml`](https://github.com/zokastech/aegis/blob/main/docker-compose.yml).

---

## Kubernetes

Official chart: **`deploy/helm/aegis/`** — install with Helm 3+.

```bash
helm upgrade --install aegis ./deploy/helm/aegis \
  --namespace aegis --create-namespace \
  -f your-values.yaml
```

Override image tags, ingress TLS, resource limits, and secrets via `values.yaml`.

---

## Cloud reference Terraform

Terraform modules are documented in depth on **[Cloud providers (AWS, GCP, Azure, OVH)](cloud-providers.md)**. Short map:

| Directory | Provider |
|-----------|----------|
| `deploy/terraform/aws/` | AWS |
| `deploy/terraform/gcp/` | Google Cloud |
| `deploy/terraform/azure/` | Azure |
| `deploy/terraform/ovh/` | OVHcloud |

These modules illustrate VPC-style networking, managed databases, and regional constraints — **review before production use**.

---

## Configuration & secrets

- Mount **`aegis-config.yaml`** read-only.
- Store API keys and DB passwords in **Kubernetes Secrets**, **GCP Secret Manager**, **AWS Secrets Manager**, or equivalent.
- Enable TLS at ingress; see [Hardening](security/hardening.md).

---

## Observability

- Prometheus: `GET /metrics` on the gateway
- Health: `/v1/health`, `/livez`, `/readyz` (depending on build flags)

If Prometheus and Grafana are **already** deployed in your environment, see **[Prometheus & Grafana (existing stack)](monitoring-prometheus-grafana.md)** for scrape configuration, RBAC on `/metrics`, and importing the AEGIS Grafana dashboard.

---

## Related

- [Getting started](getting-started.md)
- [Security](security/index.md)
