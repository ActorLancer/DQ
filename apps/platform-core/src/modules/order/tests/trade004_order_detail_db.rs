#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedOrder {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn trade004_order_detail_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_millis()
        );
        let seed = seed_order_graph(&client, &suffix)
            .await
            .expect("seed order graph");
        let request_id = format!("req-trade004-{suffix}");

        let app = crate::with_live_test_state(router()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(
            json["data"]["data"]["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["current_state"].as_str(),
            Some("created")
        );
        assert_eq!(
            json["data"]["data"]["payment_status"].as_str(),
            Some("unpaid")
        );
        assert_eq!(
            json["data"]["data"]["delivery_status"].as_str(),
            Some("pending_delivery")
        );
        assert_eq!(
            json["data"]["data"]["acceptance_status"].as_str(),
            Some("not_started")
        );
        assert_eq!(
            json["data"]["data"]["settlement_status"].as_str(),
            Some("not_started")
        );
        assert_eq!(
            json["data"]["data"]["dispute_status"].as_str(),
            Some("none")
        );
        assert!(json["data"]["data"]["relations"]["contract"].is_null());
        assert_eq!(
            json["data"]["data"]["relations"]["authorizations"]
                .as_array()
                .map(|items| items.len()),
            Some(0)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["deliveries"]
                .as_array()
                .map(|items| items.len()),
            Some(0)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["billing_events"]
                .as_array()
                .map(|items| items.len()),
            Some(0)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"]
                .as_array()
                .map(|items| items.len()),
            Some(0)
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.read'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_order_graph(client: &Client, suffix: &str) -> Result<SeedOrder, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade004-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade004-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade004-asset-{suffix}"),
                    &format!("trade004 asset {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id],
            )
            .await?
            .get(0);

        let product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 18.80, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade004-product-{suffix}"),
                    &format!("trade004 product {suffix}"),
                    &format!("trade004 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE004-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'unpaid', 'online', 18.80, 'CNY',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', 'TRADE004-SKU',
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '18.80',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776510000000',
                     'source', 'seed'
                   )::jsonb
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);

        Ok(SeedOrder {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.buyer_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await;
    }
}
