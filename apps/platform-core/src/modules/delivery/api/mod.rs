mod download_middleware;
mod handlers;
mod support;

use crate::AppState;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use download_middleware::validate_download_ticket_middleware;
pub use handlers::{
    commit_order_delivery_api, download_file_api, get_revision_subscription_api,
    get_share_grants_api, issue_download_ticket_api, manage_revision_subscription_api,
    manage_share_grant_api,
};

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
        .route(
            "/api/v1/orders/{id}/subscriptions",
            post(manage_revision_subscription_api).get(get_revision_subscription_api),
        )
        .route(
            "/api/v1/orders/{id}/share-grants",
            post(manage_share_grant_api).get(get_share_grants_api),
        )
}
