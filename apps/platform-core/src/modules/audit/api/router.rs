use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/audit/orders/{id}",
            get(handlers::get_order_audit_traces),
        )
        .route("/api/v1/audit/traces", get(handlers::get_audit_traces))
        .route(
            "/api/v1/audit/packages/export",
            post(handlers::export_audit_package),
        )
        .route(
            "/api/v1/audit/replay-jobs",
            post(handlers::create_audit_replay_job),
        )
        .route(
            "/api/v1/audit/replay-jobs/{id}",
            get(handlers::get_audit_replay_job),
        )
        .route(
            "/api/v1/audit/legal-holds",
            post(handlers::create_audit_legal_hold),
        )
        .route(
            "/api/v1/audit/legal-holds/{id}/release",
            post(handlers::release_audit_legal_hold),
        )
        .route(
            "/api/v1/audit/anchor-batches",
            get(handlers::get_audit_anchor_batches),
        )
        .route(
            "/api/v1/audit/anchor-batches/{id}/retry",
            post(handlers::retry_audit_anchor_batch),
        )
        .route("/api/v1/ops/outbox", get(handlers::get_ops_outbox))
        .route(
            "/api/v1/ops/dead-letters",
            get(handlers::get_ops_dead_letters),
        )
        .route(
            "/api/v1/ops/external-facts",
            get(handlers::get_ops_external_facts),
        )
        .route(
            "/api/v1/ops/fairness-incidents",
            get(handlers::get_ops_fairness_incidents),
        )
        .route(
            "/api/v1/ops/consistency/{refType}/{refId}",
            get(handlers::get_ops_consistency),
        )
        .route(
            "/api/v1/ops/consistency/reconcile",
            post(handlers::reconcile_ops_consistency),
        )
        .route(
            "/api/v1/ops/trade-monitor/orders/{orderId}",
            get(handlers::get_ops_trade_monitor_overview),
        )
        .route(
            "/api/v1/ops/trade-monitor/orders/{orderId}/checkpoints",
            get(handlers::get_ops_trade_monitor_checkpoints),
        )
        .route(
            "/api/v1/ops/external-facts/{id}/confirm",
            post(handlers::confirm_ops_external_fact),
        )
        .route(
            "/api/v1/ops/fairness-incidents/{id}/handle",
            post(handlers::handle_ops_fairness_incident),
        )
        .route(
            "/api/v1/ops/dead-letters/{id}/reprocess",
            post(handlers::reprocess_ops_dead_letter),
        )
}
