#!/usr/bin/env bash
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${ORDER_ID:?ORDER_ID is required}"
: "${SELLER_ORG_ID:?SELLER_ORG_ID is required}"
: "${QUERY_SURFACE_ID:?QUERY_SURFACE_ID is required}"
: "${SEAT_USER_ID:?SEAT_USER_ID is required}"
REQUEST_ID="${REQUEST_ID:-dlv026-s3-$(date +%s%N)}"

curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/sandbox-workspaces" \
  -H 'content-type: application/json' \
  -H 'x-role: seller_operator' \
  -H "x-tenant-id: ${SELLER_ORG_ID}" \
  -H "x-request-id: ${REQUEST_ID}" \
  --data @- <<JSON
{
  "query_surface_id": "${QUERY_SURFACE_ID}",
  "workspace_name": "dlv026-s3-workspace-${ORDER_ID}",
  "seat_user_id": "${SEAT_USER_ID}",
  "expire_at": "2027-01-01T00:00:00Z",
  "export_policy_json": {"allow_export": false, "allowed_formats": ["json"], "max_exports": 0, "network_access": "deny"},
  "clean_room_mode": "lite",
  "data_residency_mode": "seller_self_hosted"
}
JSON
