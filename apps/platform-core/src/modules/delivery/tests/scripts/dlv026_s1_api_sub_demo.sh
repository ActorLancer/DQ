#!/usr/bin/env bash
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${ORDER_ID:?ORDER_ID is required}"
: "${BUYER_ORG_ID:?BUYER_ORG_ID is required}"
: "${ASSET_OBJECT_ID:?ASSET_OBJECT_ID is required}"
REQUEST_ID="${REQUEST_ID:-dlv026-s1-$(date +%s%N)}"

curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/deliver" \
  -H 'content-type: application/json' \
  -H 'x-role: tenant_developer' \
  -H "x-tenant-id: ${BUYER_ORG_ID}" \
  -H "x-request-id: ${REQUEST_ID}" \
  --data @- <<JSON
{
  "branch": "api",
  "asset_object_id": "${ASSET_OBJECT_ID}",
  "app_name": "dlv026-s1-api-app",
  "quota_json": {"billing_mode": "subscription", "period": "monthly", "included_calls": 1000},
  "rate_limit_json": {"requests_per_minute": 60, "burst": 10, "concurrency": 3},
  "upstream_mode": "platform_proxy",
  "expire_at": "2027-01-01T00:00:00Z",
  "delivery_commit_hash": "${REQUEST_ID}-commit",
  "receipt_hash": "${REQUEST_ID}-receipt"
}
JSON
