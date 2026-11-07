# AEGIS v0.2.0 — « Secure by default »

**Date** : 7 novembre 2026 · **Type** : version de sécurité (mise à jour fortement recommandée)
**Artefacts** : CLI `aegis` (linux amd64/arm64, signée cosign), image `ghcr.io/zokastech/aegis-gateway:v0.2.0` (alias `0.2.0`, amd64/arm64, signée),
crates `aegis-*` 0.2.0, PyPI `aegis-pii` 0.2.0, npm `@aegis-pii/core` 0.2.0, chart Helm `aegis` 0.4.0, SBOM + provenance SLSA.

---

## En bref

Cette version corrige **5 vulnérabilités critiques** et **6 élevées** découvertes lors d’un audit complet
(passerelle Go, proxy LLM, moteur Rust, SDK, images, CI). Elle active l’authentification par défaut
partout et ajoute **138 nouveaux tests** (sécurité, régression, SDK).

| Gravité | Problème | Composant |
|---|---|---|
| Critique | API gRPC accessible **sans authentification** (y compris dé-anonymisation et changement de config) | gateway |
| Critique | Fichiers `security.yaml` / `--security-config` **silencieusement ignorés** (mTLS, CORS, JWT…) | gateway |
| Critique | Image Docker : démarrage impossible (`-config`) et config prévue **auth désactivée** | docker, helm |
| Critique | **Clé privée TLS** identique dans toutes les images + clé versionnée dans git | docker, dépôt |
| Critique | Toute route inconnue **fait planter** le proxy LLM (récursion infinie) | llm-proxy |
| Élevée | Requêtes LLM en **streaming envoyées sans anonymisation** | llm-proxy |
| Élevée | Mode `block` **fail-open** si le moteur est indisponible | llm-proxy |
| Élevée | **Clé API révoquée encore valide** ; pepper réécrit sur disque | gateway |
| Élevée | **Crash** concurrent du rate limiting + mémoire non bornée | gateway |
| Élevée | Rate limiting contournable via `X-Forwarded-For` usurpé | gateway |
| Élevée | Dépendances vulnérables : rustls/webpki, pyo3, Go stdlib (1.25 EOL), Vite/Storybook | toutes |

Détail complet : [CHANGELOG.md](CHANGELOG.md).

## Actions immédiates recommandées

1. **Considérez comme compromise** la clé `aegis-gateway/data/tls/server.key` présente dans l’historique git
   (commit `528999e`) et tout certificat généré par les images ≤ 0.1.0 : ne les utilisez plus, régénérez vos certificats.
2. **Restreignez l’accès au port gRPC 9090** des déploiements 0.1.0 jusqu’à la mise à jour.
3. Mettez à jour, puis **faites tourner vos clés API** (`aegis-keygen -rotate`) et le pepper si celui-ci
   a été stocké dans `api_keys.yaml`.
4. Si vous utilisez le proxy LLM en mode `anonymize` avec `"stream": true`, vérifiez les journaux de votre
   fournisseur LLM : des PII ont pu être transmises en clair.

## Migration depuis 0.1.0

| Avant (0.1.0) | Maintenant (0.2.0) | Que faire |
|---|---|---|
| Image Docker sans auth | Auth activée (clés API + RBAC) | Définir `AEGIS_API_KEY_PEPPER`, créer des clés avec `aegis-keygen` ([INSTALL.md §2](INSTALL.md#2-installation-docker-compose-recommandée)) |
| `POSTGRES_PASSWORD` par défaut `aegis-change-me` | Obligatoire | Le renseigner dans `.env` |
| Cert TLS dans `/etc/aegis/tls/*.pem` | Auto-généré dans `/var/lib/aegis/tls/` | Monter vos propres `server.crt`/`server.key` en production |
| `api_keys.yaml` dans `/etc/aegis/` | `/var/lib/aegis/api_keys.yaml` (volume) | Déplacer le fichier ou ajuster `security.api_keys.file_path` |
| gRPC ouvert, en clair | gRPC authentifié + TLS | Envoyer `x-api-key` (ou `authorization`) en métadonnée ; `x-aegis-approval-1/2` pour `Deanonymize` |
| `security.yaml` / `--security-config` sans effet | **Appliqués** | Relire vos fichiers : des réglages oubliés (ex. `mtls.require`) prennent effet |
| `disable_auth: true` + `production: true` accepté | Refus de démarrer | En dev : `AEGIS_SECURITY_PRODUCTION=false` |
| `X-Forwarded-For` toujours pris en compte | Ignoré sauf proxys de confiance | Lister vos reverse-proxys dans `security.trusted_proxies` |
| `PUT /v1/config` pouvait modifier `security.*` | HTTP 400 | Modifier la sécurité par fichier + redémarrage |
| JWT sans `exp` / `aud` tableau accepté | `exp` requis, `aud` strict, secret ≥ 32 car. | Ré-émettre des jetons conformes |
| `X-Aegis-Role: admin` (legacy) | Ignoré | Dev uniquement : `AEGIS_DEV_ALLOW_ROLE_HEADER=1` |
| Proxy LLM : moteur KO ⇒ requête transmise | HTTP 503 en mode `block` | `fail_open: true` si vous assumez ce risque |
| Helm `tag: latest` | `tag` = `appVersion` (0.2.0) | Rien ; épingler explicitement si besoin |

## Nouveautés

- **`aegis-keygen`** : gestion des clés API (création, rotation, révocation, listing) — le secret n’est
  affiché qu’une fois et seule son empreinte SHA-256 pepperisée est stockée.
- **Vérification d’intégrité du journal d’audit** (`audit.VerifyChain`) : détection des lignes modifiées ou supprimées.
- **SDK Node.js réparé** : il ne compilait pas en 0.1.0 et renvoyait `entityType: undefined`.
- **CI renforcée** : tests avec détecteur de courses Go, SDK Python/Node, dashboard (Vitest), gitleaks, npm audit.

## Couverture de tests

| Composant | 0.1.0 | 0.2.0 |
|---|---|---|
| Gateway Go | 6 paquets testés | 13 paquets (auth REST + gRPC, RBAC, clés API, JWT, mTLS, middlewares, audit, TLS, keygen) |
| Proxy LLM | 0 | 7 paquets (anonymisation, streaming, fail-closed, DoS, fournisseurs OpenAI/Anthropic) |
| Moteur Rust | ~400 tests | + AES-GCM (nonce unique, falsification, AAD), hash salé, sûreté FFI |
| SDK Python | 7 | 18 |
| SDK Node.js | 0 (non compilable) | 12 |
| Dashboard | 0 | 14 (Vitest) |

## Limites connues

- `transformers` < 5 (pipeline d’entraînement uniquement) porte des avis PYSEC-2025-217 / 2026-2290 / 2026-4174 ;
  la montée en 5.x est prévue pour 0.3.0. Ne chargez que des modèles de confiance.
- Avis « unmaintained » restants : `atomic-polyfill`, `unsafe-libyaml` (via `serde_yaml`) — aucune vulnérabilité exploitable connue.
- En mode `anonymize` + streaming, les pseudonymes ne sont **pas** restaurés dans la réponse diffusée.
- Le SDK Java n’a pas pu être testé dans l’environnement de préparation (Maven Central inaccessible) :
  le pont JNI compile, la CI du dépôt reste la référence.

## Vérifier les artefacts

```bash
cosign verify ghcr.io/zokastech/aegis-gateway:v0.2.0 \
  --certificate-identity-regexp 'https://github.com/Zokastech/AEGIS/.github/workflows/release.yml@refs/tags/v0.2.0' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
export COSIGN_CERT_IDENTITY_REGEX='https://github.com/Zokastech/AEGIS/.*' COSIGN_CERT_OIDC_ISSUER_REGEX='https://token.actions.githubusercontent.com'
./scripts/cosign-verify-aegis-cli.sh aegis-cli-v0.2.0-linux-amd64.tar.gz aegis-cli-v0.2.0-linux-amd64.tar.gz.cosign.bundle
```

Installation : voir **[INSTALL.md](INSTALL.md)**.
