use crate::modules::catalog::router::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_create_asset_processing_job_without_input_sources() {
    let app = router();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/processing-jobs")
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(Body::from(
            r#"{
              "processing_mode":"platform_managed",
              "input_sources":[],
              "processing_summary_json":{"strategy":"baseline_v1"}
            }"#,
        ))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
