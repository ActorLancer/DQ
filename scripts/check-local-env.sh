#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${1:-部署脚本/docker-compose.local.yml}"
ENV_FILE="${2:-.env}"
ENV_TEMPLATE="${3:-.env.example}"
CHECK_MODE="${4:-runtime}"

fail() {
  echo "[error] $1" >&2
  exit 1
}

warn() {
  echo "[warn] $1"
}

info() {
  echo "[ok] $1"
}

DOCKER_CMD="${DOCKER_CMD:-docker}"

[ -f "$COMPOSE_FILE" ] || fail "Compose file not found: $COMPOSE_FILE"
info "Found compose file: $COMPOSE_FILE"

if [ ! -f "$ENV_TEMPLATE" ]; then
  fail "Env template not found: $ENV_TEMPLATE"
fi
info "Found template: $ENV_TEMPLATE"

if [ ! -f "$ENV_FILE" ]; then
  warn "Env file not found: $ENV_FILE"
  warn "Run: cp $ENV_TEMPLATE $ENV_FILE"
else
  info "Found env file: $ENV_FILE"
fi

if ! command -v "${DOCKER_CMD%% *}" >/dev/null 2>&1; then
  fail "docker is not installed or not in PATH"
fi
info "Docker is installed"

if ! ${DOCKER_CMD} compose version >/dev/null 2>&1; then
  fail "docker compose plugin is not available"
fi
info "docker compose is available"

if [ "${CHECK_MODE}" = "runtime" ]; then
  # Ensure the current user can talk to the Docker daemon (compose config works without it,
  # but up/pull/ps/logs require it).
  if ! ${DOCKER_CMD} ps >/dev/null 2>&1; then
    fail "Docker daemon is not reachable for current user. Fix: start Docker, or grant access to /var/run/docker.sock (e.g. add user to docker group, then re-login)."
  fi
  info "Docker daemon is reachable"
fi

echo "[done] Local environment looks ready"
