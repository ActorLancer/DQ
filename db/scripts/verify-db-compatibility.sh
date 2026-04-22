#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> migration reset"
./db/scripts/migrate-reset.sh

echo "==> baseline seed apply"
./db/scripts/seed-up.sh

echo "==> verify baseline seeds"
./db/scripts/verify-seed-001.sh
./db/scripts/verify-seed-010-030.sh
./db/scripts/verify-seed-032.sh
./db/scripts/verify-seed-033.sh

echo "==> migration roundtrip drill"
./db/scripts/verify-migration-roundtrip.sh

echo "==> re-apply seed after roundtrip"
./db/scripts/seed-up.sh

echo "==> verify seeds after roundtrip"
./db/scripts/verify-seed-001.sh
./db/scripts/verify-seed-010-030.sh
./db/scripts/verify-seed-032.sh
./db/scripts/verify-seed-033.sh

echo "==> verify critical migration segments"
./db/scripts/verify-migration-065-068.sh
./db/scripts/verify-migration-070.sh

echo "==> status check"
status_output="$(./db/scripts/migrate-status.sh)"
echo "$status_output"
pending_count="$(printf '%s\n' "$status_output" | awk '/== pending up versions ==/{flag=1;next} flag && NF{print $1}' | wc -l | tr -d ' ')"
if [[ "$pending_count" != "0" ]]; then
  echo "[fail] compatibility check found pending migrations: ${pending_count}" >&2
  exit 1
fi

echo "[ok] db compatibility baseline verified"
