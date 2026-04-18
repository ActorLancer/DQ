mod asset_objects;
mod cat020_read_db;
mod cat021_template_policy_db;
mod cat022_search_visibility_db;
mod data_contracts;
mod listing_submit_review;
mod processing_jobs;
mod release_policy;
mod suspend;
mod template_policy_db;

#[cfg(test)]
mod tests {
    use super::super::domain::{
        STANDARD_SKU_TYPES, default_trade_mode_for_sku_type, is_standard_sku_type,
        is_trade_mode_compatible_with_sku,
    };
    use super::super::router as catalog_router;
    use super::super::router::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn standard_sku_truth_list_matches_v1_frozen_set() {
        assert_eq!(STANDARD_SKU_TYPES.len(), 8);
        assert!(is_standard_sku_type("FILE_STD"));
        assert!(is_standard_sku_type("RPT_STD"));
        assert!(!is_standard_sku_type("FILE_PREMIUM"));
    }

    #[test]
    fn sku_trade_mode_mapping_is_frozen() {
        assert_eq!(
            default_trade_mode_for_sku_type("FILE_SUB"),
            Some("revision_subscription")
        );
        assert!(is_trade_mode_compatible_with_sku(
            "QRY_LITE",
            "template_query"
        ));
        assert!(!is_trade_mode_compatible_with_sku(
            "API_PPU",
            "api_subscription"
        ));
    }

    #[tokio::test]
    async fn rejects_create_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "asset_id":"00000000-0000-0000-0000-000000000001",
                  "asset_version_id":"00000000-0000-0000-0000-000000000002",
                  "seller_org_id":"00000000-0000-0000-0000-000000000003",
                  "title":"p",
                  "category":"c",
                  "product_type":"data_product",
                  "delivery_type":"file_download"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"title":"updated"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_get_product_detail_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn api_router_wrapper_matches_catalog_router() {
        let app_from_api = router();
        let app_from_catalog_router = catalog_router::router();

        let req1 = Request::builder()
            .method("GET")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let resp1 = app_from_api.oneshot(req1).await.expect("response");

        let req2 = Request::builder()
            .method("GET")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let resp2 = app_from_catalog_router
            .oneshot(req2)
            .await
            .expect("response");

        assert_eq!(resp1.status(), resp2.status());
    }

    #[tokio::test]
    async fn returns_standard_scenario_templates_for_read_role() {
        let app = router();
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/standard-scenarios")
            .header("x-role", "tenant_admin")
            .body(Body::empty())
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_get_seller_profile_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/sellers/00000000-0000-0000-0000-000000000100/profile")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_put_product_metadata_profile_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PUT")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100/metadata-profile")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "business_description_json": {"product_name":"demo"},
                  "data_content_json": {"object_type":"table"},
                  "structure_description_json": {"schema_version":"v1"},
                  "quality_description_json": {"missing_rate":0.01},
                  "compliance_description_json": {"contains_pi":false},
                  "delivery_description_json": {"delivery_modes":["file"]},
                  "version_description_json": {"version_no":1},
                  "authorization_description_json": {"license_scope":"internal"},
                  "responsibility_description_json": {"owner":"seller"},
                  "processing_overview_json": {"processing_mode":"seller_self_managed"}
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_sku_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/skus")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "sku_code":"SKU-001",
                  "sku_type":"FILE_STD",
                  "billing_mode":"one_time",
                  "acceptance_mode":"manual_accept",
                  "refund_mode":"manual_refund"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_sku_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"trade_mode":"snapshot_sale"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_bind_product_template_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/bind-template")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{"template_id":"00000000-0000-0000-0000-000000000010"}"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_bind_sku_template_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001/bind-template")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{"template_id":"00000000-0000-0000-0000-000000000010"}"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_usage_policy_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/policies/00000000-0000-0000-0000-000000000001")
            .header("content-type", "application/json")
            .header("x-role", "tenant_operator")
            .body(Body::from(r#"{"status":"active"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_raw_ingest_batch_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/raw-ingest-batches")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "owner_org_id":"00000000-0000-0000-0000-000000000003",
                  "ingest_source_type":"seller_upload"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_raw_object_manifest_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/raw-ingest-batches/00000000-0000-0000-0000-000000000001/manifests")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "object_name":"manifest-1.csv",
                  "object_uri":"s3://raw/manifest-1.csv"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_detect_format_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/raw-object-manifests/00000000-0000-0000-0000-000000000001/detect-format")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "detected_object_family":"tabular",
                  "classification_confidence":0.92
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_extraction_job_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri(
                "/api/v1/raw-object-manifests/00000000-0000-0000-0000-000000000001/extraction-jobs",
            )
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "job_type":"schema_extract",
                  "job_config_json":{"mode":"quick"}
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_preview_artifact_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/preview-artifacts")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "preview_type":"schema_preview",
                  "preview_uri":"s3://preview/schema.json",
                  "preview_policy_json":{"mask":"pii_only"}
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_asset_field_definition_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/field-definitions")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "field_name":"amount",
                  "field_path":"amount",
                  "field_type":"decimal"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_asset_quality_report_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/quality-reports")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "report_no":1,
                  "sampling_method":"random_sample",
                  "report_uri":"s3://quality/report.json",
                  "report_hash":"sha256:quality-report"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_asset_processing_job_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/processing-jobs")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "processing_mode":"platform_managed",
                  "input_sources":[
                    {"input_asset_version_id":"00000000-0000-0000-0000-000000000002"}
                  ],
                  "processing_summary_json":{"strategy":"baseline_v1"}
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_data_contract_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001/data-contracts")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "contract_name":"Contract CAT013",
                  "business_terms_json":{"pricing":"fixed"}
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_get_data_contract_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001/data-contracts/00000000-0000-0000-0000-000000000002")
            .header("x-role", "developer")
            .body(Body::empty())
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_create_asset_object_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/objects")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "object_kind":"delivery_object",
                  "object_name":"delivery-package",
                  "object_uri":"s3://product/delivery-package.zip"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_release_policy_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/assets/00000000-0000-0000-0000-000000000001/release-policy")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "release_mode":"revision",
                  "is_revision_subscribable":true,
                  "update_frequency":"daily"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_submit_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/submit")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"submission_note":"ready"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_review_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/review/products/00000000-0000-0000-0000-000000000001")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"action_name":"approve"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_suspend_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/suspend")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"suspend_mode":"freeze"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
