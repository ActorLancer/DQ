use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STANDARD_SKU_TYPES: &[&str] = &[
    "FILE_STD", "FILE_SUB", "SHARE_RO", "API_SUB", "API_PPU", "QRY_LITE", "SBX_STD", "RPT_STD",
];

pub fn is_standard_sku_type(sku_type: &str) -> bool {
    STANDARD_SKU_TYPES.contains(&sku_type)
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
    pub product_id: String,
    pub sku_code: String,
    pub sku_type: String,
    pub unit_name: Option<String>,
    pub billing_mode: String,
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
    pub acceptance_mode: String,
    pub refund_mode: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::{STANDARD_SKU_TYPES, is_standard_sku_type};

    #[test]
    fn standard_sku_truth_list_matches_v1_frozen_set() {
        assert_eq!(STANDARD_SKU_TYPES.len(), 8);
        assert!(is_standard_sku_type("FILE_STD"));
        assert!(is_standard_sku_type("RPT_STD"));
        assert!(!is_standard_sku_type("FILE_PREMIUM"));
    }
}
