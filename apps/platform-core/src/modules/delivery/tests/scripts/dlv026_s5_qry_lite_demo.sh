#!/usr/bin/env bash
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${ORDER_ID:?ORDER_ID is required}"
: "${SELLER_ORG_ID:?SELLER_ORG_ID is required}"
: "${BUYER_ORG_ID:?BUYER_ORG_ID is required}"
: "${BUYER_USER_ID:?BUYER_USER_ID is required}"
: "${QUERY_SURFACE_ID:?QUERY_SURFACE_ID is required}"
: "${QUERY_TEMPLATE_ID:?QUERY_TEMPLATE_ID is required}"
: "${APPROVAL_TICKET_ID:?APPROVAL_TICKET_ID is required}"
REQUEST_ID="${REQUEST_ID:-dlv026-s5-$(date +%s%N)}"

grant_response="$(curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/template-grants" \
  -H 'content-type: application/json' \
  -H 'x-role: seller_operator' \
  -H "x-tenant-id: ${SELLER_ORG_ID}" \
  -H "x-request-id: ${REQUEST_ID}-grant" \
  --data @- <<JSON
{
  "query_surface_id": "${QUERY_SURFACE_ID}",
  "allowed_template_ids": ["${QUERY_TEMPLATE_ID}"],
  "execution_rule_snapshot": {"entrypoint": "template_query_lite", "grant_source": "dlv026-demo"},
  "output_boundary_json": {"allowed_formats": ["json"], "max_rows": 5, "max_cells": 15},
  "run_quota_json": {"max_runs": 5, "daily_limit": 2, "monthly_limit": 8}
}
JSON
)"

template_query_grant_id="$(printf '%s' "${grant_response}" | jq -r '.data.data.template_query_grant_id')"

curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/template-runs" \
  -H 'content-type: application/json' \
  -H 'x-role: buyer_operator' \
  -H "x-tenant-id: ${BUYER_ORG_ID}" \
  -H "x-user-id: ${BUYER_USER_ID}" \
  -H "x-request-id: ${REQUEST_ID}-run" \
  --data @- <<JSON
{
  "template_query_grant_id": "${template_query_grant_id}",
  "query_template_id": "${QUERY_TEMPLATE_ID}",
  "requester_user_id": "${BUYER_USER_ID}",
  "request_payload_json": {"start_date": "2026-01-01", "limit": 2},
  "output_boundary_json": {"selected_format": "json", "allowed_formats": ["json"], "max_rows": 2, "max_cells": 6},
  "approval_ticket_id": "${APPROVAL_TICKET_ID}",
  "execution_metadata_json": {"entrypoint": "dlv026-demo"}
}
JSON
