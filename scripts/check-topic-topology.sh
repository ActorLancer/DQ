#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT_DIR}"

TOPIC_CATALOG="infra/kafka/topics.v1.json"
EVENT_MODEL_DOC="docs/开发准备/事件模型与Topic清单正式版.md"
KAFKA_RUNBOOK="docs/04-runbooks/kafka-topics.md"
ROUTE_SEED="docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql"
TOPOLOGY_DATABASE_URL="${TOPOLOGY_DATABASE_URL:-${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}}"

# Scope:
# - static alignment for dedicated notification / fabric / audit-anchor single-entry topology
# - critical route-policy seed coverage for frozen extension events
# - runtime route-policy presence in ops.event_route_policy for the frozen dedicated topics
# This script does not replace the full smoke-local topic existence check.

fail() {
  echo "[fail] $*" >&2
  exit 1
}

ok() {
  echo "[ok]   $*"
}

command -v jq >/dev/null 2>&1 || fail "jq not found"
command -v psql >/dev/null 2>&1 || fail "psql not found"
[[ -f "${TOPIC_CATALOG}" ]] || fail "missing ${TOPIC_CATALOG}"
[[ -f "${EVENT_MODEL_DOC}" ]] || fail "missing ${EVENT_MODEL_DOC}"
[[ -f "${KAFKA_RUNBOOK}" ]] || fail "missing ${KAFKA_RUNBOOK}"
[[ -f "${ROUTE_SEED}" ]] || fail "missing ${ROUTE_SEED}"

check_topic_field() {
  local topic="$1"
  local jq_expr="$2"
  local expected="$3"
  local actual
  jq -e --arg topic "${topic}" '.topics[] | select(.name == $topic)' "${TOPIC_CATALOG}" >/dev/null \
    || fail "topic not found in catalog: ${topic}"
  actual="$(jq -r --arg topic "${topic}" ".topics[] | select(.name == \$topic) | ${jq_expr}" "${TOPIC_CATALOG}")"
  [[ "${actual}" == "${expected}" ]] || fail "catalog mismatch for ${topic}: expected '${expected}', got '${actual}'"
}

check_topic_json() {
  check_topic_field "dtp.outbox.domain-events" ".producer" "outbox-publisher"
  check_topic_field "dtp.outbox.domain-events" ".consumers | join(\",\")" ""
  check_topic_field "dtp.outbox.domain-events" ".consumer_groups | join(\",\")" ""
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
  rg -q '\| `dtp\.outbox\.domain-events` \| `outbox-publisher` \| `-` \| `-` \|' "${EVENT_MODEL_DOC}" \
    || fail "event model doc is missing canonical outbox topology row"
  rg -q '\| `dtp\.outbox\.domain-events` \| `outbox-publisher` \| `-` \| `-` \|' "${KAFKA_RUNBOOK}" \
    || fail "kafka runbook is missing canonical outbox topology row"
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

check_db_runtime_routes() {
  local sql
  local actual
  sql=$(cat <<'SQL'
SELECT aggregate_type || '|' || event_type || '|' || target_topic
FROM ops.event_route_policy
WHERE status = 'active'
  AND (aggregate_type, event_type, target_topic) IN (
    ('notification.dispatch_request', 'notification.requested', 'dtp.notification.dispatch'),
    ('audit.anchor_batch', 'audit.anchor_requested', 'dtp.audit.anchor'),
    ('chain.chain_anchor', 'fabric.proof_submit_requested', 'dtp.fabric.requests')
  )
ORDER BY 1;
SQL
)
  actual="$(psql "${TOPOLOGY_DATABASE_URL}" -v ON_ERROR_STOP=1 -X -q -tA -c "${sql}")" \
    || fail "failed to query runtime route-policy via ${TOPOLOGY_DATABASE_URL}"

  for expected in \
    "audit.anchor_batch|audit.anchor_requested|dtp.audit.anchor" \
    "chain.chain_anchor|fabric.proof_submit_requested|dtp.fabric.requests" \
    "notification.dispatch_request|notification.requested|dtp.notification.dispatch"; do
    grep -Fxq "${expected}" <<<"${actual}" \
      || fail "runtime route-policy missing: ${expected}"
  done
}

check_topic_json
check_docs
check_route_seed
check_db_runtime_routes
ok "dedicated notification/fabric topic topology, docs, route-policy seeds, and runtime DB routes are aligned"
