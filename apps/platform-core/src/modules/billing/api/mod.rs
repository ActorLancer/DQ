use crate::modules::billing::handlers::{
    cancel_payment_intent, create_payment_intent, get_billing_policies, get_payment_intent,
    get_payout_preferences, handle_payment_webhook, lock_order_payment,
};
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/billing/policies", get(get_billing_policies))
        .route(
            "/api/v1/billing/payout-preferences/{beneficiary_subject_id}",
            get(get_payout_preferences),
        )
        .route("/api/v1/payments/intents", post(create_payment_intent))
        .route("/api/v1/payments/intents/{id}", get(get_payment_intent))
        .route(
            "/api/v1/payments/intents/{id}/cancel",
            post(cancel_payment_intent),
        )
        .route(
            "/api/v1/payments/webhooks/{provider}",
            post(handle_payment_webhook),
        )
        .route("/api/v1/orders/{id}/lock", post(lock_order_payment))
}
