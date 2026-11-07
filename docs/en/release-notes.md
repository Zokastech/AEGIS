# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Release notes — v0.2.0 (2026-11-07)

Security release — upgrading is strongly recommended. Full notes (French):
[Release notes (FR)](../fr/release-notes.md) ·
[CHANGELOG](https://github.com/Zokastech/AEGIS/blob/v0.2.0/CHANGELOG.md).

**Critical fixes**: security config files (`security.yaml`, `--security-config`) silently ignored; unauthenticated gRPC API (incl. de-anonymization and config update); Docker image /
Helm chart shipped with authentication disabled; TLS private key baked into the image and committed to git;
LLM proxy crash (infinite recursion) on any unknown route.

**High**: streaming LLM requests forwarded without anonymization; `block` mode failing open; revoked API keys
still valid and pepper written to disk; data race crash in the rate limiter; `X-Forwarded-For` spoofing;
vulnerable dependencies (rustls/webpki, pyo3, Go 1.25 stdlib, Vite/Storybook).

**Breaking changes**: authentication on by default (`AEGIS_API_KEY_PEPPER` + `aegis-keygen`),
`POSTGRES_PASSWORD` required, gRPC requires credentials in metadata, runtime config cannot touch `security.*`,
JWT must carry `exp`, `disable_auth` refused in production.

**New**: `aegis-keygen`, audit chain verification, 138 new tests, hardened CI (race detector, SDKs,
dashboard, gitleaks, npm audit).
