use crate::modules::catalog::router::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_patch_release_policy_with_invalid_release_mode() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/release-policy")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(
            r#"{
              "release_mode":"rolling"
            }"#,
        ))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
