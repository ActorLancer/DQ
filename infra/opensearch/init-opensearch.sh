#!/usr/bin/env bash
set -euo pipefail

OPENSEARCH_ENDPOINT="${OPENSEARCH_ENDPOINT:-http://127.0.0.1:9200}"
INDEX_ALIAS_CATALOG_PRODUCTS="${INDEX_ALIAS_CATALOG_PRODUCTS:-catalog_products_v1}"
INDEX_ALIAS_SELLER_PROFILES="${INDEX_ALIAS_SELLER_PROFILES:-seller_profiles_v1}"
INDEX_ALIAS_SEARCH_SYNC_JOBS="${INDEX_ALIAS_SEARCH_SYNC_JOBS:-search_sync_jobs_v1}"

template_file="$(dirname "$0")/index-template-catalog.json"

curl_json() {
  local method="$1"
  local path="$2"
  local data_file="${3:-}"
  if [[ -n "${data_file}" ]]; then
    curl -sS -X "${method}" \
      -H "Content-Type: application/json" \
      "${OPENSEARCH_ENDPOINT}${path}" \
      --data-binary "@${data_file}"
  else
    curl -sS -X "${method}" "${OPENSEARCH_ENDPOINT}${path}"
  fi
}

wait_for_os() {
  for _ in $(seq 1 60); do
    if curl -fsS "${OPENSEARCH_ENDPOINT}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "[fail] opensearch endpoint not ready: ${OPENSEARCH_ENDPOINT}" >&2
  exit 1
}

create_index_with_alias() {
  local alias_name="$1"
  local index_name="${alias_name}_000001"
  curl -sS -X PUT "${OPENSEARCH_ENDPOINT}/${index_name}" \
    -H "Content-Type: application/json" \
    -d "{\"aliases\":{\"${alias_name}\":{}}}" >/dev/null
  echo "[ok] index+alias ready: ${index_name} -> ${alias_name}"
}

index_demo_doc() {
  local alias_name="$1"
  local id="$2"
  local payload="$3"
  curl -sS -X POST "${OPENSEARCH_ENDPOINT}/${alias_name}/_doc/${id}" \
    -H "Content-Type: application/json" \
    -d "${payload}" >/dev/null
}

wait_for_os

curl_json PUT "/_index_template/datab_catalog_v1_template" "${template_file}" >/dev/null
echo "[ok] index template upserted: datab_catalog_v1_template"

create_index_with_alias "${INDEX_ALIAS_CATALOG_PRODUCTS}"
create_index_with_alias "${INDEX_ALIAS_SELLER_PROFILES}"
create_index_with_alias "${INDEX_ALIAS_SEARCH_SYNC_JOBS}"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
index_demo_doc "${INDEX_ALIAS_CATALOG_PRODUCTS}" "demo-product-001" "{\"id\":\"demo-product-001\",\"tenant_id\":\"t-demo\",\"seller_id\":\"s-demo\",\"name\":\"Demo Product\",\"description\":\"Demo product for local initialization\",\"sku_code\":\"FILE_STD\",\"status\":\"listed\",\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
index_demo_doc "${INDEX_ALIAS_SELLER_PROFILES}" "demo-seller-001" "{\"id\":\"demo-seller-001\",\"tenant_id\":\"t-demo\",\"name\":\"Demo Seller\",\"description\":\"Seller profile demo\",\"status\":\"active\",\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
index_demo_doc "${INDEX_ALIAS_SEARCH_SYNC_JOBS}" "demo-sync-job-001" "{\"id\":\"demo-sync-job-001\",\"tenant_id\":\"t-demo\",\"status\":\"done\",\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
echo "[ok] demo documents indexed"

curl -sS -X POST "${OPENSEARCH_ENDPOINT}/_refresh" >/dev/null
echo "[done] opensearch initialization complete"
