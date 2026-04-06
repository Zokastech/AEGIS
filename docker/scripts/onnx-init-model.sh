#!/bin/sh
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Télécharge ZOKA-SENTINEL (bundle .tgz) ou un seul fichier .onnx dans MODELS_DIR (défaut /models).
set -eu

MODELS_DIR="${MODELS_DIR:-/models}"
mkdir -p "$MODELS_DIR"

if [ -s "$MODELS_DIR/ner.onnx" ] && [ -s "$MODELS_DIR/tokenizer.json" ]; then
  echo "[onnx-init] ner.onnx et tokenizer.json sont déjà présents — rien à faire."
  exit 0
fi

download_and_extract_bundle() {
  apk add --no-cache curl ca-certificates >/dev/null
  _url="$1"
  echo "[onnx-init] Téléchargement bundle ZOKA-SENTINEL depuis ${_url}"
  _tmp="$MODELS_DIR/.zoka-sentinel-bundle.tgz"
  curl -fsSL "$_url" -o "$_tmp"
  _ex="$(mktemp -d)"
  tar -xzf "$_tmp" -C "$_ex"
  if [ -f "$_ex/model_int8.onnx" ]; then
    cp "$_ex/model_int8.onnx" "$MODELS_DIR/ner.onnx"
  elif [ -f "$_ex/model.onnx" ]; then
    cp "$_ex/model.onnx" "$MODELS_DIR/ner.onnx"
  else
    echo "[onnx-init] ERREUR : archive sans model_int8.onnx ni model.onnx." >&2
    exit 1
  fi
  if [ ! -f "$_ex/tokenizer.json" ]; then
    echo "[onnx-init] ERREUR : tokenizer.json manquant dans l'archive." >&2
    exit 1
  fi
  cp "$_ex/tokenizer.json" "$MODELS_DIR/tokenizer.json"
  if [ -f "$_ex/zoka_sentinel_manifest.json" ]; then
    cp "$_ex/zoka_sentinel_manifest.json" "$MODELS_DIR/zoka_sentinel_manifest.json"
  fi
  rm -rf "$_ex" "$_tmp"
  echo "[onnx-init] ZOKA-SENTINEL : ner.onnx + tokenizer.json installés sous $MODELS_DIR"
}

if [ -n "${ZOKA_SENTINEL_BUNDLE_URL:-}" ]; then
  download_and_extract_bundle "$ZOKA_SENTINEL_BUNDLE_URL"
elif [ "${ZOKA_SENTINEL_FETCH_LATEST:-0}" = "1" ] && [ -n "${ZOKA_SENTINEL_REPO:-}" ]; then
  apk add --no-cache curl ca-certificates jq >/dev/null
  echo "[onnx-init] Résolution du dernier release GitHub : ${ZOKA_SENTINEL_REPO}"
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    _json="$(curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" \
      "https://api.github.com/repos/${ZOKA_SENTINEL_REPO}/releases/latest")"
  else
    _json="$(curl -fsSL "https://api.github.com/repos/${ZOKA_SENTINEL_REPO}/releases/latest")"
  fi
  _url="$(echo "$_json" | jq -r '.assets[]? | select(.name == "aegis-ner-l3-onnx.tgz") | .browser_download_url' | head -n1)"
  if [ -z "$_url" ] || [ "$_url" = "null" ]; then
    echo "[onnx-init] ERREUR : aucun asset « aegis-ner-l3-onnx.tgz » dans le dernier release de ${ZOKA_SENTINEL_REPO}." >&2
    echo "[onnx-init] Astuce : attachez le .tgz à une Release ou utilisez ZOKA_SENTINEL_BUNDLE_URL (URL directe)." >&2
    exit 1
  fi
  download_and_extract_bundle "$_url"
elif [ -n "${NER_ONNX_URL:-}" ]; then
  apk add --no-cache curl ca-certificates >/dev/null
  echo "[onnx-init] Téléchargement ONNX seul (pas de tokenizer depuis cette URL)."
  curl -fsSL "$NER_ONNX_URL" -o "$MODELS_DIR/ner.onnx"
  echo "[onnx-init] ner.onnx écrit sous $MODELS_DIR (ajoutez tokenizer.json à la main pour L3 complet)."
else
  echo "[onnx-init] Aucune de ZOKA_SENTINEL_BUNDLE_URL, ZOKA_SENTINEL_FETCH_LATEST+REPO, NER_ONNX_URL — core en L1+L2 sans bundle NER."
fi

exit 0
