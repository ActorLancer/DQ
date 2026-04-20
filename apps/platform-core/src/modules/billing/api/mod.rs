use crate::AppState;
use crate::modules::billing::billing_read_handlers::get_billing_order;
use crate::modules::billing::compensation_handlers::create_compensation;
use crate::modules::billing::dispute_handlers::{
    create_dispute_case, resolve_dispute_case, upload_dispute_evidence,
};
use crate::modules::billing::mock_payment_handlers::{
    simulate_payment_fail, simulate_payment_success, simulate_payment_timeout,
};
use crate::modules::billing::order_lock_handlers::lock_order_payment;
use crate::modules::billing::payment_intent_handlers::{
    cancel_payment_intent, create_payment_intent, get_payment_intent,
};
use crate::modules::billing::payment_result_handlers::process_payment_polled_result;
use crate::modules::billing::payout_handlers::create_manual_payout;
use crate::modules::billing::policy_handlers::{
    create_payment_corridor, create_payment_jurisdiction, create_payout_preference,
    get_payment_corridors, get_payment_jurisdictions, list_payout_preferences_v1,
};
use crate::modules::billing::reconciliation_handlers::import_reconciliation_statement;
use crate::modules::billing::refund_handlers::create_refund;
use crate::modules::billing::webhook_handlers::handle_payment_webhook;
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
            "/api/v1/payments/intents/{id}/poll-result",
            post(process_payment_polled_result),
        )
        .route(
            "/api/v1/payments/reconciliation/import",
            post(import_reconciliation_statement),
        )
        .route("/api/v1/cases", post(create_dispute_case))
        .route("/api/v1/cases/{id}/evidence", post(upload_dispute_evidence))
        .route("/api/v1/cases/{id}/resolve", post(resolve_dispute_case))
        .route("/api/v1/billing/{order_id}", get(get_billing_order))
        .route("/api/v1/payouts/manual", post(create_manual_payout))
        .route("/api/v1/compensations", post(create_compensation))
        .route("/api/v1/refunds", post(create_refund))
        .route(
            "/api/v1/payments/intents/{id}/cancel",
            post(cancel_payment_intent),
        )
        .route(
            "/api/v1/payments/webhooks/{provider}",
            post(handle_payment_webhook),
        )
        .route(
            "/api/v1/mock/payments/{id}/simulate-success",
            post(simulate_payment_success),
        )
        .route(
            "/api/v1/mock/payments/{id}/simulate-fail",
            post(simulate_payment_fail),
        )
        .route(
            "/api/v1/mock/payments/{id}/simulate-timeout",
            post(simulate_payment_timeout),
        )
        .route("/api/v1/orders/{id}/lock", post(lock_order_payment))
}
