.PHONY: build test run clean up down

# --- Rust ---
build-rust:
	cargo build --workspace --release

test-rust:
	cargo test --workspace

run-api:
	cargo run -p api

# --- Python ---
build-python:
	cd reconciliation && pip install -e ".[dev]"

test-python:
	cd reconciliation && pytest -v

run-recon:
	cd reconciliation && python -m reconciliation.ingest

# --- Unified ---
build: build-rust build-python
test: test-rust test-python

up:
	docker compose up -d

down:
	docker compose down

clean:
	cargo clean
	find . -type d -name __pycache__ -exec rm -rf {} +