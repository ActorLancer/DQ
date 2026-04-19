#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use tokio_postgres::{Client, NoTls};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct ProductSeed {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
    }

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        signer_user_id: String,
        api_sub: ProductSeed,
        rpt_std: ProductSeed,
        contract_template_id: String,
        policy_id: String,
    }

    #[tokio::test]
    async fn trade032_scenario_sku_snapshot_db_smoke() {
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
        let create_missing_request_id = format!("req-trade032-create-missing-{suffix}");
        let create_missing_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &create_missing_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "buyer_org_id": seed.buyer_org_id,
                            "product_id": seed.api_sub.product_id,
                            "sku_id": seed.api_sub.sku_id
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("create missing response");
        let create_missing_status = create_missing_response.status();
        let create_missing_body = to_bytes(create_missing_response.into_body(), usize::MAX)
            .await
            .expect("create missing body");
        assert_eq!(create_missing_status, StatusCode::CONFLICT);
        assert!(String::from_utf8_lossy(&create_missing_body)
            .contains("scenario_code is required for sku_type `API_SUB` because it belongs to multiple frozen scenarios: S1,S4"));

        let create_api_request_id = format!("req-trade032-create-api-{suffix}");
        let create_api_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &create_api_request_id)
                    .header("x-idempotency-key", format!("idem-trade032-api-{suffix}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "buyer_org_id": seed.buyer_org_id,
                            "product_id": seed.api_sub.product_id,
                            "sku_id": seed.api_sub.sku_id,
                            "scenario_code": "S4"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("create api response");
        let create_api_status = create_api_response.status();
        let create_api_body = to_bytes(create_api_response.into_body(), usize::MAX)
            .await
            .expect("create api body");
        assert_eq!(
            create_api_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_api_body)
        );
        let create_api_json: Value = serde_json::from_slice(&create_api_body).expect("json");
        let api_order_id = create_api_json["data"]["data"]["order_id"]
            .as_str()
            .expect("api order id")
            .to_string();
        assert_eq!(
            create_api_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["scenario_code"]
                .as_str(),
            Some("S4")
        );
        assert_eq!(
            create_api_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]
                ["selected_sku_role"]
                .as_str(),
            Some("primary")
        );
        assert_eq!(
            create_api_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["primary_sku"]
                .as_str(),
            Some("API_SUB")
        );
        assert_eq!(
            create_api_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]
                ["supplementary_skus"][0]
                .as_str(),
            Some("RPT_STD")
        );

        let freeze_request_id = format!("req-trade032-freeze-{suffix}");
        let freeze_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/trade/orders/{api_order_id}/price-snapshot/freeze"
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-request-id", &freeze_request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("freeze response");
        let freeze_status = freeze_response.status();
        let freeze_body = to_bytes(freeze_response.into_body(), usize::MAX)
            .await
            .expect("freeze body");
        assert_eq!(
            freeze_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&freeze_body)
        );
        let freeze_json: Value = serde_json::from_slice(&freeze_body).expect("freeze json");
        assert_eq!(
            freeze_json["data"]["data"]["snapshot"]["scenario_snapshot"]["scenario_code"].as_str(),
            Some("S4")
        );

        let create_rpt_request_id = format!("req-trade032-create-rpt-{suffix}");
        let create_rpt_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &create_rpt_request_id)
                    .header("x-idempotency-key", format!("idem-trade032-rpt-{suffix}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "buyer_org_id": seed.buyer_org_id,
                            "product_id": seed.rpt_std.product_id,
                            "sku_id": seed.rpt_std.sku_id,
                            "scenario_code": "S5"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("create rpt response");
        let create_rpt_status = create_rpt_response.status();
        let create_rpt_body = to_bytes(create_rpt_response.into_body(), usize::MAX)
            .await
            .expect("create rpt body");
        assert_eq!(
            create_rpt_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_rpt_body)
        );
        let create_rpt_json: Value = serde_json::from_slice(&create_rpt_body).expect("json");
        let rpt_order_id = create_rpt_json["data"]["data"]["order_id"]
            .as_str()
            .expect("rpt order id")
            .to_string();
        assert_eq!(
            create_rpt_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["scenario_code"]
                .as_str(),
            Some("S5")
        );
        assert_eq!(
            create_rpt_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]
                ["selected_sku_role"]
                .as_str(),
            Some("supplementary")
        );
        assert_eq!(
            create_rpt_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["primary_sku"]
                .as_str(),
            Some("QRY_LITE")
        );

        let confirm_request_id = format!("req-trade032-confirm-{suffix}");
        let confirm_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{api_order_id}/contract-confirm"))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.signer_user_id)
                    .header("x-request-id", &confirm_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "contract_template_id": seed.contract_template_id,
                            "contract_digest": format!("sha256:trade032-contract:{suffix}"),
                            "variables_json": { "term_days": 30 },
                            "signer_role": "buyer_operator"
                        })
                        .to_string(),
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("confirm response");
        let confirm_status = confirm_response.status();
        let confirm_body = to_bytes(confirm_response.into_body(), usize::MAX)
            .await
            .expect("confirm body");
        assert_eq!(
            confirm_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&confirm_body)
        );
        let confirm_json: Value = serde_json::from_slice(&confirm_body).expect("json");
        assert_eq!(
            confirm_json["data"]["data"]["variables_json"]["scenario_sku_snapshot"]
                ["scenario_code"]
                .as_str(),
            Some("S4")
        );
        assert_eq!(
            confirm_json["data"]["data"]["variables_json"]["scenario_sku_snapshot"]
                ["selected_sku_type"]
                .as_str(),
            Some("API_SUB")
        );

        let authorization_request_id = format!("req-trade032-auth-{suffix}");
        let authorization_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{api_order_id}/authorization/transition"
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &authorization_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"grant"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("authorization response");
        let authorization_status = authorization_response.status();
        let authorization_body = to_bytes(authorization_response.into_body(), usize::MAX)
            .await
            .expect("authorization body");
        assert_eq!(
            authorization_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&authorization_body)
        );
        let authorization_json: Value =
            serde_json::from_slice(&authorization_body).expect("authorization json");
        assert_eq!(
            authorization_json["data"]["data"]["policy_snapshot"]["scenario_sku_snapshot"]
                ["scenario_code"]
                .as_str(),
            Some("S4")
        );
        assert_eq!(
            authorization_json["data"]["data"]["authorization_model"]["resource"]["sku_type"]
                .as_str(),
            Some("API_SUB")
        );

        let detail_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{api_order_id}"))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", format!("req-trade032-detail-{suffix}"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("detail response");
        let detail_status = detail_response.status();
        let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .expect("detail body");
        assert_eq!(
            detail_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&detail_body)
        );
        let detail_json: Value = serde_json::from_slice(&detail_body).expect("detail json");
        assert_eq!(
            detail_json["data"]["data"]["price_snapshot"]["scenario_snapshot"]["scenario_code"]
                .as_str(),
            Some("S4")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["contract"]["variables_json"]
                ["scenario_sku_snapshot"]["selected_sku_type"]
                .as_str(),
            Some("API_SUB")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["authorizations"][0]["policy_snapshot"]
                ["scenario_sku_snapshot"]["scenario_code"]
                .as_str(),
            Some("S4")
        );

        let order_snapshot_row = client
            .query_one(
                "SELECT price_snapshot_json
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&api_order_id],
            )
            .await
            .expect("query order snapshot");
        let order_snapshot: Value = order_snapshot_row.get(0);
        assert_eq!(
            order_snapshot["scenario_snapshot"]["scenario_code"].as_str(),
            Some("S4")
        );
        assert_eq!(
            order_snapshot["scenario_snapshot"]["contract_template"].as_str(),
            Some("CONTRACT_API_SUB_V1")
        );

        let contract_row = client
            .query_one(
                "SELECT variables_json
                 FROM contract.digital_contract
                 WHERE order_id = $1::text::uuid",
                &[&api_order_id],
            )
            .await
            .expect("query contract snapshot");
        let contract_variables: Value = contract_row.get(0);
        assert_eq!(
            contract_variables["scenario_sku_snapshot"]["scenario_code"].as_str(),
            Some("S4")
        );

        let authorization_row = client
            .query_one(
                "SELECT policy_snapshot
                 FROM trade.authorization_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY updated_at DESC
                 LIMIT 1",
                &[&api_order_id],
            )
            .await
            .expect("query authorization snapshot");
        let policy_snapshot: Value = authorization_row.get(0);
        assert_eq!(
            policy_snapshot["scenario_sku_snapshot"]["scenario_code"].as_str(),
            Some("S4")
        );
        assert_eq!(
            policy_snapshot["scenario_sku_snapshot"]["selected_sku_type"].as_str(),
            Some("API_SUB")
        );

        assert_audit_count(&client, &create_api_request_id, "trade.order.create").await;
        assert_audit_count(
            &client,
            &freeze_request_id,
            "trade.order.price_snapshot.freeze",
        )
        .await;
        assert_audit_count(&client, &create_rpt_request_id, "trade.order.create").await;
        assert_audit_count(&client, &confirm_request_id, "trade.contract.confirm").await;
        assert_audit_count(
            &client,
            &authorization_request_id,
            "trade.authorization.grant",
        )
        .await;

        cleanup_seed_graph(&client, &seed, &[api_order_id, rpt_order_id]).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1::text, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade032-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1::text, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade032-seller-{suffix}")],
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
                    &format!("trade032-user-{suffix}@example.com"),
                    &format!("trade032 user {suffix}"),
                ],
            )
            .await?
            .get(0);
        let contract_template_id: String = client
            .query_one(
                "INSERT INTO contract.template_definition (
                   template_type, template_name, applicable_sku_types, status
                 ) VALUES (
                   'contract', 'CONTRACT_API_SUB_V1', ARRAY['API_SUB']::text[], 'active'
                 )
                 RETURNING template_id::text",
                &[],
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
                &[&seller_org_id, &format!("TRADE032-POL-{suffix}")],
            )
            .await?
            .get(0);

        let api_sub = insert_product_with_sku(
            client,
            suffix,
            &seller_org_id,
            "trade032-api",
            "manufacturing",
            "subscription",
            "288.00",
            "api_access",
            "API_SUB",
            "subscription",
        )
        .await?;
        let rpt_std = insert_product_with_sku(
            client,
            suffix,
            &seller_org_id,
            "trade032-rpt",
            "retail",
            "one_time",
            "88.00",
            "report_delivery",
            "RPT_STD",
            "one_time",
        )
        .await?;

        let _ = client
            .query_one(
                "INSERT INTO contract.policy_binding (policy_id, product_id, sku_id, binding_scope)
                 VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, 'sku')
                 RETURNING policy_binding_id::text",
                &[&policy_id, &api_sub.product_id, &api_sub.sku_id],
            )
            .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            signer_user_id,
            api_sub,
            rpt_std,
            contract_template_id,
            policy_id,
        })
    }

    async fn insert_product_with_sku(
        client: &Client,
        suffix: &str,
        seller_org_id: &str,
        name_prefix: &str,
        category: &str,
        price_mode: &str,
        price: &str,
        delivery_type: &str,
        sku_type: &str,
        billing_mode: &str,
    ) -> Result<ProductSeed, tokio_postgres::Error> {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'internal', 'draft', $4
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("{name_prefix}-asset-{suffix}"),
                    &category,
                    &format!("{name_prefix} asset {suffix}"),
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, $5, 'data_product',
                   $6, 'listed', $7, $8::text::numeric, 'CNY', $9,
                   ARRAY['internal_use']::text[], $10,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("{name_prefix}-product-{suffix}"),
                    &category,
                    &format!("{name_prefix} product {suffix}"),
                    &price_mode,
                    &price,
                    &delivery_type,
                    &format!("{name_prefix} search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, '次', $4, 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("{}-SKU-{}", sku_type, suffix),
                    &sku_type,
                    &billing_mode,
                ],
            )
            .await?
            .get(0);
        Ok(ProductSeed {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
        })
    }

    async fn assert_audit_count(client: &Client, request_id: &str, action_name: &str) {
        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = $2",
                &[&request_id, &action_name],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert!(audit_count >= 1, "missing audit event for {action_name}");
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, order_ids: &[String]) {
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id = ANY($1::text[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.contract_signer
                 WHERE contract_id IN (
                   SELECT contract_id FROM contract.digital_contract
                   WHERE order_id = ANY($1::text[]::uuid[])
                 )",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
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
                "DELETE FROM contract.template_definition WHERE template_id = $1::text::uuid",
                &[&seed.contract_template_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku
                 WHERE sku_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.api_sub.sku_id.clone(),
                    seed.rpt_std.sku_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product
                 WHERE product_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.api_sub.product_id.clone(),
                    seed.rpt_std.product_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version
                 WHERE asset_version_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.api_sub.asset_version_id.clone(),
                    seed.rpt_std.asset_version_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset
                 WHERE asset_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.api_sub.asset_id.clone(),
                    seed.rpt_std.asset_id.clone(),
                ]],
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
