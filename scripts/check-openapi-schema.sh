#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OPENAPI_DIR="packages/openapi"
if [[ ! -d "$OPENAPI_DIR" ]]; then
  echo "[error] missing directory: $OPENAPI_DIR" >&2
  exit 1
fi

shopt -s nullglob
yaml_files=("$OPENAPI_DIR"/*.yaml)
if [[ ${#yaml_files[@]} -eq 0 ]]; then
  echo "[error] no openapi yaml files found under $OPENAPI_DIR" >&2
  exit 1
fi

for file in "${yaml_files[@]}"; do
  grep -qE '^openapi:[[:space:]]+3\.' "$file" || {
    echo "[error] $file missing openapi 3.x header" >&2
    exit 1
  }
  grep -qE '^[[:space:]]*title:' "$file" || {
    echo "[error] $file missing info.title" >&2
    exit 1
  }
  grep -qE '^[[:space:]]*version:' "$file" || {
    echo "[error] $file missing info.version" >&2
    exit 1
  }
  grep -qE '^paths:' "$file" || {
    echo "[error] $file missing paths section" >&2
    exit 1
  }
done

assert_synced_copy() {
  local source_file="$1"
  local archive_file="$2"
  [[ -f "$archive_file" ]] || {
    echo "[error] missing archive copy: $archive_file" >&2
    exit 1
  }
  cmp -s "$source_file" "$archive_file" || {
    echo "[error] $archive_file is not synced with $source_file" >&2
    exit 1
  }
}

assert_file_contains() {
  local file="$1"
  local token="$2"
  local label="$3"
  grep -Fq "$token" "$file" || {
    echo "[error] $file missing $label: $token" >&2
    exit 1
  }
}

assert_file_not_contains() {
  local file="$1"
  local token="$2"
  local label="$3"
  if grep -Fq "$token" "$file"; then
    echo "[error] $file should not contain $label: $token" >&2
    exit 1
  fi
}

# V1 skeleton drift guard for currently implemented internal/ops endpoints.
ops_file="$OPENAPI_DIR/ops.yaml"
for path in \
  "/health/live" \
  "/health/ready" \
  "/health/deps" \
  "/internal/runtime" \
  "/internal/dev/trace-links" \
  "/internal/dev/overview" \
  "/internal/notifications/templates/preview" \
  "/internal/notifications/send" \
  "/internal/notifications/audit/search" \
  "/internal/notifications/dead-letters/{dead_letter_event_id}/replay" \
  "/api/v1/developer/trace" \
  "/api/v1/ops/outbox" \
  "/api/v1/ops/dead-letters" \
  "/api/v1/ops/dead-letters/{id}/reprocess" \
  "/api/v1/ops/external-facts" \
  "/api/v1/ops/external-facts/{id}/confirm" \
  "/api/v1/ops/fairness-incidents" \
  "/api/v1/ops/fairness-incidents/{id}/handle" \
  "/api/v1/ops/projection-gaps" \
  "/api/v1/ops/projection-gaps/{id}/resolve" \
  "/api/v1/ops/consistency/{refType}/{refId}" \
  "/api/v1/ops/consistency/reconcile" \
  "/api/v1/ops/trade-monitor/orders/{orderId}" \
  "/api/v1/ops/trade-monitor/orders/{orderId}/checkpoints" \
  "/api/v1/ops/observability/overview" \
  "/api/v1/ops/logs/query" \
  "/api/v1/ops/logs" \
  "/api/v1/ops/logs/export" \
  "/api/v1/ops/traces/{traceId}" \
  "/api/v1/ops/alerts" \
  "/api/v1/ops/incidents" \
  "/api/v1/ops/slos"; do
  grep -q "$path" "$ops_file" || {
    echo "[error] $ops_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "aggregate_type" \
  "event_type" \
  "target_topic" \
  "step_up_ticket" \
  "dtp.notification.dispatch" \
  "developer.trace.read" \
  "DeveloperTraceLookupResponse" \
  "matched_projection_gap" \
  "OpsOutboxPageResponse" \
  "OpsDeadLetterPageResponse" \
  "consumer_idempotency_records" \
  "ApiResponseExternalFactReceiptPageResponse" \
  "ApiResponseOpsExternalFactConfirmResponse" \
  "ApiResponseFairnessIncidentPageResponse" \
  "ApiResponseOpsFairnessIncidentHandleResponse" \
  "ApiResponseChainProjectionGapPageResponse" \
  "ApiResponseOpsConsistencyResponse" \
  "ApiResponseOpsConsistencyReconcileResponse" \
  "ApiResponseTradeMonitorOverviewResponse" \
  "ApiResponseOpsObservabilityOverviewResponse" \
  "risk.fairness_incident.handle" \
  "ops.log.export"; do
  grep -q "$token" "$ops_file" || {
    echo "[error] $ops_file missing notification contract token: $token" >&2
    exit 1
  }
done

docs_ops_file="docs/02-openapi/ops.yaml"
assert_synced_copy "$ops_file" "$docs_ops_file"

audit_file="$OPENAPI_DIR/audit.yaml"
for path in \
  "/api/v1/audit/orders/{id}" \
  "/api/v1/audit/traces" \
  "/api/v1/audit/packages/export" \
  "/api/v1/audit/replay-jobs" \
  "/api/v1/audit/replay-jobs/{id}" \
  "/api/v1/audit/legal-holds" \
  "/api/v1/audit/legal-holds/{id}/release" \
  "/api/v1/audit/anchor-batches" \
  "/api/v1/audit/anchor-batches/{id}/retry"; do
  grep -q "$path" "$audit_file" || {
    echo "[error] $audit_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "AUDIT_REPLAY_DRY_RUN_ONLY" \
  "AUDIT_LEGAL_HOLD_ACTIVE" \
  "x-idempotency-key" \
  "x-step-up-token" \
  "x-step-up-challenge-id" \
  "state_replay" \
  "execution_policy" \
  "audit.package.export" \
  "audit.legal_hold.manage" \
  "AuditReplayJobDetailResponse" \
  "AuditLegalHoldActionResponse" \
  "AuditAnchorBatchPageResponse" \
  "anchor_status" \
  "storage_uri" \
  "step_up_bound"; do
  grep -q "$token" "$audit_file" || {
    echo "[error] $audit_file missing audit control-plane token: $token" >&2
    exit 1
  }
done

docs_audit_file="docs/02-openapi/audit.yaml"
assert_synced_copy "$audit_file" "$docs_audit_file"

search_file="$OPENAPI_DIR/search.yaml"
for path in \
  "/api/v1/catalog/search" \
  "/api/v1/ops/search/sync" \
  "/api/v1/ops/search/reindex" \
  "/api/v1/ops/search/aliases/switch" \
  "/api/v1/ops/search/cache/invalidate" \
  "/api/v1/ops/search/ranking-profiles" \
  "/api/v1/ops/search/ranking-profiles/{id}"; do
  grep -q "$path" "$search_file" || {
    echo "[error] $search_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "Authorization: Bearer <access_token>" \
  "X-Idempotency-Key" \
  "X-Step-Up-Token" \
  "portal.search.read" \
  "ops.search_sync.read" \
  "ops.search_reindex.execute" \
  "ops.search_alias.manage" \
  "ops.search_cache.invalidate" \
  "ops.search_ranking.read" \
  "ops.search_ranking.manage" \
  "audit.audit_event" \
  "audit.access_audit" \
  "ops.system_log" \
  "SEARCH_QUERY_INVALID" \
  "SEARCH_BACKEND_UNAVAILABLE" \
  "SEARCH_RESULT_STALE" \
  "SEARCH_REINDEX_FORBIDDEN" \
  "SEARCH_ALIAS_SWITCH_FORBIDDEN" \
  "SEARCH_CACHE_INVALIDATE_FORBIDDEN"; do
  assert_file_contains "$search_file" "$token" "search contract token"
done
assert_file_not_contains "$search_file" "x-role" "legacy placeholder auth header"

docs_search_file="docs/02-openapi/search.yaml"
assert_synced_copy "$search_file" "$docs_search_file"

recommendation_file="$OPENAPI_DIR/recommendation.yaml"
for path in \
  "/api/v1/recommendations" \
  "/api/v1/recommendations/track/exposure" \
  "/api/v1/recommendations/track/click" \
  "/api/v1/ops/recommendation/placements" \
  "/api/v1/ops/recommendation/placements/{placement_code}" \
  "/api/v1/ops/recommendation/ranking-profiles" \
  "/api/v1/ops/recommendation/ranking-profiles/{id}" \
  "/api/v1/ops/recommendation/rebuild"; do
  grep -q "$path" "$recommendation_file" || {
    echo "[error] $recommendation_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "Authorization: Bearer <access_token>" \
  "X-Idempotency-Key" \
  "X-Step-Up-Token" \
  "portal.recommendation.read" \
  "ops.recommendation.read" \
  "ops.recommendation.manage" \
  "ops.recommend_rebuild.execute" \
  "recommendation.placement.patch" \
  "recommendation.ranking_profile.patch" \
  "recommendation.rebuild.execute" \
  "audit.audit_event" \
  "audit.access_audit" \
  "ops.system_log"; do
  assert_file_contains "$recommendation_file" "$token" "recommendation contract token"
done
assert_file_not_contains "$recommendation_file" "x-role" "legacy placeholder auth header"

docs_recommendation_file="docs/02-openapi/recommendation.yaml"
assert_synced_copy "$recommendation_file" "$docs_recommendation_file"

catalog_file="$OPENAPI_DIR/catalog.yaml"
for path in \
  "/api/v1/catalog/standard-scenarios" \
  "/api/v1/products" \
  "/api/v1/products/{id}" \
  "/api/v1/products/{id}/skus" \
  "/api/v1/products/{id}/metadata-profile" \
  "/api/v1/products/{id}/submit" \
  "/api/v1/review/subjects/{id}" \
  "/api/v1/review/products/{id}" \
  "/api/v1/review/compliance/{id}" \
  "/api/v1/sellers/{orgId}/profile"; do
  grep -q "$path" "$catalog_file" || {
    echo "[error] $catalog_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "getStandardScenarioTemplates" \
  "ApiResponseStandardScenarioTemplateList" \
  "listProducts" \
  "ApiResponseProductList" \
  "ApiResponseDataProduct" \
  "ApiResponseProductSku" \
  "ApiResponseProductSubmit" \
  "ApiResponseReviewDecision" \
  "reviewSubject" \
  "reviewProduct" \
  "reviewCompliance" \
  "ReviewDecisionRequest" \
  "X-Idempotency-Key" \
  "getProductDetail" \
  "ApiResponseProductDetail" \
  "ProductDetail" \
  "getSellerProfile" \
  "ApiResponseSellerProfile" \
  "SellerProfile" \
  "SellerFeaturedProduct" \
  "SellerRatingSummary" \
  "certification_tags" \
  "featured_products" \
  "rating_summary" \
  "catalog.standard.scenarios.read" \
  "StandardScenarioTemplate"; do
  assert_file_contains "$catalog_file" "$token" "catalog standard-scenarios contract token"
done

docs_catalog_file="docs/02-openapi/catalog.yaml"
assert_synced_copy "$catalog_file" "$docs_catalog_file"

iam_file="$OPENAPI_DIR/iam.yaml"
for path in \
  "/api/v1/iam/orgs" \
  "/api/v1/iam/orgs/{id}" \
  "/api/v1/apps" \
  "/api/v1/apps/{id}" \
  "/api/v1/apps/{id}/credentials/rotate" \
  "/api/v1/apps/{id}/credentials/revoke" \
  "/api/v1/auth/me"; do
  grep -q "$path" "$iam_file" || {
    echo "[error] $iam_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "listOrganizations" \
  "getOrganization" \
  "ApiResponseOrganizationAggregateView" \
  "ApiResponseOrganizationAggregateViewList" \
  "OrganizationAggregateView" \
  "review_status" \
  "risk_status" \
  "sellable_status" \
  "blacklist_active" \
  "listApplications" \
  "createApplication" \
  "patchApplication" \
  "rotateApplicationSecret" \
  "revokeApplicationSecret" \
  "ApiResponseApplicationView" \
  "ApiResponseApplicationViewList" \
  "ApplicationView" \
  "CreateApplicationRequest" \
  "PatchApplicationRequest" \
  "RotateApplicationSecretRequest" \
  "client_secret_status" \
  "X-Idempotency-Key" \
  "getAuthMe" \
  "ApiResponseSessionContextView" \
  "SessionContextView" \
  "jwt_mirror" \
  "local_test_user" \
  "auth_context_level" \
  "application/json"; do
  assert_file_contains "$iam_file" "$token" "iam auth/me contract token"
done

docs_iam_file="docs/02-openapi/iam.yaml"
assert_synced_copy "$iam_file" "$docs_iam_file"

trade_file="$OPENAPI_DIR/trade.yaml"
for path in \
  "/api/v1/orders/standard-templates" \
  "/api/v1/orders" \
  "/api/v1/orders/{id}" \
  "/api/v1/orders/{id}/lifecycle-snapshots" \
  "/api/v1/orders/{id}/cancel"; do
  grep -q "$path" "$trade_file" || {
    echo "[error] $trade_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "listStandardOrderTemplates" \
  "createOrder" \
  "getOrderDetail" \
  "getOrderLifecycleSnapshots" \
  "cancelOrder" \
  "X-Idempotency-Key" \
  "GetOrderTemplatesResponse" \
  "CreateOrderResponseData" \
  "GetOrderDetailResponseData" \
  "GetOrderLifecycleSnapshotsResponseData" \
  "ScenarioSkuSnapshot" \
  "per_sku_snapshot_required" \
  "multi_sku_requires_independent_contract_authorization_settlement"; do
  assert_file_contains "$trade_file" "$token" "trade order contract token"
done

docs_trade_file="docs/02-openapi/trade.yaml"
assert_synced_copy "$trade_file" "$docs_trade_file"

delivery_file="$OPENAPI_DIR/delivery.yaml"
for path in \
  "/api/v1/orders/{id}/deliver" \
  "/api/v1/orders/{id}/accept" \
  "/api/v1/orders/{id}/reject" \
  "/api/v1/orders/{id}/download-ticket" \
  "/api/v1/orders/{id}/subscriptions" \
  "/api/v1/orders/{id}/share-grants" \
  "/api/v1/orders/{id}/template-grants" \
  "/api/v1/orders/{id}/sandbox-workspaces" \
  "/api/v1/orders/{id}/usage-log"; do
  grep -q "$path" "$delivery_file" || {
    echo "[error] $delivery_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "CommitOrderDeliveryRequest" \
  "CommitOrderDeliveryResponseEnvelope" \
  "AcceptOrderRequest" \
  "AcceptOrderResponseEnvelope" \
  "RejectOrderRequest" \
  "RejectOrderResponseEnvelope" \
  "DownloadTicketResponseEnvelope" \
  "ManageRevisionSubscriptionRequest" \
  "ManageShareGrantRequest" \
  "ManageTemplateGrantRequest" \
  "ManageSandboxWorkspaceRequest" \
  "ApiUsageLogResponseEnvelope" \
  "X-Idempotency-Key" \
  "delivery.file.commit" \
  "delivery.report.commit" \
  "delivery.api.enable" \
  "delivery.accept.execute" \
  "delivery.reject.execute"; do
  assert_file_contains "$delivery_file" "$token" "delivery center contract token"
done

docs_delivery_file="docs/02-openapi/delivery.yaml"
assert_synced_copy "$delivery_file" "$docs_delivery_file"

billing_file="$OPENAPI_DIR/billing.yaml"
for path in \
  "/api/v1/billing/{order_id}" \
  "/api/v1/refunds" \
  "/api/v1/compensations" \
  "/api/v1/cases" \
  "/api/v1/cases/{id}/evidence" \
  "/api/v1/cases/{id}/resolve"; do
  grep -q "$path" "$billing_file" || {
    echo "[error] $billing_file missing path: $path" >&2
    exit 1
  }
done

for token in \
  "BillingOrderDetailResponse" \
  "BillingOrderDetail" \
  "BillingEvent" \
  "BillingSettlementSummary" \
  "tax_engine_status" \
  "tax_rule_code" \
  "CreateRefundRequest" \
  "RefundExecutionResponse" \
  "CreateCompensationRequest" \
  "CompensationExecutionResponse" \
  "CreateDisputeCaseRequest" \
  "DisputeCaseResponse" \
  "UploadDisputeEvidenceMultipartRequest" \
  "DisputeEvidenceResponse" \
  "ResolveDisputeCaseRequest" \
  "DisputeResolutionResponse" \
  "multipart/form-data" \
  "x-idempotency-key" \
  "x-step-up-token" \
  "x-step-up-challenge-id"; do
  assert_file_contains "$billing_file" "$token" "billing web contract token"
done

docs_billing_file="docs/02-openapi/billing.yaml"
assert_synced_copy "$billing_file" "$docs_billing_file"

echo "[ok] openapi schema skeleton check passed"
