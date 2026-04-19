#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tokio_postgres::{Client, NoTls};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        policy_id: String,
        order_expire_id: String,
        order_revoke_id: String,
    }

    #[tokio::test]
    async fn trade017_authorization_aggregate_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
            .await
            .expect("connect db");
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

        let app = router();

        let grant_req_id = format!("req-trade017-{suffix}-grant");
        let grant_resp = transition(
            &app,
            &seed.order_expire_id,
            &seed.buyer_org_id,
            &grant_req_id,
            r#"{
                "action":"grant"
            }"#,
        )
        .await;
        assert_eq!(
            grant_resp.status,
            StatusCode::OK,
            "grant resp: {}",
            grant_resp.json
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["current_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["policy_id"].as_str(),
            Some(seed.policy_id.as_str())
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["authorization_model"]["scope"]["order_id"].as_str(),
            Some(seed.order_expire_id.as_str())
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["authorization_model"]["resource"]["sku_id"].as_str(),
            Some(seed.sku_id.as_str())
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["authorization_model"]["subject"]["subject_id"]
                .as_str(),
            Some(seed.buyer_org_id.as_str())
        );
        assert_eq!(
            grant_resp.json["data"]["data"]["authorization_model"]["action"]["grant_type"].as_str(),
            Some("share_grant")
        );

        let suspend_req_id = format!("req-trade017-{suffix}-suspend");
        let suspend_resp = transition(
            &app,
            &seed.order_expire_id,
            &seed.buyer_org_id,
            &suspend_req_id,
            r#"{"action":"suspend"}"#,
        )
        .await;
        assert_eq!(
            suspend_resp.status,
            StatusCode::OK,
            "suspend resp: {}",
            suspend_resp.json
        );
        assert_eq!(
            suspend_resp.json["data"]["data"]["previous_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            suspend_resp.json["data"]["data"]["current_status"].as_str(),
            Some("suspended")
        );

        let recover_req_id = format!("req-trade017-{suffix}-recover");
        let recover_resp = transition(
            &app,
            &seed.order_expire_id,
            &seed.buyer_org_id,
            &recover_req_id,
            r#"{"action":"recover"}"#,
        )
        .await;
        assert_eq!(
            recover_resp.status,
            StatusCode::OK,
            "recover resp: {}",
            recover_resp.json
        );
        assert_eq!(
            recover_resp.json["data"]["data"]["current_status"].as_str(),
            Some("active")
        );

        let expire_req_id = format!("req-trade017-{suffix}-expire");
        let expire_resp = transition(
            &app,
            &seed.order_expire_id,
            &seed.buyer_org_id,
            &expire_req_id,
            r#"{"action":"expire"}"#,
        )
        .await;
        assert_eq!(
            expire_resp.status,
            StatusCode::OK,
            "expire resp: {}",
            expire_resp.json
        );
        assert_eq!(
            expire_resp.json["data"]["data"]["current_status"].as_str(),
            Some("expired")
        );

        let grant_revoke_req_id = format!("req-trade017-{suffix}-grant-revoke");
        let grant_revoke_resp = transition(
            &app,
            &seed.order_revoke_id,
            &seed.buyer_org_id,
            &grant_revoke_req_id,
            r#"{"action":"grant"}"#,
        )
        .await;
        assert_eq!(
            grant_revoke_resp.status,
            StatusCode::OK,
            "grant revoke resp: {}",
            grant_revoke_resp.json
        );

        let revoke_req_id = format!("req-trade017-{suffix}-revoke");
        let revoke_resp = transition(
            &app,
            &seed.order_revoke_id,
            &seed.buyer_org_id,
            &revoke_req_id,
            r#"{"action":"revoke"}"#,
        )
        .await;
        assert_eq!(
            revoke_resp.status,
            StatusCode::OK,
            "revoke resp: {}",
            revoke_resp.json
        );
        assert_eq!(
            revoke_resp.json["data"]["data"]["current_status"].as_str(),
            Some("revoked")
        );

        let row_expire = client
            .query_one(
                "SELECT status, valid_to IS NOT NULL
                 FROM trade.authorization_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&seed.order_expire_id],
            )
            .await
            .expect("query expire auth");
        assert_eq!(row_expire.get::<_, String>(0), "expired");
        assert!(row_expire.get::<_, bool>(1));

        let row_revoke = client
            .query_one(
                "SELECT status, valid_to IS NOT NULL
                 FROM trade.authorization_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&seed.order_revoke_id],
            )
            .await
            .expect("query revoke auth");
        assert_eq!(row_revoke.get::<_, String>(0), "revoked");
        assert!(row_revoke.get::<_, bool>(1));

        let order_policy_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid
                   AND policy_id = $2::text::uuid",
                &[&seed.order_expire_id, &seed.policy_id],
            )
            .await
            .expect("query order policy")
            .get(0);
        assert_eq!(order_policy_count, 1);

        for (request_id, action_name) in [
            (grant_req_id.as_str(), "trade.authorization.grant"),
            (suspend_req_id.as_str(), "trade.authorization.suspend"),
            (recover_req_id.as_str(), "trade.authorization.recover"),
            (expire_req_id.as_str(), "trade.authorization.expire"),
            (grant_revoke_req_id.as_str(), "trade.authorization.grant"),
            (revoke_req_id.as_str(), "trade.authorization.revoke"),
        ] {
            let audit_count: i64 = client
                .query_one(
                    "SELECT COUNT(*)::bigint
                     FROM audit.audit_event
                     WHERE request_id = $1
                       AND action_name = $2",
                    &[&request_id, &action_name],
                )
                .await
                .expect("query audit")
                .get(0);
            assert!(audit_count >= 1, "missing audit for {action_name}");
        }

        cleanup_seed_graph(&client, &seed).await;
    }

    struct TransitionResponse {
        status: StatusCode,
        json: Value,
    }

    async fn transition(
        app: &axum::Router,
        order_id: &str,
        buyer_org_id: &str,
        request_id: &str,
        payload: &str,
    ) -> TransitionResponse {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{order_id}/authorization/transition"
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", buyer_org_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| {
            serde_json::json!({
                "raw": String::from_utf8_lossy(&body).to_string()
            })
        });
        TransitionResponse { status, json }
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade017-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade017-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade017-asset-{suffix}"),
                    &format!("trade017 asset {suffix}"),
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 199.00, 'CNY', 'share_grant',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade017-product-{suffix}"),
                    &format!("trade017 product {suffix}"),
                    &format!("trade017 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE017-SKU-{suffix}")],
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
                 )
                 RETURNING policy_id::text"#,
                &[&seller_org_id, &format!("TRADE017-POL-{suffix}")],
            )
            .await?
            .get(0);

        let _: String = client
            .query_one(
                "INSERT INTO contract.policy_binding (
                   policy_id, product_id, sku_id, binding_scope
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, 'sku'
                 )
                 RETURNING policy_binding_id::text",
                &[&policy_id, &product_id, &sku_id],
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
                   'online', 199.00, 'CNY', '{}'::jsonb
                 )
                 RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        let order_revoke_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'shared_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 199.00, 'CNY', '{}'::jsonb
                 )
                 RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
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
            policy_id,
            order_expire_id,
            order_revoke_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.order_expire_id.clone(),
                    seed.order_revoke_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.order_expire_id.clone(),
                    seed.order_revoke_id.clone(),
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
