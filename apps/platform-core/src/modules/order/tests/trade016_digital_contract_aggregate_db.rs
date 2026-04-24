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
        confirmable_order_id: String,
        forbidden_order_id: String,
        contract_template_id: String,
        data_contract_id: String,
        signer_user_id: String,
    }

    #[tokio::test]
    async fn trade016_digital_contract_aggregate_db_smoke() {
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

        let request_id = format!("req-trade016-ok-{suffix}");
        let app = crate::with_live_test_state(router()).await;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/contract-confirm",
                        seed.confirmable_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.signer_user_id)
                    .header("x-request-id", &request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                          "contract_template_id":"{}",
                          "contract_digest":"sha256:trade016-contract:{}",
                          "data_contract_id":"{}",
                          "data_contract_digest":"sha256:trade016-data-contract:{}",
                          "variables_json":{{"term_days":30,"sla":"gold","region":"cn-east-1"}},
                          "signer_role":"buyer_operator"
                        }}"#,
                        seed.contract_template_id, suffix, seed.data_contract_id, suffix
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        if status != StatusCode::OK {
            panic!(
                "unexpected status: {}, body={}",
                status,
                String::from_utf8_lossy(&body)
            );
        }
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["code"].as_str(), Some("OK"));
        assert_eq!(json["message"].as_str(), Some("success"));
        assert_eq!(json["request_id"].as_str(), Some(request_id.as_str()));
        let contract_id = json["data"]["contract_id"]
            .as_str()
            .expect("contract_id")
            .to_string();
        assert_eq!(
            json["data"]["current_state"].as_str(),
            Some("contract_effective")
        );
        assert_eq!(json["data"]["contract_status"].as_str(), Some("signed"));
        assert_eq!(
            json["data"]["contract_template_id"].as_str(),
            Some(seed.contract_template_id.as_str())
        );
        assert_eq!(
            json["data"]["data_contract_id"].as_str(),
            Some(seed.data_contract_id.as_str())
        );
        assert_eq!(
            json["data"]["signer_id"].as_str(),
            Some(seed.signer_user_id.as_str())
        );
        assert_eq!(json["data"]["signer_type"].as_str(), Some("user"));
        assert_eq!(
            json["data"]["signature_provider_mode"].as_str(),
            Some("mock")
        );
        assert_eq!(
            json["data"]["signature_provider_kind"].as_str(),
            Some("mock")
        );
        assert!(
            json["data"]["signature_provider_ref"]
                .as_str()
                .unwrap_or_default()
                .contains("mock-signing-ok")
        );
        assert_eq!(
            json["data"]["onchain_digest_ref"].as_str(),
            Some(format!("sha256:trade016-contract:{suffix}").as_str())
        );

        let order_row = client
            .query_one(
                "SELECT status, contract_id::text, delivery_status, acceptance_status, settlement_status, dispute_status, last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.confirmable_order_id],
            )
            .await
            .expect("query order");
        assert_eq!(order_row.get::<_, String>(0), "contract_effective");
        assert_eq!(order_row.get::<_, String>(1), contract_id);
        assert_eq!(order_row.get::<_, String>(2), "pending_delivery");
        assert_eq!(order_row.get::<_, String>(3), "not_started");
        assert_eq!(order_row.get::<_, String>(4), "not_started");
        assert_eq!(order_row.get::<_, String>(5), "none");
        assert_eq!(
            order_row.get::<_, Option<String>>(6).as_deref(),
            Some("TRADE-006")
        );

        let contract_row = client
            .query_one(
                "SELECT status, contract_digest, data_contract_id::text, data_contract_digest, variables_json->>'region'
                 FROM contract.digital_contract
                 WHERE contract_id = $1::text::uuid",
                &[&contract_id],
            )
            .await
            .expect("query contract");
        assert_eq!(contract_row.get::<_, String>(0), "signed");
        assert!(
            contract_row
                .get::<_, Option<String>>(1)
                .unwrap_or_default()
                .contains("sha256:trade016-contract")
        );
        assert_eq!(
            contract_row.get::<_, Option<String>>(2).as_deref(),
            Some(seed.data_contract_id.as_str())
        );
        assert!(
            contract_row
                .get::<_, Option<String>>(3)
                .unwrap_or_default()
                .contains("sha256:trade016-data-contract")
        );
        assert_eq!(
            contract_row.get::<_, Option<String>>(4).as_deref(),
            Some("cn-east-1")
        );

        let signer_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM contract.contract_signer
                 WHERE contract_id = $1::text::uuid
                   AND signer_id = $2::text::uuid
                   AND signer_role = 'buyer_operator'",
                &[&contract_id, &seed.signer_user_id],
            )
            .await
            .expect("query signer")
            .get(0);
        assert!(signer_count >= 1);

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.contract.confirm'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);

        let forbidden_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/contract-confirm",
                        seed.forbidden_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.signer_user_id)
                    .header("x-request-id", format!("req-trade016-forbidden-{suffix}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{
                          "contract_template_id":"{}",
                          "contract_digest":"sha256:trade016-forbidden:{}",
                          "variables_json":{{"term_days":7}},
                          "signer_role":"buyer_operator"
                        }}"#,
                        seed.contract_template_id, suffix
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(forbidden_response.status(), StatusCode::CONFLICT);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_order_graph(client: &Client, suffix: &str) -> Result<SeedOrder, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade016-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade016-seller-{suffix}")],
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
                    &format!("trade016-user-{suffix}@example.com"),
                    &format!("trade016 user {suffix}"),
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
                &[&format!("TRADE016-TPL-{suffix}")],
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
                    &format!("trade016-asset-{suffix}"),
                    &format!("trade016 asset {suffix}"),
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
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
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
                   $5, 'listed', 'one_time', 18.80, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade016-product-{suffix}"),
                    &format!("trade016 product {suffix}"),
                    &format!("trade016 search {suffix}"),
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
                &[&product_id, &format!("TRADE016-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let data_contract_id: String = client
            .query_one(
                "INSERT INTO contract.data_contract (
                   version_no, contract_name, contract_scope, product_id, sku_id, business_terms_json, structure_terms_json,
                   quality_terms_json, compliance_terms_json, delivery_terms_json, version_terms_json,
                   acceptance_terms_json, rights_terms_json, responsibility_terms_json,
                   processing_terms_json, content_digest, status
                 ) VALUES (
                   1, $1, 'sku', $2::text::uuid, $3::text::uuid, '{}'::jsonb, '{}'::jsonb,
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
                   '{}'::jsonb, $4, 'active'
                 )
                 RETURNING data_contract_id::text",
                &[
                    &format!("TRADE016-DC-{suffix}"),
                    &product_id,
                    &sku_id,
                    &format!("sha256:trade016-data-contract:{suffix}"),
                ],
            )
            .await?
            .get(0);

        let confirmable_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'created', 'unpaid', 'online', 18.80, 'CNY'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await?
            .get(0);

        let forbidden_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'delivered', 'paid', 'online', 18.80, 'CNY'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
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
            confirmable_order_id,
            forbidden_order_id,
            contract_template_id,
            data_contract_id,
            signer_user_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main
                 WHERE order_id IN (
                   $1::text::uuid,
                   $2::text::uuid
                 )",
                &[&seed.confirmable_order_id, &seed.forbidden_order_id],
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
                "DELETE FROM contract.data_contract WHERE data_contract_id = $1::text::uuid",
                &[&seed.data_contract_id],
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
                "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
                &[&seed.signer_user_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.buyer_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await;
    }
}
