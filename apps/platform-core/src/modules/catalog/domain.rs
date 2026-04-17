use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STANDARD_SKU_TYPES: &[&str] = &[
    "FILE_STD", "FILE_SUB", "SHARE_RO", "API_SUB", "API_PPU", "QRY_LITE", "SBX_STD", "RPT_STD",
];
pub const SUPPORTED_TRADE_MODES: &[&str] = &[
    "snapshot_sale",
    "revision_subscription",
    "share_grant",
    "api_subscription",
    "api_pay_per_use",
    "template_query",
    "sandbox_workspace",
    "report_delivery",
];

pub fn is_standard_sku_type(sku_type: &str) -> bool {
    STANDARD_SKU_TYPES.contains(&sku_type)
}

pub fn is_supported_trade_mode(trade_mode: &str) -> bool {
    SUPPORTED_TRADE_MODES.contains(&trade_mode)
}

pub fn default_trade_mode_for_sku_type(sku_type: &str) -> Option<&'static str> {
    match sku_type {
        "FILE_STD" => Some("snapshot_sale"),
        "FILE_SUB" => Some("revision_subscription"),
        "SHARE_RO" => Some("share_grant"),
        "API_SUB" => Some("api_subscription"),
        "API_PPU" => Some("api_pay_per_use"),
        "QRY_LITE" => Some("template_query"),
        "SBX_STD" => Some("sandbox_workspace"),
        "RPT_STD" => Some("report_delivery"),
        _ => None,
    }
}

pub fn is_trade_mode_compatible_with_sku(sku_type: &str, trade_mode: &str) -> bool {
    default_trade_mode_for_sku_type(sku_type)
        .map(|v| v == trade_mode)
        .unwrap_or(false)
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDataResourceRequest {
    pub owner_org_id: String,
    pub title: String,
    pub category: String,
    pub sensitivity_level: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DataResourceView {
    pub asset_id: String,
    pub owner_org_id: String,
    pub title: String,
    pub category: String,
    pub sensitivity_level: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssetVersionRequest {
    pub asset_id: String,
    pub version_no: i32,
    pub schema_version: Option<String>,
    pub schema_hash: Option<String>,
    pub sample_hash: Option<String>,
    pub full_hash: Option<String>,
    pub data_size_bytes: Option<i64>,
    pub origin_region: Option<String>,
    #[serde(default)]
    pub allowed_region: Vec<String>,
    pub requires_controlled_execution: Option<bool>,
    #[serde(default)]
    pub trust_boundary_snapshot: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AssetVersionView {
    pub asset_version_id: String,
    pub asset_id: String,
    pub version_no: i32,
    pub schema_version: Option<String>,
    pub schema_hash: Option<String>,
    pub sample_hash: Option<String>,
    pub full_hash: Option<String>,
    pub data_size_bytes: Option<i64>,
    pub origin_region: Option<String>,
    pub allowed_region: Vec<String>,
    pub requires_controlled_execution: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDataProductRequest {
    pub asset_id: String,
    pub asset_version_id: String,
    pub seller_org_id: String,
    pub title: String,
    pub category: String,
    pub product_type: String,
    pub description: Option<String>,
    pub price_mode: Option<String>,
    pub price: Option<String>,
    pub currency_code: Option<String>,
    pub delivery_type: String,
    #[serde(default)]
    pub allowed_usage: Vec<String>,
    pub searchable_text: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchDataProductRequest {
    pub title: Option<String>,
    pub category: Option<String>,
    pub product_type: Option<String>,
    pub description: Option<String>,
    pub price_mode: Option<String>,
    pub price: Option<String>,
    pub currency_code: Option<String>,
    pub delivery_type: Option<String>,
    pub searchable_text: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DataProductView {
    pub product_id: String,
    pub asset_id: String,
    pub asset_version_id: String,
    pub seller_org_id: String,
    pub title: String,
    pub category: String,
    pub product_type: String,
    pub status: String,
    pub price_mode: String,
    pub price: String,
    pub currency_code: String,
    pub delivery_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductSkuRequest {
    pub product_id: Option<String>,
    pub sku_code: String,
    pub sku_type: String,
    pub unit_name: Option<String>,
    pub billing_mode: String,
    pub trade_mode: Option<String>,
    pub delivery_object_kind: Option<String>,
    pub subscription_cadence: Option<String>,
    pub share_protocol: Option<String>,
    pub result_form: Option<String>,
    pub template_id: Option<String>,
    pub acceptance_mode: String,
    pub refund_mode: String,
    #[serde(default)]
    pub sla_json: Value,
    #[serde(default)]
    pub quota_json: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductSkuView {
    pub sku_id: String,
    pub product_id: String,
    pub sku_code: String,
    pub sku_type: String,
    pub unit_name: Option<String>,
    pub billing_mode: String,
    pub trade_mode: String,
    pub acceptance_mode: String,
    pub refund_mode: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchProductSkuRequest {
    pub sku_code: Option<String>,
    pub sku_type: Option<String>,
    pub unit_name: Option<String>,
    pub billing_mode: Option<String>,
    pub trade_mode: Option<String>,
    pub delivery_object_kind: Option<String>,
    pub subscription_cadence: Option<String>,
    pub share_protocol: Option<String>,
    pub result_form: Option<String>,
    pub template_id: Option<String>,
    pub acceptance_mode: Option<String>,
    pub refund_mode: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRawIngestBatchRequest {
    pub asset_id: Option<String>,
    pub owner_org_id: String,
    pub ingest_source_type: String,
    pub declared_object_family: Option<String>,
    #[serde(default)]
    pub source_declared_rights_json: Value,
    #[serde(default)]
    pub ingest_policy_json: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RawIngestBatchView {
    pub raw_ingest_batch_id: String,
    pub owner_org_id: String,
    pub asset_id: Option<String>,
    pub ingest_source_type: String,
    pub declared_object_family: Option<String>,
    pub source_declared_rights_json: Value,
    pub ingest_policy_json: Value,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRawObjectManifestRequest {
    pub raw_ingest_batch_id: Option<String>,
    pub storage_binding_id: Option<String>,
    pub object_name: String,
    pub object_uri: Option<String>,
    pub mime_type: Option<String>,
    pub container_type: Option<String>,
    pub byte_size: Option<i64>,
    pub object_hash: Option<String>,
    #[serde(default)]
    pub source_time_range_json: Value,
    #[serde(default)]
    pub manifest_json: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RawObjectManifestView {
    pub raw_object_manifest_id: String,
    pub raw_ingest_batch_id: String,
    pub storage_binding_id: Option<String>,
    pub object_name: String,
    pub object_uri: Option<String>,
    pub mime_type: Option<String>,
    pub container_type: Option<String>,
    pub byte_size: Option<i64>,
    pub object_hash: Option<String>,
    pub source_time_range_json: Value,
    pub manifest_json: Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFormatDetectionRequest {
    pub raw_object_manifest_id: Option<String>,
    pub detected_object_family: String,
    pub detected_format: Option<String>,
    #[serde(default)]
    pub schema_hint_json: Value,
    pub recommended_processing_path: Option<String>,
    pub classification_confidence: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FormatDetectionResultView {
    pub format_detection_result_id: String,
    pub raw_object_manifest_id: String,
    pub detected_object_family: String,
    pub detected_format: Option<String>,
    pub schema_hint_json: Value,
    pub recommended_processing_path: Option<String>,
    pub classification_confidence: Option<f64>,
    pub detected_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExtractionJobRequest {
    pub raw_object_manifest_id: Option<String>,
    pub asset_version_id: Option<String>,
    pub job_type: String,
    #[serde(default)]
    pub job_config_json: Value,
    #[serde(default)]
    pub result_summary_json: Value,
    pub output_uri: Option<String>,
    pub output_hash: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtractionJobView {
    pub extraction_job_id: String,
    pub raw_object_manifest_id: String,
    pub asset_version_id: Option<String>,
    pub job_type: String,
    pub job_config_json: Value,
    pub result_summary_json: Value,
    pub output_uri: Option<String>,
    pub output_hash: Option<String>,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePreviewArtifactRequest {
    pub asset_version_id: Option<String>,
    pub raw_object_manifest_id: Option<String>,
    pub preview_type: String,
    pub preview_uri: Option<String>,
    pub preview_hash: Option<String>,
    #[serde(default)]
    pub preview_payload: Value,
    #[serde(default)]
    pub preview_policy_json: Value,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PreviewArtifactView {
    pub preview_artifact_id: String,
    pub asset_version_id: String,
    pub raw_object_manifest_id: Option<String>,
    pub preview_type: String,
    pub preview_uri: Option<String>,
    pub preview_hash: Option<String>,
    pub preview_payload: Value,
    pub preview_policy_json: Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PutProductMetadataProfileRequest {
    pub product_id: Option<String>,
    pub metadata_version_no: Option<i32>,
    #[serde(default)]
    pub business_description_json: Value,
    #[serde(default)]
    pub data_content_json: Value,
    #[serde(default)]
    pub structure_description_json: Value,
    #[serde(default)]
    pub quality_description_json: Value,
    #[serde(default)]
    pub compliance_description_json: Value,
    #[serde(default)]
    pub delivery_description_json: Value,
    #[serde(default)]
    pub version_description_json: Value,
    #[serde(default)]
    pub authorization_description_json: Value,
    #[serde(default)]
    pub responsibility_description_json: Value,
    #[serde(default)]
    pub processing_overview_json: Value,
    pub status: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductMetadataProfileView {
    pub product_metadata_profile_id: String,
    pub product_id: String,
    pub asset_version_id: String,
    pub metadata_version_no: i32,
    pub business_description_json: Value,
    pub data_content_json: Value,
    pub structure_description_json: Value,
    pub quality_description_json: Value,
    pub compliance_description_json: Value,
    pub delivery_description_json: Value,
    pub version_description_json: Value,
    pub authorization_description_json: Value,
    pub responsibility_description_json: Value,
    pub processing_overview_json: Value,
    pub status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssetFieldDefinitionRequest {
    pub asset_version_id: Option<String>,
    pub object_name: Option<String>,
    pub field_name: String,
    pub field_path: String,
    pub field_type: String,
    pub is_nullable: Option<bool>,
    pub is_primary_key: Option<bool>,
    pub is_partition_key: Option<bool>,
    pub is_time_field: Option<bool>,
    pub code_rule: Option<String>,
    pub unit_text: Option<String>,
    #[serde(default)]
    pub enum_values_json: Value,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AssetFieldDefinitionView {
    pub field_definition_id: String,
    pub asset_version_id: String,
    pub object_name: Option<String>,
    pub field_name: String,
    pub field_path: String,
    pub field_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_partition_key: bool,
    pub is_time_field: bool,
    pub code_rule: Option<String>,
    pub unit_text: Option<String>,
    pub enum_values_json: Value,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssetQualityReportRequest {
    pub asset_version_id: Option<String>,
    pub report_no: Option<i32>,
    pub report_type: Option<String>,
    #[serde(default)]
    pub coverage_range_json: Value,
    #[serde(default)]
    pub freshness_json: Value,
    pub missing_rate: Option<f64>,
    pub duplicate_rate: Option<f64>,
    pub anomaly_rate: Option<f64>,
    pub sampling_method: Option<String>,
    pub assessed_at: Option<String>,
    pub assessor_org_id: Option<String>,
    pub report_uri: Option<String>,
    pub report_hash: Option<String>,
    #[serde(default)]
    pub metrics_json: Value,
    pub status: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AssetQualityReportView {
    pub quality_report_id: String,
    pub asset_version_id: String,
    pub report_no: i32,
    pub report_type: String,
    pub coverage_range_json: Value,
    pub freshness_json: Value,
    pub missing_rate: Option<f64>,
    pub duplicate_rate: Option<f64>,
    pub anomaly_rate: Option<f64>,
    pub sampling_method: Option<String>,
    pub assessed_at: Option<String>,
    pub assessor_org_id: Option<String>,
    pub report_uri: Option<String>,
    pub report_hash: Option<String>,
    pub metrics_json: Value,
    pub status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}
