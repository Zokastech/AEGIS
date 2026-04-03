# AEGIS — zokastech.fr — Apache 2.0 / MIT

# API reference (REST)

Base URL depends on deployment. Examples use `http://localhost:8080`. The gateway serves **`/v1/*`** plus observability routes.

!!! note "Authentication"
    When **RBAC / gateway security** is enabled, most routes require an **API key** header (default name configurable; OpenAPI documents `X-API-Key`). Admin routes (config, deanonymize, subject erasure) require **admin** keys or dedicated permissions.

## Common headers

| Header | When |
|--------|------|
| `Content-Type: application/json` | All `POST`/`PUT` with JSON body |
| `X-API-Key: <secret>` | Secured gateway deployments |
| `Authorization: Bearer <token>` | If configured for your environment |

---

## `GET /v1/health`

Liveness / readiness style health for the gateway and engine linkage.

```bash
curl -sS "http://localhost:8080/v1/health"
```

---

## `GET /v1/openapi.yaml`

Returns the **OpenAPI 3.0** specification (YAML).

```bash
curl -sS "http://localhost:8080/v1/openapi.yaml" -o openapi.yaml
```

---

## `GET /v1/recognizers`

Catalog of recognizers (name, kind, enabled).

```bash
curl -sS "http://localhost:8080/v1/recognizers"
```

---

## `GET /v1/entities`

Supported **entity type** keys (config / policy alignment).

```bash
curl -sS "http://localhost:8080/v1/entities"
```

---

## `POST /v1/analyze`

Analyze a single text.

**Body fields**

| Field | Type | Description |
|-------|------|-------------|
| `text` | string | **Required.** Input text |
| `analysis_config_json` | string | Optional JSON overriding analysis config (engine schema) |
| `policy` | string | Optional policy pack name (e.g. `gdpr-strict`) |

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Patient Jane Doe — nir 1 85 08 75 123 456 78",
    "policy": "gdpr-strict"
  }'
```

---

## `POST /v1/analyze/batch`

Paginated batch analysis.

| Field | Type | Description |
|-------|------|-------------|
| `texts` | string[] | Chunks to analyze |
| `page` | int | Page index (optional) |
| `page_size` | int | Page size (optional) |
| `policy` | string | Optional policy name |

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze/batch" \
  -H "Content-Type: application/json" \
  -d '{"texts":["a@b.com","+33123456789"],"page":1,"page_size":10}'
```

---

## `POST /v1/anonymize`

Run detection + anonymization. `config_json` is the engine **anonymization config** (operators per entity).

| Field | Type | Description |
|-------|------|-------------|
| `text` | string | Input |
| `config_json` | string | JSON: `operators_by_entity`, `default_operator`, … |
| `policy` | string | Optional policy |
| `subject_id` | string | Optional subject id for pseudonym ledger / erasure flows |

```bash
curl -sS -X POST "http://localhost:8080/v1/anonymize" \
  -H "Content-Type: application/json" \
  -d '{"text":"Reach me at x@y.com","config_json":"{}"}'
```

---

## `POST /v1/feedback/false-positive`

Submit false-positive feedback (shape defined by gateway implementation).

```bash
curl -sS -X POST "http://localhost:8080/v1/feedback/false-positive" \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## `POST /v1/deanonymize` (admin)

Reverse reversible transforms when key material is available. **Highly sensitive.**

```bash
curl -sS -X POST "http://localhost:8080/v1/deanonymize" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ADMIN_KEY" \
  -d '{"anonymized_result_json":"{}","key_material_json":"{}"}'
```

---

## `PUT /v1/config` (admin)

Hot-reload **partial or full** engine YAML.

```bash
curl -sS -X PUT "http://localhost:8080/v1/config" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ADMIN_KEY" \
  -d '{"yaml":"pipeline_level: 2\n"}'
```

---

## Policy routes (when policy service is wired)

### `GET /v1/policies`

```bash
curl -sS "http://localhost:8080/v1/policies"
```

### `GET /v1/policy/dpia`

DPIA-oriented report stub (complement with your legal review).

```bash
curl -sS "http://localhost:8080/v1/policy/dpia"
```

### `DELETE /v1/subjects/:id` (admin)

Erasure hook for a data subject (implementation depends on storage backends).

```bash
curl -sS -X DELETE "http://localhost:8080/v1/subjects/sub-123" \
  -H "X-API-Key: ADMIN_KEY"
```

---

## `GET /v1/audit/export` (secured gateway)

Export audit records (permission `PermAuditExport`).

```bash
curl -sS "http://localhost:8080/v1/audit/export" -H "X-API-Key: KEY"
```

---

## Observability

| Method | Path | Description |
|--------|------|-------------|
| GET | `/metrics` | Prometheus metrics |
| GET | `/livez` | Liveness (secured mode) |
| GET | `/readyz` | Readiness (secured mode) |
| GET | `/health/live`, `/health/ready`, `/health/startup` | Alternate health package (if enabled in build) |

```bash
curl -sS "http://localhost:8080/metrics" | head
```

---

## Errors

JSON errors typically include `code`, `message`, and optional `request_id` (`ErrorBody` in the gateway code).

---

## Source of truth

Route table: `aegis-gateway/api/rest/server.go` — DTOs: `aegis-gateway/api/rest/dto.go` — OpenAPI: `aegis-gateway/api/rest/openapi.yaml`.
