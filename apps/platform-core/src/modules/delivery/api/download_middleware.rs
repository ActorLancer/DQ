use super::support::{DeliveryPermission, header, require_permission};
use crate::modules::delivery::repo::{
    DownloadTicketCachePayload, enforce_buyer_scope, load_download_ticket_cache,
    parse_download_token,
};
use axum::Json;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use kernel::ErrorResponse;

#[derive(Debug, Clone)]
pub struct ValidatedDownloadTicket {
    pub raw_token: String,
    pub cache_payload: DownloadTicketCachePayload,
}

pub async fn validate_download_ticket_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let headers = request.headers().clone();
    require_permission(&headers, DeliveryPermission::DownloadFile, "file download")?;

    let request_id = header(&headers, "x-request-id");
    let tenant_id = header(&headers, "x-tenant-id");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let raw_token = extract_download_token(request.uri(), &headers).ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket is required",
            request_id.as_deref(),
        )
    })?;
    let parsed = parse_download_token(&raw_token).ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket format is invalid",
            request_id.as_deref(),
        )
    })?;
    let cache_payload = load_download_ticket_cache(&parsed.ticket_id)
        .await?
        .ok_or_else(|| {
            conflict(
                "DOWNLOAD_TICKET_FORBIDDEN: ticket cache not found or expired",
                request_id.as_deref(),
            )
        })?;

    if cache_payload.order_id != parsed.order_id {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket order mismatch",
            request_id.as_deref(),
        ));
    }
    if cache_payload.download_token != raw_token {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket token mismatch",
            request_id.as_deref(),
        ));
    }
    if cache_payload.ticket_status != "active" || cache_payload.remaining_downloads <= 0 {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: ticket is not active",
            request_id.as_deref(),
        ));
    }
    let request_order_id = extract_order_id_from_path(request.uri().path()).ok_or_else(|| {
        conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: request path is invalid",
            request_id.as_deref(),
        )
    })?;
    if request_order_id != cache_payload.order_id {
        return Err(conflict(
            "DOWNLOAD_TICKET_FORBIDDEN: request order does not match ticket",
            request_id.as_deref(),
        ));
    }
    enforce_buyer_scope(
        &actor_role,
        tenant_id.as_deref(),
        &cache_payload.buyer_org_id,
        request_id.as_deref(),
    )?;

    request.extensions_mut().insert(ValidatedDownloadTicket {
        raw_token,
        cache_payload,
    });
    Ok(next.run(request).await)
}

fn extract_download_token(uri: &axum::http::Uri, headers: &HeaderMap) -> Option<String> {
    query_param(uri, "ticket").or_else(|| header(headers, "x-download-ticket"))
}

fn query_param(uri: &axum::http::Uri, key: &str) -> Option<String> {
    uri.query()?.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == key && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn extract_order_id_from_path(path: &str) -> Option<String> {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 6 {
        return None;
    }
    if parts[1] != "api" || parts[2] != "v1" || parts[3] != "orders" {
        return None;
    }
    Some(parts[4].to_string())
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
