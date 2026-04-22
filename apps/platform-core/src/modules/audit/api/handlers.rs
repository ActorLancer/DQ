use audit_kit::{AuditContext, AuditEvent, EvidencePackage, LegalHold, ReplayJob, ReplayResult};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use db::GenericClient;
use http::ApiResponse;
use kernel::{EntityId, ErrorCode, ErrorResponse};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::AppState;
use crate::modules::audit::application::{EvidenceWriteCommand, record_evidence_snapshot};
use crate::modules::audit::domain::{
    AnchorBatchPageView, AnchorBatchQuery, AuditAnchorBatchRetryRequest, AuditAnchorBatchRetryView,
    AuditLegalHoldActionView, AuditLegalHoldCreateRequest, AuditLegalHoldReleaseRequest,
    AuditPackageExportRequest, AuditPackageExportView, AuditReplayJobCreateRequest,
    AuditReplayJobDetailView, AuditTracePageView, AuditTraceQuery, ChainProjectionGapPageView,
    ChainProjectionGapQuery, DeveloperTraceLookupView, DeveloperTraceQuery,
    DeveloperTraceSubjectView, ExternalFactReceiptPageView, ExternalFactReceiptQuery,
    FairnessIncidentPageView, FairnessIncidentQuery, ObservabilityBackendStatusView,
    OpsAlertPageView, OpsAlertQuery, OpsAlertSummaryView, OpsConsistencyBusinessStateView,
    OpsConsistencyExternalFactStateView, OpsConsistencyProofStateView,
    OpsConsistencyReconcileRequest, OpsConsistencyReconcileView,
    OpsConsistencyRepairRecommendationView, OpsConsistencyView, OpsDeadLetterPageView,
    OpsDeadLetterQuery, OpsDeadLetterReprocessRequest, OpsDeadLetterReprocessView,
    OpsExternalFactConfirmRequest, OpsExternalFactConfirmView, OpsFairnessIncidentHandleRequest,
    OpsFairnessIncidentHandleView, OpsIncidentPageView, OpsIncidentQuery, OpsLogExportRequest,
    OpsLogExportView, OpsLogMirrorPageView, OpsLogMirrorQuery, OpsObservabilityOverviewView,
    OpsOutboxPageView, OpsOutboxQuery, OpsProjectionGapResolveRequest, OpsProjectionGapResolveView,
    OpsServiceHealthView, OpsSloPageView, OpsSloQuery, OpsSloSummaryView, OpsTraceLookupView,
    OrderAuditQuery, OrderAuditView, TradeMonitorCheckpointPageView, TradeMonitorCheckpointQuery,
    TradeMonitorOverviewView,
};
use crate::modules::audit::dto::{
    AlertEventView, AnchorBatchView, AuditTraceView, ChainAnchorView, ChainProjectionGapView,
    DeadLetterEventView, EvidenceManifestView, EvidencePackageView, ExternalFactReceiptView,
    FairnessIncidentView, IncidentTicketView, LegalHoldView, ObservabilityBackendView,
    OutboxEventView, ReplayJobView, ReplayResultView, SloView, SystemLogMirrorView, TraceIndexView,
    TradeLifecycleCheckpointView,
};
use crate::modules::audit::repo::{self, AccessAuditInsert, OrderAuditScope, SystemLogInsert};
use crate::modules::storage::application::{delete_object, put_object_bytes};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};

const EXPORT_BUCKET_ENV: &str = "BUCKET_EVIDENCE_PACKAGES";
const DEFAULT_EXPORT_BUCKET: &str = "evidence-packages";
const LOG_EXPORT_BUCKET_ENV: &str = "BUCKET_REPORT_RESULTS";
const DEFAULT_LOG_EXPORT_BUCKET: &str = "report-results";
const EXPORT_STEP_UP_ACTION: &str = "audit.package.export";
const EXPORT_STEP_UP_ACTION_COMPAT: &str = "audit.evidence.export";
const LOG_EXPORT_STEP_UP_ACTION: &str = "ops.log.export";
const LOG_EXPORT_MAX_ROWS: i64 = 1000;
const REPLAY_STEP_UP_ACTION: &str = "audit.replay.execute";
const LEGAL_HOLD_STEP_UP_ACTION: &str = "audit.legal_hold.manage";
const ANCHOR_STEP_UP_ACTION: &str = "audit.anchor.manage";
const DEAD_LETTER_REPROCESS_STEP_UP_ACTION: &str = "ops.dead_letter.reprocess";
const CONSISTENCY_RECONCILE_STEP_UP_ACTION: &str = "ops.consistency.reconcile";
const EXTERNAL_FACT_CONFIRM_STEP_UP_ACTION: &str = "ops.external_fact.manage";
const FAIRNESS_INCIDENT_HANDLE_STEP_UP_ACTION: &str = "risk.fairness_incident.handle";
const PROJECTION_GAP_RESOLVE_STEP_UP_ACTION: &str = "ops.projection_gap.manage";
const CONSISTENCY_RECONCILE_TARGET_TOPIC: &str = "dtp.consistency.reconcile";
const REPLAY_DRY_RUN_ONLY_ERROR: &str = "AUDIT_REPLAY_DRY_RUN_ONLY";
const DEAD_LETTER_REPROCESS_DRY_RUN_ONLY_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_DRY_RUN_ONLY";
const DEAD_LETTER_REPROCESS_NOT_SUPPORTED_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_NOT_SUPPORTED";
const DEAD_LETTER_REPROCESS_STATE_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_STATE_CONFLICT";
const CONSISTENCY_RECONCILE_DRY_RUN_ONLY_ERROR: &str = "AUDIT_CONSISTENCY_RECONCILE_DRY_RUN_ONLY";
const EXTERNAL_FACT_CONFIRM_STATE_ERROR: &str = "AUDIT_EXTERNAL_FACT_CONFIRM_STATE_CONFLICT";
const FAIRNESS_INCIDENT_HANDLE_STATE_ERROR: &str = "AUDIT_FAIRNESS_INCIDENT_HANDLE_STATE_CONFLICT";
const PROJECTION_GAP_RESOLVE_STATE_ERROR: &str = "AUDIT_PROJECTION_GAP_RESOLVE_STATE_CONFLICT";
const PROJECTION_GAP_STATE_DIGEST_ERROR: &str = "AUDIT_PROJECTION_GAP_STATE_DIGEST_CONFLICT";
const LEGAL_HOLD_ACTIVE_ERROR: &str = "AUDIT_LEGAL_HOLD_ACTIVE";
const ANCHOR_BATCH_NOT_RETRYABLE_ERROR: &str = "AUDIT_ANCHOR_BATCH_NOT_RETRYABLE";
const LOG_EXPORT_EMPTY_ERROR: &str = "OPS_LOG_EXPORT_EMPTY";
const DEVELOPER_TRACE_RECENT_LIMIT: i64 = 10;
const DEVELOPER_TRACE_LOG_LIMIT: i64 = 20;

#[derive(Debug, Clone)]
struct DeveloperTraceResolution {
    lookup_mode: String,
    lookup_value: String,
    matched_object_type: String,
    matched_object_id: Option<String>,
    resolved_ref_type: String,
    resolved_ref_id: String,
    matched_audit_trace: Option<AuditTraceView>,
    matched_outbox_event: Option<repo::OutboxEventRecord>,
    matched_dead_letter: Option<repo::DeadLetterEventRecord>,
    matched_chain_anchor: Option<repo::ChainAnchorRecord>,
    matched_projection_gap: Option<repo::ChainProjectionGapRecord>,
    matched_checkpoint: Option<repo::TradeLifecycleCheckpointRecord>,
    request_id: Option<String>,
    trace_id: Option<String>,
}

pub(in crate::modules::audit) async fn get_order_audit_traces(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Query(query): Query<OrderAuditQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OrderAuditView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&order_id, "order_id", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::TraceRead,
        "audit order trace read",
    )?;

    let client = state_client(&state)?;
    let scope = repo::load_order_audit_scope(&client, &order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit order trace target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(&headers, &scope, &request_id, "audit order trace read")?;

    let pagination = query.pagination();
    let trace_query = AuditTraceQuery {
        order_id: Some(order_id.clone()),
        page: query.page,
        page_size: query.page_size,
        ..Default::default()
    };
    let trace_page = repo::search_audit_traces(
        &client,
        &trace_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_lookup_side_effects(
        &client,
        &headers,
        "order",
        Some(order_id.clone()),
        "GET /api/v1/audit/orders/{id}",
        json!({
            "order_id": order_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": trace_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OrderAuditView {
        order_id: scope.order_id,
        buyer_org_id: scope.buyer_org_id,
        seller_org_id: scope.seller_org_id,
        status: scope.status,
        payment_status: scope.payment_status,
        total: trace_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        traces: trace_page.items,
    }))
}

pub(in crate::modules::audit) async fn get_audit_traces(
    State(state): State<AppState>,
    Query(query): Query<AuditTraceQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AuditTracePageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_optional_uuid(query.order_id.as_deref(), "order_id", &request_id)?;
    validate_optional_uuid(query.ref_id.as_deref(), "ref_id", &request_id)?;
    require_permission(&headers, AuditPermission::TraceRead, "audit trace read")?;

    let client = state_client(&state)?;
    ensure_trace_query_scope(&client, &headers, &query, &request_id).await?;

    let pagination = query.pagination();
    let trace_page = repo::search_audit_traces(
        &client,
        &query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_lookup_side_effects(
        &client,
        &headers,
        "audit_trace_query",
        query
            .effective_order_id()
            .map(ToString::to_string)
            .or_else(|| query.ref_id.clone()),
        "GET /api/v1/audit/traces",
        json!({
            "order_id": query.order_id,
            "ref_type": query.ref_type,
            "ref_id": query.ref_id,
            "request_id": query.request_id,
            "trace_id": query.trace_id,
            "action_name": query.action_name,
            "result_code": query.result_code,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": trace_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(AuditTracePageView {
        total: trace_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: trace_page.items,
    }))
}

pub(in crate::modules::audit) async fn get_developer_trace(
    State(state): State<AppState>,
    Query(query): Query<DeveloperTraceQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<DeveloperTraceLookupView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    let normalized_query = normalize_developer_trace_query(query, &request_id)?;
    require_permission(
        &headers,
        AuditPermission::DeveloperTraceRead,
        "developer trace read",
    )?;

    let client = state_client(&state)?;
    let resolution =
        resolve_developer_trace_lookup(&client, &normalized_query, &request_id).await?;
    let matched_subject = repo::load_consistency_subject(
        &client,
        resolution.resolved_ref_type.as_str(),
        resolution.resolved_ref_id.as_str(),
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        not_found(
            &request_id,
            format!(
                "developer trace subject not found: ref_type={} ref_id={}",
                resolution.resolved_ref_type, resolution.resolved_ref_id
            ),
        )
    })?;
    let resolved_order_id = developer_trace_order_id(&matched_subject).ok_or_else(|| {
        not_found(
            &request_id,
            format!(
                "developer trace target is not linked to an order: ref_type={} ref_id={}",
                matched_subject.ref_type, matched_subject.ref_id
            ),
        )
    })?;
    let order_scope = repo::load_order_audit_scope(&client, resolved_order_id.as_str())
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("developer trace order not found: {resolved_order_id}"),
            )
        })?;
    ensure_developer_trace_scope(&headers, &order_scope, &request_id)?;

    let order_subject =
        if matched_subject.ref_type == "order" && matched_subject.ref_id == resolved_order_id {
            matched_subject.clone()
        } else {
            repo::load_consistency_subject(&client, "order", resolved_order_id.as_str())
                .await
                .map_err(map_db_error)?
                .ok_or_else(|| {
                    not_found(
                        &request_id,
                        format!("developer trace order subject not found: {resolved_order_id}"),
                    )
                })?
        };

    let recent_outbox_events = repo::search_recent_outbox_events_for_aggregates(
        &client,
        &consistency_aggregate_type_candidates("order"),
        resolved_order_id.as_str(),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_dead_letters = repo::search_recent_dead_letters_for_aggregates(
        &client,
        &consistency_aggregate_type_candidates("order"),
        resolved_order_id.as_str(),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_external_facts = repo::search_recent_external_fact_receipts_for_refs(
        &client,
        &["order".to_string()],
        resolved_order_id.as_str(),
        Some(resolved_order_id.as_str()),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_projection_gaps = repo::search_recent_chain_projection_gaps_for_aggregates(
        &client,
        &consistency_aggregate_type_candidates("order"),
        resolved_order_id.as_str(),
        Some(resolved_order_id.as_str()),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_chain_anchors = repo::search_recent_chain_anchors_for_refs(
        &client,
        &["order".to_string()],
        resolved_order_id.as_str(),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_audit_traces = repo::search_recent_audit_traces_for_refs(
        &client,
        &["order".to_string()],
        resolved_order_id.as_str(),
        DEVELOPER_TRACE_RECENT_LIMIT,
    )
    .await
    .map_err(map_db_error)?;
    let recent_checkpoints = repo::search_trade_lifecycle_checkpoints_by_order(
        &client,
        resolved_order_id.as_str(),
        &TradeMonitorCheckpointQuery::default(),
        DEVELOPER_TRACE_RECENT_LIMIT,
        0,
    )
    .await
    .map_err(map_db_error)?;

    let resolved_trace_id = developer_trace_resolved_trace_id(
        resolution.trace_id.clone(),
        &recent_audit_traces,
        &recent_outbox_events,
        &recent_dead_letters,
        &recent_external_facts.items,
        &recent_projection_gaps.items,
        &recent_checkpoints.items,
    );
    let trace = if let Some(trace_id) = resolved_trace_id.as_deref() {
        repo::load_trace_index_by_trace_id(&client, trace_id)
            .await
            .map_err(map_db_error)?
    } else {
        None
    };
    let recent_logs_query = if let Some(trace_id) = trace
        .as_ref()
        .map(|record| record.trace_id.clone())
        .or_else(|| resolved_trace_id.clone())
    {
        OpsLogMirrorQuery {
            trace_id: Some(trace_id),
            page: Some(1),
            page_size: Some(DEVELOPER_TRACE_LOG_LIMIT as u32),
            ..Default::default()
        }
    } else {
        OpsLogMirrorQuery {
            object_type: Some("order".to_string()),
            object_id: Some(resolved_order_id.clone()),
            page: Some(1),
            page_size: Some(DEVELOPER_TRACE_LOG_LIMIT as u32),
            ..Default::default()
        }
    };
    let recent_logs =
        repo::search_system_log_mirrors(&client, &recent_logs_query, DEVELOPER_TRACE_LOG_LIMIT, 0)
            .await
            .map_err(map_db_error)?;

    let subject_snapshot =
        build_developer_trace_snapshot(&order_subject.snapshot, &matched_subject.snapshot);
    let subject = DeveloperTraceSubjectView {
        lookup_mode: resolution.lookup_mode.clone(),
        lookup_value: resolution.lookup_value.clone(),
        matched_object_type: resolution.matched_object_type.clone(),
        matched_object_id: resolution.matched_object_id.clone(),
        resolved_ref_type: resolution.resolved_ref_type.clone(),
        resolved_ref_id: resolution.resolved_ref_id.clone(),
        resolved_order_id: resolved_order_id.clone(),
        business_status: order_subject.business_status.clone(),
        payment_status: order_scope.payment_status.clone(),
        delivery_status: json_string(&order_subject.snapshot, "delivery_status"),
        acceptance_status: json_string(&order_subject.snapshot, "acceptance_status"),
        settlement_status: json_string(&order_subject.snapshot, "settlement_status"),
        dispute_status: json_string(&order_subject.snapshot, "dispute_status"),
        proof_commit_state: order_subject.proof_commit_state.clone(),
        proof_commit_policy: order_subject.proof_commit_policy.clone(),
        external_fact_status: order_subject.external_fact_status.clone(),
        reconcile_status: order_subject.reconcile_status.clone(),
        last_reconciled_at: order_subject.last_reconciled_at.clone(),
        request_id: resolution.request_id.clone(),
        trace_id: resolved_trace_id.clone(),
        snapshot: subject_snapshot,
    };

    record_developer_lookup_side_effects(
        &client,
        &headers,
        resolved_order_id.clone(),
        json!({
            "lookup_mode": resolution.lookup_mode,
            "lookup_value": resolution.lookup_value,
            "matched_object_type": resolution.matched_object_type,
            "matched_object_id": resolution.matched_object_id,
            "resolved_ref_type": resolution.resolved_ref_type,
            "resolved_ref_id": resolution.resolved_ref_id,
            "resolved_order_id": resolved_order_id,
            "resolved_trace_id": subject.trace_id,
            "recent_log_total": recent_logs.total,
            "recent_checkpoint_total": recent_checkpoints.total,
            "recent_external_fact_total": recent_external_facts.total,
            "recent_projection_gap_total": recent_projection_gaps.total,
            "recent_chain_anchor_total": recent_chain_anchors.len(),
            "recent_outbox_total": recent_outbox_events.len(),
            "recent_dead_letter_total": recent_dead_letters.len(),
            "recent_audit_trace_total": recent_audit_traces.len(),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(DeveloperTraceLookupView {
        subject,
        matched_audit_trace: resolution.matched_audit_trace,
        matched_outbox_event: resolution
            .matched_outbox_event
            .as_ref()
            .map(OutboxEventView::from),
        matched_dead_letter: resolution
            .matched_dead_letter
            .as_ref()
            .map(DeadLetterEventView::from),
        matched_chain_anchor: resolution
            .matched_chain_anchor
            .as_ref()
            .map(ChainAnchorView::from),
        matched_projection_gap: resolution
            .matched_projection_gap
            .as_ref()
            .map(ChainProjectionGapView::from),
        matched_checkpoint: resolution
            .matched_checkpoint
            .as_ref()
            .map(TradeLifecycleCheckpointView::from),
        trace: trace.as_ref().map(TraceIndexView::from),
        recent_logs: recent_logs
            .items
            .iter()
            .map(SystemLogMirrorView::from)
            .collect(),
        recent_checkpoints: recent_checkpoints
            .items
            .iter()
            .map(TradeLifecycleCheckpointView::from)
            .collect(),
        recent_external_facts: recent_external_facts
            .items
            .iter()
            .map(ExternalFactReceiptView::from)
            .collect(),
        recent_projection_gaps: recent_projection_gaps
            .items
            .iter()
            .map(ChainProjectionGapView::from)
            .collect(),
        recent_chain_anchors: recent_chain_anchors
            .iter()
            .map(ChainAnchorView::from)
            .collect(),
        recent_outbox_events: recent_outbox_events
            .iter()
            .map(OutboxEventView::from)
            .collect(),
        recent_dead_letters: recent_dead_letters
            .iter()
            .map(DeadLetterEventView::from)
            .collect(),
        recent_audit_traces,
    }))
}

pub(in crate::modules::audit) async fn get_ops_outbox(
    State(state): State<AppState>,
    Query(query): Query<OpsOutboxQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsOutboxPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsOutboxRead, "ops outbox read")?;

    let normalized_query = OpsOutboxQuery {
        outbox_status: normalize_optional_filter(
            query.outbox_status.as_deref(),
            "outbox_status",
            &request_id,
        )?,
        event_type: normalize_optional_filter(
            query.event_type.as_deref(),
            "event_type",
            &request_id,
        )?,
        target_topic: normalize_optional_filter(
            query.target_topic.as_deref(),
            "target_topic",
            &request_id,
        )?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            &request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", &request_id)?,
        aggregate_type: normalize_optional_filter(
            query.aggregate_type.as_deref(),
            "aggregate_type",
            &request_id,
        )?,
        idempotency_key: normalize_optional_filter(
            query.idempotency_key.as_deref(),
            "idempotency_key",
            &request_id,
        )?,
        authority_scope: normalize_optional_filter(
            query.authority_scope.as_deref(),
            "authority_scope",
            &request_id,
        )?,
        source_of_truth: normalize_optional_filter(
            query.source_of_truth.as_deref(),
            "source_of_truth",
            &request_id,
        )?,
        proof_commit_policy: normalize_optional_filter(
            query.proof_commit_policy.as_deref(),
            "proof_commit_policy",
            &request_id,
        )?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let outbox_page = repo::search_outbox_events(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "ops_outbox_query",
        None,
        "GET /api/v1/ops/outbox",
        json!({
            "outbox_status": normalized_query.outbox_status,
            "event_type": normalized_query.event_type,
            "target_topic": normalized_query.target_topic,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "aggregate_type": normalized_query.aggregate_type,
            "idempotency_key": normalized_query.idempotency_key,
            "authority_scope": normalized_query.authority_scope,
            "source_of_truth": normalized_query.source_of_truth,
            "proof_commit_policy": normalized_query.proof_commit_policy,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": outbox_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsOutboxPageView {
        total: outbox_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: outbox_page
            .items
            .iter()
            .map(OutboxEventView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_dead_letters(
    State(state): State<AppState>,
    Query(query): Query<OpsDeadLetterQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsDeadLetterPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(
        &headers,
        AuditPermission::OpsDeadLetterRead,
        "ops dead letter read",
    )?;

    let normalized_query = OpsDeadLetterQuery {
        reprocess_status: normalize_optional_filter(
            query.reprocess_status.as_deref(),
            "reprocess_status",
            &request_id,
        )?,
        failure_stage: normalize_optional_filter(
            query.failure_stage.as_deref(),
            "failure_stage",
            &request_id,
        )?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            &request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", &request_id)?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let dead_letter_page = repo::search_dead_letters(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "dead_letter_query",
        None,
        "GET /api/v1/ops/dead-letters",
        json!({
            "reprocess_status": normalized_query.reprocess_status,
            "failure_stage": normalized_query.failure_stage,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": dead_letter_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsDeadLetterPageView {
        total: dead_letter_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: dead_letter_page
            .items
            .iter()
            .map(DeadLetterEventView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_external_facts(
    State(state): State<AppState>,
    Query(query): Query<ExternalFactReceiptQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ExternalFactReceiptPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_optional_uuid(query.order_id.as_deref(), "order_id", &request_id)?;
    validate_optional_uuid(query.ref_id.as_deref(), "ref_id", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::OpsExternalFactRead,
        "ops external fact read",
    )?;

    let normalized_query = ExternalFactReceiptQuery {
        order_id: normalize_optional_filter(query.order_id.as_deref(), "order_id", &request_id)?,
        ref_type: normalize_optional_filter(query.ref_type.as_deref(), "ref_type", &request_id)?,
        ref_id: normalize_optional_filter(query.ref_id.as_deref(), "ref_id", &request_id)?,
        fact_type: normalize_optional_filter(query.fact_type.as_deref(), "fact_type", &request_id)?,
        provider_type: normalize_optional_filter(
            query.provider_type.as_deref(),
            "provider_type",
            &request_id,
        )?,
        receipt_status: normalize_optional_filter(
            query.receipt_status.as_deref(),
            "receipt_status",
            &request_id,
        )?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            &request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", &request_id)?,
        from: normalize_optional_filter(query.from.as_deref(), "from", &request_id)?,
        to: normalize_optional_filter(query.to.as_deref(), "to", &request_id)?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let external_fact_page = repo::search_external_fact_receipts(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "external_fact_query",
        normalized_query
            .order_id
            .clone()
            .or_else(|| normalized_query.ref_id.clone()),
        "GET /api/v1/ops/external-facts",
        json!({
            "order_id": normalized_query.order_id,
            "ref_type": normalized_query.ref_type,
            "ref_id": normalized_query.ref_id,
            "fact_type": normalized_query.fact_type,
            "provider_type": normalized_query.provider_type,
            "receipt_status": normalized_query.receipt_status,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "from": normalized_query.from,
            "to": normalized_query.to,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": external_fact_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(ExternalFactReceiptPageView {
        total: external_fact_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: external_fact_page
            .items
            .iter()
            .map(ExternalFactReceiptView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn confirm_ops_external_fact(
    State(state): State<AppState>,
    Path(external_fact_receipt_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OpsExternalFactConfirmRequest>,
) -> Result<Json<ApiResponse<OpsExternalFactConfirmView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&external_fact_receipt_id, "id", &request_id)?;
    let confirm_result =
        normalize_external_fact_confirm_result(&payload.confirm_result, &request_id)?;
    let reason = normalize_external_fact_confirm_reason(&payload.reason, &request_id)?;
    let operator_note = normalize_optional_long_text(
        payload.operator_note.as_deref(),
        "operator_note",
        &request_id,
    )?;
    require_permission(
        &headers,
        AuditPermission::OpsExternalFactManage,
        "ops external fact confirm",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "ops external fact confirm")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_external_fact_confirm(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        external_fact_receipt_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());

    let existing_receipt = repo::load_external_fact_receipt(&client, &external_fact_receipt_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("external fact receipt not found: {external_fact_receipt_id}"),
            )
        })?;
    if existing_receipt.receipt_status != "pending" {
        return Err(conflict_error(
            &request_id,
            EXTERNAL_FACT_CONFIRM_STATE_ERROR,
            format!(
                "external fact receipt confirm is only allowed when receipt_status=`pending`; got `{}`",
                existing_receipt.receipt_status
            ),
        ));
    }

    let confirmed_at = current_utc_timestamp(&client).await?;
    let metadata_patch = build_external_fact_confirmation_metadata(
        &headers,
        request_id.as_str(),
        trace_id.as_str(),
        confirmed_at.as_str(),
        confirm_result.as_str(),
        reason.as_str(),
        operator_note.as_deref(),
        actor_user_id.as_str(),
        &existing_receipt,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let confirmed_receipt = repo::confirm_external_fact_receipt(
        &tx,
        external_fact_receipt_id.as_str(),
        confirm_result.as_str(),
        confirmed_at.as_str(),
        &metadata_patch,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        conflict_error(
            &request_id,
            EXTERNAL_FACT_CONFIRM_STATE_ERROR,
            format!("external fact receipt is no longer pending: {external_fact_receipt_id}"),
        )
    })?;

    let audit_event = build_external_fact_confirm_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.clone(),
        actor_user_id.as_str(),
        confirm_result.as_str(),
        reason.as_str(),
        operator_note.as_deref(),
        step_up.challenge_id.clone(),
        step_up.token_present,
        &existing_receipt,
        &confirmed_receipt,
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "confirm".to_string(),
            target_type: "external_fact_receipt".to_string(),
            target_id: Some(external_fact_receipt_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/ops/external-facts/{id}/confirm",
                "order_id": confirmed_receipt.order_id.clone(),
                "ref_domain": confirmed_receipt.ref_domain.clone(),
                "ref_type": confirmed_receipt.ref_type.clone(),
                "ref_id": confirmed_receipt.ref_id.clone(),
                "fact_type": confirmed_receipt.fact_type.clone(),
                "provider_type": confirmed_receipt.provider_type.clone(),
                "confirm_result": confirm_result.clone(),
                "reason": reason.clone(),
                "operator_note": operator_note.clone(),
                "previous_receipt_status": existing_receipt.receipt_status.clone(),
                "receipt_status": confirmed_receipt.receipt_status.clone(),
                "rule_evaluation_status": "pending_follow_up",
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "ops external fact confirm executed: POST /api/v1/ops/external-facts/{id}/confirm"
                    .to_string(),
            structured_payload: json!({
                "module": "ops",
                "endpoint": "POST /api/v1/ops/external-facts/{id}/confirm",
                "access_audit_id": access_audit_id,
                "external_fact_receipt_id": external_fact_receipt_id.clone(),
                "order_id": confirmed_receipt.order_id.clone(),
                "ref_type": confirmed_receipt.ref_type.clone(),
                "ref_id": confirmed_receipt.ref_id.clone(),
                "fact_type": confirmed_receipt.fact_type.clone(),
                "provider_type": confirmed_receipt.provider_type.clone(),
                "confirm_result": confirm_result.clone(),
                "previous_receipt_status": existing_receipt.receipt_status.clone(),
                "receipt_status": confirmed_receipt.receipt_status.clone(),
                "confirmed_at": confirmed_receipt.confirmed_at.clone(),
                "rule_evaluation_status": "pending_follow_up",
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(OpsExternalFactConfirmView {
        external_fact_receipt: ExternalFactReceiptView::from(&confirmed_receipt),
        confirm_result,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        status: "manual_confirmation_recorded".to_string(),
        rule_evaluation_status: "pending_follow_up".to_string(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_fairness_incidents(
    State(state): State<AppState>,
    Query(query): Query<FairnessIncidentQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<FairnessIncidentPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_optional_uuid(query.order_id.as_deref(), "order_id", &request_id)?;
    validate_optional_uuid(
        query.assigned_user_id.as_deref(),
        "assigned_user_id",
        &request_id,
    )?;
    require_permission(
        &headers,
        AuditPermission::RiskFairnessIncidentRead,
        "risk fairness incident read",
    )?;

    let normalized_query = FairnessIncidentQuery {
        order_id: normalize_optional_filter(query.order_id.as_deref(), "order_id", &request_id)?,
        incident_type: normalize_optional_filter(
            query.incident_type.as_deref(),
            "incident_type",
            &request_id,
        )?,
        severity: normalize_optional_filter(query.severity.as_deref(), "severity", &request_id)?,
        fairness_incident_status: normalize_optional_filter(
            query.fairness_incident_status.as_deref(),
            "fairness_incident_status",
            &request_id,
        )?,
        assigned_role_key: normalize_optional_filter(
            query.assigned_role_key.as_deref(),
            "assigned_role_key",
            &request_id,
        )?,
        assigned_user_id: normalize_optional_filter(
            query.assigned_user_id.as_deref(),
            "assigned_user_id",
            &request_id,
        )?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            &request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", &request_id)?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let fairness_page = repo::search_fairness_incidents(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "fairness_incident_query",
        normalized_query
            .order_id
            .clone()
            .or_else(|| normalized_query.assigned_user_id.clone()),
        "GET /api/v1/ops/fairness-incidents",
        json!({
            "order_id": normalized_query.order_id,
            "incident_type": normalized_query.incident_type,
            "severity": normalized_query.severity,
            "fairness_incident_status": normalized_query.fairness_incident_status,
            "assigned_role_key": normalized_query.assigned_role_key,
            "assigned_user_id": normalized_query.assigned_user_id,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": fairness_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(FairnessIncidentPageView {
        total: fairness_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: fairness_page
            .items
            .iter()
            .map(FairnessIncidentView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn handle_ops_fairness_incident(
    State(state): State<AppState>,
    Path(fairness_incident_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OpsFairnessIncidentHandleRequest>,
) -> Result<Json<ApiResponse<OpsFairnessIncidentHandleView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&fairness_incident_id, "id", &request_id)?;
    let action = normalize_fairness_incident_action(&payload.action, &request_id)?;
    let resolution_summary =
        normalize_fairness_incident_resolution_summary(&payload.resolution_summary, &request_id)?;
    let auto_action_override = normalize_optional_filter(
        payload.auto_action_override.as_deref(),
        "auto_action_override",
        &request_id,
    )?;
    let freeze_settlement = payload.freeze_settlement.unwrap_or(false);
    let freeze_delivery = payload.freeze_delivery.unwrap_or(false);
    let create_dispute_suggestion = payload.create_dispute_suggestion.unwrap_or(false);
    require_permission(
        &headers,
        AuditPermission::RiskFairnessIncidentHandle,
        "risk fairness incident handle",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "risk fairness incident handle")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_fairness_incident_handle(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        fairness_incident_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());

    let existing_incident = repo::load_fairness_incident(&client, &fairness_incident_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("fairness incident not found: {fairness_incident_id}"),
            )
        })?;
    if existing_incident.fairness_incident_status != "open" {
        return Err(conflict_error(
            &request_id,
            FAIRNESS_INCIDENT_HANDLE_STATE_ERROR,
            format!(
                "fairness incident handle is only allowed when status=`open`; got `{}`",
                existing_incident.fairness_incident_status
            ),
        ));
    }

    let handled_at = current_utc_timestamp(&client).await?;
    let next_status = if action == "close" { "closed" } else { "open" };
    let closed_at = (next_status == "closed").then_some(handled_at.as_str());
    let action_plan_status = if auto_action_override.is_some()
        || freeze_settlement
        || freeze_delivery
        || create_dispute_suggestion
    {
        "suggestion_recorded"
    } else {
        "no_linked_action"
    };
    let metadata_patch = build_fairness_incident_handle_metadata(
        &headers,
        request_id.as_str(),
        trace_id.as_str(),
        handled_at.as_str(),
        action.as_str(),
        resolution_summary.as_str(),
        auto_action_override.as_deref(),
        freeze_settlement,
        freeze_delivery,
        create_dispute_suggestion,
        actor_user_id.as_str(),
        &existing_incident,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let handled_incident = repo::handle_fairness_incident(
        &tx,
        fairness_incident_id.as_str(),
        next_status,
        resolution_summary.as_str(),
        auto_action_override.as_deref(),
        closed_at,
        request_id.as_str(),
        trace_id.as_str(),
        &metadata_patch,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        conflict_error(
            &request_id,
            FAIRNESS_INCIDENT_HANDLE_STATE_ERROR,
            format!("fairness incident is no longer open: {fairness_incident_id}"),
        )
    })?;

    let audit_event = build_fairness_incident_handle_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.clone(),
        actor_user_id.as_str(),
        action.as_str(),
        resolution_summary.as_str(),
        auto_action_override.as_deref(),
        freeze_settlement,
        freeze_delivery,
        create_dispute_suggestion,
        action_plan_status,
        step_up.challenge_id.clone(),
        step_up.token_present,
        &existing_incident,
        &handled_incident,
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "handle".to_string(),
            target_type: "fairness_incident".to_string(),
            target_id: Some(fairness_incident_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/ops/fairness-incidents/{id}/handle",
                "order_id": handled_incident.order_id.clone(),
                "ref_type": handled_incident.ref_type.clone(),
                "ref_id": handled_incident.ref_id.clone(),
                "incident_type": handled_incident.incident_type.clone(),
                "severity": handled_incident.severity.clone(),
                "action": action.clone(),
                "fairness_incident_status_before": existing_incident.fairness_incident_status.clone(),
                "fairness_incident_status_after": handled_incident.fairness_incident_status.clone(),
                "resolution_summary": resolution_summary.clone(),
                "auto_action_override": auto_action_override.clone(),
                "freeze_settlement": freeze_settlement,
                "freeze_delivery": freeze_delivery,
                "create_dispute_suggestion": create_dispute_suggestion,
                "action_plan_status": action_plan_status,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "risk fairness incident handle executed: POST /api/v1/ops/fairness-incidents/{id}/handle"
                    .to_string(),
            structured_payload: json!({
                "module": "ops",
                "endpoint": "POST /api/v1/ops/fairness-incidents/{id}/handle",
                "access_audit_id": access_audit_id,
                "fairness_incident_id": fairness_incident_id.clone(),
                "order_id": handled_incident.order_id.clone(),
                "incident_type": handled_incident.incident_type.clone(),
                "severity": handled_incident.severity.clone(),
                "action": action.clone(),
                "fairness_incident_status_before": existing_incident.fairness_incident_status.clone(),
                "fairness_incident_status_after": handled_incident.fairness_incident_status.clone(),
                "resolution_summary": resolution_summary.clone(),
                "auto_action_override": auto_action_override.clone(),
                "freeze_settlement": freeze_settlement,
                "freeze_delivery": freeze_delivery,
                "create_dispute_suggestion": create_dispute_suggestion,
                "closed_at": handled_incident.closed_at.clone(),
                "action_plan_status": action_plan_status,
                "business_mutation_executed": false,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(OpsFairnessIncidentHandleView {
        fairness_incident: FairnessIncidentView::from(&handled_incident),
        action,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        status: "manual_handling_recorded".to_string(),
        action_plan_status: action_plan_status.to_string(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_projection_gaps(
    State(state): State<AppState>,
    Query(query): Query<ChainProjectionGapQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ChainProjectionGapPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_optional_uuid(query.aggregate_id.as_deref(), "aggregate_id", &request_id)?;
    validate_optional_uuid(query.order_id.as_deref(), "order_id", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::OpsProjectionGapRead,
        "ops projection gap read",
    )?;

    let normalized_query = ChainProjectionGapQuery {
        aggregate_type: normalize_optional_filter(
            query.aggregate_type.as_deref(),
            "aggregate_type",
            &request_id,
        )?,
        aggregate_id: normalize_optional_filter(
            query.aggregate_id.as_deref(),
            "aggregate_id",
            &request_id,
        )?,
        order_id: normalize_optional_filter(query.order_id.as_deref(), "order_id", &request_id)?,
        chain_id: normalize_optional_filter(query.chain_id.as_deref(), "chain_id", &request_id)?,
        gap_type: normalize_optional_filter(query.gap_type.as_deref(), "gap_type", &request_id)?,
        gap_status: normalize_optional_filter(
            query.gap_status.as_deref(),
            "gap_status",
            &request_id,
        )?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            &request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", &request_id)?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let projection_gap_page = repo::search_chain_projection_gaps(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "projection_gap_query",
        normalized_query
            .order_id
            .clone()
            .or_else(|| normalized_query.aggregate_id.clone()),
        "GET /api/v1/ops/projection-gaps",
        json!({
            "aggregate_type": normalized_query.aggregate_type,
            "aggregate_id": normalized_query.aggregate_id,
            "order_id": normalized_query.order_id,
            "chain_id": normalized_query.chain_id,
            "gap_type": normalized_query.gap_type,
            "gap_status": normalized_query.gap_status,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": projection_gap_page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(ChainProjectionGapPageView {
        total: projection_gap_page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: projection_gap_page
            .items
            .iter()
            .map(ChainProjectionGapView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn resolve_ops_projection_gap(
    State(state): State<AppState>,
    Path(chain_projection_gap_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OpsProjectionGapResolveRequest>,
) -> Result<Json<ApiResponse<OpsProjectionGapResolveView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&chain_projection_gap_id, "id", &request_id)?;
    let reason = normalize_reason(&payload.reason, &request_id)?;
    let resolution_mode =
        normalize_projection_gap_resolution_mode(payload.resolution_mode.as_deref(), &request_id)?;
    let expected_state_digest = normalize_projection_gap_expected_state_digest(
        payload.expected_state_digest.as_deref(),
        &request_id,
    )?;
    let dry_run = payload.dry_run.unwrap_or(true);
    require_permission(
        &headers,
        AuditPermission::OpsProjectionGapManage,
        "ops projection gap resolve",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "ops projection gap resolve")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_projection_gap_resolve(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        chain_projection_gap_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let existing_gap = repo::load_chain_projection_gap(&client, chain_projection_gap_id.as_str())
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("projection gap not found: {chain_projection_gap_id}"),
            )
        })?;
    if existing_gap.gap_status == "resolved" {
        return Err(conflict_error(
            &request_id,
            PROJECTION_GAP_RESOLVE_STATE_ERROR,
            format!(
                "projection gap resolve is only allowed when gap_status is not `resolved`; got `{}`",
                existing_gap.gap_status
            ),
        ));
    }

    let current_state_digest = projection_gap_state_digest(&existing_gap);
    if let Some(expected_state_digest) = expected_state_digest.as_deref() {
        if expected_state_digest != current_state_digest {
            return Err(conflict_error(
                &request_id,
                PROJECTION_GAP_STATE_DIGEST_ERROR,
                format!(
                    "expected_state_digest mismatch for projection gap `{chain_projection_gap_id}`"
                ),
            ));
        }
    }

    let resolved_at = current_utc_timestamp(&client).await?;

    if dry_run {
        let audit_event = build_projection_gap_resolve_audit_event(
            &headers,
            request_id.as_str(),
            trace_id.clone(),
            actor_user_id.as_str(),
            reason.as_str(),
            resolution_mode.as_str(),
            true,
            expected_state_digest.as_deref(),
            step_up.challenge_id.clone(),
            step_up.token_present,
            &existing_gap,
            &existing_gap,
            current_state_digest.clone(),
            current_state_digest.clone(),
        );

        let tx = client.transaction().await.map_err(map_db_error)?;
        repo::insert_audit_event(&tx, &audit_event)
            .await
            .map_err(map_db_error)?;
        let access_audit_id = repo::record_access_audit(
            &tx,
            &AccessAuditInsert {
                accessor_user_id: Some(actor_user_id.clone()),
                accessor_role_key: Some(current_role(&headers)),
                access_mode: "resolve".to_string(),
                target_type: "projection_gap".to_string(),
                target_id: Some(chain_projection_gap_id.clone()),
                masked_view: true,
                breakglass_reason: None,
                step_up_challenge_id: step_up.challenge_id.clone(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                metadata: json!({
                    "endpoint": "POST /api/v1/ops/projection-gaps/{id}/resolve",
                    "aggregate_type": existing_gap.aggregate_type.clone(),
                    "aggregate_id": existing_gap.aggregate_id.clone(),
                    "order_id": existing_gap.order_id.clone(),
                    "chain_id": existing_gap.chain_id.clone(),
                    "gap_type": existing_gap.gap_type.clone(),
                    "gap_status": existing_gap.gap_status.clone(),
                    "reason": reason.clone(),
                    "resolution_mode": resolution_mode.clone(),
                    "dry_run": true,
                    "expected_state_digest": expected_state_digest.clone(),
                    "state_digest": current_state_digest.clone(),
                    "step_up_token_present": step_up.token_present,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;
        repo::record_system_log(
            &tx,
            &SystemLogInsert {
                service_name: "platform-core".to_string(),
                log_level: "INFO".to_string(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                message_text:
                    "ops projection gap resolve prepared: POST /api/v1/ops/projection-gaps/{id}/resolve"
                        .to_string(),
                structured_payload: json!({
                    "module": "ops",
                    "endpoint": "POST /api/v1/ops/projection-gaps/{id}/resolve",
                    "access_audit_id": access_audit_id,
                    "chain_projection_gap_id": chain_projection_gap_id.clone(),
                    "aggregate_type": existing_gap.aggregate_type.clone(),
                    "aggregate_id": existing_gap.aggregate_id.clone(),
                    "order_id": existing_gap.order_id.clone(),
                    "chain_id": existing_gap.chain_id.clone(),
                    "gap_type": existing_gap.gap_type.clone(),
                    "gap_status": existing_gap.gap_status.clone(),
                    "reason": reason.clone(),
                    "resolution_mode": resolution_mode.clone(),
                    "dry_run": true,
                    "expected_state_digest": expected_state_digest.clone(),
                    "state_digest": current_state_digest.clone(),
                }),
            },
        )
        .await
        .map_err(map_db_error)?;
        tx.commit().await.map_err(map_db_error)?;

        return Ok(ApiResponse::ok(OpsProjectionGapResolveView {
            projection_gap: ChainProjectionGapView::from(&existing_gap),
            resolution_mode,
            reason,
            expected_state_digest,
            state_digest: current_state_digest,
            step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
            dry_run: true,
            status: "dry_run_ready".to_string(),
        }));
    }

    let metadata_patch = build_projection_gap_resolve_metadata(
        &headers,
        request_id.as_str(),
        trace_id.as_str(),
        resolved_at.as_str(),
        reason.as_str(),
        resolution_mode.as_str(),
        false,
        actor_user_id.as_str(),
        expected_state_digest.as_deref(),
        current_state_digest.as_str(),
        &existing_gap,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );
    let resolution_summary_patch = build_projection_gap_resolution_summary_patch(
        request_id.as_str(),
        trace_id.as_str(),
        resolved_at.as_str(),
        reason.as_str(),
        resolution_mode.as_str(),
        actor_user_id.as_str(),
        expected_state_digest.as_deref(),
        current_state_digest.as_str(),
        &existing_gap,
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let resolved_gap = repo::resolve_chain_projection_gap(
        &tx,
        chain_projection_gap_id.as_str(),
        resolved_at.as_str(),
        request_id.as_str(),
        trace_id.as_str(),
        &resolution_summary_patch,
        &metadata_patch,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        conflict_error(
            &request_id,
            PROJECTION_GAP_RESOLVE_STATE_ERROR,
            format!("projection gap is no longer open: {chain_projection_gap_id}"),
        )
    })?;
    let resolved_state_digest = projection_gap_state_digest(&resolved_gap);

    let audit_event = build_projection_gap_resolve_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.clone(),
        actor_user_id.as_str(),
        reason.as_str(),
        resolution_mode.as_str(),
        false,
        expected_state_digest.as_deref(),
        step_up.challenge_id.clone(),
        step_up.token_present,
        &existing_gap,
        &resolved_gap,
        current_state_digest.clone(),
        resolved_state_digest.clone(),
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;
    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "resolve".to_string(),
            target_type: "projection_gap".to_string(),
            target_id: Some(chain_projection_gap_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/ops/projection-gaps/{id}/resolve",
                "aggregate_type": resolved_gap.aggregate_type.clone(),
                "aggregate_id": resolved_gap.aggregate_id.clone(),
                "order_id": resolved_gap.order_id.clone(),
                "chain_id": resolved_gap.chain_id.clone(),
                "gap_type": resolved_gap.gap_type.clone(),
                "gap_status_before": existing_gap.gap_status.clone(),
                "gap_status_after": resolved_gap.gap_status.clone(),
                "reason": reason.clone(),
                "resolution_mode": resolution_mode.clone(),
                "dry_run": false,
                "expected_state_digest": expected_state_digest.clone(),
                "state_digest": resolved_state_digest.clone(),
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "ops projection gap resolve executed: POST /api/v1/ops/projection-gaps/{id}/resolve"
                    .to_string(),
            structured_payload: json!({
                "module": "ops",
                "endpoint": "POST /api/v1/ops/projection-gaps/{id}/resolve",
                "access_audit_id": access_audit_id,
                "chain_projection_gap_id": chain_projection_gap_id.clone(),
                "aggregate_type": resolved_gap.aggregate_type.clone(),
                "aggregate_id": resolved_gap.aggregate_id.clone(),
                "order_id": resolved_gap.order_id.clone(),
                "chain_id": resolved_gap.chain_id.clone(),
                "gap_type": resolved_gap.gap_type.clone(),
                "gap_status_before": existing_gap.gap_status.clone(),
                "gap_status_after": resolved_gap.gap_status.clone(),
                "resolved_at": resolved_gap.resolved_at.clone(),
                "reason": reason.clone(),
                "resolution_mode": resolution_mode.clone(),
                "dry_run": false,
                "expected_state_digest": expected_state_digest.clone(),
                "state_digest": resolved_state_digest.clone(),
                "business_mutation_executed": false,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(OpsProjectionGapResolveView {
        projection_gap: ChainProjectionGapView::from(&resolved_gap),
        resolution_mode,
        reason,
        expected_state_digest,
        state_digest: resolved_state_digest,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        dry_run: false,
        status: "resolution_recorded".to_string(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_consistency(
    State(state): State<AppState>,
    Path((ref_type, ref_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsConsistencyView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&ref_id, "refId", &request_id)?;
    let normalized_ref_type = normalize_consistency_ref_type(ref_type.as_str(), &request_id)?;
    require_permission(
        &headers,
        AuditPermission::OpsConsistencyRead,
        "ops consistency read",
    )?;

    let client = state_client(&state)?;
    let subject =
        repo::load_consistency_subject(&client, normalized_ref_type.as_str(), ref_id.as_str())
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| {
                not_found(
                    &request_id,
                    format!(
                        "consistency subject not found: ref_type={} ref_id={ref_id}",
                        normalized_ref_type
                    ),
                )
            })?;
    let ref_type_candidates = consistency_ref_type_candidates(normalized_ref_type.as_str());
    let aggregate_type_candidates =
        consistency_aggregate_type_candidates(normalized_ref_type.as_str());
    let recent_outbox_events = repo::search_recent_outbox_events_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_dead_letters = repo::search_recent_dead_letters_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_receipts = repo::search_recent_external_fact_receipts_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        subject.order_id.as_deref(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let receipt_status_breakdown = repo::count_external_fact_receipts_by_status_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        subject.order_id.as_deref(),
    )
    .await
    .map_err(map_db_error)?;
    let recent_projection_gaps = repo::search_recent_chain_projection_gaps_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        subject.order_id.as_deref(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let projection_gap_status_breakdown =
        repo::count_chain_projection_gaps_by_status_for_aggregates(
            &client,
            &aggregate_type_candidates,
            subject.ref_id.as_str(),
            subject.order_id.as_deref(),
        )
        .await
        .map_err(map_db_error)?;
    let recent_chain_anchors = repo::search_recent_chain_anchors_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_audit_traces = repo::search_recent_audit_traces_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "consistency_query",
        Some(subject.ref_id.clone()),
        "GET /api/v1/ops/consistency/{refType}/{refId}",
        json!({
            "ref_type": normalized_ref_type,
            "ref_id": subject.ref_id,
            "order_id": subject.order_id,
            "recent_outbox_total": recent_outbox_events.len(),
            "recent_dead_letter_total": recent_dead_letters.len(),
            "recent_receipt_total": recent_receipts.total,
            "recent_projection_gap_total": recent_projection_gaps.total,
            "recent_audit_trace_total": recent_audit_traces.len(),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsConsistencyView {
        ref_type: subject.ref_type.clone(),
        ref_id: subject.ref_id.clone(),
        business_state: OpsConsistencyBusinessStateView {
            ref_type: subject.ref_type.clone(),
            ref_id: subject.ref_id.clone(),
            order_id: subject.order_id.clone(),
            business_status: subject.business_status.clone(),
            authority_model: subject.authority_model.clone(),
            business_state_version: subject.business_state_version,
            proof_commit_state: subject.proof_commit_state.clone(),
            proof_commit_policy: subject.proof_commit_policy.clone(),
            external_fact_status: subject.external_fact_status.clone(),
            reconcile_status: subject.reconcile_status.clone(),
            last_reconciled_at: subject.last_reconciled_at.clone(),
            snapshot: subject.snapshot.clone(),
        },
        proof_state: OpsConsistencyProofStateView {
            proof_commit_state: subject.proof_commit_state.clone(),
            proof_commit_policy: subject.proof_commit_policy.clone(),
            latest_chain_anchor: recent_chain_anchors
                .first()
                .map(build_consistency_chain_anchor_view),
            projection_gap_status_breakdown: projection_gap_status_breakdown.clone(),
            open_projection_gap_count: count_open_projection_gaps(
                projection_gap_status_breakdown.as_object(),
            ),
            latest_projection_gap: recent_projection_gaps
                .items
                .first()
                .map(ChainProjectionGapView::from),
        },
        external_fact_state: OpsConsistencyExternalFactStateView {
            summary_status: subject.external_fact_status.clone(),
            total_receipts: recent_receipts.total,
            receipt_status_breakdown,
            latest_receipt: recent_receipts
                .items
                .first()
                .map(ExternalFactReceiptView::from),
        },
        recent_outbox_events: recent_outbox_events
            .iter()
            .map(OutboxEventView::from)
            .collect(),
        recent_dead_letters: recent_dead_letters
            .iter()
            .map(DeadLetterEventView::from)
            .collect(),
        recent_audit_traces,
    }))
}

pub(in crate::modules::audit) async fn get_ops_observability_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsObservabilityOverviewView>>, (StatusCode, Json<ErrorResponse>)> {
    let _request_id = require_request_id(&headers)?;
    require_permission(
        &headers,
        AuditPermission::OpsObservabilityRead,
        "ops observability read",
    )?;

    let client = state_client(&state)?;
    let checked_at = current_utc_timestamp(&client).await?;
    let backends = repo::search_observability_backends(&client)
        .await
        .map_err(map_db_error)?;
    let mut backend_statuses = Vec::with_capacity(backends.len());
    for backend in &backends {
        backend_statuses.push(probe_observability_backend(backend, checked_at.as_str()).await);
    }

    let alert_summary_row = client
        .query_one(
            "SELECT
               COUNT(*) FILTER (WHERE status = 'open')::bigint,
               COUNT(*) FILTER (WHERE status = 'acknowledged')::bigint,
               COUNT(*) FILTER (WHERE severity = 'critical' AND status <> 'resolved')::bigint,
               COUNT(*) FILTER (WHERE severity = 'high' AND status <> 'resolved')::bigint,
               MAX(to_char(fired_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'))
             FROM ops.alert_event",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    let recent_incidents =
        repo::search_incident_tickets(&client, &OpsIncidentQuery::default(), 5, 0)
            .await
            .map_err(map_db_error)?;
    let slo_page = repo::search_slos(&client, &OpsSloQuery::default(), 5, 0)
        .await
        .map_err(map_db_error)?;
    let slo_summary_row = client
        .query_one(
            "SELECT
               COUNT(*)::bigint,
               COUNT(*) FILTER (WHERE COALESCE(ss.status, 'unknown') = 'ok')::bigint,
               COUNT(*) FILTER (WHERE COALESCE(ss.status, 'unknown') = 'degraded')::bigint,
               COUNT(*) FILTER (WHERE COALESCE(ss.status, 'unknown') = 'breached')::bigint
             FROM ops.slo_definition sd
             LEFT JOIN LATERAL (
               SELECT status
               FROM ops.slo_snapshot
               WHERE slo_definition_id = sd.slo_definition_id
               ORDER BY window_ended_at DESC, slo_snapshot_id DESC
               LIMIT 1
             ) ss ON true",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    let key_services = load_key_service_healths(checked_at.as_str()).await;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "observability_overview",
        None,
        "GET /api/v1/ops/observability/overview",
        json!({
            "backend_total": backend_statuses.len(),
            "open_alert_count": alert_summary_row.get::<_, i64>(0),
            "open_incident_count": recent_incidents.total,
            "slo_total": slo_summary_row.get::<_, i64>(0),
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsObservabilityOverviewView {
        backend_statuses,
        alert_summary: OpsAlertSummaryView {
            open_count: alert_summary_row.get(0),
            acknowledged_count: alert_summary_row.get(1),
            critical_count: alert_summary_row.get(2),
            high_count: alert_summary_row.get(3),
            latest_fired_at: alert_summary_row.get(4),
        },
        key_services,
        slo_summary: OpsSloSummaryView {
            total: slo_summary_row.get(0),
            ok_count: slo_summary_row.get(1),
            degraded_count: slo_summary_row.get(2),
            breached_count: slo_summary_row.get(3),
            items: slo_page.items.iter().map(SloView::from).collect(),
        },
        recent_incidents: recent_incidents
            .items
            .iter()
            .map(IncidentTicketView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_logs_query(
    State(state): State<AppState>,
    Query(query): Query<OpsLogMirrorQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsLogMirrorPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsLogQuery, "ops log query")?;
    let normalized_query = normalize_ops_log_query(query, &request_id)?;
    let pagination = normalized_query.pagination();
    let client = state_client(&state)?;
    let page = repo::search_system_log_mirrors(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "system_log_query",
        normalized_query.object_id.clone(),
        "GET /api/v1/ops/logs/query",
        json!({
            "service_name": normalized_query.service_name,
            "log_level": normalized_query.log_level,
            "request_id": normalized_query.request_id,
            "trace_id": normalized_query.trace_id,
            "object_type": normalized_query.object_type,
            "object_id": normalized_query.object_id,
            "from": normalized_query.from,
            "to": normalized_query.to,
            "query": normalized_query.query,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsLogMirrorPageView {
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page.items.iter().map(SystemLogMirrorView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn export_ops_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<OpsLogExportRequest>,
) -> Result<Json<ApiResponse<OpsLogExportView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsLogExport, "ops log export")?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let normalized = normalize_ops_log_export_request(payload, &request_id)?;
    require_log_export_selector(&normalized, &request_id)?;
    ensure_step_up_header_present_for(&headers, &request_id, "ops log export")?;

    let client = state_client(&state)?;
    let step_up = require_step_up_for_log_export(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        normalized.object_id.as_deref(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let export_requested_at = current_utc_timestamp(&client).await?;
    let export_id = next_uuid(&client).await?;
    let search_query = OpsLogMirrorQuery {
        service_name: normalized.service_name.clone(),
        log_level: normalized.log_level.clone(),
        request_id: normalized.request_id.clone(),
        trace_id: normalized.trace_id.clone(),
        object_type: normalized.object_type.clone(),
        object_id: normalized.object_id.clone(),
        from: normalized.from.clone(),
        to: normalized.to.clone(),
        query: normalized.query.clone(),
        page: Some(1),
        page_size: Some(LOG_EXPORT_MAX_ROWS as u32),
    };
    let export_page =
        repo::search_system_log_mirrors(&client, &search_query, LOG_EXPORT_MAX_ROWS, 0)
            .await
            .map_err(map_db_error)?;
    if export_page.total == 0 {
        return Err(conflict_error(
            &request_id,
            LOG_EXPORT_EMPTY_ERROR,
            "ops log export requires at least one matching log mirror row",
        ));
    }

    let bucket_name = log_export_bucket_name();
    let object_key = format!("ops/log-exports/{export_id}.json");
    let object_uri = format!("s3://{bucket_name}/{object_key}");
    let exported_items: Vec<SystemLogMirrorView> = export_page
        .items
        .iter()
        .map(SystemLogMirrorView::from)
        .collect();
    let export_bytes = serde_json::to_vec(&json!({
        "export_id": export_id.clone(),
        "reason": normalized.reason.clone(),
        "filters": {
            "service_name": normalized.service_name.clone(),
            "log_level": normalized.log_level.clone(),
            "request_id": normalized.request_id.clone(),
            "trace_id": normalized.trace_id.clone(),
            "object_type": normalized.object_type.clone(),
            "object_id": normalized.object_id.clone(),
            "from": normalized.from.clone(),
            "to": normalized.to.clone(),
            "query": normalized.query.clone(),
        },
        "request_id": request_id.clone(),
        "trace_id": trace_id.clone(),
        "exported_at": export_requested_at.clone(),
        "exported_count": exported_items.len(),
        "items": exported_items,
    }))
    .map_err(|err| {
        internal_error(
            Some(request_id.clone()),
            format!("ops log export payload encode failed: {err}"),
        )
    })?;
    let object_hash = sha256_hex(export_bytes.as_slice());
    put_object_bytes(
        bucket_name.as_str(),
        object_key.as_str(),
        export_bytes,
        Some("application/json"),
    )
    .await?;

    let audit_event = build_ops_log_export_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.as_str(),
        actor_user_id.as_str(),
        normalized.reason.as_str(),
        object_uri.as_str(),
        object_hash.as_str(),
        export_page.total,
        step_up.challenge_id.clone(),
        step_up.token_present,
        normalized.object_id.clone(),
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let write_result = async {
        repo::insert_audit_event(&tx, &audit_event)
            .await
            .map_err(map_db_error)?;
        let access_audit_id = repo::record_access_audit(
            &tx,
            &AccessAuditInsert {
                accessor_user_id: Some(actor_user_id.clone()),
                accessor_role_key: Some(current_role(&headers)),
                access_mode: "export".to_string(),
                target_type: "system_log_export".to_string(),
                target_id: normalized.object_id.clone(),
                masked_view: true,
                breakglass_reason: None,
                step_up_challenge_id: step_up.challenge_id.clone(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                metadata: json!({
                    "endpoint": "POST /api/v1/ops/logs/export",
                    "reason": normalized.reason,
                    "service_name": normalized.service_name,
                    "log_level": normalized.log_level,
                    "request_id_filter": normalized.request_id,
                    "trace_id_filter": normalized.trace_id,
                    "object_type": normalized.object_type,
                    "object_id": normalized.object_id,
                    "from": normalized.from,
                    "to": normalized.to,
                    "query": normalized.query,
                    "export_id": export_id,
                    "exported_count": export_page.total,
                    "object_uri": object_uri,
                    "object_hash": object_hash,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;
        repo::record_system_log(
            &tx,
            &SystemLogInsert {
                service_name: "platform-core".to_string(),
                log_level: "INFO".to_string(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                message_text: "ops logs exported: POST /api/v1/ops/logs/export".to_string(),
                structured_payload: json!({
                    "module": "ops",
                    "endpoint": "POST /api/v1/ops/logs/export",
                    "access_audit_id": access_audit_id,
                    "export_id": export_id,
                    "object_uri": object_uri,
                    "object_hash": object_hash,
                    "exported_count": export_page.total,
                    "step_up_challenge_id": step_up.challenge_id,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;
        tx.commit().await.map_err(map_db_error)
    }
    .await;
    if let Err(err) = write_result {
        let _ = delete_object(bucket_name.as_str(), object_key.as_str()).await;
        return Err(err);
    }

    Ok(ApiResponse::ok(OpsLogExportView {
        export_id,
        bucket_name,
        object_key,
        object_uri,
        object_hash,
        exported_count: export_page.total,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        content_type: "application/json".to_string(),
        request_id,
        trace_id,
    }))
}

pub(in crate::modules::audit) async fn get_ops_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsTraceLookupView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsTraceRead, "ops trace read")?;
    let client = state_client(&state)?;
    let trace = repo::load_trace_index_by_trace_id(&client, trace_id.as_str())
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found(&request_id, format!("trace not found: {trace_id}")))?;
    let checked_at = current_utc_timestamp(&client).await?;
    let backend_status = if let Some(backend_key) = trace.backend_key.as_deref() {
        if let Some(backend) = repo::search_observability_backends(&client)
            .await
            .map_err(map_db_error)?
            .into_iter()
            .find(|backend| backend.backend_key == backend_key)
        {
            Some(probe_observability_backend(&backend, checked_at.as_str()).await)
        } else {
            None
        }
    } else {
        None
    };
    let related_log_count = repo::count_system_logs_by_trace_id(&client, trace.trace_id.as_str())
        .await
        .map_err(map_db_error)?;
    let related_alert_count =
        repo::count_alert_events_by_trace_id(&client, trace.trace_id.as_str())
            .await
            .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "trace_lookup",
        trace.object_id.clone().or_else(|| trace.ref_id.clone()),
        "GET /api/v1/ops/traces/{traceId}",
        json!({
            "trace_id": trace.trace_id.clone(),
            "request_id": trace.request_id.clone(),
            "ref_type": trace.ref_type.clone(),
            "ref_id": trace.ref_id.clone(),
            "object_type": trace.object_type.clone(),
            "object_id": trace.object_id.clone(),
            "backend_key": trace.backend_key.clone(),
            "related_log_count": related_log_count,
            "related_alert_count": related_alert_count,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsTraceLookupView {
        trace: TraceIndexView::from(&trace),
        related_log_count,
        related_alert_count,
        backend_status,
        tempo_link: build_tempo_trace_link(trace.trace_id.as_str()),
        grafana_link: build_grafana_trace_link(trace.trace_id.as_str()),
    }))
}

pub(in crate::modules::audit) async fn get_ops_alerts(
    State(state): State<AppState>,
    Query(query): Query<OpsAlertQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsAlertPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsAlertRead, "ops alert read")?;
    let normalized_query = normalize_ops_alert_query(query, &request_id)?;
    let pagination = normalized_query.pagination();
    let client = state_client(&state)?;
    let page = repo::search_alert_events(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "alert_query",
        None,
        "GET /api/v1/ops/alerts",
        json!({
            "alert_status": normalized_query.alert_status,
            "severity": normalized_query.severity,
            "source_backend_key": normalized_query.source_backend_key,
            "alert_type": normalized_query.alert_type,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsAlertPageView {
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page.items.iter().map(AlertEventView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_incidents(
    State(state): State<AppState>,
    Query(query): Query<OpsIncidentQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsIncidentPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(
        &headers,
        AuditPermission::OpsIncidentRead,
        "ops incident read",
    )?;
    let normalized_query = normalize_ops_incident_query(query, &request_id)?;
    let pagination = normalized_query.pagination();
    let client = state_client(&state)?;
    let page = repo::search_incident_tickets(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "incident_query",
        None,
        "GET /api/v1/ops/incidents",
        json!({
            "incident_status": normalized_query.incident_status,
            "severity": normalized_query.severity,
            "owner_role_key": normalized_query.owner_role_key,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsIncidentPageView {
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page.items.iter().map(IncidentTicketView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_slos(
    State(state): State<AppState>,
    Query(query): Query<OpsSloQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<OpsSloPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(&headers, AuditPermission::OpsSloRead, "ops slo read")?;
    let normalized_query = normalize_ops_slo_query(query, &request_id)?;
    let pagination = normalized_query.pagination();
    let client = state_client(&state)?;
    let page = repo::search_slos(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "slo_query",
        None,
        "GET /api/v1/ops/slos",
        json!({
            "service_name": normalized_query.service_name,
            "source_backend_key": normalized_query.source_backend_key,
            "status": normalized_query.status,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(OpsSloPageView {
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page.items.iter().map(SloView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_trade_monitor_overview(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<TradeMonitorOverviewView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&order_id, "orderId", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::OpsTradeMonitorRead,
        "ops trade monitor read",
    )?;

    let client = state_client(&state)?;
    let scope = repo::load_order_audit_scope(&client, &order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("trade monitor target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(&headers, &scope, &request_id, "ops trade monitor read")?;

    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let observed_at = current_utc_timestamp(&client).await?;
    let subject = repo::load_consistency_subject(&client, "order", order_id.as_str())
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("trade monitor consistency subject not found: {order_id}"),
            )
        })?;
    let checkpoints = repo::search_trade_lifecycle_checkpoints_by_order(
        &client,
        order_id.as_str(),
        &TradeMonitorCheckpointQuery::default(),
        5,
        0,
    )
    .await
    .map_err(map_db_error)?;
    let current_checkpoint = checkpoints.items.first();
    let recent_external_facts = repo::search_recent_external_fact_receipts_for_refs(
        &client,
        &["order".to_string()],
        order_id.as_str(),
        Some(order_id.as_str()),
        5,
    )
    .await
    .map_err(map_db_error)?;
    let recent_fairness_incidents =
        repo::search_recent_fairness_incidents_for_order(&client, order_id.as_str(), 5)
            .await
            .map_err(map_db_error)?;
    let open_fairness_incident_count =
        repo::count_open_fairness_incidents_for_order(&client, order_id.as_str())
            .await
            .map_err(map_db_error)?;
    let recent_projection_gaps = repo::search_recent_chain_projection_gaps_for_aggregates(
        &client,
        &consistency_aggregate_type_candidates("order"),
        order_id.as_str(),
        Some(order_id.as_str()),
        5,
    )
    .await
    .map_err(map_db_error)?;
    let recent_chain_anchors = repo::search_recent_chain_anchors_for_refs(
        &client,
        &["order".to_string()],
        order_id.as_str(),
        5,
    )
    .await
    .map_err(map_db_error)?;

    let last_external_fact_at = recent_external_facts.items.first().and_then(|receipt| {
        receipt
            .confirmed_at
            .clone()
            .or_else(|| receipt.received_at.clone())
            .or_else(|| receipt.occurred_at.clone())
    });
    let last_chain_confirmed_at = recent_chain_anchors
        .iter()
        .find_map(confirmed_chain_anchor_time);
    let last_observed_at = latest_timestamp(
        checkpoints
            .items
            .iter()
            .map(trade_checkpoint_observed_at)
            .chain(
                recent_external_facts
                    .items
                    .iter()
                    .map(external_fact_observed_at),
            )
            .chain(
                recent_fairness_incidents
                    .items
                    .iter()
                    .map(fairness_incident_observed_at),
            )
            .chain(
                recent_projection_gaps
                    .items
                    .iter()
                    .map(chain_projection_gap_observed_at),
            )
            .chain(recent_chain_anchors.iter().map(confirmed_chain_anchor_time)),
    )
    .unwrap_or_else(|| observed_at.clone());

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "trade_monitor_query",
        Some(order_id.clone()),
        "GET /api/v1/ops/trade-monitor/orders/{orderId}",
        json!({
            "order_id": order_id,
            "business_state": subject.business_status,
            "current_checkpoint_code": current_checkpoint.map(|checkpoint| checkpoint.checkpoint_code.clone()),
            "current_checkpoint_status": current_checkpoint.map(|checkpoint| checkpoint.checkpoint_status.clone()),
            "recent_checkpoint_total": checkpoints.total,
            "recent_external_fact_total": recent_external_facts.total,
            "recent_fairness_incident_total": recent_fairness_incidents.total,
            "recent_projection_gap_total": recent_projection_gaps.total,
            "open_fairness_incident_count": open_fairness_incident_count,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(TradeMonitorOverviewView {
        order_id: order_id.clone(),
        request_id,
        trace_id,
        business_state: subject.business_status.clone(),
        current_checkpoint_code: current_checkpoint
            .map(|checkpoint| checkpoint.checkpoint_code.clone())
            .unwrap_or_else(|| "not_started".to_string()),
        current_checkpoint_status: current_checkpoint
            .map(|checkpoint| checkpoint.checkpoint_status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        proof_commit_state: subject.proof_commit_state,
        external_fact_status: subject.external_fact_status,
        reconcile_status: subject.reconcile_status,
        open_fairness_incident_count,
        last_external_fact_at,
        last_chain_confirmed_at,
        last_observed_at,
        recent_checkpoints: checkpoints
            .items
            .iter()
            .map(TradeLifecycleCheckpointView::from)
            .collect(),
        recent_external_facts: recent_external_facts
            .items
            .iter()
            .map(ExternalFactReceiptView::from)
            .collect(),
        recent_fairness_incidents: recent_fairness_incidents
            .items
            .iter()
            .map(FairnessIncidentView::from)
            .collect(),
        recent_projection_gaps: recent_projection_gaps
            .items
            .iter()
            .map(ChainProjectionGapView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn get_ops_trade_monitor_checkpoints(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Query(query): Query<TradeMonitorCheckpointQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<TradeMonitorCheckpointPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&order_id, "orderId", &request_id)?;
    require_permission(
        &headers,
        AuditPermission::OpsTradeMonitorRead,
        "ops trade monitor read",
    )?;

    let normalized_query = TradeMonitorCheckpointQuery {
        checkpoint_code: normalize_optional_filter(
            query.checkpoint_code.as_deref(),
            "checkpoint_code",
            &request_id,
        )?,
        checkpoint_status: normalize_optional_filter(
            query.checkpoint_status.as_deref(),
            "checkpoint_status",
            &request_id,
        )?,
        lifecycle_stage: normalize_optional_filter(
            query.lifecycle_stage.as_deref(),
            "lifecycle_stage",
            &request_id,
        )?,
        from: normalize_optional_filter(query.from.as_deref(), "from", &request_id)?,
        to: normalize_optional_filter(query.to.as_deref(), "to", &request_id)?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let scope = repo::load_order_audit_scope(&client, &order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("trade monitor target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(&headers, &scope, &request_id, "ops trade monitor read")?;

    let pagination = normalized_query.pagination();
    let page = repo::search_trade_lifecycle_checkpoints_by_order(
        &client,
        order_id.as_str(),
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_ops_lookup_side_effects(
        &client,
        &headers,
        "trade_checkpoint_query",
        Some(order_id.clone()),
        "GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints",
        json!({
            "order_id": order_id,
            "checkpoint_code": normalized_query.checkpoint_code,
            "checkpoint_status": normalized_query.checkpoint_status,
            "lifecycle_stage": normalized_query.lifecycle_stage,
            "from": normalized_query.from,
            "to": normalized_query.to,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(TradeMonitorCheckpointPageView {
        order_id,
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page
            .items
            .iter()
            .map(TradeLifecycleCheckpointView::from)
            .collect(),
    }))
}

pub(in crate::modules::audit) async fn reconcile_ops_consistency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<OpsConsistencyReconcileRequest>,
) -> Result<Json<ApiResponse<OpsConsistencyReconcileView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    let normalized_ref_type = normalize_consistency_ref_type(&payload.ref_type, &request_id)?;
    validate_uuid(&payload.ref_id, "ref_id", &request_id)?;
    let mode = normalize_consistency_reconcile_mode(payload.mode.as_deref(), &request_id)?;
    let reason = normalize_consistency_reconcile_reason(&payload.reason, &request_id)?;
    let dry_run = payload.dry_run.unwrap_or(true);
    require_permission(
        &headers,
        AuditPermission::OpsConsistencyReconcile,
        "ops consistency reconcile",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "ops consistency reconcile")?;
    if !dry_run {
        return Err(consistency_reconcile_dry_run_only(&request_id));
    }

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_consistency_reconcile(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        normalized_ref_type.as_str(),
        payload.ref_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());

    let subject = repo::load_consistency_subject(
        &client,
        normalized_ref_type.as_str(),
        payload.ref_id.as_str(),
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        not_found(
            &request_id,
            format!(
                "consistency subject not found: ref_type={} ref_id={}",
                normalized_ref_type, payload.ref_id
            ),
        )
    })?;
    let ref_type_candidates = consistency_ref_type_candidates(normalized_ref_type.as_str());
    let aggregate_type_candidates =
        consistency_aggregate_type_candidates(normalized_ref_type.as_str());
    let recent_outbox_events = repo::search_recent_outbox_events_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_dead_letters = repo::search_recent_dead_letters_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_receipts = repo::search_recent_external_fact_receipts_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        subject.order_id.as_deref(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let recent_projection_gaps = repo::search_recent_chain_projection_gaps_for_aggregates(
        &client,
        &aggregate_type_candidates,
        subject.ref_id.as_str(),
        subject.order_id.as_deref(),
        10,
    )
    .await
    .map_err(map_db_error)?;
    let projection_gap_status_breakdown =
        repo::count_chain_projection_gaps_by_status_for_aggregates(
            &client,
            &aggregate_type_candidates,
            subject.ref_id.as_str(),
            subject.order_id.as_deref(),
        )
        .await
        .map_err(map_db_error)?;
    let recent_chain_anchors = repo::search_recent_chain_anchors_for_refs(
        &client,
        &ref_type_candidates,
        subject.ref_id.as_str(),
        10,
    )
    .await
    .map_err(map_db_error)?;

    let subject_snapshot = build_consistency_reconcile_subject_snapshot(
        &subject,
        recent_chain_anchors.first(),
        recent_receipts.items.first(),
        recent_projection_gaps.items.first(),
        recent_outbox_events.len() as i64,
        recent_dead_letters.len() as i64,
        recent_receipts.total,
        projection_gap_status_breakdown.clone(),
    );
    let recommendations = build_consistency_reconcile_recommendations(
        mode.as_str(),
        &subject,
        &recent_projection_gaps.items,
        &recent_outbox_events,
        &recent_dead_letters,
        recent_receipts.items.first(),
        recent_chain_anchors.first(),
    );
    let recommendation_count = recommendations.len() as i64;
    let recommendation_preview = summarize_consistency_recommendations(&recommendations);
    let related_gap_ids: Vec<String> = recent_projection_gaps
        .items
        .iter()
        .filter_map(|gap| gap.chain_projection_gap_id.clone())
        .collect();
    let related_projection_gaps: Vec<ChainProjectionGapView> = recent_projection_gaps
        .items
        .iter()
        .map(ChainProjectionGapView::from)
        .collect();
    let reconcile_plan = build_consistency_reconcile_plan(
        subject_snapshot.clone(),
        mode.as_str(),
        reason.as_str(),
        dry_run,
        &recommendations,
        &related_projection_gaps,
        projection_gap_status_breakdown.clone(),
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let audit_event = build_consistency_reconcile_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.clone(),
        actor_user_id.as_str(),
        normalized_ref_type.as_str(),
        subject.ref_id.as_str(),
        reason.as_str(),
        mode.as_str(),
        dry_run,
        subject_snapshot.clone(),
        reconcile_plan.clone(),
        step_up.challenge_id.clone(),
        step_up.token_present,
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "reconcile".to_string(),
            target_type: "consistency_reconcile".to_string(),
            target_id: Some(subject.ref_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/ops/consistency/reconcile",
                "ref_type": normalized_ref_type,
                "ref_id": subject.ref_id,
                "mode": mode,
                "reason": reason,
                "dry_run": dry_run,
                "reconcile_target_topic": CONSISTENCY_RECONCILE_TARGET_TOPIC,
                "recommendation_count": recommendation_count,
                "recommendations": recommendation_preview,
                "related_projection_gap_ids": related_gap_ids,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile"
                    .to_string(),
            structured_payload: json!({
                "module": "ops",
                "endpoint": "POST /api/v1/ops/consistency/reconcile",
                "access_audit_id": access_audit_id,
                "ref_type": normalized_ref_type,
                "ref_id": subject.ref_id,
                "mode": mode,
                "dry_run": dry_run,
                "reconcile_target_topic": CONSISTENCY_RECONCILE_TARGET_TOPIC,
                "recommendation_count": recommendation_count,
                "recommendations": recommendation_preview,
                "related_projection_gap_ids": related_gap_ids,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(OpsConsistencyReconcileView {
        ref_type: subject.ref_type.clone(),
        ref_id: subject.ref_id.clone(),
        mode,
        dry_run,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        status: "dry_run_ready".to_string(),
        reconcile_target_topic: CONSISTENCY_RECONCILE_TARGET_TOPIC.to_string(),
        recommendation_count,
        subject_snapshot,
        projection_gap_status_breakdown,
        related_projection_gaps,
        recommendations,
    }))
}

pub(in crate::modules::audit) async fn reprocess_ops_dead_letter(
    State(state): State<AppState>,
    Path(dead_letter_event_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<OpsDeadLetterReprocessRequest>,
) -> Result<Json<ApiResponse<OpsDeadLetterReprocessView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&dead_letter_event_id, "id", &request_id)?;
    let reason = normalize_reason(&payload.reason, &request_id)?;
    let dry_run = payload.dry_run.unwrap_or(true);
    require_permission(
        &headers,
        AuditPermission::OpsDeadLetterReprocess,
        "ops dead letter reprocess",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "ops dead letter reprocess")?;
    if !dry_run {
        return Err(dead_letter_reprocess_dry_run_only(&request_id));
    }

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_dead_letter_reprocess(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        dead_letter_event_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let dead_letter = repo::load_dead_letter_event(&client, dead_letter_event_id.as_str())
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("dead letter event not found: {dead_letter_event_id}"),
            )
        })?;
    if dead_letter.reprocess_status != "not_reprocessed" {
        return Err(dead_letter_reprocess_state_conflict(
            &request_id,
            dead_letter.reprocess_status.as_str(),
        ));
    }

    let consumer_names = resolve_searchrec_dead_letter_consumers(&dead_letter)
        .ok_or_else(|| dead_letter_reprocess_not_supported(&request_id, &dead_letter))?;
    let consumer_groups = searchrec_consumer_groups_for_topic(dead_letter.target_topic.as_deref())
        .ok_or_else(|| dead_letter_reprocess_not_supported(&request_id, &dead_letter))?;
    let replay_target_topic = dead_letter
        .target_topic
        .clone()
        .ok_or_else(|| dead_letter_reprocess_not_supported(&request_id, &dead_letter))?;
    let replay_plan = build_dead_letter_reprocess_plan(
        &dead_letter,
        reason.as_str(),
        dry_run,
        &consumer_names,
        &consumer_groups,
        replay_target_topic.as_str(),
        payload.metadata.clone(),
        request_id.as_str(),
        trace_id.as_str(),
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let audit_event = build_dead_letter_reprocess_audit_event(
        &headers,
        request_id.as_str(),
        trace_id.clone(),
        actor_user_id.as_str(),
        dead_letter_event_id.as_str(),
        reason.as_str(),
        &dead_letter,
        dry_run,
        &consumer_names,
        &consumer_groups,
        replay_target_topic.as_str(),
        replay_plan.clone(),
        step_up.challenge_id.clone(),
        step_up.token_present,
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "reprocess".to_string(),
            target_type: "dead_letter_event".to_string(),
            target_id: Some(dead_letter_event_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/ops/dead-letters/{id}/reprocess",
                "reason": reason,
                "dry_run": dry_run,
                "dead_letter_event_id": dead_letter_event_id,
                "consumer_names": consumer_names,
                "consumer_groups": consumer_groups,
                "replay_target_topic": replay_target_topic,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "ops dead letter reprocess prepared: POST /api/v1/ops/dead-letters/{id}/reprocess"
                    .to_string(),
            structured_payload: json!({
                "module": "ops",
                "endpoint": "POST /api/v1/ops/dead-letters/{id}/reprocess",
                "access_audit_id": access_audit_id,
                "dead_letter_event_id": dead_letter_event_id,
                "dry_run": dry_run,
                "consumer_names": consumer_names,
                "consumer_groups": consumer_groups,
                "replay_target_topic": replay_target_topic,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(OpsDeadLetterReprocessView {
        dead_letter: DeadLetterEventView::from(&dead_letter),
        dry_run,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
        status: "dry_run_ready".to_string(),
        consumer_names,
        consumer_groups,
        replay_target_topic,
        replay_plan,
    }))
}

pub(in crate::modules::audit) async fn export_audit_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuditPackageExportRequest>,
) -> Result<Json<ApiResponse<AuditPackageExportView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    let normalized_ref_type = normalize_export_ref_type(&payload.ref_type, &request_id)?;
    validate_uuid(&payload.ref_id, "ref_id", &request_id)?;
    let reason = normalize_reason(&payload.reason, &request_id)?;
    let masked_level = resolve_masked_level(payload.masked_level.as_deref(), &request_id)?;
    require_permission(
        &headers,
        AuditPermission::PackageExport,
        "audit package export",
    )?;
    ensure_step_up_header_present(&headers, &request_id)?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up =
        require_step_up_for_export(&client, &headers, &request_id, actor_user_id.as_str()).await?;
    let target =
        load_export_target(&client, &normalized_ref_type, &payload.ref_id, &request_id).await?;
    let export_requested_at = current_utc_timestamp(&client).await?;
    let trace_page = repo::search_audit_traces(
        &client,
        &AuditTraceQuery {
            order_id: target.order_id().map(ToString::to_string),
            page: Some(1),
            page_size: Some(1000),
            ..Default::default()
        },
        1000,
        0,
    )
    .await
    .map_err(map_db_error)?;
    let related_cases = load_related_cases_json(&client, target.order_id())
        .await
        .map_err(map_db_error)?;
    let evidence_manifests = load_evidence_manifests_json(
        &client,
        normalized_ref_type.as_str(),
        payload.ref_id.as_str(),
        masked_level.as_str(),
    )
    .await
    .map_err(map_db_error)?;
    let evidence_items = load_evidence_items_json(
        &client,
        normalized_ref_type.as_str(),
        payload.ref_id.as_str(),
        masked_level.as_str(),
    )
    .await
    .map_err(map_db_error)?;
    let legacy_evidence_refs =
        load_legacy_evidence_refs_json(&client, &target, masked_level.as_str())
            .await
            .map_err(map_db_error)?;
    let legal_holds = load_legal_holds_json(&client, &target)
        .await
        .map_err(map_db_error)?;
    let legal_hold_status = derive_legal_hold_status(&legal_holds);
    let export_id = next_uuid(&client).await?;
    let package_type = payload
        .package_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| target.default_package_type().to_string());
    let export_payload = build_export_payload(
        &export_id,
        &package_type,
        &target,
        masked_level.as_str(),
        &reason,
        export_requested_at.as_str(),
        &headers,
        &step_up,
        &trace_page.items,
        trace_page.total,
        &related_cases,
        &evidence_manifests,
        &evidence_items,
        &legacy_evidence_refs,
        &legal_holds,
    )?;
    let package_bytes = serde_json::to_vec_pretty(&export_payload).map_err(|err| {
        internal_error(
            Some(request_id.clone()),
            format!("audit package payload serialization failed: {err}"),
        )
    })?;
    let package_digest = sha256_hex(package_bytes.as_slice());
    let bucket_name = export_bucket_name();
    let object_key = format!(
        "exports/{}/{}/package-{}.json",
        normalized_ref_type, payload.ref_id, export_id
    );
    let storage_uri = format!("s3://{bucket_name}/{object_key}");
    put_object_bytes(
        &bucket_name,
        &object_key,
        package_bytes.clone(),
        Some("application/json"),
    )
    .await
    .map_err(|(status, Json(mut error))| {
        error.request_id = Some(request_id.clone());
        error.message = format!("audit package upload failed: {}", error.message);
        (status, Json(error))
    })?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let export_result = async {
        let snapshot = record_evidence_snapshot(
            &tx,
            &EvidenceWriteCommand {
                item_type: "audit_export_package".to_string(),
                ref_type: normalized_ref_type.clone(),
                ref_id: Some(payload.ref_id.clone()),
                object_uri: storage_uri.clone(),
                object_hash: package_digest.clone(),
                content_type: Some("application/json".to_string()),
                size_bytes: Some(package_bytes.len() as i64),
                source_system: "audit.export".to_string(),
                storage_mode: "minio".to_string(),
                retention_policy_id: None,
                worm_enabled: false,
                legal_hold_status: legal_hold_status.clone(),
                created_by: Some(actor_user_id.clone()),
                metadata: json!({
                    "package_type": package_type,
                    "masked_level": masked_level,
                    "reason": reason,
                    "export_scope": normalized_ref_type,
                    "step_up_token_present": step_up.token_present,
                    "export_requested_at": export_requested_at,
                    "export_id": export_id,
                }),
                manifest_scope: "audit_export_package".to_string(),
                manifest_ref_type: normalized_ref_type.clone(),
                manifest_ref_id: Some(payload.ref_id.clone()),
                manifest_storage_uri: Some(storage_uri.clone()),
                manifest_metadata: json!({
                    "package_type": package_type,
                    "export_scope": normalized_ref_type,
                    "export_id": export_id,
                    "reason": reason,
                }),
                legacy_bridge: None,
            },
        )
        .await
        .map_err(map_db_error)?;

        let evidence_package = repo::insert_evidence_package(
            &tx,
            &EvidencePackage {
                evidence_package_id: Some(export_id.clone()),
                package_type: package_type.clone(),
                ref_type: normalized_ref_type.clone(),
                ref_id: Some(payload.ref_id.clone()),
                evidence_manifest_id: snapshot.evidence_manifest.evidence_manifest_id.clone(),
                package_digest: Some(package_digest.clone()),
                storage_uri: Some(storage_uri.clone()),
                created_by: Some(actor_user_id.clone()),
                created_at: None,
                retention_class: "audit_default".to_string(),
                legal_hold_status: legal_hold_status.clone(),
                metadata: json!({
                    "reason": reason,
                    "masked_level": masked_level,
                    "access_mode": "export",
                    "export_scope": normalized_ref_type,
                    "request_id": request_id,
                    "trace_id": header(&headers, "x-trace-id"),
                    "step_up_challenge_id": step_up.challenge_id,
                    "step_up_token_present": step_up.token_present,
                    "audit_trace_count": trace_page.total,
                    "evidence_item_count": count_json_items(&evidence_items),
                    "export_requested_at": export_requested_at,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;

        let audit_event = build_export_audit_event(
            &headers,
            &request_id,
            header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone()),
            actor_user_id.as_str(),
            normalized_ref_type.as_str(),
            payload.ref_id.as_str(),
            snapshot.evidence_manifest.evidence_manifest_id.clone(),
            reason.as_str(),
            masked_level.as_str(),
            storage_uri.as_str(),
            legal_hold_status.as_str(),
            step_up.challenge_id.clone(),
            step_up.token_present,
        );
        repo::insert_audit_event(&tx, &audit_event)
            .await
            .map_err(map_db_error)?;

        let access_audit_id = repo::record_access_audit(
            &tx,
            &AccessAuditInsert {
                accessor_user_id: Some(actor_user_id.clone()),
                accessor_role_key: Some(current_role(&headers)),
                access_mode: "export".to_string(),
                target_type: normalized_ref_type.clone(),
                target_id: Some(payload.ref_id.clone()),
                masked_view: masked_level != "unmasked",
                breakglass_reason: None,
                step_up_challenge_id: step_up.challenge_id.clone(),
                request_id: Some(request_id.clone()),
                trace_id: header(&headers, "x-trace-id"),
                metadata: json!({
                    "endpoint": "POST /api/v1/audit/packages/export",
                    "reason": reason,
                    "masked_level": masked_level,
                    "package_type": package_type,
                    "evidence_package_id": evidence_package.evidence_package_id,
                    "storage_uri": storage_uri,
                    "step_up_token_present": step_up.token_present,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;

        repo::record_system_log(
            &tx,
            &SystemLogInsert {
                service_name: "platform-core".to_string(),
                log_level: "INFO".to_string(),
                request_id: Some(request_id.clone()),
                trace_id: header(&headers, "x-trace-id"),
                message_text: "audit package export executed: POST /api/v1/audit/packages/export"
                    .to_string(),
                structured_payload: json!({
                    "module": "audit",
                    "endpoint": "POST /api/v1/audit/packages/export",
                    "evidence_package_id": evidence_package.evidence_package_id,
                    "evidence_manifest_id": snapshot.evidence_manifest.evidence_manifest_id,
                    "access_audit_id": access_audit_id,
                    "ref_type": normalized_ref_type,
                    "ref_id": payload.ref_id,
                    "masked_level": masked_level,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;

        Ok::<_, (StatusCode, Json<ErrorResponse>)>((evidence_package, snapshot))
    }
    .await;

    let (evidence_package, snapshot) = match export_result {
        Ok(value) => value,
        Err(err) => {
            let _ = delete_object(&bucket_name, &object_key).await;
            return Err(err);
        }
    };

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(AuditPackageExportView {
        evidence_package: EvidencePackageView::from(&evidence_package),
        evidence_manifest: EvidenceManifestView::from(&snapshot.evidence_manifest),
        audit_trace_count: trace_page.total,
        evidence_item_count: count_json_items(&evidence_items),
        legal_hold_status,
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
    }))
}

pub(in crate::modules::audit) async fn create_audit_replay_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuditReplayJobCreateRequest>,
) -> Result<Json<ApiResponse<AuditReplayJobDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    let replay_type = normalize_replay_type(&payload.replay_type, &request_id)?;
    let ref_type = normalize_replay_ref_type(&payload.ref_type, &request_id)?;
    validate_uuid(&payload.ref_id, "ref_id", &request_id)?;
    let reason = normalize_replay_reason(&payload.reason, &request_id)?;
    let dry_run = payload.dry_run.unwrap_or(true);
    require_permission(
        &headers,
        AuditPermission::ReplayExecute,
        "audit replay execute",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "audit replay")?;
    if !dry_run {
        return Err(replay_dry_run_only(&request_id));
    }

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_replay(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        ref_type.as_str(),
        payload.ref_id.as_str(),
    )
    .await?;
    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let replay_job_id = next_uuid(&client).await?;
    let executed_at = current_utc_timestamp(&client).await?;
    let target = load_replay_target(&client, &ref_type, &payload.ref_id, &request_id).await?;
    let replay_scope = replay_scope_for_target(&target);
    let trace_page = load_replay_trace_page(&client, &target)
        .await
        .map_err(map_db_error)?;
    let evidence_manifests = load_replay_evidence_manifests_json(&client, &target, "summary")
        .await
        .map_err(map_db_error)?;
    let evidence_items = load_replay_evidence_items_json(&client, &target, "summary")
        .await
        .map_err(map_db_error)?;
    let legal_holds = load_replay_legal_holds_json(&client, &target)
        .await
        .map_err(map_db_error)?;
    let legal_hold_status = derive_legal_hold_status(&legal_holds);
    let report = build_replay_report(
        &replay_job_id,
        replay_type.as_str(),
        &target,
        &reason,
        executed_at.as_str(),
        &headers,
        &step_up,
        &trace_page.items,
        trace_page.total,
        &evidence_manifests,
        &evidence_items,
        &legal_holds,
        payload.options.clone(),
    )?;
    let report_bytes = serde_json::to_vec_pretty(&report.payload).map_err(|err| {
        internal_error(
            Some(request_id.clone()),
            format!("audit replay report serialization failed: {err}"),
        )
    })?;
    let report_digest = sha256_hex(report_bytes.as_slice());
    let bucket_name = export_bucket_name();
    let object_key = format!(
        "replays/{}/{}/replay-{}.json",
        target.subject_ref_type(),
        target.subject_ref_id(),
        replay_job_id
    );
    let storage_uri = format!("s3://{bucket_name}/{object_key}");
    put_object_bytes(
        &bucket_name,
        &object_key,
        report_bytes.clone(),
        Some("application/json"),
    )
    .await
    .map_err(|(status, Json(mut error))| {
        error.request_id = Some(request_id.clone());
        error.message = format!("audit replay report upload failed: {}", error.message);
        (status, Json(error))
    })?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let replay_result = async {
        let replay_job = repo::insert_replay_job(
            &tx,
            &ReplayJob {
                replay_job_id: Some(replay_job_id.clone()),
                replay_type: replay_type.clone(),
                ref_type: target.subject_ref_type().to_string(),
                ref_id: Some(target.subject_ref_id().to_string()),
                dry_run,
                status: "completed".to_string(),
                requested_by: Some(actor_user_id.clone()),
                step_up_challenge_id: step_up.challenge_id.clone(),
                request_reason: Some(reason.clone()),
                options_json: json!({
                    "requested_endpoint": "POST /api/v1/audit/replay-jobs",
                    "request_id": request_id,
                    "trace_id": trace_id,
                    "target_snapshot": target.snapshot_json(),
                    "step_up_token_present": step_up.token_present,
                    "report_storage_uri": storage_uri,
                    "report_digest": report_digest,
                    "result_count": report.results.len(),
                    "recommendation": report.recommendation,
                    "replay_scope": replay_scope,
                    "user_options": payload.options,
                }),
                created_at: None,
                started_at: Some(executed_at.clone()),
                finished_at: Some(executed_at.clone()),
                updated_at: None,
            },
        )
        .await
        .map_err(map_db_error)?;

        let mut stored_results = Vec::with_capacity(report.results.len());
        for replay_result in report.results.iter() {
            stored_results.push(
                repo::insert_replay_result(&tx, replay_result)
                    .await
                    .map_err(map_db_error)?,
            );
        }

        let replay_snapshot = record_evidence_snapshot(
            &tx,
            &EvidenceWriteCommand {
                item_type: "audit_replay_report".to_string(),
                ref_type: "replay_job".to_string(),
                ref_id: Some(replay_job_id.clone()),
                object_uri: storage_uri.clone(),
                object_hash: report_digest.clone(),
                content_type: Some("application/json".to_string()),
                size_bytes: Some(report_bytes.len() as i64),
                source_system: "audit.replay".to_string(),
                storage_mode: "minio".to_string(),
                retention_policy_id: None,
                worm_enabled: false,
                legal_hold_status: legal_hold_status.clone(),
                created_by: Some(actor_user_id.clone()),
                metadata: json!({
                    "replay_type": replay_type,
                    "request_id": request_id,
                    "trace_id": trace_id,
                    "reason": reason,
                    "report_storage_uri": storage_uri,
                    "report_digest": report_digest,
                    "target_ref_type": target.subject_ref_type(),
                    "target_ref_id": target.subject_ref_id(),
                    "dry_run": dry_run,
                    "replay_scope": replay_scope,
                }),
                manifest_scope: "audit_replay_report".to_string(),
                manifest_ref_type: "replay_job".to_string(),
                manifest_ref_id: Some(replay_job_id.clone()),
                manifest_storage_uri: Some(storage_uri.clone()),
                manifest_metadata: json!({
                    "replay_type": replay_type,
                    "target_ref_type": target.subject_ref_type(),
                    "target_ref_id": target.subject_ref_id(),
                    "request_id": request_id,
                    "result_count": stored_results.len(),
                    "replay_scope": replay_scope,
                }),
                legacy_bridge: None,
            },
        )
        .await
        .map_err(map_db_error)?;

        let requested_event = build_replay_audit_event(
            &headers,
            &request_id,
            trace_id.clone(),
            actor_user_id.as_str(),
            replay_job_id.as_str(),
            replay_snapshot
                .evidence_manifest
                .evidence_manifest_id
                .clone(),
            "audit.replay.requested",
            "accepted",
            reason.as_str(),
            replay_type.as_str(),
            target.subject_ref_type(),
            target.subject_ref_id(),
            dry_run,
            step_up.challenge_id.clone(),
            step_up.token_present,
        );
        repo::insert_audit_event(&tx, &requested_event)
            .await
            .map_err(map_db_error)?;

        let completed_event = build_replay_audit_event(
            &headers,
            &request_id,
            trace_id.clone(),
            actor_user_id.as_str(),
            replay_job_id.as_str(),
            replay_snapshot
                .evidence_manifest
                .evidence_manifest_id
                .clone(),
            "audit.replay.completed",
            "success",
            reason.as_str(),
            replay_type.as_str(),
            target.subject_ref_type(),
            target.subject_ref_id(),
            dry_run,
            step_up.challenge_id.clone(),
            step_up.token_present,
        );
        repo::insert_audit_event(&tx, &completed_event)
            .await
            .map_err(map_db_error)?;

        let access_audit_id = repo::record_access_audit(
            &tx,
            &AccessAuditInsert {
                accessor_user_id: Some(actor_user_id.clone()),
                accessor_role_key: Some(current_role(&headers)),
                access_mode: "replay".to_string(),
                target_type: "replay_job".to_string(),
                target_id: Some(replay_job_id.clone()),
                masked_view: true,
                breakglass_reason: None,
                step_up_challenge_id: step_up.challenge_id.clone(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                metadata: json!({
                    "endpoint": "POST /api/v1/audit/replay-jobs",
                    "reason": reason,
                    "replay_type": replay_type,
                    "target_ref_type": target.subject_ref_type(),
                    "target_ref_id": target.subject_ref_id(),
                    "dry_run": dry_run,
                    "report_storage_uri": storage_uri,
                    "step_up_token_present": step_up.token_present,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;

        repo::record_system_log(
            &tx,
            &SystemLogInsert {
                service_name: "platform-core".to_string(),
                log_level: "INFO".to_string(),
                request_id: Some(request_id.clone()),
                trace_id: Some(trace_id.clone()),
                message_text: "audit replay job executed: POST /api/v1/audit/replay-jobs"
                    .to_string(),
                structured_payload: json!({
                    "module": "audit",
                    "endpoint": "POST /api/v1/audit/replay-jobs",
                    "replay_job_id": replay_job_id,
                    "access_audit_id": access_audit_id,
                    "target_ref_type": target.subject_ref_type(),
                    "target_ref_id": target.subject_ref_id(),
                    "result_count": stored_results.len(),
                    "dry_run": dry_run,
                }),
            },
        )
        .await
        .map_err(map_db_error)?;

        Ok::<_, (StatusCode, Json<ErrorResponse>)>((replay_job, stored_results))
    }
    .await;

    let (replay_job, stored_results) = match replay_result {
        Ok(value) => value,
        Err(err) => {
            let _ = delete_object(&bucket_name, &object_key).await;
            return Err(err);
        }
    };

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(AuditReplayJobDetailView {
        replay_job: ReplayJobView::from(&replay_job),
        results: stored_results.iter().map(ReplayResultView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn get_audit_replay_job(
    State(state): State<AppState>,
    Path(replay_job_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AuditReplayJobDetailView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&replay_job_id, "replay_job_id", &request_id)?;
    require_permission(&headers, AuditPermission::ReplayRead, "audit replay read")?;

    let client = state_client(&state)?;
    let detail = repo::load_replay_job_detail(&client, &replay_job_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit replay job not found: {replay_job_id}"),
            )
        })?;

    record_replay_lookup_side_effects(
        &client,
        &headers,
        detail
            .replay_job
            .replay_job_id
            .as_deref()
            .unwrap_or(&replay_job_id),
        &detail.replay_job,
        detail.results.len() as i64,
    )
    .await?;

    Ok(ApiResponse::ok(AuditReplayJobDetailView {
        replay_job: ReplayJobView::from(&detail.replay_job),
        results: detail.results.iter().map(ReplayResultView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn create_audit_legal_hold(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuditLegalHoldCreateRequest>,
) -> Result<Json<ApiResponse<AuditLegalHoldActionView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    let hold_scope_type = normalize_legal_hold_scope_type(&payload.hold_scope_type, &request_id)?;
    validate_uuid(&payload.hold_scope_id, "hold_scope_id", &request_id)?;
    let reason_code = normalize_legal_hold_reason_code(&payload.reason_code, &request_id)?;
    validate_optional_uuid(
        payload.retention_policy_id.as_deref(),
        "retention_policy_id",
        &request_id,
    )?;
    require_permission(
        &headers,
        AuditPermission::LegalHoldManage,
        "audit legal hold manage",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "audit legal hold manage")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_legal_hold_create(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        hold_scope_type.as_str(),
        payload.hold_scope_id.as_str(),
    )
    .await?;
    let hold_until =
        normalize_hold_until(&client, payload.hold_until.as_deref(), &request_id).await?;
    let retention_policy_id =
        validate_retention_policy(&client, payload.retention_policy_id.as_deref(), &request_id)
            .await?;
    let target = load_export_target(
        &client,
        hold_scope_type.as_str(),
        payload.hold_scope_id.as_str(),
        &request_id,
    )
    .await?;
    if repo::load_active_legal_hold_for_scope(
        &client,
        hold_scope_type.as_str(),
        payload.hold_scope_id.as_str(),
    )
    .await
    .map_err(map_db_error)?
    .is_some()
    {
        return Err(conflict_error(
            &request_id,
            LEGAL_HOLD_ACTIVE_ERROR,
            format!(
                "active legal hold already exists for {}/{}",
                hold_scope_type, payload.hold_scope_id
            ),
        ));
    }

    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let legal_hold_id = next_uuid(&client).await?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let mut metadata = build_legal_hold_create_metadata(
        &headers,
        &request_id,
        trace_id.as_str(),
        target.snapshot_json(),
        payload.metadata,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );
    annotate_legal_hold_metadata(&mut metadata, &target);
    let created_hold = repo::insert_legal_hold(
        &tx,
        &LegalHold {
            legal_hold_id: Some(legal_hold_id.clone()),
            hold_scope_type: hold_scope_type.clone(),
            hold_scope_id: Some(payload.hold_scope_id.clone()),
            reason_code: reason_code.clone(),
            status: "active".to_string(),
            retention_policy_id: retention_policy_id.clone(),
            requested_by: Some(actor_user_id.clone()),
            approved_by: None,
            hold_until,
            created_at: None,
            released_at: None,
            updated_at: None,
            metadata,
        },
    )
    .await
    .map_err(map_db_error)?;

    let audit_event = build_legal_hold_audit_event(
        &headers,
        &request_id,
        trace_id.clone(),
        actor_user_id.as_str(),
        hold_scope_type.as_str(),
        payload.hold_scope_id.as_str(),
        "audit.legal_hold.create",
        "success",
        "active",
        Some(legal_hold_id.clone()),
        step_up.challenge_id.clone(),
        step_up.token_present,
        json!({
            "reason_code": reason_code,
            "retention_policy_id": retention_policy_id,
            "hold_until": created_hold.hold_until.clone(),
            "target_snapshot": target.snapshot_json(),
        }),
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text: "audit legal hold created: POST /api/v1/audit/legal-holds".to_string(),
            structured_payload: json!({
                "module": "audit",
                "endpoint": "POST /api/v1/audit/legal-holds",
                "legal_hold_id": legal_hold_id,
                "hold_scope_type": hold_scope_type,
                "hold_scope_id": payload.hold_scope_id,
                "step_up_challenge_id": step_up.challenge_id,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(AuditLegalHoldActionView {
        legal_hold: LegalHoldView::from(&created_hold),
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
    }))
}

pub(in crate::modules::audit) async fn release_audit_legal_hold(
    State(state): State<AppState>,
    Path(legal_hold_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AuditLegalHoldReleaseRequest>,
) -> Result<Json<ApiResponse<AuditLegalHoldActionView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&legal_hold_id, "legal_hold_id", &request_id)?;
    let release_reason = normalize_legal_hold_release_reason(&payload.reason, &request_id)?;
    require_permission(
        &headers,
        AuditPermission::LegalHoldManage,
        "audit legal hold manage",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "audit legal hold release")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_legal_hold_release(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        legal_hold_id.as_str(),
    )
    .await?;
    let existing_hold = repo::load_legal_hold(&client, &legal_hold_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit legal hold not found: {legal_hold_id}"),
            )
        })?;
    if existing_hold.status != "active" {
        return Err(bad_request(
            &request_id,
            format!("audit legal hold is not active: {legal_hold_id}"),
        ));
    }

    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let released_at = current_utc_timestamp(&client).await?;
    let target_snapshot = load_legal_hold_target_snapshot(
        &client,
        existing_hold.hold_scope_type.as_str(),
        existing_hold.hold_scope_id.as_deref(),
    )
    .await;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let metadata_patch = build_legal_hold_release_metadata(
        &headers,
        &request_id,
        trace_id.as_str(),
        released_at.as_str(),
        release_reason.as_str(),
        target_snapshot.clone(),
        payload.metadata,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );
    let released_hold = repo::release_legal_hold(
        &tx,
        legal_hold_id.as_str(),
        Some(actor_user_id.as_str()),
        released_at.as_str(),
        &metadata_patch,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        conflict_error(
            &request_id,
            LEGAL_HOLD_ACTIVE_ERROR,
            format!("audit legal hold is no longer active: {legal_hold_id}"),
        )
    })?;

    let audit_event = build_legal_hold_audit_event(
        &headers,
        &request_id,
        trace_id.clone(),
        actor_user_id.as_str(),
        released_hold.hold_scope_type.as_str(),
        released_hold
            .hold_scope_id
            .as_deref()
            .unwrap_or(&legal_hold_id),
        "audit.legal_hold.release",
        "success",
        "none",
        Some(legal_hold_id.clone()),
        step_up.challenge_id.clone(),
        step_up.token_present,
        json!({
            "release_reason": release_reason,
            "released_at": released_at,
            "target_snapshot": target_snapshot,
            "previous_reason_code": existing_hold.reason_code,
        }),
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text: "audit legal hold released: POST /api/v1/audit/legal-holds/{id}/release"
                .to_string(),
            structured_payload: json!({
                "module": "audit",
                "endpoint": "POST /api/v1/audit/legal-holds/{id}/release",
                "legal_hold_id": legal_hold_id,
                "hold_scope_type": released_hold.hold_scope_type,
                "hold_scope_id": released_hold.hold_scope_id,
                "released_at": released_at,
                "step_up_challenge_id": step_up.challenge_id,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(AuditLegalHoldActionView {
        legal_hold: LegalHoldView::from(&released_hold),
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
    }))
}

pub(in crate::modules::audit) async fn get_audit_anchor_batches(
    State(state): State<AppState>,
    Query(query): Query<AnchorBatchQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AnchorBatchPageView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    require_permission(
        &headers,
        AuditPermission::AnchorRead,
        "audit anchor batch read",
    )?;

    let normalized_query = AnchorBatchQuery {
        anchor_status: normalize_optional_anchor_filter(
            query.anchor_status.as_deref(),
            "anchor_status",
            &request_id,
        )?,
        batch_scope: normalize_optional_anchor_filter(
            query.batch_scope.as_deref(),
            "batch_scope",
            &request_id,
        )?,
        chain_id: normalize_optional_anchor_filter(
            query.chain_id.as_deref(),
            "chain_id",
            &request_id,
        )?,
        page: query.page,
        page_size: query.page_size,
    };

    let client = state_client(&state)?;
    let pagination = normalized_query.pagination();
    let page = repo::search_anchor_batches(
        &client,
        &normalized_query,
        pagination.page_size as i64,
        pagination.offset() as i64,
    )
    .await
    .map_err(map_db_error)?;

    record_lookup_side_effects(
        &client,
        &headers,
        "anchor_batch_query",
        None,
        "GET /api/v1/audit/anchor-batches",
        json!({
            "anchor_status": normalized_query.anchor_status,
            "batch_scope": normalized_query.batch_scope,
            "chain_id": normalized_query.chain_id,
            "page": pagination.page,
            "page_size": pagination.page_size,
            "result_total": page.total,
        }),
    )
    .await?;

    Ok(ApiResponse::ok(AnchorBatchPageView {
        total: page.total,
        page: pagination.page,
        page_size: pagination.page_size,
        items: page.items.iter().map(AnchorBatchView::from).collect(),
    }))
}

pub(in crate::modules::audit) async fn retry_audit_anchor_batch(
    State(state): State<AppState>,
    Path(anchor_batch_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AuditAnchorBatchRetryRequest>,
) -> Result<Json<ApiResponse<AuditAnchorBatchRetryView>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = require_request_id(&headers)?;
    validate_uuid(&anchor_batch_id, "anchor_batch_id", &request_id)?;
    let reason = normalize_anchor_retry_reason(&payload.reason, &request_id)?;
    require_permission(
        &headers,
        AuditPermission::AnchorManage,
        "audit anchor batch retry",
    )?;
    ensure_step_up_header_present_for(&headers, &request_id, "audit anchor batch retry")?;

    let client = state_client(&state)?;
    let actor_user_id = require_user_id(&headers, &request_id)?;
    let step_up = require_step_up_for_anchor_retry(
        &client,
        &headers,
        &request_id,
        actor_user_id.as_str(),
        anchor_batch_id.as_str(),
    )
    .await?;
    let existing_batch = repo::load_anchor_batch(&client, &anchor_batch_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit anchor batch not found: {anchor_batch_id}"),
            )
        })?;
    if existing_batch.status != "failed" {
        return Err(conflict_error(
            &request_id,
            ANCHOR_BATCH_NOT_RETRYABLE_ERROR,
            format!(
                "audit anchor batch is not retryable from status `{}`",
                existing_batch.status
            ),
        ));
    }

    let trace_id = header(&headers, "x-trace-id").unwrap_or_else(|| request_id.clone());
    let retried_at = current_utc_timestamp(&client).await?;
    let retry_metadata = build_anchor_retry_metadata(
        &headers,
        &request_id,
        trace_id.as_str(),
        retried_at.as_str(),
        reason.as_str(),
        &existing_batch,
        payload.metadata,
        step_up.challenge_id.clone(),
        step_up.token_present,
    );

    let tx = client.transaction().await.map_err(map_db_error)?;
    let updated = repo::mark_anchor_batch_retry_requested(
        &tx,
        anchor_batch_id.as_str(),
        &retry_metadata,
        retried_at.as_str(),
    )
    .await
    .map_err(map_db_error)?;
    if !updated {
        return Err(conflict_error(
            &request_id,
            ANCHOR_BATCH_NOT_RETRYABLE_ERROR,
            format!("audit anchor batch is no longer retryable: {anchor_batch_id}"),
        ));
    }

    let refreshed_batch = repo::load_anchor_batch(&tx, &anchor_batch_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                &request_id,
                format!("audit anchor batch not found after retry update: {anchor_batch_id}"),
            )
        })?;

    write_canonical_outbox_event(
        &tx,
        CanonicalOutboxWrite {
            aggregate_type: "audit.anchor_batch",
            aggregate_id: anchor_batch_id.as_str(),
            event_type: "audit.anchor_requested",
            producer_service: "platform-core.audit",
            request_id: Some(request_id.as_str()),
            trace_id: Some(trace_id.as_str()),
            idempotency_key: None,
            occurred_at: Some(retried_at.as_str()),
            business_payload: &build_anchor_retry_outbox_payload(
                &existing_batch,
                &refreshed_batch,
                reason.as_str(),
                actor_user_id.as_str(),
                request_id.as_str(),
                trace_id.as_str(),
                step_up.challenge_id.clone(),
                step_up.token_present,
            ),
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(map_db_error)?;

    let audit_event = build_anchor_retry_audit_event(
        &headers,
        &request_id,
        trace_id.clone(),
        actor_user_id.as_str(),
        anchor_batch_id.as_str(),
        existing_batch.status.as_str(),
        refreshed_batch.status.as_str(),
        reason.as_str(),
        step_up.challenge_id.clone(),
        step_up.token_present,
        refreshed_batch.clone(),
    );
    repo::insert_audit_event(&tx, &audit_event)
        .await
        .map_err(map_db_error)?;

    let access_audit_id = repo::record_access_audit(
        &tx,
        &AccessAuditInsert {
            accessor_user_id: Some(actor_user_id.clone()),
            accessor_role_key: Some(current_role(&headers)),
            access_mode: "retry".to_string(),
            target_type: "anchor_batch".to_string(),
            target_id: Some(anchor_batch_id.clone()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: step_up.challenge_id.clone(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            metadata: json!({
                "endpoint": "POST /api/v1/audit/anchor-batches/{id}/retry",
                "reason": reason,
                "previous_status": existing_batch.status,
                "anchor_status": refreshed_batch.status,
                "batch_scope": refreshed_batch.batch_scope,
                "chain_id": refreshed_batch.chain_id,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        &tx,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: Some(trace_id.clone()),
            message_text:
                "audit anchor batch retry requested: POST /api/v1/audit/anchor-batches/{id}/retry"
                    .to_string(),
            structured_payload: json!({
                "module": "audit",
                "endpoint": "POST /api/v1/audit/anchor-batches/{id}/retry",
                "anchor_batch_id": anchor_batch_id,
                "access_audit_id": access_audit_id,
                "previous_status": existing_batch.status,
                "anchor_status": refreshed_batch.status,
                "chain_id": refreshed_batch.chain_id,
                "batch_scope": refreshed_batch.batch_scope,
                "step_up_challenge_id": step_up.challenge_id,
                "step_up_token_present": step_up.token_present,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiResponse::ok(AuditAnchorBatchRetryView {
        anchor_batch: AnchorBatchView::from(&refreshed_batch),
        step_up_bound: step_up.challenge_id.is_some() || step_up.token_present,
    }))
}

#[derive(Debug, Clone)]
struct StepUpBinding {
    challenge_id: Option<String>,
    token_present: bool,
}

#[derive(Debug, Clone)]
enum ExportTarget {
    Order {
        order_id: String,
        buyer_org_id: String,
        seller_org_id: String,
        status: String,
        payment_status: String,
        delivery_status: String,
        acceptance_status: String,
        settlement_status: String,
        dispute_status: String,
    },
    DisputeCase {
        case_id: String,
        order_id: String,
        status: String,
        reason_code: String,
        decision_code: Option<String>,
        opened_at: String,
        resolved_at: Option<String>,
    },
}

impl ExportTarget {
    fn default_package_type(&self) -> &'static str {
        match self {
            ExportTarget::Order { .. } => "order_evidence_package",
            ExportTarget::DisputeCase { .. } => "case_evidence_package",
        }
    }

    fn order_id(&self) -> Option<&str> {
        match self {
            ExportTarget::Order { order_id, .. } => Some(order_id.as_str()),
            ExportTarget::DisputeCase { order_id, .. } => Some(order_id.as_str()),
        }
    }

    fn snapshot_json(&self) -> Value {
        match self {
            ExportTarget::Order {
                order_id,
                buyer_org_id,
                seller_org_id,
                status,
                payment_status,
                delivery_status,
                acceptance_status,
                settlement_status,
                dispute_status,
            } => json!({
                "ref_type": "order",
                "order_id": order_id,
                "buyer_org_id": buyer_org_id,
                "seller_org_id": seller_org_id,
                "status": status,
                "payment_status": payment_status,
                "delivery_status": delivery_status,
                "acceptance_status": acceptance_status,
                "settlement_status": settlement_status,
                "dispute_status": dispute_status,
            }),
            ExportTarget::DisputeCase {
                case_id,
                order_id,
                status,
                reason_code,
                decision_code,
                opened_at,
                resolved_at,
            } => json!({
                "ref_type": "dispute_case",
                "case_id": case_id,
                "order_id": order_id,
                "status": status,
                "reason_code": reason_code,
                "decision_code": decision_code,
                "opened_at": opened_at,
                "resolved_at": resolved_at,
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum ReplayTarget {
    Order {
        order_id: String,
        buyer_org_id: String,
        seller_org_id: String,
        status: String,
        payment_status: String,
        delivery_status: String,
        acceptance_status: String,
        settlement_status: String,
        dispute_status: String,
    },
    DisputeCase {
        case_id: String,
        order_id: String,
        status: String,
        reason_code: String,
        decision_code: Option<String>,
        opened_at: String,
        resolved_at: Option<String>,
    },
    EvidencePackage {
        evidence_package_id: String,
        package_type: String,
        ref_type: String,
        ref_id: String,
        evidence_manifest_id: Option<String>,
        package_digest: Option<String>,
        storage_uri: Option<String>,
        retention_class: String,
        legal_hold_status: String,
        created_by: Option<String>,
        created_at: String,
        order_id: Option<String>,
    },
    GenericAuditObject {
        ref_type: String,
        ref_id: String,
        latest_audit_id: String,
        latest_action_name: String,
        latest_result_code: String,
        latest_request_id: Option<String>,
        latest_trace_id: Option<String>,
        latest_occurred_at: String,
        audit_event_count: i64,
    },
}

impl ReplayTarget {
    fn subject_ref_type(&self) -> &str {
        match self {
            ReplayTarget::Order { .. } => "order",
            ReplayTarget::DisputeCase { .. } => "dispute_case",
            ReplayTarget::EvidencePackage { ref_type, .. } => ref_type.as_str(),
            ReplayTarget::GenericAuditObject { ref_type, .. } => ref_type.as_str(),
        }
    }

    fn subject_ref_id(&self) -> &str {
        match self {
            ReplayTarget::Order { order_id, .. } => order_id.as_str(),
            ReplayTarget::DisputeCase { case_id, .. } => case_id.as_str(),
            ReplayTarget::EvidencePackage { ref_id, .. } => ref_id.as_str(),
            ReplayTarget::GenericAuditObject { ref_id, .. } => ref_id.as_str(),
        }
    }

    fn snapshot_json(&self) -> Value {
        match self {
            ReplayTarget::Order {
                order_id,
                buyer_org_id,
                seller_org_id,
                status,
                payment_status,
                delivery_status,
                acceptance_status,
                settlement_status,
                dispute_status,
            } => json!({
                "target_type": "order",
                "order_id": order_id,
                "buyer_org_id": buyer_org_id,
                "seller_org_id": seller_org_id,
                "status": status,
                "payment_status": payment_status,
                "delivery_status": delivery_status,
                "acceptance_status": acceptance_status,
                "settlement_status": settlement_status,
                "dispute_status": dispute_status,
            }),
            ReplayTarget::DisputeCase {
                case_id,
                order_id,
                status,
                reason_code,
                decision_code,
                opened_at,
                resolved_at,
            } => json!({
                "target_type": "dispute_case",
                "case_id": case_id,
                "order_id": order_id,
                "status": status,
                "reason_code": reason_code,
                "decision_code": decision_code,
                "opened_at": opened_at,
                "resolved_at": resolved_at,
            }),
            ReplayTarget::EvidencePackage {
                evidence_package_id,
                package_type,
                ref_type,
                ref_id,
                evidence_manifest_id,
                package_digest,
                storage_uri,
                retention_class,
                legal_hold_status,
                created_by,
                created_at,
                order_id,
            } => json!({
                "target_type": "evidence_package",
                "evidence_package_id": evidence_package_id,
                "package_type": package_type,
                "ref_type": ref_type,
                "ref_id": ref_id,
                "evidence_manifest_id": evidence_manifest_id,
                "package_digest": package_digest,
                "storage_uri": storage_uri,
                "retention_class": retention_class,
                "legal_hold_status": legal_hold_status,
                "created_by": created_by,
                "created_at": created_at,
                "order_id": order_id,
            }),
            ReplayTarget::GenericAuditObject {
                ref_type,
                ref_id,
                latest_audit_id,
                latest_action_name,
                latest_result_code,
                latest_request_id,
                latest_trace_id,
                latest_occurred_at,
                audit_event_count,
            } => json!({
                "target_type": "audit_object",
                "ref_type": ref_type,
                "ref_id": ref_id,
                "latest_audit_id": latest_audit_id,
                "latest_action_name": latest_action_name,
                "latest_result_code": latest_result_code,
                "latest_request_id": latest_request_id,
                "latest_trace_id": latest_trace_id,
                "latest_occurred_at": latest_occurred_at,
                "audit_event_count": audit_event_count,
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct ReplayReport {
    payload: Value,
    recommendation: String,
    results: Vec<ReplayResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::modules::audit) enum AuditPermission {
    DeveloperTraceRead,
    TraceRead,
    OpsObservabilityRead,
    OpsLogQuery,
    OpsLogExport,
    OpsTraceRead,
    OpsAlertRead,
    OpsIncidentRead,
    OpsSloRead,
    OpsTradeMonitorRead,
    OpsOutboxRead,
    OpsDeadLetterRead,
    OpsExternalFactRead,
    OpsExternalFactManage,
    RiskFairnessIncidentRead,
    RiskFairnessIncidentHandle,
    OpsProjectionGapRead,
    OpsProjectionGapManage,
    OpsConsistencyRead,
    OpsConsistencyReconcile,
    OpsDeadLetterReprocess,
    PackageExport,
    ReplayExecute,
    ReplayRead,
    LegalHoldManage,
    AnchorRead,
    AnchorManage,
}

impl AuditPermission {
    pub(in crate::modules::audit) fn permission_code(self) -> &'static str {
        match self {
            AuditPermission::DeveloperTraceRead => "developer.trace.read",
            AuditPermission::TraceRead => "audit.trace.read",
            AuditPermission::OpsObservabilityRead => "ops.observability.read",
            AuditPermission::OpsLogQuery => "ops.log.query",
            AuditPermission::OpsLogExport => "ops.log.export",
            AuditPermission::OpsTraceRead => "ops.trace.read",
            AuditPermission::OpsAlertRead => "ops.alert.read",
            AuditPermission::OpsIncidentRead => "ops.incident.read",
            AuditPermission::OpsSloRead => "ops.slo.read",
            AuditPermission::OpsTradeMonitorRead => "ops.trade_monitor.read",
            AuditPermission::OpsOutboxRead => "ops.outbox.read",
            AuditPermission::OpsDeadLetterRead => "ops.dead_letter.read",
            AuditPermission::OpsExternalFactRead => "ops.external_fact.read",
            AuditPermission::OpsExternalFactManage => "ops.external_fact.manage",
            AuditPermission::RiskFairnessIncidentRead => "risk.fairness_incident.read",
            AuditPermission::RiskFairnessIncidentHandle => "risk.fairness_incident.handle",
            AuditPermission::OpsProjectionGapRead => "ops.projection_gap.read",
            AuditPermission::OpsProjectionGapManage => "ops.projection_gap.manage",
            AuditPermission::OpsConsistencyRead => "ops.consistency.read",
            AuditPermission::OpsConsistencyReconcile => "ops.consistency.reconcile",
            AuditPermission::OpsDeadLetterReprocess => "ops.dead_letter.reprocess",
            AuditPermission::PackageExport => "audit.package.export",
            AuditPermission::ReplayExecute => "audit.replay.execute",
            AuditPermission::ReplayRead => "audit.replay.read",
            AuditPermission::LegalHoldManage => "audit.legal_hold.manage",
            AuditPermission::AnchorRead => "audit.anchor.read",
            AuditPermission::AnchorManage => "audit.anchor.manage",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::modules::audit) fn requires_step_up(self) -> bool {
        matches!(
            self,
            AuditPermission::OpsLogExport
                | AuditPermission::OpsExternalFactManage
                | AuditPermission::RiskFairnessIncidentHandle
                | AuditPermission::OpsProjectionGapManage
                | AuditPermission::OpsConsistencyReconcile
                | AuditPermission::OpsDeadLetterReprocess
                | AuditPermission::PackageExport
                | AuditPermission::ReplayExecute
                | AuditPermission::LegalHoldManage
                | AuditPermission::AnchorManage
        )
    }

    fn allowed_roles(self) -> &'static [&'static str] {
        match self {
            AuditPermission::DeveloperTraceRead => &["tenant_developer", "platform_audit_security"],
            AuditPermission::TraceRead => &[
                "tenant_admin",
                "tenant_audit_readonly",
                "platform_admin",
                "platform_audit_security",
                "platform_reviewer",
                "platform_risk_settlement",
                "regulator_readonly",
            ],
            AuditPermission::OpsObservabilityRead
            | AuditPermission::OpsLogQuery
            | AuditPermission::OpsLogExport
            | AuditPermission::OpsTraceRead
            | AuditPermission::OpsAlertRead
            | AuditPermission::OpsIncidentRead
            | AuditPermission::OpsSloRead => &["platform_audit_security"],
            AuditPermission::OpsTradeMonitorRead => &[
                "tenant_admin",
                "tenant_audit_readonly",
                "platform_admin",
                "platform_audit_security",
                "platform_risk_settlement",
            ],
            AuditPermission::OpsOutboxRead
            | AuditPermission::OpsDeadLetterRead
            | AuditPermission::OpsConsistencyRead
            | AuditPermission::OpsConsistencyReconcile
            | AuditPermission::OpsDeadLetterReprocess
            | AuditPermission::AnchorRead
            | AuditPermission::AnchorManage => &["platform_admin", "platform_audit_security"],
            AuditPermission::OpsExternalFactRead | AuditPermission::RiskFairnessIncidentRead => &[
                "platform_admin",
                "platform_audit_security",
                "platform_risk_settlement",
            ],
            AuditPermission::OpsExternalFactManage
            | AuditPermission::OpsProjectionGapRead
            | AuditPermission::OpsProjectionGapManage => {
                &["platform_admin", "platform_audit_security"]
            }
            AuditPermission::RiskFairnessIncidentHandle => {
                &["platform_admin", "platform_risk_settlement"]
            }
            AuditPermission::PackageExport
            | AuditPermission::ReplayExecute
            | AuditPermission::ReplayRead
            | AuditPermission::LegalHoldManage => &["platform_audit_security"],
        }
    }
}

pub(in crate::modules::audit) fn canonical_role_key(role: &str) -> &str {
    match role {
        "developer_admin" => "tenant_developer",
        "audit_admin" | "platform_auditor" | "consistency_operator" | "node_ops_admin" => {
            "platform_audit_security"
        }
        "subject_reviewer" | "product_reviewer" | "compliance_reviewer" => "platform_reviewer",
        "risk_operator" => "platform_risk_settlement",
        "regulator_observer" => "regulator_readonly",
        "data_custody_admin" => "platform_admin",
        _ => role,
    }
}

pub(in crate::modules::audit) fn is_allowed(role: &str, permission: AuditPermission) -> bool {
    let normalized_role = canonical_role_key(role);
    permission.allowed_roles().contains(&normalized_role)
}

fn is_tenant_scoped_role(role: &str) -> bool {
    matches!(
        canonical_role_key(role),
        "tenant_admin" | "tenant_audit_readonly"
    )
}

fn require_permission(
    headers: &HeaderMap,
    permission: AuditPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if is_allowed(&role, permission) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!(
                "{action} is forbidden for current role ({})",
                permission.permission_code()
            ),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn require_request_id(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    header(headers, "x-request-id").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
                message: "x-request-id is required for audit access".to_string(),
                request_id: None,
            }),
        )
    })
}

fn require_user_id(
    headers: &HeaderMap,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let user_id = header(headers, "x-user-id").ok_or_else(|| {
        bad_request(
            request_id,
            "x-user-id is required for high-risk audit action",
        )
    })?;
    validate_uuid(&user_id, "x-user-id", request_id)?;
    Ok(user_id)
}

fn ensure_step_up_header_present(
    headers: &HeaderMap,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    ensure_step_up_header_present_for(headers, request_id, "audit package export")
}

fn ensure_step_up_header_present_for(
    headers: &HeaderMap,
    request_id: &str,
    action_label: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let challenge_id = header(headers, "x-step-up-challenge-id");
    let token_present = header(headers, "x-step-up-token").is_some();
    if challenge_id.is_none() && !token_present {
        return Err(bad_request(
            request_id,
            format!("x-step-up-token or x-step-up-challenge-id is required for {action_label}"),
        ));
    }
    Ok(())
}

async fn ensure_trace_query_scope(
    client: &db::Client,
    headers: &HeaderMap,
    query: &AuditTraceQuery,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if !is_tenant_scoped_role(&role) {
        return Ok(());
    }

    let Some(order_id) = query.effective_order_id() else {
        return Err(bad_request(
            request_id,
            "tenant-scoped audit trace queries require order_id or ref_type=order + ref_id",
        ));
    };

    let scope = repo::load_order_audit_scope(client, order_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                request_id,
                format!("tenant-scoped audit trace target not found: {order_id}"),
            )
        })?;
    ensure_order_scope(headers, &scope, request_id, "audit trace read")
}

fn ensure_order_scope(
    headers: &HeaderMap,
    scope: &OrderAuditScope,
    request_id: &str,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if !is_tenant_scoped_role(&role) {
        return Ok(());
    }

    let tenant_id = header(headers, "x-tenant-id")
        .ok_or_else(|| bad_request(request_id, "x-tenant-id is required for tenant audit scope"))?;
    if tenant_id == scope.buyer_org_id || tenant_id == scope.seller_org_id {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden outside tenant order scope"),
            request_id: Some(request_id.to_string()),
        }),
    ))
}

async fn require_step_up_for_export(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        EXPORT_STEP_UP_ACTION,
        Some(EXPORT_STEP_UP_ACTION_COMPAT),
        None,
        None,
        "audit package export",
    )
    .await
}

async fn require_step_up_for_replay(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    ref_type: &str,
    ref_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        REPLAY_STEP_UP_ACTION,
        None,
        Some(ref_type),
        Some(ref_id),
        "audit replay",
    )
    .await
}

async fn require_step_up_for_legal_hold_create(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    hold_scope_type: &str,
    hold_scope_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        LEGAL_HOLD_STEP_UP_ACTION,
        None,
        Some(hold_scope_type),
        Some(hold_scope_id),
        "audit legal hold manage",
    )
    .await
}

async fn require_step_up_for_legal_hold_release(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    legal_hold_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        LEGAL_HOLD_STEP_UP_ACTION,
        None,
        Some("legal_hold"),
        Some(legal_hold_id),
        "audit legal hold release",
    )
    .await
}

async fn require_step_up_for_anchor_retry(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    anchor_batch_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        ANCHOR_STEP_UP_ACTION,
        None,
        Some("anchor_batch"),
        Some(anchor_batch_id),
        "audit anchor batch retry",
    )
    .await
}

async fn require_step_up_for_log_export(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    object_id: Option<&str>,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        LOG_EXPORT_STEP_UP_ACTION,
        None,
        Some("system_log_query"),
        object_id,
        "ops log export",
    )
    .await
}

async fn require_step_up_for_dead_letter_reprocess(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    dead_letter_event_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        DEAD_LETTER_REPROCESS_STEP_UP_ACTION,
        None,
        Some("dead_letter_event"),
        Some(dead_letter_event_id),
        "ops dead letter reprocess",
    )
    .await
}

async fn require_step_up_for_consistency_reconcile(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    ref_type: &str,
    ref_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        CONSISTENCY_RECONCILE_STEP_UP_ACTION,
        None,
        Some(ref_type),
        Some(ref_id),
        "ops consistency reconcile",
    )
    .await
}

async fn require_step_up_for_external_fact_confirm(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    external_fact_receipt_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        EXTERNAL_FACT_CONFIRM_STEP_UP_ACTION,
        None,
        Some("external_fact_receipt"),
        Some(external_fact_receipt_id),
        "ops external fact confirm",
    )
    .await
}

async fn require_step_up_for_fairness_incident_handle(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    fairness_incident_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        FAIRNESS_INCIDENT_HANDLE_STEP_UP_ACTION,
        None,
        Some("fairness_incident"),
        Some(fairness_incident_id),
        "risk fairness incident handle",
    )
    .await
}

async fn require_step_up_for_projection_gap_resolve(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    chain_projection_gap_id: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    require_step_up_for_action(
        client,
        headers,
        request_id,
        actor_user_id,
        PROJECTION_GAP_RESOLVE_STEP_UP_ACTION,
        None,
        Some("projection_gap"),
        Some(chain_projection_gap_id),
        "ops projection gap resolve",
    )
    .await
}

async fn require_step_up_for_action(
    client: &db::Client,
    headers: &HeaderMap,
    request_id: &str,
    actor_user_id: &str,
    expected_action: &str,
    compat_action: Option<&str>,
    expected_ref_type: Option<&str>,
    expected_ref_id: Option<&str>,
    action_label: &str,
) -> Result<StepUpBinding, (StatusCode, Json<ErrorResponse>)> {
    let challenge_id = header(headers, "x-step-up-challenge-id");
    let token_present = header(headers, "x-step-up-token").is_some();
    if challenge_id.is_none() && !token_present {
        return Err(bad_request(
            request_id,
            format!("x-step-up-token or x-step-up-challenge-id is required for {action_label}"),
        ));
    }
    if let Some(challenge_id) = challenge_id.clone() {
        let row = client
            .query_opt(
                "SELECT step_up_challenge_id::text,
                        target_action,
                        target_ref_type,
                        target_ref_id::text,
                        challenge_status,
                        user_id::text,
                        (expires_at > now()) AS is_active
                 FROM iam.step_up_challenge
                 WHERE step_up_challenge_id = $1::text::uuid",
                &[&challenge_id],
            )
            .await
            .map_err(map_db_error)?;
        let row = row.ok_or_else(|| {
            not_found(
                request_id,
                format!("step-up challenge not found: {challenge_id}"),
            )
        })?;
        let target_action: String = row.get(1);
        let target_ref_type: Option<String> = row.get(2);
        let target_ref_id: Option<String> = row.get(3);
        let challenge_status: String = row.get(4);
        let user_id: String = row.get(5);
        let is_active: bool = row.get(6);
        if user_id != actor_user_id {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "step-up challenge does not belong to current actor".to_string(),
                    request_id: Some(request_id.to_string()),
                }),
            ));
        }
        if challenge_status != "verified" || !is_active {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: format!("verified step-up challenge is required for {action_label}"),
                    request_id: Some(request_id.to_string()),
                }),
            ));
        }
        let action_matches =
            target_action == expected_action || compat_action == Some(target_action.as_str());
        if !action_matches {
            let expected_description = compat_action
                .map(|compat| format!("`{expected_action}` or `{compat}`"))
                .unwrap_or_else(|| format!("`{expected_action}`"));
            return Err(bad_request(
                request_id,
                format!("step-up challenge target_action must be {expected_description}"),
            ));
        }
        if let Some(expected_ref_type) = expected_ref_type {
            if let Some(target_ref_type) = target_ref_type.as_deref() {
                if target_ref_type != expected_ref_type {
                    return Err(bad_request(
                        request_id,
                        format!("step-up challenge target_ref_type must be `{expected_ref_type}`"),
                    ));
                }
            }
        }
        if let Some(expected_ref_id) = expected_ref_id {
            if let Some(target_ref_id) = target_ref_id.as_deref() {
                if target_ref_id != expected_ref_id {
                    return Err(bad_request(
                        request_id,
                        format!("step-up challenge target_ref_id must be `{expected_ref_id}`"),
                    ));
                }
            }
        }
    }

    Ok(StepUpBinding {
        challenge_id,
        token_present,
    })
}

async fn load_export_target(
    client: &db::Client,
    ref_type: &str,
    ref_id: &str,
    request_id: &str,
) -> Result<ExportTarget, (StatusCode, Json<ErrorResponse>)> {
    match ref_type {
        "order" => {
            let row = client
                .query_opt(
                    "SELECT order_id::text,
                            buyer_org_id::text,
                            seller_org_id::text,
                            status,
                            payment_status,
                            delivery_status,
                            acceptance_status,
                            settlement_status,
                            dispute_status
                     FROM trade.order_main
                     WHERE order_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await
                .map_err(map_db_error)?;
            let row = row.ok_or_else(|| {
                not_found(
                    request_id,
                    format!("audit export target not found: order/{ref_id}"),
                )
            })?;
            Ok(ExportTarget::Order {
                order_id: row.get(0),
                buyer_org_id: row.get(1),
                seller_org_id: row.get(2),
                status: row.get(3),
                payment_status: row.get(4),
                delivery_status: row.get(5),
                acceptance_status: row.get(6),
                settlement_status: row.get(7),
                dispute_status: row.get(8),
            })
        }
        "dispute_case" => {
            let row = client
                .query_opt(
                    "SELECT case_id::text,
                            order_id::text,
                            status,
                            reason_code,
                            decision_code,
                            to_char(opened_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                     FROM support.dispute_case
                     WHERE case_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await
                .map_err(map_db_error)?;
            let row = row.ok_or_else(|| {
                not_found(
                    request_id,
                    format!("audit export target not found: dispute_case/{ref_id}"),
                )
            })?;
            Ok(ExportTarget::DisputeCase {
                case_id: row.get(0),
                order_id: row.get(1),
                status: row.get(2),
                reason_code: row.get(3),
                decision_code: row.get(4),
                opened_at: row.get(5),
                resolved_at: row.get(6),
            })
        }
        _ => Err(bad_request(
            request_id,
            format!("unsupported audit export ref_type: {ref_type}"),
        )),
    }
}

async fn load_replay_target(
    client: &db::Client,
    ref_type: &str,
    ref_id: &str,
    request_id: &str,
) -> Result<ReplayTarget, (StatusCode, Json<ErrorResponse>)> {
    match ref_type {
        "order" => {
            let target = load_export_target(client, "order", ref_id, request_id).await?;
            match target {
                ExportTarget::Order {
                    order_id,
                    buyer_org_id,
                    seller_org_id,
                    status,
                    payment_status,
                    delivery_status,
                    acceptance_status,
                    settlement_status,
                    dispute_status,
                } => Ok(ReplayTarget::Order {
                    order_id,
                    buyer_org_id,
                    seller_org_id,
                    status,
                    payment_status,
                    delivery_status,
                    acceptance_status,
                    settlement_status,
                    dispute_status,
                }),
                ExportTarget::DisputeCase { .. } => {
                    unreachable!("order export target must stay order")
                }
            }
        }
        "dispute_case" => {
            let target = load_export_target(client, "dispute_case", ref_id, request_id).await?;
            match target {
                ExportTarget::DisputeCase {
                    case_id,
                    order_id,
                    status,
                    reason_code,
                    decision_code,
                    opened_at,
                    resolved_at,
                } => Ok(ReplayTarget::DisputeCase {
                    case_id,
                    order_id,
                    status,
                    reason_code,
                    decision_code,
                    opened_at,
                    resolved_at,
                }),
                ExportTarget::Order { .. } => {
                    unreachable!("dispute_case export target must stay case")
                }
            }
        }
        "evidence_package" => {
            let row = client
                .query_opt(
                    "SELECT
                       evidence_package_id::text,
                       package_type,
                       ref_type,
                       ref_id::text,
                       evidence_manifest_id::text,
                       package_digest,
                       storage_uri,
                       retention_class,
                       legal_hold_status,
                       created_by::text,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                     FROM audit.evidence_package
                     WHERE evidence_package_id = $1::text::uuid",
                    &[&ref_id],
                )
                .await
                .map_err(map_db_error)?;
            let row = row.ok_or_else(|| {
                not_found(
                    request_id,
                    format!("audit replay target not found: evidence_package/{ref_id}"),
                )
            })?;
            let subject_ref_type: String = row.get(2);
            let subject_ref_id: Option<String> = row.get(3);
            let subject_ref_id = subject_ref_id.ok_or_else(|| {
                bad_request(
                    request_id,
                    format!("evidence_package/{ref_id} does not bind a replayable ref_id"),
                )
            })?;
            let order_id = if subject_ref_type == "order" {
                Some(subject_ref_id.clone())
            } else if subject_ref_type == "dispute_case" {
                client
                    .query_opt(
                        "SELECT order_id::text
                         FROM support.dispute_case
                         WHERE case_id = $1::text::uuid",
                        &[&subject_ref_id],
                    )
                    .await
                    .map_err(map_db_error)?
                    .map(|row| row.get(0))
            } else {
                None
            };
            Ok(ReplayTarget::EvidencePackage {
                evidence_package_id: row.get(0),
                package_type: row.get(1),
                ref_type: subject_ref_type,
                ref_id: subject_ref_id,
                evidence_manifest_id: row.get(4),
                package_digest: row.get(5),
                storage_uri: row.get(6),
                retention_class: row.get(7),
                legal_hold_status: row.get(8),
                created_by: row.get(9),
                created_at: row.get(10),
                order_id,
            })
        }
        _ => {
            let row = client
                .query_opt(
                    "WITH ranked AS (
                       SELECT
                         audit_id::text,
                         action_name,
                         result_code,
                         request_id,
                         trace_id,
                         to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') AS event_time,
                         COUNT(*) OVER ()::bigint AS total_count,
                         ROW_NUMBER() OVER (ORDER BY event_time DESC, audit_id DESC) AS rn
                       FROM audit.audit_event
                       WHERE ref_type = $1
                         AND ref_id = $2::text::uuid
                     )
                     SELECT audit_id,
                            action_name,
                            result_code,
                            request_id,
                            trace_id,
                            event_time,
                            total_count
                     FROM ranked
                     WHERE rn = 1",
                    &[&ref_type, &ref_id],
                )
                .await
                .map_err(map_db_error)?;
            let row = row.ok_or_else(|| {
                not_found(
                    request_id,
                    format!("audit replay target not found: {ref_type}/{ref_id}"),
                )
            })?;
            Ok(ReplayTarget::GenericAuditObject {
                ref_type: ref_type.to_string(),
                ref_id: ref_id.to_string(),
                latest_audit_id: row.get(0),
                latest_action_name: row.get(1),
                latest_result_code: row.get(2),
                latest_request_id: row.get(3),
                latest_trace_id: row.get(4),
                latest_occurred_at: row.get(5),
                audit_event_count: row.get(6),
            })
        }
    }
}

fn replay_scope_for_target(target: &ReplayTarget) -> &'static str {
    match target {
        ReplayTarget::Order { .. } => "order_timeline",
        ReplayTarget::DisputeCase { .. } => "dispute_timeline",
        ReplayTarget::EvidencePackage { .. } => "evidence_package_projection",
        ReplayTarget::GenericAuditObject { .. } => "audit_object_projection",
    }
}

async fn load_replay_trace_page(
    client: &db::Client,
    target: &ReplayTarget,
) -> Result<repo::AuditTracePage, db::Error> {
    let query = match target {
        ReplayTarget::Order { order_id, .. } => AuditTraceQuery {
            order_id: Some(order_id.clone()),
            page: Some(1),
            page_size: Some(1000),
            ..Default::default()
        },
        ReplayTarget::DisputeCase { case_id, .. } => AuditTraceQuery {
            ref_type: Some("dispute_case".to_string()),
            ref_id: Some(case_id.clone()),
            page: Some(1),
            page_size: Some(1000),
            ..Default::default()
        },
        ReplayTarget::EvidencePackage {
            ref_type, ref_id, ..
        }
        | ReplayTarget::GenericAuditObject {
            ref_type, ref_id, ..
        } => AuditTraceQuery {
            ref_type: Some(ref_type.clone()),
            ref_id: Some(ref_id.clone()),
            page: Some(1),
            page_size: Some(1000),
            ..Default::default()
        },
    };
    repo::search_audit_traces(client, &query, 1000, 0).await
}

async fn load_replay_evidence_manifests_json(
    client: &db::Client,
    target: &ReplayTarget,
    masked_level: &str,
) -> Result<Value, db::Error> {
    load_evidence_manifests_json(
        client,
        target.subject_ref_type(),
        target.subject_ref_id(),
        masked_level,
    )
    .await
}

async fn load_replay_evidence_items_json(
    client: &db::Client,
    target: &ReplayTarget,
    masked_level: &str,
) -> Result<Value, db::Error> {
    load_evidence_items_json(
        client,
        target.subject_ref_type(),
        target.subject_ref_id(),
        masked_level,
    )
    .await
}

async fn load_replay_legal_holds_json(
    client: &db::Client,
    target: &ReplayTarget,
) -> Result<Value, db::Error> {
    match target {
        ReplayTarget::Order { order_id, .. } => {
            let export_target = ExportTarget::Order {
                order_id: order_id.clone(),
                buyer_org_id: String::new(),
                seller_org_id: String::new(),
                status: String::new(),
                payment_status: String::new(),
                delivery_status: String::new(),
                acceptance_status: String::new(),
                settlement_status: String::new(),
                dispute_status: String::new(),
            };
            load_legal_holds_json(client, &export_target).await
        }
        ReplayTarget::DisputeCase {
            case_id, order_id, ..
        } => {
            let export_target = ExportTarget::DisputeCase {
                case_id: case_id.clone(),
                order_id: order_id.clone(),
                status: String::new(),
                reason_code: String::new(),
                decision_code: None,
                opened_at: String::new(),
                resolved_at: None,
            };
            load_legal_holds_json(client, &export_target).await
        }
        ReplayTarget::EvidencePackage {
            ref_type,
            ref_id,
            order_id,
            ..
        } if ref_type == "order" => {
            let export_target = ExportTarget::Order {
                order_id: ref_id.clone(),
                buyer_org_id: String::new(),
                seller_org_id: String::new(),
                status: String::new(),
                payment_status: String::new(),
                delivery_status: String::new(),
                acceptance_status: String::new(),
                settlement_status: String::new(),
                dispute_status: String::new(),
            };
            load_legal_holds_json(client, &export_target).await
        }
        ReplayTarget::EvidencePackage {
            ref_type,
            ref_id,
            order_id,
            ..
        } if ref_type == "dispute_case" && order_id.is_some() => {
            let export_target = ExportTarget::DisputeCase {
                case_id: ref_id.clone(),
                order_id: order_id.clone().unwrap_or_default(),
                status: String::new(),
                reason_code: String::new(),
                decision_code: None,
                opened_at: String::new(),
                resolved_at: None,
            };
            load_legal_holds_json(client, &export_target).await
        }
        _ => Ok(json!([])),
    }
}

async fn current_utc_timestamp(
    client: &db::Client,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "SELECT to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[],
        )
        .await
        .map(|row| row.get(0))
        .map_err(map_db_error)
}

async fn next_uuid(client: &db::Client) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .map(|row| row.get(0))
        .map_err(map_db_error)
}

async fn load_related_cases_json(
    client: &db::Client,
    order_id: Option<&str>,
) -> Result<Value, db::Error> {
    let Some(order_id) = order_id else {
        return Ok(json!([]));
    };
    let row = client
        .query_one(
            "SELECT COALESCE(
                jsonb_agg(
                  jsonb_build_object(
                    'case_id', case_id::text,
                    'status', status,
                    'reason_code', reason_code,
                    'decision_code', decision_code,
                    'opened_at', to_char(opened_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    'resolved_at', to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                  )
                  ORDER BY opened_at DESC
                ),
                '[]'::jsonb
             )
             FROM support.dispute_case
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    Ok(row.get(0))
}

async fn load_evidence_manifests_json(
    client: &db::Client,
    ref_type: &str,
    ref_id: &str,
    masked_level: &str,
) -> Result<Value, db::Error> {
    let row = client
        .query_one(
            "SELECT COALESCE(
                jsonb_agg(
                  jsonb_build_object(
                    'evidence_manifest_id', evidence_manifest_id::text,
                    'manifest_scope', manifest_scope,
                    'ref_type', ref_type,
                    'ref_id', ref_id::text,
                    'manifest_hash', manifest_hash,
                    'item_count', item_count,
                    'storage_uri', CASE WHEN $3 = 'summary' THEN NULL ELSE storage_uri END,
                    'created_by', created_by::text,
                    'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    'metadata', CASE WHEN $3 = 'summary' THEN '{}'::jsonb ELSE metadata END
                  )
                  ORDER BY created_at DESC
                ),
                '[]'::jsonb
             )
             FROM audit.evidence_manifest
             WHERE ref_type = $1
               AND ref_id = $2::text::uuid",
            &[&ref_type, &ref_id, &masked_level],
        )
        .await?;
    Ok(row.get(0))
}

async fn load_evidence_items_json(
    client: &db::Client,
    ref_type: &str,
    ref_id: &str,
    masked_level: &str,
) -> Result<Value, db::Error> {
    let row = client
        .query_one(
            "SELECT COALESCE(
                jsonb_agg(
                  jsonb_build_object(
                    'evidence_item_id', evidence_item_id::text,
                    'item_type', item_type,
                    'ref_type', ref_type,
                    'ref_id', ref_id::text,
                    'object_uri', CASE WHEN $3 = 'summary' THEN NULL ELSE object_uri END,
                    'object_hash', object_hash,
                    'content_type', content_type,
                    'size_bytes', size_bytes,
                    'source_system', source_system,
                    'storage_mode', storage_mode,
                    'retention_policy_id', retention_policy_id::text,
                    'legal_hold_status', legal_hold_status,
                    'created_by', created_by::text,
                    'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    'metadata', CASE WHEN $3 = 'summary' THEN '{}'::jsonb ELSE metadata END
                  )
                  ORDER BY created_at DESC
                ),
                '[]'::jsonb
             )
             FROM audit.evidence_item
             WHERE ref_type = $1
               AND ref_id = $2::text::uuid",
            &[&ref_type, &ref_id, &masked_level],
        )
        .await?;
    Ok(row.get(0))
}

async fn load_legacy_evidence_refs_json(
    client: &db::Client,
    target: &ExportTarget,
    masked_level: &str,
) -> Result<Value, db::Error> {
    match target {
        ExportTarget::DisputeCase { case_id, .. } => {
            let row = client
                .query_one(
                    "SELECT COALESCE(
                        jsonb_agg(
                          jsonb_build_object(
                            'evidence_id', evidence_id::text,
                            'case_id', case_id::text,
                            'object_type', object_type,
                            'object_uri', CASE WHEN $2 = 'summary' THEN NULL ELSE object_uri END,
                            'object_hash', object_hash,
                            'metadata', CASE WHEN $2 = 'summary' THEN '{}'::jsonb ELSE metadata END,
                            'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                          )
                          ORDER BY created_at DESC
                        ),
                        '[]'::jsonb
                     )
                     FROM support.evidence_object
                     WHERE case_id = $1::text::uuid",
                    &[&case_id.as_str(), &masked_level],
                )
                .await?;
            Ok(row.get(0))
        }
        ExportTarget::Order { order_id, .. } => {
            let row = client
                .query_one(
                    "SELECT COALESCE(
                        jsonb_agg(
                          jsonb_build_object(
                            'evidence_id', eo.evidence_id::text,
                            'case_id', eo.case_id::text,
                            'object_type', eo.object_type,
                            'object_uri', CASE WHEN $2 = 'summary' THEN NULL ELSE eo.object_uri END,
                            'object_hash', eo.object_hash,
                            'metadata', CASE WHEN $2 = 'summary' THEN '{}'::jsonb ELSE eo.metadata END,
                            'created_at', to_char(eo.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                          )
                          ORDER BY eo.created_at DESC
                        ),
                        '[]'::jsonb
                     )
                     FROM support.evidence_object eo
                     INNER JOIN support.dispute_case dc
                       ON dc.case_id = eo.case_id
                     WHERE dc.order_id = $1::text::uuid",
                    &[&order_id.as_str(), &masked_level],
                )
                .await?;
            Ok(row.get(0))
        }
    }
}

async fn load_legal_holds_json(
    client: &db::Client,
    target: &ExportTarget,
) -> Result<Value, db::Error> {
    match target {
        ExportTarget::Order { order_id, .. } => {
            let row = client
                .query_one(
                    "SELECT COALESCE(
                        jsonb_agg(
                          jsonb_build_object(
                            'legal_hold_id', legal_hold_id::text,
                            'hold_scope_type', hold_scope_type,
                            'hold_scope_id', hold_scope_id::text,
                            'reason_code', reason_code,
                            'status', status,
                            'requested_by', requested_by::text,
                            'approved_by', approved_by::text,
                            'hold_until', to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'released_at', to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'metadata', metadata
                          )
                          ORDER BY created_at DESC
                        ),
                        '[]'::jsonb
                     )
                     FROM audit.legal_hold
                     WHERE (hold_scope_type = 'order' AND hold_scope_id = $1::text::uuid)
                        OR (hold_scope_type = 'dispute_case' AND metadata ->> 'order_id' = $1)",
                    &[&order_id.as_str()],
                )
                .await?;
            Ok(row.get(0))
        }
        ExportTarget::DisputeCase {
            case_id, order_id, ..
        } => {
            let row = client
                .query_one(
                    "SELECT COALESCE(
                        jsonb_agg(
                          jsonb_build_object(
                            'legal_hold_id', legal_hold_id::text,
                            'hold_scope_type', hold_scope_type,
                            'hold_scope_id', hold_scope_id::text,
                            'reason_code', reason_code,
                            'status', status,
                            'requested_by', requested_by::text,
                            'approved_by', approved_by::text,
                            'hold_until', to_char(hold_until AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'released_at', to_char(released_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                            'metadata', metadata
                          )
                          ORDER BY created_at DESC
                        ),
                        '[]'::jsonb
                     )
                     FROM audit.legal_hold
                     WHERE (hold_scope_type = 'order' AND hold_scope_id = $1::text::uuid)
                        OR (hold_scope_type = 'dispute_case' AND hold_scope_id = $2::text::uuid)
                        OR metadata ->> 'case_id' = $2",
                    &[&order_id.as_str(), &case_id.as_str()],
                )
                .await?;
            Ok(row.get(0))
        }
    }
}

fn derive_legal_hold_status(legal_holds: &Value) -> String {
    let active = legal_holds
        .as_array()
        .map(|items| {
            items.iter().any(|item| {
                item.get("status")
                    .and_then(Value::as_str)
                    .map(|status| status == "active")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    if active {
        "active".to_string()
    } else {
        "none".to_string()
    }
}

async fn load_legal_hold_target_snapshot(
    client: &db::Client,
    hold_scope_type: &str,
    hold_scope_id: Option<&str>,
) -> Value {
    let Some(hold_scope_id) = hold_scope_id else {
        return json!({
            "ref_type": hold_scope_type,
            "ref_id": Value::Null,
        });
    };
    match hold_scope_type {
        "order" | "dispute_case" => {
            match load_export_target(client, hold_scope_type, hold_scope_id, hold_scope_id).await {
                Ok(target) => target.snapshot_json(),
                Err(_) => json!({
                    "ref_type": hold_scope_type,
                    "ref_id": hold_scope_id,
                    "target_missing": true,
                }),
            }
        }
        other => json!({
            "ref_type": other,
            "ref_id": hold_scope_id,
        }),
    }
}

async fn validate_retention_policy(
    client: &db::Client,
    retention_policy_id: Option<&str>,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(retention_policy_id) = retention_policy_id else {
        return Ok(None);
    };
    let row = client
        .query_opt(
            "SELECT retention_policy_id::text, legal_hold_allowed, status
             FROM audit.retention_policy
             WHERE retention_policy_id = $1::text::uuid",
            &[&retention_policy_id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        bad_request(
            request_id,
            format!("retention_policy_id does not exist: {retention_policy_id}"),
        )
    })?;
    let legal_hold_allowed: bool = row.get(1);
    let status: String = row.get(2);
    if status != "active" || !legal_hold_allowed {
        return Err(bad_request(
            request_id,
            format!(
                "retention_policy_id must be active and legal_hold_allowed=true: {retention_policy_id}"
            ),
        ));
    }
    Ok(Some(row.get(0)))
}

async fn normalize_hold_until(
    client: &db::Client,
    hold_until: Option<&str>,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(hold_until) = hold_until.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let row = client
        .query_one(
            "SELECT to_char($1::timestamptz AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    ($1::timestamptz > now()) AS future_hold",
            &[&hold_until],
        )
        .await
        .map_err(|_| bad_request(request_id, format!("hold_until must be a valid future timestamp: {hold_until}")))?;
    let is_future: bool = row.get(1);
    if !is_future {
        return Err(bad_request(
            request_id,
            "hold_until must be later than current time",
        ));
    }
    Ok(Some(row.get(0)))
}

fn build_legal_hold_create_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    target_snapshot: Value,
    request_metadata: Value,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "endpoint": "POST /api/v1/audit/legal-holds",
        "request_id": request_id,
        "trace_id": trace_id,
        "requested_by_role": current_role(headers),
        "requested_by_tenant_id": header(headers, "x-tenant-id"),
        "step_up_challenge_id": step_up_challenge_id,
        "step_up_token_present": step_up_token_present,
        "target_snapshot": target_snapshot,
        "request_metadata": request_metadata,
    })
}

fn annotate_legal_hold_metadata(metadata: &mut Value, target: &ExportTarget) {
    let Some(object) = metadata.as_object_mut() else {
        return;
    };
    match target {
        ExportTarget::Order { order_id, .. } => {
            object.insert("ref_type".to_string(), Value::String("order".to_string()));
            object.insert("order_id".to_string(), Value::String(order_id.clone()));
        }
        ExportTarget::DisputeCase {
            case_id, order_id, ..
        } => {
            object.insert(
                "ref_type".to_string(),
                Value::String("dispute_case".to_string()),
            );
            object.insert("case_id".to_string(), Value::String(case_id.clone()));
            object.insert("order_id".to_string(), Value::String(order_id.clone()));
        }
    }
}

fn build_legal_hold_release_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    released_at: &str,
    release_reason: &str,
    target_snapshot: Value,
    request_metadata: Value,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "release_endpoint": "POST /api/v1/audit/legal-holds/{id}/release",
        "release_request_id": request_id,
        "release_trace_id": trace_id,
        "release_reason": release_reason,
        "released_at": released_at,
        "released_by_role": current_role(headers),
        "released_by_tenant_id": header(headers, "x-tenant-id"),
        "step_up_challenge_id": step_up_challenge_id,
        "step_up_token_present": step_up_token_present,
        "target_snapshot": target_snapshot,
        "release_request_metadata": request_metadata,
    })
}

fn build_legal_hold_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    ref_type: &str,
    ref_id: &str,
    action_name: &str,
    result_code: &str,
    legal_hold_status: &str,
    legal_hold_id: Option<String>,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    metadata: Value,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "audit",
        ref_type,
        Some(ref_id.to_string()),
        action_name,
        result_code,
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "legal_hold_id": legal_hold_id,
                "step_up_token_present": step_up_token_present,
                "details": metadata,
            }),
        },
    );
    event.legal_hold_status = legal_hold_status.to_string();
    event.sensitivity_level = "high".to_string();
    event
}

fn build_anchor_retry_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    retried_at: &str,
    reason: &str,
    batch: &crate::modules::audit::domain::AnchorBatch,
    request_metadata: Value,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "retry_requested_at": retried_at,
        "retry_requested_by": {
            "user_id": header(headers, "x-user-id"),
            "role": current_role(headers),
            "tenant_id": header(headers, "x-tenant-id"),
        },
        "retry_request": {
            "endpoint": "POST /api/v1/audit/anchor-batches/{id}/retry",
            "request_id": request_id,
            "trace_id": trace_id,
            "reason": reason,
            "step_up_challenge_id": step_up_challenge_id,
            "step_up_token_present": step_up_token_present,
            "request_metadata": request_metadata,
        },
        "previous_status": batch.status,
        "last_tx_hash": batch.metadata.get("tx_hash").and_then(Value::as_str),
        "last_authority_model": batch.metadata.get("authority_model").and_then(Value::as_str),
        "last_reconcile_status": batch.metadata.get("reconcile_status").and_then(Value::as_str),
    })
}

fn anchor_batch_snapshot(batch: &crate::modules::audit::domain::AnchorBatch) -> Value {
    json!({
        "anchor_batch_id": batch.anchor_batch_id,
        "batch_scope": batch.batch_scope,
        "chain_id": batch.chain_id,
        "record_count": batch.record_count,
        "batch_root": batch.batch_root,
        "anchor_status": batch.status,
        "tx_hash": batch.metadata.get("tx_hash").and_then(Value::as_str),
        "anchored_at": batch.anchored_at,
        "chain_anchor_id": batch.chain_anchor_id,
        "window_started_at": batch.window_started_at,
        "window_ended_at": batch.window_ended_at,
        "authority_model": batch.metadata.get("authority_model").and_then(Value::as_str),
        "reconcile_status": batch.metadata.get("reconcile_status").and_then(Value::as_str),
        "last_reconciled_at": batch.metadata.get("last_reconciled_at").and_then(Value::as_str),
    })
}

fn build_anchor_retry_outbox_payload(
    previous_batch: &crate::modules::audit::domain::AnchorBatch,
    batch: &crate::modules::audit::domain::AnchorBatch,
    reason: &str,
    actor_user_id: &str,
    request_id: &str,
    trace_id: &str,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "anchor_batch_id": batch.anchor_batch_id,
        "batch_scope": batch.batch_scope,
        "chain_id": batch.chain_id,
        "record_count": batch.record_count,
        "batch_root": batch.batch_root,
        "anchor_status": batch.status,
        "previous_anchor_status": previous_batch.status,
        "chain_anchor_id": batch.chain_anchor_id,
        "tx_hash": batch.metadata.get("tx_hash").and_then(Value::as_str),
        "requested_by": actor_user_id,
        "request_id": request_id,
        "trace_id": trace_id,
        "retry_reason": reason,
        "step_up_challenge_id": step_up_challenge_id,
        "step_up_token_present": step_up_token_present,
        "source": "audit.anchor.retry",
    })
}

fn build_anchor_retry_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    anchor_batch_id: &str,
    previous_status: &str,
    new_status: &str,
    reason: &str,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    batch: crate::modules::audit::domain::AnchorBatch,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "audit",
        "anchor_batch",
        Some(anchor_batch_id.to_string()),
        "audit.anchor.retry",
        "accepted",
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "previous_status": previous_status,
                "anchor_status": new_status,
                "step_up_token_present": step_up_token_present,
                "anchor_batch": anchor_batch_snapshot(&batch),
            }),
        },
    );
    event.sensitivity_level = "high".to_string();
    event
}

fn build_export_payload(
    export_id: &str,
    package_type: &str,
    target: &ExportTarget,
    masked_level: &str,
    reason: &str,
    exported_at: &str,
    headers: &HeaderMap,
    step_up: &StepUpBinding,
    traces: &[crate::modules::audit::dto::AuditTraceView],
    trace_total: i64,
    related_cases: &Value,
    evidence_manifests: &Value,
    evidence_items: &Value,
    legacy_evidence_refs: &Value,
    legal_holds: &Value,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let trace_value = if masked_level == "summary" {
        serde_json::to_value(
            traces
                .iter()
                .map(|trace| {
                    json!({
                        "audit_id": trace.audit_id,
                        "ref_type": trace.ref_type,
                        "ref_id": trace.ref_id,
                        "action_name": trace.action_name,
                        "result_code": trace.result_code,
                        "occurred_at": trace.occurred_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|err| {
            internal_error(
                header(headers, "x-request-id"),
                format!("audit trace payload serialization failed: {err}"),
            )
        })?
    } else {
        serde_json::to_value(traces).map_err(|err| {
            internal_error(
                header(headers, "x-request-id"),
                format!("audit trace export serialization failed: {err}"),
            )
        })?
    };

    Ok(json!({
        "export_version": "v1",
        "evidence_package_id": export_id,
        "package_type": package_type,
        "masked_level": masked_level,
        "reason": reason,
        "exported_at": exported_at,
        "requested_by": {
            "user_id": header(headers, "x-user-id"),
            "role": current_role(headers),
            "tenant_id": header(headers, "x-tenant-id"),
        },
        "step_up": {
            "challenge_id": step_up.challenge_id,
            "token_present": step_up.token_present,
        },
        "target": target.snapshot_json(),
        "counts": {
            "audit_trace_total": trace_total,
            "related_case_count": count_json_items(related_cases),
            "evidence_manifest_count": count_json_items(evidence_manifests),
            "evidence_item_count": count_json_items(evidence_items),
            "legacy_evidence_ref_count": count_json_items(legacy_evidence_refs),
            "legal_hold_count": count_json_items(legal_holds),
        },
        "audit_traces": trace_value,
        "related_cases": related_cases,
        "evidence": {
            "manifests": evidence_manifests,
            "items": evidence_items,
            "legacy_refs": legacy_evidence_refs,
        },
        "legal_holds": legal_holds,
    }))
}

fn build_export_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    ref_type: &str,
    ref_id: &str,
    evidence_manifest_id: Option<String>,
    reason: &str,
    masked_level: &str,
    storage_uri: &str,
    legal_hold_status: &str,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "audit",
        ref_type,
        Some(ref_id.to_string()),
        "audit.package.export",
        "success",
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "masked_level": masked_level,
                "storage_uri": storage_uri,
                "step_up_token_present": step_up_token_present,
            }),
        },
    );
    event.evidence_manifest_id = evidence_manifest_id;
    event.legal_hold_status = legal_hold_status.to_string();
    event.sensitivity_level = if masked_level == "unmasked" {
        "restricted".to_string()
    } else {
        "high".to_string()
    };
    event
}

fn build_replay_report(
    replay_job_id: &str,
    replay_type: &str,
    target: &ReplayTarget,
    reason: &str,
    executed_at: &str,
    headers: &HeaderMap,
    step_up: &StepUpBinding,
    traces: &[crate::modules::audit::dto::AuditTraceView],
    trace_total: i64,
    evidence_manifests: &Value,
    evidence_items: &Value,
    legal_holds: &Value,
    options: Value,
) -> Result<ReplayReport, (StatusCode, Json<ErrorResponse>)> {
    let snapshot = target.snapshot_json();
    let snapshot_digest = sha256_hex(snapshot.to_string().as_bytes());
    let trace_preview = serde_json::to_value(
        traces
            .iter()
            .take(20)
            .map(|trace| {
                json!({
                    "audit_id": trace.audit_id,
                    "action_name": trace.action_name,
                    "result_code": trace.result_code,
                    "request_id": trace.request_id,
                    "trace_id": trace.trace_id,
                    "occurred_at": trace.occurred_at,
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|err| {
        internal_error(
            header(headers, "x-request-id"),
            format!("audit replay trace serialization failed: {err}"),
        )
    })?;
    let trace_digest = sha256_hex(trace_preview.to_string().as_bytes());
    let evidence_preview = json!({
        "manifest_count": count_json_items(evidence_manifests),
        "item_count": count_json_items(evidence_items),
        "latest_manifest_id": evidence_manifests
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item.get("evidence_manifest_id"))
            .and_then(Value::as_str),
        "latest_manifest_hash": evidence_manifests
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item.get("manifest_hash"))
            .and_then(Value::as_str),
        "legal_hold_status": derive_legal_hold_status(legal_holds),
    });
    let evidence_digest = sha256_hex(evidence_preview.to_string().as_bytes());

    let recommendation =
        if count_json_items(evidence_manifests) == 0 && count_json_items(evidence_items) == 0 {
            "collect_evidence_before_replay".to_string()
        } else if replay_type == "compensation_replay" {
            "dry_run_only_whitelist_required".to_string()
        } else {
            "dry_run_completed".to_string()
        };

    let results = vec![
        ReplayResult {
            replay_result_id: None,
            replay_job_id: Some(replay_job_id.to_string()),
            step_name: "target_snapshot".to_string(),
            result_code: "loaded".to_string(),
            expected_digest: None,
            actual_digest: Some(snapshot_digest.clone()),
            diff_summary: json!({
                "target": snapshot,
            }),
            created_at: None,
        },
        ReplayResult {
            replay_result_id: None,
            replay_job_id: Some(replay_job_id.to_string()),
            step_name: "audit_timeline".to_string(),
            result_code: if trace_total > 0 {
                "ready".to_string()
            } else {
                "missing_audit_trace".to_string()
            },
            expected_digest: None,
            actual_digest: Some(trace_digest.clone()),
            diff_summary: json!({
                "trace_total": trace_total,
                "preview": trace_preview,
            }),
            created_at: None,
        },
        ReplayResult {
            replay_result_id: None,
            replay_job_id: Some(replay_job_id.to_string()),
            step_name: "evidence_projection".to_string(),
            result_code: if count_json_items(evidence_manifests) > 0
                || count_json_items(evidence_items) > 0
            {
                "ready".to_string()
            } else {
                "missing_evidence".to_string()
            },
            expected_digest: evidence_manifests
                .as_array()
                .and_then(|items| items.first())
                .and_then(|item| item.get("manifest_hash"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            actual_digest: Some(evidence_digest.clone()),
            diff_summary: evidence_preview,
            created_at: None,
        },
        ReplayResult {
            replay_result_id: None,
            replay_job_id: Some(replay_job_id.to_string()),
            step_name: "execution_policy".to_string(),
            result_code: REPLAY_DRY_RUN_ONLY_ERROR.to_string(),
            expected_digest: None,
            actual_digest: None,
            diff_summary: json!({
                "dry_run": true,
                "side_effects_executed": false,
                "recommendation": recommendation,
            }),
            created_at: None,
        },
    ];

    Ok(ReplayReport {
        payload: json!({
            "report_version": "v1",
            "replay_job_id": replay_job_id,
            "replay_type": replay_type,
            "dry_run": true,
            "reason": reason,
            "executed_at": executed_at,
            "requested_by": {
                "user_id": header(headers, "x-user-id"),
                "role": current_role(headers),
                "tenant_id": header(headers, "x-tenant-id"),
            },
            "step_up": {
                "challenge_id": step_up.challenge_id,
                "token_present": step_up.token_present,
            },
            "target": target.snapshot_json(),
            "counts": {
                "audit_trace_total": trace_total,
                "evidence_manifest_count": count_json_items(evidence_manifests),
                "evidence_item_count": count_json_items(evidence_items),
                "legal_hold_count": count_json_items(legal_holds),
            },
            "results": results.iter().map(|item| json!({
                "step_name": item.step_name,
                "result_code": item.result_code,
                "expected_digest": item.expected_digest,
                "actual_digest": item.actual_digest,
                "diff_summary": item.diff_summary,
            })).collect::<Vec<_>>(),
            "recommendation": recommendation,
            "user_options": options,
        }),
        recommendation,
        results,
    })
}

fn build_replay_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    replay_job_id: &str,
    evidence_manifest_id: Option<String>,
    action_name: &str,
    result_code: &str,
    reason: &str,
    replay_type: &str,
    target_ref_type: &str,
    target_ref_id: &str,
    dry_run: bool,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "audit",
        "replay_job",
        Some(replay_job_id.to_string()),
        action_name,
        result_code,
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "replay_type": replay_type,
                "target_ref_type": target_ref_type,
                "target_ref_id": target_ref_id,
                "dry_run": dry_run,
                "step_up_token_present": step_up_token_present,
            }),
        },
    );
    event.evidence_manifest_id = evidence_manifest_id;
    event.sensitivity_level = "high".to_string();
    event
}

async fn record_replay_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    replay_job_id: &str,
    replay_job: &ReplayJob,
    result_count: i64,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let role = current_role(headers);
    let access_audit_id = repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: parse_uuid_header(headers, "x-user-id"),
            accessor_role_key: Some(role.clone()),
            access_mode: "replay".to_string(),
            target_type: "replay_job".to_string(),
            target_id: Some(replay_job_id.to_string()),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: parse_uuid_header(headers, "x-step-up-challenge-id"),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            metadata: json!({
                "endpoint": "GET /api/v1/audit/replay-jobs/{id}",
                "replay_type": replay_job.replay_type,
                "replay_status": replay_job.status,
                "target_ref_type": replay_job.ref_type,
                "target_ref_id": replay_job.ref_id,
                "result_count": result_count,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id,
            trace_id,
            message_text: "audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}"
                .to_string(),
            structured_payload: json!({
                "module": "audit",
                "endpoint": "GET /api/v1/audit/replay-jobs/{id}",
                "access_audit_id": access_audit_id,
                "replay_job_id": replay_job_id,
                "replay_type": replay_job.replay_type,
                "replay_status": replay_job.status,
                "target_ref_type": replay_job.ref_type,
                "target_ref_id": replay_job.ref_id,
                "result_count": result_count,
                "role": role,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

async fn record_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
    filters: serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let filters_for_access = filters.clone();
    let role = current_role(headers);
    let access_audit_id = repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: parse_uuid_header(headers, "x-user-id"),
            accessor_role_key: Some(role.clone()),
            access_mode: "masked".to_string(),
            target_type: target_type.to_string(),
            target_id,
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: parse_uuid_header(headers, "x-step-up-challenge-id"),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            metadata: json!({
                "endpoint": endpoint,
                "filters": filters_for_access,
                "step_up_token_present": header(headers, "x-step-up-token").is_some(),
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id,
            trace_id,
            message_text: format!("audit lookup executed: {endpoint}"),
            structured_payload: json!({
                "module": "audit",
                "endpoint": endpoint,
                "access_audit_id": access_audit_id,
                "role": role,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

async fn record_ops_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    target_type: &str,
    target_id: Option<String>,
    endpoint: &str,
    filters: serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let filters_for_access = filters.clone();
    let role = current_role(headers);
    let access_audit_id = repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: parse_uuid_header(headers, "x-user-id"),
            accessor_role_key: Some(role.clone()),
            access_mode: "masked".to_string(),
            target_type: target_type.to_string(),
            target_id,
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: parse_uuid_header(headers, "x-step-up-challenge-id"),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            metadata: json!({
                "endpoint": endpoint,
                "filters": filters_for_access,
                "step_up_token_present": header(headers, "x-step-up-token").is_some(),
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id,
            trace_id,
            message_text: format!("ops lookup executed: {endpoint}"),
            structured_payload: json!({
                "module": "ops",
                "endpoint": endpoint,
                "access_audit_id": access_audit_id,
                "role": role,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

async fn record_developer_lookup_side_effects(
    client: &db::Client,
    headers: &HeaderMap,
    target_id: String,
    filters: serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(headers, "x-request-id");
    let trace_id = header(headers, "x-trace-id");
    let filters_for_access = filters.clone();
    let role = current_role(headers);
    let access_audit_id = repo::record_access_audit(
        client,
        &AccessAuditInsert {
            accessor_user_id: parse_uuid_header(headers, "x-user-id"),
            accessor_role_key: Some(role.clone()),
            access_mode: "masked".to_string(),
            target_type: "developer_trace_query".to_string(),
            target_id: Some(target_id),
            masked_view: true,
            breakglass_reason: None,
            step_up_challenge_id: parse_uuid_header(headers, "x-step-up-challenge-id"),
            request_id: request_id.clone(),
            trace_id: trace_id.clone(),
            metadata: json!({
                "endpoint": "GET /api/v1/developer/trace",
                "filters": filters_for_access,
                "step_up_token_present": header(headers, "x-step-up-token").is_some(),
            }),
        },
    )
    .await
    .map_err(map_db_error)?;

    repo::record_system_log(
        client,
        &SystemLogInsert {
            service_name: "platform-core".to_string(),
            log_level: "INFO".to_string(),
            request_id,
            trace_id,
            message_text: "developer trace lookup executed: GET /api/v1/developer/trace"
                .to_string(),
            structured_payload: json!({
                "module": "developer",
                "endpoint": "GET /api/v1/developer/trace",
                "access_audit_id": access_audit_id,
                "role": role,
                "filters": filters,
            }),
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

fn normalize_developer_trace_query(
    query: DeveloperTraceQuery,
    request_id: &str,
) -> Result<DeveloperTraceQuery, (StatusCode, Json<ErrorResponse>)> {
    validate_optional_uuid(query.order_id.as_deref(), "order_id", request_id)?;
    validate_optional_uuid(query.event_id.as_deref(), "event_id", request_id)?;
    let normalized = DeveloperTraceQuery {
        order_id: normalize_optional_filter(query.order_id.as_deref(), "order_id", request_id)?,
        event_id: normalize_optional_filter(query.event_id.as_deref(), "event_id", request_id)?,
        tx_hash: normalize_optional_filter(query.tx_hash.as_deref(), "tx_hash", request_id)?,
    };
    let selector_count = u8::from(normalized.order_id.is_some())
        + u8::from(normalized.event_id.is_some())
        + u8::from(normalized.tx_hash.is_some());
    if selector_count != 1 {
        return Err(bad_request(
            request_id,
            "developer trace requires exactly one of order_id, event_id, tx_hash",
        ));
    }
    Ok(normalized)
}

async fn resolve_developer_trace_lookup(
    client: &db::Client,
    query: &DeveloperTraceQuery,
    request_id: &str,
) -> Result<DeveloperTraceResolution, (StatusCode, Json<ErrorResponse>)> {
    if let Some(order_id) = query.order_id.clone() {
        return Ok(DeveloperTraceResolution {
            lookup_mode: "order_id".to_string(),
            lookup_value: order_id.clone(),
            matched_object_type: "order".to_string(),
            matched_object_id: Some(order_id.clone()),
            resolved_ref_type: "order".to_string(),
            resolved_ref_id: order_id,
            matched_audit_trace: None,
            matched_outbox_event: None,
            matched_dead_letter: None,
            matched_chain_anchor: None,
            matched_projection_gap: None,
            matched_checkpoint: None,
            request_id: None,
            trace_id: None,
        });
    }

    if let Some(event_id) = query.event_id.clone() {
        if let Some(outbox_event) = repo::load_outbox_event_by_id(client, event_id.as_str())
            .await
            .map_err(map_db_error)?
        {
            let resolved_ref_type = developer_trace_ref_type_from_aggregate(
                outbox_event.aggregate_type.as_str(),
                request_id,
            )?;
            let resolved_ref_id = outbox_event.aggregate_id.clone().ok_or_else(|| {
                internal_error(
                    Some(request_id.to_string()),
                    format!("developer trace outbox event missing aggregate_id: {event_id}"),
                )
            })?;
            let matched_dead_letter =
                repo::load_latest_dead_letter_by_outbox_event_id(client, event_id.as_str())
                    .await
                    .map_err(map_db_error)?;
            return Ok(DeveloperTraceResolution {
                lookup_mode: "event_id".to_string(),
                lookup_value: event_id.clone(),
                matched_object_type: "outbox_event".to_string(),
                matched_object_id: outbox_event.outbox_event_id.clone(),
                resolved_ref_type,
                resolved_ref_id,
                matched_audit_trace: None,
                matched_outbox_event: Some(outbox_event.clone()),
                matched_dead_letter,
                matched_chain_anchor: None,
                matched_projection_gap: None,
                matched_checkpoint: None,
                request_id: outbox_event.request_id.clone(),
                trace_id: outbox_event.trace_id.clone(),
            });
        }

        if let Some(audit_trace) = repo::load_audit_trace_by_id(client, event_id.as_str())
            .await
            .map_err(map_db_error)?
        {
            let resolved_ref_type =
                normalize_consistency_ref_type(audit_trace.ref_type.as_str(), request_id)?;
            let resolved_ref_id = audit_trace.ref_id.clone().ok_or_else(|| {
                internal_error(
                    Some(request_id.to_string()),
                    format!("developer trace audit event missing ref_id: {event_id}"),
                )
            })?;
            return Ok(DeveloperTraceResolution {
                lookup_mode: "event_id".to_string(),
                lookup_value: event_id,
                matched_object_type: "audit_event".to_string(),
                matched_object_id: audit_trace.audit_id.clone(),
                resolved_ref_type,
                resolved_ref_id,
                matched_audit_trace: Some(audit_trace.clone()),
                matched_outbox_event: None,
                matched_dead_letter: None,
                matched_chain_anchor: None,
                matched_projection_gap: None,
                matched_checkpoint: None,
                request_id: audit_trace.request_id.clone(),
                trace_id: audit_trace.trace_id.clone(),
            });
        }

        return Err(not_found(
            request_id,
            format!("developer trace event target not found: {event_id}"),
        ));
    }

    let tx_hash = query
        .tx_hash
        .clone()
        .ok_or_else(|| bad_request(request_id, "developer trace requires one lookup selector"))?;
    if let Some(chain_anchor) = repo::load_chain_anchor_by_tx_hash(client, tx_hash.as_str())
        .await
        .map_err(map_db_error)?
    {
        let resolved_ref_type =
            normalize_consistency_ref_type(chain_anchor.ref_type.as_str(), request_id)?;
        let resolved_ref_id = chain_anchor.ref_id.clone().ok_or_else(|| {
            internal_error(
                Some(request_id.to_string()),
                format!("developer trace chain anchor missing ref_id for tx_hash: {tx_hash}"),
            )
        })?;
        let matched_audit_trace =
            repo::load_latest_audit_trace_by_tx_hash(client, tx_hash.as_str())
                .await
                .map_err(map_db_error)?;
        return Ok(DeveloperTraceResolution {
            lookup_mode: "tx_hash".to_string(),
            lookup_value: tx_hash,
            matched_object_type: "chain_anchor".to_string(),
            matched_object_id: chain_anchor.chain_anchor_id.clone(),
            resolved_ref_type,
            resolved_ref_id,
            matched_audit_trace: matched_audit_trace.clone(),
            matched_outbox_event: None,
            matched_dead_letter: None,
            matched_chain_anchor: Some(chain_anchor),
            matched_projection_gap: None,
            matched_checkpoint: None,
            request_id: matched_audit_trace
                .as_ref()
                .and_then(|trace| trace.request_id.clone()),
            trace_id: matched_audit_trace
                .as_ref()
                .and_then(|trace| trace.trace_id.clone()),
        });
    }

    if let Some(audit_trace) = repo::load_latest_audit_trace_by_tx_hash(client, tx_hash.as_str())
        .await
        .map_err(map_db_error)?
    {
        let resolved_ref_type =
            normalize_consistency_ref_type(audit_trace.ref_type.as_str(), request_id)?;
        let resolved_ref_id = audit_trace.ref_id.clone().ok_or_else(|| {
            internal_error(
                Some(request_id.to_string()),
                format!("developer trace audit event missing ref_id for tx_hash: {tx_hash}"),
            )
        })?;
        return Ok(DeveloperTraceResolution {
            lookup_mode: "tx_hash".to_string(),
            lookup_value: tx_hash,
            matched_object_type: "audit_event".to_string(),
            matched_object_id: audit_trace.audit_id.clone(),
            resolved_ref_type,
            resolved_ref_id,
            matched_audit_trace: Some(audit_trace.clone()),
            matched_outbox_event: None,
            matched_dead_letter: None,
            matched_chain_anchor: None,
            matched_projection_gap: None,
            matched_checkpoint: None,
            request_id: audit_trace.request_id.clone(),
            trace_id: audit_trace.trace_id.clone(),
        });
    }

    if let Some(projection_gap) =
        repo::load_latest_chain_projection_gap_by_tx_hash(client, tx_hash.as_str())
            .await
            .map_err(map_db_error)?
    {
        let resolved_ref_type = developer_trace_ref_type_from_aggregate(
            projection_gap.aggregate_type.as_str(),
            request_id,
        )?;
        let resolved_ref_id = projection_gap
            .aggregate_id
            .clone()
            .or_else(|| {
                if resolved_ref_type == "order" {
                    projection_gap.order_id.clone()
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                internal_error(
                    Some(request_id.to_string()),
                    format!(
                        "developer trace projection gap missing aggregate_id for tx_hash: {tx_hash}"
                    ),
                )
            })?;
        return Ok(DeveloperTraceResolution {
            lookup_mode: "tx_hash".to_string(),
            lookup_value: tx_hash,
            matched_object_type: "chain_projection_gap".to_string(),
            matched_object_id: projection_gap.chain_projection_gap_id.clone(),
            resolved_ref_type,
            resolved_ref_id,
            matched_audit_trace: None,
            matched_outbox_event: None,
            matched_dead_letter: None,
            matched_chain_anchor: None,
            matched_projection_gap: Some(projection_gap.clone()),
            matched_checkpoint: None,
            request_id: projection_gap.request_id.clone(),
            trace_id: projection_gap.trace_id.clone(),
        });
    }

    if let Some(checkpoint) =
        repo::load_latest_trade_lifecycle_checkpoint_by_tx_hash(client, tx_hash.as_str())
            .await
            .map_err(map_db_error)?
    {
        let resolved_ref_type =
            normalize_consistency_ref_type(checkpoint.ref_type.as_str(), request_id)?;
        return Ok(DeveloperTraceResolution {
            lookup_mode: "tx_hash".to_string(),
            lookup_value: tx_hash,
            matched_object_type: "trade_lifecycle_checkpoint".to_string(),
            matched_object_id: checkpoint.trade_lifecycle_checkpoint_id.clone(),
            resolved_ref_type,
            resolved_ref_id: checkpoint.ref_id.clone(),
            matched_audit_trace: None,
            matched_outbox_event: None,
            matched_dead_letter: None,
            matched_chain_anchor: None,
            matched_projection_gap: None,
            matched_checkpoint: Some(checkpoint.clone()),
            request_id: checkpoint.request_id.clone(),
            trace_id: checkpoint.trace_id.clone(),
        });
    }

    Err(not_found(
        request_id,
        format!("developer trace tx target not found: {tx_hash}"),
    ))
}

fn developer_trace_ref_type_from_aggregate(
    aggregate_type: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match aggregate_type {
        "order" | "trade.order_main" => Ok("order".to_string()),
        "contract" | "digital_contract" | "contract.digital_contract" => {
            Ok("digital_contract".to_string())
        }
        "delivery" | "delivery_record" | "delivery.delivery_record" => {
            Ok("delivery_record".to_string())
        }
        "settlement" | "settlement_record" | "billing.settlement_record" => {
            Ok("settlement_record".to_string())
        }
        "payment" | "payment_intent" | "payment.payment_intent" => Ok("payment_intent".to_string()),
        "refund" | "refund_intent" | "payment.refund_intent" => Ok("refund_intent".to_string()),
        "payout" | "payout_instruction" | "payment.payout_instruction" => {
            Ok("payout_instruction".to_string())
        }
        other => Err(not_found(
            request_id,
            format!("developer trace aggregate type is not supported: {other}"),
        )),
    }
}

fn developer_trace_order_id(subject: &repo::ConsistencySubjectRecord) -> Option<String> {
    if subject.ref_type == "order" {
        Some(subject.ref_id.clone())
    } else {
        subject.order_id.clone()
    }
}

fn developer_trace_resolved_trace_id(
    preferred: Option<String>,
    audit_traces: &[AuditTraceView],
    outbox_events: &[repo::OutboxEventRecord],
    dead_letters: &[repo::DeadLetterEventRecord],
    external_facts: &[repo::ExternalFactReceiptRecord],
    projection_gaps: &[repo::ChainProjectionGapRecord],
    checkpoints: &[repo::TradeLifecycleCheckpointRecord],
) -> Option<String> {
    preferred
        .or_else(|| audit_traces.iter().find_map(|trace| trace.trace_id.clone()))
        .or_else(|| {
            outbox_events
                .iter()
                .find_map(|event| event.trace_id.clone())
        })
        .or_else(|| dead_letters.iter().find_map(|event| event.trace_id.clone()))
        .or_else(|| {
            external_facts
                .iter()
                .find_map(|receipt| receipt.trace_id.clone())
        })
        .or_else(|| projection_gaps.iter().find_map(|gap| gap.trace_id.clone()))
        .or_else(|| {
            checkpoints
                .iter()
                .find_map(|checkpoint| checkpoint.trace_id.clone())
        })
}

fn build_developer_trace_snapshot(order_snapshot: &Value, matched_snapshot: &Value) -> Value {
    if order_snapshot == matched_snapshot {
        json!({
            "order": order_snapshot,
        })
    } else {
        json!({
            "order": order_snapshot,
            "matched_ref": matched_snapshot,
        })
    }
}

fn is_developer_tenant_scoped_role(role: &str) -> bool {
    matches!(canonical_role_key(role), "tenant_developer")
}

fn ensure_developer_trace_scope(
    headers: &HeaderMap,
    scope: &OrderAuditScope,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = current_role(headers);
    if !is_developer_tenant_scoped_role(&role) {
        return Ok(());
    }

    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        bad_request(
            request_id,
            "x-tenant-id is required for developer trace tenant scope",
        )
    })?;
    if tenant_id == scope.buyer_org_id || tenant_id == scope.seller_org_id {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "developer trace is forbidden outside tenant order scope".to_string(),
            request_id: Some(request_id.to_string()),
        }),
    ))
}

fn json_string(snapshot: &Value, key: &str) -> Option<String> {
    snapshot
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn resolve_searchrec_dead_letter_consumers(
    dead_letter: &repo::DeadLetterEventRecord,
) -> Option<Vec<String>> {
    if dead_letter.failure_stage.as_deref() != Some("consumer_handler") {
        return None;
    }

    let expected = match dead_letter.target_topic.as_deref() {
        Some("dtp.search.sync") => vec!["search-indexer".to_string()],
        Some("dtp.recommend.behavior") => vec!["recommendation-aggregator".to_string()],
        _ => return None,
    };

    let mut observed: Vec<String> = dead_letter
        .consumer_idempotency_records
        .iter()
        .map(|record| record.consumer_name.clone())
        .collect();
    observed.sort();
    observed.dedup();
    if observed.is_empty() {
        return Some(expected);
    }

    if observed.iter().all(|consumer| expected.contains(consumer)) {
        Some(observed)
    } else {
        None
    }
}

fn searchrec_consumer_groups_for_topic(topic: Option<&str>) -> Option<Vec<String>> {
    match topic {
        Some("dtp.search.sync") => Some(vec!["cg-search-indexer".to_string()]),
        Some("dtp.recommend.behavior") => Some(vec!["cg-recommendation-aggregator".to_string()]),
        _ => None,
    }
}

fn build_dead_letter_reprocess_plan(
    dead_letter: &repo::DeadLetterEventRecord,
    reason: &str,
    dry_run: bool,
    consumer_names: &[String],
    consumer_groups: &[String],
    replay_target_topic: &str,
    request_metadata: Value,
    request_id: &str,
    trace_id: &str,
) -> Value {
    json!({
        "mode": if dry_run { "dry_run" } else { "execute" },
        "reason": reason,
        "reprocess_strategy": "searchrec_worker_replay_preview",
        "target_topic": replay_target_topic,
        "target_consumers": consumer_names,
        "target_consumer_groups": consumer_groups,
        "lineage": {
            "dead_letter_event_id": dead_letter.dead_letter_event_id,
            "original_event_id": dead_letter.outbox_event_id,
            "event_type": dead_letter.event_type,
            "aggregate_type": dead_letter.aggregate_type,
            "aggregate_id": dead_letter.aggregate_id,
        },
        "dead_letter_state": {
            "failure_stage": dead_letter.failure_stage,
            "reprocess_status": dead_letter.reprocess_status,
            "first_failed_at": dead_letter.first_failed_at,
            "last_failed_at": dead_letter.last_failed_at,
            "failed_reason": dead_letter.failed_reason,
        },
        "request_context": {
            "request_id": request_id,
            "trace_id": trace_id,
            "request_metadata": request_metadata,
        },
        "v1_constraints": {
            "dry_run_only": true,
            "actual_worker_reprocess_pending_task": "SEARCHREC-020",
        },
    })
}

fn build_dead_letter_reprocess_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    dead_letter_event_id: &str,
    reason: &str,
    dead_letter: &repo::DeadLetterEventRecord,
    dry_run: bool,
    consumer_names: &[String],
    consumer_groups: &[String],
    replay_target_topic: &str,
    replay_plan: Value,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "ops",
        "dead_letter_event",
        Some(dead_letter_event_id.to_string()),
        "ops.dead_letter.reprocess.dry_run",
        "dry_run_completed",
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "dry_run": dry_run,
                "step_up_token_present": step_up_token_present,
                "current_role": current_role(headers),
                "target_topic": replay_target_topic,
                "consumer_names": consumer_names,
                "consumer_groups": consumer_groups,
                "dead_letter": {
                    "outbox_event_id": dead_letter.outbox_event_id,
                    "event_type": dead_letter.event_type,
                    "failure_stage": dead_letter.failure_stage,
                    "reprocess_status": dead_letter.reprocess_status,
                },
                "replay_plan": replay_plan,
            }),
        },
    );
    event.sensitivity_level = "high".to_string();
    event
}

fn build_consistency_reconcile_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    ref_type: &str,
    ref_id: &str,
    reason: &str,
    mode: &str,
    dry_run: bool,
    subject_snapshot: Value,
    reconcile_plan: Value,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "ops",
        ref_type,
        Some(ref_id.to_string()),
        "ops.consistency.reconcile.dry_run",
        "dry_run_completed",
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "mode": mode,
                "dry_run": dry_run,
                "step_up_token_present": step_up_token_present,
                "current_role": current_role(headers),
                "reconcile_target_topic": CONSISTENCY_RECONCILE_TARGET_TOPIC,
                "subject_snapshot": subject_snapshot,
                "reconcile_plan": reconcile_plan,
            }),
        },
    );
    event.sensitivity_level = "high".to_string();
    event
}

fn build_external_fact_confirmation_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    confirmed_at: &str,
    confirm_result: &str,
    reason: &str,
    operator_note: Option<&str>,
    actor_user_id: &str,
    existing_receipt: &repo::ExternalFactReceiptRecord,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "manual_confirmation": {
            "endpoint": "POST /api/v1/ops/external-facts/{id}/confirm",
            "request_id": request_id,
            "trace_id": trace_id,
            "confirmed_at": confirmed_at,
            "confirm_result": confirm_result,
            "reason": reason,
            "operator_note": operator_note,
            "operator_user_id": actor_user_id,
            "operator_role": current_role(headers),
            "tenant_id": header(headers, "x-tenant-id"),
            "step_up_challenge_id": step_up_challenge_id,
            "step_up_token_present": step_up_token_present,
            "previous_receipt_status": existing_receipt.receipt_status.clone(),
        },
        "rule_evaluation": {
            "status": "pending_follow_up",
            "requested_at": confirmed_at,
            "requested_by": actor_user_id,
            "request_id": request_id,
            "trace_id": trace_id,
            "reason": reason,
        }
    })
}

fn build_external_fact_confirm_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    confirm_result: &str,
    reason: &str,
    operator_note: Option<&str>,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    existing_receipt: &repo::ExternalFactReceiptRecord,
    confirmed_receipt: &repo::ExternalFactReceiptRecord,
) -> AuditEvent {
    let ref_type = confirmed_receipt
        .ref_type
        .clone()
        .unwrap_or_else(|| "external_fact_receipt".to_string());
    let ref_id = confirmed_receipt
        .ref_id
        .clone()
        .or_else(|| confirmed_receipt.external_fact_receipt_id.clone());
    let mut event = AuditEvent::business(
        "ops",
        ref_type.as_str(),
        ref_id,
        "ops.external_fact.confirm",
        confirm_result,
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "external_fact_receipt_id": confirmed_receipt.external_fact_receipt_id.clone(),
                "order_id": confirmed_receipt.order_id.clone(),
                "ref_domain": confirmed_receipt.ref_domain.clone(),
                "ref_type": confirmed_receipt.ref_type.clone(),
                "ref_id": confirmed_receipt.ref_id.clone(),
                "fact_type": confirmed_receipt.fact_type.clone(),
                "provider_type": confirmed_receipt.provider_type.clone(),
                "provider_key": confirmed_receipt.provider_key.clone(),
                "provider_reference": confirmed_receipt.provider_reference.clone(),
                "previous_receipt_status": existing_receipt.receipt_status.clone(),
                "receipt_status": confirmed_receipt.receipt_status.clone(),
                "confirmed_at": confirmed_receipt.confirmed_at.clone(),
                "reason": reason,
                "operator_note": operator_note,
                "step_up_token_present": step_up_token_present,
                "rule_evaluation_status": "pending_follow_up",
            }),
        },
    );
    event.sensitivity_level = "high".to_string();
    event
}

#[allow(clippy::too_many_arguments)]
fn build_projection_gap_resolve_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    resolved_at: &str,
    reason: &str,
    resolution_mode: &str,
    dry_run: bool,
    actor_user_id: &str,
    expected_state_digest: Option<&str>,
    current_state_digest: &str,
    existing_gap: &repo::ChainProjectionGapRecord,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "manual_resolution": {
            "endpoint": "POST /api/v1/ops/projection-gaps/{id}/resolve",
            "request_id": request_id,
            "trace_id": trace_id,
            "resolved_at": resolved_at,
            "reason": reason,
            "resolution_mode": resolution_mode,
            "dry_run": dry_run,
            "operator_user_id": actor_user_id,
            "operator_role": current_role(headers),
            "tenant_id": header(headers, "x-tenant-id"),
            "step_up_challenge_id": step_up_challenge_id,
            "step_up_token_present": step_up_token_present,
            "expected_state_digest": expected_state_digest,
            "current_state_digest": current_state_digest,
            "previous_gap_status": existing_gap.gap_status,
            "formal_persistent_object": "ops.chain_projection_gap",
            "control_plane_action": "projection_gap.resolve",
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn build_projection_gap_resolution_summary_patch(
    request_id: &str,
    trace_id: &str,
    resolved_at: &str,
    reason: &str,
    resolution_mode: &str,
    actor_user_id: &str,
    expected_state_digest: Option<&str>,
    current_state_digest: &str,
    existing_gap: &repo::ChainProjectionGapRecord,
) -> Value {
    json!({
        "manual_resolution": {
            "request_id": request_id,
            "trace_id": trace_id,
            "resolved_at": resolved_at,
            "reason": reason,
            "resolution_mode": resolution_mode,
            "operator_user_id": actor_user_id,
            "previous_gap_status": existing_gap.gap_status,
            "expected_state_digest": expected_state_digest,
            "current_state_digest": current_state_digest,
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn build_projection_gap_resolve_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    reason: &str,
    resolution_mode: &str,
    dry_run: bool,
    expected_state_digest: Option<&str>,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    previous_gap: &repo::ChainProjectionGapRecord,
    current_gap: &repo::ChainProjectionGapRecord,
    before_state_digest: String,
    after_state_digest: String,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "ops",
        "projection_gap",
        current_gap.chain_projection_gap_id.clone(),
        "ops.projection_gap.resolve",
        if dry_run {
            "dry_run_ready"
        } else {
            "resolution_recorded"
        },
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "chain_projection_gap_id": current_gap.chain_projection_gap_id.clone(),
                "aggregate_type": current_gap.aggregate_type.clone(),
                "aggregate_id": current_gap.aggregate_id.clone(),
                "order_id": current_gap.order_id.clone(),
                "chain_id": current_gap.chain_id.clone(),
                "gap_type": current_gap.gap_type.clone(),
                "gap_status_before": previous_gap.gap_status.clone(),
                "gap_status_after": current_gap.gap_status.clone(),
                "resolved_at": current_gap.resolved_at.clone(),
                "reason": reason,
                "resolution_mode": resolution_mode,
                "dry_run": dry_run,
                "expected_state_digest": expected_state_digest,
                "before_state_digest": before_state_digest,
                "after_state_digest": after_state_digest,
                "step_up_token_present": step_up_token_present,
                "formal_persistent_object": "ops.chain_projection_gap",
                "control_plane_action": "projection_gap.resolve",
            }),
        },
    );
    event.before_state_digest = Some(before_state_digest);
    event.after_state_digest = Some(after_state_digest);
    event.sensitivity_level = "high".to_string();
    event
}

#[allow(clippy::too_many_arguments)]
fn build_fairness_incident_handle_metadata(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    handled_at: &str,
    action: &str,
    resolution_summary: &str,
    auto_action_override: Option<&str>,
    freeze_settlement: bool,
    freeze_delivery: bool,
    create_dispute_suggestion: bool,
    actor_user_id: &str,
    existing_incident: &repo::FairnessIncidentRecord,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
) -> Value {
    json!({
        "handling": {
            "endpoint": "POST /api/v1/ops/fairness-incidents/{id}/handle",
            "request_id": request_id,
            "trace_id": trace_id,
            "handled_at": handled_at,
            "action": action,
            "resolution_summary": resolution_summary,
            "auto_action_override": auto_action_override,
            "freeze_settlement": freeze_settlement,
            "freeze_delivery": freeze_delivery,
            "create_dispute_suggestion": create_dispute_suggestion,
            "operator_user_id": actor_user_id,
            "operator_role": current_role(headers),
            "tenant_id": header(headers, "x-tenant-id"),
            "step_up_challenge_id": step_up_challenge_id,
            "step_up_token_present": step_up_token_present,
            "previous_status": existing_incident.fairness_incident_status.clone(),
            "business_mutation_executed": false,
        },
        "linked_action_plan": {
            "status": if auto_action_override.is_some()
                || freeze_settlement
                || freeze_delivery
                || create_dispute_suggestion
            {
                "suggestion_recorded"
            } else {
                "no_linked_action"
            },
            "auto_action_override": auto_action_override,
            "freeze_settlement": freeze_settlement,
            "freeze_delivery": freeze_delivery,
            "create_dispute_suggestion": create_dispute_suggestion,
            "execution_mode": "suggestion_only",
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn build_fairness_incident_handle_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: String,
    actor_user_id: &str,
    action: &str,
    resolution_summary: &str,
    auto_action_override: Option<&str>,
    freeze_settlement: bool,
    freeze_delivery: bool,
    create_dispute_suggestion: bool,
    action_plan_status: &str,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    existing_incident: &repo::FairnessIncidentRecord,
    handled_incident: &repo::FairnessIncidentRecord,
) -> AuditEvent {
    let ref_id = handled_incident
        .ref_id
        .clone()
        .or_else(|| handled_incident.fairness_incident_id.clone());
    let mut event = AuditEvent::business(
        "risk",
        handled_incident.ref_type.as_str(),
        ref_id,
        "risk.fairness_incident.handle",
        action,
        AuditContext {
            request_id: request_id.to_string(),
            trace_id,
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "fairness_incident_id": handled_incident.fairness_incident_id.clone(),
                "order_id": handled_incident.order_id.clone(),
                "incident_type": handled_incident.incident_type.clone(),
                "severity": handled_incident.severity.clone(),
                "lifecycle_stage": handled_incident.lifecycle_stage.clone(),
                "status_before": existing_incident.fairness_incident_status.clone(),
                "status_after": handled_incident.fairness_incident_status.clone(),
                "closed_at": handled_incident.closed_at.clone(),
                "resolution_summary": resolution_summary,
                "auto_action_override": auto_action_override,
                "freeze_settlement": freeze_settlement,
                "freeze_delivery": freeze_delivery,
                "create_dispute_suggestion": create_dispute_suggestion,
                "action_plan_status": action_plan_status,
                "step_up_token_present": step_up_token_present,
                "business_mutation_executed": false,
            }),
        },
    );
    event.sensitivity_level = "high".to_string();
    event
}

fn state_client(state: &AppState) -> Result<db::Client, (StatusCode, Json<ErrorResponse>)> {
    state.db.client().map_err(map_db_error)
}

fn map_db_error(err: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    internal_error(None, format!("audit persistence failed: {err}"))
}

fn internal_error(
    request_id: Option<String>,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: message.into(),
            request_id,
        }),
    )
}

fn bad_request(request_id: &str, message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
            message: message.into(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn not_found(request_id: &str, message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::AudEvidenceInvalid.as_str().to_string(),
            message: message.into(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn conflict_error(
    request_id: &str,
    code: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.into(),
            request_id: Some(request_id.to_string()),
        }),
    )
}

fn validate_uuid(
    raw: &str,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    validate_optional_uuid(Some(raw), field_name, request_id)
}

fn validate_optional_uuid(
    raw: Option<&str>,
    field_name: &str,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(raw) = raw {
        EntityId::parse(raw).map_err(|_| {
            bad_request(
                request_id,
                format!("{field_name} must be a valid uuid: {raw}"),
            )
        })?;
    }
    Ok(())
}

fn normalize_export_ref_type(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim() {
        "order" => Ok("order".to_string()),
        "case" | "dispute_case" => Ok("dispute_case".to_string()),
        other => Err(bad_request(
            request_id,
            format!("ref_type must be one of: order, case, dispute_case; got `{other}`"),
        )),
    }
}

fn normalize_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for audit package export",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_consistency_ref_type(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = match raw.trim() {
        "order" => "order",
        "contract" | "digital_contract" => "digital_contract",
        "delivery" | "delivery_record" => "delivery_record",
        "settlement" | "settlement_record" => "settlement_record",
        "payment" | "payment_intent" => "payment_intent",
        "refund" | "refund_intent" => "refund_intent",
        "payout" | "payout_instruction" => "payout_instruction",
        other => {
            return Err(bad_request(
                request_id,
                format!(
                    "refType must be one of: order, contract, digital_contract, delivery, delivery_record, settlement, settlement_record, payment, payment_intent, refund, refund_intent, payout, payout_instruction; got `{other}`"
                ),
            ));
        }
    };
    Ok(normalized.to_string())
}

fn normalize_consistency_reconcile_mode(
    raw: Option<&str>,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let mode = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("projection_gap");
    match mode {
        "projection_gap" | "full" => Ok(mode.to_string()),
        other => Err(bad_request(
            request_id,
            format!("mode must be one of: projection_gap, full; got `{other}`"),
        )),
    }
}

fn normalize_consistency_reconcile_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for ops consistency reconcile",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_external_fact_confirm_result(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = match raw.trim() {
        "confirmed" | "confirm" => "confirmed",
        "matched" | "match" => "matched",
        "mismatched" | "mismatch" => "mismatched",
        "rejected" | "reject" => "rejected",
        other => {
            return Err(bad_request(
                request_id,
                format!(
                    "confirm_result must be one of: confirmed, matched, mismatched, rejected; got `{other}`"
                ),
            ));
        }
    };
    Ok(normalized.to_string())
}

fn normalize_external_fact_confirm_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for ops external fact confirm",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_fairness_incident_action(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = match raw.trim() {
        "ack" | "acknowledge" | "review" => "acknowledge",
        "escalate" | "escalated" => "escalate",
        "close" | "resolve" | "resolved" => "close",
        other => {
            return Err(bad_request(
                request_id,
                format!("action must be one of: acknowledge, escalate, close; got `{other}`"),
            ));
        }
    };
    Ok(normalized.to_string())
}

fn normalize_fairness_incident_resolution_summary(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let resolution_summary = raw.trim();
    if resolution_summary.is_empty() {
        return Err(bad_request(
            request_id,
            "resolution_summary is required for fairness incident handle",
        ));
    }
    if resolution_summary.len() > 1000 {
        return Err(bad_request(
            request_id,
            "resolution_summary must be shorter than 1001 characters",
        ));
    }
    Ok(resolution_summary.to_string())
}

fn normalize_projection_gap_resolution_mode(
    raw: Option<&str>,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let resolution_mode = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual_close");
    if resolution_mode.len() > 64 {
        return Err(bad_request(
            request_id,
            "resolution_mode must be shorter than 65 characters",
        ));
    }
    Ok(resolution_mode.to_string())
}

fn normalize_projection_gap_expected_state_digest(
    raw: Option<&str>,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(digest) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if digest.len() > 128 {
        return Err(bad_request(
            request_id,
            "expected_state_digest must be shorter than 129 characters",
        ));
    }
    Ok(Some(digest.to_string()))
}

fn normalize_anchor_retry_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for audit anchor batch retry",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_optional_anchor_filter(
    raw: Option<&str>,
    field_name: &str,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    normalize_optional_filter(raw, field_name, request_id)
}

fn normalize_optional_filter(
    raw: Option<&str>,
    field_name: &str,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 128 {
        return Err(bad_request(
            request_id,
            format!("{field_name} must be shorter than 129 characters"),
        ));
    }
    Ok(Some(value.to_string()))
}

fn normalize_optional_long_text(
    raw: Option<&str>,
    field_name: &str,
    request_id: &str,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 1000 {
        return Err(bad_request(
            request_id,
            format!("{field_name} must be shorter than 1001 characters"),
        ));
    }
    Ok(Some(value.to_string()))
}

fn normalize_required_reason(
    raw: &str,
    request_id: &str,
    action_label: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            format!("reason is required for {action_label}"),
        ));
    }
    if reason.len() > 1000 {
        return Err(bad_request(
            request_id,
            "reason must be shorter than 1001 characters",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_ops_log_query(
    query: OpsLogMirrorQuery,
    request_id: &str,
) -> Result<OpsLogMirrorQuery, (StatusCode, Json<ErrorResponse>)> {
    validate_optional_uuid(query.object_id.as_deref(), "object_id", request_id)?;
    Ok(OpsLogMirrorQuery {
        service_name: normalize_optional_filter(
            query.service_name.as_deref(),
            "service_name",
            request_id,
        )?,
        log_level: normalize_optional_filter(query.log_level.as_deref(), "log_level", request_id)?,
        request_id: normalize_optional_filter(
            query.request_id.as_deref(),
            "request_id",
            request_id,
        )?,
        trace_id: normalize_optional_filter(query.trace_id.as_deref(), "trace_id", request_id)?,
        object_type: normalize_optional_filter(
            query.object_type.as_deref(),
            "object_type",
            request_id,
        )?,
        object_id: normalize_optional_filter(query.object_id.as_deref(), "object_id", request_id)?,
        from: normalize_optional_filter(query.from.as_deref(), "from", request_id)?,
        to: normalize_optional_filter(query.to.as_deref(), "to", request_id)?,
        query: normalize_optional_long_text(query.query.as_deref(), "query", request_id)?,
        page: query.page,
        page_size: query.page_size,
    })
}

fn normalize_ops_log_export_request(
    payload: OpsLogExportRequest,
    request_id: &str,
) -> Result<OpsLogExportRequest, (StatusCode, Json<ErrorResponse>)> {
    validate_optional_uuid(payload.object_id.as_deref(), "object_id", request_id)?;
    Ok(OpsLogExportRequest {
        reason: normalize_required_reason(payload.reason.as_str(), request_id, "ops log export")?,
        service_name: normalize_optional_filter(
            payload.service_name.as_deref(),
            "service_name",
            request_id,
        )?,
        log_level: normalize_optional_filter(
            payload.log_level.as_deref(),
            "log_level",
            request_id,
        )?,
        request_id: normalize_optional_filter(
            payload.request_id.as_deref(),
            "request_id",
            request_id,
        )?,
        trace_id: normalize_optional_filter(payload.trace_id.as_deref(), "trace_id", request_id)?,
        object_type: normalize_optional_filter(
            payload.object_type.as_deref(),
            "object_type",
            request_id,
        )?,
        object_id: normalize_optional_filter(
            payload.object_id.as_deref(),
            "object_id",
            request_id,
        )?,
        from: normalize_optional_filter(payload.from.as_deref(), "from", request_id)?,
        to: normalize_optional_filter(payload.to.as_deref(), "to", request_id)?,
        query: normalize_optional_long_text(payload.query.as_deref(), "query", request_id)?,
    })
}

fn require_log_export_selector(
    payload: &OpsLogExportRequest,
    request_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_selector = payload.service_name.is_some()
        || payload.log_level.is_some()
        || payload.request_id.is_some()
        || payload.trace_id.is_some()
        || payload.object_type.is_some()
        || payload.object_id.is_some()
        || payload.from.is_some()
        || payload.to.is_some()
        || payload.query.is_some();
    if has_selector {
        Ok(())
    } else {
        Err(bad_request(
            request_id,
            "ops log export requires at least one filter or time window selector",
        ))
    }
}

fn normalize_ops_alert_query(
    query: OpsAlertQuery,
    request_id: &str,
) -> Result<OpsAlertQuery, (StatusCode, Json<ErrorResponse>)> {
    Ok(OpsAlertQuery {
        alert_status: normalize_optional_filter(
            query.alert_status.as_deref(),
            "alert_status",
            request_id,
        )?,
        severity: normalize_optional_filter(query.severity.as_deref(), "severity", request_id)?,
        source_backend_key: normalize_optional_filter(
            query.source_backend_key.as_deref(),
            "source_backend_key",
            request_id,
        )?,
        alert_type: normalize_optional_filter(
            query.alert_type.as_deref(),
            "alert_type",
            request_id,
        )?,
        page: query.page,
        page_size: query.page_size,
    })
}

fn normalize_ops_incident_query(
    query: OpsIncidentQuery,
    request_id: &str,
) -> Result<OpsIncidentQuery, (StatusCode, Json<ErrorResponse>)> {
    Ok(OpsIncidentQuery {
        incident_status: normalize_optional_filter(
            query.incident_status.as_deref(),
            "incident_status",
            request_id,
        )?,
        severity: normalize_optional_filter(query.severity.as_deref(), "severity", request_id)?,
        owner_role_key: normalize_optional_filter(
            query.owner_role_key.as_deref(),
            "owner_role_key",
            request_id,
        )?,
        page: query.page,
        page_size: query.page_size,
    })
}

fn normalize_ops_slo_query(
    query: OpsSloQuery,
    request_id: &str,
) -> Result<OpsSloQuery, (StatusCode, Json<ErrorResponse>)> {
    Ok(OpsSloQuery {
        service_name: normalize_optional_filter(
            query.service_name.as_deref(),
            "service_name",
            request_id,
        )?,
        source_backend_key: normalize_optional_filter(
            query.source_backend_key.as_deref(),
            "source_backend_key",
            request_id,
        )?,
        status: normalize_optional_filter(query.status.as_deref(), "status", request_id)?,
        page: query.page,
        page_size: query.page_size,
    })
}

fn build_ops_log_export_audit_event(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
    actor_user_id: &str,
    reason: &str,
    object_uri: &str,
    object_hash: &str,
    exported_count: i64,
    step_up_challenge_id: Option<String>,
    step_up_token_present: bool,
    object_id: Option<String>,
) -> AuditEvent {
    let mut event = AuditEvent::business(
        "ops",
        "system_log_query",
        object_id,
        "ops.log.export",
        "exported",
        AuditContext {
            request_id: request_id.to_string(),
            trace_id: trace_id.to_string(),
            actor_type: "user".to_string(),
            actor_id: Some(actor_user_id.to_string()),
            actor_org_id: parse_uuid_header(headers, "x-tenant-id"),
            tenant_id: header(headers, "x-tenant-id").unwrap_or_else(|| "platform".to_string()),
            session_id: None,
            trusted_device_id: None,
            application_id: None,
            parent_audit_id: None,
            source_ip: None,
            client_fingerprint: None,
            auth_assurance_level: Some("step_up_required".to_string()),
            step_up_challenge_id,
            metadata: json!({
                "reason": reason,
                "object_uri": object_uri,
                "object_hash": object_hash,
                "exported_count": exported_count,
                "current_role": current_role(headers),
                "step_up_token_present": step_up_token_present,
            }),
        },
    );
    event.after_state_digest = Some(object_hash.to_string());
    event.evidence_hash = Some(object_hash.to_string());
    event.payload_digest = Some(object_hash.to_string());
    event.sensitivity_level = "high".to_string();
    event
}

fn observability_port(var: &str, default_port: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| default_port.to_string())
}

fn prometheus_base_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("PROMETHEUS_PORT", "9090")
    )
}

fn alertmanager_base_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("ALERTMANAGER_PORT", "9093")
    )
}

fn grafana_base_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("GRAFANA_PORT", "3000")
    )
}

fn loki_base_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("LOKI_PORT", "3100")
    )
}

fn tempo_base_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("TEMPO_PORT", "3200")
    )
}

fn otel_collector_health_url() -> String {
    format!(
        "http://127.0.0.1:{}",
        observability_port("OTEL_COLLECTOR_HEALTH_PORT", "13133")
    )
}

fn observability_backend_probe_url(backend_key: &str) -> Option<String> {
    match backend_key {
        "prometheus_main" => Some(format!("{}/-/ready", prometheus_base_url())),
        "alertmanager_main" => Some(format!("{}/-/ready", alertmanager_base_url())),
        "grafana_main" => Some(format!("{}/api/health", grafana_base_url())),
        "loki_main" => Some(format!("{}/ready", loki_base_url())),
        "tempo_main" => Some(format!("{}/metrics", tempo_base_url())),
        "otel_collector" => Some(otel_collector_health_url()),
        _ => None,
    }
}

fn observability_backend_detail_url(backend_key: &str) -> Option<String> {
    match backend_key {
        "prometheus_main" => Some(format!("{}/graph", prometheus_base_url())),
        "alertmanager_main" => Some(format!("{}/#/alerts", alertmanager_base_url())),
        "grafana_main" => Some(format!("{}/dashboards", grafana_base_url())),
        "loki_main" => Some(format!("{}/explore", grafana_base_url())),
        "tempo_main" => Some(tempo_base_url()),
        "otel_collector" => Some(otel_collector_health_url()),
        _ => None,
    }
}

async fn probe_observability_backend(
    backend: &repo::ObservabilityBackendRecord,
    checked_at: &str,
) -> ObservabilityBackendStatusView {
    let local_probe_url = observability_backend_probe_url(backend.backend_key.as_str());
    let detail_url = observability_backend_detail_url(backend.backend_key.as_str());
    let signals: Vec<String> = backend
        .capability_json
        .get("signals")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();
    let mut metadata = json!({
        "configured_endpoint_uri": backend.endpoint_uri,
        "signals": signals,
    });
    let (probe_status, http_status) = if let Some(probe_url) = local_probe_url.as_deref() {
        match reqwest::Client::new()
            .get(probe_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let healthy = response.status().is_success();
                if let Some(map) = metadata.as_object_mut() {
                    map.insert(
                        "status_family".to_string(),
                        Value::String(if healthy { "success" } else { "http_error" }.to_string()),
                    );
                }
                (
                    if healthy { "up" } else { "down" }.to_string(),
                    Some(status_code),
                )
            }
            Err(err) => {
                if let Some(map) = metadata.as_object_mut() {
                    map.insert("error".to_string(), Value::String(err.to_string()));
                }
                ("down".to_string(), None)
            }
        }
    } else {
        ("unknown".to_string(), None)
    };

    ObservabilityBackendStatusView {
        backend: ObservabilityBackendView::from(backend),
        probe_status,
        checked_at: Some(checked_at.to_string()),
        local_probe_url,
        http_status,
        detail_url,
        metadata,
    }
}

fn build_tempo_trace_link(trace_id: &str) -> Option<String> {
    Some(format!("{}/api/traces/{}", tempo_base_url(), trace_id))
}

fn build_grafana_trace_link(trace_id: &str) -> Option<String> {
    Some(format!(
        "{}/explore?traceId={}",
        grafana_base_url(),
        trace_id
    ))
}

async fn query_prometheus_up(job: &str) -> Option<f64> {
    let query = format!("up{{job=\"{job}\"}}");
    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/query", prometheus_base_url()))
        .query(&[("query", query.as_str())])
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body: Value = response.json().await.ok()?;
    body.get("data")
        .and_then(|data| data.get("result"))
        .and_then(|result| result.as_array())
        .and_then(|items| items.first())
        .and_then(|item| item.get("value"))
        .and_then(|value| value.as_array())
        .and_then(|value| value.get(1))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<f64>().ok())
}

async fn load_key_service_healths(checked_at: &str) -> Vec<OpsServiceHealthView> {
    let services = [
        ("platform-core", "Platform Core"),
        ("notification-worker", "Notification Worker"),
        ("outbox-publisher", "Outbox Publisher"),
    ];
    let mut result = Vec::with_capacity(services.len());
    for (job, display_name) in services {
        let observed_value = query_prometheus_up(job).await;
        let status = match observed_value {
            Some(value) if value >= 1.0 => "up",
            Some(_) => "down",
            None => "unknown",
        };
        result.push(OpsServiceHealthView {
            service_name: job.to_string(),
            status: status.to_string(),
            metric_name: format!("up{{job=\"{job}\"}}"),
            backend_key: "prometheus_main".to_string(),
            observed_value,
            checked_at: Some(checked_at.to_string()),
            detail_url: Some(format!("{}/graph", prometheus_base_url())),
            metadata: json!({
                "job": job,
                "display_name": display_name,
                "source": "prometheus_instant_query",
            }),
        });
    }
    result
}

fn log_export_bucket_name() -> String {
    std::env::var(LOG_EXPORT_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_LOG_EXPORT_BUCKET.to_string())
}

fn consistency_ref_type_candidates(ref_type: &str) -> Vec<String> {
    match ref_type {
        "order" => vec!["order".to_string()],
        "digital_contract" => vec!["digital_contract".to_string(), "contract".to_string()],
        "delivery_record" => vec!["delivery_record".to_string(), "delivery".to_string()],
        "settlement_record" => vec!["settlement_record".to_string(), "settlement".to_string()],
        "payment_intent" => vec!["payment_intent".to_string(), "payment".to_string()],
        "refund_intent" => vec!["refund_intent".to_string(), "refund".to_string()],
        "payout_instruction" => vec!["payout_instruction".to_string(), "payout".to_string()],
        _ => vec![ref_type.to_string()],
    }
}

fn consistency_aggregate_type_candidates(ref_type: &str) -> Vec<String> {
    match ref_type {
        "order" => vec!["order".to_string(), "trade.order_main".to_string()],
        "digital_contract" => vec![
            "digital_contract".to_string(),
            "contract".to_string(),
            "contract.digital_contract".to_string(),
        ],
        "delivery_record" => vec![
            "delivery_record".to_string(),
            "delivery".to_string(),
            "delivery.delivery_record".to_string(),
        ],
        "settlement_record" => vec![
            "settlement_record".to_string(),
            "settlement".to_string(),
            "billing.settlement_record".to_string(),
        ],
        "payment_intent" => vec![
            "payment_intent".to_string(),
            "payment".to_string(),
            "payment.payment_intent".to_string(),
        ],
        "refund_intent" => vec![
            "refund_intent".to_string(),
            "refund".to_string(),
            "payment.refund_intent".to_string(),
        ],
        "payout_instruction" => vec![
            "payout_instruction".to_string(),
            "payout".to_string(),
            "payment.payout_instruction".to_string(),
        ],
        _ => vec![ref_type.to_string()],
    }
}

fn build_consistency_chain_anchor_view(anchor: &repo::ChainAnchorRecord) -> Value {
    json!({
        "chain_anchor_id": anchor.chain_anchor_id,
        "chain_id": anchor.chain_id,
        "anchor_type": anchor.anchor_type,
        "ref_type": anchor.ref_type,
        "ref_id": anchor.ref_id,
        "digest": anchor.digest,
        "tx_hash": anchor.tx_hash,
        "status": anchor.status,
        "anchored_at": anchor.anchored_at,
        "created_at": anchor.created_at,
        "authority_model": anchor.authority_model,
        "reconcile_status": anchor.reconcile_status,
        "last_reconciled_at": anchor.last_reconciled_at,
    })
}

fn count_open_projection_gaps(counts: Option<&serde_json::Map<String, Value>>) -> i64 {
    counts
        .into_iter()
        .flat_map(|map| map.iter())
        .filter(|(status, _)| status.as_str() != "resolved")
        .filter_map(|(_, value)| value.as_i64())
        .sum()
}

fn trade_checkpoint_observed_at(
    checkpoint: &repo::TradeLifecycleCheckpointRecord,
) -> Option<String> {
    checkpoint
        .occurred_at
        .clone()
        .or_else(|| checkpoint.expected_by.clone())
        .or_else(|| checkpoint.created_at.clone())
}

fn external_fact_observed_at(receipt: &repo::ExternalFactReceiptRecord) -> Option<String> {
    receipt
        .confirmed_at
        .clone()
        .or_else(|| receipt.received_at.clone())
        .or_else(|| receipt.occurred_at.clone())
}

fn fairness_incident_observed_at(incident: &repo::FairnessIncidentRecord) -> Option<String> {
    incident
        .closed_at
        .clone()
        .or_else(|| incident.updated_at.clone())
        .or_else(|| incident.created_at.clone())
}

fn chain_projection_gap_observed_at(gap: &repo::ChainProjectionGapRecord) -> Option<String> {
    gap.resolved_at
        .clone()
        .or_else(|| gap.last_detected_at.clone())
        .or_else(|| gap.created_at.clone())
        .or_else(|| gap.first_detected_at.clone())
}

fn projection_gap_state_snapshot(gap: &repo::ChainProjectionGapRecord) -> Value {
    json!({
        "chain_projection_gap_id": gap.chain_projection_gap_id,
        "aggregate_type": gap.aggregate_type,
        "aggregate_id": gap.aggregate_id,
        "order_id": gap.order_id,
        "chain_id": gap.chain_id,
        "source_event_type": gap.source_event_type,
        "expected_tx_id": gap.expected_tx_id,
        "projected_tx_hash": gap.projected_tx_hash,
        "gap_type": gap.gap_type,
        "gap_status": gap.gap_status,
        "first_detected_at": gap.first_detected_at,
        "last_detected_at": gap.last_detected_at,
        "resolved_at": gap.resolved_at,
        "request_id": gap.request_id,
        "trace_id": gap.trace_id,
        "outbox_event_id": gap.outbox_event_id,
        "anchor_id": gap.anchor_id,
        "resolution_summary": gap.resolution_summary,
        "metadata": gap.metadata,
        "created_at": gap.created_at,
        "updated_at": gap.updated_at,
    })
}

fn projection_gap_state_digest(gap: &repo::ChainProjectionGapRecord) -> String {
    sha256_hex(projection_gap_state_snapshot(gap).to_string().as_bytes())
}

fn confirmed_chain_anchor_time(anchor: &repo::ChainAnchorRecord) -> Option<String> {
    match anchor.status.as_str() {
        "anchored" | "confirmed" | "committed" | "matched" => anchor
            .anchored_at
            .clone()
            .or_else(|| anchor.created_at.clone()),
        _ => None,
    }
}

fn latest_timestamp<I>(timestamps: I) -> Option<String>
where
    I: IntoIterator<Item = Option<String>>,
{
    timestamps.into_iter().flatten().max()
}

fn build_consistency_reconcile_subject_snapshot(
    subject: &repo::ConsistencySubjectRecord,
    latest_chain_anchor: Option<&repo::ChainAnchorRecord>,
    latest_receipt: Option<&repo::ExternalFactReceiptRecord>,
    latest_projection_gap: Option<&repo::ChainProjectionGapRecord>,
    recent_outbox_total: i64,
    recent_dead_letter_total: i64,
    recent_receipt_total: i64,
    projection_gap_status_breakdown: Value,
) -> Value {
    json!({
        "ref_type": subject.ref_type,
        "ref_id": subject.ref_id,
        "order_id": subject.order_id,
        "business_status": subject.business_status,
        "authority_model": subject.authority_model,
        "business_state_version": subject.business_state_version,
        "proof_commit_state": subject.proof_commit_state,
        "proof_commit_policy": subject.proof_commit_policy,
        "external_fact_status": subject.external_fact_status,
        "reconcile_status": subject.reconcile_status,
        "last_reconciled_at": subject.last_reconciled_at,
        "business_snapshot": subject.snapshot,
        "latest_chain_anchor": latest_chain_anchor.map(build_consistency_chain_anchor_view),
        "latest_receipt_id": latest_receipt.and_then(|receipt| receipt.external_fact_receipt_id.clone()),
        "latest_projection_gap_id": latest_projection_gap.and_then(|gap| gap.chain_projection_gap_id.clone()),
        "recent_outbox_total": recent_outbox_total,
        "recent_dead_letter_total": recent_dead_letter_total,
        "recent_receipt_total": recent_receipt_total,
        "projection_gap_status_breakdown": projection_gap_status_breakdown,
    })
}

fn build_consistency_reconcile_recommendations(
    mode: &str,
    subject: &repo::ConsistencySubjectRecord,
    projection_gaps: &[repo::ChainProjectionGapRecord],
    recent_outbox_events: &[repo::OutboxEventRecord],
    recent_dead_letters: &[repo::DeadLetterEventRecord],
    latest_receipt: Option<&repo::ExternalFactReceiptRecord>,
    latest_chain_anchor: Option<&repo::ChainAnchorRecord>,
) -> Vec<OpsConsistencyRepairRecommendationView> {
    let open_gaps: Vec<&repo::ChainProjectionGapRecord> = projection_gaps
        .iter()
        .filter(|gap| gap.gap_status != "resolved")
        .collect();
    let mut recommendations = Vec::new();

    for gap in &open_gaps {
        let (code, recommended_action, priority) = match gap.gap_type.as_str() {
            "missing_callback" => (
                "reconcile_missing_callback",
                "review fabric callback lineage and prepare reconcile worker preview",
                "high",
            ),
            "anchor_missing" => (
                "reconcile_missing_anchor",
                "review anchor submission lineage and prepare anchor-side projection repair",
                "high",
            ),
            "tx_hash_mismatch" | "digest_mismatch" => (
                "review_chain_anchor_mismatch",
                "compare expected chain digest / tx hash before queueing reconcile execution",
                "high",
            ),
            _ => (
                "reconcile_projection_gap",
                "prepare consistency reconcile preview for the recorded projection gap",
                "medium",
            ),
        };
        recommendations.push(OpsConsistencyRepairRecommendationView {
            code: code.to_string(),
            summary: format!(
                "projection gap `{}` remains `{}` for {} {}",
                gap.gap_type, gap.gap_status, subject.ref_type, subject.ref_id
            ),
            priority: priority.to_string(),
            recommended_action: recommended_action.to_string(),
            target_topic: Some(CONSISTENCY_RECONCILE_TARGET_TOPIC.to_string()),
            related_gap_id: gap.chain_projection_gap_id.clone(),
            metadata: json!({
                "aggregate_type": gap.aggregate_type,
                "aggregate_id": gap.aggregate_id,
                "order_id": gap.order_id,
                "chain_id": gap.chain_id,
                "source_event_type": gap.source_event_type,
                "expected_tx_id": gap.expected_tx_id,
                "projected_tx_hash": gap.projected_tx_hash,
                "outbox_event_id": gap.outbox_event_id,
                "anchor_id": gap.anchor_id,
                "proof_commit_state": subject.proof_commit_state,
                "external_fact_status": subject.external_fact_status,
                "reconcile_status": subject.reconcile_status,
            }),
        });
    }

    if mode == "full" {
        if subject.proof_commit_state != "anchored" && subject.proof_commit_state != "committed" {
            recommendations.push(OpsConsistencyRepairRecommendationView {
                code: "review_proof_commit_state".to_string(),
                summary: format!(
                    "proof commit state is `{}` for {} {}",
                    subject.proof_commit_state, subject.ref_type, subject.ref_id
                ),
                priority: if open_gaps.is_empty() {
                    "medium".to_string()
                } else {
                    "high".to_string()
                },
                recommended_action:
                    "check latest outbox publish / chain anchor lineage before reconcile execution"
                        .to_string(),
                target_topic: Some(CONSISTENCY_RECONCILE_TARGET_TOPIC.to_string()),
                related_gap_id: open_gaps
                    .first()
                    .and_then(|gap| gap.chain_projection_gap_id.clone()),
                metadata: json!({
                    "latest_outbox_event_id": recent_outbox_events
                        .first()
                        .and_then(|event| event.outbox_event_id.clone()),
                    "latest_chain_anchor_id": latest_chain_anchor
                        .and_then(|anchor| anchor.chain_anchor_id.clone()),
                    "proof_commit_policy": subject.proof_commit_policy,
                }),
            });
        }

        if subject.external_fact_status != "confirmed" && subject.external_fact_status != "matched"
        {
            recommendations.push(OpsConsistencyRepairRecommendationView {
                code: "review_external_fact_state".to_string(),
                summary: format!(
                    "external fact status is `{}` for {} {}",
                    subject.external_fact_status, subject.ref_type, subject.ref_id
                ),
                priority: "medium".to_string(),
                recommended_action:
                    "verify latest external fact receipt and keep dry-run reconcile evidence"
                        .to_string(),
                target_topic: Some(CONSISTENCY_RECONCILE_TARGET_TOPIC.to_string()),
                related_gap_id: open_gaps
                    .first()
                    .and_then(|gap| gap.chain_projection_gap_id.clone()),
                metadata: json!({
                    "latest_receipt_id": latest_receipt
                        .and_then(|receipt| receipt.external_fact_receipt_id.clone()),
                    "latest_receipt_status": latest_receipt.map(|receipt| receipt.receipt_status.clone()),
                    "receipt_provider_type": latest_receipt.map(|receipt| receipt.provider_type.clone()),
                }),
            });
        }

        if !recent_dead_letters.is_empty() {
            recommendations.push(OpsConsistencyRepairRecommendationView {
                code: "review_dead_letter_lineage".to_string(),
                summary: format!(
                    "{} recent dead letter record(s) still reference {} {}",
                    recent_dead_letters.len(),
                    subject.ref_type,
                    subject.ref_id
                ),
                priority: "high".to_string(),
                recommended_action:
                    "inspect dead letter lineage before any execution-mode reconcile is introduced"
                        .to_string(),
                target_topic: Some(CONSISTENCY_RECONCILE_TARGET_TOPIC.to_string()),
                related_gap_id: open_gaps
                    .first()
                    .and_then(|gap| gap.chain_projection_gap_id.clone()),
                metadata: json!({
                    "latest_dead_letter_id": recent_dead_letters
                        .first()
                        .and_then(|dead_letter| dead_letter.dead_letter_event_id.clone()),
                    "latest_dead_letter_stage": recent_dead_letters
                        .first()
                        .and_then(|dead_letter| dead_letter.failure_stage.clone()),
                    "latest_dead_letter_topic": recent_dead_letters
                        .first()
                        .and_then(|dead_letter| dead_letter.target_topic.clone()),
                }),
            });
        }
    }

    if recommendations.is_empty() {
        recommendations.push(OpsConsistencyRepairRecommendationView {
            code: "monitor_only".to_string(),
            summary: format!(
                "no open projection gap requires reconcile preview for {} {}",
                subject.ref_type, subject.ref_id
            ),
            priority: "low".to_string(),
            recommended_action:
                "keep monitoring dual-authority mirror fields until execution-mode reconcile is implemented"
                    .to_string(),
            target_topic: None,
            related_gap_id: None,
            metadata: json!({
                "proof_commit_state": subject.proof_commit_state,
                "external_fact_status": subject.external_fact_status,
                "reconcile_status": subject.reconcile_status,
            }),
        });
    }

    recommendations
}

fn summarize_consistency_recommendations(
    recommendations: &[OpsConsistencyRepairRecommendationView],
) -> Value {
    Value::Array(
        recommendations
            .iter()
            .map(|recommendation| {
                json!({
                    "code": recommendation.code,
                    "priority": recommendation.priority,
                    "related_gap_id": recommendation.related_gap_id,
                    "target_topic": recommendation.target_topic,
                })
            })
            .collect(),
    )
}

fn build_consistency_reconcile_plan(
    subject_snapshot: Value,
    mode: &str,
    reason: &str,
    dry_run: bool,
    recommendations: &[OpsConsistencyRepairRecommendationView],
    related_projection_gaps: &[ChainProjectionGapView],
    projection_gap_status_breakdown: Value,
) -> Value {
    json!({
        "mode": mode,
        "reason": reason,
        "dry_run": dry_run,
        "reconcile_target_topic": CONSISTENCY_RECONCILE_TARGET_TOPIC,
        "subject_snapshot": subject_snapshot,
        "projection_gap_status_breakdown": projection_gap_status_breakdown,
        "related_projection_gaps": related_projection_gaps,
        "recommendations": recommendations,
        "v1_constraints": {
            "dry_run_only": true,
            "formal_persistent_object": "ops.chain_projection_gap",
            "execution_worker_pending_tasks": ["AUD-013+", "AUD-021"],
        },
    })
}

fn normalize_replay_ref_type(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = match raw.trim() {
        "case" => "dispute_case",
        other => other,
    };
    if normalized.is_empty() {
        return Err(bad_request(
            request_id,
            "ref_type is required for audit replay",
        ));
    }
    Ok(normalized.to_string())
}

fn normalize_replay_type(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = match raw.trim() {
        "forensic" | "forensic_replay" => "forensic_replay",
        "state" | "state_replay" => "state_replay",
        "reconciliation" | "reconciliation_replay" => "reconciliation_replay",
        "compensation" | "compensation_replay" => "compensation_replay",
        other => {
            return Err(bad_request(
                request_id,
                format!(
                    "replay_type must be one of: forensic_replay, state_replay, reconciliation_replay, compensation_replay; got `{other}`"
                ),
            ));
        }
    };
    Ok(normalized.to_string())
}

fn normalize_replay_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for audit replay",
        ));
    }
    Ok(reason.to_string())
}

fn normalize_legal_hold_scope_type(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim() {
        "order" => Ok("order".to_string()),
        "case" | "dispute_case" => Ok("dispute_case".to_string()),
        other => Err(bad_request(
            request_id,
            format!("hold_scope_type must be one of: order, case, dispute_case; got `{other}`"),
        )),
    }
}

fn normalize_legal_hold_reason_code(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason_code = raw.trim();
    if reason_code.is_empty() {
        return Err(bad_request(
            request_id,
            "reason_code is required for audit legal hold",
        ));
    }
    Ok(reason_code.to_string())
}

fn normalize_legal_hold_release_reason(
    raw: &str,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let reason = raw.trim();
    if reason.is_empty() {
        return Err(bad_request(
            request_id,
            "reason is required for audit legal hold release",
        ));
    }
    Ok(reason.to_string())
}

fn replay_dry_run_only(request_id: &str) -> (StatusCode, Json<ErrorResponse>) {
    conflict_error(
        request_id,
        REPLAY_DRY_RUN_ONLY_ERROR,
        "V1 audit replay currently supports dry_run=true only",
    )
}

fn dead_letter_reprocess_dry_run_only(request_id: &str) -> (StatusCode, Json<ErrorResponse>) {
    conflict_error(
        request_id,
        DEAD_LETTER_REPROCESS_DRY_RUN_ONLY_ERROR,
        "V1 dead letter reprocess currently supports dry_run=true only",
    )
}

fn consistency_reconcile_dry_run_only(request_id: &str) -> (StatusCode, Json<ErrorResponse>) {
    conflict_error(
        request_id,
        CONSISTENCY_RECONCILE_DRY_RUN_ONLY_ERROR,
        "V1 ops consistency reconcile currently supports dry_run=true only",
    )
}

fn dead_letter_reprocess_not_supported(
    request_id: &str,
    dead_letter: &repo::DeadLetterEventRecord,
) -> (StatusCode, Json<ErrorResponse>) {
    conflict_error(
        request_id,
        DEAD_LETTER_REPROCESS_NOT_SUPPORTED_ERROR,
        format!(
            "dead letter `{}` is not a SEARCHREC consumer failure eligible for AUD-010 dry-run reprocess",
            dead_letter
                .dead_letter_event_id
                .as_deref()
                .unwrap_or("unknown")
        ),
    )
}

fn dead_letter_reprocess_state_conflict(
    request_id: &str,
    reprocess_status: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    conflict_error(
        request_id,
        DEAD_LETTER_REPROCESS_STATE_ERROR,
        format!(
            "dead letter reprocess is only allowed when reprocess_status=`not_reprocessed`; got `{reprocess_status}`"
        ),
    )
}

fn resolve_masked_level(
    raw: Option<&str>,
    request_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let level = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("summary");
    match level {
        "summary" | "masked" | "unmasked" => Ok(level.to_string()),
        other => Err(bad_request(
            request_id,
            format!("masked_level must be one of: summary, masked, unmasked; got `{other}`"),
        )),
    }
}

fn count_json_items(value: &Value) -> i64 {
    value
        .as_array()
        .map(|items| items.len() as i64)
        .unwrap_or(0)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn export_bucket_name() -> String {
    std::env::var(EXPORT_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_EXPORT_BUCKET.to_string())
}

fn current_role(headers: &HeaderMap) -> String {
    let raw_role = header(headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    canonical_role_key(raw_role.as_str()).to_string()
}

fn parse_uuid_header(headers: &HeaderMap, key: &str) -> Option<String> {
    header(headers, key).and_then(|value| {
        if EntityId::parse(&value).is_ok() {
            Some(value)
        } else {
            None
        }
    })
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
