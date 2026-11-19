#!/usr/bin/env bash
# Runs every check CI runs. It must pass before anything is called done.
# The tests need a Postgres they may create databases in, for example:
#   docker run --rm -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16
set -euo pipefail
cd "$(dirname "$0")/.."
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/postgres}"
./scripts/tests/bump-graph.sh
./scripts/tests/deploy.sh
python3 scripts/tests/photos.py
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
