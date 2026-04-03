#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Point d’entrée du service docker-compose « aegis-gateway » : compile libaegis_ffi puis lance air (CGO + tag aegisffi).
set -euo pipefail

cd /work
echo "==> [aegis-gateway dev] cargo build --release -p aegis-ffi"
cargo build --release -p aegis-ffi

export CGO_ENABLED=1
export LD_LIBRARY_PATH="/work/target/release${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

cd /work/aegis-gateway
chmod +x ./scripts/air-build.sh 2>/dev/null || true
go mod tidy
exec air -c .air.toml
