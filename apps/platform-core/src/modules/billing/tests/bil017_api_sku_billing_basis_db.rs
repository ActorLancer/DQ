#[cfg(test)]
mod tests {
    use super::super::super::api::router as billing_router;
    use crate::modules::order::api::router as order_router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        app_id: String,
        api_credential_id: String,
    }

    #[tokio::test]
    async fn bil017_api_sku_billing_basis_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil017-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil017-seller-{suffix}")).await;
        let api_sub = seed_api_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "sub",
            "API_SUB",
            "active",
            "subscription",
            "subscription",
            json!({
                "period": "monthly",
                "included_calls": 1000,
                "overage_policy": "metered"
            }),
        )
        .await;
        let api_ppu = seed_api_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &suffix,
            "ppu",
            "API_PPU",
            "quota_ready",
            "usage_metered",
            "api_ppu",
            json!({
                "period": "per_call",
                "included_calls": 0,
                "overage_policy": "per_call"
            }),
        )
        .await;
        seed_usage_logs(&client, &api_ppu, &suffix).await;

        let app = crate::with_live_test_state(billing_router().merge(order_router())).await;

        let sub_first = transition_api_sub(
            &app,
            &api_sub.order_id,
            &api_sub.buyer_org_id,
            &format!("req-bil017-sub-{suffix}"),
            &json!({
                "action": "bill_cycle",
                "billing_cycle_code": "2026-04",
                "billing_amount": "66.00000000",
                "reason_note": "monthly subscription billing"
            }),
        )
        .await;
        assert_eq!(
            sub_first["data"]["billing_event_type"].as_str(),
            Some("recurring_charge")
        );
        assert_eq!(
            sub_first["data"]["billing_event_replayed"].as_bool(),
            Some(false)
        );
        let sub_event_id = sub_first["data"]["billing_event_id"]
            .as_str()
            .expect("sub event id")
            .to_string();

        let sub_replay = transition_api_sub(
            &app,
            &api_sub.order_id,
            &api_sub.buyer_org_id,
            &format!("req-bil017-sub-replay-{suffix}"),
            &json!({
                "action": "bill_cycle",
                "billing_cycle_code": "2026-04",
                "billing_amount": "66.00000000",
                "reason_note": "monthly subscription billing replay"
            }),
        )
        .await;
        assert_eq!(
            sub_replay["data"]["billing_event_id"].as_str(),
            Some(sub_event_id.as_str())
        );
        assert_eq!(
            sub_replay["data"]["billing_event_replayed"].as_bool(),
            Some(true)
        );

        let ppu_request_id = format!("req-bil017-ppu-{suffix}");
        let ppu_first = transition_api_ppu(
            &app,
            &api_ppu.order_id,
            &api_ppu.buyer_org_id,
            &ppu_request_id,
            &json!({
                "action": "settle_success_call",
                "billing_amount": "12.50000000",
                "usage_units": "128",
                "meter_window_code": "2026-04-20T10",
                "reason_note": "successful call batch"
            }),
        )
        .await;
        assert_eq!(
            ppu_first["data"]["billing_event_type"].as_str(),
            Some("usage_charge")
        );
        assert_eq!(
            ppu_first["data"]["billing_event_replayed"].as_bool(),
            Some(false)
        );
        let ppu_event_id = ppu_first["data"]["billing_event_id"]
            .as_str()
            .expect("ppu event id")
            .to_string();

        let ppu_replay = transition_api_ppu(
            &app,
            &api_ppu.order_id,
            &api_ppu.buyer_org_id,
            &ppu_request_id,
            &json!({
                "action": "settle_success_call",
                "billing_amount": "12.50000000",
                "usage_units": "128",
                "meter_window_code": "2026-04-20T10",
                "reason_note": "successful call batch replay"
            }),
        )
        .await;
        assert_eq!(
            ppu_replay["data"]["billing_event_id"].as_str(),
            Some(ppu_event_id.as_str())
        );
        assert_eq!(
            ppu_replay["data"]["billing_event_replayed"].as_bool(),
            Some(true)
        );

        let sub_billing = get_billing_order(
            &app,
            &api_sub.order_id,
            &api_sub.buyer_org_id,
            &suffix,
            "sub",
        )
        .await;
        assert_eq!(
            sub_billing["data"]["api_billing_basis"]["sku_type"].as_str(),
            Some("API_SUB")
        );
        assert_eq!(
            sub_billing["data"]["api_billing_basis"]["base_event_type"].as_str(),
            Some("recurring_charge")
        );
        assert_eq!(
            sub_billing["data"]["api_billing_basis"]["cycle_period"].as_str(),
            Some("monthly")
        );
        assert_eq!(
            sub_billing["data"]["api_billing_basis"]["included_units"].as_str(),
            Some("1000")
        );
        assert_eq!(
            sub_billing["data"]["billing_events"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );

        let ppu_billing = get_billing_order(
            &app,
            &api_ppu.order_id,
            &api_ppu.buyer_org_id,
            &suffix,
            "ppu",
        )
        .await;
        assert_eq!(
            ppu_billing["data"]["api_billing_basis"]["sku_type"].as_str(),
            Some("API_PPU")
        );
        assert_eq!(
            ppu_billing["data"]["api_billing_basis"]["usage_event_type"].as_str(),
            Some("usage_charge")
        );
        assert_eq!(
            ppu_billing["data"]["api_billing_basis"]["cycle_period"].as_str(),
            Some("per_call")
        );
        assert_eq!(
            ppu_billing["data"]["api_billing_basis"]["latest_usage_call_count"].as_str(),
            Some("2")
        );
        assert_eq!(
            ppu_billing["data"]["api_billing_basis"]["latest_usage_units"].as_str(),
            Some("128.00000000")
        );
        assert_eq!(
            ppu_billing["data"]["billing_events"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );

        let recurring_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'recurring_charge'",
                &[&api_sub.order_id],
            )
            .await
            .expect("count recurring")
            .get(0);
        assert_eq!(recurring_count, 1);
        let usage_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'usage_charge'",
                &[&api_ppu.order_id],
            )
            .await
            .expect("count usage")
            .get(0);
        assert_eq!(usage_count, 1);

        let recurring_outbox: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_id = $1::text::uuid AND target_topic = 'dtp.outbox.domain-events'",
                &[&sub_event_id],
            )
            .await
            .expect("count recurring outbox")
            .get(0);
        assert_eq!(recurring_outbox, 1);
        let usage_outbox: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_id = $1::text::uuid AND target_topic = 'dtp.outbox.domain-events'",
                &[&ppu_event_id],
            )
            .await
            .expect("count usage outbox")
            .get(0);
        assert_eq!(usage_outbox, 1);

        cleanup_seed_graph(&client, &api_sub).await;
        cleanup_seed_graph(&client, &api_ppu).await;
        cleanup_orgs(&client, &buyer_org_id, &seller_org_id).await;
    }

    async fn transition_api_sub(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
        payload: &Value,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/api-sub/transition"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("api sub transition request"),
            )
            .await
            .expect("api sub transition response");
        json_response(response, StatusCode::OK).await
    }

    async fn transition_api_ppu(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
        payload: &Value,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/api-ppu/transition"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("api ppu transition request"),
            )
            .await
            .expect("api ppu transition response");
        json_response(response, StatusCode::OK).await
    }

    async fn get_billing_order(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        suffix: &str,
        scenario: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header(
                        "x-request-id",
                        format!("req-bil017-billing-{scenario}-{suffix}"),
                    )
                    .body(Body::empty())
                    .expect("billing request"),
            )
            .await
            .expect("billing response");
        json_response(response, StatusCode::OK).await
    }

    async fn json_response(response: axum::response::Response, expected: StatusCode) -> Value {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        assert_eq!(status, expected, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).expect("json body")
    }

    async fn seed_org(client: &Client, org_name: &str) -> String {
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

    async fn seed_api_order(
        client: &Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
        scenario: &str,
        sku_type: &str,
        status: &str,
        pricing_mode: &str,
        billing_mode: &str,
        quota_json: Value,
    ) -> SeedOrderGraph {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil017-asset-{scenario}-{suffix}"),
                    &format!("bil017 asset {scenario} {suffix}"),
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
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'api_access',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil017-product-{scenario}-{suffix}"),
                    &format!("bil017 product {scenario} {suffix}"),
                    &format!("bil017 summary {scenario} {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, 'auto_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("BIL017-{scenario}-{suffix}"),
                    &sku_type,
                    &if sku_type == "API_SUB" { "月" } else { "次" },
                    &billing_mode,
                ],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   jsonb_build_object(
                     'pricing_mode', $7,
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'sku_type', $8,
                     'selected_sku_type', $8,
                     'pricing_mode', $7,
                     'billing_mode', $9,
                     'settlement_basis', CASE WHEN $8 = 'API_SUB' THEN 'subscription_cycle' ELSE 'usage' END,
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD',
                     'source', 'bil017-db-smoke'
                   ),
                   'bil017_seed'
                 )
                 RETURNING order_id::text"#,
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &status,
                    &pricing_mode,
                    &sku_type,
                    &billing_mode,
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        let app_id: String = client
            .query_one(
                "INSERT INTO core.application (
                   org_id, app_name, app_type, status, client_id, metadata
                 ) VALUES (
                   $1::text::uuid, $2, 'api_client', 'active', $3, '{}'::jsonb
                 ) RETURNING app_id::text",
                &[
                    &buyer_org_id,
                    &format!("bil017-app-{scenario}-{suffix}"),
                    &format!("bil017-client-{scenario}-{suffix}"),
                ],
            )
            .await
            .expect("insert app")
            .get(0);
        let api_credential_id: String = client
            .query_one(
                "INSERT INTO delivery.api_credential (
                   order_id, app_id, api_key_hash, upstream_mode, quota_json, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'platform_proxy', $4::jsonb, 'active'
                 ) RETURNING api_credential_id::text",
                &[
                    &order_id,
                    &app_id,
                    &format!("hash:bil017:{scenario}:{suffix}"),
                    &quota_json,
                ],
            )
            .await
            .expect("insert api credential")
            .get(0);

        SeedOrderGraph {
            buyer_org_id: buyer_org_id.to_string(),
            seller_org_id: seller_org_id.to_string(),
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            app_id,
            api_credential_id,
        }
    }

    async fn seed_usage_logs(client: &Client, seed: &SeedOrderGraph, suffix: &str) {
        client
            .execute(
                "INSERT INTO delivery.api_usage_log (
                   api_credential_id, order_id, app_id, request_id, response_code, usage_units, occurred_at
                 ) VALUES
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 200, 64.00000000, '2026-04-20T10:00:00Z'::timestamptz),
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $5, 200, 64.00000000, '2026-04-20T10:05:00Z'::timestamptz),
                   ($1::text::uuid, $2::text::uuid, $3::text::uuid, $6, 429, 1.00000000, '2026-04-20T10:10:00Z'::timestamptz)",
                &[
                    &seed.api_credential_id,
                    &seed.order_id,
                    &seed.app_id,
                    &format!("bil017-success-a-{suffix}"),
                    &format!("bil017-success-b-{suffix}"),
                    &format!("bil017-failed-{suffix}"),
                ],
            )
            .await
            .expect("insert usage logs");
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrderGraph) {
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id IN (
                   SELECT billing_event_id FROM billing.billing_event WHERE order_id = $1::text::uuid
                 )",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.api_usage_log WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.api_credential WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.application WHERE app_id = $1::text::uuid",
                &[&seed.app_id],
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
    }

    async fn cleanup_orgs(client: &Client, buyer_org_id: &str, seller_org_id: &str) {
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&buyer_org_id, &seller_org_id],
            )
            .await;
    }
}
