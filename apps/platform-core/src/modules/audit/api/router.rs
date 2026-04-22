use axum::Router;
use axum::routing::get;

use crate::AppState;

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/audit/orders/{id}",
            get(handlers::get_order_audit_traces),
        )
        .route("/api/v1/audit/traces", get(handlers::get_audit_traces))
}
