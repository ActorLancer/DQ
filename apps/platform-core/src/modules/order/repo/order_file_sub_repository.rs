use crate::modules::order::domain::{LayeredOrderStatus, derive_layered_status};
use crate::modules::order::dto::{FileSubTransitionRequest, FileSubTransitionResponseData};
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::ensure_pre_payment_lock_checks;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::Client;

pub async fn transition_file_sub_order(
    client: &mut Client,
    order_id: &str,
    payload: &FileSubTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<FileSubTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
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
    if sku_type != "FILE_SUB" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "FILE_SUB_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not FILE_SUB"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_file_sub_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "FILE_SUB_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    if matches!(
        normalized_action.as_str(),
        "establish_subscription" | "renew_subscription"
    ) {
        ensure_pre_payment_lock_checks(&tx, order_id, request_id).await?;
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
        "trade.order.file_sub.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(FileSubTransitionResponseData {
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

struct FileSubTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_file_sub_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<FileSubTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "establish_subscription"
            if matches!(
                current_state,
                "created" | "contract_pending" | "contract_effective"
            ) =>
        {
            ("buyer_locked", "paid".to_string(), "file_sub_establish")
        }
        "start_cycle_delivery" if matches!(current_state, "buyer_locked" | "accepted") => (
            "seller_delivering",
            payment_status.to_string(),
            "file_sub_cycle_delivery_started",
        ),
        "mark_cycle_delivered" if current_state == "seller_delivering" => (
            "delivered",
            payment_status.to_string(),
            "file_sub_cycle_delivered",
        ),
        "accept_cycle_delivery" if current_state == "delivered" => (
            "accepted",
            payment_status.to_string(),
            "file_sub_cycle_accepted",
        ),
        "pause_subscription" if matches!(current_state, "buyer_locked" | "accepted") => {
            ("paused", payment_status.to_string(), "file_sub_paused")
        }
        "expire_subscription"
            if matches!(current_state, "buyer_locked" | "accepted" | "paused") =>
        {
            ("expired", payment_status.to_string(), "file_sub_expired")
        }
        "renew_subscription" if matches!(current_state, "paused" | "expired") => {
            ("buyer_locked", "paid".to_string(), "file_sub_renewed")
        }
        "request_refund"
            if matches!(
                current_state,
                "buyer_locked" | "seller_delivering" | "delivered" | "accepted" | "paused"
            ) =>
        {
            ("closed", "refunded".to_string(), "file_sub_refund_closed")
        }
        "open_dispute" if matches!(current_state, "delivered" | "accepted" | "paused") => (
            "dispute_opened",
            payment_status.to_string(),
            "file_sub_dispute_opened",
        ),
        "resolve_dispute_refund" if current_state == "dispute_opened" => (
            "closed",
            "refunded".to_string(),
            "file_sub_dispute_refund_closed",
        ),
        "resolve_dispute_complete" if current_state == "dispute_opened" => (
            "accepted",
            "paid".to_string(),
            "file_sub_dispute_resolved_continue",
        ),
        _ => return None,
    };

    let layered_status =
        derive_file_sub_layered_status(target_state, &target_payment_status, reason_code);
    Some(FileSubTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_file_sub_layered_status(
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
    if target_state == "paused" {
        return LayeredOrderStatus {
            delivery_status: "paused".to_string(),
            acceptance_status: "paused".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        };
    }
    if target_state == "expired" {
        return LayeredOrderStatus {
            delivery_status: "expired".to_string(),
            acceptance_status: "expired".to_string(),
            settlement_status: "expired".to_string(),
            dispute_status: "none".to_string(),
        };
    }
    derive_layered_status(target_state, target_payment_status)
}

#[cfg(test)]
mod tests {
    use super::derive_file_sub_transition;

    #[test]
    fn can_pause_and_renew_subscription() {
        let paused = derive_file_sub_transition("pause_subscription", "accepted", "paid");
        assert!(paused.is_some());
        let pause = paused.expect("pause transition");
        assert_eq!(pause.target_state, "paused");
        assert_eq!(pause.layered_status.delivery_status, "paused");

        let renewed = derive_file_sub_transition("renew_subscription", "paused", "paid");
        assert!(renewed.is_some());
        let renew = renewed.expect("renew transition");
        assert_eq!(renew.target_state, "buyer_locked");
        assert_eq!(renew.target_payment_status, "paid");
    }

    #[test]
    fn dispute_refund_closes_subscription() {
        let transition =
            derive_file_sub_transition("resolve_dispute_refund", "dispute_opened", "paid");
        assert!(transition.is_some());
        let t = transition.expect("transition");
        assert_eq!(t.target_state, "closed");
        assert_eq!(t.target_payment_status, "refunded");
        assert_eq!(t.layered_status.dispute_status, "resolved");
    }
}
