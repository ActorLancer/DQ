use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLogResponse {
    pub data: ApiUsageLogListResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLogListResponseData {
    pub order_id: String,
    pub sku_id: String,
    pub sku_type: String,
    pub current_state: String,
    pub payment_status: String,
    pub app: ApiUsageLogAppData,
    pub summary: ApiUsageLogSummaryData,
    pub logs: Vec<ApiUsageLogEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLogAppData {
    pub app_id: String,
    pub app_name: String,
    pub client_id: String,
    pub api_credential_id: String,
    pub credential_status: String,
    pub upstream_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLogSummaryData {
    pub total_calls: i64,
    pub successful_calls: i64,
    pub failed_calls: i64,
    pub total_usage_units: String,
    pub last_occurred_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageLogEntryData {
    pub api_usage_log_id: String,
    pub api_credential_id: String,
    pub request_ref: Option<String>,
    pub response_code: Option<i32>,
    pub response_class: Option<String>,
    pub usage_units: String,
    pub occurred_at: String,
}
