use base64::Engine;

mod recommendation_api_db;

pub(super) fn authorization_header(user_id: &str, tenant_id: &str, roles: &[&str]) -> String {
    let header =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        serde_json::json!({
            "sub": user_id,
            "tenant_id": tenant_id,
            "realm_access": {
                "roles": roles,
            },
        })
        .to_string(),
    );
    format!("Bearer {header}.{payload}.sig")
}

#[cfg(test)]
mod route_tests {
    use super::authorization_header;
    use crate::modules::recommendation::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_recommendation_read_without_bearer() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/recommendations?placement_code=home_featured")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_recommendation_read_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/recommendations?placement_code=home_featured")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["developer"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_recommendation_read_with_invalid_query() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/recommendations?placement_code=home_featured&subject_scope=tenant")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["buyer_operator"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_recommendation_ops_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/recommendation/placements")
            .header("x-role", "buyer_operator")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn recommendation_write_requires_idempotency_key() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/recommendations/track/click")
            .header("content-type", "application/json")
            .header("x-role", "buyer_operator")
            .body(Body::from(
                r#"{
                  "recommendation_request_id":"00000000-0000-0000-0000-000000000000",
                  "recommendation_result_id":"00000000-0000-0000-0000-000000000000",
                  "recommendation_result_item_id":"00000000-0000-0000-0000-000000000000",
                  "entity_scope":"product",
                  "entity_id":"00000000-0000-0000-0000-000000000000"
                }"#,
            ))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
