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

#[cfg(test)]
mod tests {
    use super::{
        STANDARD_SKU_TYPES, default_trade_mode_for_sku_type, is_standard_sku_type,
        is_trade_mode_compatible_with_sku,
    };

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
}
