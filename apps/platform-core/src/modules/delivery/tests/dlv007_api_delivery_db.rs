#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::order::api::router as order_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
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
        asset_object_id: String,
    }

    #[tokio::test]
    async fn dlv007_api_delivery_db_smoke() {
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
        let api_sub_seed = seed_graph(&client, &suffix, "API_SUB")
            .await
            .expect("seed api_sub");
        let api_ppu_seed = seed_graph(&client, &format!("{suffix}-ppu"), "API_PPU")
            .await
            .expect("seed api_ppu");
        let app = crate::with_live_test_state(delivery_router().merge(order_router())).await;

        let sub_req_id = format!("req-dlv007-sub-{suffix}");
        let sub_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", api_sub_seed.order_id))
                    .header("x-role", "tenant_developer")
                    .header("x-tenant-id", &api_sub_seed.buyer_org_id)
                    .header("x-request-id", &sub_req_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "branch": "api",
                            "asset_object_id": api_sub_seed.asset_object_id,
                            "app_name": format!("dlv007-sub-app-{suffix}"),
                            "quota_json": {
                                "billing_mode": "subscription",
                                "period": "monthly",
                                "included_calls": 9000
                            },
                            "rate_limit_json": {
                                "requests_per_minute": 80,
                                "burst": 20,
                                "concurrency": 5
                            },
                            "upstream_mode": "platform_proxy",
                            "expire_at": "2026-08-01T00:00:00Z",
                            "delivery_commit_hash": format!("api-sub-commit-{suffix}"),
                            "receipt_hash": format!("api-sub-receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("api_sub request"),
            )
            .await
            .expect("api_sub response");
        let sub_status = sub_response.status();
        let sub_body = to_bytes(sub_response.into_body(), usize::MAX)
            .await
            .expect("api_sub body");
        assert_eq!(
            sub_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&sub_body)
        );
        let sub_json: Value = serde_json::from_slice(&sub_body).expect("api_sub json");
        assert_eq!(
            sub_json["data"]["data"]["current_state"].as_str(),
            Some("api_key_issued")
        );
        assert_eq!(sub_json["data"]["data"]["branch"].as_str(), Some("api"));
        assert!(sub_json["data"]["data"]["app_id"].as_str().is_some());
        assert!(
            sub_json["data"]["data"]["api_credential_id"]
                .as_str()
                .is_some()
        );
        assert!(sub_json["data"]["data"]["api_key"].as_str().is_some());
        assert_eq!(
            sub_json["data"]["data"]["upstream_mode"].as_str(),
            Some("platform_proxy")
        );

        let ppu_req_id = format!("req-dlv007-ppu-{suffix}");
        let ppu_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", api_ppu_seed.order_id))
                    .header("x-role", "tenant_developer")
                    .header("x-tenant-id", &api_ppu_seed.buyer_org_id)
                    .header("x-request-id", &ppu_req_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "branch": "api",
                            "asset_object_id": api_ppu_seed.asset_object_id,
                            "app_name": format!("dlv007-ppu-app-{suffix}"),
                            "rate_limit_json": {
                                "requests_per_minute": 30,
                                "burst": 10,
                                "concurrency": 3
                            },
                            "upstream_mode": "seller_gateway",
                            "expire_at": "2026-08-15T00:00:00Z",
                            "delivery_commit_hash": format!("api-ppu-commit-{suffix}"),
                            "receipt_hash": format!("api-ppu-receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("api_ppu request"),
            )
            .await
            .expect("api_ppu response");
        let ppu_status = ppu_response.status();
        let ppu_body = to_bytes(ppu_response.into_body(), usize::MAX)
            .await
            .expect("api_ppu body");
        assert_eq!(
            ppu_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&ppu_body)
        );
        let ppu_json: Value = serde_json::from_slice(&ppu_body).expect("api_ppu json");
        assert_eq!(
            ppu_json["data"]["data"]["current_state"].as_str(),
            Some("quota_ready")
        );
        assert_eq!(
            ppu_json["data"]["data"]["credential_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            ppu_json["data"]["data"]["upstream_mode"].as_str(),
            Some("seller_gateway")
        );

        let sub_order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&api_sub_seed.order_id],
            )
            .await
            .expect("query api_sub order");
        assert_eq!(sub_order_row.get::<_, String>(0), "api_key_issued");
        assert_eq!(sub_order_row.get::<_, String>(1), "in_progress");
        assert_eq!(sub_order_row.get::<_, String>(2), "not_started");
        assert_eq!(sub_order_row.get::<_, String>(3), "pending_settlement");

        let ppu_order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&api_ppu_seed.order_id],
            )
            .await
            .expect("query api_ppu order");
        assert_eq!(ppu_order_row.get::<_, String>(0), "quota_ready");
        assert_eq!(ppu_order_row.get::<_, String>(1), "in_progress");
        assert_eq!(ppu_order_row.get::<_, String>(2), "not_started");
        assert_eq!(ppu_order_row.get::<_, String>(3), "pending_settlement");

        let sub_delivery_row = client
            .query_one(
                "SELECT status, delivery_type, delivery_route, receipt_hash
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY committed_at DESC NULLS LAST, updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&api_sub_seed.order_id],
            )
            .await
            .expect("query api_sub delivery");
        assert_eq!(sub_delivery_row.get::<_, String>(0), "committed");
        assert_eq!(sub_delivery_row.get::<_, String>(1), "api_access");
        assert_eq!(
            sub_delivery_row.get::<_, Option<String>>(2).as_deref(),
            Some("platform_proxy")
        );
        assert_eq!(
            sub_delivery_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("api-sub-receipt-{suffix}").as_str())
        );

        let sub_credential_row = client
            .query_one(
                "SELECT ac.status,
                        ac.upstream_mode,
                        app.org_id::text,
                        app.app_name,
                        app.metadata -> 'rate_limit_profile'
                 FROM delivery.api_credential ac
                 JOIN core.application app ON app.app_id = ac.app_id
                 WHERE ac.order_id = $1::text::uuid
                 ORDER BY ac.created_at DESC, ac.api_credential_id DESC
                 LIMIT 1",
                &[&api_sub_seed.order_id],
            )
            .await
            .expect("query api_sub credential");
        assert_eq!(sub_credential_row.get::<_, String>(0), "active");
        assert_eq!(sub_credential_row.get::<_, String>(1), "platform_proxy");
        assert_eq!(
            sub_credential_row.get::<_, String>(2),
            api_sub_seed.buyer_org_id
        );
        assert_eq!(
            sub_credential_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("dlv007-sub-app-{suffix}").as_str())
        );
        let rate_limit: Value = sub_credential_row.get(4);
        assert_eq!(rate_limit["requests_per_minute"].as_i64(), Some(80));

        let api_enable_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.api.enable'
                   AND request_id IN ($1, $2)",
                &[&sub_req_id, &ppu_req_id],
            )
            .await
            .expect("api enable audit")
            .get(0);
        assert_eq!(api_enable_audit, 2);

        let api_sub_trade_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'trade.order.api_sub.transition'
                   AND request_id = $1",
                &[&sub_req_id],
            )
            .await
            .expect("api_sub trade audit")
            .get(0);
        assert_eq!(api_sub_trade_audit, 1);

        let api_ppu_trade_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'trade.order.api_ppu.transition'
                   AND request_id = $1",
                &[&ppu_req_id],
            )
            .await
            .expect("api_ppu trade audit")
            .get(0);
        assert_eq!(api_ppu_trade_audit, 1);

        let api_outbox_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE event_type = 'delivery.committed'
                   AND request_id IN ($1, $2)
                   AND payload ->> 'delivery_branch' = 'api'
                   AND target_topic = 'dtp.outbox.domain-events'",
                &[&sub_req_id, &ppu_req_id],
            )
            .await
            .expect("api outbox count")
            .get(0);
        assert_eq!(api_outbox_count, 2);

        cleanup_seed_graph(&client, &api_sub_seed).await;
        cleanup_seed_graph(&client, &api_ppu_seed).await;
    }

    async fn seed_graph(
        client: &Client,
        suffix: &str,
        sku_type: &str,
    ) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv007-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv007-seller-{suffix}")],
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
                    &format!("dlv007-asset-{suffix}"),
                    &format!("dlv007 asset {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false,
                   '{"api_mode":"service"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);

        let asset_object_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'api_endpoint', $2, $3,
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
                   '{"rate_limit_profile":{"requests_per_minute":50,"burst":15,"concurrency":4}}'::jsonb,
                   '{"owner":"seller"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv007-api-{suffix}"),
                    &format!("https://api.example.com/{suffix}/v1"),
                ],
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
                   $5, 'listed', 'subscription', 88.00, 'CNY', 'api_access',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv007-product-{suffix}"),
                    &format!("dlv007 product {suffix}"),
                    &format!("dlv007 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let (billing_mode, trade_mode, sku_code) = if sku_type == "API_SUB" {
            (
                "subscription",
                "subscription_access",
                format!("DLV007-SUB-{suffix}"),
            )
        } else {
            (
                "usage_based",
                "usage_access",
                format!("DLV007-PPU-{suffix}"),
            )
        };
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6,
                   'api_access', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &sku_code,
                    &sku_type,
                    &if sku_type == "API_SUB" { "月" } else { "次" },
                    &billing_mode,
                    &trade_mode,
                ],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 88.00, 'CNY',
                   jsonb_build_object('sku_type', $6, 'delivery_mode', 'api_access'),
                   'api_gateway',
                   '{"api_delivery":"platform_proxy"}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id, &sku_type],
            )
            .await?
            .get(0);

        client
            .execute(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&order_id, &format!("sha256:dlv007:{suffix}")],
            )
            .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
