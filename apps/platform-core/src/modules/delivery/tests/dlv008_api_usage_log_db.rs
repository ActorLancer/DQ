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
    async fn dlv008_api_usage_log_db_smoke() {
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

        let enable_request_id = format!("req-dlv008-enable-{suffix}");
        let enable_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", seed.order_id))
                    .header("x-role", "tenant_developer")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &enable_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "branch": "api",
                            "asset_object_id": seed.asset_object_id,
                            "app_name": format!("dlv008-app-{suffix}"),
                            "rate_limit_json": {
                                "requests_per_minute": 64,
                                "burst": 16,
                                "concurrency": 4
                            },
                            "expire_at": "2026-09-01T00:00:00Z",
                            "delivery_commit_hash": format!("dlv008-commit-{suffix}"),
                            "receipt_hash": format!("dlv008-receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("enable request"),
            )
            .await
            .expect("enable response");
        let enable_status = enable_response.status();
        let enable_body = to_bytes(enable_response.into_body(), usize::MAX)
            .await
            .expect("enable body");
        assert_eq!(
            enable_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&enable_body)
        );
        let enable_json: Value = serde_json::from_slice(&enable_body).expect("enable json");
        let app_id = enable_json["data"]["data"]["app_id"]
            .as_str()
            .expect("app id")
            .to_string();
        let api_credential_id = enable_json["data"]["data"]["api_credential_id"]
            .as_str()
            .expect("api credential id")
            .to_string();

        client
            .execute(
                "INSERT INTO delivery.api_usage_log (
                   api_credential_id,
                   order_id,
                   app_id,
                   request_id,
                   response_code,
                   usage_units,
                   occurred_at
                 ) VALUES
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 200, 1.25000000, '2026-04-20T10:00:00Z'::timestamptz),
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $5, 429, 2.00000000, '2026-04-20T10:05:00Z'::timestamptz),
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $6, 500, 1.50000000, '2026-04-20T10:10:00Z'::timestamptz)",
                &[
                    &api_credential_id,
                    &seed.order_id,
                    &app_id,
                    &format!("usage-success-{suffix}-abcd"),
                    &format!("usage-rate-limit-{suffix}-efgh"),
                    &format!("usage-failure-{suffix}-ijkl"),
                ],
            )
            .await
            .expect("insert usage logs");

        let get_request_id = format!("req-dlv008-get-{suffix}");
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/usage-log", seed.order_id))
                    .header("x-role", "tenant_developer")
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
            get_json["data"]["data"]["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            get_json["data"]["data"]["sku_type"].as_str(),
            Some("API_SUB")
        );
        assert_eq!(
            get_json["data"]["data"]["app"]["app_id"].as_str(),
            Some(app_id.as_str())
        );
        assert_eq!(
            get_json["data"]["data"]["summary"]["total_calls"].as_i64(),
            Some(3)
        );
        assert_eq!(
            get_json["data"]["data"]["summary"]["successful_calls"].as_i64(),
            Some(1)
        );
        assert_eq!(
            get_json["data"]["data"]["summary"]["failed_calls"].as_i64(),
            Some(2)
        );
        assert_eq!(
            get_json["data"]["data"]["summary"]["total_usage_units"].as_str(),
            Some("4.75000000")
        );
        assert_eq!(
            get_json["data"]["data"]["logs"].as_array().map(Vec::len),
            Some(3)
        );
        assert_eq!(
            get_json["data"]["data"]["logs"][0]["response_class"].as_str(),
            Some("5xx")
        );
        assert_eq!(
            get_json["data"]["data"]["logs"][0]["request_ref"]
                .as_str()
                .map(|value| value.starts_with("***")),
            Some(true)
        );

        let forbidden_request_id = format!("req-dlv008-forbidden-{suffix}");
        let forbidden_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/usage-log", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &forbidden_request_id)
                    .body(Body::empty())
                    .expect("forbidden request"),
            )
            .await
            .expect("forbidden response");
        let forbidden_status = forbidden_response.status();
        let forbidden_body = to_bytes(forbidden_response.into_body(), usize::MAX)
            .await
            .expect("forbidden body");
        assert_eq!(
            forbidden_status,
            StatusCode::FORBIDDEN,
            "{}",
            String::from_utf8_lossy(&forbidden_body)
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.api.log.read'
                   AND request_id = $1",
                &[&get_request_id],
            )
            .await
            .expect("usage log audit")
            .get(0);
        assert_eq!(audit_count, 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv008-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv008-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'api', 'low', 'active', 'dlv008 api asset'
                 )
                 RETURNING asset_id::text",
                &[&seller_org_id, &format!("dlv008-asset-{suffix}")],
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
                   '{"rate_limit_profile":{"requests_per_minute":64,"burst":16,"concurrency":4}}'::jsonb,
                   '{"owner":"seller"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv008-api-{suffix}"),
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
                    &format!("dlv008-product-{suffix}"),
                    &format!("dlv008 product {suffix}"),
                    &format!("dlv008 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, acceptance_mode, refund_mode, status
                ) VALUES (
                   $1::text::uuid, $2, 'API_SUB', '月', 'subscription', 'subscription_access',
                   'api_access', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV008-API-SUB-{suffix}")],
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
                   jsonb_build_object('sku_type', 'API_SUB', 'delivery_mode', 'api_access'),
                   'api_gateway',
                   '{"api_delivery":"platform_proxy"}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
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
                &[&order_id, &format!("sha256:dlv008:{suffix}")],
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
        client
            .execute(
                "DELETE FROM delivery.api_usage_log WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("cleanup api usage log");
        client
            .execute(
                "DELETE FROM delivery.api_credential WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("cleanup api credential");
        client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("cleanup delivery record");
        client
            .execute(
                "DELETE FROM core.application WHERE org_id = $1::text::uuid",
                &[&seed.buyer_org_id],
            )
            .await
            .expect("cleanup application");
        client
            .execute(
                "DELETE FROM contract.digital_contract WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("cleanup digital contract");
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("cleanup order");
        client
            .execute(
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
            )
            .await
            .expect("cleanup asset object");
        client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
            )
            .await
            .expect("cleanup sku");
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .expect("cleanup product");
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await
            .expect("cleanup asset version");
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await
            .expect("cleanup asset");
        client
            .execute(
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await
            .expect("cleanup organizations");
    }
}
