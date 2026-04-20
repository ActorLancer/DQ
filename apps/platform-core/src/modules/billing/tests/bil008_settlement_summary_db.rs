#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        settlement_id: String,
    }

    #[tokio::test]
    async fn bil008_settlement_summary_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
            .to_string();
        let buyer_org_id = seed_org(&client, &format!("bil008-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil008-seller-{suffix}")).await;
        let order =
            seed_order_with_settlement(&client, &buyer_org_id, &seller_org_id, &suffix).await;

        let app = crate::with_live_test_state(router()).await;
        let request_id = format!("req-bil008-read-{suffix}");
        let detail = get_billing_order(&app, &order.order_id, &buyer_org_id, &request_id).await;
        assert_eq!(
            detail["data"]["settlement_summary"]["gross_amount"].as_str(),
            Some("88.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["platform_commission_amount"].as_str(),
            Some("2.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["channel_fee_amount"].as_str(),
            Some("1.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["refund_adjustment_amount"].as_str(),
            Some("5.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["compensation_adjustment_amount"].as_str(),
            Some("3.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["supplier_receivable_amount"].as_str(),
            Some("85.00000000")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["proof_commit_state"].as_str(),
            Some("pending_anchor")
        );

        cleanup(&client, &buyer_org_id, &seller_org_id, &order).await;
    }

    async fn get_billing_order(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .body(Body::empty())
                    .expect("billing request should build"),
            )
            .await
            .expect("billing response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("billing body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("billing json")
    }

    async fn seed_org(client: &db::Client, org_name: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn seed_order_with_settlement(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> SeedOrderGraph {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil008-asset-{suffix}"),
                    &format!("bil008 asset {suffix}"),
                ],
            )
            .await
            .expect("insert asset")
            .get(0);
        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   2048, 'SG', ARRAY['SG']::text[], false,
                   '{"payment_mode":"online"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await
            .expect("insert asset version")
            .get(0);
        let product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_download',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil008-product-{suffix}"),
                    &format!("bil008 product {suffix}"),
                    &format!("bil008 summary {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL008-SKU-{suffix}");
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &sku_code],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   jsonb_build_object('pricing_mode', 'one_time', 'platform_fee_amount', '2.00', 'channel_fee_amount', '1.00', 'payable_total_amount', '88.00', 'currency_code', 'SGD'),
                   jsonb_build_object('pricing_mode', 'one_time', 'billing_mode', 'one_time', 'settlement_basis', 'one_time', 'price_currency_code', 'USD', 'currency_code', 'SGD'),
                   $6
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("bil008-order-{suffix}"),
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        let settlement_id: String = client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id, settlement_type, settlement_status, settlement_mode,
                   payable_amount, platform_fee_amount, channel_fee_amount, net_receivable_amount,
                   refund_amount, compensation_amount, reason_code, settled_at
                 ) VALUES (
                   $1::text::uuid, 'order_settlement', 'pending', 'manual',
                   88.00, 2.00, 1.00, 85.00,
                   5.00, 3.00, 'bil008_summary', now()
                 )
                 RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);
        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            settlement_id,
        }
    }

    async fn cleanup(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        order: &SeedOrderGraph,
    ) {
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE settlement_id = $1::text::uuid",
                &[&order.settlement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&order.sku_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&order.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&order.asset_version_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&order.asset_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&buyer_org_id, &seller_org_id],
            )
            .await;
    }
}
