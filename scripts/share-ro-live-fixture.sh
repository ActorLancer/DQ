#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ENV_FILE="${ENV_FILE:-infra/docker/.env.local}"
PSQL_BIN="${PSQL_BIN:-psql}"
JQ_BIN="${JQ_BIN:-jq}"

usage() {
  cat <<'EOF'
Usage:
  ENV_FILE=infra/docker/.env.local ./scripts/share-ro-live-fixture.sh prepare > target/test-artifacts/share-ro-e2e/live-fixture.json
  ENV_FILE=infra/docker/.env.local ./scripts/share-ro-live-fixture.sh cleanup --fixture target/test-artifacts/share-ro-e2e/live-fixture.json
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
  local product_id sku_id buyer_org_id seller_org_id demo_order_id order_amount suffix fixture_json

  product_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S3" and .scenario_role == "supplementary") | .product_id')"
  sku_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S3" and .scenario_role == "supplementary") | .sku_id')"
  demo_order_id="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S3" and .scenario_role == "supplementary") | .order_blueprint_id')"
  order_amount="$(read_demo_value '.order_blueprints[] | select(.scenario_code == "S3" and .scenario_role == "supplementary") | .order_amount')"
  buyer_org_id="$(read_demo_subject_value '.users[] | select(.persona == "buyer_operator") | .org_id' | head -n 1)"
  seller_org_id="$(read_demo_subject_value '.users[] | select(.persona == "seller_operator") | .org_id' | head -n 1)"

  [[ -n "${product_id}" && "${product_id}" != "null" ]] || fail "missing S3 SHARE_RO product_id from fixtures/demo/orders.json"
  [[ -n "${sku_id}" && "${sku_id}" != "null" ]] || fail "missing S3 SHARE_RO sku_id from fixtures/demo/orders.json"
  [[ -n "${demo_order_id}" && "${demo_order_id}" != "null" ]] || fail "missing S3 SHARE_RO order_blueprint_id from fixtures/demo/orders.json"
  [[ -n "${order_amount}" && "${order_amount}" != "null" ]] || fail "missing S3 SHARE_RO order_amount from fixtures/demo/orders.json"
  [[ -n "${buyer_org_id}" && "${buyer_org_id}" != "null" ]] || fail "missing buyer org from fixtures/demo/subjects.json"
  [[ -n "${seller_org_id}" && "${seller_org_id}" != "null" ]] || fail "missing seller org from fixtures/demo/subjects.json"

  suffix="test025-$(date +%s)-$$"
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
share_object_row AS (
  INSERT INTO catalog.asset_object_binding (
    asset_version_id,
    object_kind,
    object_name,
    object_locator,
    share_protocol,
    schema_json,
    output_schema_json,
    freshness_json,
    access_constraints,
    metadata
  )
  SELECT
    product_ctx.asset_version_id::uuid,
    'share_object',
    'test025-share-object-${suffix}',
    'share://seller/test025/${suffix}/dataset',
    'share_grant',
    '{}'::jsonb,
    '{}'::jsonb,
    '{}'::jsonb,
    '{}'::jsonb,
    jsonb_build_object(
      'task_id', 'TEST-025',
      'fixture_suffix', '${suffix}',
      'seed_source', 'scripts/share-ro-live-fixture.sh'
    )
  FROM product_ctx
  RETURNING asset_object_id::text
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
    idempotency_key
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
    'CNY',
    product_ctx.price_snapshot_json,
    product_ctx.delivery_route_snapshot,
    product_ctx.trust_boundary_snapshot,
    'test025-share-live-${suffix}'
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
    'sha256:test025:' || order_row.order_id,
    'signed',
    now(),
    jsonb_build_object('task_id', 'TEST-025', 'fixture_suffix', '${suffix}')
  FROM order_row
)
SELECT json_build_object(
  'fixture_id', 'test025-share-live-fixture',
  'task_id', 'TEST-025',
  'suffix', '${suffix}',
  'product_id', product_ctx.product_id,
  'sku_id', '${sku_id}',
  'buyer_org_id', '${buyer_org_id}',
  'seller_org_id', '${seller_org_id}',
  'asset_object_id', share_object_row.asset_object_id,
  'order_id', order_row.order_id
)::text
FROM product_ctx, share_object_row, order_row;
SQL
)"

  [[ -n "${fixture_json}" ]] || fail "prepare did not return fixture payload; ensure seed-demo.sh loaded formal S3 SHARE_RO demo order ${demo_order_id}"
  printf '%s\n' "${fixture_json}"
}

cleanup_fixture() {
  local fixture_file order_id asset_object_id

  [[ $# -ge 2 && "$1" == "--fixture" ]] || fail "cleanup requires --fixture <path>"
  fixture_file="$2"
  [[ -f "${fixture_file}" ]] || fail "missing fixture file ${fixture_file}"
  order_id="$("${JQ_BIN}" -r '.order_id' "${fixture_file}")"
  asset_object_id="$("${JQ_BIN}" -r '.asset_object_id // empty' "${fixture_file}")"
  [[ -n "${order_id}" && "${order_id}" != "null" ]] || fail "fixture file missing order_id"

  "${PSQL_BIN}" "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<SQL >/dev/null
WITH target_outbox AS (
  SELECT outbox_event_id
  FROM ops.outbox_event
  WHERE ordering_key = '${order_id}'
     OR aggregate_id = '${order_id}'
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
DELETE FROM delivery.data_share_grant WHERE order_id = '${order_id}'::uuid;
DELETE FROM delivery.delivery_record WHERE order_id = '${order_id}'::uuid;
DELETE FROM billing.billing_event WHERE order_id = '${order_id}'::uuid;
DELETE FROM billing.settlement_record WHERE order_id = '${order_id}'::uuid;
DELETE FROM contract.digital_contract WHERE order_id = '${order_id}'::uuid;
DELETE FROM trade.order_main WHERE order_id = '${order_id}'::uuid;
$(if [[ -n "${asset_object_id}" ]]; then
  cat <<EOSQL
DELETE FROM catalog.asset_object_binding
WHERE asset_object_id = '${asset_object_id}'::uuid
  AND metadata ->> 'task_id' = 'TEST-025';
EOSQL
fi)
SQL

  echo "[ok] TEST-025 live fixture cleaned: ${order_id}"
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
