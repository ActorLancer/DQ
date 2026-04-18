mod trade001_pre_request_db;
mod trade002_price_snapshot_db;
mod trade003_create_order_db;
mod trade004_order_detail_db;
mod trade005_order_cancel_db;
mod trade006_contract_confirm_db;
mod trade007_state_machine_fields_db;
mod trade008_file_std_state_machine_db;

#[cfg(test)]
mod tests {
    use super::super::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn rejects_trade_pre_request_create_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/trade/pre-requests")
                    .header("x-role", "developer")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                          "buyer_org_id":"10000000-0000-0000-0000-000000000102",
                          "product_id":"20000000-0000-0000-0000-000000000301",
                          "request_kind":"rfq",
                          "details":{"title":"need quote"}
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_trade_price_snapshot_freeze_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/trade/orders/30000000-0000-0000-0000-000000000101/price-snapshot/freeze")
                    .header("x-role", "developer")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_order_create_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("x-role", "developer")
                    .header("x-tenant-id", "10000000-0000-0000-0000-000000000102")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                          "buyer_org_id":"10000000-0000-0000-0000-000000000102",
                          "product_id":"20000000-0000-0000-0000-000000000301",
                          "sku_id":"21000000-0000-0000-0000-000000000401"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_order_detail_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101")
                    .header("x-role", "developer")
                    .header("x-tenant-id", "10000000-0000-0000-0000-000000000102")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_order_cancel_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101/cancel")
                    .header("x-role", "developer")
                    .header("x-tenant-id", "10000000-0000-0000-0000-000000000102")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_order_contract_confirm_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101/contract-confirm")
                    .header("x-role", "developer")
                    .header("x-tenant-id", "10000000-0000-0000-0000-000000000102")
                    .header("x-user-id", "10000000-0000-0000-0000-000000000999")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                          "contract_template_id":"20000000-0000-0000-0000-000000000501",
                          "contract_digest":"sha256:test",
                          "variables_json":{"term_days":30},
                          "signer_role":"buyer_operator"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_file_std_transition_without_permission() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders/30000000-0000-0000-0000-000000000101/file-std/transition")
                    .header("x-role", "developer")
                    .header("x-tenant-id", "10000000-0000-0000-0000-000000000102")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                          "action":"lock_funds"
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
