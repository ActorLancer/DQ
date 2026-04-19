use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::dto::{QryLiteTransitionRequest, QryLiteTransitionResponseData};
use crate::modules::order::repo::ensure_order_deliverable_and_prepare_delivery;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub async fn transition_qry_lite_order(
    client: &mut Client,
    order_id: &str,
    payload: &QryLiteTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<QryLiteTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
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
    if sku_type != "QRY_LITE" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "QRY_LITE_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not QRY_LITE"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_qry_lite_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "QRY_LITE_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    if normalized_action == "authorize_template" {
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

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        "trade.order.qry_lite.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(QryLiteTransitionResponseData {
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

struct QryLiteTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_qry_lite_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<QryLiteTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "authorize_template" if current_state == "buyer_locked" => (
            "template_authorized",
            payment_status.to_string(),
            "qry_lite_template_authorized",
        ),
        "validate_params" if current_state == "template_authorized" => (
            "params_validated",
            payment_status.to_string(),
            "qry_lite_params_validated",
        ),
        "execute_query" if current_state == "params_validated" => (
            "query_executed",
            "paid".to_string(),
            "qry_lite_query_executed",
        ),
        "make_result_available" if current_state == "query_executed" => (
            "result_available",
            payment_status.to_string(),
            "qry_lite_result_available",
        ),
        "close_acceptance" if current_state == "result_available" => (
            "closed",
            payment_status.to_string(),
            "qry_lite_acceptance_closed",
        ),
        _ => return None,
    };

    let layered_status = derive_qry_lite_layered_status(target_state, &target_payment_status);
    Some(QryLiteTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_qry_lite_layered_status(
    target_state: &str,
    target_payment_status: &str,
) -> LayeredOrderStatus {
    match target_state {
        "template_authorized" | "params_validated" => LayeredOrderStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "query_executed" | "result_available" => LayeredOrderStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "accepted".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "closed" => LayeredOrderStatus {
            delivery_status: "closed".to_string(),
            acceptance_status: "closed".to_string(),
            settlement_status: "closed".to_string(),
            dispute_status: "none".to_string(),
        },
        _ => LayeredOrderStatus {
            delivery_status: "pending_delivery".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: "not_started".to_string(),
            dispute_status: "none".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::derive_qry_lite_transition;

    #[test]
    fn qry_lite_lifecycle_transitions_to_closed() {
        let t1 = derive_qry_lite_transition("authorize_template", "buyer_locked", "paid");
        assert!(t1.is_some());
        let t2 = derive_qry_lite_transition("validate_params", "template_authorized", "paid");
        assert!(t2.is_some());
        let t3 = derive_qry_lite_transition("execute_query", "params_validated", "paid");
        assert!(t3.is_some());
        let t4 = derive_qry_lite_transition("make_result_available", "query_executed", "paid");
        assert!(t4.is_some());
        let t5 = derive_qry_lite_transition("close_acceptance", "result_available", "paid");
        assert!(t5.is_some());
        assert_eq!(t5.expect("close").target_state, "closed");
    }

    #[test]
    fn closed_cannot_execute_again() {
        let denied = derive_qry_lite_transition("execute_query", "closed", "paid");
        assert!(denied.is_none());
    }
}
