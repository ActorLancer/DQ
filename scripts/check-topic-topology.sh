#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT_DIR}"

TOPIC_CATALOG="infra/kafka/topics.v1.json"
EVENT_MODEL_DOC="docs/开发准备/事件模型与Topic清单正式版.md"
KAFKA_RUNBOOK="docs/04-runbooks/kafka-topics.md"
ROUTE_SEED="docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql"

fail() {
  echo "[fail] $*" >&2
  exit 1
}

ok() {
  echo "[ok]   $*"
}

command -v jq >/dev/null 2>&1 || fail "jq not found"
[[ -f "${TOPIC_CATALOG}" ]] || fail "missing ${TOPIC_CATALOG}"
[[ -f "${EVENT_MODEL_DOC}" ]] || fail "missing ${EVENT_MODEL_DOC}"
[[ -f "${KAFKA_RUNBOOK}" ]] || fail "missing ${KAFKA_RUNBOOK}"
[[ -f "${ROUTE_SEED}" ]] || fail "missing ${ROUTE_SEED}"

check_topic_field() {
  local topic="$1"
  local jq_expr="$2"
  local expected="$3"
  local actual
  actual="$(jq -r --arg topic "${topic}" ".topics[] | select(.name == \$topic) | ${jq_expr}" "${TOPIC_CATALOG}")"
  [[ -n "${actual}" ]] || fail "topic not found in catalog: ${topic}"
  [[ "${actual}" == "${expected}" ]] || fail "catalog mismatch for ${topic}: expected '${expected}', got '${actual}'"
}

check_topic_json() {
  check_topic_field "dtp.outbox.domain-events" ".producer" "outbox-publisher"
  check_topic_field "dtp.outbox.domain-events" ".consumers | join(\",\")" "notification-worker,fabric-adapter"
  check_topic_field "dtp.outbox.domain-events" ".consumer_groups | join(\",\")" "cg-notification-worker,cg-fabric-adapter"
  check_topic_field "dtp.notification.dispatch" ".consumers | join(\",\")" "notification-worker"
  check_topic_field "dtp.notification.dispatch" ".consumer_groups | join(\",\")" "cg-notification-worker"
  check_topic_field "dtp.fabric.requests" ".consumers | join(\",\")" "fabric-adapter"
  check_topic_field "dtp.fabric.requests" ".consumer_groups | join(\",\")" "cg-fabric-adapter"
  check_topic_field "dtp.fabric.callbacks" ".producer" "fabric-event-listener"
  check_topic_field "dtp.fabric.callbacks" ".consumers | join(\",\")" "platform-core.consistency"
  check_topic_field "dtp.fabric.callbacks" ".consumer_groups | join(\",\")" "cg-platform-core-consistency"
  check_topic_field "dtp.audit.anchor" ".producer" "platform-core.audit"
  check_topic_field "dtp.audit.anchor" ".consumers | join(\",\")" "fabric-adapter"
  check_topic_field "dtp.audit.anchor" ".consumer_groups | join(\",\")" "cg-fabric-adapter"
}

check_docs() {
  rg -q '\| `dtp\.outbox\.domain-events` \| `outbox-publisher` \| `notification-worker` / `fabric-adapter` \| `cg-notification-worker` / `cg-fabric-adapter` \|' "${EVENT_MODEL_DOC}" \
    || fail "event model doc is missing canonical outbox topology row"
  rg -q '\| `dtp\.fabric\.callbacks` \| `fabric-event-listener` \| `platform-core\.consistency` \| `cg-platform-core-consistency` \|' "${EVENT_MODEL_DOC}" \
    || fail "event model doc is missing canonical fabric callback topology row"
  rg -q '\| `dtp\.notification\.dispatch` \| `platform-core\.integration` \| `notification-worker` \| `cg-notification-worker` \|' "${KAFKA_RUNBOOK}" \
    || fail "kafka runbook is missing canonical notification topology row"
  rg -q '\| `dtp\.audit\.anchor` \| `platform-core\.audit` \| `fabric-adapter` \| `cg-fabric-adapter` \|' "${KAFKA_RUNBOOK}" \
    || fail "kafka runbook is missing canonical audit anchor topology row"
}

check_route_seed() {
  rg -q "'notification\\.dispatch_request'.*'notification\\.requested'" "${ROUTE_SEED}" \
    || fail "route seed missing notification.requested"
  rg -q "'audit\\.anchor_batch'.*'audit\\.anchor_requested'" "${ROUTE_SEED}" \
    || fail "route seed missing audit.anchor_requested"
  rg -q "'chain\\.chain_anchor'.*'fabric\\.proof_submit_requested'" "${ROUTE_SEED}" \
    || fail "route seed missing fabric.proof_submit_requested"
}

check_topic_json
check_docs
check_route_seed
ok "topic topology catalog, docs, and route-policy seeds are aligned"
