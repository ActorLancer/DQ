use super::outbox_repository::{
    build_delivery_receipt_outbox_payload, write_billing_trigger_bridge_event,
    write_delivery_receipt_outbox_event,
};
use crate::modules::billing::repo::share_ro_billing_repository::record_share_ro_revoke_refund_placeholder_in_tx;
use crate::modules::delivery::domain::is_accepted_state;
use crate::modules::delivery::dto::{
    ManageShareGrantRequest, ShareGrantListResponseData, ShareGrantResponseData,
};
use crate::modules::integration::application::{
    DeliveryCompletionNotificationDispatchInput, queue_delivery_completion_notifications,
};
use crate::modules::order::repo::{
    apply_authorization_cutoff_if_needed, ensure_order_deliverable_and_prepare_delivery,
    map_db_error, write_trade_audit_event,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Map, Value, json};

const DELIVERY_SHARE_MANAGE_EVENT: &str = "delivery.share.enable";
const DELIVERY_SHARE_READ_EVENT: &str = "delivery.share.read";

pub async fn manage_share_grant(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageShareGrantRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<ShareGrantResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let mut context = load_share_context(&tx, order_id, request_id).await?;

    enforce_subject_status(&context, request_id)?;
    enforce_manage_scope(actor_role, tenant_id, &context.seller_org_id, request_id)?;
    enforce_share_ro(&context, request_id)?;

    let normalized_operation = normalize_operation(payload.operation.as_deref(), request_id)?;
    if normalized_operation == "grant" {
        let prepared = ensure_order_deliverable_and_prepare_delivery(
            &tx, order_id, actor_role, request_id, trace_id,
        )
        .await?;
        context.prepared_delivery_id = Some(prepared.delivery_id);
        let share_object = resolve_share_object(
            &tx,
            &context.asset_version_id,
            payload.asset_object_id.as_deref(),
            payload.share_protocol.as_deref(),
            request_id,
        )
        .await?;
        let share_protocol = normalize_share_protocol(
            payload.share_protocol.as_deref(),
            share_object.share_protocol.as_deref(),
            context.default_share_protocol.as_deref(),
            request_id,
        )?;
        let recipient_ref = require_non_empty(
            payload.recipient_ref.as_deref(),
            "SHARE_GRANT_FORBIDDEN: recipient_ref is required",
            request_id,
        )?;
        let access_locator = payload
            .access_locator
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| share_object.object_locator.clone())
            .ok_or_else(|| {
                conflict(
                    "SHARE_GRANT_FORBIDDEN: access_locator is required when share object has no locator",
                    request_id,
                )
            })?;
        let expires_at = require_non_empty(
            payload.expires_at.as_deref(),
            "SHARE_GRANT_FORBIDDEN: expires_at is required",
            request_id,
        )?;
        let receipt_hash = require_non_empty(
            payload.receipt_hash.as_deref(),
            "SHARE_GRANT_FORBIDDEN: receipt_hash is required",
            request_id,
        )?;
        let subscriber_ref = payload
            .subscriber_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        validate_share_expiry(&expires_at, request_id)?;
        let metadata = merged_share_metadata(
            None,
            payload.metadata.as_ref(),
            payload.scope_json.as_ref(),
            subscriber_ref.as_deref(),
            actor_role,
            if context.active_grant_id.is_some() {
                "updated"
            } else if matches!(context.current_state.as_str(), "revoked" | "expired") {
                "regranted"
            } else {
                "granted"
            },
        );

        let row = if let Some(active_grant_id) = context.active_grant_id.as_deref() {
            tx.query_one(
                "UPDATE delivery.data_share_grant
                 SET asset_object_id = $2::text::uuid,
                     recipient_ref = $3,
                     share_protocol = $4,
                     access_locator = $5,
                     grant_status = 'active',
                     read_only = true,
                     receipt_hash = $6,
                     granted_at = COALESCE(granted_at, now()),
                     revoked_at = NULL,
                     expires_at = $7::timestamptz,
                     metadata = $8::jsonb,
                     updated_at = now()
                 WHERE data_share_grant_id = $1::text::uuid
                 RETURNING data_share_grant_id::text,
                           order_id::text,
                           asset_object_id::text,
                           recipient_ref,
                           share_protocol,
                           access_locator,
                           grant_status,
                           read_only,
                           receipt_hash,
                           to_char(granted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           metadata,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &active_grant_id,
                    &share_object.asset_object_id,
                    &recipient_ref,
                    &share_protocol,
                    &access_locator,
                    &receipt_hash,
                    &expires_at,
                    &metadata,
                ],
            )
            .await
            .map_err(map_db_error)?
        } else {
            tx.query_one(
                "INSERT INTO delivery.data_share_grant (
                   order_id,
                   asset_object_id,
                   recipient_ref,
                   share_protocol,
                   access_locator,
                   grant_status,
                   read_only,
                   receipt_hash,
                   granted_at,
                   expires_at,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3,
                   $4,
                   $5,
                   'active',
                   true,
                   $6,
                   now(),
                   $7::timestamptz,
                   $8::jsonb
                 )
                 RETURNING data_share_grant_id::text,
                           order_id::text,
                           asset_object_id::text,
                           recipient_ref,
                           share_protocol,
                           access_locator,
                           grant_status,
                           read_only,
                           receipt_hash,
                           to_char(granted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           metadata,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &order_id,
                    &share_object.asset_object_id,
                    &recipient_ref,
                    &share_protocol,
                    &access_locator,
                    &receipt_hash,
                    &expires_at,
                    &metadata,
                ],
            )
            .await
            .map_err(map_db_error)?
        };

        let operation = if context.active_grant_id.is_some() {
            "updated"
        } else if matches!(context.current_state.as_str(), "revoked" | "expired") {
            "regranted"
        } else {
            "granted"
        };
        let target_state = if context.current_state == "shared_active" {
            "shared_active"
        } else {
            "share_granted"
        };
        let layered_status =
            derive_share_order_layered_status(target_state, &context.payment_status);
        update_share_delivery_record(
            &tx,
            order_id,
            context.prepared_delivery_id.as_deref(),
            context.committed_delivery_id.as_deref(),
            &share_protocol,
            &receipt_hash,
            &expires_at,
            false,
        )
        .await?;
        update_share_order_state(
            &tx,
            order_id,
            target_state,
            &context.payment_status,
            &layered_status,
            if operation == "updated" {
                "share_ro_access_grant_updated"
            } else if operation == "regranted" {
                "share_ro_access_regranted"
            } else {
                "share_ro_access_granted"
            },
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
        write_delivery_share_audit_event(
            &tx,
            order_id,
            actor_role,
            DELIVERY_SHARE_MANAGE_EVENT,
            operation,
            request_id,
            trace_id,
            json!({
                "order_id": order_id,
                "seller_org_id": context.seller_org_id,
                "buyer_org_id": context.buyer_org_id,
                "asset_object_id": share_object.asset_object_id,
                "share_protocol": share_protocol,
                "recipient_ref": recipient_ref,
                "subscriber_ref": subscriber_ref,
                "access_locator": access_locator,
                "current_state": target_state,
            }),
        )
        .await?;
        let delivery_id = context
            .committed_delivery_id
            .clone()
            .or_else(|| context.prepared_delivery_id.clone())
            .expect("share delivery id must exist");
        let delivery_commit_hash = format!("share-grant:{share_protocol}:{receipt_hash}");
        write_delivery_receipt_outbox_event(
            &tx,
            &delivery_id,
            &build_delivery_receipt_outbox_payload(
                "share",
                order_id,
                &delivery_id,
                &context.sku_type,
                actor_role,
                &context.buyer_org_id,
                &context.seller_org_id,
                target_state,
                &context.payment_status,
                &layered_status.delivery_status,
                &layered_status.acceptance_status,
                &layered_status.settlement_status,
                &layered_status.dispute_status,
                Some(receipt_hash.as_str()),
                Some(delivery_commit_hash.as_str()),
                Some("share_grant"),
                Some(share_protocol.as_str()),
                None,
                json!({
                    "data_share_grant_id": row.get::<_, String>(0),
                    "asset_object_id": share_object.asset_object_id,
                    "recipient_ref": recipient_ref,
                    "subscriber_ref": subscriber_ref,
                    "access_locator": access_locator,
                    "grant_status": row.get::<_, String>(6),
                    "expires_at": expires_at,
                    "operation": operation,
                }),
            ),
            request_id,
            trace_id,
            idempotency_key,
        )
        .await?;
        let _ = queue_delivery_completion_notifications(
            &tx,
            DeliveryCompletionNotificationDispatchInput {
                order_id,
                delivery_branch: "share",
                result_ref_type: "delivery_record",
                result_ref_id: &delivery_id,
                source_event_aggregate_type: "delivery.delivery_record",
                source_event_event_type: "delivery.committed",
                source_event_occurred_at: None,
                delivery_type: Some("share_grant"),
                delivery_route: Some(share_protocol.as_str()),
                receipt_hash: Some(receipt_hash.as_str()),
                delivery_commit_hash: Some(delivery_commit_hash.as_str()),
                request_id,
                trace_id,
            },
        )
        .await
        .map_err(map_db_error)?;
        let billing_bridge_idempotency_key = format!("billing-trigger:share-grant:{delivery_id}");
        write_billing_trigger_bridge_event(
            &tx,
            order_id,
            "delivery_committed",
            "delivery_record",
            &delivery_id,
            DELIVERY_SHARE_MANAGE_EVENT,
            actor_role,
            request_id,
            trace_id,
            billing_bridge_idempotency_key.as_str(),
            json!({
                "delivery_branch": "share",
                "delivery_id": delivery_id,
                "share_protocol": share_protocol,
                "operation": operation,
                "grant_status": row.get::<_, String>(6),
                "data_share_grant_id": row.get::<_, String>(0),
            }),
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(map_share_grant_response(
            &row,
            &context.sku_id,
            &context.sku_type,
            target_state,
            &context.payment_status,
            &layered_status.delivery_status,
            Some(operation),
        ));
    }

    let active_grant_id = context.active_grant_id.clone().ok_or_else(|| {
        not_found(
            &format!("active share grant not found for order: {order_id}"),
            request_id,
        )
    })?;
    let latest_committed_delivery_id = context.committed_delivery_id.clone();
    let receipt_hash = payload
        .receipt_hash
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or(context.active_receipt_hash.clone())
        .ok_or_else(|| {
            conflict(
                "SHARE_GRANT_FORBIDDEN: receipt_hash is required for revoke",
                request_id,
            )
        })?;
    let metadata = merged_share_metadata(
        context.active_metadata.as_ref(),
        payload.metadata.as_ref(),
        payload.scope_json.as_ref(),
        payload.subscriber_ref.as_deref(),
        actor_role,
        "revoked",
    );
    let row = tx
        .query_one(
            "UPDATE delivery.data_share_grant
             SET grant_status = 'revoked',
                 revoked_at = now(),
                 receipt_hash = $2,
                 metadata = $3::jsonb,
                 updated_at = now()
             WHERE data_share_grant_id = $1::text::uuid
             RETURNING data_share_grant_id::text,
                       order_id::text,
                       asset_object_id::text,
                       recipient_ref,
                       share_protocol,
                       access_locator,
                       grant_status,
                       read_only,
                       receipt_hash,
                       to_char(granted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       metadata,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&active_grant_id, &receipt_hash, &metadata],
        )
        .await
        .map_err(map_db_error)?;

    let layered_status = derive_share_order_layered_status("revoked", &context.payment_status);
    update_share_delivery_record(
        &tx,
        order_id,
        context.prepared_delivery_id.as_deref(),
        latest_committed_delivery_id.as_deref(),
        context
            .active_share_protocol
            .as_deref()
            .unwrap_or("share_link"),
        &receipt_hash,
        payload
            .expires_at
            .as_deref()
            .unwrap_or("1970-01-01T00:00:00Z"),
        true,
    )
    .await?;
    update_share_order_state(
        &tx,
        order_id,
        "revoked",
        &context.payment_status,
        &layered_status,
        "share_ro_revoked",
    )
    .await?;
    apply_authorization_cutoff_if_needed(
        &tx,
        order_id,
        "revoked",
        &layered_status.delivery_status,
        &layered_status.dispute_status,
        "share_ro_revoked",
        actor_role,
        request_id,
        trace_id,
    )
    .await?;
    let _ = record_share_ro_revoke_refund_placeholder_in_tx(
        &tx,
        order_id,
        actor_role,
        request_id,
        trace_id,
        "share_ro_revoked_placeholder_refund",
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
    write_delivery_share_audit_event(
        &tx,
        order_id,
        actor_role,
        DELIVERY_SHARE_MANAGE_EVENT,
        "revoked",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "seller_org_id": context.seller_org_id,
            "buyer_org_id": context.buyer_org_id,
            "data_share_grant_id": active_grant_id,
            "current_state": "revoked",
        }),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(map_share_grant_response(
        &row,
        &context.sku_id,
        &context.sku_type,
        "revoked",
        &context.payment_status,
        &layered_status.delivery_status,
        Some("revoked"),
    ))
}

pub async fn get_share_grants(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ShareGrantListResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_share_context(&tx, order_id, request_id).await?;

    enforce_subject_status(&context, request_id)?;
    enforce_read_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_share_ro(&context, request_id)?;

    let rows = tx
        .query(
            "SELECT data_share_grant_id::text,
                    order_id::text,
                    asset_object_id::text,
                    recipient_ref,
                    share_protocol,
                    access_locator,
                    grant_status,
                    read_only,
                    receipt_hash,
                    to_char(granted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    metadata,
                    to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.data_share_grant
             WHERE order_id = $1::text::uuid
             ORDER BY created_at DESC, data_share_grant_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    if rows.is_empty() {
        return Err(not_found(
            &format!("share grant not found for order: {order_id}"),
            request_id,
        ));
    }

    write_delivery_share_audit_event(
        &tx,
        order_id,
        actor_role,
        DELIVERY_SHARE_READ_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "buyer_org_id": context.buyer_org_id,
            "seller_org_id": context.seller_org_id,
            "grant_count": rows.len(),
        }),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    let sku_id = context.sku_id.clone();
    let sku_type = context.sku_type.clone();
    let current_state = context.current_state.clone();
    let payment_status = context.payment_status.clone();
    let delivery_status = context.delivery_status.clone();

    Ok(ShareGrantListResponseData {
        order_id: order_id.to_string(),
        sku_id: sku_id.clone(),
        sku_type: sku_type.clone(),
        current_state: current_state.clone(),
        payment_status: payment_status.clone(),
        grants: rows
            .iter()
            .map(|row| {
                map_share_grant_response(
                    row,
                    &sku_id,
                    &sku_type,
                    &current_state,
                    &payment_status,
                    &delivery_status,
                    None,
                )
            })
            .collect(),
    })
}

#[derive(Debug)]
struct ShareContext {
    buyer_org_id: String,
    seller_org_id: String,
    asset_version_id: String,
    sku_id: String,
    sku_type: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
    buyer_status: String,
    buyer_metadata: Value,
    seller_status: String,
    seller_metadata: Value,
    product_status: String,
    product_metadata: Value,
    asset_version_status: String,
    sku_status: String,
    default_share_protocol: Option<String>,
    prepared_delivery_id: Option<String>,
    committed_delivery_id: Option<String>,
    active_grant_id: Option<String>,
    active_receipt_hash: Option<String>,
    active_share_protocol: Option<String>,
    active_metadata: Option<Value>,
}

#[derive(Debug)]
struct ShareObjectBinding {
    asset_object_id: String,
    object_locator: Option<String>,
    share_protocol: Option<String>,
}

async fn load_share_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<ShareContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.buyer_org_id::text,
                    o.seller_org_id::text,
                    o.asset_version_id::text,
                    o.sku_id::text,
                    s.sku_type,
                    o.status,
                    o.payment_status,
                    o.delivery_status,
                    buyer.status,
                    buyer.metadata,
                    seller.status,
                    seller.metadata,
                    p.status,
                    p.metadata,
                    v.status,
                    s.status,
                    COALESCE(NULLIF(s.share_protocol, ''), NULLIF(p.metadata ->> 'share_protocol', ''), NULLIF(o.price_snapshot_json ->> 'share_protocol', '')),
                    prepared.delivery_id::text,
                    committed.delivery_id::text,
                    active.data_share_grant_id::text,
                    active.receipt_hash,
                    active.share_protocol,
                    active.metadata
             FROM trade.order_main o
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.asset_version v ON v.asset_version_id = o.asset_version_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'prepared'
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) prepared ON true
             LEFT JOIN LATERAL (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'committed'
               ORDER BY committed_at DESC NULLS LAST, created_at DESC, delivery_id DESC
               LIMIT 1
             ) committed ON true
             LEFT JOIN LATERAL (
               SELECT data_share_grant_id, receipt_hash, share_protocol, metadata
               FROM delivery.data_share_grant
               WHERE order_id = o.order_id
                 AND grant_status = 'active'
               ORDER BY granted_at DESC NULLS LAST, created_at DESC, data_share_grant_id DESC
               LIMIT 1
             ) active ON true
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };

    Ok(ShareContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        asset_version_id: row.get(2),
        sku_id: row.get(3),
        sku_type: row.get(4),
        current_state: row.get(5),
        payment_status: row.get(6),
        delivery_status: row.get(7),
        buyer_status: row.get(8),
        buyer_metadata: row.get(9),
        seller_status: row.get(10),
        seller_metadata: row.get(11),
        product_status: row.get(12),
        product_metadata: row.get(13),
        asset_version_status: row.get(14),
        sku_status: row.get(15),
        default_share_protocol: row.get(16),
        prepared_delivery_id: row.get(17),
        committed_delivery_id: row.get(18),
        active_grant_id: row.get(19),
        active_receipt_hash: row.get(20),
        active_share_protocol: row.get(21),
        active_metadata: row.get(22),
    })
}

async fn resolve_share_object(
    client: &(impl GenericClient + Sync),
    asset_version_id: &str,
    asset_object_id: Option<&str>,
    requested_protocol: Option<&str>,
    request_id: Option<&str>,
) -> Result<ShareObjectBinding, (StatusCode, Json<ErrorResponse>)> {
    if let Some(asset_object_id) = asset_object_id {
        let row = client
            .query_opt(
                "SELECT asset_object_id::text,
                        object_locator,
                        share_protocol
                 FROM catalog.asset_object_binding
                 WHERE asset_object_id = $1::text::uuid
                   AND asset_version_id = $2::text::uuid
                   AND object_kind = 'share_object'",
                &[&asset_object_id, &asset_version_id],
            )
            .await
            .map_err(map_db_error)?;
        return row
            .map(|row| ShareObjectBinding {
                asset_object_id: row.get(0),
                object_locator: row.get(1),
                share_protocol: row.get(2),
            })
            .ok_or_else(|| {
                conflict(
                    "SHARE_GRANT_FORBIDDEN: requested asset_object_id is not a share_object for current asset version",
                    request_id,
                )
            });
    }

    let rows = client
        .query(
            "SELECT asset_object_id::text,
                    object_locator,
                    share_protocol
             FROM catalog.asset_object_binding
             WHERE asset_version_id = $1::text::uuid
               AND object_kind = 'share_object'
             ORDER BY asset_object_id DESC",
            &[&asset_version_id],
        )
        .await
        .map_err(map_db_error)?;
    if rows.is_empty() {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: share_object binding not found for current asset version",
            request_id,
        ));
    }

    let filtered: Vec<Row> = if let Some(requested_protocol) = requested_protocol.map(str::trim) {
        rows.into_iter()
            .filter(|row| {
                row.get::<_, Option<String>>(2)
                    .map(|value| value.eq_ignore_ascii_case(requested_protocol))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        rows
    };
    if filtered.is_empty() {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: share_object binding matching share_protocol not found",
            request_id,
        ));
    }
    if filtered.len() > 1 {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: multiple share_object bindings found; asset_object_id is required",
            request_id,
        ));
    }
    let row = &filtered[0];
    Ok(ShareObjectBinding {
        asset_object_id: row.get(0),
        object_locator: row.get(1),
        share_protocol: row.get(2),
    })
}

fn normalize_operation(
    operation: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = operation.unwrap_or("grant").trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "grant" | "revoke") {
        return Ok(normalized);
    }
    Err(bad_request(
        "operation must be `grant` or `revoke`",
        request_id,
    ))
}

fn normalize_share_protocol(
    requested: Option<&str>,
    binding: Option<&str>,
    default_value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let resolved = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| binding.map(str::trim).filter(|value| !value.is_empty()))
        .or_else(|| {
            default_value
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .ok_or_else(|| {
            conflict(
                "SHARE_GRANT_FORBIDDEN: share_protocol is required",
                request_id,
            )
        })?;
    let normalized = resolved.to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "share_grant"
            | "linked_dataset"
            | "datashare"
            | "delta_share"
            | "secure_view"
            | "readonly_schema"
            | "snowflake_share"
    ) {
        return Ok(normalized);
    }
    Err(conflict(
        &format!("SHARE_GRANT_FORBIDDEN: unsupported share_protocol `{resolved}`"),
        request_id,
    ))
}

fn validate_share_expiry(
    expires_at: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if !expires_at.contains('T') || !expires_at.ends_with('Z') {
        return Err(bad_request(
            "expires_at must be an RFC3339 UTC timestamp",
            request_id,
        ));
    }
    Ok(())
}

fn require_non_empty<'a>(
    value: Option<&'a str>,
    message: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| conflict(message, request_id))
}

fn enforce_subject_status(
    context: &ShareContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.buyer_status != "active" || context.seller_status != "active" {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: buyer/seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&context.buyer_metadata)
        || !is_subject_deliverable(&context.seller_metadata)
    {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: buyer/seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    if context.product_status != "listed" {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: product status is not listed",
            request_id,
        ));
    }
    if !matches!(
        context.asset_version_status.as_str(),
        "active" | "published"
    ) {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: asset version status is not active/published",
            request_id,
        ));
    }
    if !matches!(context.sku_status.as_str(), "active" | "listed") {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: sku status is not active/listed",
            request_id,
        ));
    }
    if !is_review_status_approved(&context.product_metadata) {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: product review status is not approved",
            request_id,
        ));
    }
    if is_product_risk_blocked(&context.product_metadata) {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: product is blocked by risk policy",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_manage_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(
        actor_role,
        "tenant_admin" | "platform_admin" | "platform_risk_settlement"
    ) {
        if let Some(tenant_id) = tenant_id
            && tenant_id != seller_org_id
            && actor_role == "tenant_admin"
        {
            return Err(conflict(
                "SHARE_GRANT_FORBIDDEN: tenant admin is outside seller scope",
                request_id,
            ));
        }
        return Ok(());
    }
    if matches!(actor_role, "seller_operator" | "seller_storage_operator")
        && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    Err(conflict(
        "SHARE_GRANT_FORBIDDEN: actor is outside seller scope",
        request_id,
    ))
}

fn enforce_read_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(actor_role, "platform_admin" | "platform_risk_settlement") {
        return Ok(());
    }
    if actor_role == "tenant_admin" {
        if tenant_id == Some(buyer_org_id) || tenant_id == Some(seller_org_id) {
            return Ok(());
        }
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: tenant admin is outside order scope",
            request_id,
        ));
    }
    if matches!(actor_role, "buyer_operator") && tenant_id == Some(buyer_org_id) {
        return Ok(());
    }
    if matches!(actor_role, "seller_operator" | "seller_storage_operator")
        && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    if actor_role == "tenant_audit_readonly"
        && (tenant_id == Some(buyer_org_id) || tenant_id == Some(seller_org_id))
    {
        return Ok(());
    }
    Err(conflict(
        "SHARE_GRANT_FORBIDDEN: actor is outside order scope",
        request_id,
    ))
}

fn enforce_share_ro(
    context: &ShareContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type != "SHARE_RO" {
        return Err(conflict(
            &format!(
                "SHARE_GRANT_FORBIDDEN: order sku_type `{}` is not SHARE_RO",
                context.sku_type
            ),
            request_id,
        ));
    }
    if context.payment_status != "paid" {
        return Err(conflict(
            "SHARE_GRANT_FORBIDDEN: payment status is not paid",
            request_id,
        ));
    }
    if !matches!(
        context.current_state.as_str(),
        "buyer_locked"
            | "share_enabled"
            | "share_granted"
            | "shared_active"
            | "revoked"
            | "expired"
    ) {
        return Err(conflict(
            &format!(
                "SHARE_GRANT_FORBIDDEN: current_state `{}` does not allow share grant management",
                context.current_state
            ),
            request_id,
        ));
    }
    Ok(())
}

async fn update_share_delivery_record(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    prepared_delivery_id: Option<&str>,
    committed_delivery_id: Option<&str>,
    share_protocol: &str,
    receipt_hash: &str,
    expires_at: &str,
    revoked: bool,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(delivery_id) = committed_delivery_id {
        client
            .execute(
                "UPDATE delivery.delivery_record
                 SET status = CASE WHEN $3 THEN 'revoked' ELSE 'committed' END,
                     delivery_type = 'share_grant',
                     delivery_route = $2,
                     delivery_commit_hash = $4,
                     receipt_hash = $5,
                     committed_at = COALESCE(committed_at, now()),
                     expires_at = COALESCE($6::timestamptz, expires_at),
                     updated_at = now()
                 WHERE delivery_id = $1::text::uuid",
                &[
                    &delivery_id,
                    &share_protocol,
                    &revoked,
                    &format!("share-grant:{share_protocol}:{receipt_hash}"),
                    &receipt_hash,
                    &Some(expires_at.to_string()),
                ],
            )
            .await
            .map_err(map_db_error)?;
        return Ok(());
    }

    let prepared_delivery_id = prepared_delivery_id.ok_or_else(|| {
        conflict(
            &format!(
                "SHARE_GRANT_FORBIDDEN: prepared delivery record not found for order `{order_id}`"
            ),
            None,
        )
    })?;
    client
        .execute(
            "UPDATE delivery.delivery_record
             SET status = CASE WHEN $3 THEN 'revoked' ELSE 'committed' END,
                 delivery_type = 'share_grant',
                 delivery_route = $2,
                 delivery_commit_hash = $4,
                 receipt_hash = $5,
                 committed_at = now(),
                 expires_at = $6::timestamptz,
                 updated_at = now()
             WHERE delivery_id = $1::text::uuid",
            &[
                &prepared_delivery_id,
                &share_protocol,
                &revoked,
                &format!("share-grant:{share_protocol}:{receipt_hash}"),
                &receipt_hash,
                &expires_at,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

async fn update_share_order_state(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    target_state: &str,
    payment_status: &str,
    layered_status: &LayeredStatus,
    reason_code: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .execute(
            "UPDATE trade.order_main
             SET status = $2,
                 payment_status = $3,
                 delivery_status = $4,
                 acceptance_status = $5,
                 settlement_status = $6,
                 dispute_status = $7,
                 last_reason_code = $8,
                 closed_at = CASE WHEN $2 IN ('revoked', 'expired') THEN now() ELSE closed_at END,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &target_state,
                &payment_status,
                &layered_status.delivery_status,
                &layered_status.acceptance_status,
                &layered_status.settlement_status,
                &layered_status.dispute_status,
                &reason_code,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

#[derive(Debug)]
struct LayeredStatus {
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
}

fn derive_share_order_layered_status(target_state: &str, payment_status: &str) -> LayeredStatus {
    match target_state {
        state if is_accepted_state("SHARE_RO", state) => LayeredStatus {
            delivery_status: "delivered".to_string(),
            acceptance_status: "accepted".to_string(),
            settlement_status: if payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "share_enabled" => LayeredStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        "revoked" => LayeredStatus {
            delivery_status: "closed".to_string(),
            acceptance_status: "closed".to_string(),
            settlement_status: "closed".to_string(),
            dispute_status: "none".to_string(),
        },
        "expired" => LayeredStatus {
            delivery_status: "expired".to_string(),
            acceptance_status: "expired".to_string(),
            settlement_status: "expired".to_string(),
            dispute_status: "none".to_string(),
        },
        _ => LayeredStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
    }
}

fn merged_share_metadata(
    base_metadata: Option<&Value>,
    metadata: Option<&Value>,
    scope_json: Option<&Value>,
    subscriber_ref: Option<&str>,
    actor_role: &str,
    operation: &str,
) -> Value {
    let mut merged = match base_metadata.cloned() {
        Some(Value::Object(map)) => map,
        _ => Map::new(),
    };
    if let Some(Value::Object(map)) = metadata.cloned() {
        for (key, value) in map {
            merged.insert(key, value);
        }
    }
    if merged.is_empty() {
        merged = match metadata.cloned() {
            Some(Value::Object(map)) => map,
            _ => Map::new(),
        };
    }
    if let Some(scope_json) = scope_json {
        merged.insert("scope_json".to_string(), scope_json.clone());
    }
    if let Some(subscriber_ref) = subscriber_ref {
        merged.insert(
            "subscriber_ref".to_string(),
            Value::String(subscriber_ref.to_string()),
        );
    }
    merged.insert(
        "managed_by_role".to_string(),
        Value::String(actor_role.to_string()),
    );
    merged.insert(
        "operation".to_string(),
        Value::String(operation.to_string()),
    );
    Value::Object(merged)
}

fn map_share_grant_response(
    row: &Row,
    sku_id: &str,
    sku_type: &str,
    current_state: &str,
    payment_status: &str,
    delivery_status: &str,
    operation: Option<&str>,
) -> ShareGrantResponseData {
    let metadata: Value = row.get(12);
    ShareGrantResponseData {
        data_share_grant_id: row.get(0),
        order_id: row.get(1),
        asset_object_id: row.get(2),
        sku_id: sku_id.to_string(),
        sku_type: sku_type.to_string(),
        recipient_ref: row.get(3),
        subscriber_ref: metadata
            .get("subscriber_ref")
            .and_then(Value::as_str)
            .map(str::to_string),
        share_protocol: row.get(4),
        access_locator: row.get(5),
        grant_status: row.get(6),
        read_only: row.get::<_, Option<bool>>(7).unwrap_or(true),
        receipt_hash: row.get(8),
        granted_at: row.get(9),
        revoked_at: row.get(10),
        expires_at: row.get(11),
        operation: operation.map(str::to_string),
        current_state: current_state.to_string(),
        payment_status: payment_status.to_string(),
        delivery_status: delivery_status.to_string(),
        metadata,
        created_at: row.get(13),
        updated_at: row.get(14),
    }
}

async fn write_delivery_share_audit_event(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    _actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    metadata: Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               'delivery',
               'order',
               $1::text::uuid,
               'role',
               $2,
               $3,
               $4,
               $5,
               $6::jsonb
             )",
            &[
                &order_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn is_subject_deliverable(metadata: &Value) -> bool {
    let risk_status = metadata
        .get("risk_status")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase());
    if matches!(
        risk_status.as_deref(),
        Some("blocked" | "frozen" | "high" | "high_risk" | "deny")
    ) {
        return false;
    }

    let sellable_status = metadata
        .get("sellable_status")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase());
    if matches!(
        sellable_status.as_deref(),
        Some("blocked" | "disabled" | "frozen" | "suspended")
    ) {
        return false;
    }

    let freeze_reason = metadata
        .get("freeze_reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    freeze_reason.is_empty()
}

fn is_review_status_approved(product_metadata: &Value) -> bool {
    let Some(status) = product_metadata
        .get("review_status")
        .and_then(Value::as_str)
    else {
        return true;
    };
    matches!(status, "approved" | "auto_approved" | "passed")
}

fn is_product_risk_blocked(product_metadata: &Value) -> bool {
    let risk_blocked = product_metadata
        .get("risk_blocked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let block_submit = product_metadata
        .get("risk_flags")
        .and_then(Value::as_object)
        .and_then(|flags| flags.get("block_submit"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    risk_blocked || block_submit
}

fn not_found(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn bad_request(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
