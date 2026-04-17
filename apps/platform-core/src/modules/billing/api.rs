use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile, PayoutPreference};
use crate::modules::billing::service::{
    BillingPermission, is_allowed, list_corridor_policies, list_jurisdictions,
    list_payout_preferences,
};
use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use serde::Serialize;
use tracing::info;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BillingPolicyView {
    pub jurisdictions: Vec<JurisdictionProfile>,
    pub corridor_policies: Vec<CorridorPolicy>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/billing/policies", get(get_billing_policies))
        .route(
            "/api/v1/billing/payout-preferences/{beneficiary_subject_id}",
            get(get_payout_preferences),
        )
        .layer(middleware::from_fn(read_policy_permission_guard))
}

async fn get_billing_policies() -> Json<ApiResponse<BillingPolicyView>> {
    info!(
        action = "billing.policy.read",
        "billing policy placeholder served"
    );
    ApiResponse::ok(BillingPolicyView {
        jurisdictions: list_jurisdictions(),
        corridor_policies: list_corridor_policies(),
    })
}

async fn get_payout_preferences(
    Path(beneficiary_subject_id): Path<String>,
) -> Json<ApiResponse<Vec<PayoutPreference>>> {
    info!(
        action = "billing.payout_preference.read",
        beneficiary_subject_id = %beneficiary_subject_id,
        "billing payout preference placeholder served"
    );
    ApiResponse::ok(list_payout_preferences(&beneficiary_subject_id))
}

async fn read_policy_permission_guard(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let role = request
        .headers()
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !is_allowed(role, BillingPermission::ReadPolicy) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "billing policy read is forbidden for current role".to_string(),
                request_id: request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .map(ToOwned::to_owned),
            }),
        ));
    }
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn rejects_request_without_role() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/billing/policies")
                    .method("GET")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn returns_policy_view_for_allowed_role() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/billing/policies")
                    .method("GET")
                    .header("x-role", "platform_admin")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
