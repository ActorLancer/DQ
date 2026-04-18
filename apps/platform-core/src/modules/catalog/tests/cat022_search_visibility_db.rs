use crate::modules::catalog::api::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tokio_postgres::{Client, NoTls};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_id: String,
    asset_version_id: String,
}

async fn seed_minimum_graph(
    client: &Client,
    suffix: &str,
) -> Result<SeedIds, tokio_postgres::Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', jsonb_build_object('description', $2::text)
             )
             RETURNING org_id::text",
            &[
                &format!("cat022-org-{suffix}"),
                &format!("seller profile {suffix}"),
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
            &[&org_id, &format!("cat022-asset-{suffix}")],
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

    Ok(SeedIds {
        org_id,
        asset_id,
        asset_version_id,
    })
}

async fn cleanup_seed_graph(client: &Client, ids: &SeedIds, product_id: Option<&str>) {
    if let Some(product_id) = product_id {
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&product_id],
            )
            .await;
    }
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

#[tokio::test]
async fn cat022_search_visibility_fields_and_events_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .expect("connect database");
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
    let req_create = format!("req-cat022-create-{suffix}");
    let req_patch = format!("req-cat022-patch-{suffix}");
    let req_product_get = format!("req-cat022-product-get-{suffix}");
    let req_seller_get = format!("req-cat022-seller-get-{suffix}");
    let mut created_product_id: Option<String> = None;

    let outcome: Result<(), String> = async {
        let app = router();

        let create_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/products")
                    .header("content-type", "application/json")
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &req_create)
                    .body(Body::from(
                        json!({
                            "asset_id": ids.asset_id,
                            "asset_version_id": ids.asset_version_id,
                            "seller_org_id": ids.org_id,
                            "title": format!("cat022-product-{suffix}"),
                            "category": "manufacturing",
                            "product_type": "data_product",
                            "delivery_type": "file_download",
                            "searchable_text": "工业 质量 数据 商品",
                            "subtitle": "初始副标题",
                            "industry": "industrial_manufacturing",
                            "use_cases": ["质量日报", "产线巡检"],
                            "data_classification": "P1",
                            "quality_score": "0.82"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build create request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call create endpoint: {err}"))?;
        if create_resp.status() != StatusCode::OK {
            let status = create_resp.status();
            let body = to_bytes(create_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read create error body: {err}"))?;
            return Err(format!(
                "create status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let create_body = to_bytes(create_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read create response body: {err}"))?;
        let create_json: Value = serde_json::from_slice(&create_body)
            .map_err(|err| format!("decode create response json: {err}"))?;
        let product_id = create_json["data"]["product_id"]
            .as_str()
            .ok_or_else(|| "product_id missing in create response".to_string())?
            .to_string();
        created_product_id = Some(product_id.clone());

        let patch_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/products/{product_id}"))
                    .header("content-type", "application/json")
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &req_patch)
                    .body(Body::from(
                        json!({
                            "searchable_text": "工业 数据 质量 指标",
                            "subtitle": "更新后副标题",
                            "industry": "industry_iot",
                            "use_cases": ["设备稼动率", "能耗监控"],
                            "data_classification": "P2",
                            "quality_score": "0.91"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build patch request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call patch endpoint: {err}"))?;
        if patch_resp.status() != StatusCode::OK {
            let status = patch_resp.status();
            let body = to_bytes(patch_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read patch error body: {err}"))?;
            return Err(format!(
                "patch status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }

        client
            .execute(
                "INSERT INTO search.product_search_document (
                   product_id, org_id, title, document_version, index_sync_status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 6, 'indexed'
                 )
                 ON CONFLICT (product_id) DO UPDATE
                 SET document_version = EXCLUDED.document_version,
                     index_sync_status = EXCLUDED.index_sync_status,
                     updated_at = now()",
                &[
                    &product_id,
                    &ids.org_id,
                    &format!("cat022-product-doc-{suffix}"),
                ],
            )
            .await
            .map_err(|err| format!("upsert product_search_document: {err}"))?;

        client
            .execute(
                "INSERT INTO search.seller_search_document (
                   org_id, seller_name, seller_type, document_version, index_sync_status
                 ) VALUES (
                   $1::text::uuid, $2, 'enterprise', 4, 'indexed'
                 )
                 ON CONFLICT (org_id) DO UPDATE
                 SET document_version = EXCLUDED.document_version,
                     index_sync_status = EXCLUDED.index_sync_status,
                     updated_at = now()",
                &[&ids.org_id, &format!("cat022-seller-{suffix}")],
            )
            .await
            .map_err(|err| format!("upsert seller_search_document: {err}"))?;

        let product_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/products/{product_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &ids.org_id)
                    .header("x-request-id", &req_product_get)
                    .body(Body::empty())
                    .map_err(|err| format!("build product get request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call product detail endpoint: {err}"))?;
        if product_resp.status() != StatusCode::OK {
            return Err(format!(
                "product detail status mismatch: expected 200, got {}",
                product_resp.status()
            ));
        }
        let product_body = to_bytes(product_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read product detail body: {err}"))?;
        let product_json: Value = serde_json::from_slice(&product_body)
            .map_err(|err| format!("decode product detail json: {err}"))?;
        if product_json["data"]["subtitle"].as_str() != Some("更新后副标题") {
            return Err("product subtitle mismatch".to_string());
        }
        if product_json["data"]["industry"].as_str() != Some("industry_iot") {
            return Err("product industry mismatch".to_string());
        }
        if product_json["data"]["data_classification"].as_str() != Some("P2") {
            return Err("product data_classification mismatch".to_string());
        }
        if product_json["data"]["quality_score"].as_str() != Some("0.91") {
            return Err("product quality_score mismatch".to_string());
        }
        if product_json["data"]["search_document_version"].as_i64() != Some(6) {
            return Err("product search_document_version mismatch".to_string());
        }
        if product_json["data"]["index_sync_status"].as_str() != Some("indexed") {
            return Err("product index_sync_status mismatch".to_string());
        }

        let seller_resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/sellers/{}/profile", ids.org_id))
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &req_seller_get)
                    .body(Body::empty())
                    .map_err(|err| format!("build seller request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call seller profile endpoint: {err}"))?;
        if seller_resp.status() != StatusCode::OK {
            return Err(format!(
                "seller profile status mismatch: expected 200, got {}",
                seller_resp.status()
            ));
        }
        let seller_body = to_bytes(seller_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read seller profile body: {err}"))?;
        let seller_json: Value = serde_json::from_slice(&seller_body)
            .map_err(|err| format!("decode seller profile json: {err}"))?;
        if seller_json["data"]["search_document_version"].as_i64() != Some(4) {
            return Err("seller search_document_version mismatch".to_string());
        }
        if seller_json["data"]["index_sync_status"].as_str() != Some("indexed") {
            return Err("seller index_sync_status mismatch".to_string());
        }

        for request_id in [&req_create, &req_patch] {
            let row = client
                .query_one(
                    "SELECT count(*)::bigint
                     FROM ops.outbox_event
                     WHERE request_id = $1
                       AND event_type = 'search.product.changed'
                       AND ref_type = 'product'
                       AND ref_id = $2::text::uuid",
                    &[request_id, &product_id],
                )
                .await
                .map_err(|err| format!("query outbox event for {request_id}: {err}"))?;
            let count: i64 = row.get(0);
            if count < 1 {
                return Err(format!(
                    "missing outbox search.product.changed for {request_id}"
                ));
            }
        }

        let patch_audit = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'catalog.product.patch'
                   AND ref_type = 'product'
                   AND ref_id = $2::text::uuid",
                &[&req_patch, &product_id],
            )
            .await
            .map_err(|err| format!("query patch audit event: {err}"))?;
        let patch_audit_count: i64 = patch_audit.get(0);
        if patch_audit_count < 1 {
            return Err("catalog.product.patch audit event missing".to_string());
        }
        Ok(())
    }
    .await;

    cleanup_seed_graph(&client, &ids, created_product_id.as_deref()).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
