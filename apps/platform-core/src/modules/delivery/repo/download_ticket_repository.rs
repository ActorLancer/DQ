use crate::modules::delivery::dto::DownloadTicketResponseData;
use crate::modules::order::repo::map_db_error;
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DOWNLOAD_TICKET_AUDIT_EVENT: &str = "delivery.file.download";
const REDIS_DOWNLOAD_TICKET_DB: i64 = 3;
const REDIS_DOWNLOAD_TICKET_TTL_SECONDS: i64 = 300;

#[derive(Debug, Serialize, Deserialize)]
struct RedisDownloadTicketCachePayload {
    order_id: String,
    delivery_id: String,
    ticket_id: String,
    buyer_org_id: String,
    object_uri: String,
    bucket_name: String,
    object_key: String,
    envelope_id: String,
    delivery_commit_hash: String,
    ticket_status: String,
    expire_at: String,
    download_limit: i32,
    download_count: i32,
    remaining_downloads: i32,
    download_token: String,
    token_hash: String,
    issued_at: String,
}

pub async fn issue_download_ticket(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DownloadTicketResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT
               o.status,
               o.buyer_org_id::text,
               s.sku_type,
               dr.delivery_id::text,
               so.object_uri,
               ns.bucket_name,
               ke.envelope_id::text,
               dr.delivery_commit_hash,
               ticket.ticket_id::text,
               ticket.token_hash,
               to_char(ticket.expire_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ticket.download_limit,
               ticket.download_count,
               ticket.status,
               (ticket.expire_at > now()) AS ticket_not_expired,
               GREATEST(1, LEAST($2::int, FLOOR(EXTRACT(EPOCH FROM (ticket.expire_at - now())))::int))::bigint AS redis_ttl_seconds,
               to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             JOIN LATERAL (
               SELECT delivery_id, object_id, envelope_id, delivery_commit_hash
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'committed'
               ORDER BY committed_at DESC NULLS LAST, created_at DESC, delivery_id DESC
               LIMIT 1
             ) dr ON true
             JOIN delivery.storage_object so ON so.object_id = dr.object_id
             LEFT JOIN catalog.storage_namespace ns ON ns.storage_namespace_id = so.storage_namespace_id
             LEFT JOIN delivery.key_envelope ke ON ke.envelope_id = dr.envelope_id
             JOIN LATERAL (
               SELECT ticket_id, token_hash, expire_at, download_limit, download_count, status
               FROM delivery.delivery_ticket
               WHERE order_id = o.order_id
                 AND status = 'active'
               ORDER BY created_at DESC, ticket_id DESC
               LIMIT 1
             ) ticket ON true
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id, &REDIS_DOWNLOAD_TICKET_TTL_SECONDS],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    let current_state: String = row.get(0);
    let buyer_org_id: String = row.get(1);
    let sku_type: String = row.get(2);
    let delivery_id: String = row.get(3);
    let object_uri: String = row.get(4);
    let bucket_name = row
        .get::<_, Option<String>>(5)
        .unwrap_or_else(default_bucket_name);
    let envelope_id = row.get::<_, Option<String>>(6).ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: key envelope missing",
            request_id,
        )
    })?;
    let delivery_commit_hash = row.get::<_, Option<String>>(7).ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: delivery commit hash missing",
            request_id,
        )
    })?;
    let ticket_id: String = row.get(8);
    let previous_token_hash: String = row.get(9);
    let expire_at: String = row.get(10);
    let download_limit: i32 = row.get(11);
    let download_count: i32 = row.get(12);
    let ticket_status: String = row.get(13);
    let ticket_not_expired: bool = row.get(14);
    let redis_ttl_seconds: i64 = row.get(15);
    let issued_at: String = row.get(16);

    enforce_buyer_scope(actor_role, tenant_id, &buyer_org_id, request_id)?;

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
                "DOWNLOAD_TICKET_FORBIDDEN: current_state `{current_state}` does not allow ticket issuance"
            ),
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
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: download limit reached",
            request_id,
        ));
    }

    let resolved = resolve_storage_object_location(&object_uri, Some(bucket_name.as_str()));
    let object_key = resolved.object_key.clone().ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: object key cannot be resolved",
            request_id,
        )
    })?;

    let remaining_downloads = (download_limit - download_count).max(0);
    let download_token = format!(
        "dlt.{}.{}.{}",
        ticket_id,
        order_id,
        new_external_readable_id("tok")
    );
    let token_hash: String = tx
        .query_one("SELECT md5($1)", &[&download_token])
        .await
        .map_err(map_db_error)?
        .get(0);

    tx.execute(
        "UPDATE delivery.delivery_ticket
         SET token_hash = $2,
             status = 'active'
         WHERE ticket_id = $1::text::uuid",
        &[&ticket_id, &token_hash],
    )
    .await
    .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;

    let redis_key = redis_download_ticket_key(&ticket_id);
    let cache_payload = RedisDownloadTicketCachePayload {
        order_id: order_id.to_string(),
        delivery_id: delivery_id.clone(),
        ticket_id: ticket_id.clone(),
        buyer_org_id: buyer_org_id.clone(),
        object_uri: object_uri.clone(),
        bucket_name: bucket_name.clone(),
        object_key: object_key.clone(),
        envelope_id: envelope_id.clone(),
        delivery_commit_hash: delivery_commit_hash.clone(),
        ticket_status: ticket_status.clone(),
        expire_at: expire_at.clone(),
        download_limit,
        download_count,
        remaining_downloads,
        download_token: download_token.clone(),
        token_hash: token_hash.clone(),
        issued_at: issued_at.clone(),
    };

    let mut redis_written = false;
    let redis_result =
        set_download_ticket_cache(&redis_key, redis_ttl_seconds, &cache_payload).await;
    if redis_result.is_ok() {
        redis_written = true;
    }
    if let Err(err) = redis_result {
        restore_previous_ticket_hash(client, &ticket_id, &previous_token_hash).await;
        return Err(err);
    }

    let audit_result = write_download_ticket_audit_event(
        client,
        &ticket_id,
        actor_role,
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "delivery_id": delivery_id,
            "buyer_org_id": buyer_org_id,
            "bucket_name": bucket_name,
            "object_key": object_key,
            "download_limit": download_limit,
            "download_count": download_count,
            "remaining_downloads": remaining_downloads,
            "redis_key": redis_key,
        }),
    )
    .await;
    if let Err(err) = audit_result {
        restore_previous_ticket_hash(client, &ticket_id, &previous_token_hash).await;
        if redis_written {
            delete_download_ticket_cache(&redis_key).await;
        }
        return Err(err);
    }

    Ok(DownloadTicketResponseData {
        order_id: order_id.to_string(),
        delivery_id,
        ticket_id,
        download_token,
        ticket_status,
        issued_at,
        expire_at,
        download_limit,
        download_count,
        remaining_downloads,
        bucket_name,
        object_key,
        envelope_id,
        delivery_commit_hash,
    })
}

async fn set_download_ticket_cache(
    redis_key: &str,
    ttl_seconds: i64,
    payload: &RedisDownloadTicketCachePayload,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let redis_url = redis_download_ticket_url();
    let client = redis::Client::open(redis_url.as_str()).map_err(map_redis_error)?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(map_redis_error)?;
    let serialized = serde_json::to_string(payload).map_err(map_redis_error)?;
    connection
        .set_ex::<_, _, ()>(redis_key, serialized, ttl_seconds as u64)
        .await
        .map_err(map_redis_error)?;
    Ok(())
}

async fn delete_download_ticket_cache(redis_key: &str) {
    let redis_url = redis_download_ticket_url();
    let Ok(client) = redis::Client::open(redis_url.as_str()) else {
        return;
    };
    let Ok(mut connection) = client.get_multiplexed_async_connection().await else {
        return;
    };
    let _: Result<(), _> = connection.del(redis_key).await;
}

async fn restore_previous_ticket_hash(client: &Client, ticket_id: &str, previous_token_hash: &str) {
    let _ = client
        .execute(
            "UPDATE delivery.delivery_ticket
             SET token_hash = $2,
                 status = 'active'
             WHERE ticket_id = $1::text::uuid",
            &[&ticket_id, &previous_token_hash],
        )
        .await;
}

fn redis_download_ticket_key(ticket_id: &str) -> String {
    let namespace = std::env::var("REDIS_NAMESPACE").unwrap_or_else(|_| "datab:v1".to_string());
    format!("{namespace}:download-ticket:{ticket_id}")
}

fn redis_download_ticket_url() -> String {
    if let Ok(url) = std::env::var("REDIS_URL") {
        if !url.trim().is_empty() {
            return url;
        }
    }
    let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let password =
        std::env::var("REDIS_PASSWORD").unwrap_or_else(|_| "datab_redis_pass".to_string());
    format!(
        "redis://:{}@{}:{}/{REDIS_DOWNLOAD_TICKET_DB}",
        password, host, port
    )
}

fn enforce_buyer_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(actor_role, "tenant_admin" | "buyer_operator") && tenant_id == Some(buyer_org_id) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "download ticket issuance is forbidden for tenant scope".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

async fn write_download_ticket_audit_event(
    client: &(impl GenericClient + Sync),
    ticket_id: &str,
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
               'delivery_ticket',
               $1::text::uuid,
               'role',
               $2,
               'ticket_issued',
               $3,
               $4,
               $5::jsonb || jsonb_build_object('actor_role', $6)
             )",
            &[
                &ticket_id,
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

fn map_redis_error(err: impl std::fmt::Display) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("download ticket cache failed: {err}"),
            request_id: None,
        }),
    )
}

fn default_bucket_name() -> String {
    std::env::var("BUCKET_DELIVERY_OBJECTS").unwrap_or_else(|_| "delivery-objects".to_string())
}

fn not_found(order_id: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("order not found: {order_id}"),
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
