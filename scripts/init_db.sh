#!/usr/bin/env bash
# scripts/init_db.sh
# Automatically switches to the Abacus project root, loads .env,
# checks the PostgreSQL server, and runs SQLx migrations.

# Stop the script immediately if any command fails
set -euo pipefail

# 1. Automatically find the Abacus workspace root (one level above /scripts)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Move into the project root directory
cd "${PROJECT_ROOT}"
echo "Working inside project root: ${PROJECT_ROOT}"

# 2. Check that the .env file exists in the project root
if [ ! -f ".env" ]; then
    echo "Error: .env file not found in ${PROJECT_ROOT}"
    echo "Please create a .env file in the root of your Abacus workspace."
    exit 1
fi

# 3. Export all variables from .env into the script's environment
set -a
source .env
set +a

# 4. If DATABASE_URL is missing or still has '{' placeholders, build it from individual variables
if [[ -z "${DATABASE_URL:-}" || "$DATABASE_URL" == *"{"* ]]; then
    echo "Assembling DATABASE_URL from individual .env variables..."
    DATABASE_URL="postgres://${DATABASE_USERNAME}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"
    export DATABASE_URL
fi

# 5. Test if the PostgreSQL database is reachable (conditionally)
echo "Checking PostgreSQL server connection..."
if command -v pg_isready >/dev/null 2>&1; then
    if pg_isready -d "$DATABASE_URL"; then
        echo "Success! PostgreSQL server is accepting connections."
    else
        echo "Error: Could not reach the PostgreSQL server."
        echo "Please verify DATABASE_URL in .env and ensure your Docker container is running."
        exit 1
    fi
else
    echo "pg_isready not installed on host. Skipping explicit ping and relying on sqlx for connection verification..."
fi

# 6. Create the database (if it doesn't exist yet) and apply migrations
echo "Ensuring database exists and applying migrations from ./migrations..."
sqlx database create
sqlx migrate run

echo "Abacus database initialization complete!"