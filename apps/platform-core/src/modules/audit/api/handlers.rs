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
    AuditReplayJobDetailView, AuditTracePageView, AuditTraceQuery, OpsConsistencyBusinessStateView,
    OpsConsistencyExternalFactStateView, OpsConsistencyProofStateView, OpsConsistencyView,
    OpsDeadLetterPageView, OpsDeadLetterQuery, OpsDeadLetterReprocessRequest,
    OpsDeadLetterReprocessView, OpsOutboxPageView, OpsOutboxQuery, OrderAuditQuery, OrderAuditView,
};
use crate::modules::audit::dto::{
    AnchorBatchView, ChainProjectionGapView, DeadLetterEventView, EvidenceManifestView,
    EvidencePackageView, ExternalFactReceiptView, LegalHoldView, OutboxEventView, ReplayJobView,
    ReplayResultView,
};
use crate::modules::audit::repo::{self, AccessAuditInsert, OrderAuditScope, SystemLogInsert};
use crate::modules::storage::application::{delete_object, put_object_bytes};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};

const EXPORT_BUCKET_ENV: &str = "BUCKET_EVIDENCE_PACKAGES";
const DEFAULT_EXPORT_BUCKET: &str = "evidence-packages";
const EXPORT_STEP_UP_ACTION: &str = "audit.package.export";
const EXPORT_STEP_UP_ACTION_COMPAT: &str = "audit.evidence.export";
const REPLAY_STEP_UP_ACTION: &str = "audit.replay.execute";
const LEGAL_HOLD_STEP_UP_ACTION: &str = "audit.legal_hold.manage";
const ANCHOR_STEP_UP_ACTION: &str = "audit.anchor.manage";
const DEAD_LETTER_REPROCESS_STEP_UP_ACTION: &str = "ops.dead_letter.reprocess";
const REPLAY_DRY_RUN_ONLY_ERROR: &str = "AUDIT_REPLAY_DRY_RUN_ONLY";
const DEAD_LETTER_REPROCESS_DRY_RUN_ONLY_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_DRY_RUN_ONLY";
const DEAD_LETTER_REPROCESS_NOT_SUPPORTED_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_NOT_SUPPORTED";
const DEAD_LETTER_REPROCESS_STATE_ERROR: &str = "AUDIT_DEAD_LETTER_REPROCESS_STATE_CONFLICT";
const LEGAL_HOLD_ACTIVE_ERROR: &str = "AUDIT_LEGAL_HOLD_ACTIVE";
const ANCHOR_BATCH_NOT_RETRYABLE_ERROR: &str = "AUDIT_ANCHOR_BATCH_NOT_RETRYABLE";

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
enum AuditPermission {
    TraceRead,
    OpsOutboxRead,
    OpsDeadLetterRead,
    OpsConsistencyRead,
    OpsDeadLetterReprocess,
    PackageExport,
    ReplayExecute,
    ReplayRead,
    LegalHoldManage,
    AnchorRead,
    AnchorManage,
}

fn is_allowed(role: &str, permission: AuditPermission) -> bool {
    match permission {
        AuditPermission::TraceRead => matches!(
            role,
            "tenant_admin"
                | "tenant_audit_readonly"
                | "platform_admin"
                | "platform_auditor"
                | "platform_audit_security"
                | "platform_reviewer"
                | "platform_risk_settlement"
                | "audit_admin"
                | "subject_reviewer"
                | "product_reviewer"
                | "compliance_reviewer"
                | "risk_operator"
                | "data_custody_admin"
                | "regulator_readonly"
                | "regulator_observer"
        ),
        AuditPermission::OpsOutboxRead => matches!(
            role,
            "platform_admin"
                | "platform_audit_security"
                | "consistency_operator"
                | "node_ops_admin"
        ),
        AuditPermission::OpsDeadLetterRead => matches!(
            role,
            "platform_admin"
                | "platform_audit_security"
                | "consistency_operator"
                | "node_ops_admin"
                | "audit_admin"
        ),
        AuditPermission::OpsConsistencyRead => matches!(
            role,
            "platform_admin"
                | "platform_audit_security"
                | "consistency_operator"
                | "node_ops_admin"
                | "audit_admin"
        ),
        AuditPermission::OpsDeadLetterReprocess => matches!(
            role,
            "platform_admin"
                | "platform_audit_security"
                | "consistency_operator"
                | "node_ops_admin"
                | "audit_admin"
        ),
        AuditPermission::PackageExport => matches!(
            role,
            "platform_admin" | "platform_auditor" | "platform_audit_security" | "audit_admin"
        ),
        AuditPermission::ReplayExecute
        | AuditPermission::ReplayRead
        | AuditPermission::LegalHoldManage
        | AuditPermission::AnchorRead
        | AuditPermission::AnchorManage => matches!(
            role,
            "platform_admin" | "platform_auditor" | "platform_audit_security" | "audit_admin"
        ),
    }
}

fn is_tenant_scoped_role(role: &str) -> bool {
    matches!(role, "tenant_admin" | "tenant_audit_readonly")
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
            message: format!("{action} is forbidden for current role"),
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
    header(headers, "x-role").unwrap_or_else(|| "unknown".to_string())
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
