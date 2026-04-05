# AEGIS — zokastech.fr — https://zokastech.fr
# just (https://github.com/casey/just) — même surface que le Makefile

compose := "docker compose -f docker-compose.dev.yml"

# Stack dev Docker : Rust cargo-watch, Go air, Vite, Redis, Postgres
dev:
	@test -f .env || echo "Astuce : cp .env.example .env"
	{{compose}} up --build

build:
	cargo build --release
	cd aegis-gateway && go build -o ../target/aegis-gateway ./cmd/aegis-gateway

test:
	cargo test --workspace
	cd aegis-policy && go test ./...
	cd aegis-gateway && go test ./...

lint:
	cargo fmt --check
	cargo clippy --workspace -- -D warnings
	cd aegis-policy && go vet ./...
	cd aegis-gateway && go vet ./...
	cd aegis-gateway && test -z "$(gofmt -l .)"
	command -v npm >/dev/null 2>&1 && cd aegis-dashboard && npx --yes tsc --noEmit || true

bench:
	cargo bench --workspace || true

docker:
	docker build -f docker/Dockerfile.gateway -t aegis-gateway:local .

docker-dev:
	{{compose}} build

docs:
	command -v mkdocs >/dev/null 2>&1 && mkdocs build -f docs/mkdocs.yml -d site || echo "MkDocs non installé ou mkdocs.yml absent."

clean:
	cargo clean
	rm -f target/aegis-gateway aegis-gateway/aegis-gateway
	rm -rf aegis-gateway/tmp aegis-gateway/air.log
	{{compose}} down -v || true

gateway:
	cd aegis-gateway && go run ./cmd/aegis-gateway
