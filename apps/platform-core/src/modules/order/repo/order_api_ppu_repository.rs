use crate::modules::delivery::repo::{
    apply_delivery_cutoff_if_needed, invalidate_delivery_cutoff_download_ticket_caches,
};
use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::dto::{ApiPpuTransitionRequest, ApiPpuTransitionResponseData};
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::ensure_order_deliverable_and_prepare_delivery;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};

pub async fn transition_api_ppu_order(
    client: &mut Client,
    order_id: &str,
    payload: &ApiPpuTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ApiPpuTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
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
    if sku_type != "API_PPU" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "API_PPU_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not API_PPU"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_api_ppu_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "API_PPU_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    if normalized_action == "authorize_access" {
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
                 closed_at = CASE WHEN $2 IN ('closed', 'disabled') THEN now() ELSE closed_at END
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
        "trade.order.api_ppu.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    invalidate_delivery_cutoff_download_ticket_caches(&delivery_cutoff).await;

    Ok(ApiPpuTransitionResponseData {
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

struct ApiPpuTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_api_ppu_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<ApiPpuTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "authorize_access" if current_state == "buyer_locked" => (
            "api_authorized",
            payment_status.to_string(),
            "api_ppu_authorized",
        ),
        "configure_quota" if current_state == "api_authorized" => (
            "quota_ready",
            payment_status.to_string(),
            "api_ppu_quota_configured",
        ),
        "record_failed_call" if matches!(current_state, "quota_ready" | "usage_active") => (
            "usage_active",
            payment_status.to_string(),
            "api_ppu_failed_call_not_billed",
        ),
        "settle_success_call" if matches!(current_state, "quota_ready" | "usage_active") => (
            "usage_active",
            "paid".to_string(),
            "api_ppu_success_call_billed",
        ),
        "expire_access"
            if matches!(
                current_state,
                "buyer_locked" | "api_authorized" | "quota_ready" | "usage_active"
            ) =>
        {
            ("expired", payment_status.to_string(), "api_ppu_expired")
        }
        "disable_access"
            if matches!(
                current_state,
                "buyer_locked" | "api_authorized" | "quota_ready" | "usage_active" | "expired"
            ) =>
        {
            (
                "disabled",
                payment_status.to_string(),
                "api_ppu_risk_frozen",
            )
        }
        _ => return None,
    };

    let layered_status = derive_api_ppu_layered_status(target_state, &target_payment_status);
    Some(ApiPpuTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_api_ppu_layered_status(
    target_state: &str,
    target_payment_status: &str,
) -> LayeredOrderStatus {
    match target_state {
        "api_authorized" | "quota_ready" => LayeredOrderStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "usage_active" => LayeredOrderStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "accepted".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "expired" => LayeredOrderStatus {
            delivery_status: "expired".to_string(),
            acceptance_status: "expired".to_string(),
            settlement_status: "expired".to_string(),
            dispute_status: "none".to_string(),
        },
        "disabled" | "closed" => LayeredOrderStatus {
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
    use super::derive_api_ppu_transition;

    #[test]
    fn api_ppu_success_call_is_billed() {
        let authorized = derive_api_ppu_transition("authorize_access", "buyer_locked", "paid");
        assert!(authorized.is_some());
        let quota = derive_api_ppu_transition("configure_quota", "api_authorized", "paid");
        assert!(quota.is_some());

        let failed_call = derive_api_ppu_transition("record_failed_call", "quota_ready", "paid")
            .expect("failed-call transition");
        assert_eq!(failed_call.target_state, "usage_active");
        assert_eq!(failed_call.target_payment_status, "paid");

        let success_call = derive_api_ppu_transition("settle_success_call", "usage_active", "paid")
            .expect("success-call transition");
        assert_eq!(success_call.target_payment_status, "paid");
    }

    #[test]
    fn disabled_cannot_reenter_settlement() {
        let denied = derive_api_ppu_transition("settle_success_call", "disabled", "paid");
        assert!(denied.is_none());
    }
}
