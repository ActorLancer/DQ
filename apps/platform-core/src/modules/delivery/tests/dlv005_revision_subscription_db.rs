#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::order::api::router as order_router;
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
    }

    #[tokio::test]
    async fn dlv005_revision_subscription_db_smoke() {
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
        let app = crate::with_live_test_state(delivery_router().merge(order_router())).await;

        let create_request_id = format!("req-dlv005-create-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/subscriptions", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"cadence":"monthly","delivery_channel":"file_ticket","start_version_no":12,"metadata":{"source":"dlv005-smoke"}}"#,
                    ))
                    .expect("create request"),
            )
            .await
            .expect("create response");
        let create_status = create_response.status();
        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create body");
        assert_eq!(
            create_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_body)
        );
        let create_json: Value = serde_json::from_slice(&create_body).expect("create json");
        assert_eq!(
            create_json["data"]["data"]["operation"].as_str(),
            Some("created")
        );
        assert_eq!(
            create_json["data"]["data"]["subscription_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            create_json["data"]["data"]["current_state"].as_str(),
            Some("buyer_locked")
        );

        let get_request_id = format!("req-dlv005-get-{suffix}");
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/subscriptions", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &get_request_id)
                    .body(Body::empty())
                    .expect("get request"),
            )
            .await
            .expect("get response");
        let get_status = get_response.status();
        let get_body = to_bytes(get_response.into_body(), usize::MAX)
            .await
            .expect("get body");
        assert_eq!(
            get_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&get_body)
        );
        let get_json: Value = serde_json::from_slice(&get_body).expect("get json");
        assert_eq!(
            get_json["data"]["data"]["subscription_status"].as_str(),
            Some("active")
        );

        let pause_request_id = format!("req-dlv005-pause-{suffix}");
        let pause_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-sub/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &pause_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"pause_subscription"}"#))
                    .expect("pause request"),
            )
            .await
            .expect("pause response");
        assert_eq!(pause_response.status(), StatusCode::OK);

        let paused_get_request_id = format!("req-dlv005-get-paused-{suffix}");
        let paused_get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/subscriptions", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &paused_get_request_id)
                    .body(Body::empty())
                    .expect("paused get request"),
            )
            .await
            .expect("paused get response");
        let paused_get_body = to_bytes(paused_get_response.into_body(), usize::MAX)
            .await
            .expect("paused get body");
        let paused_get_json: Value =
            serde_json::from_slice(&paused_get_body).expect("paused get json");
        assert_eq!(
            paused_get_json["data"]["data"]["subscription_status"].as_str(),
            Some("paused")
        );
        assert_eq!(
            paused_get_json["data"]["data"]["current_state"].as_str(),
            Some("paused")
        );

        let renew_request_id = format!("req-dlv005-renew-{suffix}");
        let renew_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/subscriptions", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &renew_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"cadence":"quarterly","delivery_channel":"file_ticket","start_version_no":12,"metadata":{"renewal":"Q2"}}"#,
                    ))
                    .expect("renew request"),
            )
            .await
            .expect("renew response");
        let renew_status = renew_response.status();
        let renew_body = to_bytes(renew_response.into_body(), usize::MAX)
            .await
            .expect("renew body");
        assert_eq!(
            renew_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&renew_body)
        );
        let renew_json: Value = serde_json::from_slice(&renew_body).expect("renew json");
        assert_eq!(
            renew_json["data"]["data"]["operation"].as_str(),
            Some("renewed")
        );
        assert_eq!(
            renew_json["data"]["data"]["cadence"].as_str(),
            Some("quarterly")
        );
        assert_eq!(
            renew_json["data"]["data"]["subscription_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            renew_json["data"]["data"]["current_state"].as_str(),
            Some("buyer_locked")
        );

        let db_row = client
            .query_one(
                "SELECT cadence,
                        delivery_channel,
                        start_version_no,
                        last_delivered_version_no,
                        subscription_status,
                        metadata ->> 'renewal'
                 FROM delivery.revision_subscription
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query subscription row");
        assert_eq!(db_row.get::<_, String>(0), "quarterly");
        assert_eq!(db_row.get::<_, String>(1), "file_ticket");
        assert_eq!(db_row.get::<_, i32>(2), 12);
        assert_eq!(db_row.get::<_, Option<i32>>(3), None);
        assert_eq!(db_row.get::<_, String>(4), "active");
        assert_eq!(db_row.get::<_, Option<String>>(5).as_deref(), Some("Q2"));

        let order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order row");
        assert_eq!(order_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "pending_delivery");

        let manage_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.subscription.manage'
                   AND request_id IN ($1, $2)",
                &[&create_request_id, &renew_request_id],
            )
            .await
            .expect("manage audit count")
            .get(0);
        assert_eq!(manage_audit_count, 2);

        let read_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.subscription.read'
                   AND request_id IN ($1, $2)",
                &[&get_request_id, &paused_get_request_id],
            )
            .await
            .expect("read audit count")
            .get(0);
        assert_eq!(read_audit_count, 2);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv005-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv005-seller-{suffix}")],
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
                    &format!("dlv005-asset-{suffix}"),
                    &format!("dlv005 asset {suffix}"),
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
                   $1::text::uuid, 12, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   4096, 'CN', ARRAY['CN']::text[], false,
                   '{}'::jsonb, 'active'
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
                   $5, 'listed', 'subscription', 188.00, 'CNY', 'revision_subscription',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","subscription_cadence":"monthly"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv005-product-{suffix}"),
                    &format!("dlv005 product {suffix}"),
                    &format!("dlv005 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_SUB', '期', 'subscription', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV005-SKU-{suffix}")],
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
                   'buyer_locked', 'paid', 'online', 188.00, 'CNY',
                   'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'FILE_SUB',
                     'pricing_mode', 'subscription',
                     'unit_price', '188.00',
                     'currency_code', 'CNY',
                     'billing_mode', 'subscription',
                     'refund_mode', 'manual_refund',
                     'delivery_route', 'revision_subscription',
                     'subscription_cadence', 'monthly',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'subscription_cycle', 'settlement_mode', 'manual_v1')
                   ),
                   '{}'::jsonb,
                   'file_ticket'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("DLV005-SKU-{suffix}"),
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
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM audit.audit_event WHERE ref_id = $1::text::uuid OR ref_id = $2::text::uuid",
                &[&seed.order_id, &seed.order_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
