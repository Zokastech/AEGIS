#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Mesure CLI avec hyperfine (si installé). Même machine, même échantillon texte.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CLI="${AEGIS_BIN:-$ROOT/target/release/aegis}"
OUT="$ROOT/benchmarks/reports/hyperfine_aegis.json"
mkdir -p "$(dirname "$OUT")"

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "hyperfine absent : installez https://github.com/sharkdp/hyperfine — ignoré."
  exit 0
fi

if [[ ! -x "$CLI" ]]; then
  echo "Binaire aegis introuvable : $CLI (cargo build -p aegis-cli --release)"
  exit 0
fi

export AEGIS_BIN="$CLI"
HF_ONCE="$ROOT/benchmarks/scripts/hf_aegis_once.sh"
chmod +x "$HF_ONCE"

hyperfine --warmup 2 --runs 25 --export-json "$OUT" "$HF_ONCE"

echo "hyperfine → $OUT"
