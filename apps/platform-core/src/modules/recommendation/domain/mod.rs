use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationQuery {
    pub placement_code: String,
    pub subject_scope: Option<String>,
    pub subject_org_id: Option<String>,
    pub subject_user_id: Option<String>,
    pub anonymous_session_key: Option<String>,
    pub context_entity_scope: Option<String>,
    pub context_entity_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationItem {
    pub recommendation_result_item_id: String,
    pub entity_scope: String,
    pub entity_id: String,
    pub title: String,
    pub seller_name: Option<String>,
    pub price: Option<String>,
    pub currency_code: Option<String>,
    pub final_score: f64,
    pub explanation_codes: Vec<String>,
    pub recall_sources: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationResponse {
    pub recommendation_request_id: String,
    pub recommendation_result_id: String,
    pub placement_code: String,
    pub strategy_version: String,
    pub cache_hit: bool,
    pub items: Vec<RecommendationItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExposureItemInput {
    pub recommendation_result_item_id: Option<String>,
    pub entity_scope: String,
    pub entity_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackExposureRequest {
    pub recommendation_request_id: String,
    pub recommendation_result_id: String,
    pub placement_code: String,
    #[serde(default)]
    pub items: Vec<ExposureItemInput>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackClickRequest {
    pub recommendation_request_id: String,
    pub recommendation_result_id: String,
    pub recommendation_result_item_id: String,
    pub entity_scope: String,
    pub entity_id: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorTrackResponse {
    pub accepted_count: u64,
    pub deduplicated_count: u64,
    pub behavior_event_ids: Vec<String>,
    pub outbox_enqueued_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlacementView {
    pub placement_code: String,
    pub placement_name: String,
    pub placement_scope: String,
    pub page_context: String,
    pub candidate_policy_json: Value,
    pub filter_policy_json: Value,
    pub default_ranking_profile_key: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchPlacementRequest {
    pub candidate_policy_json: Option<Value>,
    pub filter_policy_json: Option<Value>,
    pub default_ranking_profile_key: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationRankingProfileView {
    pub recommendation_ranking_profile_id: String,
    pub profile_key: String,
    pub placement_scope: String,
    pub backend_type: String,
    pub weights_json: Value,
    pub diversity_policy_json: Value,
    pub exploration_policy_json: Value,
    pub explain_codes: Vec<String>,
    pub status: String,
    pub stage_from: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchRecommendationRankingProfileRequest {
    pub weights_json: Option<Value>,
    pub diversity_policy_json: Option<Value>,
    pub exploration_policy_json: Option<Value>,
    pub explain_codes: Option<Vec<String>>,
    pub status: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationRebuildRequest {
    #[serde(default = "default_rebuild_scope")]
    pub scope: String,
    pub placement_code: Option<String>,
    pub subject_scope: Option<String>,
    pub subject_org_id: Option<String>,
    pub subject_user_id: Option<String>,
    pub anonymous_session_key: Option<String>,
    pub entity_scope: Option<String>,
    pub entity_id: Option<String>,
    pub purge_cache: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationRebuildResponse {
    pub scope: String,
    pub cache_keys_deleted: usize,
    pub refreshed_subject_profiles: u64,
    pub refreshed_cohort_rows: u64,
    pub refreshed_signal_rows: u64,
    pub refreshed_similarity_rows: u64,
    pub refreshed_bundle_rows: u64,
}

fn default_rebuild_scope() -> String {
    "all".to_string()
}
