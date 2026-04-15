#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${1:-docker-compose.local.profiled.yml}"
ENV_FILE="${2:-.env}"

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

[ -f "$COMPOSE_FILE" ] || fail "Compose file not found: $COMPOSE_FILE"
info "Found compose file: $COMPOSE_FILE"

if [ ! -f ".env.example" ]; then
  fail ".env.example not found in current directory"
fi
info "Found template: .env.example"

if [ ! -f "$ENV_FILE" ]; then
  warn "Env file not found: $ENV_FILE"
  warn "Run: cp .env.example $ENV_FILE"
else
  info "Found env file: $ENV_FILE"
fi

if ! command -v docker >/dev/null 2>&1; then
  fail "docker is not installed or not in PATH"
fi
info "Docker is installed"

if ! docker compose version >/dev/null 2>&1; then
  fail "docker compose plugin is not available"
fi
info "docker compose is available"

echo "[done] Local environment looks ready"
