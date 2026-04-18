use axum::Json;
use axum::http::StatusCode;
use kernel::ErrorResponse;

/// Shared API error type for catalog handlers.
pub type ApiError = (StatusCode, Json<ErrorResponse>);
