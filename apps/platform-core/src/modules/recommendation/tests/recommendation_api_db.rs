use super::authorization_header;
use crate::modules::recommendation::api::router;
use crate::modules::recommendation::domain::{
    RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES, RECOMMENDATION_BASELINE_PLACEMENTS,
    RECOMMENDATION_BASELINE_RANKING_PROFILE_KEYS,
};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use redis::AsyncCommands;
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("RECOMMEND_DB_SMOKE").ok().as_deref() == Some("1")
}

static RECOMMEND_ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        // SAFETY: test-only mutation guarded by RECOMMEND_ENV_TEST_LOCK.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            // SAFETY: test-only mutation guarded by RECOMMEND_ENV_TEST_LOCK.
            unsafe { std::env::set_var(self.key, previous) };
        } else {
            // SAFETY: test-only mutation guarded by RECOMMEND_ENV_TEST_LOCK.
            unsafe { std::env::remove_var(self.key) };
        }
    }
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_ids: Vec<String>,
    asset_version_ids: Vec<String>,
    product_ids: Vec<String>,
    operator_user_id: String,
}

#[derive(Debug)]
struct StandardScenarioSample {
    scenario_code: String,
    scenario_name: String,
    product_id: String,
}

#[derive(Debug)]
struct PlacementConfigSnapshot {
    default_ranking_profile_key: Option<String>,
    metadata: Value,
}

#[derive(Debug)]
struct RankingProfileMetadataSnapshot {
    metadata: Value,
}

async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', jsonb_build_object('description', $2::text)
             )
             RETURNING org_id::text",
            &[
                &format!("recommend-org-{suffix}"),
                &format!("recommend seller {suffix}"),
            ],
        )
        .await?;
    let org_id: String = org.get(0);

    let mut asset_ids = Vec::new();
    let mut asset_version_ids = Vec::new();
    let mut product_ids = Vec::new();
    for index in 0..2 {
        let asset = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
                 )
                 RETURNING asset_id::text",
                &[&org_id, &format!("recommend-asset-{suffix}-{index}")],
            )
            .await?;
        let asset_id: String = asset.get(0);

        let asset_version = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id],
            )
            .await?;
        let asset_version_id: String = asset_version.get(0);

        let product = client
            .query_one(
                "INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing',
                   CASE WHEN $5::int = 0 THEN 'data_product' ELSE 'service' END,
                   $6, 'listed', 'one_time', $7, 'CNY',
                   CASE WHEN $5::int = 0 THEN 'file_download' ELSE 'api' END,
                   ARRAY['internal_use']::text[], $8,
                   jsonb_build_object(
                     'subtitle', $9,
                     'industry', 'industrial_manufacturing',
                     'quality_score', CASE WHEN $5::int = 0 THEN '0.92' ELSE '0.88' END,
                     'use_cases', jsonb_build_array('质量日报', '设备巡检')
                   )
                 )
                 RETURNING product_id::text",
                &[
                    &asset_id,
                    &asset_version_id,
                    &org_id,
                    &format!("recommend-product-{suffix}-{index}"),
                    &(index as i32),
                    &format!("recommend product {suffix} {index}"),
                    &(88.0f64 + index as f64),
                    &format!("recommend keyword {suffix} {index}"),
                    &format!("recommend subtitle {suffix} {index}"),
                ],
            )
            .await?;
        let product_id: String = product.get(0);
        client
            .execute(
                "SELECT search.refresh_product_search_document_by_id($1::text::uuid)",
                &[&product_id],
            )
            .await?;

        asset_ids.push(asset_id);
        asset_version_ids.push(asset_version_id);
        product_ids.push(product_id);
    }
    client
        .execute(
            "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
            &[&org_id],
        )
        .await?;

    client
        .execute(
            "INSERT INTO recommend.cohort_definition (
               cohort_key, subject_scope, dimension_json, status
             ) VALUES (
               $1, 'organization', jsonb_build_object('smoke', true), 'active'
             )
             ON CONFLICT (cohort_key) DO NOTHING",
            &[&format!("org:{org_id}")],
        )
        .await?;
    client
        .execute(
            "INSERT INTO recommend.cohort_popularity (
               cohort_key, entity_scope, entity_id, exposure_count, click_count, hotness_score
             ) VALUES (
               $1, 'product', $2::text::uuid, 3, 2, 5.0
             )
             ON CONFLICT (cohort_key, entity_scope, entity_id) DO UPDATE
             SET hotness_score = EXCLUDED.hotness_score",
            &[&format!("org:{org_id}"), &product_ids[0]],
        )
        .await?;
    client
        .execute(
            "INSERT INTO recommend.bundle_relation (
               source_entity_scope, source_entity_id, target_entity_scope, target_entity_id,
               relation_type, relation_score, status
             ) VALUES (
               'product', $1::text::uuid, 'product', $2::text::uuid, 'co_recommended', 2.5, 'active'
             )
             ON CONFLICT (source_entity_scope, source_entity_id, target_entity_scope, target_entity_id, relation_type)
             DO UPDATE SET relation_score = EXCLUDED.relation_score",
            &[&product_ids[0], &product_ids[1]],
        )
        .await?;

    let operator_user_id = seed_operator_user(client, &org_id, suffix).await?;

    Ok(SeedIds {
        org_id,
        asset_ids,
        asset_version_ids,
        product_ids,
        operator_user_id,
    })
}

async fn load_standard_scenario_samples(
    client: &Client,
) -> Result<Vec<StandardScenarioSample>, String> {
    let rows = client
        .query(
            "SELECT
               metadata ->> 'scenario_code',
               scenario_name,
               metadata ->> 'primary_product_id'
             FROM developer.test_application
             WHERE metadata ->> 'seed' = 'db035'
               AND metadata ->> 'recommended_placement_code' = 'home_featured'
             ORDER BY metadata ->> 'scenario_code'",
            &[],
        )
        .await
        .map_err(|err| format!("load standard scenario homepage samples failed: {err}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let product_id: Option<String> = row.get(2);
            product_id.map(|product_id| StandardScenarioSample {
                scenario_code: row.get(0),
                scenario_name: row.get(1),
                product_id,
            })
        })
        .collect())
}

async fn load_placement_config_snapshot(
    client: &Client,
    placement_code: &str,
) -> Result<PlacementConfigSnapshot, Error> {
    let row = client
        .query_one(
            "SELECT
               default_ranking_profile_key,
               metadata
             FROM recommend.placement_definition
             WHERE placement_code = $1",
            &[&placement_code],
        )
        .await?;
    Ok(PlacementConfigSnapshot {
        default_ranking_profile_key: row.get(0),
        metadata: row.get(1),
    })
}

async fn restore_placement_config_snapshot(
    client: &Client,
    placement_code: &str,
    snapshot: &PlacementConfigSnapshot,
) -> Result<(), Error> {
    client
        .execute(
            "UPDATE recommend.placement_definition
             SET default_ranking_profile_key = $2,
                 metadata = $3::jsonb,
                 updated_at = now()
             WHERE placement_code = $1",
            &[
                &placement_code,
                &snapshot.default_ranking_profile_key,
                &snapshot.metadata,
            ],
        )
        .await?;
    Ok(())
}

async fn load_ranking_profile_metadata_snapshot(
    client: &Client,
    profile_id: &str,
) -> Result<RankingProfileMetadataSnapshot, Error> {
    let row = client
        .query_one(
            "SELECT metadata
             FROM recommend.ranking_profile
             WHERE recommendation_ranking_profile_id = $1::text::uuid",
            &[&profile_id],
        )
        .await?;
    Ok(RankingProfileMetadataSnapshot {
        metadata: row.get(0),
    })
}

async fn restore_ranking_profile_metadata_snapshot(
    client: &Client,
    profile_id: &str,
    snapshot: &RankingProfileMetadataSnapshot,
) -> Result<(), Error> {
    client
        .execute(
            "UPDATE recommend.ranking_profile
             SET metadata = $2::jsonb,
                 updated_at = now()
             WHERE recommendation_ranking_profile_id = $1::text::uuid",
            &[&profile_id, &snapshot.metadata],
        )
        .await?;
    Ok(())
}

async fn seed_operator_user(client: &Client, org_id: &str, suffix: &str) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO core.user_account (
               org_id, login_id, display_name, user_type, status, mfa_status, attrs
             ) VALUES (
               $1::text::uuid, $2, $3, 'person', 'active', 'verified', jsonb_build_object('seed', $4)
             )
             RETURNING user_id::text",
            &[
                &org_id,
                &format!("recommend-ops-user-{suffix}"),
                &format!("Recommendation Ops User {suffix}"),
                &suffix,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn seed_verified_step_up_challenge(
    client: &Client,
    user_id: &str,
    target_action: &str,
    target_ref_type: &str,
    target_ref_id: Option<&str>,
    seed_label: &str,
) -> Result<String, Error> {
    client
        .query_one(
            "INSERT INTO iam.step_up_challenge (
               user_id,
               challenge_type,
               target_action,
               target_ref_type,
               target_ref_id,
               challenge_status,
               expires_at,
               completed_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'mock_otp',
               $2,
               $3,
               $4::text::uuid,
               'verified',
               now() + interval '10 minutes',
               now(),
               jsonb_build_object('seed', $5)
             )
             RETURNING step_up_challenge_id::text",
            &[
                &user_id,
                &target_action,
                &target_ref_type,
                &target_ref_id,
                &seed_label,
            ],
        )
        .await
        .map(|row| row.get(0))
}

async fn cleanup_graph(client: &Client, ids: &SeedIds) -> Result<(), String> {
    let entity_ids = vec![
        ids.org_id.clone(),
        ids.product_ids[0].clone(),
        ids.product_ids[1].clone(),
    ];
    let cleanup_marker = json!({
        "cleanup_state": "tombstoned",
        "cleanup_source": "recommendation_api_db_smoke",
        "cleanup_strategy": "principal_inactive_tombstone",
    });

    client
        .execute(
            "DELETE FROM ops.outbox_event
             WHERE event_type = 'recommend.behavior_recorded'
               AND payload -> 'payload' ->> 'recommendation_request_id' IN (
                 SELECT recommendation_request_id::text
                 FROM recommend.recommendation_request
                 WHERE subject_org_id = $1::text::uuid
               )",
            &[&ids.org_id],
        )
        .await
        .map_err(|err| format!("cleanup recommendation outbox events failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.behavior_event WHERE subject_org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await
        .map_err(|err| format!("cleanup recommendation behavior events failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.recommendation_request WHERE subject_org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await
        .map_err(|err| format!("cleanup recommendation requests failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.subject_profile_snapshot
             WHERE org_id = $1::text::uuid
                OR user_id = $2::text::uuid",
            &[&ids.org_id, &ids.operator_user_id],
        )
        .await
        .map_err(|err| format!("cleanup recommendation subject profiles failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.cohort_popularity WHERE cohort_key = $1",
            &[&format!("org:{}", ids.org_id)],
        )
        .await
        .map_err(|err| format!("cleanup recommendation cohort popularity failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.cohort_definition WHERE cohort_key = $1",
            &[&format!("org:{}", ids.org_id)],
        )
        .await
        .map_err(|err| format!("cleanup recommendation cohort definition failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.entity_similarity
             WHERE source_entity_id = ANY($1::text[]::uuid[])
                OR target_entity_id = ANY($1::text[]::uuid[])",
            &[&ids.product_ids],
        )
        .await
        .map_err(|err| format!("cleanup recommendation entity similarity failed: {err}"))?;
    client
        .execute(
            "DELETE FROM recommend.bundle_relation
             WHERE source_entity_id = $1::text::uuid OR target_entity_id = $1::text::uuid
                OR source_entity_id = $2::text::uuid OR target_entity_id = $2::text::uuid",
            &[&ids.product_ids[0], &ids.product_ids[1]],
        )
        .await
        .map_err(|err| format!("cleanup recommendation bundle relations failed: {err}"))?;
    client
        .execute(
            "DELETE FROM search.search_signal_aggregate
             WHERE entity_id = ANY($1::text[]::uuid[])",
            &[&entity_ids],
        )
        .await
        .map_err(|err| format!("cleanup search signal aggregate failed: {err}"))?;
    client
        .execute(
            "DELETE FROM search.index_sync_task
             WHERE entity_id = ANY($1::text[]::uuid[])",
            &[&entity_ids],
        )
        .await
        .map_err(|err| format!("cleanup search index sync tasks failed: {err}"))?;
    for product_id in &ids.product_ids {
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[product_id],
            )
            .await
            .map_err(|err| format!("cleanup catalog product {product_id} failed: {err}"))?;
    }
    for asset_version_id in &ids.asset_version_ids {
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[asset_version_id],
            )
            .await
            .map_err(|err| {
                format!("cleanup catalog asset version {asset_version_id} failed: {err}")
            })?;
    }
    for asset_id in &ids.asset_ids {
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[asset_id],
            )
            .await
            .map_err(|err| format!("cleanup catalog data asset {asset_id} failed: {err}"))?;
    }
    client
        .execute(
            "UPDATE core.user_account
             SET status = 'inactive',
                 attrs = attrs || $2::jsonb,
                 updated_at = now()
             WHERE user_id = $1::text::uuid",
            &[&ids.operator_user_id, &cleanup_marker],
        )
        .await
        .map_err(|err| format!("tombstone recommendation operator user failed: {err}"))?;
    client
        .execute(
            "UPDATE core.organization
             SET status = 'inactive',
                 metadata = metadata || $2::jsonb,
                 updated_at = now()
             WHERE org_id = $1::text::uuid",
            &[&ids.org_id, &cleanup_marker],
        )
        .await
        .map_err(|err| format!("tombstone recommendation organization failed: {err}"))?;

    let request_count = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM recommend.recommendation_request
             WHERE subject_org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await
        .map_err(|err| format!("verify recommendation request cleanup failed: {err}"))?
        .get::<_, i64>(0);
    if request_count != 0 {
        return Err(format!(
            "recommendation cleanup left {request_count} recommendation_request rows for org {}",
            ids.org_id
        ));
    }

    let product_count = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM catalog.product
             WHERE product_id = ANY($1::text[]::uuid[])",
            &[&ids.product_ids],
        )
        .await
        .map_err(|err| format!("verify catalog product cleanup failed: {err}"))?
        .get::<_, i64>(0);
    if product_count != 0 {
        return Err(format!(
            "recommendation cleanup left {product_count} catalog.product rows for seeded ids"
        ));
    }

    let user_row = client
        .query_opt(
            "SELECT status, attrs ->> 'cleanup_state'
             FROM core.user_account
             WHERE user_id = $1::text::uuid",
            &[&ids.operator_user_id],
        )
        .await
        .map_err(|err| format!("verify recommendation user tombstone failed: {err}"))?;
    match user_row {
        Some(row)
            if row.get::<_, String>(0) == "inactive"
                && row.get::<_, Option<String>>(1).as_deref() == Some("tombstoned") => {}
        Some(row) => {
            return Err(format!(
                "recommendation operator user not tombstoned: status={} cleanup_state={:?}",
                row.get::<_, String>(0),
                row.get::<_, Option<String>>(1)
            ));
        }
        None => {
            return Err(format!(
                "recommendation operator user {} missing after cleanup",
                ids.operator_user_id
            ));
        }
    }

    let org_row = client
        .query_opt(
            "SELECT status, metadata ->> 'cleanup_state'
             FROM core.organization
             WHERE org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await
        .map_err(|err| format!("verify recommendation organization tombstone failed: {err}"))?;
    match org_row {
        Some(row)
            if row.get::<_, String>(0) == "inactive"
                && row.get::<_, Option<String>>(1).as_deref() == Some("tombstoned") => {}
        Some(row) => {
            return Err(format!(
                "recommendation organization not tombstoned: status={} cleanup_state={:?}",
                row.get::<_, String>(0),
                row.get::<_, Option<String>>(1)
            ));
        }
        None => {
            return Err(format!(
                "recommendation organization {} missing after cleanup",
                ids.org_id
            ));
        }
    }

    Ok(())
}

async fn update_product_status_and_refresh_projection(
    client: &Client,
    product_id: &str,
    status: &str,
) -> Result<(), Error> {
    client
        .execute(
            "UPDATE catalog.product
             SET status = $2,
                 updated_at = now()
             WHERE product_id = $1::text::uuid",
            &[&product_id, &status],
        )
        .await?;
    client
        .execute(
            "SELECT search.refresh_product_search_document_by_id($1::text::uuid)",
            &[&product_id],
        )
        .await?;
    Ok(())
}

async fn seed_opensearch_documents(ids: &SeedIds, suffix: &str) -> Result<(), String> {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let client = reqwest::Client::new();
    for (index, product_id) in ids.product_ids.iter().enumerate() {
        let product_doc = json!({
            "entity_scope": "product",
            "id": product_id,
            "seller_id": ids.org_id,
            "name": format!("recommend-product-{suffix}-{index}"),
            "title": format!("recommend-product-{suffix}-{index}"),
            "subtitle": format!("recommend subtitle {suffix} {index}"),
            "description": format!("recommend keyword {suffix} {index}"),
            "category": "manufacturing",
            "product_type": if index == 0 { "data_product" } else { "service" },
            "industry": "industrial_manufacturing",
            "tags": ["质量", "巡检"],
            "seller_name": format!("recommend-org-{suffix}"),
            "delivery_modes": if index == 0 { vec!["file_download"] } else { vec!["api"] },
            "price_amount": 88.0 + index as f64,
            "currency_code": "CNY",
            "status": "listed",
            "review_status": "approved",
            "visibility_status": "visible",
            "visible_to_search": true,
            "quality_score": if index == 0 { 0.92 } else { 0.88 },
            "seller_reputation_score": 0.86,
            "hotness_score": 0.66,
            "updated_at": "2026-04-21T00:00:00.000Z"
        });
        let response = client
            .put(format!(
                "{}/product_search_write/_doc/{}?refresh=wait_for",
                endpoint.trim_end_matches('/'),
                product_id
            ))
            .json(&product_doc)
            .send()
            .await
            .map_err(|err| format!("seed recommendation product opensearch doc failed: {err}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "seed recommendation product opensearch status={}",
                response.status()
            ));
        }
    }

    let seller_doc = json!({
        "entity_scope": "seller",
        "id": ids.org_id,
        "name": format!("recommend-org-{suffix}"),
        "description": format!("recommend seller {suffix}"),
        "status": "active",
        "country_code": "CN",
        "region_code": "SH",
        "industry_tags": ["industrial_manufacturing"],
        "certification_tags": ["real_name_verified", "compliance:l2"],
        "featured_products": [{
            "product_id": ids.product_ids[0],
            "title": format!("recommend-product-a-{suffix}"),
            "category": "manufacturing",
            "price_amount": 88.0,
            "currency_code": "CNY"
        }],
        "rating_summary": {
            "rating_count": 9,
            "average_rating": 4.6,
            "reputation_score": 0.88,
            "credit_level": 4,
            "risk_level": 1
        },
        "reputation_score": 0.88,
        "listing_product_count": 2,
        "updated_at": "2026-04-21T00:00:00.000Z"
    });
    let response = client
        .put(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .json(&seller_doc)
        .send()
        .await
        .map_err(|err| format!("seed recommendation seller opensearch doc failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "seed recommendation seller opensearch status={}",
            response.status()
        ));
    }
    Ok(())
}

async fn cleanup_opensearch_documents(ids: &SeedIds) {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let client = reqwest::Client::new();
    for product_id in &ids.product_ids {
        let _ = client
            .delete(format!(
                "{}/product_search_write/_doc/{}?refresh=wait_for",
                endpoint.trim_end_matches('/'),
                product_id
            ))
            .send()
            .await;
    }
    let _ = client
        .delete(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .send()
        .await;
}

async fn response_json(response: axum::response::Response) -> Result<Value, String> {
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|err| format!("read response body failed: {err}"))?;
    serde_json::from_slice(&body).map_err(|_err| {
        format!(
            "decode response body failed: status={status} body={}",
            String::from_utf8_lossy(&body)
        )
    })
}

async fn count_access_audit(
    client: &Client,
    request_id: &str,
    target_type: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = $1
               AND target_type = $2",
            &[&request_id, &target_type],
        )
        .await
        .map(|row| row.get(0))
}

async fn count_audit_events(
    client: &Client,
    request_id: &str,
    action_name: &str,
) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE request_id = $1
               AND action_name = $2",
            &[&request_id, &action_name],
        )
        .await
        .map(|row| row.get(0))
}

async fn count_system_logs(client: &Client, request_id: &str, message: &str) -> Result<i64, Error> {
    client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = $2",
            &[&request_id, &message],
        )
        .await
        .map(|row| row.get(0))
}

async fn latest_access_step_up(client: &Client, request_id: &str) -> Result<Option<String>, Error> {
    client
        .query_opt(
            "SELECT step_up_challenge_id::text
             FROM audit.access_audit
             WHERE request_id = $1
             ORDER BY created_at DESC
             LIMIT 1",
            &[&request_id],
        )
        .await
        .map(|row| row.and_then(|row| row.get::<usize, Option<String>>(0)))
}

fn redis_url() -> String {
    std::env::var("REDIS_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://default:datab_redis_pass@127.0.0.1:6379/1".to_string())
}

async fn seed_redis_value(key: &str, payload: &str, ttl_secs: u64) -> Result<(), String> {
    let client = redis::Client::open(redis_url())
        .map_err(|err| format!("init redis client failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("connect redis failed: {err}"))?;
    connection
        .set_ex::<_, _, ()>(key, payload, ttl_secs)
        .await
        .map_err(|err| format!("seed redis key failed: {err}"))?;
    Ok(())
}

async fn load_redis_value(key: &str) -> Result<Option<String>, String> {
    let client = redis::Client::open(redis_url())
        .map_err(|err| format!("init redis client failed: {err}"))?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| format!("connect redis failed: {err}"))?;
    connection
        .get(key)
        .await
        .map_err(|err| format!("load redis key failed: {err}"))
}

#[tokio::test]
async fn recommendation_model_baseline_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
    );
    let ids = seed_graph(&client, &suffix).await.expect("seed graph");

    let expected_codes = RECOMMENDATION_BASELINE_PLACEMENTS
        .iter()
        .map(|(code, _, _, _, _)| (*code).to_string())
        .collect::<Vec<_>>();
    let placement_rows = client
        .query(
            "SELECT
               placement_code,
               placement_scope,
               page_context,
               default_ranking_profile_key,
               status,
               candidate_policy_json -> 'recall'
             FROM recommend.placement_definition
             WHERE placement_code = ANY($1::text[])
             ORDER BY placement_code",
            &[&expected_codes],
        )
        .await
        .expect("load baseline placements");
    let placement_map = placement_rows
        .into_iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                (
                    row.get::<_, String>(1),
                    row.get::<_, String>(2),
                    row.get::<_, Option<String>>(3),
                    row.get::<_, String>(4),
                    row.get::<_, Option<Value>>(5),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    for (code, scope, page_context, ranking_key, recall_sources) in
        RECOMMENDATION_BASELINE_PLACEMENTS
    {
        let Some((actual_scope, actual_page_context, actual_ranking_key, status, recall_json)) =
            placement_map.get(*code)
        else {
            panic!("missing baseline placement: {code}");
        };
        assert_eq!(actual_scope, scope, "placement_scope mismatch for {code}");
        assert_eq!(
            actual_page_context, page_context,
            "page_context mismatch for {code}"
        );
        assert_eq!(
            actual_ranking_key.as_deref(),
            Some(*ranking_key),
            "default_ranking_profile_key mismatch for {code}"
        );
        assert_eq!(status, "active", "placement status mismatch for {code}");

        let actual_recall = recall_json
            .as_ref()
            .and_then(|value| value.as_array())
            .expect("placement recall array")
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<BTreeSet<_>>();
        let expected_recall = recall_sources.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            actual_recall, expected_recall,
            "candidate recall sources mismatch for {code}"
        );
    }

    let expected_profiles = RECOMMENDATION_BASELINE_RANKING_PROFILE_KEYS
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<_>>();
    let ranking_rows = client
        .query(
            "SELECT profile_key, placement_scope, status
             FROM recommend.ranking_profile
             WHERE profile_key = ANY($1::text[])
             ORDER BY profile_key",
            &[&expected_profiles],
        )
        .await
        .expect("load baseline ranking profiles");
    let ranking_map = ranking_rows
        .into_iter()
        .map(|row| {
            (
                row.get::<_, String>(0),
                (row.get::<_, String>(1), row.get::<_, String>(2)),
            )
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(
        ranking_map.len(),
        RECOMMENDATION_BASELINE_RANKING_PROFILE_KEYS.len()
    );
    for profile_key in RECOMMENDATION_BASELINE_RANKING_PROFILE_KEYS {
        let Some((placement_scope, status)) = ranking_map.get(*profile_key) else {
            panic!("missing baseline ranking profile: {profile_key}");
        };
        assert_eq!(status, "active", "ranking profile status mismatch");
        assert!(
            !placement_scope.is_empty(),
            "ranking profile scope is empty"
        );
    }

    let route_policy = client
        .query_one(
            "SELECT target_topic, consumer_group_hint
             FROM ops.event_route_policy
             WHERE aggregate_type = 'recommend.behavior_event'
               AND event_type = 'recommend.behavior_recorded'",
            &[],
        )
        .await
        .expect("load recommendation route policy");
    assert_eq!(route_policy.get::<_, String>(0), "dtp.recommend.behavior");
    assert_eq!(
        route_policy.get::<_, String>(1),
        "cg-recommendation-aggregator"
    );

    let legacy_trigger_missing = client
        .query_one(
            "SELECT NOT EXISTS (
               SELECT 1
               FROM pg_trigger
               WHERE tgname = 'trg_recommend_behavior_event_outbox'
                 AND NOT tgisinternal
             )",
            &[],
        )
        .await
        .expect("check legacy outbox trigger")
        .get::<_, bool>(0);
    assert!(legacy_trigger_missing);

    for (index, event_type) in [
        RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES[1],
        RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES[2],
    ]
    .iter()
    .enumerate()
    {
        client
            .execute(
                "INSERT INTO recommend.behavior_event (
                   subject_scope,
                   subject_org_id,
                   event_type,
                   placement_code,
                   entity_scope,
                   entity_id,
                   request_id,
                   trace_id,
                   attrs
                 ) VALUES (
                   'organization',
                   $1::text::uuid,
                   $2,
                   'home_featured',
                   'product',
                   $3::text::uuid,
                   $4,
                   $5,
                   jsonb_build_object(
                     'category', 'manufacturing',
                     'delivery_mode', 'file_download',
                     'tags', jsonb_build_array('质量', '巡检'),
                     'seed', $6
                   )
                 )",
                &[
                    &ids.org_id,
                    event_type,
                    &ids.product_ids[0],
                    &format!("recommend-base-request-{suffix}-{index}"),
                    &format!("recommend-base-trace-{suffix}-{index}"),
                    &suffix,
                ],
            )
            .await
            .expect("insert baseline behavior event");
    }

    let profile_row = client
        .query_one(
            "SELECT
               preferred_categories,
               preferred_delivery_modes,
               feature_snapshot ->> 'last_event_type',
               feature_snapshot ->> 'last_placement_code'
             FROM recommend.subject_profile_snapshot
             WHERE subject_scope = 'organization'
               AND subject_ref = $1",
            &[&ids.org_id],
        )
        .await
        .expect("load subject profile snapshot");
    let preferred_categories: Vec<String> = profile_row.get(0);
    let preferred_delivery_modes: Vec<String> = profile_row.get(1);
    assert!(
        preferred_categories
            .iter()
            .any(|item| item == "manufacturing")
    );
    assert!(
        preferred_delivery_modes
            .iter()
            .any(|item| item == "file_download")
    );
    assert_eq!(
        profile_row.get::<_, Option<String>>(2).as_deref(),
        Some("recommendation_item_clicked")
    );
    assert_eq!(
        profile_row.get::<_, Option<String>>(3).as_deref(),
        Some("home_featured")
    );

    let cohort_row = client
        .query_one(
            "SELECT exposure_count, click_count, hotness_score::float8
             FROM recommend.cohort_popularity
             WHERE cohort_key = $1
               AND entity_scope = 'product'
               AND entity_id = $2::text::uuid",
            &[&format!("org:{}", ids.org_id), &ids.product_ids[0]],
        )
        .await
        .expect("load cohort popularity");
    assert!(cohort_row.get::<_, i64>(0) >= 1);
    assert!(cohort_row.get::<_, i64>(1) >= 1);
    assert!(cohort_row.get::<_, f64>(2) >= 1.2);

    let behavior_count = client
        .query_one(
            "SELECT count(*)::bigint
             FROM recommend.behavior_event
             WHERE subject_org_id = $1::text::uuid
               AND event_type = ANY($2::text[])",
            &[
                &ids.org_id,
                &vec![
                    RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES[1].to_string(),
                    RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES[2].to_string(),
                ],
            ],
        )
        .await
        .expect("count baseline behavior events")
        .get::<_, i64>(0);
    assert_eq!(behavior_count, 2);

    cleanup_graph(&client, &ids).await.expect("cleanup graph");
}

#[tokio::test]
async fn recommendation_api_full_runtime_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = RECOMMEND_ENV_TEST_LOCK.lock().expect("lock recommend env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
    );
    let ids = seed_graph(&client, &suffix).await.expect("seed graph");
    seed_opensearch_documents(&ids, &suffix)
        .await
        .expect("seed opensearch docs");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "staging");
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);
    let admin_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["platform_admin"]);
    let get_request_id = format!("recommend-get-{suffix}");
    let get_trace_id = format!("recommend-get-trace-{suffix}");
    let exposure_request_id = format!("recommend-exposure-{suffix}");
    let exposure_trace_id = format!("recommend-exposure-trace-{suffix}");
    let click_request_id = format!("recommend-click-{suffix}");
    let click_trace_id = format!("recommend-click-trace-{suffix}");
    let placement_list_request_id = format!("recommend-placement-list-{suffix}");
    let placement_list_trace_id = format!("recommend-placement-list-trace-{suffix}");
    let ranking_list_request_id = format!("recommend-ranking-list-{suffix}");
    let ranking_list_trace_id = format!("recommend-ranking-list-trace-{suffix}");
    let placement_patch_request_id = format!("recommend-placement-patch-{suffix}");
    let placement_patch_trace_id = format!("recommend-placement-patch-trace-{suffix}");
    let ranking_patch_request_id = format!("recommend-ranking-patch-{suffix}");
    let ranking_patch_trace_id = format!("recommend-ranking-patch-trace-{suffix}");
    let rebuild_request_id = format!("recommend-rebuild-{suffix}");
    let rebuild_trace_id = format!("recommend-rebuild-trace-{suffix}");
    let placement_idempotency_key = format!("recommend-placement-{suffix}");
    let ranking_idempotency_key = format!("recommend-ranking-{suffix}");
    let rebuild_idempotency_key = format!("recommend-rebuild-{suffix}");
    let exposure_idempotency_key = format!("recommend-exposure-{suffix}");
    let click_idempotency_key = format!("recommend-click-{suffix}");
    let redis_namespace =
        std::env::var("REDIS_NAMESPACE").unwrap_or_else(|_| "datab:v1".to_string());
    let placement_cache_key = format!(
        "{redis_namespace}:recommend:{}:{}:placement-{suffix}",
        ids.org_id, ids.operator_user_id
    );
    let placement_seen_key =
        format!("{redis_namespace}:recommend:seen:placement-smoke-{suffix}:home_featured");
    let rebuild_cache_key = format!(
        "{redis_namespace}:recommend:{}:anonymous:rebuild-{suffix}",
        ids.org_id
    );
    let rebuild_seen_key = format!(
        "{redis_namespace}:recommend:seen:{}:home_featured",
        ids.org_id
    );

    let app = crate::with_live_test_state(router()).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/recommendations?placement_code=home_featured&subject_scope=organization&subject_org_id={}&limit=3",
                    ids.org_id
                ))
                .header("authorization", &buyer_auth)
                .header("x-request-id", &get_request_id)
                .header("x-trace-id", &get_trace_id)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("recommend response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let json_body: Value = serde_json::from_slice(&body).expect("json");
    let data = &json_body["data"];
    assert!(data["recommendation_request_id"].is_string());
    assert!(data["recommendation_result_id"].is_string());
    assert!(
        data["items"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let recommendation_request_id = data["recommendation_request_id"]
        .as_str()
        .expect("request id");
    let recommendation_result_id = data["recommendation_result_id"]
        .as_str()
        .expect("result id");
    let items = data["items"].as_array().expect("items");
    let first_item = &items[0];

    let exposure_payload = json!({
        "recommendation_request_id": recommendation_request_id,
        "recommendation_result_id": recommendation_result_id,
        "placement_code": "home_featured",
        "trace_id": format!("recommend-trace-{suffix}"),
        "items": items.iter().take(2).map(|item| json!({
          "recommendation_result_item_id": item["recommendation_result_item_id"],
          "entity_scope": item["entity_scope"],
          "entity_id": item["entity_id"]
        })).collect::<Vec<_>>()
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recommendations/track/exposure")
                .header("content-type", "application/json")
                .header("authorization", &buyer_auth)
                .header("x-idempotency-key", &exposure_idempotency_key)
                .header("x-request-id", &exposure_request_id)
                .header("x-trace-id", &exposure_trace_id)
                .body(Body::from(exposure_payload.to_string()))
                .expect("exposure request"),
        )
        .await
        .expect("exposure response");
    let exposure_status = response.status();
    let exposure_json = response_json(response).await.expect("exposure json");
    assert_eq!(exposure_status, StatusCode::OK, "{exposure_json}");
    assert_eq!(exposure_json["data"]["accepted_count"].as_u64(), Some(3));
    assert_eq!(
        exposure_json["data"]["outbox_enqueued_count"].as_u64(),
        Some(3)
    );

    let duplicate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recommendations/track/exposure")
                .header("content-type", "application/json")
                .header("authorization", &buyer_auth)
                .header("x-idempotency-key", &exposure_idempotency_key)
                .header("x-request-id", &exposure_request_id)
                .header("x-trace-id", &exposure_trace_id)
                .body(Body::from(exposure_payload.to_string()))
                .expect("duplicate exposure request"),
        )
        .await
        .expect("duplicate exposure response");
    let duplicate_status = duplicate_response.status();
    let duplicate_json = response_json(duplicate_response)
        .await
        .expect("duplicate exposure json");
    assert_eq!(duplicate_status, StatusCode::OK, "{duplicate_json}");
    assert_eq!(duplicate_json["data"]["accepted_count"].as_u64(), Some(0));
    assert!(
        duplicate_json["data"]["deduplicated_count"]
            .as_u64()
            .is_some_and(|count| count >= 1)
    );

    let click_payload = json!({
      "recommendation_request_id": recommendation_request_id,
      "recommendation_result_id": recommendation_result_id,
      "recommendation_result_item_id": first_item["recommendation_result_item_id"],
      "entity_scope": first_item["entity_scope"],
      "entity_id": first_item["entity_id"],
      "trace_id": format!("recommend-click-trace-{suffix}")
    });
    let click_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recommendations/track/click")
                .header("content-type", "application/json")
                .header("authorization", &buyer_auth)
                .header("x-idempotency-key", &click_idempotency_key)
                .header("x-request-id", &click_request_id)
                .header("x-trace-id", &click_trace_id)
                .body(Body::from(click_payload.to_string()))
                .expect("click request"),
        )
        .await
        .expect("click response");
    let click_status = click_response.status();
    let click_json = response_json(click_response).await.expect("click json");
    assert_eq!(click_status, StatusCode::OK, "{click_json}");
    assert_eq!(click_json["data"]["accepted_count"].as_u64(), Some(1));
    assert_eq!(
        click_json["data"]["outbox_enqueued_count"].as_u64(),
        Some(1)
    );

    let placements_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/recommendation/placements")
                .header("authorization", &admin_auth)
                .header("x-request-id", &placement_list_request_id)
                .header("x-trace-id", &placement_list_trace_id)
                .body(Body::empty())
                .expect("placements request"),
        )
        .await
        .expect("placements response");
    let placements_status = placements_response.status();
    let placements_json = response_json(placements_response)
        .await
        .expect("placements json");
    assert_eq!(placements_status, StatusCode::OK, "{placements_json}");
    assert!(
        placements_json["data"].as_array().is_some_and(|items| items
            .iter()
            .any(|item| { item["placement_code"].as_str() == Some("home_featured") })),
        "{placements_json}"
    );

    let ranking_profiles_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/recommendation/ranking-profiles")
                .header("authorization", &admin_auth)
                .header("x-request-id", &ranking_list_request_id)
                .header("x-trace-id", &ranking_list_trace_id)
                .body(Body::empty())
                .expect("ranking profiles request"),
        )
        .await
        .expect("ranking profiles response");
    assert_eq!(ranking_profiles_response.status(), StatusCode::OK);
    let ranking_profiles_body = to_bytes(ranking_profiles_response.into_body(), usize::MAX)
        .await
        .expect("ranking body");
    let ranking_profiles_json: Value =
        serde_json::from_slice(&ranking_profiles_body).expect("ranking json");
    let ranking_profile_id = ranking_profiles_json["data"][0]["recommendation_ranking_profile_id"]
        .as_str()
        .expect("ranking profile id");
    let ranking_profile_key = ranking_profiles_json["data"][0]["profile_key"]
        .as_str()
        .expect("ranking profile key");
    let placement_snapshot = load_placement_config_snapshot(&client, "home_featured")
        .await
        .expect("load placement snapshot");
    let ranking_profile_snapshot =
        load_ranking_profile_metadata_snapshot(&client, ranking_profile_id)
            .await
            .expect("load ranking profile snapshot");

    seed_redis_value(&placement_cache_key, "{\"seed\":true}", 300)
        .await
        .expect("seed placement cache key");
    seed_redis_value(&placement_seen_key, "1", 300)
        .await
        .expect("seed placement seen key");
    let placement_step_up = seed_verified_step_up_challenge(
        &client,
        &ids.operator_user_id,
        "recommendation.placement.patch",
        "recommendation_placement",
        None,
        &format!("recommend-placement-step-up-{suffix}"),
    )
    .await
    .expect("seed placement step-up");

    let patch_placement_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/ops/recommendation/placements/home_featured")
                .header("content-type", "application/json")
                .header("authorization", &admin_auth)
                .header("x-request-id", &placement_patch_request_id)
                .header("x-trace-id", &placement_patch_trace_id)
                .header("x-idempotency-key", &placement_idempotency_key)
                .header("x-step-up-token", &placement_step_up)
                .body(Body::from(
                    json!({
                      "default_ranking_profile_key": ranking_profile_key,
                      "metadata": { "smoke_suffix": suffix }
                    })
                    .to_string(),
                ))
                .expect("patch placement request"),
        )
        .await
        .expect("patch placement response");
    let patch_placement_status = patch_placement_response.status();
    let patch_placement_json = response_json(patch_placement_response)
        .await
        .expect("patch placement json");
    assert_eq!(
        patch_placement_status,
        StatusCode::OK,
        "{patch_placement_json}"
    );
    assert_eq!(
        patch_placement_json["data"]["metadata"]["smoke_suffix"].as_str(),
        Some(suffix.as_str())
    );
    assert_eq!(
        patch_placement_json["data"]["metadata"]["fixed_sample_set"].as_str(),
        placement_snapshot.metadata["fixed_sample_set"].as_str()
    );
    assert_eq!(
        patch_placement_json["data"]["metadata"]["fixed_samples"]
            .as_array()
            .map(|items| items.len()),
        placement_snapshot.metadata["fixed_samples"]
            .as_array()
            .map(|items| items.len())
    );
    assert_eq!(
        patch_placement_json["data"]["default_ranking_profile_key"].as_str(),
        Some(ranking_profile_key)
    );
    let placement_row = client
        .query_one(
            "SELECT
               default_ranking_profile_key,
               metadata ->> 'smoke_suffix'
             FROM recommend.placement_definition
             WHERE placement_code = 'home_featured'",
            &[],
        )
        .await
        .expect("load patched placement row");
    assert_eq!(
        placement_row.get::<_, Option<String>>(0).as_deref(),
        Some(ranking_profile_key)
    );
    assert_eq!(
        placement_row.get::<_, Option<String>>(1).as_deref(),
        Some(suffix.as_str())
    );
    assert_eq!(
        placement_snapshot.metadata["fixed_sample_set"].as_str(),
        Some("five_standard_scenarios_v1")
    );
    assert_eq!(
        count_access_audit(
            &client,
            &placement_list_request_id,
            "recommendation_placement"
        )
        .await
        .expect("count placement list access audit"),
        1
    );
    assert_eq!(
        count_system_logs(
            &client,
            &placement_list_request_id,
            "recommendation ops lookup executed: GET /api/v1/ops/recommendation/placements",
        )
        .await
        .expect("count placement list system log"),
        1
    );
    assert_eq!(
        count_access_audit(
            &client,
            &ranking_list_request_id,
            "recommendation_ranking_profile"
        )
        .await
        .expect("count ranking list access audit"),
        1
    );
    assert_eq!(
        count_system_logs(
            &client,
            &ranking_list_request_id,
            "recommendation ops lookup executed: GET /api/v1/ops/recommendation/ranking-profiles",
        )
        .await
        .expect("count ranking list system log"),
        1
    );
    assert_eq!(
        count_audit_events(
            &client,
            &placement_patch_request_id,
            "recommendation.placement.patch",
        )
        .await
        .expect("count placement patch audit event"),
        1
    );
    assert_eq!(
        count_access_audit(
            &client,
            &placement_patch_request_id,
            "recommendation_placement"
        )
        .await
        .expect("count placement patch access audit"),
        1
    );
    assert_eq!(
        latest_access_step_up(&client, &placement_patch_request_id)
            .await
            .expect("load placement patch step-up")
            .as_deref(),
        Some(placement_step_up.as_str())
    );
    assert_eq!(
        count_system_logs(
            &client,
            &placement_patch_request_id,
            "recommendation ops action executed: PATCH /api/v1/ops/recommendation/placements/{placement_code}",
        )
        .await
        .expect("count placement patch system log"),
        1
    );
    let placement_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'endpoint',
               metadata ->> 'permission_code',
               metadata -> 'details' -> 'cache_invalidation' ->> 'cache_keys_deleted'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&placement_patch_request_id],
        )
        .await
        .expect("load placement audit row");
    assert_eq!(placement_audit_row.get::<_, String>(0), "updated");
    assert_eq!(
        placement_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("PATCH /api/v1/ops/recommendation/placements/{placement_code}")
    );
    assert_eq!(
        placement_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("ops.recommendation.manage")
    );
    assert!(
        placement_audit_row
            .get::<_, Option<String>>(3)
            .as_deref()
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|deleted| deleted >= 2)
    );
    assert!(
        load_redis_value(&placement_cache_key)
            .await
            .expect("load placement cache after patch")
            .is_none()
    );
    assert!(
        load_redis_value(&placement_seen_key)
            .await
            .expect("load placement seen after patch")
            .is_none()
    );

    let missing_ranking_step_up: String = client
        .query_one("SELECT gen_random_uuid()::text", &[])
        .await
        .expect("generate missing ranking step-up id")
        .get(0);
    let missing_ranking_step_up_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/ops/recommendation/ranking-profiles/{ranking_profile_id}"
                ))
                .header("content-type", "application/json")
                .header("authorization", &admin_auth)
                .header(
                    "x-request-id",
                    format!("{ranking_patch_request_id}-missing"),
                )
                .header("x-trace-id", format!("{ranking_patch_trace_id}-missing"))
                .header(
                    "x-idempotency-key",
                    format!("{ranking_idempotency_key}-missing"),
                )
                .header("x-step-up-token", &missing_ranking_step_up)
                .body(Body::from(
                    json!({
                      "metadata": { "smoke_suffix": format!("{suffix}-missing") }
                    })
                    .to_string(),
                ))
                .expect("patch ranking missing step-up request"),
        )
        .await
        .expect("patch ranking missing step-up response");
    let missing_ranking_step_up_status = missing_ranking_step_up_response.status();
    let missing_ranking_step_up_json = response_json(missing_ranking_step_up_response)
        .await
        .expect("patch ranking missing step-up json");
    assert_eq!(
        missing_ranking_step_up_status,
        StatusCode::NOT_FOUND,
        "{missing_ranking_step_up_json}"
    );

    let ranking_step_up = seed_verified_step_up_challenge(
        &client,
        &ids.operator_user_id,
        "recommendation.ranking_profile.patch",
        "recommendation_ranking_profile",
        Some(ranking_profile_id),
        &format!("recommend-ranking-step-up-{suffix}"),
    )
    .await
    .expect("seed ranking step-up");

    let patch_ranking_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/ops/recommendation/ranking-profiles/{ranking_profile_id}"
                ))
                .header("content-type", "application/json")
                .header("authorization", &admin_auth)
                .header("x-request-id", &ranking_patch_request_id)
                .header("x-trace-id", &ranking_patch_trace_id)
                .header("x-idempotency-key", &ranking_idempotency_key)
                .header("x-step-up-token", &ranking_step_up)
                .body(Body::from(
                    json!({
                      "metadata": { "smoke_suffix": suffix }
                    })
                    .to_string(),
                ))
                .expect("patch ranking request"),
        )
        .await
        .expect("patch ranking response");
    let patch_ranking_status = patch_ranking_response.status();
    let patch_ranking_json = response_json(patch_ranking_response)
        .await
        .expect("patch ranking json");
    assert_eq!(patch_ranking_status, StatusCode::OK, "{patch_ranking_json}");
    assert_eq!(
        patch_ranking_json["data"]["metadata"]["smoke_suffix"].as_str(),
        Some(suffix.as_str())
    );
    let ranking_row = client
        .query_one(
            "SELECT metadata ->> 'smoke_suffix'
             FROM recommend.ranking_profile
             WHERE recommendation_ranking_profile_id = $1::text::uuid",
            &[&ranking_profile_id],
        )
        .await
        .expect("load patched ranking row");
    assert_eq!(
        ranking_row.get::<_, Option<String>>(0).as_deref(),
        Some(suffix.as_str())
    );
    assert_eq!(
        count_audit_events(
            &client,
            &ranking_patch_request_id,
            "recommendation.ranking_profile.patch",
        )
        .await
        .expect("count ranking patch audit event"),
        1
    );
    assert_eq!(
        count_access_audit(
            &client,
            &ranking_patch_request_id,
            "recommendation_ranking_profile"
        )
        .await
        .expect("count ranking patch access audit"),
        1
    );
    assert_eq!(
        latest_access_step_up(&client, &ranking_patch_request_id)
            .await
            .expect("load ranking patch step-up")
            .as_deref(),
        Some(ranking_step_up.as_str())
    );
    assert_eq!(
        count_system_logs(
            &client,
            &ranking_patch_request_id,
            "recommendation ops action executed: PATCH /api/v1/ops/recommendation/ranking-profiles/{id}",
        )
        .await
        .expect("count ranking patch system log"),
        1
    );
    let ranking_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'endpoint',
               metadata ->> 'permission_code',
               metadata -> 'details' ->> 'idempotency_key',
               metadata -> 'details' -> 'request_patch' -> 'metadata' ->> 'smoke_suffix'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&ranking_patch_request_id],
        )
        .await
        .expect("load ranking audit row");
    assert_eq!(ranking_audit_row.get::<_, String>(0), "updated");
    assert_eq!(
        ranking_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("PATCH /api/v1/ops/recommendation/ranking-profiles/{id}")
    );
    assert_eq!(
        ranking_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("ops.recommendation.manage")
    );
    assert_eq!(
        ranking_audit_row.get::<_, Option<String>>(3).as_deref(),
        Some(ranking_idempotency_key.as_str())
    );
    assert_eq!(
        ranking_audit_row.get::<_, Option<String>>(4).as_deref(),
        Some(suffix.as_str())
    );

    seed_redis_value(&rebuild_cache_key, "{\"seed\":true}", 300)
        .await
        .expect("seed rebuild cache key");
    seed_redis_value(&rebuild_seen_key, "1", 300)
        .await
        .expect("seed rebuild seen key");
    let rebuild_step_up = seed_verified_step_up_challenge(
        &client,
        &ids.operator_user_id,
        "recommendation.rebuild.execute",
        "recommendation_rebuild",
        None,
        &format!("recommend-rebuild-step-up-{suffix}"),
    )
    .await
    .expect("seed rebuild step-up");

    let rebuild_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ops/recommendation/rebuild")
                .header("content-type", "application/json")
                .header("authorization", &admin_auth)
                .header("x-request-id", &rebuild_request_id)
                .header("x-trace-id", &rebuild_trace_id)
                .header("x-idempotency-key", &rebuild_idempotency_key)
                .header("x-step-up-token", &rebuild_step_up)
                .body(Body::from(
                    json!({
                      "scope": "all",
                      "placement_code": "home_featured",
                      "subject_scope": "organization",
                      "subject_org_id": ids.org_id,
                      "purge_cache": true
                    })
                    .to_string(),
                ))
                .expect("rebuild request"),
        )
        .await
        .expect("rebuild response");
    let rebuild_status = rebuild_response.status();
    let rebuild_json = response_json(rebuild_response).await.expect("rebuild json");
    assert_eq!(rebuild_status, StatusCode::OK, "{rebuild_json}");
    assert_eq!(rebuild_json["data"]["scope"].as_str(), Some("all"));
    assert!(
        rebuild_json["data"]["cache_keys_deleted"]
            .as_u64()
            .is_some_and(|deleted| deleted >= 2),
        "{rebuild_json}"
    );
    assert!(
        rebuild_json["data"]["refreshed_subject_profiles"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "{rebuild_json}"
    );
    assert!(
        rebuild_json["data"]["refreshed_cohort_rows"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "{rebuild_json}"
    );
    assert!(
        rebuild_json["data"]["refreshed_signal_rows"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "{rebuild_json}"
    );
    assert!(
        rebuild_json["data"]["refreshed_similarity_rows"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "{rebuild_json}"
    );
    assert!(
        rebuild_json["data"]["refreshed_bundle_rows"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "{rebuild_json}"
    );

    let outbox_row = client
        .query_one(
            "SELECT count(*)::bigint
             FROM ops.outbox_event
             WHERE event_type = 'recommend.behavior_recorded'
               AND target_topic = 'dtp.recommend.behavior'
               AND payload -> 'payload' ->> 'recommendation_request_id' = $1",
            &[&recommendation_request_id],
        )
        .await
        .expect("outbox row count");
    let outbox_count: i64 = outbox_row.get(0);
    assert!(outbox_count >= 4);

    let exposure_audit_count = count_audit_events(
        &client,
        &exposure_request_id,
        "recommendation.exposure.track",
    )
    .await
    .expect("count exposure audit events");
    assert_eq!(exposure_audit_count, 2);
    let exposure_access_count =
        count_access_audit(&client, &exposure_request_id, "recommendation_behavior")
            .await
            .expect("count exposure access audit");
    assert_eq!(exposure_access_count, 2);
    let exposure_log_count = count_system_logs(
        &client,
        &exposure_request_id,
        "recommendation behavior tracked: POST /api/v1/recommendations/track/exposure",
    )
    .await
    .expect("count exposure system log");
    assert_eq!(exposure_log_count, 2);

    let click_audit_count =
        count_audit_events(&client, &click_request_id, "recommendation.click.track")
            .await
            .expect("count click audit events");
    assert_eq!(click_audit_count, 1);
    let click_access_count =
        count_access_audit(&client, &click_request_id, "recommendation_behavior")
            .await
            .expect("count click access audit");
    assert_eq!(click_access_count, 1);
    let click_log_count = count_system_logs(
        &client,
        &click_request_id,
        "recommendation behavior tracked: POST /api/v1/recommendations/track/click",
    )
    .await
    .expect("count click system log");
    assert_eq!(click_log_count, 1);

    assert_eq!(
        count_audit_events(
            &client,
            &rebuild_request_id,
            "recommendation.rebuild.execute",
        )
        .await
        .expect("count rebuild audit events"),
        1
    );
    assert_eq!(
        count_access_audit(&client, &rebuild_request_id, "recommendation_rebuild")
            .await
            .expect("count rebuild access audit"),
        1
    );
    assert_eq!(
        latest_access_step_up(&client, &rebuild_request_id)
            .await
            .expect("load rebuild step-up")
            .as_deref(),
        Some(rebuild_step_up.as_str())
    );
    assert_eq!(
        count_system_logs(
            &client,
            &rebuild_request_id,
            "recommendation ops action executed: POST /api/v1/ops/recommendation/rebuild",
        )
        .await
        .expect("count rebuild system log"),
        1
    );
    let rebuild_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'endpoint',
               metadata ->> 'permission_code',
               metadata -> 'details' ->> 'idempotency_key',
               metadata -> 'details' ->> 'cache_keys_deleted'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&rebuild_request_id],
        )
        .await
        .expect("load rebuild audit row");
    assert_eq!(rebuild_audit_row.get::<_, String>(0), "rebuilt");
    assert_eq!(
        rebuild_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("POST /api/v1/ops/recommendation/rebuild")
    );
    assert_eq!(
        rebuild_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("ops.recommend_rebuild.execute")
    );
    assert_eq!(
        rebuild_audit_row.get::<_, Option<String>>(3).as_deref(),
        Some(rebuild_idempotency_key.as_str())
    );
    assert!(
        rebuild_audit_row
            .get::<_, Option<String>>(4)
            .as_deref()
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|deleted| deleted >= 2)
    );
    assert!(
        load_redis_value(&rebuild_cache_key)
            .await
            .expect("load rebuild cache after patch")
            .is_none()
    );
    assert!(
        load_redis_value(&rebuild_seen_key)
            .await
            .expect("load rebuild seen after patch")
            .is_none()
    );
    assert!(
        client
            .query_opt(
                "SELECT 1
                 FROM recommend.subject_profile_snapshot
                 WHERE subject_scope = 'organization'
                   AND org_id = $1::text::uuid",
                &[&ids.org_id],
            )
            .await
            .expect("load rebuilt subject profile")
            .is_some()
    );
    assert!(
        client
            .query_one(
                "SELECT count(*)::bigint
                 FROM recommend.cohort_popularity
                 WHERE cohort_key = $1",
                &[&format!("org:{}", ids.org_id)],
            )
            .await
            .expect("load rebuilt cohort rows")
            .get::<_, i64>(0)
            >= 1
    );
    assert!(
        client
            .query_one(
                "SELECT count(*)::bigint
                 FROM search.search_signal_aggregate
                 WHERE entity_scope IN ('product', 'seller')",
                &[],
            )
            .await
            .expect("load rebuilt signal rows")
            .get::<_, i64>(0)
            >= 1
    );
    assert!(
        client
            .query_opt(
                "SELECT 1
                 FROM recommend.entity_similarity
                 WHERE evidence_json ->> 'rebuild_source' = 'recommendation_result_item'",
                &[],
            )
            .await
            .expect("load rebuilt similarity row")
            .is_some()
    );
    assert!(
        client
            .query_opt(
                "SELECT 1
                 FROM recommend.bundle_relation
                 WHERE metadata ->> 'rebuild_source' = 'recommendation_result_item'",
                &[],
            )
            .await
            .expect("load rebuilt bundle row")
            .is_some()
    );

    let exposure_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'endpoint',
               metadata ->> 'permission_code',
               metadata ->> 'idempotency_key'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time ASC, audit_id ASC
             LIMIT 1",
            &[&exposure_request_id],
        )
        .await
        .expect("load exposure audit row");
    assert_eq!(exposure_audit_row.get::<_, String>(0), "accepted");
    assert_eq!(
        exposure_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("POST /api/v1/recommendations/track/exposure")
    );
    assert_eq!(
        exposure_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("portal.recommendation.read")
    );
    assert_eq!(
        exposure_audit_row.get::<_, Option<String>>(3).as_deref(),
        Some(exposure_idempotency_key.as_str())
    );

    let duplicate_exposure_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'idempotency_key'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&exposure_request_id],
        )
        .await
        .expect("load duplicate exposure audit row");
    assert_eq!(
        duplicate_exposure_audit_row.get::<_, String>(0),
        "deduplicated"
    );
    assert_eq!(
        duplicate_exposure_audit_row
            .get::<_, Option<String>>(1)
            .as_deref(),
        Some(exposure_idempotency_key.as_str())
    );

    let click_audit_row = client
        .query_one(
            "SELECT
               result_code,
               metadata ->> 'endpoint',
               metadata ->> 'permission_code',
               metadata ->> 'idempotency_key'
             FROM audit.audit_event
             WHERE request_id = $1
             ORDER BY event_time DESC, audit_id DESC
             LIMIT 1",
            &[&click_request_id],
        )
        .await
        .expect("load click audit row");
    assert_eq!(click_audit_row.get::<_, String>(0), "accepted");
    assert_eq!(
        click_audit_row.get::<_, Option<String>>(1).as_deref(),
        Some("POST /api/v1/recommendations/track/click")
    );
    assert_eq!(
        click_audit_row.get::<_, Option<String>>(2).as_deref(),
        Some("portal.recommendation.read")
    );
    assert_eq!(
        click_audit_row.get::<_, Option<String>>(3).as_deref(),
        Some(click_idempotency_key.as_str())
    );

    restore_ranking_profile_metadata_snapshot(
        &client,
        ranking_profile_id,
        &ranking_profile_snapshot,
    )
    .await
    .expect("restore ranking profile snapshot");
    restore_placement_config_snapshot(&client, "home_featured", &placement_snapshot)
        .await
        .expect("restore placement snapshot");
    cleanup_opensearch_documents(&ids).await;
    cleanup_graph(&client, &ids).await.expect("cleanup graph");
}

#[tokio::test(flavor = "current_thread")]
async fn recommendation_get_api_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = RECOMMEND_ENV_TEST_LOCK.lock().expect("lock recommend env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let ids = seed_graph(&client, &suffix)
        .await
        .expect("seed recommendation graph");
    seed_opensearch_documents(&ids, &suffix)
        .await
        .expect("seed recommendation opensearch docs");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "staging");

    let app = crate::with_live_test_state(router()).await;
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);
    let request_id = format!("req-recommend-get-{suffix}");
    let trace_id = format!("trace-recommend-get-{suffix}");

    let outcome: Result<(), String> = async {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=product_detail_bundle&subject_scope=organization&subject_org_id={}&context_entity_scope=product&context_entity_id={}&limit=3",
                        ids.org_id,
                        ids.product_ids[0]
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &request_id)
                    .header("x-trace-id", &trace_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build recommendation get request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call recommendation get endpoint failed: {err}"))?;
        let response_status = response.status();
        let json_body = response_json(response).await?;
        if response_status != StatusCode::OK {
            return Err(format!(
                "recommendation get unexpected status={response_status}: {json_body}"
            ));
        }
        let data = &json_body["data"];
        let items = data["items"]
            .as_array()
            .ok_or_else(|| format!("recommendation items missing: {json_body}"))?;
        if items.is_empty() {
            return Err(format!("recommendation items should not be empty: {json_body}"));
        }
        if !items.iter().any(|item| {
            item["entity_id"].as_str() == Some(ids.product_ids[1].as_str())
        }) {
            return Err(format!(
                "recommendation result missing seeded bundle target {}: {json_body}",
                ids.product_ids[1]
            ));
        }

        let recommendation_request_id = data["recommendation_request_id"]
            .as_str()
            .ok_or_else(|| format!("recommendation_request_id missing: {json_body}"))?;
        let recommendation_result_id = data["recommendation_result_id"]
            .as_str()
            .ok_or_else(|| format!("recommendation_result_id missing: {json_body}"))?;

        let request_row = client
            .query_one(
                "SELECT
                   placement_code,
                   subject_scope,
                   subject_org_id::text,
                   request_id,
                   trace_id
                 FROM recommend.recommendation_request
                 WHERE recommendation_request_id = $1::text::uuid",
                &[&recommendation_request_id],
            )
            .await
            .map_err(|err| format!("load recommendation_request failed: {err}"))?;
        if request_row.get::<_, String>(0) != "product_detail_bundle" {
            return Err("recommendation_request placement_code mismatch".to_string());
        }
        if request_row.get::<_, String>(1) != "organization" {
            return Err("recommendation_request subject_scope mismatch".to_string());
        }
        if request_row.get::<_, Option<String>>(2).as_deref() != Some(ids.org_id.as_str()) {
            return Err("recommendation_request subject_org_id mismatch".to_string());
        }
        if request_row.get::<_, Option<String>>(3).as_deref() != Some(request_id.as_str()) {
            return Err("recommendation_request request_id mismatch".to_string());
        }
        if request_row.get::<_, Option<String>>(4).as_deref() != Some(trace_id.as_str()) {
            return Err("recommendation_request trace_id mismatch".to_string());
        }

        let result_row = client
            .query_one(
                "SELECT
                   placement_code,
                   returned_count,
                   result_status
                 FROM recommend.recommendation_result
                 WHERE recommendation_result_id = $1::text::uuid",
                &[&recommendation_result_id],
            )
            .await
            .map_err(|err| format!("load recommendation_result failed: {err}"))?;
        if result_row.get::<_, String>(0) != "product_detail_bundle" {
            return Err("recommendation_result placement_code mismatch".to_string());
        }
        let returned_count: i32 = result_row.get(1);
        if returned_count < 1 {
            return Err("recommendation_result returned_count should be >= 1".to_string());
        }
        if result_row.get::<_, String>(2) != "served" {
            return Err("recommendation_result status mismatch".to_string());
        }

        let result_item_count = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM recommend.recommendation_result_item
                 WHERE recommendation_result_id = $1::text::uuid",
                &[&recommendation_result_id],
            )
            .await
            .map_err(|err| format!("count recommendation_result_item failed: {err}"))?
            .get::<_, i64>(0);
        if result_item_count < 1 {
            return Err("recommendation_result_item rows missing".to_string());
        }

        if count_access_audit(&client, &request_id, "recommendation_result")
            .await
            .map_err(|err| format!("count recommendation access audit failed: {err}"))?
            != 1
        {
            return Err("recommendation access audit missing".to_string());
        }
        if count_system_logs(
            &client,
            &request_id,
            "recommendation lookup executed: GET /api/v1/recommendations",
        )
        .await
        .map_err(|err| format!("count recommendation system log failed: {err}"))?
            != 1
        {
            return Err("recommendation system log missing".to_string());
        }

        let audit_row = client
            .query_one(
                "SELECT
                   metadata ->> 'endpoint',
                   metadata ->> 'permission_code',
                   metadata -> 'filters' ->> 'placement_code',
                   metadata -> 'filters' ->> 'recommendation_result_id'
                 FROM audit.access_audit
                 WHERE request_id = $1
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&request_id],
            )
            .await
            .map_err(|err| format!("load recommendation access audit failed: {err}"))?;
        if audit_row.get::<_, String>(0) != "GET /api/v1/recommendations" {
            return Err("recommendation access audit endpoint mismatch".to_string());
        }
        if audit_row.get::<_, String>(1) != "portal.recommendation.read" {
            return Err("recommendation access audit permission mismatch".to_string());
        }
        if audit_row.get::<_, Option<String>>(2).as_deref() != Some("product_detail_bundle") {
            return Err("recommendation access audit placement_code mismatch".to_string());
        }
        if audit_row.get::<_, Option<String>>(3).as_deref() != Some(recommendation_result_id) {
            return Err("recommendation access audit result id mismatch".to_string());
        }

        Ok(())
    }
    .await;

    cleanup_opensearch_documents(&ids).await;
    cleanup_graph(&client, &ids).await.expect("cleanup graph");

    if let Err(message) = outcome {
        panic!("{message}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn recommendation_filters_frozen_product_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = RECOMMEND_ENV_TEST_LOCK.lock().expect("lock recommend env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let ids = seed_graph(&client, &suffix)
        .await
        .expect("seed recommendation graph");
    seed_opensearch_documents(&ids, &suffix)
        .await
        .expect("seed recommendation opensearch docs");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "staging");

    let app = crate::with_live_test_state(router()).await;
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);
    let request_id_before = format!("req-recommend-freeze-before-{suffix}");
    let request_id_after = format!("req-recommend-freeze-after-{suffix}");

    let outcome: Result<(), String> = async {
        let before_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=product_detail_bundle&subject_scope=organization&subject_org_id={}&context_entity_scope=product&context_entity_id={}&limit=3",
                        ids.org_id,
                        ids.product_ids[0]
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &request_id_before)
                    .body(Body::empty())
                    .map_err(|err| format!("build pre-freeze recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call pre-freeze recommendation endpoint failed: {err}"))?;
        let before_json = response_json(before_response).await?;
        if !before_json["data"]["items"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["entity_id"].as_str() == Some(ids.product_ids[1].as_str())))
        {
            return Err(format!(
                "pre-freeze recommendation missing expected bundle product {}: {before_json}",
                ids.product_ids[1]
            ));
        }

        // Keep the OpenSearch document intact while freezing in PostgreSQL to prove final business gating.
        update_product_status_and_refresh_projection(&client, &ids.product_ids[1], "frozen")
            .await
            .map_err(|err| format!("freeze product and refresh projection failed: {err}"))?;
        let projection_row = client
            .query_one(
                "SELECT listing_status, visibility_status, visible_to_search
                 FROM search.product_search_document
                 WHERE product_id = $1::text::uuid",
                &[&ids.product_ids[1]],
            )
            .await
            .map_err(|err| format!("load search projection after freeze failed: {err}"))?;
        if projection_row.get::<_, String>(0) != "frozen"
            || projection_row.get::<_, String>(1) != "frozen"
            || projection_row.get::<_, bool>(2)
        {
            return Err(format!(
                "search projection did not flip to frozen/invisible after status update: listing_status={} visibility_status={} visible_to_search={}",
                projection_row.get::<_, String>(0),
                projection_row.get::<_, String>(1),
                projection_row.get::<_, bool>(2),
            ));
        }

        let after_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=product_detail_bundle&subject_scope=organization&subject_org_id={}&context_entity_scope=product&context_entity_id={}&limit=4",
                        ids.org_id,
                        ids.product_ids[0]
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &request_id_after)
                    .body(Body::empty())
                    .map_err(|err| format!("build post-freeze recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call post-freeze recommendation endpoint failed: {err}"))?;
        let after_json = response_json(after_response).await?;
        if after_json["data"]["items"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["entity_id"].as_str() == Some(ids.product_ids[1].as_str())))
        {
            return Err(format!(
                "frozen product should be filtered out of recommendation results: {after_json}"
            ));
        }
        let recommendation_result_id = after_json["data"]["recommendation_result_id"]
            .as_str()
            .ok_or_else(|| format!("post-freeze recommendation_result_id missing: {after_json}"))?;
        let frozen_item_count = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM recommend.recommendation_result_item
                 WHERE recommendation_result_id = $1::text::uuid
                   AND entity_id = $2::text::uuid",
                &[&recommendation_result_id, &ids.product_ids[1]],
            )
            .await
            .map_err(|err| format!("count frozen recommendation_result_item failed: {err}"))?
            .get::<_, i64>(0);
        if frozen_item_count != 0 {
            return Err(format!(
                "frozen product should not be written into recommendation_result_item rows: count={frozen_item_count}"
            ));
        }
        Ok(())
    }
    .await;

    cleanup_opensearch_documents(&ids).await;
    cleanup_graph(&client, &ids).await.expect("cleanup graph");

    if let Err(message) = outcome {
        panic!("{message}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn recommendation_local_minimal_candidate_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = RECOMMEND_ENV_TEST_LOCK.lock().expect("lock recommend env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .to_string();
    let ids = seed_graph(&client, &suffix)
        .await
        .expect("seed recommendation local graph");
    let _app_mode = ScopedEnvVar::set("APP_MODE", "local");
    let _opensearch_endpoint = ScopedEnvVar::set("OPENSEARCH_ENDPOINT", "http://127.0.0.1:1");

    let app = crate::with_live_test_state(router()).await;
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);
    let home_request_id_1 = format!("req-recommend-local-home-1-{suffix}");
    let home_request_id_2 = format!("req-recommend-local-home-2-{suffix}");
    let bundle_request_id = format!("req-recommend-local-bundle-{suffix}");
    let zero_request_id = format!("req-recommend-local-zero-{suffix}");

    let outcome: Result<(), String> = async {
        let home_response_1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=home_featured&subject_scope=organization&subject_org_id={}&limit=3",
                        ids.org_id
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &home_request_id_1)
                    .body(Body::empty())
                    .map_err(|err| format!("build first local recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call first local recommendation endpoint failed: {err}"))?;
        let home_status_1 = home_response_1.status();
        let home_json_1 = response_json(home_response_1).await?;
        if home_status_1 != StatusCode::OK {
            return Err(format!(
                "first local recommendation unexpected status={home_status_1}: {home_json_1}"
            ));
        }
        if home_json_1["data"]["cache_hit"].as_bool() != Some(false) {
            return Err(format!(
                "first local recommendation should be cache miss: {home_json_1}"
            ));
        }
        let home_items = home_json_1["data"]["items"]
            .as_array()
            .ok_or_else(|| format!("local recommendation items missing: {home_json_1}"))?;
        if home_items.is_empty() {
            return Err(format!(
                "local recommendation should return minimal candidates: {home_json_1}"
            ));
        }
        let home_request_row = client
            .query_one(
                "SELECT
                   request_attrs ->> 'candidate_backend',
                   request_attrs ->> 'runtime_mode'
                 FROM recommend.recommendation_request
                 WHERE recommendation_request_id = $1::text::uuid",
                &[&home_json_1["data"]["recommendation_request_id"]
                    .as_str()
                    .ok_or_else(|| {
                        format!("local recommendation_request_id missing: {home_json_1}")
                    })?],
            )
            .await
            .map_err(|err| format!("load local recommendation request metadata failed: {err}"))?;
        if home_request_row.get::<_, Option<String>>(0).as_deref()
            != Some("postgresql_local_minimal")
        {
            return Err("local recommendation request backend mismatch".to_string());
        }
        if home_request_row.get::<_, Option<String>>(1).as_deref() != Some("local") {
            return Err("local recommendation request runtime mismatch".to_string());
        }
        let home_result_row = client
            .query_one(
                "SELECT
                   metadata ->> 'candidate_backend',
                   metadata ->> 'runtime_mode'
                 FROM recommend.recommendation_result
                 WHERE recommendation_result_id = $1::text::uuid",
                &[&home_json_1["data"]["recommendation_result_id"]
                    .as_str()
                    .ok_or_else(|| {
                        format!("local recommendation_result_id missing: {home_json_1}")
                    })?],
            )
            .await
            .map_err(|err| format!("load local recommendation result metadata failed: {err}"))?;
        if home_result_row.get::<_, Option<String>>(0).as_deref()
            != Some("postgresql_local_minimal")
        {
            return Err("local recommendation result backend mismatch".to_string());
        }
        if home_result_row.get::<_, Option<String>>(1).as_deref() != Some("local") {
            return Err("local recommendation result runtime mismatch".to_string());
        }
        if count_access_audit(&client, &home_request_id_1, "recommendation_result")
            .await
            .map_err(|err| format!("count local recommendation access audit failed: {err}"))?
            != 1
        {
            return Err("local recommendation access audit missing".to_string());
        }
        if count_system_logs(
            &client,
            &home_request_id_1,
            "recommendation lookup executed: GET /api/v1/recommendations",
        )
        .await
        .map_err(|err| format!("count local recommendation system log failed: {err}"))?
            != 1
        {
            return Err("local recommendation system log missing".to_string());
        }

        let home_response_2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=home_featured&subject_scope=organization&subject_org_id={}&limit=3",
                        ids.org_id
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &home_request_id_2)
                    .body(Body::empty())
                    .map_err(|err| format!("build second local recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call second local recommendation endpoint failed: {err}"))?;
        let home_json_2 = response_json(home_response_2).await?;
        if home_json_2["data"]["cache_hit"].as_bool() != Some(true) {
            return Err(format!(
                "second local recommendation should be cache hit: {home_json_2}"
            ));
        }

        let bundle_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=product_detail_bundle&subject_scope=organization&subject_org_id={}&context_entity_scope=product&context_entity_id={}&limit=3",
                        ids.org_id,
                        ids.product_ids[0]
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &bundle_request_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build local bundle recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call local bundle recommendation endpoint failed: {err}"))?;
        let bundle_status = bundle_response.status();
        let bundle_json = response_json(bundle_response).await?;
        if bundle_status != StatusCode::OK {
            return Err(format!(
                "local bundle recommendation unexpected status={bundle_status}: {bundle_json}"
            ));
        }
        let bundle_items = bundle_json["data"]["items"]
            .as_array()
            .ok_or_else(|| format!("local bundle recommendation items missing: {bundle_json}"))?;
        if !bundle_items
            .iter()
            .any(|item| item["entity_id"].as_str() == Some(ids.product_ids[1].as_str()))
        {
            return Err(format!(
                "local same-seller recommendation missing service candidate {}: {bundle_json}",
                ids.product_ids[1]
            ));
        }

        let zero_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/recommendations?placement_code=search_zero_result_fallback&subject_scope=organization&subject_org_id={}&context_entity_scope=product&context_entity_id={}&limit=3",
                        ids.org_id,
                        ids.product_ids[0]
                    ))
                    .header("authorization", &buyer_auth)
                    .header("x-request-id", &zero_request_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build local zero-result recommendation request failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("call local zero-result recommendation endpoint failed: {err}"))?;
        let zero_status = zero_response.status();
        let zero_json = response_json(zero_response).await?;
        if zero_status != StatusCode::OK {
            return Err(format!(
                "local zero-result recommendation unexpected status={zero_status}: {zero_json}"
            ));
        }
        let zero_items = zero_json["data"]["items"].as_array().ok_or_else(|| {
            format!("local zero-result recommendation items missing: {zero_json}")
        })?;
        if zero_items.is_empty() {
            return Err(format!(
                "local zero-result recommendation should not be empty: {zero_json}"
            ));
        }
        if !zero_items.iter().any(|item| {
            item["explanation_codes"]
                .as_array()
                .is_some_and(|codes| codes.iter().any(|code| code.as_str() == Some("fallback:zero_result")))
        }) {
            return Err(format!(
                "local zero-result recommendation missing fallback explanation code: {zero_json}"
            ));
        }

        Ok(())
    }
    .await;

    cleanup_graph(&client, &ids).await.expect("cleanup graph");

    if let Err(message) = outcome {
        panic!("{message}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn recommendation_home_featured_standard_scenarios_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let _env_lock = RECOMMEND_ENV_TEST_LOCK.lock().expect("lock recommend env");
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let _app_mode = ScopedEnvVar::set("APP_MODE", "staging");
    crate::modules::recommendation::repo::invalidate_placement_runtime_cache("home_featured")
        .await
        .expect("invalidate home_featured runtime cache");

    let placement_row = client
        .query_one(
            "SELECT
               metadata ->> 'fixed_sample_set',
               jsonb_array_length(metadata -> 'fixed_samples')
             FROM recommend.placement_definition
             WHERE placement_code = 'home_featured'",
            &[],
        )
        .await
        .expect("load home_featured fixed sample metadata");
    assert_eq!(
        placement_row.get::<_, Option<String>>(0).as_deref(),
        Some("five_standard_scenarios_v1")
    );
    assert_eq!(placement_row.get::<_, i32>(1), 5);

    let samples = load_standard_scenario_samples(&client)
        .await
        .expect("load standard scenario samples");
    assert_eq!(samples.len(), 5);

    let app = crate::with_live_test_state(router()).await;
    let buyer_auth = authorization_header(
        "10000000-0000-0000-0000-000000000302",
        "10000000-0000-0000-0000-000000000102",
        &["buyer_operator"],
    );
    let request_id = format!(
        "req-recommend-home-standard-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/recommendations?placement_code=home_featured&subject_scope=organization&subject_org_id=10000000-0000-0000-0000-000000000102&limit=5")
                .header("authorization", &buyer_auth)
                .header("x-request-id", &request_id)
                .body(Body::empty())
                .expect("build standard scenario recommendation request"),
        )
        .await
        .expect("call standard scenario recommendation endpoint");
    let status = response.status();
    let json = response_json(response)
        .await
        .expect("decode standard scenario response");
    assert_eq!(status, StatusCode::OK, "{json}");

    let items = json["data"]["items"]
        .as_array()
        .expect("home_featured items");
    assert_eq!(items.len(), 5, "{json}");

    let actual_titles = items
        .iter()
        .map(|item| item["title"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    let expected_titles = samples
        .iter()
        .map(|sample| sample.scenario_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(actual_titles, expected_titles, "{json}");

    for (item, sample) in items.iter().zip(samples.iter()) {
        let scenario_explanation = format!("scenario:{}", sample.scenario_code);
        assert_eq!(
            item["entity_id"].as_str(),
            Some(sample.product_id.as_str()),
            "{json}"
        );
        let explanation_codes = item["explanation_codes"]
            .as_array()
            .expect("scenario explanation codes");
        assert!(
            explanation_codes
                .iter()
                .any(|code| { code.as_str() == Some("placement:fixed_sample") })
        );
        assert!(
            explanation_codes
                .iter()
                .any(|code| { code.as_str() == Some(scenario_explanation.as_str()) })
        );
    }

    let request_row = client
        .query_one(
            "SELECT
               candidate_source_summary ->> 'placement_sample',
               request_attrs ->> 'cache_hit'
             FROM recommend.recommendation_request
             WHERE recommendation_request_id = $1::text::uuid",
            &[&json["data"]["recommendation_request_id"]
                .as_str()
                .expect("recommendation_request_id")],
        )
        .await
        .expect("load standard scenario recommendation request");
    assert_eq!(
        request_row.get::<_, Option<String>>(0).as_deref(),
        Some("5")
    );
    assert_eq!(
        request_row.get::<_, Option<String>>(1).as_deref(),
        Some("false")
    );

    assert_eq!(
        count_access_audit(&client, &request_id, "recommendation_result")
            .await
            .expect("count standard scenario access audit"),
        1
    );
    assert_eq!(
        count_system_logs(
            &client,
            &request_id,
            "recommendation lookup executed: GET /api/v1/recommendations",
        )
        .await
        .expect("count standard scenario system log"),
        1
    );
}
