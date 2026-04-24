#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/test-artifacts/failure-drills}"
APP_LOG_FILE="${APP_LOG_FILE:-${ARTIFACT_DIR}/test-019-platform-core.log}"
FABRIC_ADAPTER_LOG_FILE="${FABRIC_ADAPTER_LOG_FILE:-${ARTIFACT_DIR}/test-019-fabric-adapter.log}"
KEYCLOAK_BASE_URL="${KEYCLOAK_BASE_URL:-http://127.0.0.1:8081}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-platform-local}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-portal-web}"
MOCK_PAYMENT_BASE_URL="${MOCK_PAYMENT_BASE_URL:-http://127.0.0.1:${MOCK_PAYMENT_PORT:-8089}}"
SEARCH_QUERY="${SEARCH_QUERY:-工业设备运行指标 API 订阅}"
MIN_TIMEOUT_SECONDS="${MIN_TIMEOUT_SECONDS:-14.0}"

APP_PID=""
APP_STARTED_BY_CHECKER="false"
FABRIC_ADAPTER_PID=""
FABRIC_ADAPTER_STARTED_BY_CHECKER="false"
KAFKA_STOPPED="false"
OPENSEARCH_STOPPED="false"

RUN_ID=""
BASE_URL=""
BUYER_TOKEN=""
BUYER_USER_ID=""
BUYER_ORG_ID=""
BUYER_ROLE=""
PRODUCT_ID=""
SKU_ID=""
ORDER_ID=""
ORDER_REQUEST_ID=""
ORDER_OUTBOX_COUNT="0"

SEARCH_REQUEST_ID_1=""
SEARCH_REQUEST_ID_2=""
SEARCH_BACKEND_1=""
SEARCH_BACKEND_2=""
SEARCH_CACHE_HIT_1=""
SEARCH_CACHE_HIT_2=""

FABRIC_GROUP=""
FABRIC_WARMUP_REQUEST_ID=""
FABRIC_WARMUP_EVENT_ID=""
FABRIC_WARMUP_TRACE_ID=""
FABRIC_WARMUP_ANCHOR_BATCH_ID=""
FABRIC_WARMUP_CHAIN_ANCHOR_ID=""
FABRIC_WARMUP_BATCH_ROOT=""
FABRIC_DOWN_REQUEST_ID=""
FABRIC_DOWN_EVENT_ID=""
FABRIC_DOWN_TRACE_ID=""
FABRIC_DOWN_ANCHOR_BATCH_ID=""
FABRIC_DOWN_CHAIN_ANCHOR_ID=""
FABRIC_DOWN_BATCH_ROOT=""
FABRIC_LAG_WHILE_STOPPED="0"
FABRIC_RECEIPT_AFTER_RECOVERY="0"

MOCK_TIMEOUT_SECONDS="0"
MOCK_TIMEOUT_STATUS=""

log() {
  echo "[info] $*"
}

ok() {
  echo "[ok]   $*"
}

fail() {
  echo "[fail] $*" >&2
  if [[ -f "${APP_LOG_FILE}" ]]; then
    echo "[fail] platform-core log: ${APP_LOG_FILE}" >&2
    tail -n 80 "${APP_LOG_FILE}" >&2 || true
  fi
  if [[ -f "${FABRIC_ADAPTER_LOG_FILE}" ]]; then
    echo "[fail] fabric-adapter log: ${FABRIC_ADAPTER_LOG_FILE}" >&2
    tail -n 80 "${FABRIC_ADAPTER_LOG_FILE}" >&2 || true
  fi
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

compose() {
  docker compose --env-file "${ENV_FILE}" -f infra/docker/docker-compose.local.yml "$@"
}

redis_cli() {
  compose exec -T -e "REDISCLI_AUTH=${REDIS_PASSWORD}" redis redis-cli "$@"
}

wait_http_code() {
  local name="$1"
  local url="$2"
  local expected_regex="$3"
  local timeout_seconds="${4:-180}"
  local sleep_seconds="${5:-2}"
  local deadline=$((SECONDS + timeout_seconds))
  local code=""

  while (( SECONDS < deadline )); do
    code="$(curl -sS -o /dev/null -w '%{http_code}' "${url}" || true)"
    if [[ "${code}" =~ ${expected_regex} ]]; then
      ok "${name} responded with HTTP ${code}"
      return 0
    fi
    sleep "${sleep_seconds}"
  done

  fail "${name} did not reach expected HTTP state at ${url} (last=${code})"
}

wait_for_condition() {
  local name="$1"
  local timeout_seconds="$2"
  local sleep_seconds="$3"
  shift 3
  local deadline=$((SECONDS + timeout_seconds))

  while (( SECONDS < deadline )); do
    if "$@" >/dev/null 2>&1; then
      ok "${name}"
      return 0
    fi
    sleep "${sleep_seconds}"
  done

  fail "${name} did not complete within ${timeout_seconds}s"
}

stop_platform_core() {
  if [[ "${APP_STARTED_BY_CHECKER}" == "true" ]] && [[ -n "${APP_PID}" ]] && kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}

stop_fabric_adapter() {
  if [[ "${FABRIC_ADAPTER_STARTED_BY_CHECKER}" == "true" ]] && [[ -n "${FABRIC_ADAPTER_PID}" ]]; then
    kill -TERM -- "-${FABRIC_ADAPTER_PID}" >/dev/null 2>&1 || true
    sleep 2
    if kill -0 "${FABRIC_ADAPTER_PID}" >/dev/null 2>&1; then
      kill -KILL -- "-${FABRIC_ADAPTER_PID}" >/dev/null 2>&1 || true
    fi
    wait "${FABRIC_ADAPTER_PID}" >/dev/null 2>&1 || true
  fi
  FABRIC_ADAPTER_PID=""
  FABRIC_ADAPTER_STARTED_BY_CHECKER="false"
}

query_db_value() {
  local sql="$1"
  psql "${DATABASE_URL}" -Atqc "${sql}"
}

new_uuid() {
  node -e 'console.log(require("crypto").randomUUID())'
}

fetch_access_token() {
  local username="$1"
  local password="$2"
  local token

  token="$(
    curl -sS -X POST \
      "${KEYCLOAK_BASE_URL}/realms/${KEYCLOAK_REALM}/protocol/openid-connect/token" \
      -H 'content-type: application/x-www-form-urlencoded' \
      --data-urlencode 'grant_type=password' \
      --data-urlencode "client_id=${KEYCLOAK_CLIENT_ID}" \
      --data-urlencode "username=${username}" \
      --data-urlencode "password=${password}" \
      | jq -r '.access_token // empty'
  )"
  [[ -n "${token}" ]] || fail "keycloak password grant failed for ${username}"
  printf '%s' "${token}"
}

token_claims_tsv() {
  local token="$1"
  node -e '
    const token = process.argv[1];
    const payload = JSON.parse(Buffer.from(token.split(".")[1], "base64url").toString("utf8"));
    const roles = Array.isArray(payload.realm_access?.roles) ? payload.realm_access.roles : [];
    console.log([payload.user_id ?? "", payload.org_id ?? "", roles[0] ?? ""].join("\t"));
  ' "${token}"
}

threshold_ok() {
  local actual="$1"
  local threshold="$2"
  awk -v actual="${actual}" -v threshold="${threshold}" 'BEGIN { exit(actual >= threshold ? 0 : 1) }'
}

assert_ok_envelope() {
  local file="$1"
  local label="$2"
  jq -e '
    .code == "OK" and
    .message == "success" and
    (.request_id | type == "string" and length > 0) and
    (.data | type == "object")
  ' "${file}" >/dev/null || fail "${label} response is not the formal success envelope"
}

assert_json_has_scalar() {
  local file="$1"
  local expected="$2"
  local label="$3"
  jq -e --arg expected "${expected}" 'any(.. | scalars; . == $expected)' "${file}" >/dev/null \
    || fail "${label} response did not contain expected scalar ${expected}"
}

start_or_reuse_platform_core() {
  local live_code ready_code

  live_code="$(curl -sS -o /dev/null -w '%{http_code}' "${BASE_URL}${HEALTH_LIVE_PATH}" || true)"
  ready_code="$(curl -sS -o /dev/null -w '%{http_code}' "${BASE_URL}${HEALTH_READY_PATH}" || true)"
  if [[ "${live_code}" == "200" && "${ready_code}" == "200" ]]; then
    ok "reusing existing platform-core on ${BASE_URL}"
    return 0
  fi

  mkdir -p "${ARTIFACT_DIR}"
  rm -f "${APP_LOG_FILE}"

  log "starting host platform-core TEST-019 instance on ${BASE_URL}"
  cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_CHECKER="true"

  wait_http_code "platform-core live" "${BASE_URL}${HEALTH_LIVE_PATH}" '^200$' 240 2
  wait_http_code "platform-core ready" "${BASE_URL}${HEALTH_READY_PATH}" '^200$' 240 2
}

check_dependency_state() {
  local dependency_name="$1"
  local expected="$2"
  local output_file="$3"
  curl -fsS "${BASE_URL}${HEALTH_DEPS_PATH}" >"${output_file}"
  jq -e --arg dependency_name "${dependency_name}" --arg expected "${expected}" '
    .code == "OK" and
    any(.data.checks[]?; .name == $dependency_name and .reachable == ($expected == "true"))
  ' "${output_file}" >/dev/null
}

wait_dependency_state() {
  local dependency_name="$1"
  local expected="$2"
  local output_file="$3"
  local timeout_seconds="${4:-120}"
  local sleep_seconds="${5:-2}"
  local deadline=$((SECONDS + timeout_seconds))

  while (( SECONDS < deadline )); do
    if check_dependency_state "${dependency_name}" "${expected}" "${output_file}"; then
      ok "dependency ${dependency_name} reachable=${expected}"
      return 0
    fi
    sleep "${sleep_seconds}"
  done

  fail "dependency ${dependency_name} did not reach reachable=${expected}"
}

kafka_ready() {
  compose exec -T kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list >/dev/null
}

wait_kafka_ready() {
  wait_for_condition "kafka bootstrap recovered" 120 2 kafka_ready
}

opensearch_ready() {
  curl -fsS "http://127.0.0.1:${OPENSEARCH_HTTP_PORT}/_cluster/health" >/dev/null
}

opensearch_down() {
  ! curl -fsS "http://127.0.0.1:${OPENSEARCH_HTTP_PORT}/_cluster/health" >/dev/null
}

wait_opensearch_ready() {
  wait_for_condition "opensearch HTTP endpoint recovered" 180 2 opensearch_ready
}

wait_opensearch_down() {
  wait_for_condition "opensearch became unavailable" 60 2 opensearch_down
}

clear_search_cache() {
  local pattern="${REDIS_NAMESPACE}:search:catalog:*"
  local keys
  keys="$(redis_cli --scan --pattern "${pattern}" | tr -d '\r')"
  if [[ -z "${keys}" ]]; then
    ok "no search cache keys to clear"
    return 0
  fi

  while IFS= read -r key; do
    [[ -n "${key}" ]] || continue
    redis_cli DEL "${key}" >/dev/null
  done <<<"${keys}"
  ok "cleared search cache keys matching ${pattern}"
}

measure_json_request() {
  local name="$1"
  local method="$2"
  local url="$3"
  local response_file="$4"
  local body_file="$5"
  shift 5

  local -a curl_args=(curl -sS -X "${method}" "${url}" -o "${response_file}" -w '%{http_code}')
  if [[ -n "${body_file}" ]]; then
    curl_args+=(-H 'content-type: application/json' --data @"${body_file}")
  fi
  while (( "$#" )); do
    curl_args+=(-H "$1")
    shift
  done

  local code
  code="$("${curl_args[@]}")" || fail "${name} request failed to execute"
  [[ "${code}" == "200" ]] || {
    echo "[fail] ${name} response body:" >&2
    cat "${response_file}" >&2
    fail "${name} returned HTTP ${code}"
  }
}

start_fabric_adapter() {
  rm -f "${FABRIC_ADAPTER_LOG_FILE}"
  log "starting fabric-adapter with consumer group ${FABRIC_GROUP}"
  setsid bash -lc "cd '${ROOT_DIR}' && FABRIC_ADAPTER_CONSUMER_GROUP='${FABRIC_GROUP}' ./scripts/fabric-adapter-run.sh" \
    >"${FABRIC_ADAPTER_LOG_FILE}" 2>&1 &
  FABRIC_ADAPTER_PID="$!"
  FABRIC_ADAPTER_STARTED_BY_CHECKER="true"
  sleep 3
  kill -0 "${FABRIC_ADAPTER_PID}" >/dev/null 2>&1 || fail "fabric-adapter exited before processing began"
}

describe_fabric_group() {
  local output_file="$1"
  compose exec -T kafka /opt/kafka/bin/kafka-consumer-groups.sh \
    --bootstrap-server localhost:9092 \
    --group "${FABRIC_GROUP}" \
    --describe >"${output_file}"
}

fabric_group_lag_sum() {
  local output_file="$1"
  describe_fabric_group "${output_file}"
  awk 'NR > 1 && $6 ~ /^[0-9]+$/ {sum += $6} END {print sum + 0}' "${output_file}"
}

reset_consumer_group_to_latest() {
  local group="$1"
  local topic="$2"
  local output_file="$3"

  compose exec -T kafka /opt/kafka/bin/kafka-consumer-groups.sh \
    --bootstrap-server localhost:9092 \
    --group "${group}" \
    --topic "${topic}" \
    --reset-offsets \
    --to-latest \
    --execute >"${output_file}"
}

wait_fabric_receipt_count() {
  local request_id="$1"
  local expected_count="$2"
  local timeout_seconds="${3:-90}"
  local deadline=$((SECONDS + timeout_seconds))
  local count

  while (( SECONDS < deadline )); do
    count="$(query_db_value "SELECT COUNT(*)::text FROM ops.external_fact_receipt WHERE request_id = '${request_id}'")"
    if [[ "${count}" == "${expected_count}" ]]; then
      ok "ops.external_fact_receipt request_id=${request_id} count=${expected_count}"
      return 0
    fi
    sleep 2
  done

  fail "ops.external_fact_receipt request_id=${request_id} did not reach count=${expected_count}"
}

seed_fabric_anchor() {
  local anchor_batch_id="$1"
  local chain_anchor_id="$2"
  local batch_root="$3"
  local event_id="$4"
  local tag="$5"

  psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 >/dev/null <<SQL
INSERT INTO chain.chain_anchor (
  chain_anchor_id,
  chain_id,
  anchor_type,
  ref_type,
  ref_id,
  digest,
  status,
  authority_model,
  reconcile_status,
  created_at
) VALUES (
  '${chain_anchor_id}'::uuid,
  'fabric-local',
  'audit_batch',
  'anchor_batch',
  '${anchor_batch_id}'::uuid,
  '${batch_root}',
  'pending',
  'dual_authority',
  'pending_check',
  now()
);

INSERT INTO audit.anchor_batch (
  anchor_batch_id,
  batch_scope,
  chain_id,
  record_count,
  batch_root,
  window_started_at,
  window_ended_at,
  status,
  chain_anchor_id,
  metadata
) VALUES (
  '${anchor_batch_id}'::uuid,
  'audit_event',
  'fabric-local',
  1,
  '${batch_root}',
  now() - interval '3 minutes',
  now() - interval '1 minute',
  'pending',
  '${chain_anchor_id}'::uuid,
  jsonb_build_object('seed', '${tag}', 'event_id', '${event_id}')
);
SQL
}

write_fabric_event() {
  local output_file="$1"
  local event_id="$2"
  local request_id="$3"
  local trace_id="$4"
  local anchor_batch_id="$5"
  local chain_anchor_id="$6"
  local batch_root="$7"

  jq -c -n \
    --arg event_id "${event_id}" \
    --arg request_id "${request_id}" \
    --arg trace_id "${trace_id}" \
    --arg anchor_batch_id "${anchor_batch_id}" \
    --arg chain_anchor_id "${chain_anchor_id}" \
    --arg batch_root "${batch_root}" \
    '{
      event_id: $event_id,
      event_type: "audit.anchor_requested",
      event_version: 1,
      occurred_at: "2026-04-24T00:00:00Z",
      producer_service: "platform-core.audit",
      aggregate_type: "audit.anchor_batch",
      aggregate_id: $anchor_batch_id,
      request_id: $request_id,
      trace_id: $trace_id,
      event_schema_version: "v1",
      authority_scope: "audit",
      source_of_truth: "postgresql",
      proof_commit_policy: "async_evidence",
      anchor_batch_id: $anchor_batch_id,
      chain_anchor_id: $chain_anchor_id,
      batch_root: $batch_root,
      payload: {
        batch_root: $batch_root,
        record_count: 1
      }
    }' >"${output_file}"
}

publish_kafka_event() {
  local topic="$1"
  local payload_file="$2"
  local kafka_container_id

  kafka_container_id="$(compose ps -q kafka)"
  [[ -n "${kafka_container_id}" ]] || fail "cannot resolve kafka container id"
  docker run --rm -i --network "container:${kafka_container_id}" edenhill/kcat:1.7.1 \
    -P -b localhost:9092 -t "${topic}" <"${payload_file}" >/dev/null
}

cleanup_order_data() {
  if [[ -z "${ORDER_ID}" ]]; then
    return 0
  fi

  log "cleaning TEST-019 order drill data for order ${ORDER_ID}"
  psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 >/dev/null <<SQL
DELETE FROM ops.outbox_event
WHERE request_id = '${ORDER_REQUEST_ID}';

DELETE FROM trade.order_line
WHERE order_id = '${ORDER_ID}'::uuid;

DELETE FROM trade.order_main
WHERE order_id = '${ORDER_ID}'::uuid;
SQL
  ok "TEST-019 order drill data cleaned"
}

join_by_comma() {
  local IFS=','
  printf '%s' "$*"
}

cleanup_fabric_data() {
  local event_ids=()
  local batch_ids=()
  local anchor_ids=()
  local request_ids=()

  [[ -n "${FABRIC_WARMUP_EVENT_ID}" ]] && event_ids+=("'${FABRIC_WARMUP_EVENT_ID}'::uuid")
  [[ -n "${FABRIC_DOWN_EVENT_ID}" ]] && event_ids+=("'${FABRIC_DOWN_EVENT_ID}'::uuid")
  [[ -n "${FABRIC_WARMUP_ANCHOR_BATCH_ID}" ]] && batch_ids+=("'${FABRIC_WARMUP_ANCHOR_BATCH_ID}'::uuid")
  [[ -n "${FABRIC_DOWN_ANCHOR_BATCH_ID}" ]] && batch_ids+=("'${FABRIC_DOWN_ANCHOR_BATCH_ID}'::uuid")
  [[ -n "${FABRIC_WARMUP_CHAIN_ANCHOR_ID}" ]] && anchor_ids+=("'${FABRIC_WARMUP_CHAIN_ANCHOR_ID}'::uuid")
  [[ -n "${FABRIC_DOWN_CHAIN_ANCHOR_ID}" ]] && anchor_ids+=("'${FABRIC_DOWN_CHAIN_ANCHOR_ID}'::uuid")
  [[ -n "${FABRIC_WARMUP_REQUEST_ID}" ]] && request_ids+=("'${FABRIC_WARMUP_REQUEST_ID}'")
  [[ -n "${FABRIC_DOWN_REQUEST_ID}" ]] && request_ids+=("'${FABRIC_DOWN_REQUEST_ID}'")

  if (( ${#event_ids[@]} == 0 )) && (( ${#batch_ids[@]} == 0 )) && (( ${#anchor_ids[@]} == 0 )) && (( ${#request_ids[@]} == 0 )); then
    return 0
  fi

  log "cleaning TEST-019 fabric drill data"
  if (( ${#request_ids[@]} > 0 )); then
    query_db_value "DELETE FROM ops.external_fact_receipt WHERE request_id IN ($(join_by_comma "${request_ids[@]}"))" >/dev/null
  fi
  if (( ${#event_ids[@]} > 0 )); then
    query_db_value "DELETE FROM ops.consumer_idempotency_record WHERE consumer_name = 'fabric-adapter' AND event_id IN ($(join_by_comma "${event_ids[@]}"))" >/dev/null
  fi
  if (( ${#batch_ids[@]} > 0 )); then
    query_db_value "DELETE FROM audit.anchor_batch WHERE anchor_batch_id IN ($(join_by_comma "${batch_ids[@]}"))" >/dev/null
  fi
  if (( ${#anchor_ids[@]} > 0 )); then
    query_db_value "DELETE FROM chain.chain_anchor WHERE chain_anchor_id IN ($(join_by_comma "${anchor_ids[@]}"))" >/dev/null
  fi
  ok "TEST-019 fabric drill data cleaned"
}

cleanup() {
  stop_fabric_adapter

  if [[ "${KAFKA_STOPPED}" == "true" ]]; then
    compose start kafka >/dev/null 2>&1 || true
    wait_kafka_ready || true
  fi

  if [[ "${OPENSEARCH_STOPPED}" == "true" ]]; then
    compose start opensearch >/dev/null 2>&1 || true
    wait_opensearch_ready || true
  fi

  cleanup_fabric_data
  cleanup_order_data
  stop_platform_core
}

write_summary_json() {
  jq -n \
    --arg run_id "${RUN_ID}" \
    --arg order_id "${ORDER_ID}" \
    --arg order_request_id "${ORDER_REQUEST_ID}" \
    --arg order_outbox_count "${ORDER_OUTBOX_COUNT}" \
    --arg search_request_id_1 "${SEARCH_REQUEST_ID_1}" \
    --arg search_request_id_2 "${SEARCH_REQUEST_ID_2}" \
    --arg search_backend_1 "${SEARCH_BACKEND_1}" \
    --arg search_backend_2 "${SEARCH_BACKEND_2}" \
    --arg search_cache_hit_1 "${SEARCH_CACHE_HIT_1}" \
    --arg search_cache_hit_2 "${SEARCH_CACHE_HIT_2}" \
    --arg fabric_group "${FABRIC_GROUP}" \
    --arg fabric_warmup_request_id "${FABRIC_WARMUP_REQUEST_ID}" \
    --arg fabric_down_request_id "${FABRIC_DOWN_REQUEST_ID}" \
    --arg fabric_lag_while_stopped "${FABRIC_LAG_WHILE_STOPPED}" \
    --arg fabric_receipt_after_recovery "${FABRIC_RECEIPT_AFTER_RECOVERY}" \
    --arg mock_timeout_status "${MOCK_TIMEOUT_STATUS}" \
    --arg mock_timeout_seconds "${MOCK_TIMEOUT_SECONDS}" \
    --arg min_timeout_seconds "${MIN_TIMEOUT_SECONDS}" \
    '{
      task_id: "TEST-019",
      run_id: $run_id,
      kafka_down: {
        degraded_boundary: "Kafka is asynchronous only; main order transaction must still commit into PostgreSQL + outbox.",
        request_id: $order_request_id,
        order_id: $order_id,
        outbox_count: ($order_outbox_count | tonumber)
      },
      opensearch_down: {
        degraded_boundary: "OpenSearch is not the source of truth; local/demo search must remain available via PostgreSQL fallback + Redis cache.",
        search_requests: [
          {
            request_id: $search_request_id_1,
            backend: $search_backend_1,
            cache_hit: ($search_cache_hit_1 == "true")
          },
          {
            request_id: $search_request_id_2,
            backend: $search_backend_2,
            cache_hit: ($search_cache_hit_2 == "true")
          }
        ]
      },
      fabric_adapter_down: {
        degraded_boundary: "Audit/Fabric anchor submission is asynchronous; adapter outage must accumulate lag without faking receipts, and recover after restart.",
        consumer_group: $fabric_group,
        warmup_request_id: $fabric_warmup_request_id,
        down_request_id: $fabric_down_request_id,
        lag_while_stopped: ($fabric_lag_while_stopped | tonumber),
        receipt_count_after_recovery: ($fabric_receipt_after_recovery | tonumber)
      },
      mock_payment_delay: {
        degraded_boundary: "Mock payment timeout must remain a real delayed upstream and platform-core must drive the timeout path to expired state.",
        timeout_http_status: ($mock_timeout_status | tonumber),
        timeout_seconds: ($mock_timeout_seconds | tonumber),
        threshold_seconds: ($min_timeout_seconds | tonumber)
      }
    }' >"${ARTIFACT_DIR}/summary.json"
}

trap cleanup EXIT

require_cmd cargo
require_cmd curl
require_cmd docker
require_cmd jq
require_cmd node
require_cmd psql
require_cmd rg
require_cmd awk

[[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
[[ -f "${RUNTIME_BASELINE_FILE}" ]] || fail "missing runtime baseline ${RUNTIME_BASELINE_FILE}"

mkdir -p "${ARTIFACT_DIR}"

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
# shellcheck disable=SC1090
source "${RUNTIME_BASELINE_FILE}"
set +a

HOST_KAFKA_BROKERS="${HOST_KAFKA_BROKERS:-127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094}}"
KAFKA_BROKERS="${KAFKA_BROKERS:-${HOST_KAFKA_BROKERS}}"
KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-${KAFKA_BROKERS}}"
DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
APP_PUBLIC_HOST="${APP_PUBLIC_HOST:-127.0.0.1}"
APP_PORT="${APP_PORT:-8094}"

export APP_MODE PROVIDER_MODE APP_HOST APP_PORT APP_PACKAGE
export DATABASE_URL KAFKA_BROKERS KAFKA_BOOTSTRAP_SERVERS

RUN_ID="$(date +%s%N)"
BASE_URL="http://${APP_PUBLIC_HOST}:${APP_PORT}"

log "running TEST-019 failure drill checker with env=${ENV_FILE}"

log "ensuring TEST-005 local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "aligning local Keycloak identities and demo baseline"
./scripts/seed-local-iam-test-identities.sh
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh
./scripts/check-mock-payment.sh

start_or_reuse_platform_core

log "verifying formal Keycloak subject for buyer drill traffic"
KEYCLOAK_TOKEN_USERNAME="local-buyer-operator" \
KEYCLOAK_TOKEN_PASSWORD="LocalBuyerOperator123!" \
KEYCLOAK_EXPECTED_ROLE="buyer_operator" \
./scripts/check-keycloak-realm.sh >/dev/null

BUYER_TOKEN="$(fetch_access_token "local-buyer-operator" "LocalBuyerOperator123!")"
IFS=$'\t' read -r BUYER_USER_ID BUYER_ORG_ID BUYER_ROLE < <(token_claims_tsv "${BUYER_TOKEN}")
[[ "${BUYER_ROLE}" == "buyer_operator" ]] || fail "buyer token role drifted: ${BUYER_ROLE}"

ORDER_FIXTURE_TSV="$(
  jq -r '
    .order_blueprints[]
    | select(.scenario_code == "S1" and .scenario_role == "primary")
    | [.product_id, .sku_id, .buyer_org_id]
    | @tsv
  ' fixtures/demo/orders.json
)"
[[ -n "${ORDER_FIXTURE_TSV}" ]] || fail "missing S1 primary order blueprint in fixtures/demo/orders.json"
IFS=$'\t' read -r PRODUCT_ID SKU_ID FIXTURE_BUYER_ORG_ID <<<"${ORDER_FIXTURE_TSV}"
[[ "${BUYER_ORG_ID}" == "${FIXTURE_BUYER_ORG_ID}" ]] || fail "buyer org drifted: token=${BUYER_ORG_ID} fixture=${FIXTURE_BUYER_ORG_ID}"

log "drill 1/4: kafka outage must not block order transaction commit"
compose stop kafka >/dev/null
KAFKA_STOPPED="true"
wait_dependency_state "kafka" "false" "${ARTIFACT_DIR}/health-deps-kafka-down.json" 90 2

ORDER_REQUEST_ID="req-test019-order-${RUN_ID}"
ORDER_CREATE_BODY="${ARTIFACT_DIR}/order-create-request.json"
cat >"${ORDER_CREATE_BODY}" <<JSON
{
  "buyer_org_id": "${FIXTURE_BUYER_ORG_ID}",
  "product_id": "${PRODUCT_ID}",
  "sku_id": "${SKU_ID}",
  "scenario_code": "S1"
}
JSON
ORDER_CREATE_RESPONSE="${ARTIFACT_DIR}/order-create-response.json"
measure_json_request \
  "order-create-kafka-down" \
  "POST" \
  "${BASE_URL}/api/v1/orders" \
  "${ORDER_CREATE_RESPONSE}" \
  "${ORDER_CREATE_BODY}" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${ORDER_REQUEST_ID}" \
  "x-idempotency-key: test019-order-create-${RUN_ID}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${ORDER_CREATE_RESPONSE}" "order-create-kafka-down"
ORDER_ID="$(jq -r '.data.order_id // empty' "${ORDER_CREATE_RESPONSE}")"
[[ -n "${ORDER_ID}" ]] || fail "kafka-down order response missing order_id"
jq -e '.data.current_state == "created"' "${ORDER_CREATE_RESPONSE}" >/dev/null \
  || fail "kafka-down order current_state drifted"

ORDER_DB_ROW="$(query_db_value "SELECT status || E'\t' || payment_status FROM trade.order_main WHERE order_id = '${ORDER_ID}'::uuid")"
[[ "${ORDER_DB_ROW}" == $'created\tunpaid' ]] || fail "unexpected order db state while kafka down: ${ORDER_DB_ROW}"
ORDER_OUTBOX_COUNT="$(query_db_value "SELECT COUNT(*)::text FROM ops.outbox_event WHERE request_id = '${ORDER_REQUEST_ID}' AND aggregate_type = 'trade.order'")"
[[ "${ORDER_OUTBOX_COUNT}" =~ ^[0-9]+$ ]] || fail "cannot read outbox count for kafka-down order"
(( ORDER_OUTBOX_COUNT > 0 )) || fail "expected outbox rows for kafka-down order request ${ORDER_REQUEST_ID}"
printf '%s\n' "${ORDER_DB_ROW}" >"${ARTIFACT_DIR}/kafka-down-order-db.txt"
printf '%s\n' "${ORDER_OUTBOX_COUNT}" >"${ARTIFACT_DIR}/kafka-down-outbox-count.txt"
ok "kafka outage preserved order + outbox commit (order_id=${ORDER_ID}, outbox_count=${ORDER_OUTBOX_COUNT})"

compose start kafka >/dev/null
KAFKA_STOPPED="false"
wait_kafka_ready
wait_dependency_state "kafka" "true" "${ARTIFACT_DIR}/health-deps-kafka-recovered.json" 120 2

log "drill 2/4: opensearch outage must keep local search on PostgreSQL fallback"
clear_search_cache
compose stop opensearch >/dev/null
OPENSEARCH_STOPPED="true"
wait_opensearch_down
compose ps opensearch >"${ARTIFACT_DIR}/opensearch-down-compose-ps.txt"

SEARCH_REQUEST_ID_1="req-test019-search-a-${RUN_ID}"
SEARCH_REQUEST_ID_2="req-test019-search-b-${RUN_ID}"
SEARCH_QUERY_ENCODED="$(jq -rn --arg value "${SEARCH_QUERY}" '$value | @uri')"
SEARCH_URL="${BASE_URL}/api/v1/catalog/search?q=${SEARCH_QUERY_ENCODED}&page=1&page_size=5"
SEARCH_RESPONSE_1="${ARTIFACT_DIR}/search-response-down-1.json"
SEARCH_RESPONSE_2="${ARTIFACT_DIR}/search-response-down-2.json"

measure_json_request \
  "search-opensearch-down-1" \
  "GET" \
  "${SEARCH_URL}" \
  "${SEARCH_RESPONSE_1}" \
  "" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${SEARCH_REQUEST_ID_1}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${SEARCH_RESPONSE_1}" "search-opensearch-down-1"
jq -e '.data.backend == "postgresql" and .data.cache_hit == false and (.data.items | length > 0)' "${SEARCH_RESPONSE_1}" >/dev/null \
  || fail "first search under opensearch outage did not stay on postgresql fallback"
assert_json_has_scalar "${SEARCH_RESPONSE_1}" "${PRODUCT_ID}" "search-opensearch-down-1"
SEARCH_REQUEST_ID_1="$(jq -r '.request_id' "${SEARCH_RESPONSE_1}")"
SEARCH_BACKEND_1="$(jq -r '.data.backend' "${SEARCH_RESPONSE_1}")"
SEARCH_CACHE_HIT_1="$(jq -r '.data.cache_hit' "${SEARCH_RESPONSE_1}")"

measure_json_request \
  "search-opensearch-down-2" \
  "GET" \
  "${SEARCH_URL}" \
  "${SEARCH_RESPONSE_2}" \
  "" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${SEARCH_REQUEST_ID_2}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${SEARCH_RESPONSE_2}" "search-opensearch-down-2"
jq -e '.data.backend == "postgresql" and .data.cache_hit == true and (.data.items | length > 0)' "${SEARCH_RESPONSE_2}" >/dev/null \
  || fail "second search under opensearch outage did not hit Redis-backed postgresql cache"
assert_json_has_scalar "${SEARCH_RESPONSE_2}" "${PRODUCT_ID}" "search-opensearch-down-2"
SEARCH_REQUEST_ID_2="$(jq -r '.request_id' "${SEARCH_RESPONSE_2}")"
SEARCH_BACKEND_2="$(jq -r '.data.backend' "${SEARCH_RESPONSE_2}")"
SEARCH_CACHE_HIT_2="$(jq -r '.data.cache_hit' "${SEARCH_RESPONSE_2}")"
ok "opensearch outage preserved PostgreSQL fallback + Redis cache"

compose start opensearch >/dev/null
OPENSEARCH_STOPPED="false"
wait_opensearch_ready
curl -fsS "http://127.0.0.1:${OPENSEARCH_HTTP_PORT}/_cluster/health" >"${ARTIFACT_DIR}/opensearch-cluster-health-recovered.json"

log "stopping host platform-core before fabric drill to isolate background anchor traffic"
stop_platform_core

log "drill 3/4: fabric-adapter outage must accumulate lag without faking receipts"
FABRIC_GROUP="cg-fabric-adapter-test019-${RUN_ID}"
reset_consumer_group_to_latest "${FABRIC_GROUP}" "${TOPIC_AUDIT_ANCHOR}" "${ARTIFACT_DIR}/fabric-group-reset-audit.txt"
reset_consumer_group_to_latest "${FABRIC_GROUP}" "${TOPIC_FABRIC_REQUESTS}" "${ARTIFACT_DIR}/fabric-group-reset-requests.txt"
start_fabric_adapter

FABRIC_WARMUP_EVENT_ID="$(new_uuid)"
FABRIC_WARMUP_REQUEST_ID="req-test019-fabric-warmup-${RUN_ID}"
FABRIC_WARMUP_TRACE_ID="trace-test019-fabric-warmup-${RUN_ID}"
FABRIC_WARMUP_ANCHOR_BATCH_ID="$(new_uuid)"
FABRIC_WARMUP_CHAIN_ANCHOR_ID="$(new_uuid)"
FABRIC_WARMUP_BATCH_ROOT="test019-warmup-${RUN_ID}"
seed_fabric_anchor \
  "${FABRIC_WARMUP_ANCHOR_BATCH_ID}" \
  "${FABRIC_WARMUP_CHAIN_ANCHOR_ID}" \
  "${FABRIC_WARMUP_BATCH_ROOT}" \
  "${FABRIC_WARMUP_EVENT_ID}" \
  "test019-fabric-warmup"
write_fabric_event \
  "${ARTIFACT_DIR}/fabric-warmup-event.json" \
  "${FABRIC_WARMUP_EVENT_ID}" \
  "${FABRIC_WARMUP_REQUEST_ID}" \
  "${FABRIC_WARMUP_TRACE_ID}" \
  "${FABRIC_WARMUP_ANCHOR_BATCH_ID}" \
  "${FABRIC_WARMUP_CHAIN_ANCHOR_ID}" \
  "${FABRIC_WARMUP_BATCH_ROOT}"
publish_kafka_event "${TOPIC_AUDIT_ANCHOR}" "${ARTIFACT_DIR}/fabric-warmup-event.json"
wait_fabric_receipt_count "${FABRIC_WARMUP_REQUEST_ID}" "1" 90
FABRIC_WARMUP_GROUP_DESCRIBE="${ARTIFACT_DIR}/fabric-group-warmup.txt"
FABRIC_WARMUP_LAG="$(fabric_group_lag_sum "${FABRIC_WARMUP_GROUP_DESCRIBE}")"
[[ "${FABRIC_WARMUP_LAG}" == "0" ]] || fail "fabric warmup group lag expected 0, got ${FABRIC_WARMUP_LAG}"

stop_fabric_adapter

FABRIC_DOWN_EVENT_ID="$(new_uuid)"
FABRIC_DOWN_REQUEST_ID="req-test019-fabric-down-${RUN_ID}"
FABRIC_DOWN_TRACE_ID="trace-test019-fabric-down-${RUN_ID}"
FABRIC_DOWN_ANCHOR_BATCH_ID="$(new_uuid)"
FABRIC_DOWN_CHAIN_ANCHOR_ID="$(new_uuid)"
FABRIC_DOWN_BATCH_ROOT="test019-down-${RUN_ID}"
seed_fabric_anchor \
  "${FABRIC_DOWN_ANCHOR_BATCH_ID}" \
  "${FABRIC_DOWN_CHAIN_ANCHOR_ID}" \
  "${FABRIC_DOWN_BATCH_ROOT}" \
  "${FABRIC_DOWN_EVENT_ID}" \
  "test019-fabric-down"
write_fabric_event \
  "${ARTIFACT_DIR}/fabric-down-event.json" \
  "${FABRIC_DOWN_EVENT_ID}" \
  "${FABRIC_DOWN_REQUEST_ID}" \
  "${FABRIC_DOWN_TRACE_ID}" \
  "${FABRIC_DOWN_ANCHOR_BATCH_ID}" \
  "${FABRIC_DOWN_CHAIN_ANCHOR_ID}" \
  "${FABRIC_DOWN_BATCH_ROOT}"
publish_kafka_event "${TOPIC_AUDIT_ANCHOR}" "${ARTIFACT_DIR}/fabric-down-event.json"
sleep 5

FABRIC_RECEIPT_DURING_STOP="$(query_db_value "SELECT COUNT(*)::text FROM ops.external_fact_receipt WHERE request_id = '${FABRIC_DOWN_REQUEST_ID}'")"
[[ "${FABRIC_RECEIPT_DURING_STOP}" == "0" ]] || fail "fabric-adapter down drill unexpectedly wrote receipt count=${FABRIC_RECEIPT_DURING_STOP}; stop any other fabric-adapter process first"
FABRIC_LAG_WHILE_STOPPED="$(fabric_group_lag_sum "${ARTIFACT_DIR}/fabric-group-down.txt")"
[[ "${FABRIC_LAG_WHILE_STOPPED}" =~ ^[0-9]+$ ]] || fail "cannot parse fabric group lag while stopped"
(( FABRIC_LAG_WHILE_STOPPED > 0 )) || fail "expected fabric group lag while adapter stopped"
ok "fabric-adapter outage accumulated lag=${FABRIC_LAG_WHILE_STOPPED} without receipt write-back"

start_fabric_adapter
wait_fabric_receipt_count "${FABRIC_DOWN_REQUEST_ID}" "1" 120
FABRIC_RECEIPT_AFTER_RECOVERY="$(query_db_value "SELECT COUNT(*)::text FROM ops.external_fact_receipt WHERE request_id = '${FABRIC_DOWN_REQUEST_ID}'")"
FABRIC_RECOVERY_LAG="$(fabric_group_lag_sum "${ARTIFACT_DIR}/fabric-group-recovered.txt")"
[[ "${FABRIC_RECOVERY_LAG}" == "0" ]] || fail "fabric recovery lag expected 0, got ${FABRIC_RECOVERY_LAG}"
ok "fabric-adapter recovery drained lag and persisted receipt"
stop_fabric_adapter

log "drill 4/4: mock payment timeout must remain a real delayed upstream and drive timeout state"
MOCK_TIMEOUT_RESPONSE="${ARTIFACT_DIR}/mock-payment-timeout-response.json"
read -r MOCK_TIMEOUT_STATUS MOCK_TIMEOUT_SECONDS <<EOF
$(curl -sS -X POST "${MOCK_PAYMENT_BASE_URL}/mock/payment/charge/timeout" \
  -o "${MOCK_TIMEOUT_RESPONSE}" \
  -w '%{http_code} %{time_total}')
EOF
[[ "${MOCK_TIMEOUT_STATUS}" == "504" ]] || fail "mock payment timeout endpoint returned HTTP ${MOCK_TIMEOUT_STATUS}"
threshold_ok "${MOCK_TIMEOUT_SECONDS}" "${MIN_TIMEOUT_SECONDS}" \
  || fail "mock payment timeout endpoint returned too quickly: ${MOCK_TIMEOUT_SECONDS}s < ${MIN_TIMEOUT_SECONDS}s"
ok "mock payment provider kept real delay (${MOCK_TIMEOUT_SECONDS}s)"

log "running official live mock payment timeout smoke against platform-core"
TRADE_DB_SMOKE=1 \
MOCK_PAYMENT_ADAPTER_MODE=live \
cargo test -p platform-core bil004_mock_payment_adapter_db_smoke -- --nocapture \
  | tee "${ARTIFACT_DIR}/bil004-mock-payment-live.log"
ok "platform-core live mock payment timeout smoke passed"

write_summary_json
ok "TEST-019 failure drill checker passed"
