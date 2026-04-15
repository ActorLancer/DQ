#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker/docker-compose.local.yml}"
COMPOSE_ENV_FILE="${COMPOSE_ENV_FILE:-infra/docker/.env.local}"
FABRIC_COMPOSE_FILE="${FABRIC_COMPOSE_FILE:-infra/fabric/docker-compose.fabric.local.yml}"
FABRIC_STATE_DIR="${FABRIC_STATE_DIR:-infra/fabric/state}"

MODE="dry-run"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/prune-local.sh [--dry-run] [--force]

Description:
  Safely clean local resources for this repository only:
  - compose project containers/volumes/networks from infra/docker/docker-compose.local.yml
  - fabric local compose resources from infra/fabric/docker-compose.fabric.local.yml
  - generated fabric state under infra/fabric/state/*

Options:
  --dry-run   Show what would be removed (default).
  --force     Execute cleanup.
  -h, --help  Show help.
EOF
}

log() { echo "[info] $*"; }
ok() { echo "[ok]   $*"; }
warn() { echo "[warn] $*"; }
fail() { echo "[fail] $*" >&2; exit 1; }

for arg in "$@"; do
  case "$arg" in
    --dry-run) MODE="dry-run" ;;
    --force) MODE="force" ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: ${arg}" ;;
  esac
done

command -v docker >/dev/null 2>&1 || fail "docker not found"
docker compose version >/dev/null 2>&1 || fail "docker compose not available"
docker info >/dev/null 2>&1 || fail "docker daemon is not reachable for current user"

PROJECT_NAME="$(docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" config | sed -n 's/^name: //p' | head -n1)"
if [[ -z "${PROJECT_NAME}" || "${PROJECT_NAME}" == "null" ]]; then
  fail "unable to resolve compose project name from ${COMPOSE_FILE}"
fi

log "Target project: ${PROJECT_NAME}"
log "Mode: ${MODE}"

mapfile -t project_volumes < <(docker volume ls --filter "label=com.docker.compose.project=${PROJECT_NAME}" --format '{{.Name}}')
mapfile -t project_networks < <(docker network ls --filter "label=com.docker.compose.project=${PROJECT_NAME}" --format '{{.Name}}')

show_list() {
  local title="$1"
  shift
  if [[ "$#" -eq 0 ]]; then
    echo "  - ${title}: none"
    return
  fi
  echo "  - ${title}:"
  for item in "$@"; do
    echo "    - ${item}"
  done
}

echo "[plan] Resources scoped to project '${PROJECT_NAME}'"
show_list "volumes" "${project_volumes[@]}"
show_list "networks" "${project_networks[@]}"
echo "  - fabric state dir: ${FABRIC_STATE_DIR}"

if [[ "${MODE}" != "force" ]]; then
  warn "dry-run only; add --force to execute cleanup."
  exit 0
fi

log "Stopping local compose stack and removing project-scoped resources..."
docker compose --env-file "${COMPOSE_ENV_FILE}" -f "${COMPOSE_FILE}" down -v --remove-orphans
ok "local compose resources removed"

if [[ -f "${FABRIC_COMPOSE_FILE}" ]]; then
  log "Stopping fabric local compose..."
  docker compose -f "${FABRIC_COMPOSE_FILE}" down -v --remove-orphans || true
  ok "fabric compose cleanup done"
fi

if [[ -d "${FABRIC_STATE_DIR}" ]]; then
  log "Pruning fabric state under ${FABRIC_STATE_DIR}"
  find "${FABRIC_STATE_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
  mkdir -p "${FABRIC_STATE_DIR}"/{ca,orderer,peer,channel,chaincode}
  ok "fabric state reset"
fi

ok "prune-local completed"
