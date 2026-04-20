use crate::AppState;
use crate::modules::billing::handlers::handle_payment_webhook;
use crate::modules::billing::order_lock_handlers::lock_order_payment;
use crate::modules::billing::payment_intent_handlers::{
    cancel_payment_intent, create_payment_intent, get_payment_intent,
};
use crate::modules::billing::policy_handlers::{
    create_payment_corridor, create_payment_jurisdiction, create_payout_preference,
    get_payment_corridors, get_payment_jurisdictions, list_payout_preferences_v1,
};
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/payment-jurisdictions",
            get(get_payment_jurisdictions).post(create_payment_jurisdiction),
        )
        .route(
            "/api/v1/payment-corridors",
            get(get_payment_corridors).post(create_payment_corridor),
        )
        .route(
            "/api/v1/payout-preferences",
            get(list_payout_preferences_v1).post(create_payout_preference),
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
