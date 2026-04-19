#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        share_sku_id: String,
        api_sku_id: String,
        policy_id: String,
        order_cancel_id: String,
        order_expire_id: String,
        order_dispute_id: String,
        order_risk_id: String,
    }

    #[tokio::test]
    async fn trade018_auto_cutoff_db_smoke() {
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
        let app = crate::with_live_test_state(router()).await;

        for order_id in [
            &seed.order_cancel_id,
            &seed.order_expire_id,
            &seed.order_dispute_id,
            &seed.order_risk_id,
        ] {
            let grant_req_id = format!("req-trade018-{suffix}-grant-{order_id}");
            let grant_resp = call(
                &app,
                format!("/api/v1/orders/{order_id}/authorization/transition"),
                &seed.buyer_org_id,
                &grant_req_id,
                r#"{"action":"grant"}"#,
            )
            .await;
            assert_eq!(
                grant_resp.status,
                StatusCode::OK,
                "grant: {}",
                grant_resp.body
            );
        }

        let cancel_req_id = format!("req-trade018-{suffix}-cancel");
        let cancel_resp = call(
            &app,
            format!("/api/v1/orders/{}/cancel", seed.order_cancel_id),
            &seed.buyer_org_id,
            &cancel_req_id,
            "",
        )
        .await;
        assert_eq!(
            cancel_resp.status,
            StatusCode::OK,
            "cancel: {}",
            cancel_resp.body
        );

        let expire_req_id = format!("req-trade018-{suffix}-expire");
        let expire_resp = call(
            &app,
            format!(
                "/api/v1/orders/{}/share-ro/transition",
                seed.order_expire_id
            ),
            &seed.buyer_org_id,
            &expire_req_id,
            r#"{"action":"expire_share"}"#,
        )
        .await;
        assert_eq!(
            expire_resp.status,
            StatusCode::OK,
            "expire: {}",
            expire_resp.body
        );

        let dispute_req_id = format!("req-trade018-{suffix}-dispute");
        let dispute_resp = call(
            &app,
            format!(
                "/api/v1/orders/{}/share-ro/transition",
                seed.order_dispute_id
            ),
            &seed.buyer_org_id,
            &dispute_req_id,
            r#"{"action":"interrupt_dispute"}"#,
        )
        .await;
        assert_eq!(
            dispute_resp.status,
            StatusCode::OK,
            "dispute: {}",
            dispute_resp.body
        );

        let risk_req_id = format!("req-trade018-{suffix}-risk");
        let risk_resp = call(
            &app,
            format!("/api/v1/orders/{}/api-ppu/transition", seed.order_risk_id),
            &seed.buyer_org_id,
            &risk_req_id,
            r#"{"action":"disable_access"}"#,
        )
        .await;
        assert_eq!(risk_resp.status, StatusCode::OK, "risk: {}", risk_resp.body);

        assert_order_auth_status(&client, &seed.order_cancel_id, "revoked").await;
        assert_order_auth_status(&client, &seed.order_expire_id, "expired").await;
        assert_order_auth_status(&client, &seed.order_dispute_id, "suspended").await;
        assert_order_auth_status(&client, &seed.order_risk_id, "suspended").await;

        assert_audit_count(
            &client,
            &cancel_req_id,
            "trade.authorization.auto_cutoff.revoked",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &expire_req_id,
            "trade.authorization.auto_cutoff.expired",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &dispute_req_id,
            "trade.authorization.auto_cutoff.suspended",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &risk_req_id,
            "trade.authorization.auto_cutoff.suspended",
            1,
        )
        .await;

        cleanup_seed_graph(&client, &seed).await;
    }

    struct CallResponse {
        status: StatusCode,
        body: String,
    }

    async fn call(
        app: &axum::Router,
        uri: String,
        buyer_org_id: &str,
        request_id: &str,
        payload: &str,
    ) -> CallResponse {
        let method = if payload.is_empty() { "POST" } else { "POST" };
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-role", "buyer_operator")
            .header("x-tenant-id", buyer_org_id)
            .header("x-request-id", request_id)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .expect("request");
        let resp = app.clone().oneshot(req).await.expect("response");
        let status = resp.status();
        let body = to_bytes(resp.into_body(), usize::MAX).await.expect("body");
        CallResponse {
            status,
            body: String::from_utf8_lossy(&body).to_string(),
        }
    }

    async fn assert_order_auth_status(client: &Client, order_id: &str, expected: &str) {
        let row = client
            .query_one(
                "SELECT status FROM trade.authorization_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&order_id],
            )
            .await
            .expect("query authorization");
        assert_eq!(row.get::<_, String>(0), expected);
    }

    async fn assert_audit_count(client: &Client, request_id: &str, action_name: &str, min: i64) {
        let count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM audit.audit_event
                 WHERE request_id = $1 AND action_name = $2",
                &[&request_id, &action_name],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(count >= min, "missing audit: {action_name}");
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade018-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade018-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'draft', $3
                 ) RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade018-asset-{suffix}"),
                    &format!("trade018 asset {suffix}"),
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
                   2048, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text",
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 288.00, 'CNY', 'share_grant',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade018-product-{suffix}"),
                    &format!("trade018 product {suffix}"),
                    &format!("trade018 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let share_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("TRADE018-SHARE-{suffix}")],
            )
            .await?
            .get(0);

        let api_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'API_PPU', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("TRADE018-API-{suffix}")],
            )
            .await?
            .get(0);

        let policy_id: String = client
            .query_one(
                r#"INSERT INTO contract.usage_policy (
                   owner_org_id, policy_name, stage_from,
                   subject_constraints, usage_constraints, time_constraints,
                   region_constraints, output_constraints, exportable, status
                 ) VALUES (
                   $1::text::uuid, $2, 'V1',
                   '{"principal_type":"org"}'::jsonb,
                   '{"allowed_usage":["internal_use"]}'::jsonb,
                   '{"ttl_days":30}'::jsonb,
                   '{"allow_regions":["CN"]}'::jsonb,
                   '{"allow_export":false}'::jsonb,
                   false,
                   'active'
                 ) RETURNING policy_id::text"#,
                &[&seller_org_id, &format!("TRADE018-POL-{suffix}")],
            )
            .await?
            .get(0);

        let _: String = client
            .query_one(
                "INSERT INTO contract.policy_binding (policy_id, product_id, binding_scope)
                 VALUES ($1::text::uuid, $2::text::uuid, 'product')
                 RETURNING policy_binding_id::text",
                &[&policy_id, &product_id],
            )
            .await?
            .get(0);

        let order_cancel_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'unpaid', 'pending_delivery', 'not_started', 'not_started', 'none',
                   'online', 288.00, 'CNY', '{}'::jsonb
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &share_sku_id],
            )
            .await?
            .get(0);

        let order_expire_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'shared_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 288.00, 'CNY', '{}'::jsonb
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &share_sku_id],
            )
            .await?
            .get(0);

        let order_dispute_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'shared_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 288.00, 'CNY', '{}'::jsonb
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &share_sku_id],
            )
            .await?
            .get(0);

        let order_risk_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'usage_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 288.00, 'CNY', '{}'::jsonb
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &api_sku_id],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            share_sku_id,
            api_sku_id,
            policy_id,
            order_cancel_id,
            order_expire_id,
            order_dispute_id,
            order_risk_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.order_cancel_id.clone(),
                    seed.order_expire_id.clone(),
                    seed.order_dispute_id.clone(),
                    seed.order_risk_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.order_cancel_id.clone(),
                    seed.order_expire_id.clone(),
                    seed.order_dispute_id.clone(),
                    seed.order_risk_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.policy_binding WHERE policy_id = $1::text::uuid",
                &[&seed.policy_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.usage_policy WHERE policy_id = $1::text::uuid",
                &[&seed.policy_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.share_sku_id.clone(), seed.api_sku_id.clone()]],
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
