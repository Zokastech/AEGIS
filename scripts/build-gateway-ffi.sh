#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Compile la gateway Go avec le moteur Rust (aegis-ffi). Prérequis : Rust toolchain, Go, gcc/clang.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Go absent du PATH (souvent avec conda/miniforge) : essayer les emplacements macOS usuels.
if ! command -v go >/dev/null 2>&1; then
	for cand in /opt/homebrew/bin/go /usr/local/go/bin/go; do
		if [[ -x "$cand" ]]; then
			export PATH="$(dirname "$cand"):$PATH"
			break
		fi
	done
fi
if ! command -v go >/dev/null 2>&1; then
	echo "Erreur : la commande « go » est introuvable (PATH actuel sans Go)." >&2
	echo "  • Installez Go : https://go.dev/dl/ ou  brew install go" >&2
	echo "  • Puis vérifiez :  which go   (ex. /opt/homebrew/bin/go ou /usr/local/go/bin/go)" >&2
	echo "  • Si Go est déjà installé, ajoutez son bin au PATH dans ce terminal, ou désactivez conda." >&2
	exit 1
fi

echo "==> cargo build -p aegis-ffi --release"
cargo build --release -p aegis-ffi
cd "$ROOT/aegis-gateway"
out="${1:-$ROOT/bin/aegis-gateway}"
mkdir -p "$(dirname "$out")"
echo "==> go build -tags aegisffi (CGO_ENABLED=1)"
export CGO_ENABLED=1
go build -tags aegisffi -o "$out" ./cmd/aegis-gateway
echo "OK → $out"
