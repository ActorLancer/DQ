mod download_middleware;
mod handlers;
mod support;

use crate::AppState;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use download_middleware::validate_download_ticket_middleware;
pub use handlers::{commit_order_delivery_api, download_file_api, issue_download_ticket_api};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/orders/{id}/deliver",
            post(commit_order_delivery_api),
        )
        .route(
            "/api/v1/orders/{id}/download-ticket",
            get(issue_download_ticket_api),
        )
        .route(
            "/api/v1/orders/{id}/download",
            get(download_file_api)
                .route_layer(middleware::from_fn(validate_download_ticket_middleware)),
        )
}
