mod handlers;

use axum::Router;
use axum::routing::{get, post};
pub use handlers::{
    cancel_order_api, create_order_api, create_trade_pre_request, freeze_order_price_snapshot_api,
    get_order_detail_api, get_trade_pre_request,
};

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/orders", post(create_order_api))
        .route("/api/v1/orders/{id}", get(get_order_detail_api))
        .route("/api/v1/orders/{id}/cancel", post(cancel_order_api))
        .route("/api/v1/trade/pre-requests", post(create_trade_pre_request))
        .route(
            "/api/v1/trade/pre-requests/{id}",
            get(get_trade_pre_request),
        )
        .route(
            "/api/v1/trade/orders/{id}/price-snapshot/freeze",
            post(freeze_order_price_snapshot_api),
        )
}
