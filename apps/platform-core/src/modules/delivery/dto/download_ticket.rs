use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTicketResponse {
    pub data: DownloadTicketResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTicketResponseData {
    pub order_id: String,
    pub delivery_id: String,
    pub ticket_id: String,
    pub download_token: String,
    pub ticket_status: String,
    pub issued_at: String,
    pub expire_at: String,
    pub download_limit: i32,
    pub download_count: i32,
    pub remaining_downloads: i32,
    pub bucket_name: String,
    pub object_key: String,
    pub envelope_id: String,
    pub delivery_commit_hash: String,
}
