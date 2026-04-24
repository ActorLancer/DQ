#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
RUNTIME_BASELINE_FILE="${RUNTIME_BASELINE_FILE:-fixtures/smoke/test-005/runtime-baseline.env}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/test-artifacts/performance-smoke}"
APP_LOG_FILE="${APP_LOG_FILE:-${ARTIFACT_DIR}/test-018-platform-core.log}"
KEYCLOAK_BASE_URL="${KEYCLOAK_BASE_URL:-http://127.0.0.1:8081}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-platform-local}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-portal-web}"
PROMETHEUS_BASE_URL="${PROMETHEUS_BASE_URL:-http://127.0.0.1:9090}"

MAX_SEARCH_SECONDS="${MAX_SEARCH_SECONDS:-2.0}"
MAX_ORDER_SECONDS="${MAX_ORDER_SECONDS:-2.0}"
MAX_DELIVERY_SECONDS="${MAX_DELIVERY_SECONDS:-2.0}"
MAX_AUDIT_SECONDS="${MAX_AUDIT_SECONDS:-2.0}"

APP_PID=""
APP_STARTED_BY_CHECKER="false"
ORDER_ID=""
APP_ID=""
DELIVERY_ID=""
CONTRACT_TEMPLATE_ID=""
ASSET_OBJECT_ID=""

declare -A TIMES=()
declare -A STATUS_CODES=()
declare -A REQUEST_IDS=()

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
    tail -n 60 "${APP_LOG_FILE}" >&2 || true
  fi
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
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

stop_platform_core() {
  if [[ "${APP_STARTED_BY_CHECKER}" == "true" ]] && [[ -n "${APP_PID}" ]] && kill -0 "${APP_PID}" >/dev/null 2>&1; then
    kill "${APP_PID}" >/dev/null 2>&1 || true
    wait "${APP_PID}" >/dev/null 2>&1 || true
  fi
}

cleanup_test_data() {
  if [[ -z "${ORDER_ID}" ]]; then
    return 0
  fi

  log "cleaning TEST-018 business data for order ${ORDER_ID}"
  psql "${DATABASE_URL}" \
    -v ON_ERROR_STOP=1 \
    -v order_id="${ORDER_ID}" \
    -v app_id="${APP_ID}" \
    -v contract_template_id="${CONTRACT_TEMPLATE_ID}" \
    -v asset_object_id="${ASSET_OBJECT_ID}" >/dev/null <<'SQL'
DELETE FROM trade.order_main
WHERE order_id = :'order_id'::uuid;

DELETE FROM catalog.asset_object_binding
WHERE asset_object_id = NULLIF(:'asset_object_id', '')::uuid;

DELETE FROM core.application
WHERE app_id = NULLIF(:'app_id', '')::uuid;

DELETE FROM contract.contract_signer
WHERE contract_id IN (
  SELECT contract_id
  FROM contract.digital_contract
  WHERE order_id = :'order_id'::uuid
);

DELETE FROM contract.digital_contract
WHERE order_id = :'order_id'::uuid;

DELETE FROM contract.template_definition
WHERE template_id = NULLIF(:'contract_template_id', '')::uuid;
SQL
  ok "TEST-018 business data cleaned"
}

cleanup() {
  cleanup_test_data
  stop_platform_core
}

trap cleanup EXIT

start_or_reuse_platform_core() {
  local base_url="http://${APP_PUBLIC_HOST}:${APP_PORT}"
  local live_code ready_code

  live_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${HEALTH_LIVE_PATH}" || true)"
  ready_code="$(curl -sS -o /dev/null -w '%{http_code}' "${base_url}${HEALTH_READY_PATH}" || true)"
  if [[ "${live_code}" == "200" && "${ready_code}" == "200" ]]; then
    ok "reusing existing platform-core on ${base_url}"
    return 0
  fi

  mkdir -p "${ARTIFACT_DIR}"
  rm -f "${APP_LOG_FILE}"

  log "starting host platform-core TEST-018 instance on ${base_url}"
  cargo run -p "${APP_PACKAGE}" >"${APP_LOG_FILE}" 2>&1 &
  APP_PID="$!"
  APP_STARTED_BY_CHECKER="true"

  wait_http_code "platform-core live" "${base_url}${HEALTH_LIVE_PATH}" '^200$' 240 2
  wait_http_code "platform-core ready" "${base_url}${HEALTH_READY_PATH}" '^200$' 240 2
}

query_db_value() {
  local sql="$1"
  psql "${DATABASE_URL}" -Atqc "${sql}"
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
    const role = Array.isArray(payload.realm_access?.roles) ? payload.realm_access.roles[0] ?? "" : "";
    console.log([payload.user_id ?? "", payload.org_id ?? "", role].join("\t"));
  ' "${token}"
}

threshold_ok() {
  local actual="$1"
  local threshold="$2"
  awk -v actual="${actual}" -v threshold="${threshold}" 'BEGIN { exit(actual <= threshold ? 0 : 1) }'
}

measure_request() {
  local name="$1"
  local method="$2"
  local url="$3"
  local response_file="$4"
  local body_file="$5"
  local threshold="$6"
  shift 6

  local -a curl_args=(curl -sS -X "${method}" "${url}" -o "${response_file}" -w '%{http_code} %{time_total}')
  if [[ -n "${body_file}" ]]; then
    curl_args+=(-H 'content-type: application/json' --data @"${body_file}")
  fi
  while (( "$#" )); do
    curl_args+=(-H "$1")
    shift
  done

  local meta
  meta="$("${curl_args[@]}")" || fail "${name} request failed to execute"
  read -r STATUS_CODES["${name}"] TIMES["${name}"] <<<"${meta}"

  if [[ "${STATUS_CODES[${name}]}" != "200" ]]; then
    echo "[fail] ${name} response body:" >&2
    cat "${response_file}" >&2
    fail "${name} returned HTTP ${STATUS_CODES[${name}]}"
  fi

  if [[ -n "${threshold}" ]] && ! threshold_ok "${TIMES[${name}]}" "${threshold}"; then
    fail "${name} exceeded threshold ${threshold}s (actual ${TIMES[${name}]}s)"
  fi

  printf '%s\n' "${STATUS_CODES[${name}]}" >"${ARTIFACT_DIR}/${name}.status.txt"
  printf '%s\n' "${TIMES[${name}]}" >"${ARTIFACT_DIR}/${name}.time.txt"
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

query_prometheus() {
  local query="$1"
  local output_file="$2"
  local encoded

  encoded="$(jq -rn --arg value "${query}" '$value | @uri')"
  curl -fsS "${PROMETHEUS_BASE_URL}/api/v1/query?query=${encoded}" >"${output_file}"
}

check_metrics_snapshot() {
  local metrics_file="${ARTIFACT_DIR}/platform-core-metrics.prom"
  curl -fsS "http://${APP_PUBLIC_HOST}:${APP_PORT}/metrics" >"${metrics_file}"

  local -a patterns=(
    'platform_core_http_request_duration_seconds_count{method="GET",path="/api/v1/catalog/search"}'
    'platform_core_http_request_duration_seconds_count{method="POST",path="/api/v1/orders"}'
    'platform_core_http_request_duration_seconds_count{method="POST",path="/api/v1/orders/{id}/deliver"}'
    'platform_core_http_request_duration_seconds_count{method="GET",path="/api/v1/audit/orders/{id}"}'
  )
  local pattern
  for pattern in "${patterns[@]}"; do
    rg -F "${pattern}" "${metrics_file}" >/dev/null || fail "missing metrics evidence for ${pattern}"
  done
}

write_summary_json() {
  jq -n \
    --arg run_id "${RUN_ID}" \
    --arg order_id "${ORDER_ID}" \
    --arg app_id "${APP_ID}" \
    --arg delivery_id "${DELIVERY_ID}" \
    --arg search_threshold "${MAX_SEARCH_SECONDS}" \
    --arg order_threshold "${MAX_ORDER_SECONDS}" \
    --arg delivery_threshold "${MAX_DELIVERY_SECONDS}" \
    --arg audit_threshold "${MAX_AUDIT_SECONDS}" \
    --arg search_time "${TIMES[search]}" \
    --arg create_time "${TIMES[order-create]}" \
    --arg deliver_time "${TIMES[delivery]}" \
    --arg audit_time "${TIMES[audit-order]}" \
    --arg search_request_id "${REQUEST_IDS[search]}" \
    --arg create_request_id "${REQUEST_IDS[order-create]}" \
    --arg deliver_request_id "${REQUEST_IDS[delivery]}" \
    --arg audit_request_id "${REQUEST_IDS[audit-order]}" \
    '{
      task_id: "TEST-018",
      run_id: $run_id,
      threshold_source: "26.2 性能 SLO：标准下单/合同查看/账单查询 p95 <= 2 秒；本 smoke 按单次正式 API <= 2.0 秒守门",
      cleanup_target: {
        order_id: $order_id,
        app_id: ($app_id | select(length > 0)),
        delivery_id: ($delivery_id | select(length > 0))
      },
      calls: [
        {
          name: "search",
          method: "GET",
          path: "/api/v1/catalog/search",
          request_id: $search_request_id,
          threshold_seconds: ($search_threshold | tonumber),
          actual_seconds: ($search_time | tonumber)
        },
        {
          name: "order-create",
          method: "POST",
          path: "/api/v1/orders",
          request_id: $create_request_id,
          threshold_seconds: ($order_threshold | tonumber),
          actual_seconds: ($create_time | tonumber)
        },
        {
          name: "delivery",
          method: "POST",
          path: "/api/v1/orders/{id}/deliver",
          request_id: $deliver_request_id,
          threshold_seconds: ($delivery_threshold | tonumber),
          actual_seconds: ($deliver_time | tonumber)
        },
        {
          name: "audit-order",
          method: "GET",
          path: "/api/v1/audit/orders/{id}",
          request_id: $audit_request_id,
          threshold_seconds: ($audit_threshold | tonumber),
          actual_seconds: ($audit_time | tonumber)
        }
      ]
    }' >"${ARTIFACT_DIR}/summary.json"
}

require_cmd cargo
require_cmd curl
require_cmd docker
require_cmd jq
require_cmd node
require_cmd psql
require_cmd rg

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
LATEST_MIGRATION_VERSION="$(awk -F, 'NR > 1 {version = $1} END {print version}' db/migrations/v1/manifest.csv)"
[[ -n "${LATEST_MIGRATION_VERSION}" ]] || fail "cannot resolve latest migration version"
MIGRATION_VERSION="${MIGRATION_VERSION:-${LATEST_MIGRATION_VERSION}}"

export APP_MODE PROVIDER_MODE APP_HOST APP_PORT APP_PACKAGE MIGRATION_VERSION
export DATABASE_URL KAFKA_BROKERS KAFKA_BOOTSTRAP_SERVERS

RUN_ID="$(date +%s%N)"
BASE_URL="http://${APP_PUBLIC_HOST}:${APP_PORT}"

log "running TEST-018 performance smoke checker with env=${ENV_FILE}"
start_or_reuse_platform_core

log "ensuring TEST-005 local environment baseline"
ENV_FILE="${ENV_FILE}" bash ./scripts/smoke-local.sh

log "aligning local Keycloak identities with core/authz principals"
./scripts/seed-local-iam-test-identities.sh

log "loading formal demo order baseline"
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh

log "verifying formal Keycloak subjects for search/order/delivery/audit"
KEYCLOAK_TOKEN_USERNAME="local-buyer-operator" \
KEYCLOAK_TOKEN_PASSWORD="LocalBuyerOperator123!" \
KEYCLOAK_EXPECTED_ROLE="buyer_operator" \
./scripts/check-keycloak-realm.sh >/dev/null
KEYCLOAK_TOKEN_USERNAME="local-tenant-developer" \
KEYCLOAK_TOKEN_PASSWORD="LocalTenantDeveloper123!" \
KEYCLOAK_EXPECTED_ROLE="tenant_developer" \
./scripts/check-keycloak-realm.sh >/dev/null
KEYCLOAK_TOKEN_USERNAME="local-audit-security" \
KEYCLOAK_TOKEN_PASSWORD="LocalAuditSecurity123!" \
KEYCLOAK_EXPECTED_ROLE="platform_audit_security" \
./scripts/check-keycloak-realm.sh >/dev/null
ok "keycloak buyer/developer/audit grant probe passed"

log "loading Bearer tokens and formal subject claims"
BUYER_TOKEN="$(fetch_access_token "local-buyer-operator" "LocalBuyerOperator123!")"
TENANT_DEVELOPER_TOKEN="$(fetch_access_token "local-tenant-developer" "LocalTenantDeveloper123!")"
AUDIT_TOKEN="$(fetch_access_token "local-audit-security" "LocalAuditSecurity123!")"

IFS=$'\t' read -r BUYER_USER_ID BUYER_ORG_ID BUYER_ROLE < <(token_claims_tsv "${BUYER_TOKEN}")
IFS=$'\t' read -r TENANT_DEVELOPER_USER_ID TENANT_DEVELOPER_ORG_ID TENANT_DEVELOPER_ROLE < <(token_claims_tsv "${TENANT_DEVELOPER_TOKEN}")
IFS=$'\t' read -r AUDIT_USER_ID AUDIT_ORG_ID AUDIT_ROLE < <(token_claims_tsv "${AUDIT_TOKEN}")

[[ "${BUYER_ROLE}" == "buyer_operator" ]] || fail "buyer token role drifted: ${BUYER_ROLE}"
[[ "${TENANT_DEVELOPER_ROLE}" == "tenant_developer" ]] || fail "developer token role drifted: ${TENANT_DEVELOPER_ROLE}"
[[ "${AUDIT_ROLE}" == "platform_audit_security" ]] || fail "audit token role drifted: ${AUDIT_ROLE}"

log "capturing Prometheus readiness for platform-core"
query_prometheus 'up{job="platform-core"}' "${ARTIFACT_DIR}/prometheus-platform-core-up.json"
jq -e '.status == "success" and any(.data.result[]?; .value[1] == "1")' \
  "${ARTIFACT_DIR}/prometheus-platform-core-up.json" >/dev/null \
  || fail "Prometheus does not report platform-core as up"

ORDER_FIXTURE_TSV="$(
  jq -r '
    .order_blueprints[]
    | select(.scenario_code == "S1" and .scenario_role == "primary")
    | [.product_id, .sku_id, .buyer_org_id, .template_codes.contract_template]
    | @tsv
  ' fixtures/demo/orders.json
)"
[[ -n "${ORDER_FIXTURE_TSV}" ]] || fail "missing S1 primary order blueprint in fixtures/demo/orders.json"
IFS=$'\t' read -r PRODUCT_ID SKU_ID FIXTURE_BUYER_ORG_ID CONTRACT_TEMPLATE_NAME <<<"${ORDER_FIXTURE_TSV}"

CONTRACT_TEMPLATE_ID="$(
  query_db_value "INSERT INTO contract.template_definition (
                    template_id,
                    template_type,
                    template_name,
                    version_no,
                    applicable_sku_types,
                    configurable_fields,
                    locked_fields,
                    content_digest,
                    status,
                    metadata
                  ) VALUES (
                    gen_random_uuid(),
                    'contract',
                    '${CONTRACT_TEMPLATE_NAME}',
                    1,
                    ARRAY['API_SUB'],
                    '[]'::jsonb,
                    '[]'::jsonb,
                    'digest-test018-${RUN_ID}',
                    'active',
                    jsonb_build_object('seed', 'test018-performance-smoke', 'run_id', '${RUN_ID}')
                  )
                  RETURNING template_id::text"
)"
[[ -n "${CONTRACT_TEMPLATE_ID}" ]] || fail "cannot create TEST-018 contract template for ${CONTRACT_TEMPLATE_NAME}"

REQUEST_IDS[search]="req-test018-search-${RUN_ID}"
REQUEST_IDS[order-create]="req-test018-create-${RUN_ID}"
REQUEST_IDS[contract-confirm]="req-test018-contract-${RUN_ID}"
REQUEST_IDS[api-sub-lock]="req-test018-lock-${RUN_ID}"
REQUEST_IDS[delivery]="req-test018-deliver-${RUN_ID}"
REQUEST_IDS[audit-order]="req-test018-audit-${RUN_ID}"

SEARCH_RESPONSE="${ARTIFACT_DIR}/search-response.json"
SEARCH_QUERY='工业设备运行指标 API 订阅'
SEARCH_QUERY_ENCODED="$(jq -rn --arg value "${SEARCH_QUERY}" '$value | @uri')"
SEARCH_URL="${BASE_URL}/api/v1/catalog/search?q=${SEARCH_QUERY_ENCODED}&page=1&page_size=5"
measure_request \
  "search" \
  "GET" \
  "${SEARCH_URL}" \
  "${SEARCH_RESPONSE}" \
  "" \
  "${MAX_SEARCH_SECONDS}" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[search]}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${SEARCH_RESPONSE}" "search"
REQUEST_IDS[search]="$(jq -r '.request_id' "${SEARCH_RESPONSE}")"
jq -e '.data.items | length > 0' "${SEARCH_RESPONSE}" >/dev/null || fail "search returned zero items"
assert_json_has_scalar "${SEARCH_RESPONSE}" "${PRODUCT_ID}" "search"
ok "search latency ${TIMES[search]}s <= ${MAX_SEARCH_SECONDS}s"

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
measure_request \
  "order-create" \
  "POST" \
  "${BASE_URL}/api/v1/orders" \
  "${ORDER_CREATE_RESPONSE}" \
  "${ORDER_CREATE_BODY}" \
  "${MAX_ORDER_SECONDS}" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[order-create]}" \
  "x-idempotency-key: test018-order-create-${RUN_ID}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${ORDER_CREATE_RESPONSE}" "order-create"
REQUEST_IDS[order-create]="$(jq -r '.request_id' "${ORDER_CREATE_RESPONSE}")"
ORDER_ID="$(jq -r '.data.order_id // empty' "${ORDER_CREATE_RESPONSE}")"
[[ -n "${ORDER_ID}" ]] || fail "order-create response missing order_id"
jq -e '.data.current_state == "created"' "${ORDER_CREATE_RESPONSE}" >/dev/null \
  || fail "order-create current_state drifted"
assert_json_has_scalar "${ORDER_CREATE_RESPONSE}" "${PRODUCT_ID}" "order-create"
ok "order create latency ${TIMES[order-create]}s <= ${MAX_ORDER_SECONDS}s"

CONTRACT_CONFIRM_BODY="${ARTIFACT_DIR}/contract-confirm-request.json"
cat >"${CONTRACT_CONFIRM_BODY}" <<JSON
{
  "contract_template_id": "${CONTRACT_TEMPLATE_ID}",
  "contract_digest": "sha256:test018-contract:${RUN_ID}",
  "signer_role": "buyer_operator",
  "variables_json": {
    "source": "test018-performance-smoke"
  }
}
JSON
CONTRACT_CONFIRM_RESPONSE="${ARTIFACT_DIR}/contract-confirm-response.json"
measure_request \
  "contract-confirm" \
  "POST" \
  "${BASE_URL}/api/v1/orders/${ORDER_ID}/contract-confirm" \
  "${CONTRACT_CONFIRM_RESPONSE}" \
  "${CONTRACT_CONFIRM_BODY}" \
  "" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[contract-confirm]}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${CONTRACT_CONFIRM_RESPONSE}" "contract-confirm"
REQUEST_IDS[contract-confirm]="$(jq -r '.request_id' "${CONTRACT_CONFIRM_RESPONSE}")"
jq -e '.data.current_state == "contract_effective"' "${CONTRACT_CONFIRM_RESPONSE}" >/dev/null \
  || fail "contract-confirm current_state drifted"

API_SUB_LOCK_BODY="${ARTIFACT_DIR}/api-sub-lock-request.json"
cat >"${API_SUB_LOCK_BODY}" <<JSON
{
  "action": "lock_funds"
}
JSON
API_SUB_LOCK_RESPONSE="${ARTIFACT_DIR}/api-sub-lock-response.json"
measure_request \
  "api-sub-lock" \
  "POST" \
  "${BASE_URL}/api/v1/orders/${ORDER_ID}/api-sub/transition" \
  "${API_SUB_LOCK_RESPONSE}" \
  "${API_SUB_LOCK_BODY}" \
  "" \
  "authorization: Bearer ${BUYER_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[api-sub-lock]}" \
  "x-user-id: ${BUYER_USER_ID}" \
  "x-tenant-id: ${BUYER_ORG_ID}" \
  "x-role: ${BUYER_ROLE}"
assert_ok_envelope "${API_SUB_LOCK_RESPONSE}" "api-sub-lock"
REQUEST_IDS[api-sub-lock]="$(jq -r '.request_id' "${API_SUB_LOCK_RESPONSE}")"
jq -e '.data.current_state == "buyer_locked" and .data.payment_status == "paid"' \
  "${API_SUB_LOCK_RESPONSE}" >/dev/null || fail "api-sub lock_funds did not reach buyer_locked/paid"

ASSET_OBJECT_ID="$(
  query_db_value "WITH order_asset AS (
                    SELECT asset_version_id
                    FROM trade.order_main
                    WHERE order_id = '${ORDER_ID}'::uuid
                  )
                  INSERT INTO catalog.asset_object_binding (
                    asset_version_id,
                    object_kind,
                    object_name,
                    object_locator,
                    schema_json,
                    output_schema_json,
                    freshness_json,
                    access_constraints,
                    metadata
                  )
                  SELECT
                    order_asset.asset_version_id,
                    'api_endpoint',
                    'test018-api-endpoint-${RUN_ID}',
                    'https://api.local.test/test018/${RUN_ID}/v1',
                    '{}'::jsonb,
                    '{}'::jsonb,
                    '{}'::jsonb,
                    '{\"rate_limit_profile\":{\"requests_per_minute\":60,\"burst\":10,\"concurrency\":3}}'::jsonb,
                    jsonb_build_object('seed', 'test018-performance-smoke', 'run_id', '${RUN_ID}')
                  FROM order_asset
                  RETURNING asset_object_id::text"
)"
[[ -n "${ASSET_OBJECT_ID}" ]] || fail "cannot seed api_endpoint asset_object_id for ${ORDER_ID}"

DELIVERY_BODY="${ARTIFACT_DIR}/delivery-request.json"
cat >"${DELIVERY_BODY}" <<JSON
{
  "branch": "api",
  "asset_object_id": "${ASSET_OBJECT_ID}",
  "app_name": "test018-app-${RUN_ID}",
  "quota_json": {
    "billing_mode": "subscription",
    "period": "monthly",
    "included_calls": 1000
  },
  "rate_limit_json": {
    "requests_per_minute": 60,
    "burst": 10,
    "concurrency": 3
  },
  "upstream_mode": "platform_proxy",
  "expire_at": "2027-01-01T00:00:00Z",
  "delivery_commit_hash": "test018-delivery-commit-${RUN_ID}",
  "receipt_hash": "test018-delivery-receipt-${RUN_ID}"
}
JSON
DELIVERY_RESPONSE="${ARTIFACT_DIR}/delivery-response.json"
measure_request \
  "delivery" \
  "POST" \
  "${BASE_URL}/api/v1/orders/${ORDER_ID}/deliver" \
  "${DELIVERY_RESPONSE}" \
  "${DELIVERY_BODY}" \
  "${MAX_DELIVERY_SECONDS}" \
  "authorization: Bearer ${TENANT_DEVELOPER_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[delivery]}" \
  "x-idempotency-key: test018-delivery-${RUN_ID}" \
  "x-user-id: ${TENANT_DEVELOPER_USER_ID}" \
  "x-tenant-id: ${TENANT_DEVELOPER_ORG_ID}" \
  "x-role: ${TENANT_DEVELOPER_ROLE}"
assert_ok_envelope "${DELIVERY_RESPONSE}" "delivery"
REQUEST_IDS[delivery]="$(jq -r '.request_id' "${DELIVERY_RESPONSE}")"
jq -e '
  .data.current_state == "api_key_issued" and
  (.data.api_credential_id | type == "string" and length > 0)
' "${DELIVERY_RESPONSE}" >/dev/null || fail "delivery response drifted from API_SUB committed state"
APP_ID="$(jq -r '.data.app_id // empty' "${DELIVERY_RESPONSE}")"
DELIVERY_ID="$(jq -r '.data.delivery_id // empty' "${DELIVERY_RESPONSE}")"
ok "delivery latency ${TIMES[delivery]}s <= ${MAX_DELIVERY_SECONDS}s"

AUDIT_RESPONSE="${ARTIFACT_DIR}/audit-order-response.json"
measure_request \
  "audit-order" \
  "GET" \
  "${BASE_URL}/api/v1/audit/orders/${ORDER_ID}?page=1&page_size=20" \
  "${AUDIT_RESPONSE}" \
  "" \
  "${MAX_AUDIT_SECONDS}" \
  "authorization: Bearer ${AUDIT_TOKEN}" \
  "x-request-id: ${REQUEST_IDS[audit-order]}" \
  "x-user-id: ${AUDIT_USER_ID}" \
  "x-tenant-id: ${AUDIT_ORG_ID}" \
  "x-role: ${AUDIT_ROLE}"
assert_ok_envelope "${AUDIT_RESPONSE}" "audit-order"
REQUEST_IDS[audit-order]="$(jq -r '.request_id' "${AUDIT_RESPONSE}")"
jq -e --arg order_id "${ORDER_ID}" '
  .data.order_id == $order_id and
  (.data.traces | length > 0)
' "${AUDIT_RESPONSE}" >/dev/null || fail "audit order lookup returned no traces"
ok "audit lookup latency ${TIMES[audit-order]}s <= ${MAX_AUDIT_SECONDS}s"

log "capturing HTTP metrics evidence for TEST-018"
check_metrics_snapshot

write_summary_json

ok "TEST-018 performance smoke passed"
