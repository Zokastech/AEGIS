#!/usr/bin/env bash
# AEGIS — zokastech.fr — Apache 2.0 / MIT
#
# Signature Sigstore (cosign) en mode keyless (OIDC) ou clé statique.
# Usage :
#   export COSIGN_EXPERIMENTAL=1   # keyless local (interactive)
#   ./scripts/release-sign-cosign.sh path/to/aegis-cli-linux-amd64.tar.gz
#
# Sort : <fichier>.cosign.bundle (à joindre à la GitHub Release avec le fichier signé).
set -euo pipefail

[[ $# -ge 1 ]] || { echo "usage: $0 <fichier> [fichier ...]" >&2; exit 1; }

need_cmd() { command -v "$1" >/dev/null 2>&1; }
need_cmd cosign || { echo "Installez cosign : https://docs.sigstore.dev/cosign/installation/" >&2; exit 1; }

for f in "$@"; do
  [[ -f "$f" ]] || { echo "Fichier introuvable: $f" >&2; exit 1; }
  bundle="${f}.cosign.bundle"
  echo "Signing: $f → $bundle"
  cosign sign-blob --yes "$f" --bundle="$bundle"
done

echo "Vérification locale (keyless / certificat selon contexte) :"
echo "  cosign verify-blob <fichier> --bundle=<fichier>.cosign.bundle \\"
echo "    --certificate-identity-regexp='.*' --certificate-oidc-issuer-regexp='.*'"
echo "(Affinez les regex en production — voir SECURITY.md.)"
