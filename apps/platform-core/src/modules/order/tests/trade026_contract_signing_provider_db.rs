#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedOrder {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        contract_template_id: String,
        signer_user_id: String,
    }

    #[tokio::test]
    async fn trade026_contract_signing_provider_db_smoke() {
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
        let seed = seed_order_graph(&client, &suffix)
            .await
            .expect("seed order graph");

        let app = crate::with_live_test_state(router()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/contract-confirm", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.signer_user_id)
                    .header("x-request-id", format!("req-trade026-{suffix}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                          "contract_template_id":"{}",
                          "contract_digest":"sha256:trade026:{}",
                          "variables_json":{{"term_days":45}},
                          "signer_role":"buyer_operator"
                        }}"#,
                        seed.contract_template_id, suffix
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
        let contract_id = json["data"]["data"]["contract_id"]
            .as_str()
            .expect("contract_id")
            .to_string();
        let provider_ref = json["data"]["data"]["signature_provider_ref"]
            .as_str()
            .expect("signature_provider_ref")
            .to_string();
        assert_eq!(
            json["data"]["data"]["signature_provider_mode"].as_str(),
            Some("mock")
        );
        assert_eq!(
            json["data"]["data"]["signature_provider_kind"].as_str(),
            Some("mock")
        );
        assert!(provider_ref.contains("mock-signing-ok"));

        let signer_row = client
            .query_one(
                "SELECT signer_type, signature_digest
                 FROM contract.contract_signer
                 WHERE contract_id = $1::text::uuid
                   AND signer_id = $2::text::uuid
                   AND signer_role = 'buyer_operator'",
                &[&contract_id, &seed.signer_user_id],
            )
            .await
            .expect("query signer");
        assert_eq!(signer_row.get::<_, String>(0), "user");
        assert_eq!(
            signer_row.get::<_, Option<String>>(1).as_deref(),
            Some(provider_ref.as_str())
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_order_graph(client: &Client, suffix: &str) -> Result<SeedOrder, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade026-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade026-seller-{suffix}")],
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
                    &format!("trade026-user-{suffix}@example.com"),
                    &format!("trade026 user {suffix}"),
                ],
            )
            .await?
            .get(0);
        let contract_template_id: String = client
            .query_one(
                "INSERT INTO contract.template_definition (
                   template_type, template_name, applicable_sku_types, status
                 ) VALUES (
                   'contract', $1, ARRAY['FILE_STD']::text[], 'active'
                 )
                 RETURNING template_id::text",
                &[&format!("TRADE026-TPL-{suffix}")],
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
                    &format!("trade026-asset-{suffix}"),
                    &format!("trade026 asset {suffix}"),
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 288.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade026-product-{suffix}"),
                    &format!("trade026 product {suffix}"),
                    &format!("trade026 search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("TRADE026-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'contract_pending', 'unpaid', 'not_started', 'not_started', 'not_started', 'none',
                   'online', 288.00, 'CNY', '{}'::jsonb
                 )
                 RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        Ok(SeedOrder {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            contract_template_id,
            signer_user_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.contract_signer
                 WHERE contract_id IN (
                   SELECT contract_id FROM contract.digital_contract WHERE order_id = $1::text::uuid
                 )",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM contract.template_definition WHERE template_id = $1::text::uuid",
                &[&seed.contract_template_id],
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
