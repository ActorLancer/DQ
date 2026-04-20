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
    }

    #[tokio::test]
    async fn bil007_billing_read_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil007-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil007-seller-{suffix}")).await;
        let outsider_org_id = seed_org(&client, &format!("bil007-outsider-{suffix}")).await;
        let order = seed_order(&client, &buyer_org_id, &seller_org_id, &suffix).await;
        seed_billing_records(&client, &order.order_id, &buyer_org_id, &suffix).await;

        let app = crate::with_live_test_state(router()).await;
        let request_id = format!("req-bil007-read-{suffix}");
        let detail = get_billing_order(&app, &order.order_id, &buyer_org_id, &request_id).await;
        assert_eq!(
            detail["data"]["order_id"].as_str(),
            Some(order.order_id.as_str())
        );
        assert_eq!(
            detail["data"]["billing_events"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            detail["data"]["settlements"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(detail["data"]["refunds"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            detail["data"]["compensations"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(detail["data"]["invoices"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            detail["data"]["tax_placeholder"]["tax_engine_status"].as_str(),
            Some("placeholder")
        );
        assert_eq!(
            detail["data"]["invoice_placeholder"]["invoice_mode"].as_str(),
            Some("manual_placeholder")
        );
        assert_eq!(
            detail["data"]["invoice_placeholder"]["invoice_required"].as_bool(),
            Some(true)
        );

        let forbidden = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{}", order.order_id))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", outsider_org_id.as_str())
                    .header("x-request-id", format!("req-bil007-forbidden-{suffix}"))
                    .body(Body::empty())
                    .expect("forbidden request should build"),
            )
            .await
            .expect("forbidden response");
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1 AND action_name = 'billing.order.read'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert_eq!(audit_count, 1);

        cleanup(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &outsider_org_id,
            &order,
        )
        .await;
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

    async fn seed_order(
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
                    &format!("bil007-asset-{suffix}"),
                    &format!("bil007 asset {suffix}"),
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
                    &format!("bil007-product-{suffix}"),
                    &format!("bil007 product {suffix}"),
                    &format!("bil007 summary {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL007-SKU-{suffix}");
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
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'billing_mode', 'one_time',
                     'settlement_basis', 'one_time',
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD'
                   ),
                   $6
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("bil007-order-{suffix}"),
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn seed_billing_records(
        client: &db::Client,
        order_id: &str,
        buyer_org_id: &str,
        suffix: &str,
    ) {
        client
            .execute(
                "INSERT INTO billing.billing_event (
                   order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
                 ) VALUES (
                   $1::text::uuid, 'one_time_charge', 'payment_webhook', 88.00, 'SGD', 1, now(),
                   jsonb_build_object('idempotency_key', $2, 'source', 'bil007-db-smoke')
                 )",
                &[&order_id, &format!("bil007:event:{suffix}")],
            )
            .await
            .expect("insert billing event");
        let settlement_id: String = client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id, settlement_type, settlement_status, settlement_mode,
                   payable_amount, platform_fee_amount, channel_fee_amount, net_receivable_amount,
                   refund_amount, compensation_amount, reason_code, settled_at
                 ) VALUES (
                   $1::text::uuid, 'order_settlement', 'pending', 'manual',
                   88.00, 2.00, 1.00, 85.00,
                   5.00, 3.00, 'bil007_pending_review', now()
                 )
                 RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);
        client
            .execute(
                "INSERT INTO billing.refund_record (
                   order_id, amount, currency_code, status, executed_at
                 ) VALUES (
                   $1::text::uuid, 5.00, 'SGD', 'processing', now()
                 )",
                &[&order_id],
            )
            .await
            .expect("insert refund");
        client
            .execute(
                "INSERT INTO billing.compensation_record (
                   order_id, amount, currency_code, status, executed_at
                 ) VALUES (
                   $1::text::uuid, 3.00, 'SGD', 'pending', now()
                 )",
                &[&order_id],
            )
            .await
            .expect("insert compensation");
        client
            .execute(
                "INSERT INTO billing.invoice_request (
                   order_id, settlement_id, requester_org_id, invoice_title, tax_no, amount, currency_code, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'TAX-BIL007', 88.00, 'SGD', 'pending'
                 )",
                &[
                    &order_id,
                    &settlement_id,
                    &buyer_org_id,
                    &format!("bil007-invoice-{suffix}"),
                ],
            )
            .await
            .expect("insert invoice request");
    }

    async fn cleanup(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        outsider_org_id: &str,
        order: &SeedOrderGraph,
    ) {
        let _ = client
            .execute(
                "DELETE FROM billing.invoice_request WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.compensation_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.refund_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&order.order_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid, $3::text::uuid)",
                &[&buyer_org_id, &seller_org_id, &outsider_org_id],
            )
            .await;
    }
}
