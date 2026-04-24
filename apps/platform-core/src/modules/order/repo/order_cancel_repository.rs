use crate::modules::delivery::repo::{
    apply_delivery_cutoff_if_needed, invalidate_delivery_cutoff_download_ticket_caches,
};
use crate::modules::order::domain::derive_closed_layered_status_by_reason;
use crate::modules::order::dto::CancelOrderResponseData;
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub struct CancelOrderContext {
    pub buyer_org_id: String,
    pub seller_org_id: String,
}

pub async fn load_order_cancel_context(
    client: &Client,
    order_id: &str,
) -> Result<Option<CancelOrderContext>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT buyer_org_id::text, seller_org_id::text
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|v| CancelOrderContext {
        buyer_org_id: v.get(0),
        seller_org_id: v.get(1),
    }))
}

pub async fn cancel_order_with_state_machine(
    client: &mut Client,
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<CancelOrderResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT status, payment_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid
             FOR UPDATE",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };

    let current_status: String = row.get(0);
    let current_payment_status: String = row.get(1);
    let Some(transition) = derive_cancel_transition(&current_status, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "ORDER_CANCEL_FORBIDDEN".to_string(),
                message: format!(
                    "ORDER_CANCEL_FORBIDDEN: current state `{current_status}` is not cancelable"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };

    let updated_row = tx
        .query_one(
            "UPDATE trade.order_main
             SET status = 'closed',
                 payment_status = $2,
                 delivery_status = $3,
                 acceptance_status = $4,
                 settlement_status = $5,
                 dispute_status = $6,
                 last_reason_code = $7,
                 closed_at = now()
             WHERE order_id = $1::text::uuid
             RETURNING to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &transition.target_payment_status,
                &transition.layered_status.delivery_status,
                &transition.layered_status.acceptance_status,
                &transition.layered_status.settlement_status,
                &transition.layered_status.dispute_status,
                &transition.reason_code,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let canceled_at: String = updated_row.get(0);

    apply_authorization_cutoff_if_needed(
        &tx,
        order_id,
        "closed",
        &transition.layered_status.delivery_status,
        &transition.layered_status.dispute_status,
        transition.reason_code,
        actor_role,
        request_id,
        trace_id,
    )
    .await?;
    let delivery_cutoff = apply_delivery_cutoff_if_needed(
        &tx,
        order_id,
        "closed",
        &transition.layered_status.delivery_status,
        &transition.layered_status.dispute_status,
        transition.reason_code,
        actor_role,
        request_id,
        trace_id,
    )
    .await?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        "trade.order.cancel",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    invalidate_delivery_cutoff_download_ticket_caches(&delivery_cutoff).await;

    Ok(CancelOrderResponseData {
        order_id: order_id.to_string(),
        previous_state: current_status,
        current_state: "closed".to_string(),
        payment_status: transition.target_payment_status,
        refund_branch: transition.refund_branch.to_string(),
        refund_required: transition.refund_required,
        reason_code: transition.reason_code.to_string(),
        canceled_at,
    })
}

struct CancelTransition {
    target_payment_status: String,
    layered_status: crate::modules::order::domain::LayeredOrderStatus,
    refund_branch: &'static str,
    refund_required: bool,
    reason_code: &'static str,
}

fn derive_cancel_transition(status: &str, payment_status: &str) -> Option<CancelTransition> {
    match status {
        "created" => Some(CancelTransition {
            target_payment_status: payment_status.to_string(),
            layered_status: derive_closed_layered_status_by_reason("order_cancel_before_lock"),
            refund_branch: "no_refund",
            refund_required: false,
            reason_code: "order_cancel_before_lock",
        }),
        "buyer_locked" => {
            if payment_status == "paid" {
                Some(CancelTransition {
                    target_payment_status: "refund_pending".to_string(),
                    layered_status: derive_closed_layered_status_by_reason(
                        "order_cancel_refund_required_after_lock",
                    ),
                    refund_branch: "refund_required",
                    refund_required: true,
                    reason_code: "order_cancel_refund_required_after_lock",
                })
            } else {
                Some(CancelTransition {
                    target_payment_status: payment_status.to_string(),
                    layered_status: derive_closed_layered_status_by_reason(
                        "order_cancel_before_payment_success",
                    ),
                    refund_branch: "no_refund",
                    refund_required: false,
                    reason_code: "order_cancel_before_payment_success",
                })
            }
        }
        "payment_failed_pending_resolution" => Some(CancelTransition {
            target_payment_status: "failed".to_string(),
            layered_status: derive_closed_layered_status_by_reason(
                "order_cancel_after_payment_failed",
            ),
            refund_branch: "no_refund",
            refund_required: false,
            reason_code: "order_cancel_after_payment_failed",
        }),
        "payment_timeout_pending_compensation_cancel" => Some(CancelTransition {
            target_payment_status: "expired".to_string(),
            layered_status: derive_closed_layered_status_by_reason(
                "order_cancel_after_payment_timeout",
            ),
            refund_branch: "no_refund",
            refund_required: false,
            reason_code: "order_cancel_after_payment_timeout",
        }),
        _ => None,
    }
}
