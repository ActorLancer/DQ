#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[info] running query compile checks for platform-core/db"
cargo check -p db --features query-compile-check

echo "[ok] query compile check passed"
