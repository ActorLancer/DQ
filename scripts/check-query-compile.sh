#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[info] running query compile checks for platform-core/db"
if [[ -z "$(find .sqlx -type f -print -quit 2>/dev/null)" ]]; then
  echo "[error] missing .sqlx metadata; run DATABASE_URL=... cargo sqlx prepare --workspace first" >&2
  exit 1
fi

SQLX_OFFLINE=true cargo check -p db --features query-compile-check

echo "[ok] query compile check passed"
