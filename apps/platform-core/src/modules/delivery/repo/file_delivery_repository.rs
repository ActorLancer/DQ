use super::outbox_repository::{
    build_delivery_receipt_outbox_payload, write_delivery_receipt_outbox_event,
};
use crate::modules::delivery::domain::{build_watermark_placeholder_patch, merge_snapshot_patch};
use crate::modules::delivery::dto::{CommitOrderDeliveryRequest, CommitOrderDeliveryResponseData};
use crate::modules::order::domain::derive_layered_status;
use crate::modules::order::repo::map_db_error;
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

pub async fn commit_file_delivery(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &CommitOrderDeliveryRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<CommitOrderDeliveryResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_commit_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_route_snapshot,
               o.trust_boundary_snapshot,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               s.sku_type,
               committed.delivery_id::text,
               committed.object_id::text,
               committed.envelope_id::text,
               committed.delivery_commit_hash,
               committed.receipt_hash,
               to_char(committed.committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(committed.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               prepared.delivery_id::text,
               active_ticket.ticket_id::text,
               active_ticket.download_limit,
               to_char(active_ticket.expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT delivery_id, object_id, envelope_id, delivery_commit_hash, receipt_hash, committed_at, expires_at
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'committed'
               ORDER BY committed_at DESC NULLS LAST, created_at DESC, delivery_id DESC
               LIMIT 1
             ) committed ON true
             LEFT JOIN LATERAL (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'prepared'
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) prepared ON true
             LEFT JOIN LATERAL (
               SELECT ticket_id, download_limit, expire_at
               FROM delivery.delivery_ticket
               WHERE order_id = o.order_id
                 AND status = 'active'
               ORDER BY created_at DESC, ticket_id DESC
               LIMIT 1
             ) active_ticket ON true
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    let current_state: String = row.get(0);
    let payment_status: String = row.get(1);
    let delivery_route_snapshot: Option<String> = row.get(2);
    let trust_boundary_snapshot: Value = row.get(3);
    let buyer_org_id: String = row.get(4);
    let seller_org_id: String = row.get(5);
    let sku_type: String = row.get(6);
    let committed_delivery_id: Option<String> = row.get(7);
    let committed_object_id: Option<String> = row.get(8);
    let committed_envelope_id: Option<String> = row.get(9);
    let committed_delivery_commit_hash: Option<String> = row.get(10);
    let committed_receipt_hash: Option<String> = row.get(11);
    let committed_at: Option<String> = row.get(12);
    let committed_expires_at: Option<String> = row.get(13);
    let prepared_delivery_id: Option<String> = row.get(14);
    let active_ticket_id: Option<String> = row.get(15);
    let active_download_limit: Option<i32> = row.get(16);
    let active_ticket_expire_at: Option<String> = row.get(17);

    enforce_seller_scope(actor_role, tenant_id, &seller_org_id, request_id)?;

    if payload.branch.trim().to_ascii_lowercase() != "file" {
        return Err(conflict(
            "FILE_DELIVERY_COMMIT_FORBIDDEN: only `file` branch is supported by DLV-002",
            request_id,
        ));
    }
    if sku_type != "FILE_STD" {
        return Err(conflict(
            &format!("FILE_DELIVERY_COMMIT_FORBIDDEN: order sku_type `{sku_type}` is not FILE_STD"),
            request_id,
        ));
    }

    let fallback_bucket = default_bucket_name();
    let object_uri = payload
        .object_uri
        .as_deref()
        .expect("file delivery validated object_uri");
    let resolved = resolve_storage_object_location(object_uri, Some(fallback_bucket.as_str()));
    let bucket_name = resolved.bucket_name.clone().unwrap_or(fallback_bucket);
    let object_key = resolved.object_key.clone().ok_or_else(|| {
        conflict(
            "FILE_DELIVERY_COMMIT_FORBIDDEN: object key cannot be empty",
            request_id,
        )
    })?;

    let storage_namespace_id = resolve_storage_namespace_id(
        &tx,
        &seller_org_id,
        payload.storage_namespace_id.as_deref(),
        &bucket_name,
        request_id,
    )
    .await?;
    let delivery_trust_boundary_snapshot = merge_snapshot_patch(
        &trust_boundary_snapshot,
        &build_watermark_placeholder_patch(&trust_boundary_snapshot, "file", None),
    );

    if matches!(
        current_state.as_str(),
        "delivered" | "accepted" | "settled" | "closed"
    ) && committed_delivery_id.is_some()
        && committed_object_id.is_some()
        && committed_envelope_id.is_some()
        && active_ticket_id.is_some()
        && committed_at.is_some()
        && committed_expires_at.is_some()
        && committed_delivery_commit_hash.is_some()
        && committed_receipt_hash.is_some()
        && active_ticket_expire_at.is_some()
        && active_download_limit.is_some()
    {
        write_delivery_audit_event(
            &tx,
            "delivery_record",
            committed_delivery_id
                .as_deref()
                .expect("committed delivery id"),
            actor_role,
            "delivery.file.commit",
            "already_committed",
            request_id,
            trace_id,
            json!({
                "order_id": order_id,
                "branch": "file",
                "bucket_name": bucket_name,
                "object_key": object_key,
                "buyer_org_id": buyer_org_id,
                "seller_org_id": seller_org_id,
            }),
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(CommitOrderDeliveryResponseData {
            order_id: order_id.to_string(),
            delivery_id: committed_delivery_id.expect("committed delivery id"),
            branch: "file".to_string(),
            previous_state: current_state.clone(),
            current_state,
            payment_status,
            delivery_status: derive_layered_status("delivered", "paid").delivery_status,
            acceptance_status: derive_layered_status("delivered", "paid").acceptance_status,
            settlement_status: derive_layered_status("delivered", "paid").settlement_status,
            dispute_status: derive_layered_status("delivered", "paid").dispute_status,
            object_id: committed_object_id,
            envelope_id: committed_envelope_id,
            ticket_id: active_ticket_id,
            bucket_name: Some(bucket_name),
            object_key: Some(object_key),
            expires_at: committed_expires_at,
            download_limit: active_download_limit,
            receipt_hash: committed_receipt_hash,
            delivery_commit_hash: committed_delivery_commit_hash,
            committed_at: committed_at.expect("committed at"),
            app_id: None,
            app_name: None,
            app_type: None,
            client_id: None,
            api_credential_id: None,
            api_key: None,
            api_key_hint: None,
            quota_json: None,
            rate_limit_json: None,
            upstream_mode: None,
            operation: Some("already_committed".to_string()),
            endpoint_uri: None,
            credential_status: None,
            report_artifact_id: None,
            report_type: None,
            report_version_no: None,
            report_status: None,
            report_hash: None,
        });
    }

    if current_state != "seller_delivering" {
        return Err(conflict(
            &format!(
                "FILE_DELIVERY_COMMIT_FORBIDDEN: current_state `{current_state}` is not seller_delivering"
            ),
            request_id,
        ));
    }
    let prepared_delivery_id = prepared_delivery_id.ok_or_else(|| {
        conflict(
            "FILE_DELIVERY_COMMIT_FORBIDDEN: prepared delivery record not found",
            request_id,
        )
    })?;

    let object_id: String = tx
        .query_one(
            "INSERT INTO delivery.storage_object (
               org_id,
               object_type,
               object_uri,
               location_type,
               managed_by_org_id,
               content_type,
               size_bytes,
               content_hash,
               encryption_algo,
               plaintext_visible_to_platform,
               storage_namespace_id,
               storage_zone,
               storage_class
             ) VALUES (
               $1::text::uuid,
               'delivery_object',
               $2,
               'platform_object_storage',
               $1::text::uuid,
               $3,
               $4,
               $5,
               $6,
               $7,
               $8::text::uuid,
               'delivery',
               'standard'
             )
             RETURNING object_id::text",
            &[
                &seller_org_id,
                &object_uri,
                &payload
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
                &payload.size_bytes.expect("validated size_bytes"),
                &payload
                    .content_hash
                    .as_deref()
                    .expect("validated content_hash"),
                &payload.encryption_algo.as_deref().unwrap_or("AES-GCM"),
                &payload.plaintext_visible_to_platform.unwrap_or(false),
                &storage_namespace_id,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let envelope_id: String = tx
        .query_one(
            "INSERT INTO delivery.key_envelope (
               order_id,
               recipient_type,
               recipient_id,
               key_cipher,
               key_control_mode,
               unwrap_policy_json,
               key_version
             ) VALUES (
               $1::text::uuid,
               'organization',
               $2::text::uuid,
               $3,
               $4,
               $5::jsonb,
               $6
             )
             RETURNING envelope_id::text",
            &[
                &order_id,
                &buyer_org_id,
                &payload.key_cipher.as_deref().expect("validated key_cipher"),
                &payload
                    .key_control_mode
                    .as_deref()
                    .unwrap_or("seller_managed"),
                &payload
                    .unwrap_policy_json
                    .clone()
                    .unwrap_or_else(|| json!({"mode": "local_mock", "buyer_org_id": buyer_org_id})),
                &payload.key_version,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    tx.execute(
        "UPDATE delivery.delivery_ticket
         SET status = 'superseded'
         WHERE order_id = $1::text::uuid
           AND status = 'active'",
        &[&order_id],
    )
    .await
    .map_err(map_db_error)?;

    let generated_token_hash = format!("ticket-pending:{}:{}", order_id, prepared_delivery_id);
    let ticket_id: String = tx
        .query_one(
            "INSERT INTO delivery.delivery_ticket (
               order_id,
               buyer_org_id,
               token_hash,
               expire_at,
               download_limit,
               download_count,
               status
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3,
               $4::timestamptz,
               $5,
               0,
               'active'
             )
             RETURNING ticket_id::text",
            &[
                &order_id,
                &buyer_org_id,
                &generated_token_hash,
                &payload.expire_at.as_deref().expect("validated expire_at"),
                &payload.download_limit.expect("validated download_limit"),
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let committed_at: String = tx
        .query_one(
            "UPDATE delivery.delivery_record
             SET object_id = $2::text::uuid,
                 delivery_type = 'file_download',
                 delivery_route = $3,
                 status = 'committed',
                 delivery_commit_hash = $4,
                 envelope_id = $5::text::uuid,
                 trust_boundary_snapshot = COALESCE(trust_boundary_snapshot, '{}'::jsonb) || $6::jsonb,
                 receipt_hash = $7,
                 committed_at = now(),
                 expires_at = $8::timestamptz,
                 updated_at = now()
             WHERE delivery_id = $1::text::uuid
             RETURNING to_char(committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &prepared_delivery_id,
                &object_id,
                &delivery_route_snapshot.as_deref().unwrap_or("signed_url"),
                &payload
                    .delivery_commit_hash
                    .as_deref()
                    .expect("validated delivery_commit_hash"),
                &envelope_id,
                &delivery_trust_boundary_snapshot,
                &payload
                    .receipt_hash
                    .as_deref()
                    .expect("validated receipt_hash"),
                &payload
                    .expire_at
                    .as_deref()
                    .expect("validated expire_at"),
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let layered_status = derive_layered_status("delivered", &payment_status);
    tx.execute(
        "UPDATE trade.order_main
         SET status = 'delivered',
             delivery_status = $2,
             acceptance_status = $3,
             settlement_status = $4,
             dispute_status = $5,
             last_reason_code = 'delivery_file_committed',
             updated_at = now()
         WHERE order_id = $1::text::uuid",
        &[
            &order_id,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
        ],
    )
    .await
    .map_err(map_db_error)?;

    write_delivery_audit_event(
        &tx,
        "delivery_record",
        &prepared_delivery_id,
        actor_role,
        "delivery.file.commit",
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "branch": "file",
            "bucket_name": bucket_name,
            "object_key": object_key,
            "ticket_id": ticket_id,
            "envelope_id": envelope_id,
            "buyer_org_id": buyer_org_id,
            "seller_org_id": seller_org_id,
        }),
    )
    .await?;
    write_delivery_receipt_outbox_event(
        &tx,
        &prepared_delivery_id,
        &build_delivery_receipt_outbox_payload(
            "file",
            order_id,
            &prepared_delivery_id,
            &sku_type,
            actor_role,
            &buyer_org_id,
            &seller_org_id,
            "delivered",
            &payment_status,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
            payload.receipt_hash.as_deref(),
            payload.delivery_commit_hash.as_deref(),
            Some("file_download"),
            delivery_route_snapshot.as_deref(),
            Some(&committed_at),
            json!({
                "object_id": object_id,
                "envelope_id": envelope_id,
                "ticket_id": ticket_id,
                "bucket_name": bucket_name,
                "object_key": object_key,
                "download_limit": payload.download_limit,
                "expires_at": payload.expire_at,
            }),
        ),
        request_id,
        trace_id,
        idempotency_key,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(CommitOrderDeliveryResponseData {
        order_id: order_id.to_string(),
        delivery_id: prepared_delivery_id,
        branch: "file".to_string(),
        previous_state: current_state,
        current_state: "delivered".to_string(),
        payment_status,
        delivery_status: layered_status.delivery_status,
        acceptance_status: layered_status.acceptance_status,
        settlement_status: layered_status.settlement_status,
        dispute_status: layered_status.dispute_status,
        object_id: Some(object_id),
        envelope_id: Some(envelope_id),
        ticket_id: Some(ticket_id),
        bucket_name: Some(bucket_name),
        object_key: Some(object_key),
        expires_at: payload.expire_at.clone(),
        download_limit: payload.download_limit,
        receipt_hash: payload.receipt_hash.clone(),
        delivery_commit_hash: payload.delivery_commit_hash.clone(),
        committed_at,
        app_id: None,
        app_name: None,
        app_type: None,
        client_id: None,
        api_credential_id: None,
        api_key: None,
        api_key_hint: None,
        quota_json: None,
        rate_limit_json: None,
        upstream_mode: None,
        operation: Some("committed".to_string()),
        endpoint_uri: None,
        credential_status: None,
        report_artifact_id: None,
        report_type: None,
        report_version_no: None,
        report_status: None,
        report_hash: None,
    })
}

pub(crate) async fn resolve_storage_namespace_id(
    client: &(impl GenericClient + Sync),
    seller_org_id: &str,
    storage_namespace_id: Option<&str>,
    bucket_name: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let row = if let Some(storage_namespace_id) = storage_namespace_id {
        client
            .query_opt(
                "SELECT storage_namespace_id::text
                 FROM catalog.storage_namespace
                 WHERE storage_namespace_id = $1::text::uuid
                   AND owner_org_id = $2::text::uuid
                   AND status = 'active'",
                &[&storage_namespace_id, &seller_org_id],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_opt(
                "SELECT storage_namespace_id::text
                 FROM catalog.storage_namespace
                 WHERE owner_org_id = $1::text::uuid
                   AND bucket_name = $2
                   AND status = 'active'
                 ORDER BY storage_namespace_id DESC
                 LIMIT 1",
                &[&seller_org_id, &bucket_name],
            )
            .await
            .map_err(map_db_error)?
    };

    row.map(|value| value.get::<_, String>(0)).ok_or_else(|| {
        conflict(
            "FILE_DELIVERY_COMMIT_FORBIDDEN: active storage namespace not found for seller/bucket",
            request_id,
        )
    })
}

pub(crate) fn validate_commit_request(
    payload: &CommitOrderDeliveryRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload
        .object_uri
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("object_uri is required", request_id));
    }
    if payload.size_bytes.unwrap_or_default() <= 0 {
        return Err(bad_request("size_bytes must be > 0", request_id));
    }
    if payload
        .content_hash
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("content_hash is required", request_id));
    }
    if payload
        .key_cipher
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("key_cipher is required", request_id));
    }
    if payload.download_limit.unwrap_or_default() <= 0 {
        return Err(bad_request("download_limit must be > 0", request_id));
    }
    if payload
        .delivery_commit_hash
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("delivery_commit_hash is required", request_id));
    }
    if payload
        .receipt_hash
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("receipt_hash is required", request_id));
    }
    if payload
        .expire_at
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("expire_at is required", request_id));
    }
    Ok(())
}

pub(crate) fn enforce_seller_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(actor_role, "tenant_admin" | "seller_operator") && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "file delivery commit is forbidden for tenant scope".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

pub(crate) async fn write_delivery_audit_event(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
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
               $2,
               $1::text::uuid,
               'role',
               $3,
               $4,
               $5,
               $6,
               $7::jsonb || jsonb_build_object('actor_role', $8)
             )",
            &[
                &ref_id,
                &ref_type,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &metadata,
                &actor_role,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub(crate) fn default_bucket_name() -> String {
    std::env::var("BUCKET_DELIVERY_OBJECTS").unwrap_or_else(|_| "delivery-objects".to_string())
}

pub(crate) fn not_found(
    order_id: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("order not found: {order_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}

pub(crate) fn bad_request(
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

pub(crate) fn conflict(
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
