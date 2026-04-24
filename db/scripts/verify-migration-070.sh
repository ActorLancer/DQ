#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q -tA)

roles=(
  "platform_admin"
  "platform_reviewer"
  "platform_risk_settlement"
  "platform_audit_security"
  "tenant_admin"
  "seller_operator"
  "buyer_operator"
  "tenant_developer"
  "tenant_audit_readonly"
  "tenant_app_identity"
  "platform_service_identity"
  "regulator_readonly"
)

permissions=(
  "ops.trade_monitor.read"
  "ops.external_fact.read"
  "risk.fairness_incident.read"
  "risk.fairness_incident.handle"
  "delivery.template_query.use"
)

route_checks=(
  "notification.dispatch_request|notification.requested|dtp.notification.dispatch"
  "audit.anchor_batch|audit.anchor_requested|dtp.audit.anchor"
  "chain.chain_anchor|fabric.proof_submit_requested|dtp.fabric.requests"
  "recommend.behavior_event|recommend.behavior_recorded|dtp.recommend.behavior"
)

retired_triggers=(
  "trade.order_main|trg_order_outbox"
  "catalog.product|trg_product_outbox"
  "support.dispute_case|trg_dispute_outbox"
  "payment.payment_intent|trg_payment_intent_outbox"
  "payment.payout_instruction|trg_payout_instruction_outbox"
  "recommend.behavior_event|trg_recommend_behavior_event_outbox"
)

check_count_ge() {
  local sql="$1"
  local min="$2"
  local label="$3"
  local count
  count="$("${PSQL[@]}" -c "$sql")"
  if [[ "$count" -lt "$min" ]]; then
    echo "[fail] ${label} count too small: ${count} (< ${min})" >&2
    exit 1
  fi
}

for role in "${roles[@]}"; do
  exists="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM authz.role_definition WHERE role_key='${role}';")"
  if [[ "$exists" -lt 1 ]]; then
    echo "[fail] role missing: ${role}" >&2
    exit 1
  fi
done

for permission in "${permissions[@]}"; do
  exists="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM authz.permission_definition WHERE permission_code='${permission}';")"
  if [[ "$exists" -lt 1 ]]; then
    echo "[fail] permission missing: ${permission}" >&2
    exit 1
  fi
done

for route_check in "${route_checks[@]}"; do
  IFS='|' read -r aggregate_type event_type target_topic <<<"${route_check}"
  exists="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM ops.event_route_policy WHERE aggregate_type='${aggregate_type}' AND event_type='${event_type}' AND target_topic='${target_topic}' AND status='active';")"
  if [[ "$exists" -lt 1 ]]; then
    echo "[fail] route policy missing: ${aggregate_type}/${event_type} -> ${target_topic}" >&2
    exit 1
  fi
done

for retired_trigger in "${retired_triggers[@]}"; do
  IFS='|' read -r schema_table trigger_name <<<"${retired_trigger}"
  exists="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname || '.' || c.relname='${schema_table}' AND t.tgname='${trigger_name}' AND NOT t.tgisinternal;")"
  if [[ "$exists" -ne 0 ]]; then
    echo "[fail] retired trigger still present: ${schema_table}.${trigger_name}" >&2
    exit 1
  fi
done

function_def="$("${PSQL[@]}" -c "SELECT pg_get_functiondef('common.tg_write_outbox()'::regprocedure);")"
if [[ "${function_def}" != *"retired"* ]]; then
  echo "[fail] common.tg_write_outbox() is not in retired state" >&2
  exit 1
fi

check_count_ge "SELECT COUNT(*) FROM authz.permission_definition;" 220 "authz.permission_definition"
check_count_ge "SELECT COUNT(*) FROM authz.role_permission;" 240 "authz.role_permission"

echo "[ok] migration 070/072/073/074 authz seed, route-policy, and retired-trigger baseline verified"
