mod search_api_db;

#[cfg(test)]
mod route_tests {
    use crate::modules::search::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_catalog_search_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/search?q=test")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_search_ops_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/search/sync")
            .header("x-role", "buyer_operator")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn search_write_requires_idempotency_key() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/ops/search/cache/invalidate")
            .header("content-type", "application/json")
            .header("x-role", "platform_admin")
            .body(Body::from(r#"{"entity_scope":"product"}"#))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
