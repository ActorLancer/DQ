#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"
MANIFEST_PATH="${MANIFEST_PATH:-db/seeds/manifest.csv}"
DRY_RUN="false"

usage() {
  cat <<'EOF'
Usage: db/scripts/seed-runner.sh up [--manifest <path>] [--dry-run]
EOF
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

DIRECTION="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest)
      MANIFEST_PATH="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN="true"
      shift
      ;;
    *)
      echo "[error] unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "$DIRECTION" != "up" ]]; then
  echo "[error] unknown direction: $DIRECTION (only up supported)" >&2
  usage
  exit 1
fi

if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "[error] seed manifest not found: $MANIFEST_PATH" >&2
  exit 1
fi

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q)

ensure_seed_history() {
  "${PSQL[@]}" <<'SQL'
CREATE TABLE IF NOT EXISTS public.seed_history (
  id BIGSERIAL PRIMARY KEY,
  version TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  checksum_sha256 TEXT NOT NULL,
  executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SQL
}

calc_sha256() {
  local file="$1"
  sha256sum "$file" | awk '{print $1}'
}

query_single() {
  local sql="$1"
  "${PSQL[@]}" -tAc "$sql"
}

run_file() {
  local file="$1"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] $file"
    return 0
  fi
  echo "[run] $file"
  "${PSQL[@]}" -f "$file"
}

record_history() {
  local version="$1"
  local name="$2"
  local checksum="$3"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] record seed version=$version checksum=$checksum"
    return 0
  fi
  "${PSQL[@]}" -c "INSERT INTO public.seed_history (version, name, checksum_sha256) VALUES ('$version', '$name', '$checksum') ON CONFLICT (version) DO UPDATE SET name = EXCLUDED.name, checksum_sha256 = EXCLUDED.checksum_sha256, executed_at = NOW();"
}

run_up() {
  ensure_seed_history
  tail -n +2 "$MANIFEST_PATH" | while IFS=, read -r version seed_sql; do
    [[ -n "$version" ]] || continue
    if [[ ! -f "$seed_sql" ]]; then
      echo "[error] seed sql missing: $seed_sql" >&2
      exit 1
    fi
    checksum="$(calc_sha256 "$seed_sql")"
    existing_checksum="$(query_single "SELECT checksum_sha256 FROM public.seed_history WHERE version = '$version' LIMIT 1;")"
    if [[ -n "$existing_checksum" && "$existing_checksum" != "$checksum" ]]; then
      echo "[error] seed checksum drift detected for version=$version (recorded=$existing_checksum current=$checksum)" >&2
      exit 1
    fi
    run_file "$seed_sql"
    record_history "$version" "$(basename "$seed_sql")" "$checksum"
  done
}

run_up
