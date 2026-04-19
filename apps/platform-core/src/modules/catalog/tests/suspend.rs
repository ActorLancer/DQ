use crate::modules::catalog::router::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_suspend_product_with_invalid_suspend_mode() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/suspend")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(r#"{"suspend_mode":"pause"}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_suspend_product_with_empty_reason() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/suspend")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(r#"{"suspend_mode":"freeze","reason":"  "}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_freeze_without_step_up_header() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/suspend")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(r#"{"suspend_mode":"freeze","reason":"risk"}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
