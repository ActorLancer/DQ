#!/usr/bin/env bash
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${ORDER_ID:?ORDER_ID is required}"
: "${SELLER_ORG_ID:?SELLER_ORG_ID is required}"
: "${BUYER_ORG_ID:?BUYER_ORG_ID is required}"
REQUEST_ID="${REQUEST_ID:-dlv026-s2-$(date +%s%N)}"

curl --fail-with-body -sS -X POST "${BASE_URL}/api/v1/orders/${ORDER_ID}/deliver" \
  -H 'content-type: application/json' \
  -H 'x-role: seller_operator' \
  -H "x-tenant-id: ${SELLER_ORG_ID}" \
  -H "x-request-id: ${REQUEST_ID}" \
  --data @- <<JSON
{
  "branch": "file",
  "object_uri": "s3://delivery-objects/dlv026/${ORDER_ID}/payload.enc",
  "content_type": "application/octet-stream",
  "size_bytes": 2048,
  "content_hash": "sha256:dlv026:s2:${ORDER_ID}",
  "encryption_algo": "AES-GCM",
  "key_cipher": "cipher-${ORDER_ID}",
  "key_control_mode": "seller_managed",
  "unwrap_policy_json": {"kms": "local-mock", "buyer_org_id": "${BUYER_ORG_ID}"},
  "key_version": "v1",
  "expire_at": "2027-01-01T00:00:00Z",
  "download_limit": 3,
  "delivery_commit_hash": "${REQUEST_ID}-commit",
  "receipt_hash": "${REQUEST_ID}-receipt"
}
JSON
