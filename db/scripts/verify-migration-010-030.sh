#!/usr/bin/env bash
set -euo pipefail

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"

export PGPASSWORD="${DB_PASSWORD}"
PSQL=(psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -X -q -tA)

required_tables=(
  core.organization
  core.department
  core.user_account
  core.application
  core.connector
  core.execution_environment
  iam.invitation
  iam.trusted_device
  iam.user_session
  authz.role_definition
  authz.permission_definition
  authz.role_permission
  authz.subject_role_binding
  catalog.data_asset
  catalog.asset_version
  catalog.product
  catalog.product_sku
  contract.template_definition
  contract.template_binding
  contract.usage_policy
  review.review_task
  review.review_step
  ops.approval_ticket
  ops.approval_step
  trade.order_main
  trade.order_line
  trade.order_status_history
  trade.authorization_grant
  delivery.delivery_record
  delivery.delivery_ticket
  delivery.delivery_receipt
  delivery.api_credential
  delivery.api_usage_log
  delivery.sandbox_workspace
  delivery.sandbox_session
  delivery.report_artifact
)

required_indexes=(
  core.idx_user_account_org_id
  iam.idx_user_session_user_status
  catalog.idx_product_status_type
  review.idx_review_task_ref
  trade.idx_order_main_status_created_at
  delivery.idx_delivery_record_order_id
)

required_triggers=(
  core.trg_organization_updated_at
  iam.trg_user_session_updated_at
  catalog.trg_product_updated_at
  review.trg_review_task_updated_at
  trade.trg_order_status_history
  delivery.trg_delivery_record_updated_at
)

required_constraints=(
  trade.fk_order_contract
  contract.fk_digital_contract_order
  delivery.fk_delivery_record_envelope
)

check_table() {
  local fq_table="$1"
  local schema="${fq_table%%.*}"
  local table="${fq_table#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema='${schema}' AND table_name='${table}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] table missing: ${fq_table}" >&2
    exit 1
  fi
}

check_index() {
  local fq_index="$1"
  local schema="${fq_index%%.*}"
  local index="${fq_index#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname='${schema}' AND indexname='${index}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] index missing: ${fq_index}" >&2
    exit 1
  fi
}

check_trigger() {
  local fq_trigger="$1"
  local schema="${fq_trigger%%.*}"
  local trigger="${fq_trigger#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.triggers WHERE trigger_schema='${schema}' AND trigger_name='${trigger}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] trigger missing: ${fq_trigger}" >&2
    exit 1
  fi
}

check_constraint() {
  local fq_constraint="$1"
  local schema="${fq_constraint%%.*}"
  local constraint="${fq_constraint#*.}"
  local exists
  exists="$("${PSQL[@]}" -c "SELECT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE constraint_schema='${schema}' AND constraint_name='${constraint}');")"
  if [[ "${exists}" != "t" ]]; then
    echo "[fail] constraint missing: ${fq_constraint}" >&2
    exit 1
  fi
}

for table in "${required_tables[@]}"; do
  check_table "${table}"
done

for index in "${required_indexes[@]}"; do
  check_index "${index}"
done

for trigger in "${required_triggers[@]}"; do
  check_trigger "${trigger}"
done

for constraint in "${required_constraints[@]}"; do
  check_constraint "${constraint}"
done

echo "[ok] migrations 010/020/025/030 baseline verified"
