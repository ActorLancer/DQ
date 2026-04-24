use super::{
    DownloadTicketCachePayload, delete_download_ticket_cache,
    load_download_ticket_cache_ttl_seconds, redis_download_ticket_key, set_download_ticket_cache,
};
use crate::modules::delivery::dto::{DownloadFileAccessData, DownloadKeyEnvelopeData};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Value, json};

const DOWNLOAD_TICKET_AUDIT_EVENT: &str = "delivery.file.download";

pub async fn consume_download_ticket(
    client: &mut Client,
    cache_payload: &DownloadTicketCachePayload,
    actor_role: &str,
    raw_token: &str,
    client_fingerprint: Option<&str>,
    source_ip: Option<&str>,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DownloadFileAccessData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT
               o.status,
               o.buyer_org_id::text,
               s.sku_type,
               dr.delivery_id::text,
               dr.delivery_commit_hash,
               ticket.ticket_id::text,
               ticket.token_hash,
               to_char(ticket.expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ticket.download_limit,
               ticket.download_count,
               ticket.status,
               (ticket.expire_at > now()) AS ticket_not_expired,
               to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             JOIN delivery.delivery_record dr
               ON dr.delivery_id = $3::text::uuid
              AND dr.order_id = o.order_id
              AND dr.status = 'committed'
             JOIN delivery.delivery_ticket ticket
               ON ticket.ticket_id = $2::text::uuid
              AND ticket.order_id = o.order_id
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o, ticket",
            &[
                &cache_payload.order_id,
                &cache_payload.ticket_id,
                &cache_payload.delivery_id,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: committed delivery or ticket not found",
            request_id,
        ));
    };

    let current_state: String = row.get(0);
    let buyer_org_id: String = row.get(1);
    let sku_type: String = row.get(2);
    let delivery_id: String = row.get(3);
    let delivery_commit_hash: String = row.get(4);
    let ticket_id: String = row.get(5);
    let token_hash: String = row.get(6);
    let expire_at: String = row.get(7);
    let download_limit: i32 = row.get(8);
    let download_count: i32 = row.get(9);
    let ticket_status: String = row.get(10);
    let ticket_not_expired: bool = row.get(11);
    let downloaded_at: String = row.get(12);

    if sku_type != "FILE_STD" {
        return Err(conflict(
            &format!("DOWNLOAD_TICKET_FORBIDDEN: order sku_type `{sku_type}` is not FILE_STD"),
            request_id,
        ));
    }
    if !matches!(
        current_state.as_str(),
        "delivered" | "accepted" | "settled" | "closed"
    ) {
        return Err(conflict(
            &format!(
                "DOWNLOAD_TICKET_FORBIDDEN: current_state `{current_state}` does not allow file download"
            ),
            request_id,
        ));
    }
    if buyer_org_id != cache_payload.buyer_org_id {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: buyer scope mismatch",
            request_id,
        ));
    }
    if delivery_id != cache_payload.delivery_id
        || delivery_commit_hash != cache_payload.delivery_commit_hash
    {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: delivery snapshot mismatch",
            request_id,
        ));
    }
    if ticket_status != "active" {
        return Err(conflict(
            &format!("DOWNLOAD_TICKET_FORBIDDEN: ticket status `{ticket_status}` is not active"),
            request_id,
        ));
    }
    if !ticket_not_expired {
        tx.execute(
            "UPDATE delivery.delivery_ticket
             SET status = 'expired'
             WHERE ticket_id = $1::text::uuid",
            &[&ticket_id],
        )
        .await
        .map_err(map_db_error)?;
        tx.commit().await.map_err(map_db_error)?;
        delete_download_ticket_cache(&redis_download_ticket_key(&ticket_id)).await;
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket expired",
            request_id,
        ));
    }
    if download_count >= download_limit {
        tx.execute(
            "UPDATE delivery.delivery_ticket
             SET status = 'exhausted'
             WHERE ticket_id = $1::text::uuid",
            &[&ticket_id],
        )
        .await
        .map_err(map_db_error)?;
        tx.commit().await.map_err(map_db_error)?;
        delete_download_ticket_cache(&redis_download_ticket_key(&ticket_id)).await;
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: download limit reached",
            request_id,
        ));
    }

    let expected_token_hash: String = tx
        .query_one("SELECT md5($1)", &[&raw_token])
        .await
        .map_err(map_db_error)?
        .get(0);
    if expected_token_hash != token_hash {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: token hash mismatch",
            request_id,
        ));
    }

    let next_download_count = download_count + 1;
    let next_ticket_status = if next_download_count >= download_limit {
        "exhausted".to_string()
    } else {
        "active".to_string()
    };
    let remaining_downloads = (download_limit - next_download_count).max(0);
    let receipt_seed = format!(
        "{ticket_id}:{delivery_id}:{next_download_count}:{}:{}:{}:{}",
        request_id.unwrap_or_default(),
        trace_id.unwrap_or_default(),
        client_fingerprint.unwrap_or_default(),
        source_ip.unwrap_or_default()
    );
    let receipt_hash: String = tx
        .query_one("SELECT md5($1)", &[&receipt_seed])
        .await
        .map_err(map_db_error)?
        .get(0);

    tx.execute(
        "UPDATE delivery.delivery_ticket
         SET download_count = $2,
             status = $3
         WHERE ticket_id = $1::text::uuid",
        &[&ticket_id, &next_download_count, &next_ticket_status],
    )
    .await
    .map_err(map_db_error)?;

    let receipt_row = tx
        .query_one(
            "INSERT INTO delivery.delivery_receipt (
               delivery_id, order_id, receipt_hash, client_fingerprint, source_ip
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3, NULLIF($4, ''), NULLIF($5, '')::inet
             )
             RETURNING receipt_id::text,
                       to_char(downloaded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &delivery_id,
                &cache_payload.order_id,
                &receipt_hash,
                &client_fingerprint.unwrap_or_default(),
                &source_ip.unwrap_or_default(),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let receipt_id: String = receipt_row.get(0);
    let downloaded_at: String = receipt_row.get(1);

    write_download_audit_event(
        &tx,
        &receipt_id,
        actor_role,
        request_id,
        trace_id,
        json!({
            "order_id": cache_payload.order_id,
            "delivery_id": delivery_id,
            "ticket_id": ticket_id,
            "bucket_name": cache_payload.bucket_name,
            "object_key": cache_payload.object_key,
            "download_limit": download_limit,
            "download_count": next_download_count,
            "remaining_downloads": remaining_downloads,
            "client_fingerprint": client_fingerprint,
            "source_ip": source_ip,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    let _ = sync_download_ticket_cache(
        cache_payload,
        next_download_count,
        remaining_downloads,
        &next_ticket_status,
    )
    .await;

    Ok(DownloadFileAccessData {
        order_id: cache_payload.order_id.clone(),
        delivery_id,
        ticket_id,
        receipt_id,
        receipt_hash,
        downloaded_at,
        ticket_status: next_ticket_status,
        download_limit,
        download_count: next_download_count,
        remaining_downloads,
        bucket_name: cache_payload.bucket_name.clone(),
        object_key: cache_payload.object_key.clone(),
        content_type: cache_payload.content_type.clone(),
        content_hash: cache_payload.content_hash.clone(),
        delivery_commit_hash,
        key_envelope: DownloadKeyEnvelopeData {
            envelope_id: cache_payload.envelope_id.clone(),
            key_cipher: cache_payload.key_cipher.clone(),
            key_control_mode: cache_payload.key_control_mode.clone(),
            unwrap_policy_json: cache_payload.unwrap_policy_json.clone(),
            key_version: cache_payload.key_version.clone(),
        },
    })
}

async fn sync_download_ticket_cache(
    cache_payload: &DownloadTicketCachePayload,
    next_download_count: i32,
    remaining_downloads: i32,
    next_ticket_status: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let redis_key = redis_download_ticket_key(&cache_payload.ticket_id);
    if next_ticket_status != "active" {
        delete_download_ticket_cache(&redis_key).await;
        return Ok(());
    }
    let Some(ttl_seconds) =
        load_download_ticket_cache_ttl_seconds(&cache_payload.ticket_id).await?
    else {
        return Ok(());
    };
    let mut next_payload = cache_payload.clone();
    next_payload.download_count = next_download_count;
    next_payload.remaining_downloads = remaining_downloads;
    next_payload.ticket_status = next_ticket_status.to_string();
    set_download_ticket_cache(&redis_key, ttl_seconds, &next_payload).await
}

async fn write_download_audit_event(
    client: &(impl GenericClient + Sync),
    receipt_id: &str,
    actor_role: &str,
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
               'delivery_receipt',
               $1::text::uuid,
               'role',
               $2,
               'downloaded',
               $3,
               $4,
               $5::jsonb || jsonb_build_object('actor_role', $6)
             )",
            &[
                &receipt_id,
                &DOWNLOAD_TICKET_AUDIT_EVENT,
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

fn conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: "DOWNLOAD_TICKET_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
