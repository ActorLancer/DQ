use crate::AppState;
use axum::Router;
use axum::routing::{get, patch, post};

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/recommendations", get(handlers::get_recommendations))
        .route(
            "/api/v1/recommendations/track/exposure",
            post(handlers::post_track_exposure),
        )
        .route(
            "/api/v1/recommendations/track/click",
            post(handlers::post_track_click),
        )
        .route(
            "/api/v1/ops/recommendation/placements",
            get(handlers::get_placements),
        )
        .route(
            "/api/v1/ops/recommendation/placements/{placement_code}",
            patch(handlers::patch_placement),
        )
        .route(
            "/api/v1/ops/recommendation/ranking-profiles",
            get(handlers::get_ranking_profiles),
        )
        .route(
            "/api/v1/ops/recommendation/ranking-profiles/{id}",
            patch(handlers::patch_ranking_profile),
        )
        .route(
            "/api/v1/ops/recommendation/rebuild",
            post(handlers::post_rebuild),
        )
}
