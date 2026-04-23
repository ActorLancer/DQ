use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PRODUCT_SEARCH_READ_ALIAS: &str = "product_search_read";
pub const PRODUCT_SEARCH_WRITE_ALIAS: &str = "product_search_write";
pub const SELLER_SEARCH_READ_ALIAS: &str = "seller_search_read";
pub const SELLER_SEARCH_WRITE_ALIAS: &str = "seller_search_write";
pub const SEARCH_SYNC_JOBS_INDEX: &str = "search_sync_jobs_v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    #[serde(default = "default_entity_scope")]
    pub entity_scope: String,
    pub industry: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub delivery_mode: Option<String>,
    pub seller_org_id: Option<String>,
    pub seller_type: Option<String>,
    pub data_classification: Option<String>,
    pub price_mode: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    #[serde(default = "default_sort")]
    pub sort: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default = "default_include_facets")]
    pub include_facets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub entity_scope: String,
    pub entity_id: String,
    pub score: f64,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub seller_org_id: Option<String>,
    pub seller_name: Option<String>,
    pub category: Option<String>,
    pub product_type: Option<String>,
    pub status: String,
    pub price: Option<String>,
    pub currency_code: Option<String>,
    pub delivery_modes: Vec<String>,
    pub tags: Vec<String>,
    pub industry_tags: Vec<String>,
    pub country_code: Option<String>,
    pub reputation_score: Option<String>,
    pub quality_score: Option<String>,
    pub hotness_score: Option<String>,
    pub listing_product_count: Option<i64>,
    pub seller_type: Option<String>,
    pub data_classification: Option<String>,
    pub price_mode: Option<String>,
    pub document_version: i64,
    pub index_sync_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacetBucket {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacetSummary {
    pub seller_org_ids: Vec<SearchFacetBucket>,
    pub seller_types: Vec<SearchFacetBucket>,
    pub data_classifications: Vec<SearchFacetBucket>,
    pub price_modes: Vec<SearchFacetBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub entity_scope: String,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub cache_hit: bool,
    pub backend: String,
    pub facets: SearchFacetSummary,
    pub items: Vec<SearchResultItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchSyncQuery {
    pub entity_scope: Option<String>,
    pub sync_status: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchSyncTaskView {
    pub index_sync_task_id: String,
    pub entity_scope: String,
    pub entity_id: String,
    pub document_version: i64,
    pub target_backend: String,
    pub target_index: Option<String>,
    pub active_index_name: Option<String>,
    pub source_event_id: Option<String>,
    pub sync_status: String,
    pub retry_count: i32,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub reconcile_status: String,
    pub last_reconciled_at: Option<String>,
    pub dead_letter_event_id: Option<String>,
    pub open_exception_count: i32,
    pub latest_exception_id: Option<String>,
    pub latest_exception_type: Option<String>,
    pub latest_exception_status: Option<String>,
    pub latest_exception_error_code: Option<String>,
    pub latest_exception_error_message: Option<String>,
    pub latest_exception_retryable: bool,
    pub latest_exception_detected_at: Option<String>,
    pub latest_exception_resolved_at: Option<String>,
    pub projection_document_version: Option<i64>,
    pub projection_index_sync_status: Option<String>,
    pub projection_indexed_at: Option<String>,
    pub scheduled_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReindexRequest {
    pub entity_scope: String,
    pub entity_id: Option<String>,
    pub mode: String,
    pub force: Option<bool>,
    pub target_index: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReindexResponse {
    pub entity_scope: String,
    pub mode: String,
    pub enqueued_count: u64,
    pub target_backend: String,
    pub target_index: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AliasSwitchRequest {
    pub entity_scope: String,
    pub next_index_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AliasSwitchResponse {
    pub entity_scope: String,
    pub read_alias: String,
    pub write_alias: String,
    pub previous_index_name: Option<String>,
    pub active_index_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheInvalidateRequest {
    pub entity_scope: Option<String>,
    pub query_hash: Option<String>,
    pub purge_all: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheInvalidateResponse {
    pub entity_scope: Option<String>,
    pub deleted_keys: usize,
    pub invalidated_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankingProfileView {
    pub ranking_profile_id: String,
    pub profile_key: String,
    pub entity_scope: String,
    pub backend_type: String,
    pub weights_json: Value,
    pub filter_policy_json: Value,
    pub status: String,
    pub stage_from: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchRankingProfileRequest {
    pub weights_json: Option<Value>,
    pub filter_policy_json: Option<Value>,
    pub status: Option<String>,
}

fn default_entity_scope() -> String {
    "all".to_string()
}

fn default_sort() -> String {
    "composite".to_string()
}

fn default_include_facets() -> bool {
    true
}
