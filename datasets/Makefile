# AEGIS — zokastech.fr — Apache 2.0 / MIT

REPO_ROOT := $(abspath ..)
AEGIS_CLI ?= $(REPO_ROOT)/target/release/aegis
BENCH_LIMIT ?= 500

.PHONY: benchmark generate

generate:
	python3 generate_dataset.py --output generated/synthetic_pii.jsonl --n 10000 --seed 42

benchmark:
	python3 -m pip install -q -r requirements.txt
	@test -f $(AEGIS_CLI) || (cd $(REPO_ROOT) && cargo build -p aegis-cli --release)
	@test -f generated/synthetic_pii.jsonl || python3 generate_dataset.py --output generated/synthetic_pii.jsonl --n 10000 --seed 42
	python3 benchmark_vs_presidio.py --aegis-bin $(AEGIS_CLI) --output reports/benchmark_report.html --limit $(BENCH_LIMIT)
	@echo "Rapport HTML : datasets/reports/benchmark_report.html"
