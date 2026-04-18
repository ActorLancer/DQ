mod errors;
mod handlers;
pub mod support;
pub mod validators;

use axum::Router;

pub fn router() -> Router {
    super::router::router()
}

pub(in crate::modules::catalog) use handlers::asset_pipeline::{
    create_asset_field_definition, create_asset_object, create_asset_processing_job,
    create_asset_quality_report, create_extraction_job, create_preview_artifact,
    create_raw_ingest_batch, create_raw_object_manifest, detect_raw_object_format,
    patch_asset_release_policy,
};
pub(in crate::modules::catalog) use handlers::product_and_review::{
    create_product_draft, get_standard_scenario_templates, patch_product_draft,
    put_product_metadata_profile, review_compliance, review_product, review_subject,
    submit_product, suspend_product,
};
pub(in crate::modules::catalog) use handlers::product_read::{
    get_product_detail, get_seller_profile,
};
pub(in crate::modules::catalog) use handlers::sku_contract::{
    create_data_contract, create_product_sku, get_data_contract, patch_product_sku,
};
pub(in crate::modules::catalog) use handlers::template_policy::{
    bind_product_template, bind_sku_template, patch_usage_policy,
};
