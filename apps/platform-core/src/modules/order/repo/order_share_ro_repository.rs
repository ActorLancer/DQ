use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::dto::{ShareRoTransitionRequest, ShareRoTransitionResponseData};
use crate::modules::order::repo::apply_authorization_cutoff_if_needed;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::Client;

pub async fn transition_share_ro_order(
    client: &mut Client,
    order_id: &str,
    payload: &ShareRoTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ShareRoTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
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
    if sku_type != "SHARE_RO" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "SHARE_RO_TRANSITION_FORBIDDEN: order sku_type `{sku_type}` is not SHARE_RO"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let normalized_action = payload.action.trim().to_lowercase();
    let Some(transition) =
        derive_share_ro_transition(&normalized_action, &current_state, &current_payment_status)
    else {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "SHARE_RO_TRANSITION_FORBIDDEN: action `{}` cannot apply on current_state `{current_state}`",
                    payload.action
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };

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
                 closed_at = CASE WHEN $2 IN ('revoked', 'expired') THEN now() ELSE closed_at END
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
        "trade.order.share_ro.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ShareRoTransitionResponseData {
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

struct ShareRoTransition {
    target_state: &'static str,
    target_payment_status: String,
    layered_status: LayeredOrderStatus,
    reason_code: &'static str,
}

fn derive_share_ro_transition(
    action: &str,
    current_state: &str,
    payment_status: &str,
) -> Option<ShareRoTransition> {
    let (target_state, target_payment_status, reason_code) = match action {
        "enable_share"
            if matches!(
                current_state,
                "created" | "contract_pending" | "contract_effective"
            ) =>
        {
            (
                "share_enabled",
                payment_status.to_string(),
                "share_ro_enabled",
            )
        }
        "grant_read_access" if current_state == "share_enabled" => (
            "share_granted",
            payment_status.to_string(),
            "share_ro_access_granted",
        ),
        "confirm_first_query" if current_state == "share_granted" => (
            "shared_active",
            payment_status.to_string(),
            "share_ro_first_query_confirmed",
        ),
        "revoke_share"
            if matches!(
                current_state,
                "share_enabled" | "share_granted" | "shared_active" | "expired"
            ) =>
        {
            ("revoked", payment_status.to_string(), "share_ro_revoked")
        }
        "expire_share"
            if matches!(
                current_state,
                "share_enabled" | "share_granted" | "shared_active"
            ) =>
        {
            ("expired", payment_status.to_string(), "share_ro_expired")
        }
        "interrupt_dispute"
            if matches!(
                current_state,
                "share_enabled" | "share_granted" | "shared_active"
            ) =>
        {
            (
                "dispute_interrupted",
                payment_status.to_string(),
                "share_ro_dispute_interrupted",
            )
        }
        _ => return None,
    };

    let layered_status = derive_share_ro_layered_status(target_state, &target_payment_status);
    Some(ShareRoTransition {
        target_state,
        target_payment_status,
        layered_status,
        reason_code,
    })
}

fn derive_share_ro_layered_status(
    target_state: &str,
    target_payment_status: &str,
) -> LayeredOrderStatus {
    match target_state {
        "share_enabled" | "share_granted" => LayeredOrderStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "shared_active" => LayeredOrderStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "accepted".to_string(),
            settlement_status: if target_payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "revoked" => LayeredOrderStatus {
            delivery_status: "closed".to_string(),
            acceptance_status: "closed".to_string(),
            settlement_status: "closed".to_string(),
            dispute_status: "none".to_string(),
        },
        "expired" => LayeredOrderStatus {
            delivery_status: "expired".to_string(),
            acceptance_status: "expired".to_string(),
            settlement_status: "expired".to_string(),
            dispute_status: "none".to_string(),
        },
        "dispute_interrupted" => LayeredOrderStatus {
            delivery_status: "blocked".to_string(),
            acceptance_status: "blocked".to_string(),
            settlement_status: "frozen".to_string(),
            dispute_status: "opened".to_string(),
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
    use super::derive_share_ro_transition;

    #[test]
    fn share_ro_lifecycle_allows_enable_to_revoke() {
        let t1 = derive_share_ro_transition("enable_share", "created", "unpaid");
        assert!(t1.is_some());
        let t2 = derive_share_ro_transition("grant_read_access", "share_enabled", "unpaid");
        assert!(t2.is_some());
        let t3 = derive_share_ro_transition("confirm_first_query", "share_granted", "paid");
        assert!(t3.is_some());
        let t4 = derive_share_ro_transition("revoke_share", "shared_active", "paid");
        assert!(t4.is_some());
        assert_eq!(t4.expect("revoke").target_state, "revoked");
    }

    #[test]
    fn revoked_cannot_grant_again() {
        let denied = derive_share_ro_transition("grant_read_access", "revoked", "paid");
        assert!(denied.is_none());
    }
}
