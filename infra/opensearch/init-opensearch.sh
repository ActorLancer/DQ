#!/usr/bin/env bash
set -euo pipefail

OPENSEARCH_ENDPOINT="${OPENSEARCH_ENDPOINT:-http://127.0.0.1:9200}"
INDEX_ALIAS_PRODUCT_SEARCH_READ="${INDEX_ALIAS_PRODUCT_SEARCH_READ:-product_search_read}"
INDEX_ALIAS_PRODUCT_SEARCH_WRITE="${INDEX_ALIAS_PRODUCT_SEARCH_WRITE:-product_search_write}"
INDEX_ALIAS_SELLER_SEARCH_READ="${INDEX_ALIAS_SELLER_SEARCH_READ:-seller_search_read}"
INDEX_ALIAS_SELLER_SEARCH_WRITE="${INDEX_ALIAS_SELLER_SEARCH_WRITE:-seller_search_write}"
INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP="${INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP:-product_search_v1_bootstrap}"
INDEX_NAME_SELLER_SEARCH_BOOTSTRAP="${INDEX_NAME_SELLER_SEARCH_BOOTSTRAP:-seller_search_v1_bootstrap}"
INDEX_NAME_SEARCH_SYNC_JOBS="${INDEX_NAME_SEARCH_SYNC_JOBS:-search_sync_jobs_v1}"

template_file="$(dirname "$0")/index-template-catalog.json"

curl_json() {
  local method="$1"
  local path="$2"
  local data_file="${3:-}"
  if [[ -n "${data_file}" ]]; then
    curl -fsS -X "${method}" \
      -H "Content-Type: application/json" \
      "${OPENSEARCH_ENDPOINT}${path}" \
      --data-binary "@${data_file}"
  else
    curl -fsS -X "${method}" "${OPENSEARCH_ENDPOINT}${path}"
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

create_index_with_aliases() {
  local index_name="$1"
  local read_alias="$2"
  local write_alias="$3"
  local aliases_json
  aliases_json="{\"${read_alias}\":{},\"${write_alias}\":{\"is_write_index\":true}}"
  curl -fsS -X PUT "${OPENSEARCH_ENDPOINT}/${index_name}" \
    -H "Content-Type: application/json" \
    -d "{\"aliases\":${aliases_json}}" >/dev/null
  echo "[ok] index+aliases ready: ${index_name}"
}

rebind_aliases() {
  local index_name="$1"
  local read_alias="$2"
  local write_alias="$3"
  curl -fsS -X POST "${OPENSEARCH_ENDPOINT}/_aliases" \
    -H "Content-Type: application/json" \
    -d "{
      \"actions\": [
        {\"remove\": {\"index\": \"*\", \"alias\": \"${read_alias}\"}},
        {\"remove\": {\"index\": \"*\", \"alias\": \"${write_alias}\"}},
        {\"add\": {\"index\": \"${index_name}\", \"alias\": \"${read_alias}\"}},
        {\"add\": {\"index\": \"${index_name}\", \"alias\": \"${write_alias}\", \"is_write_index\": true}}
      ]
    }" >/dev/null
  echo "[ok] aliases rebound: ${index_name} -> ${read_alias}/${write_alias}"
}

create_index() {
  local index_name="$1"
  curl -fsS -X PUT "${OPENSEARCH_ENDPOINT}/${index_name}" \
    -H "Content-Type: application/json" \
    -d "{}" >/dev/null
  echo "[ok] index ready: ${index_name}"
}

delete_index_if_exists() {
  local index_name="$1"
  local status
  status="$(curl -sS -o /dev/null -w "%{http_code}" -X DELETE "${OPENSEARCH_ENDPOINT}/${index_name}")"
  case "${status}" in
    200|202)
      echo "[ok] deleted existing index: ${index_name}"
      ;;
    404)
      ;;
    *)
      echo "[fail] unexpected delete status for ${index_name}: ${status}" >&2
      exit 1
      ;;
  esac
}

index_demo_doc() {
  local alias_name="$1"
  local id="$2"
  local payload="$3"
  curl -fsS -X PUT "${OPENSEARCH_ENDPOINT}/${alias_name}/_doc/${id}" \
    -H "Content-Type: application/json" \
    -d "${payload}" >/dev/null
}

sync_alias_binding_authority() {
  local database_url="${DATABASE_URL:-postgres://datab:datab_local_pass@127.0.0.1:5432/datab}"
  if ! command -v psql >/dev/null 2>&1; then
    echo "[warn] psql not found; skipped search.index_alias_binding sync" >&2
    return 0
  fi
  psql "${database_url}" -v ON_ERROR_STOP=1 <<SQL >/dev/null
UPDATE search.index_alias_binding
SET read_alias = CASE entity_scope
      WHEN 'product' THEN '${INDEX_ALIAS_PRODUCT_SEARCH_READ}'
      WHEN 'seller' THEN '${INDEX_ALIAS_SELLER_SEARCH_READ}'
      ELSE read_alias
    END,
    write_alias = CASE entity_scope
      WHEN 'product' THEN '${INDEX_ALIAS_PRODUCT_SEARCH_WRITE}'
      WHEN 'seller' THEN '${INDEX_ALIAS_SELLER_SEARCH_WRITE}'
      ELSE write_alias
    END,
    active_index_name = CASE entity_scope
      WHEN 'product' THEN '${INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP}'
      WHEN 'seller' THEN '${INDEX_NAME_SELLER_SEARCH_BOOTSTRAP}'
      ELSE active_index_name
    END,
    updated_at = now()
WHERE backend_type = 'opensearch'
  AND entity_scope IN ('product', 'seller');
SQL
  echo "[ok] search.index_alias_binding authority synced"
}

wait_for_os

curl_json PUT "/_index_template/datab_catalog_v1_template" "${template_file}" >/dev/null
echo "[ok] index template upserted: datab_catalog_v1_template"

delete_index_if_exists "catalog_products_v1_000001"
delete_index_if_exists "seller_profiles_v1_000001"
delete_index_if_exists "search_sync_jobs_v1_000001"
delete_index_if_exists "${INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP}"
delete_index_if_exists "${INDEX_NAME_SELLER_SEARCH_BOOTSTRAP}"
delete_index_if_exists "${INDEX_NAME_SEARCH_SYNC_JOBS}"

create_index_with_aliases \
  "${INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP}" \
  "${INDEX_ALIAS_PRODUCT_SEARCH_READ}" \
  "${INDEX_ALIAS_PRODUCT_SEARCH_WRITE}"
rebind_aliases \
  "${INDEX_NAME_PRODUCT_SEARCH_BOOTSTRAP}" \
  "${INDEX_ALIAS_PRODUCT_SEARCH_READ}" \
  "${INDEX_ALIAS_PRODUCT_SEARCH_WRITE}"
create_index_with_aliases \
  "${INDEX_NAME_SELLER_SEARCH_BOOTSTRAP}" \
  "${INDEX_ALIAS_SELLER_SEARCH_READ}" \
  "${INDEX_ALIAS_SELLER_SEARCH_WRITE}"
rebind_aliases \
  "${INDEX_NAME_SELLER_SEARCH_BOOTSTRAP}" \
  "${INDEX_ALIAS_SELLER_SEARCH_READ}" \
  "${INDEX_ALIAS_SELLER_SEARCH_WRITE}"
create_index "${INDEX_NAME_SEARCH_SYNC_JOBS}"

echo "[info] alias authority defaults: ${INDEX_ALIAS_PRODUCT_SEARCH_READ}/${INDEX_ALIAS_PRODUCT_SEARCH_WRITE} and ${INDEX_ALIAS_SELLER_SEARCH_READ}/${INDEX_ALIAS_SELLER_SEARCH_WRITE} must stay aligned with search.index_alias_binding"
echo "[info] auxiliary ops index: ${INDEX_NAME_SEARCH_SYNC_JOBS} (not part of alias authority)"
sync_alias_binding_authority

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
index_demo_doc "${INDEX_ALIAS_PRODUCT_SEARCH_WRITE}" "demo-product-001" "{\"id\":\"demo-product-001\",\"tenant_id\":\"t-demo\",\"seller_id\":\"s-demo\",\"name\":\"Demo Product\",\"description\":\"Demo product for local initialization\",\"sku_code\":\"FILE_STD\",\"status\":\"listed\",\"review_status\":\"approved\",\"visibility_status\":\"visible\",\"visible_to_search\":true,\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
index_demo_doc "${INDEX_ALIAS_SELLER_SEARCH_WRITE}" "demo-seller-001" "{\"id\":\"demo-seller-001\",\"tenant_id\":\"t-demo\",\"name\":\"Demo Seller\",\"description\":\"Seller profile demo\",\"status\":\"active\",\"country_code\":\"CN\",\"region_code\":\"SH\",\"industry_tags\":[\"industrial_manufacturing\"],\"certification_tags\":[\"real_name_verified\",\"compliance:l2\",\"certification:enhanced\"],\"featured_products\":[{\"product_id\":\"demo-product-001\",\"title\":\"Demo Product\",\"category\":\"manufacturing\",\"price_amount\":88.0,\"currency_code\":\"CNY\"}],\"rating_summary\":{\"rating_count\":9,\"average_rating\":4.6,\"reputation_score\":0.91,\"credit_level\":4,\"risk_level\":1},\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
index_demo_doc "${INDEX_NAME_SEARCH_SYNC_JOBS}" "demo-sync-job-001" "{\"id\":\"demo-sync-job-001\",\"tenant_id\":\"t-demo\",\"status\":\"done\",\"created_at\":\"${now}\",\"updated_at\":\"${now}\"}"
echo "[ok] demo documents indexed"

curl -fsS -X POST "${OPENSEARCH_ENDPOINT}/_refresh" >/dev/null
echo "[done] opensearch initialization complete"
