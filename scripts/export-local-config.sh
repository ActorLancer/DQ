#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"
OUT_DIR="${OUT_DIR:-fixtures/local/config-snapshots}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
TARGET_DIR="${OUT_DIR}/${STAMP}"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/export-local-config.sh [--out-dir <path>] [--stamp <id>]

Description:
  Export resolved docker compose configuration snapshots as read-only files.
  Outputs are generated for:
    - default profiles
    - core profile
    - demo profile
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      OUT_DIR="$2"
      shift 2
      ;;
    --stamp)
      STAMP="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[fail] unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

TARGET_DIR="${OUT_DIR}/${STAMP}"

command -v docker >/dev/null 2>&1 || { echo "[fail] docker not found" >&2; exit 1; }
docker compose version >/dev/null 2>&1 || { echo "[fail] docker compose not available" >&2; exit 1; }

mkdir -p "${TARGET_DIR}"

export_snapshot() {
  local profile_label="$1"
  local profiles="$2"
  local out_file="${TARGET_DIR}/compose.${profile_label}.resolved.yml"

  if [[ -n "${profiles}" ]]; then
    COMPOSE_PROFILES="${profiles}" docker compose \
      --env-file "${COMPOSE_ENV_FILE}" \
      -f "${COMPOSE_FILE}" config > "${out_file}"
  else
    docker compose \
      --env-file "${COMPOSE_ENV_FILE}" \
      -f "${COMPOSE_FILE}" config > "${out_file}"
  fi

  chmod 0444 "${out_file}"
  echo "[ok] exported ${out_file}"
}

export_snapshot "default" ""
export_snapshot "core" "core"
export_snapshot "demo" "demo"

git_ref="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
meta_file="${TARGET_DIR}/meta.txt"
{
  echo "generated_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "compose_file=${COMPOSE_FILE}"
  echo "compose_env_file=${COMPOSE_ENV_FILE}"
  echo "git_ref=${git_ref}"
  echo "profiles=default,core,demo"
} > "${meta_file}"
chmod 0444 "${meta_file}"
echo "[ok] exported ${meta_file}"

echo "[done] local config snapshot exported to ${TARGET_DIR}"
