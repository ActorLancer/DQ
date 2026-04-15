use axum::{Json, Router, http::StatusCode, routing::get};
use kernel::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize)]
pub enum ErrorCode {
    Internal,
    Validation,
    Unauthorized,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

pub fn build_router() -> Router {
    Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
}

pub async fn live_handler() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("ok")
}

pub async fn ready_handler() -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(ApiResponse::ok("ready"))
}

pub async fn serve(
    addr: SocketAddr,
    app: Router,
    shutdown: impl Future<Output = Result<(), std::io::Error>> + Send + 'static,
) -> AppResult<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Startup(format!("bind listener failed: {e}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(|e| AppError::Shutdown(format!("http server stopped with error: {e}")))?;
    Ok(())
}
