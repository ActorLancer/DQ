mod handlers;

use axum::Router;
use axum::routing::{get, post};
pub use handlers::{create_trade_pre_request, get_trade_pre_request};

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/trade/pre-requests", post(create_trade_pre_request))
        .route(
            "/api/v1/trade/pre-requests/{id}",
            get(get_trade_pre_request),
        )
}
