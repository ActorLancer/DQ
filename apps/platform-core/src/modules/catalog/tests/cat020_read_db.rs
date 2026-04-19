use crate::modules::catalog::api::router;
use crate::modules::catalog::repository::PostgresCatalogRepository;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::Value;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
}

async fn seed_product_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', jsonb_build_object('description', $2::text)
             )
             RETURNING org_id::text",
            &[
                &format!("cat020-org-{suffix}"),
                &format!("seller profile {suffix}"),
            ],
        )
        .await?;
    let org_id: String = org.get(0);

    let asset = client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status, description
             ) VALUES (
               $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
             )
             RETURNING asset_id::text",
            &[
                &org_id,
                &format!("cat020-asset-{suffix}"),
                &format!("asset desc {suffix}"),
            ],
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
               2048, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
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
               description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
               $5, 'listed', 'one_time', 10, 'CNY', 'file_download', ARRAY['internal_use']::text[], $6, '{}'::jsonb
             )
             RETURNING product_id::text",
            &[
                &asset_id,
                &asset_version_id,
                &org_id,
                &format!("cat020-product-{suffix}"),
                &format!("product desc {suffix}"),
                &format!("search text {suffix}"),
            ],
        )
        .await?;
    let product_id: String = product.get(0);

    client
        .query_one(
            "INSERT INTO catalog.product_sku (
               product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
             ) VALUES (
               $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
             )
             RETURNING sku_id::text",
            &[&product_id, &format!("CAT020-SKU-{suffix}")],
        )
        .await?;

    Ok(SeedIds {
        org_id,
        asset_id,
        asset_version_id,
        product_id,
    })
}

async fn cleanup_product_graph(client: &Client, ids: &SeedIds) {
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

#[tokio::test]
async fn cat020_read_endpoints_db_smoke() {
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
    let ids = seed_product_graph(&client, &suffix)
        .await
        .expect("seed product graph");
    let product_request_id = format!("req-cat020-product-{suffix}");
    let seller_request_id = format!("req-cat020-seller-{suffix}");

    let outcome: Result<(), String> = async {
        PostgresCatalogRepository::get_product_detail(&client, &ids.product_id)
            .await
            .map_err(|err| format!("repository product detail query failed: {err:?}"))?;
        PostgresCatalogRepository::list_product_skus(&client, &ids.product_id)
            .await
            .map_err(|err| format!("repository product sku list query failed: {err:?}"))?;

        let app = crate::with_live_test_state(router()).await;

        let product_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/products/{}", ids.product_id))
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &product_request_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build product request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call product detail endpoint: {err}"))?;
        if product_resp.status() != StatusCode::OK {
            let status = product_resp.status();
            let body = to_bytes(product_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read product error body: {err}"))?;
            return Err(format!(
                "product detail status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let product_body = to_bytes(product_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read product response body: {err}"))?;
        let product_json: Value = serde_json::from_slice(&product_body)
            .map_err(|err| format!("decode product response json: {err}"))?;
        let returned_product_id = product_json["data"]["product_id"]
            .as_str()
            .ok_or_else(|| "product_id missing in product response".to_string())?;
        if returned_product_id != ids.product_id {
            return Err(format!(
                "product_id mismatch: expected {}, got {}",
                ids.product_id, returned_product_id
            ));
        }

        let seller_resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/sellers/{}/profile", ids.org_id))
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &seller_request_id)
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
            .map_err(|err| format!("read seller response body: {err}"))?;
        let seller_json: Value = serde_json::from_slice(&seller_body)
            .map_err(|err| format!("decode seller response json: {err}"))?;
        let returned_org_id = seller_json["data"]["org_id"]
            .as_str()
            .ok_or_else(|| "org_id missing in seller response".to_string())?;
        if returned_org_id != ids.org_id {
            return Err(format!(
                "org_id mismatch: expected {}, got {}",
                ids.org_id, returned_org_id
            ));
        }

        let product_audit = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'catalog.product.read'
                   AND ref_type = 'product'
                   AND ref_id = $2::text::uuid",
                &[&product_request_id, &ids.product_id],
            )
            .await
            .map_err(|err| format!("query product audit event: {err}"))?;
        let product_audit_count: i64 = product_audit.get(0);
        if product_audit_count < 1 {
            return Err("catalog.product.read audit event not found".to_string());
        }

        let seller_audit = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'catalog.seller.profile.read'
                   AND ref_type = 'seller'
                   AND ref_id = $2::text::uuid",
                &[&seller_request_id, &ids.org_id],
            )
            .await
            .map_err(|err| format!("query seller audit event: {err}"))?;
        let seller_audit_count: i64 = seller_audit.get(0);
        if seller_audit_count < 1 {
            return Err("catalog.seller.profile.read audit event not found".to_string());
        }
        Ok(())
    }
    .await;

    cleanup_product_graph(&client, &ids).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
