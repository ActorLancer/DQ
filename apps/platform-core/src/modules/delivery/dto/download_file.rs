use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFileResponse {
    pub data: DownloadFileResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFileResponseData {
    pub order_id: String,
    pub delivery_id: String,
    pub ticket_id: String,
    pub receipt_id: String,
    pub receipt_hash: String,
    pub downloaded_at: String,
    pub ticket_status: String,
    pub download_limit: i32,
    pub download_count: i32,
    pub remaining_downloads: i32,
    pub bucket_name: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub content_hash: String,
    pub delivery_commit_hash: String,
    pub key_envelope: DownloadKeyEnvelopeData,
    pub object_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadKeyEnvelopeData {
    pub envelope_id: String,
    pub key_cipher: String,
    pub key_control_mode: Option<String>,
    pub unwrap_policy_json: Value,
    pub key_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadFileAccessData {
    pub order_id: String,
    pub delivery_id: String,
    pub ticket_id: String,
    pub receipt_id: String,
    pub receipt_hash: String,
    pub downloaded_at: String,
    pub ticket_status: String,
    pub download_limit: i32,
    pub download_count: i32,
    pub remaining_downloads: i32,
    pub bucket_name: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub content_hash: String,
    pub delivery_commit_hash: String,
    pub key_envelope: DownloadKeyEnvelopeData,
}
