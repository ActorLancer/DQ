use super::authorization_header;
use crate::modules::recommendation::api::router;
use crate::modules::recommendation::domain::{
    RECOMMENDATION_BASELINE_BEHAVIOR_EVENT_TYPES, RECOMMENDATION_BASELINE_PLACEMENTS,
    RECOMMENDATION_BASELINE_RANKING_PROFILE_KEYS,
};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("RECOMMEND_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_ids: Vec<String>,
    asset_version_ids: Vec<String>,
    product_ids: Vec<String>,
    operator_user_id: String,
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

async fn cleanup_graph(client: &Client, ids: &SeedIds) {
    let _ = client
        .execute(
            "DELETE FROM ops.outbox_event
             WHERE event_type = 'recommend.behavior_recorded'
               AND payload -> 'payload' ->> 'recommendation_request_id' IN (
                 SELECT recommendation_request_id::text FROM recommend.recommendation_request
               )",
            &[],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM recommend.behavior_event WHERE subject_org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM recommend.recommendation_request WHERE subject_org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM recommend.cohort_popularity WHERE cohort_key = $1",
            &[&format!("org:{}", ids.org_id)],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM recommend.bundle_relation
             WHERE source_entity_id = $1::text::uuid OR target_entity_id = $1::text::uuid
                OR source_entity_id = $2::text::uuid OR target_entity_id = $2::text::uuid",
            &[&ids.product_ids[0], &ids.product_ids[1]],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM search.index_sync_task
             WHERE entity_id = ANY($1::text[]::uuid[])",
            &[&vec![
                ids.org_id.clone(),
                ids.product_ids[0].clone(),
                ids.product_ids[1].clone(),
            ]],
        )
        .await;
    for product_id in &ids.product_ids {
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[product_id],
            )
            .await;
    }
    for asset_version_id in &ids.asset_version_ids {
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[asset_version_id],
            )
            .await;
    }
    for asset_id in &ids.asset_ids {
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[asset_id],
            )
            .await;
    }
    let _ = client
        .execute(
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&ids.operator_user_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await;
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

    cleanup_graph(&client, &ids).await;
}

#[tokio::test]
async fn recommendation_api_full_runtime_db_smoke() {
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
    seed_opensearch_documents(&ids, &suffix)
        .await
        .expect("seed opensearch docs");
    let buyer_auth = authorization_header(&ids.operator_user_id, &ids.org_id, &["buyer_operator"]);

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
                .header("x-role", "buyer_operator")
                .header("x-idempotency-key", format!("recommend-exposure-{suffix}"))
                .header("x-request-id", format!("recommend-req-{suffix}"))
                .body(Body::from(exposure_payload.to_string()))
                .expect("exposure request"),
        )
        .await
        .expect("exposure response");
    assert_eq!(response.status(), StatusCode::OK);

    let duplicate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/recommendations/track/exposure")
                .header("content-type", "application/json")
                .header("x-role", "buyer_operator")
                .header("x-idempotency-key", format!("recommend-exposure-{suffix}"))
                .header("x-request-id", format!("recommend-req-{suffix}"))
                .body(Body::from(exposure_payload.to_string()))
                .expect("duplicate exposure request"),
        )
        .await
        .expect("duplicate exposure response");
    assert_eq!(duplicate_response.status(), StatusCode::OK);

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
                .header("x-role", "buyer_operator")
                .header("x-idempotency-key", format!("recommend-click-{suffix}"))
                .header("x-request-id", format!("recommend-click-req-{suffix}"))
                .body(Body::from(click_payload.to_string()))
                .expect("click request"),
        )
        .await
        .expect("click response");
    assert_eq!(click_response.status(), StatusCode::OK);

    let placements_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/recommendation/placements")
                .header("x-role", "platform_admin")
                .body(Body::empty())
                .expect("placements request"),
        )
        .await
        .expect("placements response");
    assert_eq!(placements_response.status(), StatusCode::OK);

    let ranking_profiles_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/ops/recommendation/ranking-profiles")
                .header("x-role", "platform_admin")
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

    let patch_placement_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/ops/recommendation/placements/home_featured")
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-idempotency-key", format!("recommend-placement-{suffix}"))
                .header("x-step-up-token", "step-up-ok")
                .body(Body::from(
                    json!({
                      "metadata": { "smoke_suffix": suffix }
                    })
                    .to_string(),
                ))
                .expect("patch placement request"),
        )
        .await
        .expect("patch placement response");
    assert_eq!(patch_placement_response.status(), StatusCode::OK);

    let patch_ranking_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/ops/recommendation/ranking-profiles/{ranking_profile_id}"
                ))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-idempotency-key", format!("recommend-ranking-{suffix}"))
                .header("x-step-up-token", "step-up-ok")
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
    assert_eq!(patch_ranking_response.status(), StatusCode::OK);

    let rebuild_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ops/recommendation/rebuild")
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-idempotency-key", format!("recommend-rebuild-{suffix}"))
                .header("x-step-up-token", "step-up-ok")
                .body(Body::from(
                    json!({
                      "scope": "cache",
                      "subject_org_id": ids.org_id,
                      "purge_cache": true
                    })
                    .to_string(),
                ))
                .expect("rebuild request"),
        )
        .await
        .expect("rebuild response");
    assert_eq!(rebuild_response.status(), StatusCode::OK);

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
    assert!(outbox_count >= 3);

    cleanup_opensearch_documents(&ids).await;
    cleanup_graph(&client, &ids).await;
}

#[tokio::test(flavor = "current_thread")]
async fn recommendation_get_api_db_smoke() {
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
    cleanup_graph(&client, &ids).await;

    if let Err(message) = outcome {
        panic!("{message}");
    }
}
