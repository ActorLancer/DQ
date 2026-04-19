use super::download_middleware::ValidatedDownloadTicket;
use super::support::{DeliveryPermission, header, map_db_connect, require_permission};
use crate::AppState;
use crate::modules::delivery::dto::{
    CommitOrderDeliveryRequest, CommitOrderDeliveryResponse, DownloadFileResponse,
    DownloadFileResponseData, DownloadTicketResponse,
};
use crate::modules::delivery::repo::{
    commit_file_delivery, consume_download_ticket, issue_download_ticket,
};
use crate::modules::storage::application::fetch_object_bytes;
use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use base64::Engine;
use http::ApiResponse;
use kernel::ErrorResponse;

pub async fn commit_order_delivery_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CommitOrderDeliveryRequest>,
) -> Result<Json<ApiResponse<CommitOrderDeliveryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::CommitFileDelivery,
        "file delivery commit",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let committed = commit_file_delivery(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(CommitOrderDeliveryResponse {
        data: committed,
    }))
}

pub async fn issue_download_ticket_api(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<DownloadTicketResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        DeliveryPermission::IssueDownloadTicket,
        "download ticket issuance",
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let tenant_id = header(&headers, "x-tenant-id");
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");

    let mut client = state.db.client().map_err(map_db_connect)?;
    let ticket = issue_download_ticket(
        &mut client,
        &order_id,
        tenant_id.as_deref(),
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    Ok(ApiResponse::ok(DownloadTicketResponse { data: ticket }))
}

pub async fn download_file_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(validated): Extension<ValidatedDownloadTicket>,
) -> Result<Json<ApiResponse<DownloadFileResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let client_fingerprint = header(&headers, "x-client-fingerprint");
    let source_ip = header(&headers, "x-forwarded-for")
        .or_else(|| header(&headers, "x-real-ip"))
        .map(|value| {
            value
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .filter(|value| !value.is_empty());

    let fetched_object = fetch_object_bytes(
        &validated.cache_payload.bucket_name,
        &validated.cache_payload.object_key,
    )
    .await?;

    let mut client = state.db.client().map_err(map_db_connect)?;
    let access = consume_download_ticket(
        &mut client,
        &validated.cache_payload,
        &actor_role,
        &validated.raw_token,
        client_fingerprint.as_deref(),
        source_ip.as_deref(),
        request_id.as_deref(),
        trace_id.as_deref(),
    )
    .await?;

    let response = DownloadFileResponseData {
        order_id: access.order_id,
        delivery_id: access.delivery_id,
        ticket_id: access.ticket_id,
        receipt_id: access.receipt_id,
        receipt_hash: access.receipt_hash,
        downloaded_at: access.downloaded_at,
        ticket_status: access.ticket_status,
        download_limit: access.download_limit,
        download_count: access.download_count,
        remaining_downloads: access.remaining_downloads,
        bucket_name: access.bucket_name,
        object_key: access.object_key,
        content_type: access.content_type.or(fetched_object.content_type),
        content_hash: access.content_hash,
        delivery_commit_hash: access.delivery_commit_hash,
        key_envelope: access.key_envelope,
        object_base64: base64::engine::general_purpose::STANDARD.encode(&fetched_object.bytes),
    };

    Ok(ApiResponse::ok(DownloadFileResponse { data: response }))
}
