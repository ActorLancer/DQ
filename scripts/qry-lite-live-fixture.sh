#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
PSQL_BIN="${PSQL_BIN:-psql}"
JQ_BIN="${JQ_BIN:-jq}"
MINIO_MC_IMAGE="${MINIO_MC_IMAGE:-minio/mc:RELEASE.2025-08-13T08-35-41Z}"
RISK_USER_ID="${RISK_USER_ID:-10000000-0000-0000-0000-000000000359}"
RISK_ORG_ID="${RISK_ORG_ID:-10000000-0000-0000-0000-000000000103}"

usage() {
  cat <<'EOF'
Usage:
  ENV_FILE=infra/docker/.env.local ./scripts/qry-lite-live-fixture.sh prepare > target/test-artifacts/qry-lite-e2e/live-fixture.json
  ENV_FILE=infra/docker/.env.local ./scripts/qry-lite-live-fixture.sh cleanup --fixture target/test-artifacts/qry-lite-e2e/live-fixture.json [--portal-artifact target/test-artifacts/qry-lite-e2e/raw/portal-qry-lite-live.json]
EOF
}

fail() {
  echo "[fail] $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

load_env() {
  [[ -f "${ENV_FILE}" ]] || fail "missing env file ${ENV_FILE}"
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
  DATABASE_URL="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
  MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:${MINIO_API_PORT:-9000}}"
}

read_demo_value() {
  local jq_expr="$1"
  "${JQ_BIN}" -r "${jq_expr}" fixtures/demo/orders.json
}

read_demo_subject_value() {
  local jq_expr="$1"
  "${JQ_BIN}" -r "${jq_expr}" fixtures/demo/subjects.json
}

prepare_fixture() {
  local product_id sku_id demo_order_id order_amount buyer_org_id buyer_user_id seller_org_id suffix fixture_json

  product_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S5" and .scenario_role == "primary") | .product_id')"
  sku_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S5" and .scenario_role == "primary") | .sku_id')"
  demo_order_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S5" and .scenario_role == "primary") | .order_blueprint_id')"
  order_amount="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S5" and .scenario_role == "primary") | .order_amount')"
  buyer_org_id="$(read_demo_subject_value '.users[] | select(.persona == "buyer_operator") | .org_id' | head -n 1)"
  buyer_user_id="$(read_demo_subject_value '.users[] | select(.persona == "buyer_operator") | .user_id' | head -n 1)"
  seller_org_id="$(read_demo_subject_value '.users[] | select(.persona == "seller_operator") | .org_id' | head -n 1)"

  [[ -n "${product_id}" && "${product_id}" != "null" ]] || fail "missing S5 QRY_LITE product_id from fixtures/demo/orders.json"
  [[ -n "${sku_id}" && "${sku_id}" != "null" ]] || fail "missing S5 QRY_LITE sku_id from fixtures/demo/orders.json"
  [[ -n "${demo_order_id}" && "${demo_order_id}" != "null" ]] || fail "missing S5 QRY_LITE order_blueprint_id from fixtures/demo/orders.json"
  [[ -n "${order_amount}" && "${order_amount}" != "null" ]] || fail "missing S5 QRY_LITE order_amount from fixtures/demo/orders.json"
  [[ -n "${buyer_org_id}" && "${buyer_org_id}" != "null" ]] || fail "missing buyer org from fixtures/demo/subjects.json"
  [[ -n "${buyer_user_id}" && "${buyer_user_id}" != "null" ]] || fail "missing buyer user from fixtures/demo/subjects.json"
  [[ -n "${seller_org_id}" && "${seller_org_id}" != "null" ]] || fail "missing seller org from fixtures/demo/subjects.json"

  suffix="test026-$(date +%s)-$$"
  fixture_json="$("${PSQL_BIN}" "${DATABASE_URL}" -tA -v ON_ERROR_STOP=1 <<SQL
WITH demo_order AS (
  SELECT
    price_snapshot_json,
    trust_boundary_snapshot,
    delivery_route_snapshot
  FROM trade.order_main
  WHERE order_id = '${demo_order_id}'::uuid
),
product_ctx AS (
  SELECT
    p.product_id::text AS product_id,
    p.asset_version_id::text AS asset_version_id,
    demo_order.price_snapshot_json,
    demo_order.trust_boundary_snapshot,
    demo_order.delivery_route_snapshot
  FROM catalog.product p
  JOIN demo_order ON true
  WHERE p.product_id = '${product_id}'::uuid
),
query_surface_ctx AS (
  SELECT
    q.query_surface_id::text AS query_surface_id,
    q.asset_object_id::text AS asset_object_id,
    q.environment_id::text AS environment_id
  FROM catalog.query_surface_definition q
  JOIN product_ctx ON q.asset_version_id = product_ctx.asset_version_id::uuid
  WHERE q.status = 'active'
  ORDER BY q.updated_at DESC, q.query_surface_id DESC
  LIMIT 1
),
query_template_ctx AS (
  SELECT
    t.query_template_id::text AS query_template_id
  FROM delivery.query_template_definition t
  JOIN query_surface_ctx ON t.query_surface_id = query_surface_ctx.query_surface_id::uuid
  WHERE t.status = 'active'
  ORDER BY t.version_no DESC, t.query_template_id DESC
  LIMIT 1
),
order_row AS (
  INSERT INTO trade.order_main (
    product_id,
    asset_version_id,
    buyer_org_id,
    seller_org_id,
    sku_id,
    status,
    payment_status,
    delivery_status,
    acceptance_status,
    settlement_status,
    dispute_status,
    payment_mode,
    amount,
    currency_code,
    price_snapshot_json,
    delivery_route_snapshot,
    trust_boundary_snapshot,
    idempotency_key,
    last_reason_code
  )
  SELECT
    product_ctx.product_id::uuid,
    product_ctx.asset_version_id::uuid,
    '${buyer_org_id}'::uuid,
    '${seller_org_id}'::uuid,
    '${sku_id}'::uuid,
    'buyer_locked',
    'paid',
    'pending_delivery',
    'not_started',
    'pending_settlement',
    'none',
    'online',
    '${order_amount}'::numeric,
    'SGD',
    product_ctx.price_snapshot_json,
    product_ctx.delivery_route_snapshot,
    product_ctx.trust_boundary_snapshot,
    'test026-qry-lite-live-${suffix}',
    'test026_fixture_prepared'
  FROM product_ctx
  RETURNING order_id::text
),
contract_row AS (
  INSERT INTO contract.digital_contract (
    order_id,
    contract_digest,
    status,
    signed_at,
    variables_json
  )
  SELECT
    order_row.order_id::uuid,
    'sha256:test026:' || order_row.order_id,
    'signed',
    now(),
    jsonb_build_object('task_id', 'TEST-026', 'fixture_suffix', '${suffix}')
  FROM order_row
  RETURNING contract_id::text
),
payment_intent_row AS (
  INSERT INTO payment.payment_intent (
    order_id,
    intent_type,
    provider_key,
    provider_account_id,
    payer_subject_type,
    payer_subject_id,
    payee_subject_type,
    payee_subject_id,
    payer_jurisdiction_code,
    payee_jurisdiction_code,
    launch_jurisdiction_code,
    amount,
    payment_method,
    currency_code,
    price_currency_code,
    status,
    request_id,
    idempotency_key,
    capability_snapshot,
    metadata
  )
  SELECT
    order_row.order_id::uuid,
    'order_payment',
    'mock_payment',
    NULL,
    'organization',
    '${buyer_org_id}'::uuid,
    'organization',
    '${seller_org_id}'::uuid,
    'SG',
    'SG',
    'SG',
    '${order_amount}'::numeric,
    'wallet',
    'SGD',
    'SGD',
    'succeeded',
    'req-test026-payment-${suffix}',
    'pay:test026:${suffix}',
    '{"supports_refund":true}'::jsonb,
    jsonb_build_object('task_id', 'TEST-026', 'fixture_suffix', '${suffix}')
  FROM order_row
  RETURNING payment_intent_id::text
),
settlement_row AS (
  INSERT INTO billing.settlement_record (
    order_id,
    settlement_type,
    settlement_status,
    settlement_mode,
    payable_amount,
    platform_fee_amount,
    channel_fee_amount,
    net_receivable_amount,
    refund_amount,
    compensation_amount,
    reason_code,
    settled_at
  )
  SELECT
    order_row.order_id::uuid,
    'order_settlement',
    'pending',
    'manual',
    '${order_amount}'::numeric,
    0.00000000,
    0.00000000,
    '${order_amount}'::numeric,
    0.00000000,
    0.00000000,
    'test026_fixture',
    NULL
  FROM order_row
  RETURNING settlement_id::text
),
approval_ticket_row AS (
  INSERT INTO ops.approval_ticket (
    ticket_type,
    ref_type,
    ref_id,
    requested_by,
    status,
    requires_second_review
  )
  SELECT
    'query_run',
    'order',
    order_row.order_id::uuid,
    '${buyer_user_id}'::uuid,
    'approved',
    false
  FROM order_row
  RETURNING approval_ticket_id::text
),
case_row AS (
  INSERT INTO support.dispute_case (
    order_id,
    complainant_type,
    complainant_id,
    reason_code,
    status,
    decision_code,
    penalty_code
  )
  SELECT
    order_row.order_id::uuid,
    'organization',
    '${buyer_org_id}'::uuid,
    'query_result_refund',
    'manual_review',
    'refund_full',
    'seller_warning'
  FROM order_row
  RETURNING case_id::text
),
decision_row AS (
  INSERT INTO support.decision_record (
    case_id,
    decision_type,
    decision_code,
    liability_type,
    decision_text,
    decided_by
  )
  SELECT
    case_row.case_id::uuid,
    'manual_resolution',
    'refund_full',
    'seller',
    'test026 qry lite refund approved',
    '${RISK_USER_ID}'::uuid
  FROM case_row
  RETURNING decision_id::text
)
SELECT json_build_object(
  'fixture_id', 'test026-qry-lite-live-fixture',
  'task_id', 'TEST-026',
  'suffix', '${suffix}',
  'product_id', product_ctx.product_id,
  'sku_id', '${sku_id}',
  'buyer_org_id', '${buyer_org_id}',
  'buyer_user_id', '${buyer_user_id}',
  'seller_org_id', '${seller_org_id}',
  'risk_org_id', '${RISK_ORG_ID}',
  'risk_user_id', '${RISK_USER_ID}',
  'query_surface_id', query_surface_ctx.query_surface_id,
  'asset_object_id', query_surface_ctx.asset_object_id,
  'environment_id', query_surface_ctx.environment_id,
  'query_template_id', query_template_ctx.query_template_id,
  'approval_ticket_id', approval_ticket_row.approval_ticket_id,
  'case_id', case_row.case_id,
  'decision_id', decision_row.decision_id,
  'payment_intent_id', payment_intent_row.payment_intent_id,
  'settlement_id', settlement_row.settlement_id,
  'order_id', order_row.order_id,
  'order_amount', '${order_amount}',
  'currency_code', 'SGD'
)::text
FROM product_ctx, query_surface_ctx, query_template_ctx, order_row, approval_ticket_row, case_row, decision_row, payment_intent_row, settlement_row;
SQL
)"

  [[ -n "${fixture_json}" ]] || fail "prepare did not return fixture payload; ensure seed-demo.sh loaded formal S5 QRY_LITE demo order ${demo_order_id}"
  printf '%s\n' "${fixture_json}"
}

delete_minio_object() {
  local bucket_name="$1"
  local object_key="$2"
  local access_key secret_key host_value

  [[ -n "${bucket_name}" && -n "${object_key}" ]] || return 0
  require_cmd docker
  access_key="${MINIO_ACCESS_KEY:-${MINIO_ROOT_USER:-datab}}"
  secret_key="${MINIO_SECRET_KEY:-${MINIO_ROOT_PASSWORD:-datab_local_pass}}"
  host_value="${MINIO_ENDPOINT/http:\/\//http://${access_key}:${secret_key}@}"
  docker run --rm --network host \
    -e "MC_HOST_local=${host_value}" \
    "${MINIO_MC_IMAGE}" rm -f "local/${bucket_name}/${object_key}" >/dev/null 2>&1 || true
}

cleanup_fixture() {
  local fixture_file portal_artifact_file order_id approval_ticket_id case_id decision_id payment_intent_id settlement_id refund_id result_object_id bucket_name object_key

  [[ $# -ge 2 && "$1" == "--fixture" ]] || fail "cleanup requires --fixture <path>"
  fixture_file="$2"
  shift 2
  portal_artifact_file=""
  if [[ $# -ge 2 && "$1" == "--portal-artifact" ]]; then
    portal_artifact_file="$2"
  fi

  [[ -f "${fixture_file}" ]] || fail "missing fixture file ${fixture_file}"
  order_id="$("${JQ_BIN}" -r '.order_id' "${fixture_file}")"
  approval_ticket_id="$("${JQ_BIN}" -r '.approval_ticket_id' "${fixture_file}")"
  case_id="$("${JQ_BIN}" -r '.case_id' "${fixture_file}")"
  decision_id="$("${JQ_BIN}" -r '.decision_id' "${fixture_file}")"
  payment_intent_id="$("${JQ_BIN}" -r '.payment_intent_id' "${fixture_file}")"
  settlement_id="$("${JQ_BIN}" -r '.settlement_id' "${fixture_file}")"
  refund_id=""
  result_object_id=""
  bucket_name=""
  object_key=""

  if [[ -n "${portal_artifact_file}" && -f "${portal_artifact_file}" ]]; then
    refund_id="$("${JQ_BIN}" -r '.refund_response.refund_id // empty' "${portal_artifact_file}")"
    result_object_id="$("${JQ_BIN}" -r '.run_response.result_object_id // empty' "${portal_artifact_file}")"
    bucket_name="$("${JQ_BIN}" -r '.run_response.bucket_name // empty' "${portal_artifact_file}")"
    object_key="$("${JQ_BIN}" -r '.run_response.object_key // empty' "${portal_artifact_file}")"
  fi

  [[ -n "${order_id}" && "${order_id}" != "null" ]] || fail "fixture file missing order_id"

  "${PSQL_BIN}" "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<SQL >/dev/null
WITH target_outbox AS (
  SELECT outbox_event_id
  FROM ops.outbox_event
  WHERE ordering_key = '${order_id}'
     OR partition_key = '${order_id}'
     OR aggregate_id = '${order_id}'
     OR aggregate_id = '${case_id}'
     $(if [[ -n "${refund_id}" ]]; then printf "OR aggregate_id = '%s'" "${refund_id}"; fi)
),
cleanup_dead_letter AS (
  DELETE FROM ops.dead_letter_event
  WHERE outbox_event_id IN (SELECT outbox_event_id FROM target_outbox)
),
cleanup_publish_attempt AS (
  DELETE FROM ops.outbox_publish_attempt
  WHERE outbox_event_id IN (SELECT outbox_event_id FROM target_outbox)
),
cleanup_outbox AS (
  DELETE FROM ops.outbox_event
  WHERE outbox_event_id IN (SELECT outbox_event_id FROM target_outbox)
)
SELECT 1;
DELETE FROM support.decision_record WHERE decision_id = '${decision_id}'::uuid;
DELETE FROM support.dispute_case WHERE case_id = '${case_id}'::uuid;
DELETE FROM billing.refund_record WHERE order_id = '${order_id}'::uuid;
DELETE FROM billing.billing_event WHERE order_id = '${order_id}'::uuid;
DELETE FROM billing.settlement_record WHERE settlement_id = '${settlement_id}'::uuid;
DELETE FROM payment.payment_intent WHERE payment_intent_id = '${payment_intent_id}'::uuid;
DELETE FROM ops.approval_ticket WHERE approval_ticket_id = '${approval_ticket_id}'::uuid;
DELETE FROM delivery.query_execution_run WHERE order_id = '${order_id}'::uuid;
$(if [[ -n "${result_object_id}" ]]; then
  cat <<EOSQL
DELETE FROM delivery.storage_object WHERE object_id = '${result_object_id}'::uuid;
EOSQL
else
  cat <<EOSQL
DELETE FROM delivery.storage_object WHERE object_uri LIKE 's3://%/query-runs/${order_id}/%';
EOSQL
fi)
DELETE FROM delivery.delivery_record WHERE order_id = '${order_id}'::uuid;
DELETE FROM delivery.template_query_grant WHERE order_id = '${order_id}'::uuid;
DELETE FROM contract.digital_contract WHERE order_id = '${order_id}'::uuid;
DELETE FROM trade.order_main WHERE order_id = '${order_id}'::uuid;
SQL

  delete_minio_object "${bucket_name}" "${object_key}"
  echo "[ok] TEST-026 live fixture cleaned: ${order_id}"
}

main() {
  require_cmd "${PSQL_BIN}"
  require_cmd "${JQ_BIN}"
  load_env

  case "${1:-}" in
    prepare)
      shift
      prepare_fixture "$@"
      ;;
    cleanup)
      shift
      cleanup_fixture "$@"
      ;;
    *)
      usage
      fail "expected 'prepare' or 'cleanup'"
      ;;
  esac
}

main "$@"
