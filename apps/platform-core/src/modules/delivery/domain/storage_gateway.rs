use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageGatewaySnapshot {
    pub object_locator: Option<StorageGatewayObjectLocator>,
    pub integrity: StorageGatewayIntegrity,
    pub watermark_policy: StorageGatewayWatermarkPolicy,
    pub download_restriction: Option<StorageGatewayDownloadRestriction>,
    pub access_audit: StorageGatewayAccessAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageGatewayObjectLocator {
    pub object_id: String,
    pub object_uri: String,
    pub bucket_name: Option<String>,
    pub object_key: Option<String>,
    pub location_type: String,
    pub storage_zone: String,
    pub provider_type: Option<String>,
    pub namespace_name: Option<String>,
    pub namespace_kind: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub plaintext_visible_to_platform: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageGatewayIntegrity {
    pub content_hash: Option<String>,
    pub encryption_algo: Option<String>,
    pub delivery_commit_hash: Option<String>,
    pub receipt_hash: Option<String>,
    pub envelope_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageGatewayWatermarkPolicy {
    pub mode: String,
    pub rule: Value,
    pub fingerprint_fields: Vec<String>,
    pub watermark_hash: Option<String>,
    pub sensitive_delivery_mode: String,
    pub disclosure_review_status: String,
}

impl Default for StorageGatewayWatermarkPolicy {
    fn default() -> Self {
        Self {
            mode: "placeholder".to_string(),
            rule: serde_json::json!({"policy": "reserved_for_pipeline"}),
            fingerprint_fields: Vec::new(),
            watermark_hash: None,
            sensitive_delivery_mode: "standard".to_string(),
            disclosure_review_status: "not_required".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageGatewayDownloadRestriction {
    pub ticket_id: String,
    pub expire_at: String,
    pub download_limit: i32,
    pub download_count: i32,
    pub remaining_downloads: i32,
    pub current_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageGatewayAccessAudit {
    pub access_count: i64,
    pub last_downloaded_at: Option<String>,
    pub last_client_fingerprint: Option<String>,
}
