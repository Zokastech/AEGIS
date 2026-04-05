# AEGIS Gateway (Go)

Passerelle **REST** (Echo) + **gRPC** pour le moteur AEGIS (zokastech.fr).

## Architecture (DDD / hexagonale, progressive)

Les dépendances vont **vers l’intérieur** : le cœur métier ne dépend pas d’Echo ni du FFI.

| Couche | Rôle | Paquets (exemples) |
|--------|------|--------------------|
| **Domaine** | Règles et erreurs métier sans framework | `internal/domain` |
| **Application** | Cas d’usage, orchestration, **ports** (interfaces) | `internal/app` |
| **Infrastructure** | Implémentations des ports (circuit breaker, pool FFI, fichiers…) | `internal/infra`, `bridge`, `policy` |
| **Adaptateurs entrants** | HTTP / gRPC | `api/rest`, `api/grpc/grpcapi` |

Le flux **POST /v1/analyze** passe par le cas d’usage `internal/app.Analyze` (moteur + politiques + filtre RBAC injecté par le handler). Les références type *Domain-Driven Hexagon* (ports/adapters, inversion de dépendances) sont décrites par exemple dans [domain-driven-hexagon](https://github.com/sairyss/domain-driven-hexagon) (patterns transposables au Go).

Évolutions possibles : extraire d’autres handlers vers `internal/app`, faire implémenter les ports par des types dédiés sous `internal/infra`, et réutiliser le même use case côté gRPC.

## Choix Echo plutôt que Gin

- Middlewares intégrés (gzip, recovery, CORS) avec une chaîne claire.
- `echo.Context` homogène pour handlers et tests `httptest`.
- Performances comparables à Gin pour des API JSON de ce type.

## Démarrage (sans FFI — moteur mock)

Pratique pour les tests Go sans toolchain Rust :

```bash
cd aegis-gateway
go mod tidy
go run ./cmd/aegis-gateway --http-listen :8080 --grpc-listen :9090
```

Sans le build tag `aegisffi`, le binaire utilise **`MockEngine`** (`recognizer_name: "mock"`, réponses simplifiées).

Variables d’environnement : préfixe `AEGIS_` (via Viper), ex. `AEGIS_HTTP_LISTEN=:3000`.

## Moteur Rust réel (FFI / CGO)

1. `cargo build -p aegis-ffi --release` (produit `target/release/libaegis_ffi.{so,dylib}` à la racine du dépôt).
2. `CGO_ENABLED=1 go build -tags aegisffi -o aegis-gateway ./cmd/aegis-gateway` depuis `aegis-gateway/`.

Script tout-en-un à la racine du dépôt :

```bash
./scripts/build-gateway-ffi.sh
```

Environnement d’exécution : la lib dynamique doit être trouvée (`LD_LIBRARY_PATH` sur Linux, ou répertoire contenant `libaegis_ffi.dylib` sur macOS). Optionnel : JSON d’init moteur via la config gateway `engine_init_json` (voir `config`).

## Docker Compose (développement)

`docker-compose.dev.yml` / `make dev` : le service **aegis-gateway** utilise `docker/Dockerfile.dev.gateway` (Go + Rust + gcc). Au démarrage, `docker/scripts/dev-gateway-entry.sh` exécute `cargo build --release -p aegis-ffi` puis **air** avec `AEGIS_GATEWAY_FFI=1` (voir `scripts/air-build.sh` et `.air.toml`) : **moteur Rust réel**, plus de `MockEngine` dans ce flux.

Variables utiles dans le conteneur : `CGO_ENABLED=1`, `LD_LIBRARY_PATH=/work/target/release`.

Après une modification **uniquement** du code Rust, recompiler la bibliothèque puis relancer un build Go (air recharge souvent au prochain changement `.go`) :

```bash
docker compose -f docker-compose.dev.yml exec aegis-gateway bash -c 'cd /work && cargo build --release -p aegis-ffi'
```

## Image Docker (production)

`docker/Dockerfile.gateway` compile **aegis-ffi** puis la gateway avec **`-tags aegisffi`**, et livre `libaegis_ffi.so` + binaire dans une image **Debian slim** (`LD_LIBRARY_PATH=/opt/aegis`).

```bash
docker build -f docker/Dockerfile.gateway -t aegis/gateway:local .
```

## OpenAPI

Spec embarquée : `GET /v1/openapi.yaml` (fichier `api/rest/openapi.yaml` + annotations swag dans les handlers).

## gRPC

Le service utilise `google.protobuf.StringValue` comme enveloppe JSON (voir commentaires dans `api/grpc/aegis.proto`). Réflexion gRPC activée pour `grpcurl`.
