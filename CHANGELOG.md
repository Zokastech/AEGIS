# Changelog

**AEGIS by [zokastech.fr](https://zokastech.fr)** — format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versionnage [SemVer](https://semver.org/lang/fr/).

## [0.2.0] — 2026-11-07

Version de **durcissement sécurité**. Plusieurs correctifs changent le comportement par défaut :
lisez la section « Migration » de [RELEASE_NOTES.md](RELEASE_NOTES.md) avant de mettre à jour.

### Sécurité

- **Critique — gRPC sans authentification** : le listener gRPC (`:9090`) n’appliquait ni authentification ni RBAC ;
  `Deanonymize` et `UpdateConfig` étaient appelables par n’importe qui. Ajout d’intercepteurs unary/stream
  (mTLS vérifié > Bearer OIDC/JWT > clé API), matrice de permissions identique au REST, double approbation
  admin pour `Deanonymize`, méthodes inconnues refusées, TLS sur gRPC, réflexion désactivée en production,
  filtrage RBAC des types d’entités appliqué aussi en gRPC.
- **Critique — configuration de sécurité ignorée** : `security.yaml` et `--security-config` étaient fusionnés
  à la racine au lieu de `security.*` ; **aucun** réglage (mTLS requis, CORS, JWT/OIDC, chemin des clés,
  audit…) n’était appliqué, sans erreur. Les fichiers sont maintenant appliqués (test de non-régression).
- **Critique — image Docker** : l’image ne démarrait pas (`CMD ["-config", …]` refusé par pflag) et
  `security.docker.yaml` demandait `production: false` + `development.disable_auth: true`.
  L’image (et le chart Helm qui l’utilise) démarre désormais en mode *secure by default*.
- **Critique — clé privée TLS partagée** : `Dockerfile.gateway` générait une clé TLS au build (identique pour
  tous les utilisateurs de l’image publiée) et `aegis-gateway/data/tls/server.key` était versionnée.
  Clé retirée du dépôt, certificat généré au premier démarrage dans `/var/lib/aegis/tls`.
- **Critique — DoS du proxy LLM** : `ClassifyPath` se rappelait indéfiniment sur toute route inconnue
  (`/v1/embeddings`, `/v1/models`…) → *stack overflow* fatal du processus.
- **Élevée — fuite PII en streaming** : en mode `anonymize`, les requêtes `"stream": true` partaient vers le LLM
  sans anonymisation. Le prompt est maintenant anonymisé avant tout envoi.
- **Élevée — fail-open du mode `block`** : moteur AEGIS indisponible ⇒ la requête passait sans analyse.
  Désormais HTTP 503 (option explicite `fail_open: true`).
- **Élevée — révocation de clé API inopérante** : après `Generate`/`Rotate`, l’index interne pointait vers une
  copie obsolète ; une clé « révoquée » restait valide. Le pepper fourni par `AEGIS_API_KEY_PEPPER` était
  en outre réécrit sur disque à côté des empreintes. Écriture atomique en `0600`.
- **Élevée — crash du rate limiting** : maps de limiteurs écrites sans verrou (`concurrent map writes` fatal),
  croissance mémoire non bornée, secrets de clés API conservés en clair comme clés de map.
- **Élevée — contournement du rate limiting / usurpation d’IP** : `X-Forwarded-For` était accepté de n’importe
  quel client. Nouvelle option `security.trusted_proxies` (IP TCP directe par défaut).
- **Moyenne — JWT** : audience au format tableau non vérifiée, `exp` facultatif, `aud` absent accepté.
  Utilisation des validateurs `golang-jwt` (HS256/384/512 uniquement, `exp` obligatoire, iss/aud stricts).
- **Moyenne — mTLS** : le CN était lu dans `PeerCertificates` (non vérifiés) au lieu de `VerifiedChains` ;
  une CA client invalide n’entraînait qu’un avertissement (fail-open) → arrêt du démarrage.
- **Moyenne** : `PUT /v1/config` / `UpdateConfig` ne peuvent plus modifier `security.*`, les clés admin
  ni les adresses d’écoute ; `disable_auth` refusé si `production: true` ; secret JWT ≥ 32 caractères.
- **Moyenne** : limite de taille des corps HTTP (10 MiB, `security.max_body_bytes`) et des messages gRPC,
  `ReadHeaderTimeout` (anti-Slowloris) sur la passerelle et le proxy LLM, limites de lecture des réponses amont.
- **Moyenne** : en-tête `X-Aegis-Role: admin` (mode legacy) honoré uniquement avec `AEGIS_DEV_ALLOW_ROLE_HEADER=1` ;
  comparaison des clés admin legacy en temps constant.
- **Faible** : CORS ne combine plus jamais `*` et `credentials` ; CSP/Referrer-Policy/CORP sur l’API ;
  CSP sur le dashboard Nginx (y compris sur les assets statiques) ; `X-Request-ID` client filtré
  (injection de logs) ; échappement complet (`'`) dans l’export DPIA HTML ; neutralisation des
  séparateurs LEEF/CEF dans l’export SIEM ; webhooks d’alerte sans suivi de redirection ; page dashboard du
  proxy échappée ; le dashboard n’envoie plus la clé API vers une origine tierce.
- **Dépendances** : `rustls` 0.23.37 → ≥ 0.23.46, `rustls-webpki`, `webpki-roots`, `quick-xml` 0.41,
  `crossbeam-epoch`, `pyo3` 0.24 → 0.29 (RUSTSEC-2026-0176/0177) ; toolchain **Go 1.27.3**
  (la branche 1.25 n’est plus maintenue : ~50 avis stdlib) ; dashboard **Vite 7** + **Storybook 10**
  (2 critiques, 4 élevées) ; SDK Node : `express` 4.22.3, `fastify` 5 ; `onnx` ≥ 1.22 ;
  `pymdown-extensions` ≥ 11.0.1 ; Spring Boot 3.5.16 / Jackson 2.21.4 (SDK Java) ;
  images Node 24, Go 1.27, Rust 1.99, nginx 1.30, Alpine 3.24.

### Ajouté

- `aegis-keygen` : création / rotation / révocation / listing des clés API (secret affiché une seule fois),
  inclus dans l’image gateway.
- `audit.VerifyChain` : vérification de l’intégrité de la chaîne SHA-256 du journal d’audit.
- Options : `security.trusted_proxies`, `security.max_body_bytes`, `security.grpc.reflection`,
  `security.grpc.max_recv_msg_bytes`, `AEGIS_SECURITY_PRODUCTION`, `AEGIS_TLS_EXTRA_SANS` ;
  proxy LLM : `max_body_bytes`, `fail_open`, `engine.api_key_env` (clé API vers la passerelle désormais
  authentifiée), `engine.ca_file` (CA privée, sans jamais désactiver la vérification TLS).
- **Tests** : 138 nouveaux tests — gateway (auth REST/gRPC, RBAC, double approbation, clés API, JWT, mTLS,
  middlewares, audit, TLS, keygen), proxy LLM (0 → 7 paquets testés), chiffrement AES-GCM / hash (Rust),
  sûreté FFI (pointeurs nuls, UTF-8 invalide), SDK Python (sécurité), SDK Node (0 → 12 tests),
  dashboard (0 → 14 tests Vitest).
- CI : jobs `go-race`, `sdk-python`, `sdk-nodejs`, `dashboard` ; sécurité : `gitleaks`, `npm audit`,
  `govulncheck` sur le proxy LLM ; `.gitleaks.toml` et hook `pre-commit`.
- Documentation : [INSTALL.md](INSTALL.md), [RELEASE_NOTES.md](RELEASE_NOTES.md).

### Corrigé

- SDK Node.js : impossible à compiler (`napi-build = "2.16"` inexistant, crate hors `workspace.exclude`,
  `String::to_string_lossy`) ; addon introuvable (`*-gnu.node`) ; `entityType`/`recognizerName` toujours
  `undefined` et options `scoreThreshold`/`entities` ignorées (casse snake_case vs camelCase NAPI).
- Journal d’audit `aegis-gateway/data/audit/audit.jsonl` retiré du suivi git (donnée d’exécution).
- Image gateway : `libonnxruntime.so` n’était pas copiée dans l’image finale (`ORT_DYLIB_PATH` cassé).
- `release.yml` invalide (`secrets.*` interdit dans `jobs.<id>.if`) ; wheels PyPI produits hors de `dist/` ;
  crates non publiables (dépendances `path` sans `version`).
- `ort-sys` épinglé en `=2.0.0-rc.5` (rc.10 incompatible avec `ort` rc.5) ; avertissement clippy.
- Proxy LLM : `{"result": null}` du moteur considéré comme valide ; mode inconnu silencieusement traité
  comme `transparent` (désormais refusé au démarrage).
- Dashboard : erreurs ESLint des stories, lint de `storybook-static/`.

### Modifié (rupture)

- Image Docker / Helm : authentification **activée**, clés API dans `/var/lib/aegis/api_keys.yaml`,
  pepper **obligatoire** (`AEGIS_API_KEY_PEPPER`), `POSTGRES_PASSWORD` obligatoire dans `docker compose`.
- `PUT /v1/config` refuse les clés protégées (HTTP 400).
- gRPC : identifiants requis dans les métadonnées (`x-api-key`, `authorization`, `x-aegis-approval-1/2`).
- Helm : chart 0.4.0 / appVersion 0.2.0, tags d’image par défaut = `appVersion` (plus `latest`).

## [0.1.0]

Première version publique (moteur Rust, passerelle Go, SDK Python/Node/Java, dashboard, Helm).
