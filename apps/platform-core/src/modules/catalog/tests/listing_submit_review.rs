use crate::modules::catalog::router::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_submit_product_with_empty_submission_note() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/submit")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(r#"{"submission_note":"   "}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_review_subject_with_invalid_action_name() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/review/subjects/00000000-0000-0000-0000-000000000001")
        .header("content-type", "application/json")
        .header("x-role", "platform_reviewer")
        .body(Body::from(r#"{"action_name":"pass"}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_review_compliance_with_empty_reason() {
    let app = crate::with_stub_test_state(router());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/review/compliance/00000000-0000-0000-0000-000000000001")
        .header("content-type", "application/json")
        .header("x-role", "platform_reviewer")
        .body(Body::from(
            r#"{"action_name":"approve","action_reason":"  "}"#,
        ))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
