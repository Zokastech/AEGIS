#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
set -euo pipefail

: "${GITHUB_WORKSPACE:?GITHUB_WORKSPACE missing}"
: "${GITHUB_OUTPUT:?GITHUB_OUTPUT missing}"

TOKEN="${INPUT_GITHUB_TOKEN:-${GITHUB_TOKEN:-}}"
export GITHUB_TOKEN="${TOKEN}"

ROOT="${GITHUB_WORKSPACE}"
REPORT="${ROOT}/aegis-scan-report.md"
LANGUAGES="${INPUT_LANGUAGES:-fr,en}"
THRESHOLD="${INPUT_SCORE_THRESHOLD:-0.5}"
FAIL_ON_PII="${INPUT_FAIL_ON_PII:-true}"
CONFIG="${INPUT_CONFIG_PATH:-}"
SCAN_MODE="${INPUT_SCAN_MODE:-auto}"
CLI_VERSION="${INPUT_CLI_VERSION:-latest}"
RELEASE_REPO="${INPUT_RELEASE_REPO:-zokastech/aegis}"
COMMENT_PR="${INPUT_COMMENT_PR:-true}"
EXT_REGEX='\.(py|js|ts|tsx|jsx|mjs|cjs|java|go|rs|json|yaml|yml|md|txt|csv|sql|env|log)$'

shopt -s globstar nullglob 2>/dev/null || true

warn() { echo "::warning::$*"; }
err() { echo "::error::$*"; }

normalize_target() {
  case "${RUNNER_OS}-${RUNNER_ARCH}" in
    Linux-X64) echo "linux-amd64" ;;
    Linux-ARM64) echo "linux-arm64" ;;
    macOS-X64) echo "darwin-amd64" ;;
    macOS-ARM64) echo "darwin-arm64" ;;
    Windows-X64) echo "windows-amd64" ;;
    *) echo "unknown" ;;
  esac
}

download_cli() {
  local target="$1"
  local tag="$2"
  local repo="$3"
  [[ "$target" != "unknown" ]] || return 1
  local api_url
  if [[ "$tag" == "latest" ]]; then
    api_url="https://api.github.com/repos/${repo}/releases/latest"
  else
    api_url="https://api.github.com/repos/${repo}/releases/tags/${tag}"
  fi
  local json
  json="$(curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN:-}" -H "Accept: application/vnd.github+json" "$api_url" 2>/dev/null || true)"
  [[ -n "$json" ]] || return 1
  echo "$json" | jq -e .tag_name >/dev/null 2>&1 || return 1
  local asset_url
  asset_url="$(echo "$json" | jq -r --arg t "$target" '
    .assets[]?
    | select((.name | test($t)) and (.name | test("\\.(tar\\.gz|zip)$"; "i")))
    | .browser_download_url' | head -n1)"
  if [[ -z "$asset_url" || "$asset_url" == "null" ]]; then
    asset_url="$(echo "$json" | jq -r '.assets[]? | select(.name | test("aegis"; "i")) | select(.name | test("\\.(tar\\.gz|zip)$"; "i")) | .browser_download_url' | head -n1)"
  fi
  [[ -n "$asset_url" && "$asset_url" != "null" ]] || return 1
  local dl="${RUNNER_TEMP:-/tmp}/aegis-cli-dl"
  mkdir -p "$dl"
  if [[ "$asset_url" == *.zip ]]; then
    curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN:-}" -o "$dl/a.zip" "$asset_url"
    unzip -qo "$dl/a.zip" -d "$dl"
  else
    curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN:-}" -o "$dl/a.tgz" "$asset_url"
    tar -xzf "$dl/a.tgz" -C "$dl"
  fi
  local bin
  bin="$(find "$dl" -type f \( -name 'aegis' -o -name 'aegis.exe' \) 2>/dev/null | head -n1)"
  [[ -n "$bin" ]] || return 1
  chmod +x "$bin" 2>/dev/null || true
  echo "$bin"
}

try_cargo_build() {
  local crate_dir="${ROOT}/crates/aegis-cli"
  [[ -f "${crate_dir}/Cargo.toml" ]] || return 1
  if ! command -v cargo >/dev/null 2>&1; then
    warn "Installation rustup (cargo absent)"
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
  fi
  (cd "${ROOT}" && cargo build -q --release -p aegis-cli)
  echo "${ROOT}/target/release/aegis"
}

ensure_aegis() {
  local target
  target="$(normalize_target)"
  local bin
  bin="$(download_cli "$target" "$CLI_VERSION" "$RELEASE_REPO" 2>/dev/null || true)"
  if [[ -z "$bin" ]]; then
    warn "Binaire release indisponible — compilation locale crates/aegis-cli"
    bin="$(try_cargo_build)"
  fi
  [[ -n "$bin" && -f "$bin" ]] || {
    err "Binaire aegis introuvable"
    exit 1
  }
  [[ -x "$bin" ]] || chmod +x "$bin"
  echo "$bin"
}

print_exclude_patterns() {
  local raw="${INPUT_EXCLUDE_PATTERNS:-}"
  echo "$raw" | tr ',' '\n' | sed '/^[[:space:]]*$/d' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

excluded() {
  local path="$1"
  [[ "$path" == *"/node_modules/"* ]] && return 0
  [[ "$path" == *"/vendor/"* ]] && return 0
  [[ "$path" == *"/.git/"* ]] && return 0
  [[ "$path" == *"/target/"* ]] && return 0
  local pat
  while IFS= read -r pat; do
    [[ -z "$pat" ]] && continue
    case "$path" in
      $pat) return 0 ;;
    esac
    [[ "$path" == $pat ]] && return 0
  done < <(print_exclude_patterns)
  return 1
}

matches_extension() {
  [[ "$1" =~ $EXT_REGEX ]]
}

under_any_root() {
  local f="$1"
  local r
  for r in "${SCAN_ROOTS[@]}"; do
    [[ "$r" == "." ]] && return 0
    [[ "$f" == "$r" ]] && return 0
    [[ "$f" == "$r"/* ]] && return 0
  done
  return 1
}

load_scan_roots() {
  SCAN_ROOTS=()
  while IFS= read -r line; do
    [[ -n "$line" ]] && SCAN_ROOTS+=("$line")
  done <<< "${INPUT_PATHS:-.}"
  [[ ${#SCAN_ROOTS[@]} -gt 0 ]] || SCAN_ROOTS=(.)
}

list_files_full() {
  local f
  load_scan_roots
  (cd "$ROOT" && git ls-files -z) 2>/dev/null | while IFS= read -r -d '' f; do
    [[ -f "${ROOT}/${f}" ]] || continue
    under_any_root "$f" || continue
    matches_extension "$f" || continue
    excluded "$f" && continue
    printf '%s\n' "$f"
  done | sort -u
}

list_files_pr() {
  local base head
  if [[ -n "${GITHUB_EVENT_PATH:-}" && -f "$GITHUB_EVENT_PATH" ]]; then
    base="$(jq -r '.pull_request.base.sha // empty' "$GITHUB_EVENT_PATH")"
    head="$(jq -r '.pull_request.head.sha // empty' "$GITHUB_EVENT_PATH")"
  fi
  if [[ -z "$base" || -z "$head" || "$base" == "null" || "$head" == "null" ]]; then
    warn "PR sans SHAs valides — repli full_repo"
    list_files_full
    return
  fi
  load_scan_roots
  git -C "$ROOT" diff --name-only --diff-filter=ACMRTUXB "$base" "$head" 2>/dev/null | while IFS= read -r f; do
    [[ -f "${ROOT}/${f}" ]] || continue
    under_any_root "$f" || continue
    matches_extension "$f" || continue
    excluded "$f" && continue
    printf '%s\n' "$f"
  done | sort -u
}

resolve_scan_mode() {
  local mode="$SCAN_MODE"
  if [[ "$mode" == "auto" ]]; then
    if [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]; then
      echo "pr_files"
    else
      echo "full_repo"
    fi
  else
    echo "$mode"
  fi
}

declare -a SCAN_ROOTS=()
AEGIS_BIN="$(ensure_aegis)"
SCAN_RESOLVED="$(resolve_scan_mode)"
mapfile -t FILES < <(
  if [[ "$SCAN_RESOLVED" == "pr_files" ]]; then
    list_files_pr
  else
    list_files_full
  fi
)

TOTAL_ENTITIES=0
PII_FOUND=false
{
  echo "# AEGIS — rapport PII"
  echo
  echo "| Fichier | Entités | Types (extrait) |"
  echo "|---------|---------|----------------|"
} >"$REPORT"

CFG_ABS=""
if [[ -n "$CONFIG" ]]; then
  if [[ -f "${ROOT}/${CONFIG}" ]]; then
    CFG_ABS="${ROOT}/${CONFIG}"
  elif [[ -f "$CONFIG" ]]; then
    CFG_ABS="$CONFIG"
  else
    err "config_path introuvable : $CONFIG"
    exit 1
  fi
fi

if [[ ${#FILES[@]} -eq 0 ]]; then
  echo "| *(aucun fichier à scanner)* | — | — |" >>"$REPORT"
fi

for f in "${FILES[@]}"; do
  [[ -z "${f:-}" ]] && continue
  abs="${ROOT}/${f}"
  set +e
  if [[ -n "$CFG_ABS" ]]; then
    out="$("$AEGIS_BIN" --score-threshold "$THRESHOLD" --language "$LANGUAGES" --config "$CFG_ABS" scan "$abs" 2>&1)"
  else
    out="$("$AEGIS_BIN" --score-threshold "$THRESHOLD" --language "$LANGUAGES" scan "$abs" 2>&1)"
  fi
  code=$?
  set -e
  if [[ "$code" -ne 0 ]]; then
    echo "| \`$f\` | *(erreur CLI)* | \`${out//$'\n'/ }\` |" >>"$REPORT"
    continue
  fi
  n="$(echo "$out" | jq 'if type == "array" then [.[].entities // [] | length] | add else (.entities // []) | length end' 2>/dev/null || echo "0")"
  types="$(echo "$out" | jq -r 'if type == "array" then [.[].entities[]? | .entity_type | tostring] | unique else [.entities[]? | .entity_type | tostring] | unique end | join(", ")' 2>/dev/null || echo "")"
  if [[ "${n:-0}" =~ ^[0-9]+$ ]] && [[ "${n:-0}" -gt 0 ]]; then
    PII_FOUND=true
    TOTAL_ENTITIES=$((TOTAL_ENTITIES + n))
    echo "| \`$f\` | $n | $types |" >>"$REPORT"
    {
      echo
      echo "## Détail : \`$f\`"
      echo
      echo '```json'
      echo "$out" | jq 'if type == "array" then .[0].entities // [] else .entities // [] end' 2>/dev/null || echo "$out"
      echo '```'
    } >>"$REPORT"
  fi
done

{
  echo
  echo "---"
  echo "- **Fichiers scannés** : ${#FILES[@]}"
  echo "- **Entités totales** : $TOTAL_ENTITIES"
  echo "- **Mode** : $SCAN_RESOLVED"
  echo "- **Seuil** : $THRESHOLD — **Langues** : $LANGUAGES"
} >>"$REPORT"

echo "pii_found=${PII_FOUND}" >>"$GITHUB_OUTPUT"
echo "entity_count=${TOTAL_ENTITIES}" >>"$GITHUB_OUTPUT"
echo "report_path=aegis-scan-report.md" >>"$GITHUB_OUTPUT"

if [[ "$PII_FOUND" == "true" && "$COMMENT_PR" == "true" && "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]; then
  PR_NUMBER="$(jq -r '.pull_request.number // empty' "${GITHUB_EVENT_PATH:-}" 2>/dev/null || true)"
  if [[ -n "$PR_NUMBER" && "$PR_NUMBER" != "null" ]] && command -v gh >/dev/null 2>&1; then
    body="${RUNNER_TEMP:-/tmp}/aegis-pr-comment.md"
    {
      echo "## AEGIS — détection PII"
      echo
      echo "Entités détectées : **${TOTAL_ENTITIES}** (fichiers : ${#FILES[@]})"
      echo
      head -n 80 "$REPORT"
      echo
      echo "<details><summary>Rapport complet</summary>"
      echo
      cat "$REPORT"
      echo
      echo "</details>"
    } >"$body"
    gh pr comment "$PR_NUMBER" --repo "${GITHUB_REPOSITORY}" --body-file "$body" 2>/dev/null ||
      warn "Commentaire PR impossible (permissions token ?)"
  fi
fi

cat "$REPORT"

if [[ "$PII_FOUND" == "true" && "$FAIL_ON_PII" == "true" ]]; then
  err "PII détectées — fail_on_pii=true"
  exit 1
fi

exit 0
