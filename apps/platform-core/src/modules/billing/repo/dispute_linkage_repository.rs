use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::delivery::repo::{DeliveryCutoffSideEffects, apply_delivery_cutoff_if_needed};
use axum::Json;
use axum::http::StatusCode;
use db::{GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct DisputeOpenLinkageResult {
    pub freeze_ticket_id: String,
    pub legal_hold_id: String,
    pub settlement_freeze_count: i64,
    pub governance_action_count: i64,
    pub order_delivery_status: String,
    pub order_acceptance_status: String,
    pub order_settlement_status: String,
    pub delivery_cutoff_side_effects: DeliveryCutoffSideEffects,
}

#[derive(Debug, Clone)]
struct OrderLinkageContext {
    order_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
}

pub async fn apply_dispute_open_linkage(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    case_id: &str,
    reason_code: &str,
    actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DisputeOpenLinkageResult, (StatusCode, Json<ErrorResponse>)> {
    let context = load_order_linkage_context(client, order_id, request_id).await?;
    let settlement_freeze_count = freeze_settlement_records(client, order_id, reason_code).await?;
    let delivery_cutoff_side_effects = apply_delivery_cutoff_if_needed(
        client,
        order_id,
        &context.order_status,
        &context.delivery_status,
        "opened",
        "billing_dispute_opened",
        actor_role,
        request_id,
        trace_id,
    )
    .await?;
    let requested_by = actor_user_id.and_then(parse_uuid_text);
    let freeze_ticket_id =
        create_dispute_freeze_ticket(client, order_id, reason_code, requested_by.as_deref())
            .await?;
    let legal_hold_id = create_dispute_legal_hold(
        client,
        order_id,
        case_id,
        reason_code,
        requested_by.as_deref(),
    )
    .await?;

    let order_delivery_status =
        derive_order_delivery_status(&context.delivery_status, &context.acceptance_status);
    let order_acceptance_status =
        derive_order_acceptance_status(&context.acceptance_status, &order_delivery_status);
    let order_settlement_status =
        derive_order_settlement_status(&context.settlement_status, settlement_freeze_count);

    client
        .execute(
            "UPDATE trade.order_main
             SET delivery_status = $2,
                 acceptance_status = $3,
                 settlement_status = $4,
                 dispute_status = 'opened',
                 updated_at = now(),
                 last_reason_code = 'billing_dispute_linkage_applied'
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &order_delivery_status,
                &order_acceptance_status,
                &order_settlement_status,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let governance_action_count = create_governance_actions(
        client,
        &freeze_ticket_id,
        order_id,
        case_id,
        &legal_hold_id,
        settlement_freeze_count,
        &order_delivery_status,
        &order_settlement_status,
    )
    .await?;

    if settlement_freeze_count > 0 {
        write_audit_event(
            client,
            "billing",
            "order",
            order_id,
            actor_role,
            "billing.settlement.freeze",
            "success",
            request_id,
            trace_id,
        )
        .await?;
    }
    write_audit_event(
        client,
        "audit",
        "order",
        order_id,
        actor_role,
        "audit.legal_hold.activate",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    Ok(DisputeOpenLinkageResult {
        freeze_ticket_id,
        legal_hold_id,
        settlement_freeze_count,
        governance_action_count,
        order_delivery_status,
        order_acceptance_status,
        order_settlement_status,
        delivery_cutoff_side_effects,
    })
}

async fn load_order_linkage_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderLinkageContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status, delivery_status, acceptance_status, settlement_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            linkage_error(
                StatusCode::NOT_FOUND,
                &format!("order not found: {order_id}"),
                request_id,
            )
        })?;
    Ok(OrderLinkageContext {
        order_status: row.get(0),
        delivery_status: row.get(1),
        acceptance_status: row.get(2),
        settlement_status: row.get(3),
    })
}

async fn freeze_settlement_records(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    reason_code: &str,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "WITH affected AS (
               UPDATE billing.settlement_record
               SET settlement_status = 'frozen',
                   reason_code = $2,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND settlement_status NOT IN ('settled', 'closed', 'refunded', 'canceled', 'frozen')
               RETURNING settlement_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &format!("dispute_opened:{reason_code}")],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.get(0))
}

async fn create_dispute_freeze_ticket(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    reason_code: &str,
    requested_by: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "INSERT INTO risk.freeze_ticket (
               ref_type,
               ref_id,
               freeze_type,
               status,
               reason_code,
               requested_by,
               executed_at
             ) VALUES (
               'order',
               $1::text::uuid,
               'dispute_hold',
               'executed',
               $2,
               $3::text::uuid,
               now()
             )
             RETURNING freeze_ticket_id::text",
            &[&order_id, &reason_code, &requested_by],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.get(0))
}

async fn create_dispute_legal_hold(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    case_id: &str,
    reason_code: &str,
    requested_by: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "INSERT INTO audit.legal_hold (
               hold_scope_type,
               hold_scope_id,
               reason_code,
               status,
               requested_by,
               metadata
             ) VALUES (
               'order',
               $1::text::uuid,
               $2,
               'active',
               $3::text::uuid,
               jsonb_build_object(
                 'case_id', $4::text::uuid,
                 'trigger', 'dispute_case_opened',
                 'retention_class', 'dispute_active'
               )
             )
             RETURNING legal_hold_id::text",
            &[&order_id, &reason_code, &requested_by, &case_id],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.get(0))
}

async fn create_governance_actions(
    client: &(impl GenericClient + Sync),
    freeze_ticket_id: &str,
    order_id: &str,
    case_id: &str,
    legal_hold_id: &str,
    settlement_freeze_count: i64,
    order_delivery_status: &str,
    order_settlement_status: &str,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    let action_payloads = [
        (
            "freeze_settlement",
            json!({
                "order_id": order_id,
                "case_id": case_id,
                "settlement_freeze_count": settlement_freeze_count,
                "order_settlement_status": order_settlement_status,
            }),
        ),
        (
            "suspend_delivery",
            json!({
                "order_id": order_id,
                "case_id": case_id,
                "order_delivery_status": order_delivery_status,
            }),
        ),
        (
            "activate_legal_hold",
            json!({
                "order_id": order_id,
                "case_id": case_id,
                "legal_hold_id": legal_hold_id,
            }),
        ),
    ];
    let mut inserted = 0_i64;
    for (action_type, payload) in action_payloads {
        let _: Row = client
            .query_one(
                "INSERT INTO risk.governance_action_log (
                   freeze_ticket_id,
                   action_type,
                   action_payload
                 ) VALUES (
                   $1::text::uuid,
                   $2,
                   $3::jsonb
                 )
                 RETURNING governance_action_log_id",
                &[&freeze_ticket_id, &action_type, &payload],
            )
            .await
            .map_err(map_db_error)?;
        inserted += 1;
    }
    Ok(inserted)
}

fn derive_order_delivery_status(current: &str, current_acceptance_status: &str) -> String {
    if matches!(
        current,
        "pending_delivery" | "in_progress" | "blocked" | "delivered"
    ) && !matches!(
        current_acceptance_status,
        "accepted" | "closed" | "expired" | "canceled"
    ) {
        return "blocked".to_string();
    }
    current.to_string()
}

fn derive_order_acceptance_status(current: &str, order_delivery_status: &str) -> String {
    if order_delivery_status == "blocked" && matches!(current, "not_started" | "pending_acceptance")
    {
        return "blocked".to_string();
    }
    current.to_string()
}

fn derive_order_settlement_status(current: &str, settlement_freeze_count: i64) -> String {
    if settlement_freeze_count > 0
        || matches!(current, "pending_settlement" | "in_progress" | "pending")
    {
        return "frozen".to_string();
    }
    current.to_string()
}

fn parse_uuid_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() == 36 {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn linkage_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
