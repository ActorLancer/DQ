use crate::modules::order::domain::{LayeredOrderStatus, derive_layered_status};
use crate::modules::order::dto::{FileStdTransitionRequest, FileStdTransitionResponseData};
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::ensure_order_deliverable_and_prepare_delivery;
use crate::modules::order::repo::ensure_pre_payment_lock_checks;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub async fn transition_file_std_order(
    client: &mut Client,
    order_id: &str,
    payload: &FileStdTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<FileStdTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT o.status, o.payment_status, s.sku_type
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid
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
    let current_state: String = row.get(0);
    let current_payment_status: String = row.get(1);
    let sku_type: String = row.get(2);
    if sku_type != "FILE_STD" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "FILE_STD_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not FILE_STD"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_file_std_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "FILE_STD_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    if normalized_action == "lock_funds" {
        ensure_pre_payment_lock_checks(&tx, order_id, request_id).await?;
    } else if normalized_action == "start_delivery" {
        let prepared = ensure_order_deliverable_and_prepare_delivery(
            &tx, order_id, actor_role, request_id, trace_id,
        )
        .await?;
        let _ = prepared.delivery_id;
    }

    let updated_row = tx
        .query_one(
            "UPDATE trade.order_main
             SET status = $2,
                 payment_status = $3,
                 delivery_status = $4,
                 acceptance_status = $5,
                 settlement_status = $6,
                 dispute_status = $7,
                 last_reason_code = $8,
                 updated_at = now(),
                 closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE closed_at END
             WHERE order_id = $1::text::uuid
             RETURNING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &transition.target_state,
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
    let transitioned_at: String = updated_row.get(0);

    apply_authorization_cutoff_if_needed(
        &tx,
        order_id,
        transition.target_state,
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
        "trade.order.file_std.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(FileStdTransitionResponseData {
        order_id: order_id.to_string(),
        action: normalized_action,
        previous_state: current_state,
        current_state: transition.target_state.to_string(),
        payment_status: transition.target_payment_status.to_string(),
        delivery_status: transition.layered_status.delivery_status,
        acceptance_status: transition.layered_status.acceptance_status,
        settlement_status: transition.layered_status.settlement_status,
        dispute_status: transition.layered_status.dispute_status,
        reason_code: transition.reason_code.to_string(),
        transitioned_at,
    })
}

struct FileStdTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_file_std_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<FileStdTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "lock_funds"
            if matches!(
                current_state,
                "created" | "contract_pending" | "contract_effective"
            ) =>
        {
            ("buyer_locked", "paid".to_string(), "file_std_lock_funds")
        }
        "start_delivery" if current_state == "buyer_locked" => (
            "seller_delivering",
            payment_status.to_string(),
            "file_std_start_delivery",
        ),
        "mark_delivered" if current_state == "seller_delivering" => (
            "delivered",
            payment_status.to_string(),
            "file_std_mark_delivered",
        ),
        "accept_delivery" if current_state == "delivered" => (
            "accepted",
            payment_status.to_string(),
            "file_std_accept_delivery",
        ),
        "settle_order" if current_state == "accepted" => {
            ("settled", "paid".to_string(), "file_std_settle_order")
        }
        "close_completed" if current_state == "settled" => (
            "closed",
            payment_status.to_string(),
            "file_std_close_completed",
        ),
        "request_refund"
            if matches!(
                current_state,
                "buyer_locked" | "seller_delivering" | "delivered" | "accepted" | "settled"
            ) =>
        {
            ("closed", "refunded".to_string(), "file_std_refund_closed")
        }
        "open_dispute" if matches!(current_state, "delivered" | "accepted" | "settled") => (
            "dispute_opened",
            payment_status.to_string(),
            "file_std_dispute_opened",
        ),
        "resolve_dispute_refund" if current_state == "dispute_opened" => (
            "closed",
            "refunded".to_string(),
            "file_std_dispute_refund_closed",
        ),
        "resolve_dispute_complete" if current_state == "dispute_opened" => (
            "settled",
            "paid".to_string(),
            "file_std_dispute_resolved_settled",
        ),
        _ => return None,
    };

    let layered_status =
        derive_file_std_layered_status(target_state, &target_payment_status, reason_code);
    Some(FileStdTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_file_std_layered_status(
    target_state: &str,
    target_payment_status: &str,
    reason_code: &str,
) -> LayeredOrderStatus {
    if reason_code.contains("refund") {
        return LayeredOrderStatus {
            delivery_status: "refunded".to_string(),
            acceptance_status: "refunded".to_string(),
            settlement_status: "refunded".to_string(),
            dispute_status: "resolved".to_string(),
        };
    }
    if target_state == "dispute_opened" {
        return LayeredOrderStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "disputed".to_string(),
            settlement_status: "blocked".to_string(),
            dispute_status: "open".to_string(),
        };
    }
    derive_layered_status(target_state, target_payment_status)
}

#[cfg(test)]
mod tests {
    use super::derive_file_std_transition;

    #[test]
    fn delivered_can_open_dispute() {
        let transition = derive_file_std_transition("open_dispute", "delivered", "paid");
        assert!(transition.is_some());
        let t = transition.expect("transition");
        assert_eq!(t.target_state, "dispute_opened");
        assert_eq!(t.layered_status.dispute_status, "open");
    }

    #[test]
    fn dispute_can_resolve_refund() {
        let transition =
            derive_file_std_transition("resolve_dispute_refund", "dispute_opened", "paid");
        assert!(transition.is_some());
        let t = transition.expect("transition");
        assert_eq!(t.target_state, "closed");
        assert_eq!(t.target_payment_status, "refunded");
        assert_eq!(t.layered_status.settlement_status, "refunded");
    }
}
