#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

MODE="${CANONICAL_CHECK_MODE:-full}"
ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
TOPIC_CATALOG="infra/kafka/topics.v1.json"
EVENT_MODEL_DOC="docs/开发准备/事件模型与Topic清单正式版.md"
KAFKA_RUNBOOK="docs/04-runbooks/kafka-topics.md"
TESTCASE_README="docs/05-test-cases/README.md"
CANONICAL_CASES_DOC="docs/05-test-cases/canonical-contracts-cases.md"
CHECKLIST_DOC="docs/05-test-cases/v1-core-acceptance-checklist.md"
SCRIPTS_README="scripts/README.md"
WORKFLOWS_README=".github/workflows/README.md"
CANONICAL_WORKFLOW=".github/workflows/canonical-contracts.yml"
PORT_MATRIX_DOC="docs/04-runbooks/port-matrix.md"
LOCAL_STARTUP_DOC="docs/04-runbooks/local-startup.md"
ARTIFACT_DIR="${CANONICAL_ARTIFACT_DIR:-target/test-artifacts/canonical-contracts}"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.json"
STATUS="running"
LAST_CHECK="bootstrap"
STATIC_CHECKS_RUN=false
FULL_CHECKS_RUN=false

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

write_summary() {
  local generated_at commit_sha
  mkdir -p "${ARTIFACT_DIR}"
  generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  commit_sha="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"

  jq -n \
    --arg mode "${MODE}" \
    --arg status "${STATUS}" \
    --arg last_check "${LAST_CHECK}" \
    --arg env_file "${ENV_FILE}" \
    --arg topic_catalog "${TOPIC_CATALOG}" \
    --arg checklist_doc "${CHECKLIST_DOC}" \
    --arg canonical_cases_doc "${CANONICAL_CASES_DOC}" \
    --arg generated_at "${generated_at}" \
    --arg commit_sha "${commit_sha}" \
    --argjson static_checks_run "${STATIC_CHECKS_RUN}" \
    --argjson full_checks_run "${FULL_CHECKS_RUN}" \
    '{
      task_id: "TEST-028",
      checker: "check-canonical-contracts.sh",
      mode: $mode,
      status: $status,
      last_check: $last_check,
      generated_at: $generated_at,
      commit: $commit_sha,
      inputs: {
        env_file: $env_file,
        topic_catalog: $topic_catalog,
        acceptance_checklist: $checklist_doc,
        canonical_cases_doc: $canonical_cases_doc
      },
      official_commands: {
        local_full: "ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh",
        ci_static: "CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh"
      },
      boundaries: {
        host_kafka: "127.0.0.1:9094",
        compose_kafka: "kafka:9092",
        container_local_kafka: "localhost:9092",
        topology_checker_scope: "./scripts/check-topic-topology.sh only covers notification/fabric/audit-anchor static topology and route seeds",
        full_topic_existence_checker: "ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh"
      },
      executed: {
        static_checks_run: $static_checks_run,
        full_checks_run: $full_checks_run
      }
    }' >"${SUMMARY_FILE}"
}

on_exit() {
  local rc=$?
  if [[ ${rc} -eq 0 ]]; then
    STATUS="passed"
  else
    STATUS="failed"
  fi
  write_summary
}

trap on_exit EXIT

command -v jq >/dev/null 2>&1 || fail "jq not found"
command -v rg >/dev/null 2>&1 || fail "rg not found"

[[ -f "${TOPIC_CATALOG}" ]] || fail "missing ${TOPIC_CATALOG}"
[[ -f "${EVENT_MODEL_DOC}" ]] || fail "missing ${EVENT_MODEL_DOC}"
[[ -f "${KAFKA_RUNBOOK}" ]] || fail "missing ${KAFKA_RUNBOOK}"
[[ -f "${TESTCASE_README}" ]] || fail "missing ${TESTCASE_README}"
[[ -f "${CANONICAL_CASES_DOC}" ]] || fail "missing ${CANONICAL_CASES_DOC}"
[[ -f "${CHECKLIST_DOC}" ]] || fail "missing ${CHECKLIST_DOC}"
[[ -f "${SCRIPTS_README}" ]] || fail "missing ${SCRIPTS_README}"
[[ -f "${WORKFLOWS_README}" ]] || fail "missing ${WORKFLOWS_README}"
[[ -f "${CANONICAL_WORKFLOW}" ]] || fail "missing ${CANONICAL_WORKFLOW}"
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

check_topic_catalog_docs() {
  while IFS= read -r topic_entry; do
    local name producer consumers consumer_groups env_key
    name="$(jq -r '.name' <<<"${topic_entry}")"
    producer="$(jq -r '.producer' <<<"${topic_entry}")"
    consumers="$(jq -r 'if (.consumers | length) == 0 then "-" else (.consumers | join(",")) end' <<<"${topic_entry}")"
    consumer_groups="$(jq -r 'if (.consumer_groups | length) == 0 then "-" else (.consumer_groups | join(",")) end' <<<"${topic_entry}")"
    env_key="$(jq -r '.env_key' <<<"${topic_entry}")"

    rg -Fq "| \`${name}\` | \`${producer}\` | \`${consumers}\` | \`${consumer_groups}\` |" "${KAFKA_RUNBOOK}" \
      || fail "kafka runbook missing canonical topology row for ${name}"
    rg -Fq "| \`${name}\` | \`${producer}\` | \`${consumers}\` | \`${consumer_groups}\` |" "${EVENT_MODEL_DOC}" \
      || fail "event model doc missing canonical topology row for ${name}"
    rg -Fq "| \`${env_key}\` | \`${name}\` |" "${PORT_MATRIX_DOC}" \
      || fail "port matrix missing env binding ${env_key} -> ${name}"
  done < <(jq -c '.topics[]' "${TOPIC_CATALOG}")

  ok "canonical topic catalog, runbook, and env bindings aligned"
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
  rg -q '127\.0\.0\.1:9094' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing host kafka boundary"
  rg -q 'kafka:9092' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing compose kafka boundary"
  rg -q 'localhost:9092' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing container-local kafka boundary"

  ok "host/container kafka boundary docs aligned"
}

check_checker_authority_docs() {
  rg -q '\| `ACC-CANONICAL` \| `TEST-028` \| `ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts\.sh` \|' "${CHECKLIST_DOC}" \
    || fail "acceptance checklist missing ACC-CANONICAL official command"
  rg -q 'TEST028-CASE-001' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing TEST028 case matrix"
  rg -q './scripts/check-topic-topology\.sh' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing topology-checker boundary note"
  rg -q './scripts/smoke-local\.sh' "${CANONICAL_CASES_DOC}" \
    || fail "canonical cases doc missing full smoke boundary note"
  rg -q 'check-canonical-contracts\.sh' "${SCRIPTS_README}" \
    || fail "scripts README missing TEST-028 checker entry"
  rg -q 'canonical-contracts\.yml' "${WORKFLOWS_README}" \
    || fail "workflow README missing TEST-028 workflow entry"
  rg -q 'CANONICAL_CHECK_MODE=static bash ./scripts/check-canonical-contracts\.sh' "${CANONICAL_WORKFLOW}" \
    || fail "canonical workflow missing static checker command"
  rg -q 'actions/upload-artifact@v4' "${CANONICAL_WORKFLOW}" \
    || fail "canonical workflow missing artifact upload step"
  rg -q 'target/test-artifacts/canonical-contracts' "${CANONICAL_WORKFLOW}" \
    || fail "canonical workflow missing TEST-028 artifact path"

  ok "acceptance matrix, docs, and workflow authority aligned"
}

check_host_worker_kafka_defaults() {
  local file
  for file in \
    workers/outbox-publisher/src/main.rs \
    workers/search-indexer/src/main.rs \
    workers/recommendation-aggregator/src/main.rs; do
    rg -q '"127\.0\.0\.1:9094"' "${file}" \
      || fail "host-run worker default kafka boundary drifted in ${file}"
  done

  ok "host-run worker kafka defaults aligned to 127.0.0.1:9094"
}

check_no_secondary_topic_manifest() {
  [[ ! -e fixtures/local/kafka-topics-manifest.json ]] \
    || fail "stale fixtures/local/kafka-topics-manifest.json must not remain as a second topic authority"

  ok "no stale second topic manifest remains"
}

check_host_kafka_misuse() {
  local hit
  hit="$(
    rg -n \
      '127\.0\.0\.1:9092|localhost:9094' \
      scripts \
      docs/04-runbooks \
      docs/05-test-cases/README.md \
      docs/05-test-cases/v1-core-acceptance-checklist.md \
      .github/workflows \
      workers/outbox-publisher/src/main.rs \
      workers/search-indexer/src/main.rs \
      workers/recommendation-aggregator/src/main.rs \
      --glob '!scripts/check-canonical-contracts.sh' \
      || true
  )"

  [[ -z "${hit}" ]] || fail "formal checker/runtime artifacts still contain invalid host kafka boundary:\n${hit}"

  ok "formal checker/runtime artifacts reject host kafka misuse"
}

check_runtime_docs_no_legacy_defaults() {
  local hit
  hit="$(
    rg -n \
      -g '*.md' \
      -g '*.yaml' \
      -g '*.yml' \
      -g '*.json' \
      -g '*.sh' \
      '(^|[^[:alnum:].])(outbox\.events|search\.sync|billing\.events|recommendation\.behavior|dead-letter\.events|notification-service)([^[:alnum:]]|$)' \
      docs/04-runbooks \
      docs/05-test-cases \
      docs/02-openapi \
      packages/openapi \
      infra/kafka \
      scripts \
      .github/workflows \
      fixtures/local \
      --glob '!docs/04-runbooks/kafka-topics.md' \
      --glob '!docs/05-test-cases/canonical-contracts-cases.md' \
      --glob '!scripts/check-canonical-contracts.sh' \
      || true
  )"

  [[ -z "${hit}" ]] || fail "legacy runtime topic/name still present in formal runtime artifacts:\n${hit}"

  ok "formal runtime artifacts are free of legacy default topics/names"
}

run_static_checks() {
  log "running canonical static checks"
  LAST_CHECK="openapi-schema"
  ./scripts/check-openapi-schema.sh
  LAST_CHECK="consumer-group-catalog"
  check_consumer_group_catalog
  LAST_CHECK="topic-catalog-docs"
  check_topic_catalog_docs
  LAST_CHECK="checker-authority-docs"
  check_checker_authority_docs
  LAST_CHECK="host-kafka-boundary-docs"
  check_host_kafka_boundary_docs
  LAST_CHECK="host-worker-kafka-defaults"
  check_host_worker_kafka_defaults
  LAST_CHECK="host-kafka-misuse-guard"
  check_host_kafka_misuse
  LAST_CHECK="legacy-runtime-defaults"
  check_runtime_docs_no_legacy_defaults
  LAST_CHECK="secondary-topic-authority"
  check_no_secondary_topic_manifest
  STATIC_CHECKS_RUN=true
  ok "canonical static checks passed"
}

run_full_checks() {
  log "running canonical runtime checks"
  LAST_CHECK="check-topic-topology"
  ./scripts/check-topic-topology.sh
  LAST_CHECK="smoke-local"
  ENV_FILE="${ENV_FILE}" ./scripts/smoke-local.sh
  FULL_CHECKS_RUN=true
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

LAST_CHECK="completed"
ok "canonical contract checker passed (${MODE})"
