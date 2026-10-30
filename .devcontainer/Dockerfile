# Use Microsoft's official Rust Dev Container image (Debian Bookworm base)
# Comes pre-configured with a non-root 'vscode' user, git, and common utilities
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Avoid interactive prompts during apt-get package installation
ENV DEBIAN_FRONTEND=noninteractive

# 1. Install system dependencies required by the Abacus architecture:
# - protobuf-compiler: Compiles gRPC definitions for the API Router (Tonic/Prost)
# - postgresql-client: Provides 'psql' and 'pg_isready' to test your remote Postgres DB
# - clang & lld: Fast linker to speed up Rust compilation times
# - pkg-config & libssl-dev: Required by sqlx, reqwest, and crypto crates
# - python3, python3-pip, python3-venv: Powers Week 4-5 Reconciliation Pipeline
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    libprotobuf-dev \
    postgresql-client \
    clang \
    lld \
    pkg-config \
    libssl-dev \
    build-essential \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    curl \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# 2. Switch to the non-root 'vscode' user for development tooling installation
USER vscode

# 3. Install Rust components for linting, formatting, and IDE analysis
RUN rustup component add clippy rustfmt rust-src

# 4. Install sqlx-cli (Postgres only, for fast build) and cargo-watch
# - sqlx-cli: Runs migrations against your remote PostgreSQL database
# - cargo-watch: Automatically recompiles your Rust API when files change
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres --locked \
    && cargo install cargo-watch --locked

# 5. Install 'uv' (fast Python package & virtualenv manager) for Week 4-5 Recon Pipeline
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/home/vscode/.local/bin:${PATH}"

# Set the default working directory inside the container
WORKDIR /workspaces/abacus