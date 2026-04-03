#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
#
# Génère des SBOM par écosystème (SPDX / CycloneDX), puis fusion CycloneDX si possible.
# Prérequis (optionnels — les sections absentes sont ignorées avec un avertissement) :
#   - Rust    : cargo install cargo-sbom cyclonedx-rust-cargo  (ou Syft seul)
#   - Go      : Syft (recommandé) ou installation manuelle
#   - Node    : npx @cyclonedx/cdxgen
#   - Python  : pip install pip-audit cyclonedx-bom
#   - Fusion  : npm i -g @cyclonedx/cyclonedx-cli   ou   docker run cyclonedx/cyclonedx-cli
#
# Sortie : dist/sbom/components/ + dist/sbom/merged/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${ROOT}/dist/sbom"
CMP="${OUT}/components"
MRG="${OUT}/merged"
mkdir -p "$CMP" "$MRG"

log() { printf '%s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }
need_cmd() { command -v "$1" >/dev/null 2>&1; }

VERSION="${AEGIS_SBOM_VERSION:-$(git -C "$ROOT" describe --tags --always --dirty 2>/dev/null || echo 0.0.0-dev)}"
log "AEGIS SBOM — version logique: $VERSION"

EXCLUDES=(
  './target/**'
  './**/node_modules/**'
  './.git/**'
  './dist/**'
)

syft_exclude_args=()
for p in "${EXCLUDES[@]}"; do
  syft_exclude_args+=(--exclude "$p")
done

# ---------------------------------------------------------------------------
# Rust — cargo-sbom (SPDX) + cyclonedx-rust-cargo (CycloneDX) si disponibles
# ---------------------------------------------------------------------------
if need_cmd cargo-sbom; then
  log "[Rust] cargo-sbom → SPDX JSON"
  (cd "$ROOT" && cargo-sbom --output-format spdx2_3_json >"$CMP/rust-cargo-sbom.spdx.json") || warn "cargo-sbom a échoué"
else
  warn "cargo-sbom absent — cargo install cargo-sbom"
fi

if (cd "$ROOT" && cargo cyclonedx --version >/dev/null 2>&1); then
  log "[Rust] cargo cyclonedx → CycloneDX JSON (crate: cyclonedx-cargo)"
  (cd "$ROOT" && cargo cyclonedx --output-format json --output-file "$CMP/rust-cyclonedx.cdx.json" --all-features) || warn "cargo cyclonedx a échoué"
elif need_cmd syft; then
  log "[Rust] Syft (Cargo.lock) → SPDX + CycloneDX"
  syft scan "file:${ROOT}/Cargo.lock" -o "spdx-json=${CMP}/rust-lock.spdx.json" || true
  syft scan "file:${ROOT}/Cargo.lock" -o "cyclonedx-json=${CMP}/rust-lock.cdx.json" || true
else
  warn "Ni cargo cyclonedx ni syft — SBOM Rust partiel"
fi

# ---------------------------------------------------------------------------
# Go — Syft sur chaque module
# ---------------------------------------------------------------------------
if need_cmd syft; then
  for gdir in aegis-gateway aegis-policy aegis-llm-proxy; do
    if [[ -f "${ROOT}/${gdir}/go.mod" ]]; then
      name="${gdir//\//-}"
      log "[Go] syft $gdir"
      syft scan "dir:${ROOT}/${gdir}" "${syft_exclude_args[@]}" \
        -o "spdx-json=${CMP}/go-${name}.spdx.json" || warn "syft spdx $gdir"
      syft scan "dir:${ROOT}/${gdir}" "${syft_exclude_args[@]}" \
        -o "cyclonedx-json=${CMP}/go-${name}.cdx.json" || warn "syft cdx $gdir"
    fi
  done
else
  warn "syft absent — https://github.com/anchore/syft"
fi

# ---------------------------------------------------------------------------
# Node — cdxgen (sdk-nodejs, aegis-dashboard si package.json racine)
# ---------------------------------------------------------------------------
if [[ -f "${ROOT}/sdk-nodejs/package.json" ]]; then
  if need_cmd npx; then
    log "[Node] cdxgen sdk-nodejs"
    (cd "${ROOT}/sdk-nodejs" && npx --yes @cyclonedx/cdxgen --output "$CMP/node-sdk-nodejs.cdx.json" .) || warn "cdxgen sdk-nodejs"
  else
    warn "npx absent — impossible de lancer cdxgen"
  fi
fi
if [[ -f "${ROOT}/aegis-dashboard/package.json" ]]; then
  if need_cmd npx; then
    log "[Node] cdxgen aegis-dashboard"
    (cd "${ROOT}/aegis-dashboard" && npx --yes @cyclonedx/cdxgen --output "$CMP/node-aegis-dashboard.cdx.json" .) || warn "cdxgen dashboard"
  fi
fi

# ---------------------------------------------------------------------------
# Python — pip-audit (rapport vuln.) + cyclonedx-bom / cyclonedx-py
# ---------------------------------------------------------------------------
py_env() {
  local req="$1"
  local base="$2"
  [[ -f "$req" ]] || return 0
  if need_cmd pip-audit; then
    log "[Python] pip-audit $req"
    pip-audit -r "$req" -f json --desc on -o "$CMP/${base}-pip-audit.json" || warn "pip-audit $req"
  else
    warn "pip-audit absent — pip install pip-audit"
  fi
  if need_cmd cyclonedx-py; then
    log "[Python] cyclonedx-py $req"
    cyclonedx-py requirements "$req" -o "$CMP/${base}-cyclonedx.cdx.json" || warn "cyclonedx-py $req"
  elif need_cmd python3; then
    if python3 -c "import cyclonedx" 2>/dev/null; then
      cyclonedx-py requirements "$req" -o "$CMP/${base}-cyclonedx.cdx.json" || true
    else
      warn "cyclonedx-py / cyclonedx-bom absent — pip install cyclonedx-bom"
    fi
  fi
}

py_env "${ROOT}/training/requirements.txt" "python-training"
py_env "${ROOT}/datasets/requirements.txt" "python-datasets"
py_env "${ROOT}/benchmarks/scripts/requirements-bench.txt" "python-benchmarks"

# ---------------------------------------------------------------------------
# Fusion CycloneDX → document unique (+ SBOM SPDX agrégé via Syft repo)
# ---------------------------------------------------------------------------
shopt -s nullglob
CDX_FILES=("$CMP"/*.cdx.json)
if ((${#CDX_FILES[@]} > 0)); then
  if need_cmd cyclonedx; then
    log "[Merge] cyclonedx-cli merge"
    cyclonedx merge --merge-type inventory --output-file "$MRG/aegis-merged.cdx.json" "${CDX_FILES[@]}" || warn "cyclonedx merge"
  elif need_cmd npx; then
    log "[Merge] npx @cyclonedx/cyclonedx-cli merge"
    npx --yes @cyclonedx/cyclonedx-cli merge --merge-type inventory \
      --output-file "$MRG/aegis-merged.cdx.json" "${CDX_FILES[@]}" || warn "npx cyclonedx merge"
  else
    warn "cyclonedx-cli absent — fusion manuelle des fichiers dans $CMP"
    printf '%s\n' "${CDX_FILES[@]}" >"$MRG/cyclonedx-components-list.txt"
  fi
fi

if need_cmd syft; then
  log "[Merge] Syft scan répertoire → SPDX unique (vue globale)"
  syft scan "dir:${ROOT}" "${syft_exclude_args[@]}" \
    -o "spdx-json=${MRG}/aegis-repo-merged.spdx.json" || warn "syft merged spdx"
  syft scan "dir:${ROOT}" "${syft_exclude_args[@]}" \
    -o "cyclonedx-json=${MRG}/aegis-repo-merged.cdx.json" || warn "syft merged cdx"
fi

# Manifest des artefacts pour CI / release
{
  echo "# AEGIS — zokastech.fr — SBOM manifest"
  echo "version: $VERSION"
  echo "generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "files:"
  find "$OUT" -type f \( -name '*.json' -o -name '*.txt' \) | sort | sed 's/^/  - /'
} >"$MRG/MANIFEST.txt"

log "Terminé. Artefacts sous: $OUT"
log "À publier en release : $MRG/ (fusion) + $CMP/ (détail)"
