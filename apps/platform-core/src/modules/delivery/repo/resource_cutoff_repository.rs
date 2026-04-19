use super::download_ticket_repository::{delete_download_ticket_cache, redis_download_ticket_key};
use super::file_delivery_repository::write_delivery_audit_event;
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::ErrorResponse;
use serde_json::json;

#[derive(Debug, Default, Clone)]
pub struct DeliveryCutoffSideEffects {
    pub download_ticket_ids: Vec<String>,
}

pub async fn apply_delivery_cutoff_if_needed(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    current_state: &str,
    delivery_status: &str,
    dispute_status: &str,
    reason_code: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DeliveryCutoffSideEffects, (StatusCode, Json<ErrorResponse>)> {
    let Some(target_status) =
        derive_delivery_cutoff_target(current_state, delivery_status, dispute_status, reason_code)
    else {
        return Ok(DeliveryCutoffSideEffects::default());
    };

    let mut download_ticket_ids = client
        .query(
            "UPDATE delivery.delivery_ticket
             SET status = $2,
                 expire_at = CASE
                   WHEN $2 IN ('revoked', 'expired') THEN LEAST(expire_at, now())
                   ELSE expire_at
                 END
             WHERE order_id = $1::text::uuid
               AND (
                 ($2 = 'suspended' AND status = 'active')
                 OR ($2 IN ('revoked', 'expired') AND status IN ('active', 'suspended'))
               )
             RETURNING ticket_id::text",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<Vec<_>>();
    download_ticket_ids.sort();
    download_ticket_ids.dedup();

    let file_delivery_count = update_delivery_record_cutoff_count(
        client,
        order_id,
        target_status,
        &["file_download", "revision_push"],
    )
    .await?;
    if !download_ticket_ids.is_empty() || file_delivery_count > 0 {
        write_cutoff_audit(
            client,
            order_id,
            actor_role,
            &format!("delivery.file.auto_cutoff.{target_status}"),
            request_id,
            trace_id,
            json!({
                "resource": "file_delivery",
                "target_status": target_status,
                "download_ticket_count": download_ticket_ids.len(),
                "delivery_record_count": file_delivery_count,
                "download_ticket_ids": download_ticket_ids,
            }),
        )
        .await?;
    }

    let api_credential_count: i64 = client
        .query_one(
            "WITH affected AS (
               UPDATE delivery.api_credential
               SET status = $2,
                   valid_to = CASE
                     WHEN $2 IN ('revoked', 'expired') THEN LEAST(coalesce(valid_to, now()), now())
                     ELSE valid_to
                   END,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND (
                   ($2 = 'suspended' AND status = 'active')
                   OR ($2 IN ('revoked', 'expired') AND status IN ('active', 'suspended'))
                 )
               RETURNING api_credential_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    let api_delivery_count =
        update_delivery_record_cutoff_count(client, order_id, target_status, &["api_access"])
            .await?;
    if api_credential_count > 0 || api_delivery_count > 0 {
        write_cutoff_audit(
            client,
            order_id,
            actor_role,
            &format!("delivery.api.auto_cutoff.{target_status}"),
            request_id,
            trace_id,
            json!({
                "resource": "api_delivery",
                "target_status": target_status,
                "api_credential_count": api_credential_count,
                "delivery_record_count": api_delivery_count,
            }),
        )
        .await?;
    }

    let share_grant_count: i64 = client
        .query_one(
            "WITH affected AS (
               UPDATE delivery.data_share_grant
               SET grant_status = $2,
                   revoked_at = CASE
                     WHEN $2 = 'revoked' THEN coalesce(revoked_at, now())
                     ELSE revoked_at
                   END,
                   expires_at = CASE
                     WHEN $2 = 'expired' THEN LEAST(coalesce(expires_at, now()), now())
                     ELSE expires_at
                   END,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND (
                   ($2 = 'suspended' AND grant_status = 'active')
                   OR ($2 IN ('revoked', 'expired') AND grant_status IN ('active', 'suspended'))
                 )
               RETURNING data_share_grant_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    let share_delivery_count =
        update_delivery_record_cutoff_count(client, order_id, target_status, &["share_grant"])
            .await?;
    if share_grant_count > 0 || share_delivery_count > 0 {
        write_cutoff_audit(
            client,
            order_id,
            actor_role,
            &format!("delivery.share.auto_cutoff.{target_status}"),
            request_id,
            trace_id,
            json!({
                "resource": "share_delivery",
                "target_status": target_status,
                "data_share_grant_count": share_grant_count,
                "delivery_record_count": share_delivery_count,
            }),
        )
        .await?;
    }

    let sandbox_workspace_count: i64 = client
        .query_one(
            "WITH affected AS (
               UPDATE delivery.sandbox_workspace
               SET status = $2,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND (
                   ($2 = 'suspended' AND status IN ('active', 'provisioning'))
                   OR ($2 IN ('revoked', 'expired') AND status IN ('active', 'provisioning', 'suspended'))
                 )
               RETURNING sandbox_workspace_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    let sandbox_session_count: i64 = client
        .query_one(
            "WITH affected AS (
               UPDATE delivery.sandbox_session
               SET session_status = $2,
                   ended_at = CASE
                     WHEN $2 IN ('suspended', 'revoked', 'expired') THEN LEAST(coalesce(ended_at, now()), now())
                     ELSE ended_at
                   END
               WHERE sandbox_workspace_id IN (
                   SELECT sandbox_workspace_id
                   FROM delivery.sandbox_workspace
                   WHERE order_id = $1::text::uuid
               )
                 AND (
                   ($2 = 'suspended' AND session_status = 'active')
                   OR ($2 IN ('revoked', 'expired') AND session_status IN ('active', 'suspended'))
                 )
               RETURNING sandbox_session_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    let sandbox_delivery_count = update_delivery_record_cutoff_count(
        client,
        order_id,
        target_status,
        &["sandbox_workspace"],
    )
    .await?;
    if sandbox_workspace_count > 0 || sandbox_session_count > 0 || sandbox_delivery_count > 0 {
        write_cutoff_audit(
            client,
            order_id,
            actor_role,
            &format!("delivery.sandbox.auto_cutoff.{target_status}"),
            request_id,
            trace_id,
            json!({
                "resource": "sandbox_delivery",
                "target_status": target_status,
                "sandbox_workspace_count": sandbox_workspace_count,
                "sandbox_session_count": sandbox_session_count,
                "delivery_record_count": sandbox_delivery_count,
            }),
        )
        .await?;
    }

    Ok(DeliveryCutoffSideEffects {
        download_ticket_ids,
    })
}

pub async fn invalidate_delivery_cutoff_download_ticket_caches(
    side_effects: &DeliveryCutoffSideEffects,
) {
    for ticket_id in &side_effects.download_ticket_ids {
        delete_download_ticket_cache(&redis_download_ticket_key(ticket_id)).await;
    }
}

async fn update_delivery_record_cutoff_count(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    target_status: &str,
    delivery_types: &[&str],
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    let delivery_types = delivery_types
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    client
        .query_one(
            "WITH affected AS (
               UPDATE delivery.delivery_record
               SET status = $2,
                   expires_at = CASE
                     WHEN $2 IN ('revoked', 'expired') THEN LEAST(coalesce(expires_at, now()), now())
                     ELSE expires_at
                   END,
                   updated_at = now()
               WHERE order_id = $1::text::uuid
                 AND delivery_type = ANY($3::text[])
                 AND (
                   ($2 = 'suspended' AND status IN ('prepared', 'committed', 'active'))
                   OR ($2 IN ('revoked', 'expired') AND status IN ('prepared', 'committed', 'active', 'suspended'))
                 )
               RETURNING delivery_id
             )
             SELECT COUNT(*)::bigint FROM affected",
            &[&order_id, &target_status, &delivery_types],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.get(0))
}

async fn write_cutoff_audit(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    action_name: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    metadata: serde_json::Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    write_delivery_audit_event(
        client,
        "order",
        order_id,
        actor_role,
        action_name,
        "success",
        request_id,
        trace_id,
        metadata,
    )
    .await
}

fn derive_delivery_cutoff_target(
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
    use super::derive_delivery_cutoff_target;

    #[test]
    fn derives_delivery_cutoff_target_by_priority() {
        assert_eq!(
            derive_delivery_cutoff_target("disabled", "closed", "none", "api_ppu_risk_frozen"),
            Some("suspended")
        );
        assert_eq!(
            derive_delivery_cutoff_target("expired", "expired", "none", "share_ro_expired"),
            Some("expired")
        );
        assert_eq!(
            derive_delivery_cutoff_target("closed", "closed", "none", "order_cancel_before_lock"),
            Some("revoked")
        );
    }
}
