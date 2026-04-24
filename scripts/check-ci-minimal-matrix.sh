#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

log() {
  echo "[info] $*"
}

ok() {
  echo "[ok]   $*"
}

fail() {
  echo "[fail] $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

run_rust() {
  require_cmd cargo

  log "running Rust lint/test lane"
  cargo fmt --all --check
  cargo check -p platform-core
  cargo test -p platform-core
  ok "Rust lint/test lane passed"
}

run_ts() {
  require_cmd pnpm

  export CI="${CI:-1}"
  export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD="${PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:-1}"

  log "installing pnpm workspace dependencies"
  pnpm install --frozen-lockfile

  log "running TypeScript lint/typecheck"
  pnpm lint
  pnpm typecheck

  log "running TypeScript unit tests"
  pnpm --filter @datab/sdk-ts test
  pnpm --filter @datab/portal-web test:unit
  pnpm --filter @datab/console-web test:unit
  ok "TypeScript lint/test lane passed"
}

run_go_module() {
  local module_path="$1"

  log "running Go build/test in ${module_path}"
  pushd "${module_path}" >/dev/null
  go build ./...
  go test ./...
  popd >/dev/null
}

run_go() {
  require_cmd go

  # shellcheck disable=SC1091
  source "${ROOT_DIR}/scripts/go-env.sh"

  run_go_module "${ROOT_DIR}/services/fabric-adapter"
  run_go_module "${ROOT_DIR}/services/fabric-event-listener"
  run_go_module "${ROOT_DIR}/services/fabric-ca-admin"
  run_go_module "${ROOT_DIR}/infra/fabric/chaincode/datab-audit-anchor"
  ok "Go build/test lane passed"
}

run_migration() {
  require_cmd bash
  require_cmd cargo
  require_cmd curl
  require_cmd docker
  require_cmd psql
  require_cmd rg

  log "running migration check lane"
  ENV_FILE=infra/docker/.env.local bash ./scripts/check-migration-smoke.sh
  ok "Migration check lane passed"
}

run_openapi() {
  require_cmd bash
  require_cmd git
  require_cmd pnpm

  log "running OpenAPI check lane"
  pnpm install --frozen-lockfile
  pnpm --filter @datab/sdk-ts openapi:generate
  git diff --exit-code -- packages/sdk-ts/src/generated \
    || fail "sdk generated files drift from packages/openapi; run pnpm --filter @datab/sdk-ts openapi:generate"
  bash ./scripts/check-openapi-schema.sh
  ok "OpenAPI check lane passed"
}

TARGET="${1:-all}"

case "${TARGET}" in
  rust)
    run_rust
    ;;
  ts)
    run_ts
    ;;
  go)
    run_go
    ;;
  migration)
    run_migration
    ;;
  openapi)
    run_openapi
    ;;
  all)
    run_rust
    run_ts
    run_go
    run_migration
    run_openapi
    ok "TEST-015 CI minimal matrix checker passed"
    ;;
  *)
    fail "unknown lane '${TARGET}' (expected: rust|ts|go|migration|openapi|all)"
    ;;
esac
