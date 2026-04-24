#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        signer_user_id: String,
        file_asset_id: String,
        file_asset_version_id: String,
        file_product_id: String,
        file_sku_id: String,
        file_contract_template_id: String,
        share_asset_id: String,
        share_asset_version_id: String,
        share_product_id: String,
        share_sku_id: String,
        share_policy_id: String,
    }

    #[tokio::test]
    async fn trade027_main_trade_flow_db_smoke() {
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

        let file_order = create_order(
            &app,
            &seed.buyer_org_id,
            &seed.file_product_id,
            &seed.file_sku_id,
            &format!("req-trade027-{suffix}-file-create"),
            &format!("idem-trade027-{suffix}-file"),
        )
        .await;
        assert_eq!(file_order.status, StatusCode::OK, "{}", file_order.body);
        let file_order_id = file_order.json["data"]["order_id"]
            .as_str()
            .expect("file order id")
            .to_string();

        let contract_resp = post_json(
            &app,
            format!("/api/v1/orders/{file_order_id}/contract-confirm"),
            &seed.buyer_org_id,
            Some(&seed.signer_user_id),
            &format!("req-trade027-{suffix}-contract"),
            &format!(
                r#"{{
                  "contract_template_id":"{}",
                  "contract_digest":"sha256:trade027:file:{}",
                  "variables_json":{{"term_days":30,"workflow":"main_flow"}},
                  "signer_role":"buyer_operator"
                }}"#,
                seed.file_contract_template_id, suffix
            ),
        )
        .await;
        assert_eq!(
            contract_resp.status,
            StatusCode::OK,
            "{}",
            contract_resp.body
        );
        assert_eq!(
            contract_resp.json["data"]["signature_provider_mode"].as_str(),
            Some("mock")
        );

        client
            .execute(
                r#"UPDATE catalog.product
                   SET metadata = jsonb_set(metadata, '{review_status}', '"rejected"'::jsonb, true)
                   WHERE product_id = $1::text::uuid"#,
                &[&seed.file_product_id],
            )
            .await
            .expect("set rejected review");

        let blocked_lock = post_json(
            &app,
            format!("/api/v1/orders/{file_order_id}/file-std/transition"),
            &seed.buyer_org_id,
            None,
            &format!("req-trade027-{suffix}-lock-blocked"),
            r#"{"action":"lock_funds"}"#,
        )
        .await;
        assert_eq!(
            blocked_lock.status,
            StatusCode::CONFLICT,
            "{}",
            blocked_lock.body
        );
        assert!(
            blocked_lock
                .body
                .contains("product review status is not approved")
        );

        client
            .execute(
                r#"UPDATE catalog.product
                   SET metadata = jsonb_set(metadata, '{review_status}', '"approved"'::jsonb, true)
                   WHERE product_id = $1::text::uuid"#,
                &[&seed.file_product_id],
            )
            .await
            .expect("set approved review");

        let lock_ok = post_json(
            &app,
            format!("/api/v1/orders/{file_order_id}/file-std/transition"),
            &seed.buyer_org_id,
            None,
            &format!("req-trade027-{suffix}-lock-ok"),
            r#"{"action":"lock_funds"}"#,
        )
        .await;
        assert_eq!(lock_ok.status, StatusCode::OK, "{}", lock_ok.body);
        assert_eq!(
            lock_ok.json["data"]["current_state"].as_str(),
            Some("buyer_locked")
        );

        let illegal_jump = post_json(
            &app,
            format!("/api/v1/orders/{file_order_id}/file-std/transition"),
            &seed.buyer_org_id,
            None,
            &format!("req-trade027-{suffix}-illegal"),
            r#"{"action":"close_completed"}"#,
        )
        .await;
        assert_eq!(
            illegal_jump.status,
            StatusCode::CONFLICT,
            "{}",
            illegal_jump.body
        );
        assert!(illegal_jump.body.contains("FILE_STD_TRANSITION_FORBIDDEN"));

        let share_order = create_order(
            &app,
            &seed.buyer_org_id,
            &seed.share_product_id,
            &seed.share_sku_id,
            &format!("req-trade027-{suffix}-share-create"),
            &format!("idem-trade027-{suffix}-share"),
        )
        .await;
        assert_eq!(share_order.status, StatusCode::OK, "{}", share_order.body);
        let share_order_id = share_order.json["data"]["order_id"]
            .as_str()
            .expect("share order id")
            .to_string();

        let grant_resp = post_json(
            &app,
            format!("/api/v1/orders/{share_order_id}/authorization/transition"),
            &seed.buyer_org_id,
            None,
            &format!("req-trade027-{suffix}-grant"),
            r#"{"action":"grant"}"#,
        )
        .await;
        assert_eq!(grant_resp.status, StatusCode::OK, "{}", grant_resp.body);

        client
            .execute(
                "UPDATE trade.order_main
                 SET status = 'shared_active',
                     delivery_status = 'active',
                     payment_status = 'paid',
                     updated_at = now()
                 WHERE order_id = $1::text::uuid",
                &[&share_order_id],
            )
            .await
            .expect("set share active");

        let cutoff_resp = post_json(
            &app,
            format!("/api/v1/orders/{share_order_id}/share-ro/transition"),
            &seed.buyer_org_id,
            None,
            &format!("req-trade027-{suffix}-cutoff"),
            r#"{"action":"expire_share"}"#,
        )
        .await;
        assert_eq!(cutoff_resp.status, StatusCode::OK, "{}", cutoff_resp.body);

        let file_row = client
            .query_one(
                "SELECT status, payment_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&file_order_id],
            )
            .await
            .expect("query file order");
        assert_eq!(file_row.get::<_, String>(0), "buyer_locked");
        assert_eq!(file_row.get::<_, String>(1), "paid");

        let share_auth_row = client
            .query_one(
                "SELECT status
                 FROM trade.authorization_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&share_order_id],
            )
            .await
            .expect("query share auth");
        assert_eq!(share_auth_row.get::<_, String>(0), "expired");

        assert_audit(
            &client,
            &format!("req-trade027-{suffix}-file-create"),
            "trade.order.create",
        )
        .await;
        assert_audit(
            &client,
            &format!("req-trade027-{suffix}-contract"),
            "trade.contract.confirm",
        )
        .await;
        assert_audit(
            &client,
            &format!("req-trade027-{suffix}-lock-ok"),
            "trade.order.file_std.transition",
        )
        .await;
        assert_audit(
            &client,
            &format!("req-trade027-{suffix}-cutoff"),
            "trade.authorization.auto_cutoff.expired",
        )
        .await;

        cleanup_seed_graph(&client, &seed, &file_order_id, &share_order_id).await;
    }

    struct ApiResponse {
        status: StatusCode,
        json: Value,
        body: String,
    }

    async fn create_order(
        app: &axum::Router,
        buyer_org_id: &str,
        product_id: &str,
        sku_id: &str,
        request_id: &str,
        idempotency_key: &str,
    ) -> ApiResponse {
        post_json_with_headers(
            app,
            "/api/v1/orders".to_string(),
            buyer_org_id,
            None,
            request_id,
            Some(idempotency_key),
            &format!(
                r#"{{
                  "buyer_org_id":"{}",
                  "product_id":"{}",
                  "sku_id":"{}"
                }}"#,
                buyer_org_id, product_id, sku_id
            ),
        )
        .await
    }

    async fn post_json(
        app: &axum::Router,
        uri: String,
        buyer_org_id: &str,
        user_id: Option<&str>,
        request_id: &str,
        payload: &str,
    ) -> ApiResponse {
        post_json_with_headers(app, uri, buyer_org_id, user_id, request_id, None, payload).await
    }

    async fn post_json_with_headers(
        app: &axum::Router,
        uri: String,
        buyer_org_id: &str,
        user_id: Option<&str>,
        request_id: &str,
        idempotency_key: Option<&str>,
        payload: &str,
    ) -> ApiResponse {
        let mut builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header("x-role", "buyer_operator")
            .header("x-tenant-id", buyer_org_id)
            .header("x-request-id", request_id)
            .header("content-type", "application/json");
        if let Some(user_id) = user_id {
            builder = builder.header("x-user-id", user_id);
        }
        if let Some(idempotency_key) = idempotency_key {
            builder = builder.header("x-idempotency-key", idempotency_key);
        }
        let response = app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body_text = String::from_utf8_lossy(&body).to_string();
        let json = serde_json::from_slice(&body).unwrap_or(Value::Null);
        ApiResponse {
            status,
            json,
            body: body_text,
        }
    }

    async fn assert_audit(client: &Client, request_id: &str, action_name: &str) {
        let count: i64 = client
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
        assert!(count >= 1, "missing audit: {action_name}");
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade027-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade027-seller-{suffix}")],
            )
            .await?
            .get(0);
        let signer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status)
                 VALUES ($1::text::uuid, $2, $3, 'human', 'active', 'verified')
                 RETURNING user_id::text",
                &[
                    &buyer_org_id,
                    &format!("trade027-user-{suffix}@example.com"),
                    &format!("trade027 user {suffix}"),
                ],
            )
            .await?
            .get(0);
        let file_contract_template_id: String = client
            .query_one(
                "INSERT INTO contract.template_definition (
                   template_type, template_name, applicable_sku_types, status
                 ) VALUES (
                   'contract', $1, ARRAY['FILE_STD']::text[], 'active'
                 )
                 RETURNING template_id::text",
                &[&format!("TRADE027-TPL-{suffix}")],
            )
            .await?
            .get(0);

        let file_asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 ) RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade027-file-asset-{suffix}"),
                    &format!("trade027 file asset {suffix}"),
                ],
            )
            .await?
            .get(0);
        let file_asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text",
                &[&file_asset_id],
            )
            .await?
            .get(0);
        let file_product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 188.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &file_asset_id,
                    &file_asset_version_id,
                    &seller_org_id,
                    &format!("trade027-file-product-{suffix}"),
                    &format!("trade027 file product {suffix}"),
                    &format!("trade027 file search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let file_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&file_product_id, &format!("TRADE027-FILE-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let share_asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'draft', $3
                 ) RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade027-share-asset-{suffix}"),
                    &format!("trade027 share asset {suffix}"),
                ],
            )
            .await?
            .get(0);
        let share_asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   2048, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text",
                &[&share_asset_id],
            )
            .await?
            .get(0);
        let share_product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'subscription', 21.50, 'CNY', 'read_only_share',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &share_asset_id,
                    &share_asset_version_id,
                    &seller_org_id,
                    &format!("trade027-share-product-{suffix}"),
                    &format!("trade027 share product {suffix}"),
                    &format!("trade027 share search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let share_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '月', 'subscription', 'auto_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&share_product_id, &format!("TRADE027-SHARE-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let share_policy_id: String = client
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
                &[&seller_org_id, &format!("TRADE027-POL-{suffix}")],
            )
            .await?
            .get(0);
        client
            .execute(
                "INSERT INTO contract.policy_binding (policy_id, product_id, sku_id, binding_scope)
                 VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, 'sku')",
                &[&share_policy_id, &share_product_id, &share_sku_id],
            )
            .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            signer_user_id,
            file_asset_id,
            file_asset_version_id,
            file_product_id,
            file_sku_id,
            file_contract_template_id,
            share_asset_id,
            share_asset_version_id,
            share_product_id,
            share_sku_id,
            share_policy_id,
        })
    }

    async fn cleanup_seed_graph(
        client: &Client,
        seed: &SeedGraph,
        file_order_id: &str,
        share_order_id: &str,
    ) {
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![file_order_id.to_string(), share_order_id.to_string()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.contract_signer
                 WHERE contract_id IN (
                   SELECT contract_id FROM contract.digital_contract
                   WHERE order_id = ANY($1::text[]::uuid[])
                 )",
                &[&vec![file_order_id.to_string(), share_order_id.to_string()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![file_order_id.to_string(), share_order_id.to_string()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract
                 WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![file_order_id.to_string(), share_order_id.to_string()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.policy_binding WHERE policy_id = $1::text::uuid",
                &[&seed.share_policy_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.usage_policy WHERE policy_id = $1::text::uuid",
                &[&seed.share_policy_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.file_sku_id.clone(), seed.share_sku_id.clone()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_product_id.clone(),
                    seed.share_product_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_asset_version_id.clone(),
                    seed.share_asset_version_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_asset_id.clone(),
                    seed.share_asset_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.template_definition WHERE template_id = $1::text::uuid",
                &[&seed.file_contract_template_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.signer_user_id],
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
