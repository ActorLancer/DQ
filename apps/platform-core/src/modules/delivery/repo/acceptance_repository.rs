use super::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use super::outbox_repository::write_billing_trigger_bridge_event;
use crate::modules::billing::repo::billing_adjustment_repository::ensure_provisional_dispute_hold_in_tx;
use crate::modules::delivery::domain::{
    is_manual_acceptance_state, manual_acceptance_delivery_branch,
};
use crate::modules::delivery::dto::{
    AcceptOrderRequest, OrderAcceptanceResponseData, RejectOrderRequest,
};
use crate::modules::order::repo::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

const DELIVERY_ACCEPT_EVENT: &str = "delivery.accept";
const DELIVERY_REJECT_EVENT: &str = "delivery.reject";
const TRADE_ACCEPT_EVENT: &str = "trade.order.accept";
const TRADE_REJECT_EVENT: &str = "trade.order.reject";
const ACCEPT_REASON_CODE: &str = "delivery_accept_passed";

pub async fn accept_order_delivery(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &AcceptOrderRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<OrderAcceptanceResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let ctx = load_acceptance_context(&tx, order_id, request_id).await?;
    enforce_acceptance_scope(actor_role, tenant_id, &ctx.buyer_org_id, request_id)?;

    let delivery_branch = delivery_branch_for_sku(&ctx.sku_type).ok_or_else(|| {
        conflict(
            &format!(
                "ORDER_ACCEPT_FORBIDDEN: sku_type `{}` does not require manual acceptance",
                ctx.sku_type
            ),
            request_id,
        )
    })?;

    if ctx.current_state == "accepted" && ctx.delivery_record_status.as_deref() == Some("accepted")
    {
        if let Some(delivery_id) = ctx.delivery_id.as_deref() {
            write_delivery_audit_event(
                &tx,
                "delivery_record",
                delivery_id,
                actor_role,
                DELIVERY_ACCEPT_EVENT,
                "already_accepted",
                request_id,
                trace_id,
                acceptance_audit_metadata(
                    order_id,
                    &ctx,
                    delivery_branch,
                    "accepted",
                    ACCEPT_REASON_CODE,
                    None,
                    payload.note.as_deref(),
                    payload.verification_summary.as_ref(),
                ),
            )
            .await?;
        }
        tx.commit().await.map_err(map_db_error)?;
        return Ok(build_response(
            order_id,
            ctx,
            delivery_branch,
            "accept",
            "accepted",
            "accepted",
            ACCEPT_REASON_CODE,
            payload.note.clone(),
            Some("already_accepted".to_string()),
        ));
    }

    if !is_accept_allowed(&ctx.sku_type, &ctx.current_state) {
        return Err(conflict(
            &format!(
                "ORDER_ACCEPT_FORBIDDEN: current_state `{}` cannot be accepted for sku_type `{}`",
                ctx.current_state, ctx.sku_type
            ),
            request_id,
        ));
    }

    let delivery_id = ctx.delivery_id.clone().ok_or_else(|| {
        conflict(
            "ORDER_ACCEPT_FORBIDDEN: delivery record not found for delivered order",
            request_id,
        )
    })?;
    let acceptance_snapshot = json!({
        "decision": "accepted",
        "reason_code": ACCEPT_REASON_CODE,
        "note": payload.note,
        "verification_summary": payload.verification_summary,
    });
    let accepted_row = tx
        .query_one(
            "UPDATE trade.order_main
             SET status = 'accepted',
                 delivery_status = 'delivered',
                 acceptance_status = 'accepted',
                 settlement_status = CASE
                   WHEN payment_status = 'paid' THEN 'pending_settlement'
                   ELSE 'not_started'
                 END,
                 dispute_status = 'none',
                 last_reason_code = $2,
                 accepted_at = COALESCE(accepted_at, now()),
                 updated_at = now()
             WHERE order_id = $1::text::uuid
             RETURNING to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       settlement_status",
            &[&order_id, &ACCEPT_REASON_CODE],
        )
        .await
        .map_err(map_db_error)?;
    let accepted_at: String = accepted_row.get(0);
    let processed_at: String = accepted_row.get(1);
    let settlement_status: String = accepted_row.get(2);

    tx.execute(
        "UPDATE delivery.delivery_record
         SET status = 'accepted',
             trust_boundary_snapshot = COALESCE(trust_boundary_snapshot, '{}'::jsonb)
               || jsonb_build_object('acceptance', $2::jsonb),
             updated_at = now()
         WHERE delivery_id = $1::text::uuid",
        &[&delivery_id, &acceptance_snapshot],
    )
    .await
    .map_err(map_db_error)?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        TRADE_ACCEPT_EVENT,
        "success",
        request_id,
        trace_id,
    )
    .await?;
    write_delivery_audit_event(
        &tx,
        "delivery_record",
        &delivery_id,
        actor_role,
        DELIVERY_ACCEPT_EVENT,
        "success",
        request_id,
        trace_id,
        acceptance_audit_metadata(
            order_id,
            &ctx,
            delivery_branch,
            "accepted",
            ACCEPT_REASON_CODE,
            None,
            payload.note.as_deref(),
            payload.verification_summary.as_ref(),
        ),
    )
    .await?;
    let billing_bridge_idempotency_key = format!("billing-trigger:acceptance-passed:{delivery_id}");
    write_billing_trigger_bridge_event(
        &tx,
        order_id,
        "acceptance_passed",
        "delivery_record",
        &delivery_id,
        DELIVERY_ACCEPT_EVENT,
        actor_role,
        request_id,
        trace_id,
        billing_bridge_idempotency_key.as_str(),
        json!({
            "delivery_branch": delivery_branch,
            "delivery_id": delivery_id,
            "reason_code": ACCEPT_REASON_CODE,
            "accepted_at": accepted_at,
            "note": payload.note,
            "verification_summary": payload.verification_summary,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(OrderAcceptanceResponseData {
        order_id: order_id.to_string(),
        delivery_id: Some(delivery_id),
        delivery_branch: Some(delivery_branch.to_string()),
        action: "accept".to_string(),
        previous_state: ctx.current_state,
        current_state: "accepted".to_string(),
        payment_status: ctx.payment_status,
        delivery_status: "delivered".to_string(),
        acceptance_status: "accepted".to_string(),
        settlement_status,
        dispute_status: "none".to_string(),
        reason_code: ACCEPT_REASON_CODE.to_string(),
        reason_detail: payload.note.clone(),
        accepted_at: Some(accepted_at),
        processed_at,
        operation: Some("accepted".to_string()),
    })
}

pub async fn reject_order_delivery(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &RejectOrderRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<OrderAcceptanceResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_reject_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let ctx = load_acceptance_context(&tx, order_id, request_id).await?;
    enforce_acceptance_scope(actor_role, tenant_id, &ctx.buyer_org_id, request_id)?;

    let delivery_branch = delivery_branch_for_sku(&ctx.sku_type).ok_or_else(|| {
        conflict(
            &format!(
                "ORDER_REJECT_FORBIDDEN: sku_type `{}` does not require manual acceptance",
                ctx.sku_type
            ),
            request_id,
        )
    })?;

    if ctx.current_state == "rejected" && ctx.delivery_record_status.as_deref() == Some("rejected")
    {
        if let Some(delivery_id) = ctx.delivery_id.as_deref() {
            write_delivery_audit_event(
                &tx,
                "delivery_record",
                delivery_id,
                actor_role,
                DELIVERY_REJECT_EVENT,
                "already_rejected",
                request_id,
                trace_id,
                acceptance_audit_metadata(
                    order_id,
                    &ctx,
                    delivery_branch,
                    "rejected",
                    payload.reason_code.as_str(),
                    payload.reason_detail.as_deref(),
                    None,
                    payload.verification_summary.as_ref(),
                ),
            )
            .await?;
        }
        tx.commit().await.map_err(map_db_error)?;
        return Ok(build_response(
            order_id,
            ctx,
            delivery_branch,
            "reject",
            "rejected",
            "rejected",
            payload.reason_code.as_str(),
            payload.reason_detail.clone(),
            Some("already_rejected".to_string()),
        ));
    }

    if !is_reject_allowed(&ctx.sku_type, &ctx.current_state) {
        return Err(conflict(
            &format!(
                "ORDER_REJECT_FORBIDDEN: current_state `{}` cannot be rejected for sku_type `{}`",
                ctx.current_state, ctx.sku_type
            ),
            request_id,
        ));
    }

    let delivery_id = ctx.delivery_id.clone().ok_or_else(|| {
        conflict(
            "ORDER_REJECT_FORBIDDEN: delivery record not found for delivered order",
            request_id,
        )
    })?;
    let rejection_snapshot = json!({
        "decision": "rejected",
        "reason_code": payload.reason_code,
        "reason_detail": payload.reason_detail,
        "verification_summary": payload.verification_summary,
    });
    let processed_at: String = tx
        .query_one(
            "UPDATE trade.order_main
             SET status = 'rejected',
                 delivery_status = 'delivered',
                 acceptance_status = 'rejected',
                 settlement_status = 'blocked',
                 dispute_status = 'open',
                 last_reason_code = $2,
                 updated_at = now()
             WHERE order_id = $1::text::uuid
             RETURNING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&order_id, &payload.reason_code],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    tx.execute(
        "UPDATE delivery.delivery_record
         SET status = 'rejected',
             trust_boundary_snapshot = COALESCE(trust_boundary_snapshot, '{}'::jsonb)
               || jsonb_build_object('acceptance', $2::jsonb),
             updated_at = now()
         WHERE delivery_id = $1::text::uuid",
        &[&delivery_id, &rejection_snapshot],
    )
    .await
    .map_err(map_db_error)?;

    ensure_provisional_dispute_hold_in_tx(
        &tx,
        order_id,
        payload.reason_code.as_str(),
        DELIVERY_REJECT_EVENT,
        actor_role,
        request_id,
        trace_id,
    )
    .await?;

    tx.execute(
        "UPDATE trade.order_main
         SET settlement_status = 'blocked',
             updated_at = now()
         WHERE order_id = $1::text::uuid",
        &[&order_id],
    )
    .await
    .map_err(map_db_error)?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        TRADE_REJECT_EVENT,
        "success",
        request_id,
        trace_id,
    )
    .await?;
    write_delivery_audit_event(
        &tx,
        "delivery_record",
        &delivery_id,
        actor_role,
        DELIVERY_REJECT_EVENT,
        "success",
        request_id,
        trace_id,
        acceptance_audit_metadata(
            order_id,
            &ctx,
            delivery_branch,
            "rejected",
            payload.reason_code.as_str(),
            payload.reason_detail.as_deref(),
            None,
            payload.verification_summary.as_ref(),
        ),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(OrderAcceptanceResponseData {
        order_id: order_id.to_string(),
        delivery_id: Some(delivery_id),
        delivery_branch: Some(delivery_branch.to_string()),
        action: "reject".to_string(),
        previous_state: ctx.current_state,
        current_state: "rejected".to_string(),
        payment_status: ctx.payment_status,
        delivery_status: "delivered".to_string(),
        acceptance_status: "rejected".to_string(),
        settlement_status: "blocked".to_string(),
        dispute_status: "open".to_string(),
        reason_code: payload.reason_code.clone(),
        reason_detail: payload.reason_detail.clone(),
        accepted_at: ctx.accepted_at,
        processed_at,
        operation: Some("rejected".to_string()),
    })
}

#[derive(Debug, Clone)]
struct AcceptanceContext {
    current_state: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    sku_type: String,
    accepted_at: Option<String>,
    delivery_id: Option<String>,
    delivery_record_status: Option<String>,
    processed_at: String,
}

async fn load_acceptance_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<AcceptanceContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               o.buyer_org_id::text,
               s.sku_type,
               to_char(o.accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               latest.delivery_id::text,
               latest.status,
               to_char(o.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT delivery_id, status
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
               ORDER BY committed_at DESC NULLS LAST, created_at DESC, delivery_id DESC
               LIMIT 1
             ) latest ON true
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    Ok(AcceptanceContext {
        current_state: row.get(0),
        payment_status: row.get(1),
        delivery_status: row.get(2),
        acceptance_status: row.get(3),
        settlement_status: row.get(4),
        dispute_status: row.get(5),
        buyer_org_id: row.get(6),
        sku_type: row.get(7),
        accepted_at: row.get(8),
        delivery_id: row.get(9),
        delivery_record_status: row.get(10),
        processed_at: row.get(11),
    })
}

fn validate_reject_request(
    payload: &RejectOrderRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload.reason_code.trim().is_empty() {
        return Err(bad_request("reason_code is required", request_id));
    }
    Ok(())
}

fn delivery_branch_for_sku(sku_type: &str) -> Option<&'static str> {
    manual_acceptance_delivery_branch(sku_type)
}

fn is_accept_allowed(sku_type: &str, current_state: &str) -> bool {
    is_manual_acceptance_state(sku_type, current_state)
}

fn is_reject_allowed(sku_type: &str, current_state: &str) -> bool {
    is_accept_allowed(sku_type, current_state)
}

fn enforce_acceptance_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(
        actor_role,
        "tenant_admin" | "buyer_operator" | "procurement_manager"
    ) && tenant_id == Some(buyer_org_id)
    {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "order acceptance is forbidden for tenant scope".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

fn acceptance_audit_metadata(
    order_id: &str,
    ctx: &AcceptanceContext,
    delivery_branch: &str,
    decision: &str,
    reason_code: &str,
    reason_detail: Option<&str>,
    note: Option<&str>,
    verification_summary: Option<&Value>,
) -> Value {
    json!({
        "order_id": order_id,
        "sku_type": ctx.sku_type,
        "delivery_branch": delivery_branch,
        "previous_state": ctx.current_state,
        "decision": decision,
        "reason_code": reason_code,
        "reason_detail": reason_detail,
        "note": note,
        "verification_summary": verification_summary,
    })
}

fn build_response(
    order_id: &str,
    ctx: AcceptanceContext,
    delivery_branch: &str,
    action: &str,
    current_state: &str,
    acceptance_status: &str,
    reason_code: &str,
    reason_detail: Option<String>,
    operation: Option<String>,
) -> OrderAcceptanceResponseData {
    OrderAcceptanceResponseData {
        order_id: order_id.to_string(),
        delivery_id: ctx.delivery_id,
        delivery_branch: Some(delivery_branch.to_string()),
        action: action.to_string(),
        previous_state: ctx.current_state,
        current_state: current_state.to_string(),
        payment_status: ctx.payment_status,
        delivery_status: ctx.delivery_status,
        acceptance_status: acceptance_status.to_string(),
        settlement_status: ctx.settlement_status,
        dispute_status: ctx.dispute_status,
        reason_code: reason_code.to_string(),
        reason_detail,
        accepted_at: ctx.accepted_at,
        processed_at: ctx.processed_at,
        operation,
    }
}
