#[cfg(test)]
mod tests {
    use crate::modules::{billing, order};
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        payment_intent_id: String,
    }

    #[tokio::test]
    async fn trade024_illegal_state_regression_db_smoke() {
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
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let request_id = format!("req-trade024-{suffix}");
        let provider_event_id = format!("evt-trade024-{suffix}");
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis();

        let app = app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/webhooks/mock_payment")
                    .header("content-type", "application/json")
                    .header("x-provider-signature", "mock-signature")
                    .header("x-webhook-timestamp", timestamp_ms.to_string())
                    .header("x-request-id", &request_id)
                    .body(Body::from(format!(
                        r#"{{
                          "provider_event_id":"{}",
                          "event_type":"payment.failed",
                          "payment_intent_id":"{}",
                          "provider_status":"failed",
                          "occurred_at_ms":{}
                        }}"#,
                        provider_event_id, seed.payment_intent_id, timestamp_ms
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["processed_status"], "processed");
        assert_eq!(json["data"]["out_of_order_ignored"], false);
        assert_eq!(json["data"]["applied_payment_status"], "failed");

        let order_row = client
            .query_one(
                "SELECT status, payment_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order");
        assert_eq!(order_row.get::<_, String>(0), "seller_delivering");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(
            order_row.get::<_, Option<String>>(2).as_deref(),
            Some("trade024_seed_seller_delivering")
        );

        let payment_row = client
            .query_one(
                "SELECT status
                 FROM payment.payment_intent
                 WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await
            .expect("query payment intent");
        assert_eq!(payment_row.get::<_, String>(0), "failed");

        let audit_row = client
            .query_one(
                "SELECT action_name, result_code
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND ref_type = 'order'
                   AND ref_id = $2::text::uuid
                 ORDER BY event_time DESC
                 LIMIT 1",
                &[&request_id, &seed.order_id],
            )
            .await
            .expect("query order audit");
        assert_eq!(
            audit_row.get::<_, String>(0),
            "order.payment.result.ignored"
        );
        assert_eq!(audit_row.get::<_, String>(1), "ignored");

        cleanup_seed_graph(&client, &seed, &request_id).await;
    }

    async fn app() -> Router {
        crate::with_live_test_state(
            Router::new()
                .merge(billing::api::router())
                .merge(order::api::router()),
        )
        .await
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade024-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1::text, 'enterprise', 'active', '{}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("trade024-seller-{suffix}")],
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
                    &format!("trade024-asset-{suffix}"),
                    &format!("trade024 asset desc {suffix}"),
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
                   $5, 'listed', 'one_time', 19.90, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade024-product-{suffix}"),
                    &format!("trade024 product desc {suffix}"),
                    &format!("trade024 search text {suffix}"),
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
                &[&product_id, &format!("TRADE024-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'seller_delivering', 'paid', 'in_progress', 'not_started', 'pending_settlement', 'none',
                   'online', 19.90, 'CNY',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', 'TRADE024-SKU',
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '19.90',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776570000024',
                     'source', 'seed'
                   )::jsonb,
                   'trade024_seed_seller_delivering'
                 )
                 RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        let payment_intent_id: String = client
            .query_one(
                "INSERT INTO payment.payment_intent (
                   order_id, intent_type, provider_key, payer_subject_type, payer_subject_id,
                   payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
                   launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
                   status, request_id, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_payment', 'mock_payment', 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 19.90, 'wallet', 'CNY', 'CNY',
                   'processing', $4, '{}'::jsonb
                 )
                 RETURNING payment_intent_id::text",
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("req-trade024-intent-{suffix}"),
                ],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, request_id: &str) {
        let _ = client
            .execute(
                "DELETE FROM audit.audit_event
                 WHERE request_id = $1
                    OR ref_id = $2::text::uuid
                    OR ref_id = $3::text::uuid",
                &[&request_id, &seed.order_id, &seed.payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_webhook_event
                 WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_intent
                 WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
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
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid OR org_id = $2::text::uuid",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
