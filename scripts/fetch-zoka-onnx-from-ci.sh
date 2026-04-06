#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Télécharge le bundle ZOKA-SENTINEL (aegis-ner-l3-onnx.tgz) depuis une exécution réussie du
# workflow « Helm & NER L3 pipeline » sur https://github.com/Zokastech/AEGIS
#
# Prérequis : GitHub CLI (`brew install gh` / https://cli.github.com) et `gh auth login`.
#
# Usage (à la racine du dépôt) :
#   ./scripts/fetch-zoka-onnx-from-ci.sh
#   ./scripts/fetch-zoka-onnx-from-ci.sh 1234567890   # run id précis (onglet Actions)
#
# Variables optionnelles :
#   AEGIS_GITHUB_REPO   défaut Zokastech/AEGIS
#   NER_MODELS_DIR      défaut ./models (créé si absent)
set -euo pipefail

REPO="${AEGIS_GITHUB_REPO:-Zokastech/AEGIS}"
OUT="${NER_MODELS_DIR:-./models}"
RUN_ID="${1:-}"

if ! command -v gh >/dev/null 2>&1; then
  echo "Erreur : la commande « gh » est requise (GitHub CLI). Voir https://cli.github.com" >&2
  exit 1
fi

resolve_run_id() {
  local b
  for b in master main; do
    local id
    id="$(gh run list --workflow=helm-training.yml --repo "$REPO" --branch "$b" --status success -L 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)"
    if [[ -n "$id" && "$id" != "null" ]]; then
      echo "$id"
      return 0
    fi
  done
  return 1
}

if [[ -z "$RUN_ID" ]]; then
  RUN_ID="$(resolve_run_id)" || {
    echo "Erreur : aucune exécution réussie trouvée pour helm-training.yml sur $REPO (branches master/main)." >&2
    echo "Ouvrez https://github.com/${REPO}/actions/workflows/helm-training.yml et relancez un workflow ou choisissez un run id." >&2
    exit 1
  }
fi

echo "Téléchargement artefact « aegis-ner-l3-onnx » (run $RUN_ID, $REPO) …"
tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

gh run download "$RUN_ID" --repo "$REPO" -n aegis-ner-l3-onnx -D "$tmpdir"

tgz="$(find "$tmpdir" -name 'aegis-ner-l3-onnx.tgz' -type f | head -n1)"
if [[ -z "$tgz" ]]; then
  echo "Erreur : aegis-ner-l3-onnx.tgz introuvable dans l’artefact téléchargé." >&2
  exit 1
fi

mkdir -p "$OUT"
mkdir -p "$tmpdir/ex"
tar -xzf "$tgz" -C "$tmpdir/ex"
ex="$tmpdir/ex"
if [[ -f "$ex/model_int8.onnx" ]]; then
  cp -f "$ex/model_int8.onnx" "$OUT/ner.onnx"
elif [[ -f "$ex/model.onnx" ]]; then
  cp -f "$ex/model.onnx" "$OUT/ner.onnx"
else
  echo "Erreur : archive sans model_int8.onnx ni model.onnx." >&2
  exit 1
fi
if [[ ! -f "$ex/tokenizer.json" ]]; then
  echo "Erreur : tokenizer.json manquant dans l’archive." >&2
  exit 1
fi
cp -f "$ex/tokenizer.json" "$OUT/tokenizer.json"

echo "OK — NER L3 prêt sous $OUT (ner.onnx, tokenizer.json). Ex. docker-compose : AEGIS_ENGINE_INIT_JSON={\"ner\":{\"model_path\":\"/work/models/ner.onnx\"}}"
