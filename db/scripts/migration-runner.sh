#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"
MANIFEST_PATH="${MANIFEST_PATH:-db/migrations/v1/manifest.csv}"
DRY_RUN="false"

usage() {
  cat <<'EOF'
Usage: db/scripts/migration-runner.sh <up|down|status> [--manifest <path>] [--dry-run]
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

if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "[error] manifest not found: $MANIFEST_PATH" >&2
  exit 1
fi

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q)

ensure_history_table() {
  "${PSQL[@]}" <<'SQL'
CREATE TABLE IF NOT EXISTS public.schema_migration_history (
  id BIGSERIAL PRIMARY KEY,
  version TEXT NOT NULL,
  name TEXT NOT NULL,
  direction TEXT NOT NULL CHECK (direction IN ('up', 'down')),
  checksum_sha256 TEXT NOT NULL,
  executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (version, direction)
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

latest_direction() {
  local version="$1"
  query_single "SELECT direction FROM public.schema_migration_history WHERE version = '$version' ORDER BY executed_at DESC, id DESC LIMIT 1;"
}

latest_up_checksum() {
  local version="$1"
  query_single "SELECT checksum_sha256 FROM public.schema_migration_history WHERE version = '$version' AND direction = 'up' ORDER BY executed_at DESC, id DESC LIMIT 1;"
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
  local direction="$3"
  local checksum="$4"
  if [[ "$DRY_RUN" == "true" ]]; then
    echo "[dry-run] record history version=$version direction=$direction checksum=$checksum"
    return 0
  fi
  "${PSQL[@]}" -c "INSERT INTO public.schema_migration_history (version, name, direction, checksum_sha256) VALUES ('$version', '$name', '$direction', '$checksum') ON CONFLICT (version, direction) DO UPDATE SET name = EXCLUDED.name, checksum_sha256 = EXCLUDED.checksum_sha256, executed_at = NOW();"
}

run_up() {
  ensure_history_table
  tail -n +2 "$MANIFEST_PATH" | while IFS=, read -r version up_sql down_sql; do
    [[ -n "$version" ]] || continue
    if [[ ! -f "$up_sql" ]]; then
      echo "[error] upgrade sql missing: $up_sql" >&2
      exit 1
    fi

    checksum="$(calc_sha256 "$up_sql")"
    existing_checksum="$(latest_up_checksum "$version")"
    if [[ -n "$existing_checksum" && "$existing_checksum" != "$checksum" ]]; then
      echo "[error] checksum drift detected for version=$version (recorded=$existing_checksum current=$checksum)" >&2
      exit 1
    fi

    current_direction="$(latest_direction "$version")"
    if [[ "$current_direction" == "up" ]]; then
      echo "[skip] version=$version already applied"
      continue
    fi

    run_file "$up_sql"
    record_history "$version" "$(basename "$up_sql")" "up" "$checksum"
  done
}

run_down() {
  ensure_history_table
  tail -n +2 "$MANIFEST_PATH" | sort -r | while IFS=, read -r version up_sql down_sql; do
    [[ -n "$version" ]] || continue
    if [[ ! -f "$down_sql" ]]; then
      echo "[error] downgrade sql missing: $down_sql" >&2
      exit 1
    fi

    current_direction="$(latest_direction "$version")"
    if [[ "$current_direction" != "up" ]]; then
      echo "[skip] version=$version not applied, skip downgrade"
      continue
    fi

    checksum="$(calc_sha256 "$down_sql")"
    run_file "$down_sql"
    record_history "$version" "$(basename "$down_sql")" "down" "$checksum"
  done
}

show_status() {
  ensure_history_table
  echo "== applied migrations =="
  "${PSQL[@]}" -c "SELECT version, direction, checksum_sha256, executed_at FROM public.schema_migration_history ORDER BY version, direction;"
  echo "== pending up versions =="
  tail -n +2 "$MANIFEST_PATH" | while IFS=, read -r version up_sql down_sql; do
    current_direction="$(latest_direction "$version")"
    if [[ "$current_direction" != "up" ]]; then
      echo "$version $(basename "$up_sql")"
    fi
  done
}

case "$DIRECTION" in
  up)
    run_up
    ;;
  down)
    run_down
    ;;
  status)
    show_status
    ;;
  *)
    echo "[error] unknown direction: $DIRECTION" >&2
    usage
    exit 1
    ;;
esac
