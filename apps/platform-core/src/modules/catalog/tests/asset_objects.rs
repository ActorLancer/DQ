use crate::modules::catalog::api::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_create_asset_object_with_invalid_object_kind() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/objects")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(
            r#"{
              "object_kind":"unknown_kind",
              "object_name":"delivery-package",
              "object_uri":"s3://product/delivery-package.zip"
            }"#,
        ))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
