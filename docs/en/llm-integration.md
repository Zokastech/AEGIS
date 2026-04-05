# AEGIS — zokastech.fr — Apache 2.0 / MIT

# LLM integration (`aegis-llm-proxy`)

The **`aegis-llm-proxy`** service sits between your applications and an upstream LLM (OpenAI-compatible, Anthropic, Ollama, …). It can **analyze** prompts/responses with the AEGIS engine and apply **transparent**, **anonymize**, **block**, or **alert** modes.

## Configuration (YAML)

Key fields (see `aegis-llm-proxy/internal/config/config.go`):

| Field | Description |
|-------|-------------|
| `listen` | Bind address (overridable with `AEGIS_LLM_LISTEN`) |
| `upstream_url` | Provider base URL |
| `mode` | `transparent` \| `anonymize` \| `block` \| `alert` |
| `score_threshold` | Passed to the engine via `analysis_config_json` |
| `block_min_score` | In `block` mode, threshold to reject |
| `webhook_url` | For `alert` mode (async JSON POST) |
| `anonymize_config_json` | Operator profile for automatic anonymization |
| `protected_entity_types` | If set, only these entity types trigger protection |
| `engine.type` | `http` (call gateway) or `cli` (spawn `aegis`) |
| `engine.base_url` | Gateway URL when `type: http` |
| `engine.timeout_seconds` | Engine call budget |
| `inject_api_key_env` | Inject `Authorization: Bearer` from env if client omitted |
| `dashboard` | Optional stats prefix for AEGIS dashboard |

## Modes

| Mode | Behavior |
|------|----------|
| `transparent` | Forward traffic; optional logging/metrics only |
| `anonymize` | Rewrite prompts (and optionally responses) after detection |
| `block` | Reject requests when high-confidence PII is found |
| `alert` | Forward but notify `webhook_url` |

## Run locally

```bash
cd aegis-llm-proxy
go build -o aegis-llm-proxy ./cmd/aegis-llm-proxy
./aegis-llm-proxy -config config.yaml
```

Provide a valid `config.yaml` pointing at a running **AEGIS gateway** or CLI path.

## LangChain

Use LangChain’s **HTTP client** pointed at the proxy instead of the vendor:

1. Set `base_url` to `http://localhost:<proxy-port>/v1` (or your mount path).
2. Keep vendor API key in the proxy config (`inject_api_key_env`) so app code never holds raw secrets twice.

The exact wrapper depends on LangChain version; conceptually the proxy is a **drop-in OpenAI-compatible base URL** when using OpenAI provider routes.

## LlamaIndex

Same pattern: configure the **OpenAI** or **custom OpenAI**-compatible LLM class with the proxy URL. Ensure request/response shapes match what `aegis-llm-proxy` forwards (`internal/proxy/handler.go`).

## Security notes

- Terminate **TLS** in front of the proxy in production.
- Treat `anonymize_config_json` and upstream API keys as **secrets**.
- Review [Threat model](security/threat-model.md) for residual PII risk (false negatives).

---

## Source

- Entrypoint: `aegis-llm-proxy/cmd/aegis-llm-proxy/main.go`
- Handler: `aegis-llm-proxy/internal/proxy/handler.go`
