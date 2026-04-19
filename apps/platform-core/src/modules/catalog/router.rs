use crate::AppState;
use axum::Router;
use axum::routing::{get, patch, post, put};

use super::api;

/// Catalog HTTP routes (assembly only).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/products", post(api::create_product_draft))
        .route(
            "/api/v1/catalog/standard-scenarios",
            get(api::get_standard_scenario_templates),
        )
        .route(
            "/api/v1/products/{id}",
            get(api::get_product_detail).patch(api::patch_product_draft),
        )
        .route(
            "/api/v1/sellers/{orgId}/profile",
            get(api::get_seller_profile),
        )
        .route("/api/v1/products/{id}/submit", post(api::submit_product))
        .route(
            "/api/v1/products/{id}/bind-template",
            post(api::bind_product_template),
        )
        .route("/api/v1/products/{id}/suspend", post(api::suspend_product))
        .route(
            "/api/v1/skus/{id}/bind-template",
            post(api::bind_sku_template),
        )
        .route("/api/v1/policies/{id}", patch(api::patch_usage_policy))
        .route(
            "/api/v1/products/{id}/metadata-profile",
            put(api::put_product_metadata_profile),
        )
        .route("/api/v1/review/subjects/{id}", post(api::review_subject))
        .route("/api/v1/review/products/{id}", post(api::review_product))
        .route(
            "/api/v1/review/compliance/{id}",
            post(api::review_compliance),
        )
        .route("/api/v1/products/{id}/skus", post(api::create_product_sku))
        .route("/api/v1/skus/{id}", patch(api::patch_product_sku))
        .route(
            "/api/v1/skus/{id}/data-contracts",
            post(api::create_data_contract),
        )
        .route(
            "/api/v1/skus/{id}/data-contracts/{contractId}",
            get(api::get_data_contract),
        )
        .route(
            "/api/v1/assets/{assetId}/raw-ingest-batches",
            post(api::create_raw_ingest_batch),
        )
        .route(
            "/api/v1/raw-ingest-batches/{id}/manifests",
            post(api::create_raw_object_manifest),
        )
        .route(
            "/api/v1/raw-object-manifests/{id}/detect-format",
            post(api::detect_raw_object_format),
        )
        .route(
            "/api/v1/raw-object-manifests/{id}/extraction-jobs",
            post(api::create_extraction_job),
        )
        .route(
            "/api/v1/assets/{versionId}/preview-artifacts",
            post(api::create_preview_artifact),
        )
        .route(
            "/api/v1/assets/{versionId}/objects",
            post(api::create_asset_object),
        )
        .route(
            "/api/v1/assets/{assetId}/release-policy",
            patch(api::patch_asset_release_policy),
        )
        .route(
            "/api/v1/assets/{versionId}/field-definitions",
            post(api::create_asset_field_definition),
        )
        .route(
            "/api/v1/assets/{versionId}/quality-reports",
            post(api::create_asset_quality_report),
        )
        .route(
            "/api/v1/assets/{versionId}/processing-jobs",
            post(api::create_asset_processing_job),
        )
}
