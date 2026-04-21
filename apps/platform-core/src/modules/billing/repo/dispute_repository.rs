use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::models::{
    CreateDisputeCaseRequest, DisputeCaseView, DisputeEvidenceView, DisputeResolutionView,
    ResolveDisputeCaseRequest, UploadDisputeEvidenceRequest,
};
use crate::modules::billing::repo::dispute_linkage_repository::apply_dispute_open_linkage;
use crate::modules::delivery::repo::invalidate_delivery_cutoff_download_ticket_caches;
use crate::modules::storage::application::{delete_object, put_object_bytes};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const EVIDENCE_BUCKET_ENV: &str = "BUCKET_EVIDENCE_PACKAGES";
const DEFAULT_EVIDENCE_BUCKET: &str = "evidence-packages";

#[derive(Debug, Clone)]
struct OrderDisputeContext {
    order_id: String,
    buyer_org_id: String,
    seller_org_id: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
}

#[derive(Debug, Clone)]
struct DisputeCaseContext {
    case_id: String,
    order_id: String,
    buyer_org_id: String,
    seller_org_id: String,
    complainant_type: String,
    complainant_id: String,
    reason_code: String,
    status: String,
    decision_code: Option<String>,
    penalty_code: Option<String>,
    opened_at: String,
    resolved_at: Option<String>,
    updated_at: String,
    evidence_count: i64,
}

#[derive(Debug, Clone)]
struct ExistingDecisionContext {
    decision_id: String,
    decision_type: String,
    decision_code: String,
    liability_type: Option<String>,
    decision_text: Option<String>,
    decided_by: Option<String>,
    decided_at: String,
}

pub async fn create_dispute_case(
    client: &Client,
    payload: &CreateDisputeCaseRequest,
    tenant_scope_id: &str,
    actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DisputeCaseView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_order_dispute_context(&tx, &payload.order_id, request_id).await?;
    enforce_case_create_scope(tenant_scope_id, &context, request_id)?;
    ensure_order_allows_dispute(&context, request_id)?;
    ensure_no_active_case(&tx, &payload.order_id, &payload.reason_code, request_id).await?;

    let row = tx
        .query_one(
            r#"INSERT INTO support.dispute_case (
                   order_id,
                   complainant_type,
                   complainant_id,
                   reason_code,
                   status
                 ) VALUES (
                   $1::text::uuid,
                   'organization',
                   $2::text::uuid,
                   $3,
                   'opened'
                 )
                 RETURNING
                   case_id::text,
                   order_id::text,
                   complainant_type,
                   complainant_id::text,
                   reason_code,
                   status,
                   decision_code,
                   penalty_code,
                   to_char(opened_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                   to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[&payload.order_id, &tenant_scope_id, &payload.reason_code],
        )
        .await
        .map_err(map_db_error)?;
    let mut dispute_case = parse_case_row(&row, 0);

    let linkage = apply_dispute_open_linkage(
        &tx,
        &payload.order_id,
        &dispute_case.case_id,
        &payload.reason_code,
        actor_user_id,
        actor_role,
        request_id,
        trace_id,
    )
    .await?;

    let outbox_payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "pending_fabric_anchor",
        "case_id": dispute_case.case_id,
        "order_id": payload.order_id,
        "status": "opened",
        "reason_code": payload.reason_code,
        "complainant_type": "organization",
        "complainant_id": tenant_scope_id,
        "requested_resolution": payload.requested_resolution,
        "claimed_amount": payload.claimed_amount,
        "evidence_scope": payload.evidence_scope,
        "blocking_effect": payload.blocking_effect,
        "actor_user_id": actor_user_id,
        "request_metadata": payload.metadata,
        "linkage": {
            "freeze_ticket_id": &linkage.freeze_ticket_id,
            "legal_hold_id": &linkage.legal_hold_id,
            "settlement_freeze_count": linkage.settlement_freeze_count,
            "governance_action_count": linkage.governance_action_count,
            "order_delivery_status": &linkage.order_delivery_status,
            "order_acceptance_status": &linkage.order_acceptance_status,
            "order_settlement_status": &linkage.order_settlement_status,
        },
    });
    write_dispute_outbox(
        &tx,
        &dispute_case.case_id,
        "dispute.created",
        &outbox_payload,
        request_id,
        trace_id,
    )
    .await?;

    write_audit_event(
        &tx,
        "dispute",
        "case",
        &dispute_case.case_id,
        actor_role,
        "dispute.case.create",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    dispute_case.evidence_count = 0;
    tx.commit().await.map_err(map_db_error)?;
    invalidate_delivery_cutoff_download_ticket_caches(&linkage.delivery_cutoff_side_effects).await;
    Ok(dispute_case)
}

pub async fn upload_dispute_evidence(
    client: &Client,
    case_id: &str,
    payload: &UploadDisputeEvidenceRequest,
    tenant_scope_id: &str,
    actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DisputeEvidenceView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let case = load_dispute_case_context(&tx, case_id, request_id).await?;
    enforce_evidence_scope(tenant_scope_id, &case, request_id)?;
    if matches!(case.status.as_str(), "resolved" | "archived") {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "evidence upload is not allowed from dispute status `{}`",
                case.status
            ),
            request_id,
        ));
    }

    let object_hash = format!("{:x}", Sha256::digest(payload.file_bytes.as_slice()));
    if let Some(existing) =
        find_existing_evidence(&tx, case_id, &payload.object_type, &object_hash).await?
    {
        write_audit_event(
            &tx,
            "dispute",
            "evidence",
            &existing.evidence_id,
            actor_role,
            "dispute.evidence.upload.idempotent_replay",
            "idempotent_replay",
            request_id,
            trace_id,
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(DisputeEvidenceView {
            idempotent_replay: true,
            ..existing
        });
    }

    let evidence_id: String = tx
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .map_err(map_db_error)?
        .get(0);
    let bucket_name = evidence_bucket_name();
    let file_name = sanitize_file_name(&payload.file_name);
    let object_key = format!("cases/{case_id}/{evidence_id}/{file_name}");
    put_object_bytes(
        bucket_name.as_str(),
        object_key.as_str(),
        payload.file_bytes.clone(),
        payload.content_type.as_deref(),
    )
    .await?;

    let metadata = build_evidence_metadata(payload, actor_user_id, request_id);
    let row = match tx
        .query_one(
            r#"INSERT INTO support.evidence_object (
                   evidence_id,
                   case_id,
                   object_type,
                   object_uri,
                   object_hash,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3,
                   $4,
                   $5,
                   $6::jsonb
                 )
                 RETURNING
                   evidence_id::text,
                   case_id::text,
                   object_type,
                   object_uri,
                   object_hash,
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &evidence_id,
                &case_id,
                &payload.object_type,
                &format!("s3://{bucket_name}/{object_key}"),
                &object_hash,
                &metadata,
            ],
        )
        .await
    {
        Ok(row) => row,
        Err(err) => {
            let _ = delete_object(bucket_name.as_str(), object_key.as_str()).await;
            return Err(map_db_error(err));
        }
    };
    let evidence = parse_evidence_row(&row, false);

    if matches!(case.status.as_str(), "opened" | "evidence_collecting") {
        let _ = tx
            .execute(
                "UPDATE support.dispute_case
                 SET status = 'evidence_collecting',
                     updated_at = now()
                 WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await
            .map_err(map_db_error)?;
        let _ = tx
            .execute(
                "UPDATE trade.order_main
                 SET dispute_status = 'evidence_collecting',
                     updated_at = now(),
                     last_reason_code = 'billing_dispute_evidence_collecting'
                 WHERE order_id = $1::text::uuid",
                &[&case.order_id],
            )
            .await
            .map_err(map_db_error)?;
    }

    write_audit_event(
        &tx,
        "dispute",
        "evidence",
        &evidence.evidence_id,
        actor_role,
        "dispute.evidence.upload",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(evidence)
}

pub async fn resolve_dispute_case(
    client: &Client,
    case_id: &str,
    payload: &ResolveDisputeCaseRequest,
    actor_user_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DisputeResolutionView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let case = load_dispute_case_context(&tx, case_id, request_id).await?;
    if case.status == "archived" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "archived dispute case cannot be resolved again",
            request_id,
        ));
    }
    let actor_user_id = parse_uuid_text(actor_user_id).ok_or_else(|| {
        billing_error(
            StatusCode::BAD_REQUEST,
            "x-user-id is required for dispute resolve",
            request_id,
        )
    })?;
    let decision_type = payload
        .decision_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual_resolution")
        .to_string();
    let decision_code = payload.decision_code.trim().to_string();
    if decision_code.is_empty() {
        return Err(billing_error(
            StatusCode::BAD_REQUEST,
            "decision_code is required for dispute resolve",
            request_id,
        ));
    }

    if let Some(existing) = load_existing_decision(&tx, case_id).await? {
        if decision_matches(&existing, &decision_type, &decision_code, payload) {
            write_audit_event(
                &tx,
                "dispute",
                "case",
                case_id,
                actor_role,
                "dispute.case.resolve.idempotent_replay",
                "idempotent_replay",
                request_id,
                trace_id,
            )
            .await?;
            tx.commit().await.map_err(map_db_error)?;
            return Ok(DisputeResolutionView {
                case_id: case.case_id,
                order_id: case.order_id,
                current_status: "resolved".to_string(),
                decision_id: existing.decision_id,
                decision_type: existing.decision_type,
                decision_code: existing.decision_code,
                liability_type: existing.liability_type,
                penalty_code: case.penalty_code,
                decision_text: existing.decision_text,
                decided_by: existing.decided_by,
                decided_at: existing.decided_at,
                resolved_at: case.resolved_at.or_else(|| Some(case.updated_at)),
                step_up_bound: true,
                idempotent_replay: true,
            });
        }
        return Err(billing_error(
            StatusCode::CONFLICT,
            "dispute case is already resolved with a different decision",
            request_id,
        ));
    }

    let row = tx
        .query_one(
            r#"INSERT INTO support.decision_record (
                   case_id,
                   decision_type,
                   decision_code,
                   liability_type,
                   decision_text,
                   decided_by
                 ) VALUES (
                   $1::text::uuid,
                   $2,
                   $3,
                   $4,
                   $5,
                   $6::text::uuid
                 )
                 RETURNING
                   decision_id::text,
                   decision_type,
                   decision_code,
                   liability_type,
                   decision_text,
                   decided_by::text,
                   to_char(decided_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &case_id,
                &decision_type,
                &decision_code,
                &payload.liability_type,
                &payload.decision_text,
                &actor_user_id,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let decision = parse_decision_row(&row);

    let resolved_row = tx
        .query_one(
            r#"UPDATE support.dispute_case
               SET status = 'resolved',
                   decision_code = $2,
                   penalty_code = $3,
                   resolved_at = now(),
                   updated_at = now()
               WHERE case_id = $1::text::uuid
               RETURNING
                   to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[&case_id, &decision_code, &payload.penalty_code],
        )
        .await
        .map_err(map_db_error)?;
    let resolved_at: Option<String> = resolved_row.get(0);

    let _ = tx
        .execute(
            "UPDATE trade.order_main
             SET dispute_status = 'resolved',
                 updated_at = now(),
                 last_reason_code = 'billing_dispute_resolved'
             WHERE order_id = $1::text::uuid",
            &[&case.order_id],
        )
        .await
        .map_err(map_db_error)?;

    let outbox_payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "pending_fabric_anchor",
        "case_id": case.case_id,
        "order_id": case.order_id,
        "status": "resolved",
        "decision_id": decision.decision_id,
        "decision_type": decision.decision_type,
        "decision_code": decision.decision_code,
        "liability_type": decision.liability_type,
        "penalty_code": payload.penalty_code,
        "decision_text": decision.decision_text,
        "resolved_at": resolved_at,
        "actor_user_id": actor_user_id,
        "request_metadata": payload.metadata,
    });
    write_dispute_outbox(
        &tx,
        case_id,
        "dispute.resolved",
        &outbox_payload,
        request_id,
        trace_id,
    )
    .await?;

    write_audit_event(
        &tx,
        "dispute",
        "case",
        case_id,
        actor_role,
        "dispute.case.resolve",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(DisputeResolutionView {
        case_id: case.case_id,
        order_id: case.order_id,
        current_status: "resolved".to_string(),
        decision_id: decision.decision_id,
        decision_type: decision.decision_type,
        decision_code: decision.decision_code,
        liability_type: decision.liability_type,
        penalty_code: payload.penalty_code.clone(),
        decision_text: decision.decision_text,
        decided_by: decision.decided_by,
        decided_at: decision.decided_at,
        resolved_at,
        step_up_bound: true,
        idempotent_replay: false,
    })
}

async fn load_order_dispute_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderDisputeContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
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
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            billing_error(
                StatusCode::NOT_FOUND,
                &format!("order not found: {order_id}"),
                request_id,
            )
        })?;
    Ok(OrderDisputeContext {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        order_status: row.get(3),
        payment_status: row.get(4),
        delivery_status: row.get(5),
        acceptance_status: row.get(6),
        settlement_status: row.get(7),
        dispute_status: row.get(8),
    })
}

async fn ensure_no_active_case(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    reason_code: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let existing = client
        .query_opt(
            "SELECT case_id::text
             FROM support.dispute_case
             WHERE order_id = $1::text::uuid
               AND reason_code = $2
               AND status NOT IN ('resolved', 'archived')
             ORDER BY opened_at DESC
             LIMIT 1",
            &[&order_id, &reason_code],
        )
        .await
        .map_err(map_db_error)?;
    if existing.is_some() {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "an active dispute case already exists for the same order and reason_code",
            request_id,
        ));
    }
    Ok(())
}

async fn load_dispute_case_context(
    client: &(impl GenericClient + Sync),
    case_id: &str,
    request_id: Option<&str>,
) -> Result<DisputeCaseContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"SELECT
                 c.case_id::text,
                 c.order_id::text,
                 o.buyer_org_id::text,
                 o.seller_org_id::text,
                 c.complainant_type,
                 c.complainant_id::text,
                 c.reason_code,
                 c.status,
                 c.decision_code,
                 c.penalty_code,
                 to_char(c.opened_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                 to_char(c.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                 to_char(c.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                 COALESCE(ec.evidence_count, 0)::bigint
               FROM support.dispute_case c
               JOIN trade.order_main o ON o.order_id = c.order_id
               LEFT JOIN (
                 SELECT case_id, COUNT(*) AS evidence_count
                 FROM support.evidence_object
                 GROUP BY case_id
               ) ec ON ec.case_id = c.case_id
               WHERE c.case_id = $1::text::uuid"#,
            &[&case_id],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            billing_error(
                StatusCode::NOT_FOUND,
                &format!("dispute case not found: {case_id}"),
                request_id,
            )
        })?;
    Ok(parse_case_context_row(&row))
}

async fn find_existing_evidence(
    client: &(impl GenericClient + Sync),
    case_id: &str,
    object_type: &str,
    object_hash: &str,
) -> Result<Option<DisputeEvidenceView>, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_opt(
            r#"SELECT
                 evidence_id::text,
                 case_id::text,
                 object_type,
                 object_uri,
                 object_hash,
                 metadata,
                 to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
               FROM support.evidence_object
               WHERE case_id = $1::text::uuid
                 AND object_type = $2
                 AND object_hash = $3
               ORDER BY created_at DESC
               LIMIT 1"#,
            &[&case_id, &object_type, &object_hash],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.map(|item| parse_evidence_row(&item, false)))
}

async fn load_existing_decision(
    client: &(impl GenericClient + Sync),
    case_id: &str,
) -> Result<Option<ExistingDecisionContext>, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_opt(
            r#"SELECT
                 decision_id::text,
                 decision_type,
                 decision_code,
                 liability_type,
                 decision_text,
                 decided_by::text,
                 to_char(decided_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
               FROM support.decision_record
               WHERE case_id = $1::text::uuid"#,
            &[&case_id],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.map(|item| parse_decision_row(&item)))
}

fn decision_matches(
    existing: &ExistingDecisionContext,
    decision_type: &str,
    decision_code: &str,
    payload: &ResolveDisputeCaseRequest,
) -> bool {
    existing.decision_type == decision_type
        && existing.decision_code == decision_code
        && existing.liability_type == payload.liability_type
        && existing.decision_text == payload.decision_text
}

fn enforce_case_create_scope(
    tenant_scope_id: &str,
    context: &OrderDisputeContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if tenant_scope_id != context.buyer_org_id {
        return Err(billing_error(
            StatusCode::FORBIDDEN,
            "buyer tenant scope does not match dispute order",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_evidence_scope(
    tenant_scope_id: &str,
    case: &DisputeCaseContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if tenant_scope_id != case.buyer_org_id || case.complainant_id != tenant_scope_id {
        return Err(billing_error(
            StatusCode::FORBIDDEN,
            "buyer tenant scope does not match dispute case",
            request_id,
        ));
    }
    Ok(())
}

fn ensure_order_allows_dispute(
    context: &OrderDisputeContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let lifecycle_started = matches!(
        context.payment_status.as_str(),
        "paid" | "buyer_locked" | "refunded" | "failed" | "expired"
    ) || context.delivery_status != "not_started"
        || context.acceptance_status != "not_started"
        || context.settlement_status != "not_started";
    if !lifecycle_started || context.order_status == "created" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "order is not yet eligible for dispute: order_status=`{}`, payment_status=`{}`",
                context.order_status, context.payment_status
            ),
            request_id,
        ));
    }
    if matches!(
        context.dispute_status.as_str(),
        "opened" | "evidence_collecting" | "manual_review"
    ) {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "order already has active dispute status `{}`",
                context.dispute_status
            ),
            request_id,
        ));
    }
    Ok(())
}

fn build_evidence_metadata(
    payload: &UploadDisputeEvidenceRequest,
    actor_user_id: Option<&str>,
    request_id: Option<&str>,
) -> Value {
    json!({
        "file_name": payload.file_name,
        "content_type": payload.content_type,
        "size_bytes": payload.file_bytes.len(),
        "actor_user_id": actor_user_id,
        "request_id": request_id,
        "request_metadata": payload.metadata,
    })
}

fn evidence_bucket_name() -> String {
    std::env::var(EVIDENCE_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_EVIDENCE_BUCKET.to_string())
}

fn sanitize_file_name(raw: &str) -> String {
    let sanitized: String = raw
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "evidence.bin".to_string()
    } else {
        sanitized
    }
}

fn parse_case_row(row: &Row, evidence_count: i64) -> DisputeCaseView {
    DisputeCaseView {
        case_id: row.get(0),
        order_id: row.get(1),
        complainant_type: row.get(2),
        complainant_id: row.get(3),
        reason_code: row.get(4),
        current_status: row.get(5),
        decision_code: row.get(6),
        penalty_code: row.get(7),
        opened_at: row.get(8),
        resolved_at: row.get(9),
        updated_at: row.get(10),
        evidence_count,
    }
}

fn parse_case_context_row(row: &Row) -> DisputeCaseContext {
    DisputeCaseContext {
        case_id: row.get(0),
        order_id: row.get(1),
        buyer_org_id: row.get(2),
        seller_org_id: row.get(3),
        complainant_type: row.get(4),
        complainant_id: row.get(5),
        reason_code: row.get(6),
        status: row.get(7),
        decision_code: row.get(8),
        penalty_code: row.get(9),
        opened_at: row.get(10),
        resolved_at: row.get(11),
        updated_at: row.get(12),
        evidence_count: row.get(13),
    }
}

fn parse_evidence_row(row: &Row, idempotent_replay: bool) -> DisputeEvidenceView {
    DisputeEvidenceView {
        evidence_id: row.get(0),
        case_id: row.get(1),
        object_type: row.get(2),
        object_uri: row.get(3),
        object_hash: row.get(4),
        metadata: row.get(5),
        created_at: row.get(6),
        idempotent_replay,
    }
}

fn parse_decision_row(row: &Row) -> ExistingDecisionContext {
    ExistingDecisionContext {
        decision_id: row.get(0),
        decision_type: row.get(1),
        decision_code: row.get(2),
        liability_type: row.get(3),
        decision_text: row.get(4),
        decided_by: row.get(5),
        decided_at: row.get(6),
    }
}

fn parse_uuid_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() == 36 {
        Some(trimmed.to_string())
    } else {
        None
    }
}

async fn write_dispute_outbox(
    client: &(impl GenericClient + Sync),
    case_id: &str,
    event_type: &str,
    payload: &Value,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let occurred_at = match event_type {
        "dispute.resolved" => payload.get("resolved_at").and_then(Value::as_str),
        _ => None,
    };
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "support.dispute_case",
            aggregate_id: case_id,
            event_type,
            producer_service: "platform-core.billing",
            request_id,
            trace_id,
            idempotency_key: None,
            occurred_at,
            business_payload: payload,
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

fn billing_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: if status == StatusCode::FORBIDDEN {
                ErrorCode::IamUnauthorized.as_str().to_string()
            } else {
                ErrorCode::BilProviderFailed.as_str().to_string()
            },
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
