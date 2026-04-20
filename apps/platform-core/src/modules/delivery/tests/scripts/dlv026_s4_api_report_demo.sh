#!/usr/bin/env bash
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${SELLER_ORG_ID:?SELLER_ORG_ID is required}"
: "${ORDER_ID:?ORDER_ID is required}"
: "${STORAGE_NAMESPACE_ID:?STORAGE_NAMESPACE_ID is required}"
REQUEST_ID="${REQUEST_ID:-dlv026-s4-$(date +%s%N)}"

if [[ -n "${API_ORDER_ID:-}" && -n "${BUYER_ORG_ID:-}" && -n "${ASSET_OBJECT_ID:-}" ]]; then
  curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${API_ORDER_ID}/deliver" \
    -H 'content-type: application/json' \
    -H 'x-role: tenant_developer' \
    -H "x-tenant-id: ${BUYER_ORG_ID}" \
    -H "x-request-id: ${REQUEST_ID}-api" \
    --data @- <<JSON
{
  "branch": "api",
  "asset_object_id": "${ASSET_OBJECT_ID}",
  "app_name": "dlv026-s4-api-app",
  "quota_json": {"billing_mode": "subscription", "period": "monthly", "included_calls": 500},
  "rate_limit_json": {"requests_per_minute": 30, "burst": 5, "concurrency": 2},
  "upstream_mode": "platform_proxy",
  "expire_at": "2027-01-01T00:00:00Z",
  "delivery_commit_hash": "${REQUEST_ID}-api-commit",
  "receipt_hash": "${REQUEST_ID}-api-receipt"
}
JSON
fi

curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/deliver" \
  -H 'content-type: application/json' \
  -H 'x-role: seller_operator' \
  -H "x-tenant-id: ${SELLER_ORG_ID}" \
  -H "x-request-id: ${REQUEST_ID}-report" \
  --data @- <<JSON
{
  "branch": "report",
  "object_uri": "s3://report-results/dlv026/${ORDER_ID}/report.pdf",
  "content_type": "application/pdf",
  "size_bytes": 4096,
  "content_hash": "sha256:dlv026:s4:${ORDER_ID}",
  "report_type": "pdf_report",
  "storage_namespace_id": "${STORAGE_NAMESPACE_ID}",
  "delivery_commit_hash": "${REQUEST_ID}-report-commit",
  "receipt_hash": "${REQUEST_ID}-report-receipt",
  "metadata": {"title": "DLV026 S4 retail report", "template_code": "REPORT_V1"}
}
JSON
