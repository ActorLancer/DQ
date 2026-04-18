use crate::modules::catalog::api::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_suspend_product_with_invalid_suspend_mode() {
    let app = router();
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
    let app = router();
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
