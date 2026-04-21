use crate::AppState;
use axum::Router;
use axum::routing::{get, patch, post};

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/catalog/search", get(handlers::search_catalog))
        .route("/api/v1/ops/search/sync", get(handlers::get_search_sync))
        .route(
            "/api/v1/ops/search/reindex",
            post(handlers::post_search_reindex),
        )
        .route(
            "/api/v1/ops/search/aliases/switch",
            post(handlers::post_search_alias_switch),
        )
        .route(
            "/api/v1/ops/search/cache/invalidate",
            post(handlers::post_search_cache_invalidate),
        )
        .route(
            "/api/v1/ops/search/ranking-profiles",
            get(handlers::get_ranking_profiles),
        )
        .route(
            "/api/v1/ops/search/ranking-profiles/{id}",
            patch(handlers::patch_ranking_profile),
        )
}
