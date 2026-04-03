#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
#
# Vérifie une archive CLI AEGIS signée avec cosign (sign-blob).
# Usage :
#   ./scripts/cosign-verify-aegis-cli.sh aegis-cli-v1.0.0-linux-amd64.tar.gz aegis-cli-v1.0.0-linux-amd64.tar.gz.cosign.bundle
#
# Pour une image Docker GHCR (pas le CLI fichier) :
#   cosign verify ghcr.io/<org>/aegis-gateway@<digest>
set -euo pipefail

FILE="${1:-}"
BUNDLE="${2:-}"

if [[ -z "$FILE" || -z "$BUNDLE" ]]; then
  echo "usage: $0 <archive.tar.gz> <fichier.cosign.bundle>" >&2
  echo "Exemple après téléchargement depuis une GitHub Release." >&2
  exit 1
fi

[[ -f "$FILE" && -f "$BUNDLE" ]] || { echo "Fichiers introuvables." >&2; exit 1; }

: "${COSIGN_CERT_IDENTITY_REGEX:?Définissez COSIGN_CERT_IDENTITY_REGEX (ex. https://github.com/<org>/<repo>/.*)}"
: "${COSIGN_CERT_OIDC_ISSUER_REGEX:?Définissez COSIGN_CERT_OIDC_ISSUER_REGEX (ex. https://token.actions.githubusercontent.com)}"

command -v cosign >/dev/null 2>&1 || { echo "cosign requis" >&2; exit 1; }

exec cosign verify-blob "$FILE" \
  --bundle="$BUNDLE" \
  --certificate-identity-regexp="$COSIGN_CERT_IDENTITY_REGEX" \
  --certificate-oidc-issuer-regexp="$COSIGN_CERT_OIDC_ISSUER_REGEX"
