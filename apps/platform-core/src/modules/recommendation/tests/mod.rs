mod recommendation_api_db;

#[cfg(test)]
mod route_tests {
    use crate::modules::recommendation::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

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
