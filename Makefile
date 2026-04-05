# AEGIS — zokastech.fr — https://zokastech.fr — Apache 2.0 / MIT
# Compatible GNU Make sur Linux, macOS et Windows (WSL2 / Git Bash).

COMPOSE_DEV := docker compose -f docker-compose.dev.yml

.PHONY: dev build test lint bench benchmark docker docker-dev docs clean gateway gateway-ffi

# Démarre Rust (cargo-watch), Go (air), Vite, Redis, Postgres, Prometheus (9090), Grafana (3334).
dev:
	@test -f .env || (echo "Copiez .env.example vers .env (optionnel mais recommandé)." && true)
	$(COMPOSE_DEV) up --build

# Build release local (hors conteneur).
build:
	cargo build --release
	cd aegis-gateway && go build -o ../target/aegis-gateway .

test:
	cargo test --workspace
	cd aegis-policy && go test ./...
	cd aegis-gateway && go test ./...

lint:
	cargo fmt --check
	cargo clippy --workspace -- -D warnings
	cd aegis-policy && go vet ./...
	cd aegis-gateway && go vet ./...
	cd aegis-gateway && test -z "$$(gofmt -l .)"
	@command -v npm >/dev/null 2>&1 && cd aegis-dashboard && npx --yes tsc --noEmit || true

bench:
	@echo "AEGIS — benchmarks (Criterion aegis-benchmarks + rapport + hyperfine + Presidio optionnel)"
	@command -v python3 >/dev/null 2>&1 || (echo "python3 requis pour le rapport" && exit 1)
	cargo build -p aegis-cli --release
	cargo bench -p aegis-benchmarks
	cargo run -p aegis-benchmarks --release --bin aegis-memory-rusage -- 2000 | tee benchmarks/results/memory_rusage_last.txt || true
	python3 -m pip install -q -r benchmarks/scripts/requirements-bench.txt 2>/dev/null || true
	python3 benchmarks/scripts/generate_report.py
	@chmod +x benchmarks/scripts/hyperfine_cli.sh benchmarks/scripts/hf_aegis_once.sh 2>/dev/null || true
	@benchmarks/scripts/hyperfine_cli.sh || true
	@python3 benchmarks/scripts/run_presidio_compare.py || true
	@echo "Rapport principal : benchmarks/reports/performance_report.html"
	@echo "Copie doc       : docs/performance/report.html"

# Benchmark PII : AEGIS vs Presidio (datasets + rapport HTML)
benchmark:
	@$(MAKE) -C datasets benchmark

# Image de production (passerelle Go + lib aegis-ffi, voir docker/Dockerfile.gateway).
docker:
	docker build -f docker/Dockerfile.gateway -t aegis-gateway:local .

# Binaire local avec moteur Rust (CGO + tag aegisffi).
gateway-ffi:
	./scripts/build-gateway-ffi.sh

# Rebuild uniquement les images du compose dev (sans démarrer).
docker-dev:
	$(COMPOSE_DEV) build

docs:
	@echo "Documentation MkDocs — module 14 du cahier (mkdocs build)."
	@command -v mkdocs >/dev/null 2>&1 && mkdocs build -f docs/mkdocs.yml -d site 2>/dev/null || echo "Installez mkdocs ou ajoutez docs/mkdocs.yml."

clean:
	cargo clean
	rm -f target/aegis-gateway aegis-gateway/aegis-gateway
	rm -rf aegis-gateway/tmp aegis-gateway/air.log
	$(COMPOSE_DEV) down -v 2>/dev/null || true

gateway:
	cd aegis-gateway && go run .
