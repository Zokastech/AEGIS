#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Appelé par air (.air.toml). Avec AEGIS_GATEWAY_FFI=1 (docker-compose dev) : lien contre libaegis_ffi.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "${AEGIS_GATEWAY_FFI:-}" = "1" ]; then
  exec env CGO_ENABLED=1 go build -tags=aegisffi -o ./tmp/main ./cmd/aegis-gateway
else
  exec go build -o ./tmp/main ./cmd/aegis-gateway
fi
