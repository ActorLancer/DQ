#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

MODE="${CANONICAL_CHECK_MODE:-full}"
ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
TOPIC_CATALOG="infra/kafka/topics.v1.json"
TESTCASE_README="docs/05-test-cases/README.md"
PORT_MATRIX_DOC="docs/04-runbooks/port-matrix.md"
LOCAL_STARTUP_DOC="docs/04-runbooks/local-startup.md"

fail() {
  echo "[fail] $*" >&2
  exit 1
}

ok() {
  echo "[ok]   $*"
}

log() {
  echo "[info] $*"
}

command -v jq >/dev/null 2>&1 || fail "jq not found"
command -v rg >/dev/null 2>&1 || fail "rg not found"

[[ -f "${TOPIC_CATALOG}" ]] || fail "missing ${TOPIC_CATALOG}"
[[ -f "${TESTCASE_README}" ]] || fail "missing ${TESTCASE_README}"
[[ -f "${PORT_MATRIX_DOC}" ]] || fail "missing ${PORT_MATRIX_DOC}"
[[ -f "${LOCAL_STARTUP_DOC}" ]] || fail "missing ${LOCAL_STARTUP_DOC}"

check_consumer_group_catalog() {
  jq -e '
    [
      .topics[]
      | select((.consumers | length) > 0)
      | select((.consumer_groups | length) == 0 or (.consumer_groups | length) != (.consumers | length))
    ] | length == 0
  ' "${TOPIC_CATALOG}" >/dev/null \
    || fail "topics.v1.json contains consumer topics without aligned consumer_groups"

  ok "topic catalog consumer_groups aligned"
}

check_host_kafka_boundary_docs() {
  rg -q 'Kafka：`127\.0\.0\.1:9094`' "${TESTCASE_README}" \
    || fail "test-case README missing host kafka boundary 127.0.0.1:9094"
  rg -q '容器内 / compose 网络：`kafka:9092` 或容器内 `localhost:9092`' "${TESTCASE_README}" \
    || fail "test-case README missing compose/container kafka boundary"
  rg -q './scripts/check-topic-topology\.sh' "${TESTCASE_README}" \
    || fail "test-case README missing topology checker boundary note"
  rg -q './scripts/smoke-local\.sh' "${TESTCASE_README}" \
    || fail "test-case README missing full smoke boundary note"
  rg -q '127\.0\.0\.1:9094' "${PORT_MATRIX_DOC}" \
    || fail "port matrix missing host kafka boundary"
  rg -q '127\.0\.0\.1:9094' "${LOCAL_STARTUP_DOC}" \
    || fail "local startup runbook missing host kafka boundary"

  ok "host/container kafka boundary docs aligned"
}

check_runtime_docs_no_legacy_defaults() {
  local hit
  hit="$(
    rg -n \
      -g '*.md' \
      -g '*.yaml' \
      -g '*.json' \
      '(^|[^[:alnum:].])(outbox\.events|search\.sync|billing\.events|recommendation\.behavior|dead-letter\.events|notification-service)([^[:alnum:]]|$)' \
      docs/04-runbooks \
      docs/05-test-cases \
      docs/02-openapi \
      packages/openapi \
      infra/kafka \
      --glob '!docs/04-runbooks/kafka-topics.md' \
      || true
  )"

  [[ -z "${hit}" ]] || fail "legacy runtime topic/name still present in formal runtime artifacts:\n${hit}"

  ok "formal runtime artifacts are free of legacy default topics/names"
}

run_static_checks() {
  log "running canonical static checks"
  ./scripts/check-openapi-schema.sh
  check_consumer_group_catalog
  check_host_kafka_boundary_docs
  check_runtime_docs_no_legacy_defaults
  ok "canonical static checks passed"
}

run_full_checks() {
  log "running canonical runtime checks"
  ./scripts/check-topic-topology.sh
  ENV_FILE="${ENV_FILE}" ./scripts/smoke-local.sh
  ok "canonical runtime checks passed"
}

case "${MODE}" in
  static)
    run_static_checks
    ;;
  full)
    run_static_checks
    run_full_checks
    ;;
  *)
    fail "unsupported CANONICAL_CHECK_MODE='${MODE}', expected 'static' or 'full'"
    ;;
esac

ok "canonical contract checker passed (${MODE})"
