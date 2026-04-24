#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use redis::AsyncCommands;
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
        storage_namespace_id: String,
        object_id: String,
        envelope_id: String,
        delivery_id: String,
        ticket_id: String,
    }

    #[tokio::test]
    async fn dlv003_download_ticket_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis()
            .to_string();
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let request_id = format!("req-dlv003-{suffix}");
        let redis_key = format!("datab:v1:download-ticket:{}", seed.ticket_id);
        let app = crate::with_live_test_state(delivery_router()).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/download-ticket", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("download ticket response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("download ticket json");
        assert_eq!(
            json["data"]["ticket_id"].as_str(),
            Some(seed.ticket_id.as_str())
        );
        assert_eq!(
            json["data"]["bucket_name"].as_str(),
            Some("delivery-objects")
        );
        assert_eq!(
            json["data"]["object_key"].as_str(),
            Some(format!("orders/{suffix}/payload.enc").as_str())
        );
        assert_eq!(json["data"]["download_limit"].as_i64(), Some(5));
        assert_eq!(json["data"]["download_count"].as_i64(), Some(2));
        assert_eq!(json["data"]["remaining_downloads"].as_i64(), Some(3));
        let issued_token = json["data"]["download_token"]
            .as_str()
            .expect("download token")
            .to_string();

        let ticket_row = client
            .query_one(
                "SELECT token_hash, download_limit, download_count, status
                 FROM delivery.delivery_ticket
                 WHERE ticket_id = $1::text::uuid",
                &[&seed.ticket_id],
            )
            .await
            .expect("query ticket row");
        let token_hash: String = ticket_row.get(0);
        assert_ne!(token_hash, format!("ticket-{suffix}"));
        assert_eq!(ticket_row.get::<_, i32>(1), 5);
        assert_eq!(ticket_row.get::<_, i32>(2), 2);
        assert_eq!(ticket_row.get::<_, String>(3), "active");

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.file.download'
                   AND request_id = $1",
                &[&request_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert_eq!(audit_count, 1);

        let mut redis_conn = redis_connection().await;
        let cached: String = redis_conn.get(&redis_key).await.expect("redis get");
        let cached_json: Value = serde_json::from_str(&cached).expect("redis json");
        assert_eq!(
            cached_json["ticket_id"].as_str(),
            Some(seed.ticket_id.as_str())
        );
        assert_eq!(
            cached_json["download_token"].as_str(),
            Some(issued_token.as_str())
        );
        assert_eq!(cached_json["remaining_downloads"].as_i64(), Some(3));
        let _: () = redis_conn.del(&redis_key).await.expect("redis cleanup");

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn redis_connection() -> redis::aio::MultiplexedConnection {
        let client = redis::Client::open("redis://:datab_redis_pass@127.0.0.1:6379/3")
            .expect("redis client");
        client
            .get_multiplexed_async_connection()
            .await
            .expect("redis connection")
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv003-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv003-seller-{suffix}")],
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
                    &format!("dlv003-asset-{suffix}"),
                    &format!("dlv003 asset {suffix}"),
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
                   2048, 'CN', ARRAY['CN']::text[], false,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $2),
                   'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id, &format!("wmk-{suffix}")],
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
                   $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv003-product-{suffix}"),
                    &format!("dlv003 product {suffix}"),
                    &format!("dlv003 search {suffix}"),
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
                &[&product_id, &format!("DLV003-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'delivered', 'paid', 'online', 88.00, 'CNY',
                   'delivered', 'pending_acceptance', 'pending_settlement', 'none',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '88.00',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776510000000',
                     'source', 'seed'
                   )::jsonb,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $7),
                   'signed_url'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("DLV003-SKU-{suffix}"),
                    &format!("wmk-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let storage_namespace_id: String = client
            .query_one(
                "INSERT INTO catalog.storage_namespace (
                   owner_org_id, namespace_name, provider_type, namespace_kind, bucket_name, prefix_rule, status
                 ) VALUES (
                   $1::text::uuid, $2, 's3_compatible', 'product', 'delivery-objects', 'orders/{order_id}', 'active'
                 )
                 RETURNING storage_namespace_id::text",
                &[&seller_org_id, &format!("dlv003-ns-{suffix}")],
            )
            .await?
            .get(0);

        let object_id: String = client
            .query_one(
                "INSERT INTO delivery.storage_object (
                   org_id, object_type, object_uri, location_type, managed_by_org_id,
                   content_type, size_bytes, content_hash, encryption_algo, plaintext_visible_to_platform,
                   storage_namespace_id, storage_zone, storage_class
                 ) VALUES (
                   $1::text::uuid, 'delivery_object', $2, 'platform_object_storage', $1::text::uuid,
                   'application/octet-stream', 1024, $3, 'AES-GCM', false,
                   $4::text::uuid, 'delivery', 'standard'
                 )
                 RETURNING object_id::text",
                &[
                    &seller_org_id,
                    &format!("s3://delivery-objects/orders/{suffix}/payload.enc"),
                    &format!("sha256:content:{suffix}"),
                    &storage_namespace_id,
                ],
            )
            .await?
            .get(0);

        let envelope_id: String = client
            .query_one(
                "INSERT INTO delivery.key_envelope (
                   order_id, recipient_type, recipient_id, key_cipher, key_control_mode, unwrap_policy_json, key_version
                 ) VALUES (
                   $1::text::uuid, 'organization', $2::text::uuid, $3, 'seller_managed',
                   jsonb_build_object('kms', 'local-mock', 'buyer_org_id', $2::text), 'v1'
                 )
                 RETURNING envelope_id::text",
                &[&order_id, &buyer_org_id, &format!("cipher-{suffix}")],
            )
            .await?
            .get(0);

        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, object_id, delivery_type, delivery_route, status, delivery_commit_hash, envelope_id,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, 'file_download', 'signed_url', 'committed', $3, $4::text::uuid,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $5),
                   $6, NOW() - INTERVAL '1 hour', NOW() + INTERVAL '7 days', 'standard', 'not_required'
                 )
                 RETURNING delivery_id::text",
                &[
                    &order_id,
                    &object_id,
                    &format!("commit-{suffix}"),
                    &envelope_id,
                    &format!("wmk-{suffix}"),
                    &format!("receipt-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let ticket_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_ticket (
                   order_id, buyer_org_id, token_hash, expire_at, download_limit, download_count, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, NOW() + INTERVAL '6 days', 5, 2, 'active'
                 )
                 RETURNING ticket_id::text",
                &[&order_id, &buyer_org_id, &format!("ticket-{suffix}")],
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
            storage_namespace_id,
            object_id,
            envelope_id,
            delivery_id,
            ticket_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.storage_object WHERE object_id = $1::text::uuid",
                &[&seed.object_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await
            .ok();
    }
}
