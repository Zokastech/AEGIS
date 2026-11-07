# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Installing AEGIS v0.2.0

Since v0.2.0 AEGIS is **secure by default**: authentication and TLS are on, and the gateway refuses to
start with dangerous settings (e.g. `disable_auth` while `production: true`).
The full guide (French) is available in [Installation (FR)](../fr/installation.md) and in
[`INSTALL.md`](https://github.com/Zokastech/AEGIS/blob/v0.2.0/INSTALL.md).

## Docker Compose quick start

```bash
git clone https://github.com/Zokastech/AEGIS.git && cd AEGIS && git checkout v0.2.0
cp .env.example .env && chmod 600 .env
# set POSTGRES_PASSWORD and AEGIS_API_KEY_PEPPER (openssl rand -hex 32) in .env
docker compose up -d --build

# create API keys (secret printed once, stored hashed)
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role operator -id app
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role admin -id admin-1
docker compose exec aegis-gateway aegis-keygen -file /var/lib/aegis/api_keys.yaml -role admin -id admin-2
docker compose restart aegis-gateway

curl -sk https://localhost:8443/v1/analyze -H "X-API-Key: $KEY" \
  -H 'Content-Type: application/json' -d '{"text":"Contact jean.dupont@example.fr"}'
```

## Packages

| Channel | Install |
|---|---|
| CLI | signed tarball from the GitHub release (`aegis-cli-v0.2.0-linux-{amd64,arm64}.tar.gz`) or `cargo install aegis-cli --version 0.2.0` |
| Python | `pip install aegis-pii==0.2.0` |
| Node.js | `npm install @aegis-pii/core@0.2.0` |
| Rust | `aegis-core`, `aegis-regex`, `aegis-anonymize` = `0.2.0` |
| Helm | `deploy/helm/aegis` (chart 0.4.0, appVersion 0.2.0) |

## Production checklist

- Mount a certificate from your PKI; enable mTLS for internal services.
- Keep `AEGIS_API_KEY_PEPPER` in a secret manager, never in `api_keys.yaml` or git.
- At least two distinct admin keys (dual approval for de-anonymization); rotate with `aegis-keygen -rotate`.
- Set `security.trusted_proxies` to your reverse proxies only; do not expose gRPC port 9090 publicly.
- JWT: `hmac_secret` ≥ 32 chars, `issuer`/`audience` set, tokens must carry `exp`.
