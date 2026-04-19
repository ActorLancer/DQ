use crate::modules::delivery::domain::{
    StorageGatewayAccessAudit, StorageGatewayDownloadRestriction, StorageGatewayIntegrity,
    StorageGatewayObjectLocator, StorageGatewaySnapshot, StorageGatewayWatermarkPolicy,
};
use crate::modules::delivery::events::STORAGE_GATEWAY_READ_EVENT;
use crate::modules::order::repo::map_db_error;
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::ErrorResponse;
use serde_json::Value;
use std::collections::HashMap;

pub async fn load_storage_gateway_snapshots(
    client: &Client,
    order_id: &str,
) -> Result<HashMap<String, StorageGatewaySnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               dr.delivery_id::text,
               so.object_id::text,
               so.object_uri,
               so.location_type,
               so.storage_zone,
               ns.bucket_name,
               ns.namespace_name,
               ns.namespace_kind,
               ns.provider_type,
               so.content_type,
               so.size_bytes,
               so.content_hash,
               so.encryption_algo,
               so.plaintext_visible_to_platform,
               ke.envelope_id::text,
               dr.delivery_commit_hash,
               dr.receipt_hash,
               dr.sensitive_delivery_mode,
               dr.disclosure_review_status,
               dr.trust_boundary_snapshot,
               ticket.ticket_id::text,
               to_char(ticket.expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ticket.download_limit,
               ticket.download_count,
               ticket.status,
               access.access_count,
               to_char(access.last_downloaded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               access.last_client_fingerprint
             FROM delivery.delivery_record dr
             LEFT JOIN delivery.storage_object so ON so.object_id = dr.object_id
             LEFT JOIN catalog.storage_namespace ns ON ns.storage_namespace_id = so.storage_namespace_id
             LEFT JOIN delivery.key_envelope ke ON ke.envelope_id = dr.envelope_id
             LEFT JOIN LATERAL (
               SELECT ticket_id, expire_at, download_limit, download_count, status
               FROM delivery.delivery_ticket
               WHERE order_id = dr.order_id
               ORDER BY created_at DESC, ticket_id DESC
               LIMIT 1
             ) AS ticket ON TRUE
             LEFT JOIN LATERAL (
               SELECT COUNT(*)::bigint AS access_count,
                      MAX(downloaded_at) AS last_downloaded_at,
                      (ARRAY_AGG(client_fingerprint ORDER BY downloaded_at DESC NULLS LAST))[1] AS last_client_fingerprint
               FROM delivery.delivery_receipt rec
               WHERE rec.delivery_id = dr.delivery_id
             ) AS access ON TRUE
             WHERE dr.order_id = $1::text::uuid
             ORDER BY dr.updated_at DESC, dr.delivery_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row: Row| {
            let delivery_id: String = row.get(0);
            let object_uri = row.get::<_, Option<String>>(2).unwrap_or_default();
            let bucket_name = row.get::<_, Option<String>>(5);
            let resolved = if object_uri.is_empty() {
                None
            } else {
                Some(resolve_storage_object_location(
                    &object_uri,
                    bucket_name.as_deref(),
                ))
            };
            let trust_boundary_snapshot: Value = row.get(19);
            let sensitive_delivery_mode = row.get::<_, String>(17);
            let disclosure_review_status = row.get::<_, String>(18);
            let download_limit = row.get::<_, Option<i32>>(22);
            let download_count = row.get::<_, Option<i32>>(23);
            let gateway = StorageGatewaySnapshot {
                object_locator: row.get::<_, Option<String>>(1).map(|object_id| {
                    StorageGatewayObjectLocator {
                        object_id,
                        object_uri,
                        bucket_name: resolved
                            .as_ref()
                            .and_then(|value| value.bucket_name.clone()),
                        object_key: resolved.as_ref().and_then(|value| value.object_key.clone()),
                        location_type: row
                            .get::<_, Option<String>>(3)
                            .unwrap_or_else(|| "platform_object_storage".to_string()),
                        storage_zone: row
                            .get::<_, Option<String>>(4)
                            .unwrap_or_else(|| "delivery".to_string()),
                        provider_type: row.get(8),
                        namespace_name: row.get(6),
                        namespace_kind: row.get(7),
                        content_type: row.get(9),
                        size_bytes: row.get(10),
                        plaintext_visible_to_platform: row
                            .get::<_, Option<bool>>(13)
                            .unwrap_or(false),
                    }
                }),
                integrity: StorageGatewayIntegrity {
                    content_hash: row.get(11),
                    encryption_algo: row.get(12),
                    delivery_commit_hash: row.get(15),
                    receipt_hash: row.get(16),
                    envelope_id: row.get(14),
                },
                watermark_policy: derive_watermark_policy(
                    &trust_boundary_snapshot,
                    &sensitive_delivery_mode,
                    &disclosure_review_status,
                ),
                download_restriction: row.get::<_, Option<String>>(20).map(|ticket_id| {
                    let limit = download_limit.unwrap_or_default();
                    let count = download_count.unwrap_or_default();
                    StorageGatewayDownloadRestriction {
                        ticket_id,
                        expire_at: row.get::<_, Option<String>>(21).unwrap_or_default(),
                        download_limit: limit,
                        download_count: count,
                        remaining_downloads: (limit - count).max(0),
                        current_status: row
                            .get::<_, Option<String>>(24)
                            .unwrap_or_else(|| "active".to_string()),
                    }
                }),
                access_audit: StorageGatewayAccessAudit {
                    access_count: row.get::<_, Option<i64>>(25).unwrap_or_default(),
                    last_downloaded_at: row.get(26),
                    last_client_fingerprint: row.get(27),
                },
            };
            (delivery_id, gateway)
        })
        .collect())
}

pub async fn write_storage_gateway_read_audit(
    client: &(impl GenericClient + Sync),
    delivery_id: &str,
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let metadata = serde_json::json!({
        "actor_role": actor_role,
        "order_id": order_id,
        "access_channel": "order_read",
    });
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
               'delivery_record',
               $1::text::uuid,
               'role',
               $2,
               'success',
               $3,
               $4,
               $5::jsonb
             )",
            &[
                &delivery_id,
                &STORAGE_GATEWAY_READ_EVENT,
                &request_id,
                &trace_id,
                &metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn derive_watermark_policy(
    trust_boundary_snapshot: &Value,
    sensitive_delivery_mode: &str,
    disclosure_review_status: &str,
) -> StorageGatewayWatermarkPolicy {
    let policy = trust_boundary_snapshot
        .get("watermark_policy")
        .cloned()
        .or_else(|| {
            trust_boundary_snapshot.get("watermark_rule").map(|value| {
                serde_json::json!({
                    "policy": value,
                })
            })
        })
        .unwrap_or_else(|| serde_json::json!({"policy": "reserved_for_pipeline"}));

    let fingerprint_fields = trust_boundary_snapshot
        .get("fingerprint_fields")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mode = trust_boundary_snapshot
        .get("watermark_mode")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if trust_boundary_snapshot.get("watermark_rule").is_some() {
                "rule_bound".to_string()
            } else {
                "placeholder".to_string()
            }
        });

    let watermark_hash = trust_boundary_snapshot
        .get("watermark_hash")
        .and_then(|value| value.as_str())
        .map(str::to_string);

    StorageGatewayWatermarkPolicy {
        mode,
        rule: policy,
        fingerprint_fields,
        watermark_hash,
        sensitive_delivery_mode: sensitive_delivery_mode.to_string(),
        disclosure_review_status: disclosure_review_status.to_string(),
    }
}
