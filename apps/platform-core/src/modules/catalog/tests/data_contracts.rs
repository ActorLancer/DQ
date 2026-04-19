use crate::modules::catalog::router::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_create_data_contract_when_sku_id_mismatch() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001/data-contracts")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(
            r#"{
              "sku_id":"00000000-0000-0000-0000-000000000099",
              "contract_name":"Contract CAT013"
            }"#,
        ))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
