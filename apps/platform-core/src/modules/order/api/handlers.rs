use crate::modules::order::dto::{
    CreateTradePreRequestRequest, TradePreRequestResponse, TradePreRequestResponseData,
};
use crate::modules::order::repo::{
    insert_trade_pre_request, load_trade_pre_request, write_trade_audit_event,
};
use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::NoTls;
use tracing::info;

pub async fn create_trade_pre_request(
    headers: HeaderMap,
    Json(payload): Json<CreateTradePreRequestRequest>,
) -> Result<Json<ApiResponse<TradePreRequestResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::CreatePreRequest,
        "trade pre-request create",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let created = insert_trade_pre_request(&client, &payload).await?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        &created.inquiry_id,
        &actor_role,
        "trade.pre_request.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.pre_request.create",
        inquiry_id = %created.inquiry_id,
        request_kind = %created.request_payload.request_kind.as_str(),
        "trade pre-request created"
    );

    Ok(ApiResponse::ok(TradePreRequestResponse {
        data: TradePreRequestResponseData::from(created),
    }))
}

pub async fn get_trade_pre_request(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TradePreRequestResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::ReadPreRequest,
        "trade pre-request read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(found) = load_trade_pre_request(&client, &id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("trade pre-request not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        &found.inquiry_id,
        &actor_role,
        "trade.pre_request.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.pre_request.read",
        inquiry_id = %found.inquiry_id,
        "trade pre-request queried"
    );

    Ok(ApiResponse::ok(TradePreRequestResponse {
        data: TradePreRequestResponseData::from(found),
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TradePermission {
    CreatePreRequest,
    ReadPreRequest,
}

fn is_allowed(role: &str, permission: TradePermission) -> bool {
    match permission {
        TradePermission::CreatePreRequest => matches!(role, "buyer_operator" | "tenant_admin"),
        TradePermission::ReadPreRequest => matches!(
            role,
            "buyer_operator" | "seller_operator" | "tenant_admin" | "auditor"
        ),
    }
}

fn require_permission(
    headers: &HeaderMap,
    permission: TradePermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = headers
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if is_allowed(role, permission) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for current role"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn database_dsn() -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    std::env::var("DATABASE_URL").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: ErrorCode::OpsInternal.as_str().to_string(),
                message: "DATABASE_URL is not configured".to_string(),
                request_id: None,
            }),
        )
    })
}

fn map_db_connect(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}
