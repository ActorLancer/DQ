mod download_middleware;
mod handlers;
mod support;

use crate::AppState;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use download_middleware::validate_download_ticket_middleware;
pub use handlers::{
    commit_order_delivery_api, download_file_api, execute_template_run_api, get_api_usage_log_api,
    get_query_runs_api, get_revision_subscription_api, get_share_grants_api,
    issue_download_ticket_api, manage_query_surface_api, manage_query_template_api,
    manage_revision_subscription_api, manage_share_grant_api, manage_template_grant_api,
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
        .route(
            "/api/v1/orders/{id}/template-grants",
            post(manage_template_grant_api),
        )
        .route(
            "/api/v1/orders/{id}/template-runs",
            post(execute_template_run_api).get(get_query_runs_api),
        )
        .route("/api/v1/orders/{id}/usage-log", get(get_api_usage_log_api))
        .route(
            "/api/v1/products/{id}/query-surfaces",
            post(manage_query_surface_api),
        )
        .route(
            "/api/v1/query-surfaces/{id}/templates",
            post(manage_query_template_api),
        )
}
