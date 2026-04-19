use crate::modules::delivery::domain::{build_watermark_placeholder_patch, merge_snapshot_patch};
use crate::modules::delivery::dto::{CommitOrderDeliveryRequest, CommitOrderDeliveryResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, enforce_seller_scope, not_found, resolve_storage_namespace_id,
    write_delivery_audit_event,
};
use crate::modules::order::repo::{map_db_error, write_trade_audit_event};
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::json;

const DELIVERY_REPORT_COMMIT_EVENT: &str = "delivery.report.commit";
const REPORT_RESULTS_BUCKET_ENV: &str = "BUCKET_REPORT_RESULTS";
const DEFAULT_REPORT_RESULTS_BUCKET: &str = "report-results";

pub async fn commit_report_delivery(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &CommitOrderDeliveryRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<CommitOrderDeliveryResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_report_commit_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               o.delivery_route_snapshot,
               o.trust_boundary_snapshot,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               s.sku_type,
               committed.delivery_id::text,
               committed.object_id::text,
               committed.object_uri,
               committed.content_hash,
               committed.delivery_commit_hash,
               committed.receipt_hash,
               to_char(committed.committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               latest_artifact.report_artifact_id::text,
               latest_artifact.report_type,
               latest_artifact.version_no,
               latest_artifact.status,
               prepared.delivery_id::text
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT d.delivery_id,
                      d.object_id,
                      so.object_uri,
                      so.content_hash,
                      d.delivery_commit_hash,
                      d.receipt_hash,
                      d.committed_at
               FROM delivery.delivery_record d
               LEFT JOIN delivery.storage_object so ON so.object_id = d.object_id
               WHERE d.order_id = o.order_id
                 AND d.status = 'committed'
                 AND d.delivery_type = 'report_delivery'
               ORDER BY d.committed_at DESC NULLS LAST, d.created_at DESC, d.delivery_id DESC
               LIMIT 1
             ) committed ON true
             LEFT JOIN LATERAL (
               SELECT report_artifact_id,
                      report_type,
                      version_no,
                      status
               FROM delivery.report_artifact
               WHERE order_id = o.order_id
                 AND status <> 'superseded'
               ORDER BY version_no DESC, created_at DESC, report_artifact_id DESC
               LIMIT 1
             ) latest_artifact ON true
             LEFT JOIN LATERAL (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'prepared'
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) prepared ON true
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
    let current_delivery_status: Option<String> = row.get(2);
    let current_acceptance_status: Option<String> = row.get(3);
    let current_settlement_status: Option<String> = row.get(4);
    let current_dispute_status: Option<String> = row.get(5);
    let delivery_route_snapshot: Option<String> = row.get(6);
    let order_trust_boundary_snapshot: serde_json::Value = row.get(7);
    let buyer_org_id: String = row.get(8);
    let seller_org_id: String = row.get(9);
    let sku_type: String = row.get(10);
    let committed_delivery_id: Option<String> = row.get(11);
    let committed_object_id: Option<String> = row.get(12);
    let committed_object_uri: Option<String> = row.get(13);
    let committed_report_hash: Option<String> = row.get(14);
    let committed_delivery_commit_hash: Option<String> = row.get(15);
    let committed_receipt_hash: Option<String> = row.get(16);
    let committed_at: Option<String> = row.get(17);
    let latest_report_artifact_id: Option<String> = row.get(18);
    let latest_report_type: Option<String> = row.get(19);
    let latest_report_version_no: Option<i32> = row.get(20);
    let latest_report_status: Option<String> = row.get(21);
    let prepared_delivery_id: Option<String> = row.get(22);

    enforce_seller_scope(actor_role, tenant_id, &seller_org_id, request_id)?;

    if payload.branch.trim().to_ascii_lowercase() != "report" {
        return Err(conflict(
            "REPORT_DELIVERY_COMMIT_FORBIDDEN: only `report` branch is supported by DLV-017",
            request_id,
        ));
    }
    if sku_type != "RPT_STD" {
        return Err(conflict(
            &format!(
                "REPORT_DELIVERY_COMMIT_FORBIDDEN: order sku_type `{sku_type}` is not RPT_STD"
            ),
            request_id,
        ));
    }

    let fallback_bucket = default_report_bucket_name();
    let object_uri = payload
        .object_uri
        .as_deref()
        .expect("report delivery validated object_uri");
    let resolved = resolve_storage_object_location(object_uri, Some(fallback_bucket.as_str()));
    let bucket_name = resolved.bucket_name.clone().unwrap_or(fallback_bucket);
    let object_key = resolved.object_key.clone().ok_or_else(|| {
        conflict(
            "REPORT_DELIVERY_COMMIT_FORBIDDEN: object key cannot be empty",
            request_id,
        )
    })?;
    let report_type = normalize_report_type(payload.report_type.as_deref(), request_id)?;

    if matches!(
        current_state.as_str(),
        "report_delivered" | "accepted" | "settled" | "closed"
    ) && committed_delivery_id.is_some()
        && committed_object_id.is_some()
        && committed_object_uri.is_some()
        && committed_at.is_some()
        && latest_report_artifact_id.is_some()
        && latest_report_type.is_some()
        && latest_report_version_no.is_some()
        && latest_report_status.is_some()
    {
        let committed_object_uri = committed_object_uri.expect("committed object uri");
        let committed_location =
            resolve_storage_object_location(&committed_object_uri, Some(bucket_name.as_str()));
        write_delivery_audit_event(
            &tx,
            "report_artifact",
            latest_report_artifact_id
                .as_deref()
                .expect("latest report artifact id"),
            actor_role,
            DELIVERY_REPORT_COMMIT_EVENT,
            "already_committed",
            request_id,
            trace_id,
            json!({
                "order_id": order_id,
                "branch": "report",
                "report_type": latest_report_type,
                "report_version_no": latest_report_version_no,
                "buyer_org_id": buyer_org_id,
                "seller_org_id": seller_org_id,
            }),
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(CommitOrderDeliveryResponseData {
            order_id: order_id.to_string(),
            delivery_id: committed_delivery_id.expect("committed delivery id"),
            branch: "report".to_string(),
            previous_state: current_state.clone(),
            current_state,
            payment_status: payment_status.clone(),
            delivery_status: current_delivery_status.unwrap_or_else(|| "delivered".to_string()),
            acceptance_status: current_acceptance_status
                .unwrap_or_else(|| "in_progress".to_string()),
            settlement_status: current_settlement_status.unwrap_or_else(|| {
                if payment_status == "paid" {
                    "pending_settlement".to_string()
                } else {
                    "not_started".to_string()
                }
            }),
            dispute_status: current_dispute_status.unwrap_or_else(|| "none".to_string()),
            object_id: committed_object_id,
            envelope_id: None,
            ticket_id: None,
            bucket_name: committed_location.bucket_name,
            object_key: committed_location.object_key,
            expires_at: None,
            download_limit: None,
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
            report_artifact_id: latest_report_artifact_id,
            report_type: latest_report_type,
            report_version_no: latest_report_version_no,
            report_status: latest_report_status,
            report_hash: committed_report_hash,
        });
    }

    if current_state != "report_generated" {
        return Err(conflict(
            &format!(
                "REPORT_DELIVERY_COMMIT_FORBIDDEN: current_state `{current_state}` is not report_generated"
            ),
            request_id,
        ));
    }
    let prepared_delivery_id = prepared_delivery_id.ok_or_else(|| {
        conflict(
            "REPORT_DELIVERY_COMMIT_FORBIDDEN: prepared delivery record not found",
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

    let version_no: i32 = tx
        .query_one(
            "SELECT COALESCE(MAX(version_no), 0) + 1
             FROM delivery.report_artifact
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

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
               'report_artifact',
               $2,
               'platform_object_storage',
               $1::text::uuid,
               $3,
               $4,
               $5,
               $6,
               $7,
               $8::text::uuid,
               'report_results',
               'standard'
             )
             RETURNING object_id::text",
            &[
                &seller_org_id,
                &object_uri,
                &payload.content_type.as_deref().unwrap_or("application/pdf"),
                &payload.size_bytes.expect("validated size_bytes"),
                &payload
                    .content_hash
                    .as_deref()
                    .expect("validated content_hash"),
                &payload.encryption_algo.as_deref().unwrap_or("none"),
                &payload.plaintext_visible_to_platform.unwrap_or(true),
                &storage_namespace_id,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let report_artifact_id: String = tx
        .query_one(
            "INSERT INTO delivery.report_artifact (
               order_id,
               object_id,
               report_type,
               version_no,
               status
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3,
               $4,
               'delivered'
             )
             RETURNING report_artifact_id::text",
            &[&order_id, &object_id, &report_type, &version_no],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let watermark_patch = build_watermark_placeholder_patch(
        &order_trust_boundary_snapshot,
        "report",
        payload.metadata.as_ref(),
    );
    let trust_boundary_patch = merge_snapshot_patch(
        &watermark_patch,
        &json!({
            "report_artifact": {
                "report_artifact_id": report_artifact_id,
                "report_type": report_type,
                "version_no": version_no,
                "report_hash": payload.content_hash,
                "bucket_name": bucket_name,
                "object_key": object_key,
                "object_id": object_id,
                "metadata": payload.metadata,
            }
        }),
    );

    let committed_at: String = tx
        .query_one(
            "UPDATE delivery.delivery_record
             SET object_id = $2::text::uuid,
                 delivery_type = 'report_delivery',
                 delivery_route = $3,
                 status = 'committed',
                 delivery_commit_hash = $4,
                 trust_boundary_snapshot = trust_boundary_snapshot || $5::jsonb,
                 receipt_hash = $6,
                 committed_at = now(),
                 updated_at = now()
             WHERE delivery_id = $1::text::uuid
             RETURNING to_char(committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &prepared_delivery_id,
                &object_id,
                &delivery_route_snapshot.as_deref().unwrap_or("result_package"),
                &payload
                    .delivery_commit_hash
                    .as_deref()
                    .expect("validated delivery_commit_hash"),
                &trust_boundary_patch,
                &payload
                    .receipt_hash
                    .as_deref()
                    .expect("validated receipt_hash"),
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let layered_status = derive_report_delivery_layered_status(&payment_status);
    tx.execute(
        "UPDATE trade.order_main
         SET status = 'report_delivered',
             delivery_status = $2,
             acceptance_status = $3,
             settlement_status = $4,
             dispute_status = $5,
             last_reason_code = 'rpt_std_report_delivered',
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

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        "trade.order.rpt_std.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    write_delivery_audit_event(
        &tx,
        "report_artifact",
        &report_artifact_id,
        actor_role,
        DELIVERY_REPORT_COMMIT_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "branch": "report",
            "report_type": report_type,
            "report_version_no": version_no,
            "bucket_name": bucket_name,
            "object_key": object_key,
            "object_id": object_id,
            "buyer_org_id": buyer_org_id,
            "seller_org_id": seller_org_id,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(CommitOrderDeliveryResponseData {
        order_id: order_id.to_string(),
        delivery_id: prepared_delivery_id,
        branch: "report".to_string(),
        previous_state: current_state,
        current_state: "report_delivered".to_string(),
        payment_status: payment_status.clone(),
        delivery_status: layered_status.delivery_status,
        acceptance_status: layered_status.acceptance_status,
        settlement_status: layered_status.settlement_status,
        dispute_status: layered_status.dispute_status,
        object_id: Some(object_id),
        envelope_id: None,
        ticket_id: None,
        bucket_name: Some(bucket_name),
        object_key: Some(object_key),
        expires_at: None,
        download_limit: None,
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
        report_artifact_id: Some(report_artifact_id),
        report_type: Some(report_type),
        report_version_no: Some(version_no),
        report_status: Some("delivered".to_string()),
        report_hash: payload.content_hash.clone(),
    })
}

fn validate_report_commit_request(
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
        .report_type
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("report_type is required", request_id));
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
    Ok(())
}

fn normalize_report_type(
    report_type: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let Some(report_type) = report_type.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(bad_request("report_type is required", request_id));
    };
    Ok(report_type.to_ascii_lowercase())
}

fn default_report_bucket_name() -> String {
    std::env::var(REPORT_RESULTS_BUCKET_ENV)
        .unwrap_or_else(|_| DEFAULT_REPORT_RESULTS_BUCKET.to_string())
}

struct ReportLayeredStatus {
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
}

fn derive_report_delivery_layered_status(payment_status: &str) -> ReportLayeredStatus {
    ReportLayeredStatus {
        delivery_status: "delivered".to_string(),
        acceptance_status: "in_progress".to_string(),
        settlement_status: if payment_status == "paid" {
            "pending_settlement".to_string()
        } else {
            "not_started".to_string()
        },
        dispute_status: "none".to_string(),
    }
}
