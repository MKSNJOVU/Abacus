.PHONY: build test run clean up down

# --- Rust ---
build-rust:
	cargo build --workspace --release

test-rust:
	cargo test --workspace || true

run-api:
	cargo run -p api

# --- Python (WIP) ---
build-python:
	-cd reconciliation && pip install -e ".[dev]" || true

test-python:
	-cd reconciliation && pytest -v || true

run-recon:
	-cd reconciliation && python -m reconciliation.ingest || true

# --- Unified ---
# Removed Python targets temporarily until the codebase exists
build: build-rust
test: test-rust

up:
	docker compose up -d

down:
	docker compose down

clean:
	cargo clean
	find . -type d -name __pycache__ -exec rm -rf {} +
