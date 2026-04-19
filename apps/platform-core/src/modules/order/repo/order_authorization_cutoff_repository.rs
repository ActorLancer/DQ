use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::ErrorResponse;

pub async fn apply_authorization_cutoff_if_needed(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    current_state: &str,
    delivery_status: &str,
    dispute_status: &str,
    reason_code: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(target_status) =
        derive_cutoff_target(current_state, delivery_status, dispute_status, reason_code)
    else {
        return Ok(());
    };

    let updated_count: i64 = client
        .query_one(
            "WITH affected AS (
               UPDATE trade.authorization_grant
               SET status = $2,
                   valid_to = CASE
                     WHEN $2 IN ('revoked', 'expired') THEN coalesce(valid_to, now())
                     ELSE valid_to
                   END,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND (
                   ($2 = 'suspended' AND status = 'active')
                   OR ($2 IN ('revoked', 'expired') AND status IN ('active', 'suspended'))
                 )
               RETURNING authorization_grant_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    if updated_count == 0 {
        return Ok(());
    }

    let action_name = format!("trade.authorization.auto_cutoff.{target_status}");
    write_trade_audit_event(
        client,
        "authorization",
        order_id,
        actor_role,
        &action_name,
        "success",
        request_id,
        trace_id,
    )
    .await?;

    Ok(())
}

fn derive_cutoff_target(
    current_state: &str,
    delivery_status: &str,
    dispute_status: &str,
    reason_code: &str,
) -> Option<&'static str> {
    let reason = reason_code.to_ascii_lowercase();
    if reason.contains("risk")
        || matches!(dispute_status, "open" | "opened" | "disputed")
        || current_state == "dispute_opened"
        || reason.contains("dispute")
    {
        return Some("suspended");
    }

    if current_state == "expired" || delivery_status == "expired" || reason.contains("expired") {
        return Some("expired");
    }

    if matches!(current_state, "closed" | "revoked" | "disabled")
        || delivery_status == "closed"
        || reason.contains("cancel")
        || reason.contains("revoke")
    {
        return Some("revoked");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::derive_cutoff_target;

    #[test]
    fn derives_cutoff_target_by_priority() {
        assert_eq!(
            derive_cutoff_target("disabled", "closed", "none", "api_ppu_risk_frozen"),
            Some("suspended")
        );
        assert_eq!(
            derive_cutoff_target("expired", "expired", "none", "share_ro_expired"),
            Some("expired")
        );
        assert_eq!(
            derive_cutoff_target("closed", "closed", "none", "order_cancel_before_lock"),
            Some("revoked")
        );
    }
}
