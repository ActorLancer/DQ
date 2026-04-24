use crate::modules::billing::repo::api_billing_repository::{
    ApiSubCycleChargeParams, record_api_sub_cycle_charge_in_tx,
};
use crate::modules::delivery::repo::{
    apply_delivery_cutoff_if_needed, invalidate_delivery_cutoff_download_ticket_caches,
};
use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::dto::{ApiSubTransitionRequest, ApiSubTransitionResponseData};
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::auto_create_delivery_task_if_needed;
use crate::modules::order::repo::ensure_order_deliverable_and_prepare_delivery;
use crate::modules::order::repo::ensure_pre_payment_lock_checks;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub async fn transition_api_sub_order(
    client: &mut Client,
    order_id: &str,
    payload: &ApiSubTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ApiSubTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
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
    if sku_type != "API_SUB" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "API_SUB_TRANSITION_FORBIDDEN".to_string(),
                message: format!(
                    "API_SUB_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not API_SUB"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_api_sub_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "API_SUB_TRANSITION_FORBIDDEN".to_string(),
                message: format!(
                    "API_SUB_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    if normalized_action == "lock_funds" {
        ensure_pre_payment_lock_checks(&tx, order_id, request_id).await?;
    } else if normalized_action == "bind_application" {
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
    let (billing_event_id, billing_event_type, billing_event_replayed) =
        if normalized_action == "bill_cycle" {
            let (event, replayed) = record_api_sub_cycle_charge_in_tx(
                &tx,
                order_id,
                actor_role,
                request_id,
                trace_id,
                &ApiSubCycleChargeParams {
                    billing_cycle_code: payload.billing_cycle_code.clone().unwrap_or_default(),
                    billing_amount: payload.billing_amount.clone(),
                    reason_note: payload.reason_note.clone(),
                },
            )
            .await?;
            (
                Some(event.billing_event_id),
                Some(event.event_type),
                replayed,
            )
        } else {
            (None, None, false)
        };
    if normalized_action == "lock_funds" {
        let _ = auto_create_delivery_task_if_needed(
            &tx,
            order_id,
            actor_role,
            request_id,
            trace_id,
            "api_sub_lock_funds",
        )
        .await?;
    }

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
    let delivery_cutoff = apply_delivery_cutoff_if_needed(
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
        "trade.order.api_sub.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    invalidate_delivery_cutoff_download_ticket_caches(&delivery_cutoff).await;

    Ok(ApiSubTransitionResponseData {
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
        billing_event_id,
        billing_event_type,
        billing_event_replayed,
        transitioned_at,
    })
}

struct ApiSubTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_api_sub_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<ApiSubTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "lock_funds"
            if matches!(
                current_state,
                "created" | "contract_pending" | "contract_effective"
            ) =>
        {
            ("buyer_locked", "paid".to_string(), "api_sub_lock_funds")
        }
        "bind_application" if current_state == "buyer_locked" => (
            "api_bound",
            payment_status.to_string(),
            "api_sub_bind_application",
        ),
        "issue_api_key" if current_state == "api_bound" => (
            "api_key_issued",
            payment_status.to_string(),
            "api_sub_issue_api_key",
        ),
        "trial_call" if current_state == "api_key_issued" => (
            "api_trial_active",
            payment_status.to_string(),
            "api_sub_trial_call",
        ),
        "activate_subscription" if current_state == "api_trial_active" => {
            ("active", payment_status.to_string(), "api_sub_activated")
        }
        "bill_cycle" if current_state == "active" => {
            ("active", "paid".to_string(), "api_sub_cycle_billed")
        }
        "terminate_subscription"
            if matches!(
                current_state,
                "buyer_locked" | "api_bound" | "api_key_issued" | "api_trial_active" | "active"
            ) =>
        {
            ("closed", payment_status.to_string(), "api_sub_terminated")
        }
        _ => return None,
    };

    let layered_status = derive_api_sub_layered_status(target_state, &target_payment_status);
    Some(ApiSubTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_api_sub_layered_status(
    target_state: &str,
    target_payment_status: &str,
) -> LayeredOrderStatus {
    match target_state {
        "buyer_locked" => LayeredOrderStatus {
            delivery_status: "pending_delivery".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "api_bound" | "api_key_issued" | "api_trial_active" => LayeredOrderStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "active" => LayeredOrderStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "accepted".to_string(),
            settlement_status: "pending_settlement".to_string(),
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
    use super::derive_api_sub_transition;

    #[test]
    fn lifecycle_to_active_is_allowed() {
        let t1 = derive_api_sub_transition("bind_application", "buyer_locked", "paid");
        assert!(t1.is_some());
        let t2 = derive_api_sub_transition("issue_api_key", "api_bound", "paid");
        assert!(t2.is_some());
        let t3 = derive_api_sub_transition("trial_call", "api_key_issued", "paid");
        assert!(t3.is_some());
        let t4 = derive_api_sub_transition("activate_subscription", "api_trial_active", "paid");
        assert!(t4.is_some());
        let active = t4.expect("transition");
        assert_eq!(active.target_state, "active");
        assert_eq!(active.layered_status.acceptance_status, "accepted");
    }

    #[test]
    fn closed_cannot_reenter_billing_cycle() {
        let denied = derive_api_sub_transition("bill_cycle", "closed", "paid");
        assert!(denied.is_none());
    }
}
