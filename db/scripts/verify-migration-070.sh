#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-55432}"
DB_NAME="${DB_NAME:-luna_data_trading}"
DB_USER="${DB_USER:-luna}"
DB_PASSWORD="${DB_PASSWORD:-5686}"

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

check_count_ge "SELECT COUNT(*) FROM authz.permission_definition;" 220 "authz.permission_definition"
check_count_ge "SELECT COUNT(*) FROM authz.role_permission;" 240 "authz.role_permission"

echo "[ok] migration 070 seed role-permission baseline verified"
