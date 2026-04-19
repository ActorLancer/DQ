mod handlers;

use crate::AppState;
use axum::Router;
use axum::routing::post;
pub use handlers::commit_order_delivery_api;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/orders/{id}/deliver",
        post(commit_order_delivery_api),
    )
}
