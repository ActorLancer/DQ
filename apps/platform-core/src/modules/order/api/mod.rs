mod handlers;

use axum::Router;
use axum::routing::{get, post};
pub use handlers::{
    cancel_order_api, confirm_order_contract_api, create_order_api, create_trade_pre_request,
    freeze_order_price_snapshot_api, get_order_detail_api, get_trade_pre_request,
    transition_api_sub_order_api, transition_file_std_order_api, transition_file_sub_order_api,
};

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/orders", post(create_order_api))
        .route("/api/v1/orders/{id}", get(get_order_detail_api))
        .route("/api/v1/orders/{id}/cancel", post(cancel_order_api))
        .route(
            "/api/v1/orders/{id}/contract-confirm",
            post(confirm_order_contract_api),
        )
        .route(
            "/api/v1/orders/{id}/file-std/transition",
            post(transition_file_std_order_api),
        )
        .route(
            "/api/v1/orders/{id}/file-sub/transition",
            post(transition_file_sub_order_api),
        )
        .route(
            "/api/v1/orders/{id}/api-sub/transition",
            post(transition_api_sub_order_api),
        )
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
