use crate::modules::search::api::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("SEARCH_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
}

async fn seed_minimum_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', jsonb_build_object('description', $2::text)
             )
             RETURNING org_id::text",
            &[
                &format!("search-org-{suffix}"),
                &format!("search seller {suffix}"),
            ],
        )
        .await?;
    let org_id: String = org.get(0);

    let asset = client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status
             ) VALUES (
               $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
             )
             RETURNING asset_id::text",
            &[&org_id, &format!("search-asset-{suffix}")],
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
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
               $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
               ARRAY['internal_use']::text[], $6,
               jsonb_build_object(
                 'subtitle', $7,
                 'industry', 'industrial_manufacturing',
                 'quality_score', '0.92',
                 'use_cases', jsonb_build_array('质量日报')
               )
             )
             RETURNING product_id::text",
            &[
                &asset_id,
                &asset_version_id,
                &org_id,
                &format!("search-product-{suffix}"),
                &format!("search product {suffix}"),
                &format!("quality search keyword {suffix}"),
                &format!("search subtitle {suffix}"),
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
    client
        .execute(
            "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
            &[&org_id],
        )
        .await?;

    Ok(SeedIds {
        org_id,
        asset_id,
        asset_version_id,
        product_id,
    })
}

async fn cleanup_seed_graph(client: &Client, ids: &SeedIds) {
    let _ = client
        .execute(
            "DELETE FROM search.index_sync_task WHERE entity_id IN ($1::text::uuid, $2::text::uuid)",
            &[&ids.product_id, &ids.org_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
            &[&ids.product_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
            &[&ids.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
            &[&ids.asset_id],
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
    let product_doc = json!({
        "entity_scope": "product",
        "id": ids.product_id,
        "seller_id": ids.org_id,
        "name": format!("search-product-{suffix}"),
        "title": format!("search-product-{suffix}"),
        "subtitle": format!("search subtitle {suffix}"),
        "description": format!("quality search keyword {suffix}"),
        "industry": "industrial_manufacturing",
        "status": "listed",
        "price_amount": 88.0,
        "quality_score": 0.92,
        "seller_reputation_score": 0.0,
        "hotness_score": 0.0,
        "updated_at": "2026-04-21T00:00:00.000Z"
    });
    let seller_doc = json!({
        "entity_scope": "seller",
        "id": ids.org_id,
        "name": format!("search-org-{suffix}"),
        "description": format!("search seller {suffix}"),
        "reputation_score": 0.0,
        "listing_product_count": 1,
        "updated_at": "2026-04-21T00:00:00.000Z"
    });

    let product_resp = client
        .put(format!(
            "{}/product_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.product_id
        ))
        .json(&product_doc)
        .send()
        .await
        .map_err(|err| format!("seed product opensearch doc failed: {err}"))?;
    if !product_resp.status().is_success() {
        return Err(format!(
            "seed product opensearch doc status={}",
            product_resp.status()
        ));
    }

    let seller_resp = client
        .put(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .json(&seller_doc)
        .send()
        .await
        .map_err(|err| format!("seed seller opensearch doc failed: {err}"))?;
    if !seller_resp.status().is_success() {
        return Err(format!(
            "seed seller opensearch doc status={}",
            seller_resp.status()
        ));
    }

    Ok(())
}

async fn cleanup_opensearch_documents(ids: &SeedIds) {
    let endpoint = std::env::var("OPENSEARCH_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());
    let client = reqwest::Client::new();
    let _ = client
        .delete(format!(
            "{}/product_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.product_id
        ))
        .send()
        .await;
    let _ = client
        .delete(format!(
            "{}/seller_search_write/_doc/{}?refresh=wait_for",
            endpoint.trim_end_matches('/'),
            ids.org_id
        ))
        .send()
        .await;
}

#[tokio::test]
async fn search_api_and_reindex_ops_db_smoke() {
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
    let ids = seed_minimum_graph(&client, &suffix)
        .await
        .expect("seed minimum graph");
    let app = crate::with_live_test_state(router()).await;
    let req_search = format!("req-search-api-{suffix}");
    let req_reindex = format!("req-search-reindex-{suffix}");

    let outcome: Result<(), String> = async {
        seed_opensearch_documents(&ids, &suffix).await?;

        let search_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/catalog/search?q={}&entity_scope=product&page=1&page_size=10",
                        suffix
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-request-id", &req_search)
                    .body(Body::empty())
                    .map_err(|err| format!("build search request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call search endpoint: {err}"))?;
        if search_resp.status() != StatusCode::OK {
            let status = search_resp.status();
            let body = to_bytes(search_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read search response body: {err}"))?;
            return Err(format!(
                "search status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let search_body = to_bytes(search_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read search success body: {err}"))?;
        let search_json: Value = serde_json::from_slice(&search_body)
            .map_err(|err| format!("decode search body: {err}"))?;
        let items = search_json["data"]["items"]
            .as_array()
            .ok_or_else(|| "search items missing".to_string())?;
        if !items
            .iter()
            .any(|item| item["entity_id"].as_str() == Some(ids.product_id.as_str()))
        {
            return Err("search result missing seeded product".to_string());
        }

        let reindex_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ops/search/reindex")
                    .header("content-type", "application/json")
                    .header("x-role", "platform_admin")
                    .header("x-request-id", &req_reindex)
                    .header("x-idempotency-key", format!("idem-search-reindex-{suffix}"))
                    .header("x-step-up-token", "search-reindex-stepup")
                    .body(Body::from(
                        json!({
                            "entity_scope": "product",
                            "entity_id": ids.product_id,
                            "mode": "single",
                            "force": true
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build reindex request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call reindex endpoint: {err}"))?;
        if reindex_resp.status() != StatusCode::OK {
            let status = reindex_resp.status();
            let body = to_bytes(reindex_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read reindex response body: {err}"))?;
            return Err(format!(
                "reindex status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }

        let queued = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM search.index_sync_task
                 WHERE entity_scope = 'product'
                   AND entity_id = $1::text::uuid
                   AND sync_status = 'queued'",
                &[&ids.product_id],
            )
            .await
            .map_err(|err| format!("query queued reindex task failed: {err}"))?;
        let queued_count: i64 = queued.get(0);
        if queued_count < 1 {
            return Err("queued reindex task missing".to_string());
        }

        Ok(())
    }
    .await;

    cleanup_opensearch_documents(&ids).await;
    cleanup_seed_graph(&client, &ids).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
