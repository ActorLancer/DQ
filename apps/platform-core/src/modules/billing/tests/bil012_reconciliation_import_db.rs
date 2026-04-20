#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use crate::modules::billing::models::ReconciliationImportDiffInput;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        payment_intent_id: String,
        settlement_id: String,
        provider_account_id: String,
    }

    #[tokio::test]
    async fn bil012_reconciliation_import_db_smoke() {
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
        let seed = seed_graph(&client, &suffix).await;
        let app = crate::with_live_test_state(router()).await;

        let diffs = vec![
            ReconciliationImportDiffInput {
                diff_type: "amount_mismatch".to_string(),
                ref_type: Some("payment_intent".to_string()),
                ref_id: Some(seed.payment_intent_id.clone()),
                provider_reference_no: Some(format!("txn-bil012-{suffix}")),
                internal_amount: Some("88.00000000".to_string()),
                provider_amount: Some("87.50000000".to_string()),
                diff_status: Some("open".to_string()),
                resolution_note: None,
                resolved_at: None,
            },
            ReconciliationImportDiffInput {
                diff_type: "settlement_lag".to_string(),
                ref_type: Some("settlement".to_string()),
                ref_id: Some(seed.settlement_id.clone()),
                provider_reference_no: Some(format!("stl-bil012-{suffix}")),
                internal_amount: Some("85.00000000".to_string()),
                provider_amount: Some("85.00000000".to_string()),
                diff_status: Some("resolved".to_string()),
                resolution_note: Some("verified".to_string()),
                resolved_at: Some("2026-04-20T10:00:00Z".to_string()),
            },
        ];

        let request_id = format!("req-bil012-import-{suffix}");
        let first = import_statement(
            &app,
            &seed.provider_account_id,
            &request_id,
            "2026-04-20",
            "daily_collection",
            &diffs,
        )
        .await;
        assert_eq!(
            first["data"]["statement"]["import_status"].as_str(),
            Some("mismatched")
        );
        assert_eq!(first["data"]["imported_diff_count"].as_u64(), Some(2));
        assert_eq!(first["data"]["open_diff_count"].as_u64(), Some(1));
        assert_eq!(first["data"]["idempotent_replay"].as_bool(), Some(false));

        let replay = import_statement(
            &app,
            &seed.provider_account_id,
            &format!("{request_id}-replay"),
            "2026-04-20",
            "daily_collection",
            &diffs,
        )
        .await;
        assert_eq!(
            replay["data"]["statement"]["reconciliation_statement_id"],
            first["data"]["statement"]["reconciliation_statement_id"]
        );
        assert_eq!(replay["data"]["idempotent_replay"].as_bool(), Some(true));

        let statement_id = first["data"]["statement"]["reconciliation_statement_id"]
            .as_str()
            .expect("statement id")
            .to_string();
        let statement_row = client
            .query_one(
                "SELECT import_status, file_hash
                 FROM payment.reconciliation_statement
                 WHERE reconciliation_statement_id = $1::text::uuid",
                &[&statement_id],
            )
            .await
            .expect("query statement");
        let import_status: String = statement_row.get(0);
        let file_hash: Option<String> = statement_row.get(1);
        assert_eq!(import_status, "mismatched");
        assert!(file_hash.is_some());

        let diff_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM payment.reconciliation_diff
                 WHERE reconciliation_statement_id = $1::text::uuid",
                &[&statement_id],
            )
            .await
            .expect("query diff count")
            .get(0);
        assert_eq!(diff_count, 2);

        let payment_intent_reconcile: String = client
            .query_one(
                "SELECT reconcile_status
                 FROM payment.payment_intent
                 WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await
            .expect("query intent reconcile")
            .get(0);
        assert_eq!(payment_intent_reconcile, "mismatched");

        let settlement_reconcile: String = client
            .query_one(
                "SELECT reconcile_status
                 FROM billing.settlement_record
                 WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await
            .expect("query settlement reconcile")
            .get(0);
        assert_eq!(settlement_reconcile, "resolved");

        let audit_import_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'payment.reconciliation.import'",
                &[&request_id],
            )
            .await
            .expect("query audit import")
            .get(0);
        assert_eq!(audit_import_count, 1);

        let audit_replay_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'payment.reconciliation.import.idempotent_replay'",
                &[&format!("{request_id}-replay")],
            )
            .await
            .expect("query audit replay")
            .get(0);
        assert_eq!(audit_replay_count, 1);

        cleanup(&client, &seed, &statement_id).await;
    }

    async fn import_statement(
        app: &Router,
        provider_account_id: &str,
        request_id: &str,
        statement_date: &str,
        statement_type: &str,
        diffs: &[ReconciliationImportDiffInput],
    ) -> Value {
        let boundary = format!("BIL012-{}", request_id.replace(':', "-"));
        let diffs_json = serde_json::to_string(diffs).expect("serialize diffs");
        let body = [
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"provider_key\"".to_string(),
            String::new(),
            "mock_payment".to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"provider_account_id\"".to_string(),
            String::new(),
            provider_account_id.to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"statement_date\"".to_string(),
            String::new(),
            statement_date.to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"statement_type\"".to_string(),
            String::new(),
            statement_type.to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"diffs_json\"".to_string(),
            String::new(),
            diffs_json,
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"file\"; filename=\"statement.csv\"".to_string(),
            "Content-Type: text/csv".to_string(),
            String::new(),
            "provider_reference_no,amount".to_string(),
            "mock-ref,88.00".to_string(),
            format!("--{boundary}--"),
            String::new(),
        ]
        .join("\r\n");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/reconciliation/import")
                    .header("x-role", "platform_risk_settlement")
                    .header("x-step-up-token", "bil012-stepup")
                    .header("x-request-id", request_id)
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("import request should build"),
            )
            .await
            .expect("import response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("import body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("import json")
    }

    async fn seed_graph(client: &db::Client, suffix: &str) -> SeedGraph {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil012-buyer-{suffix}")],
            )
            .await
            .expect("insert buyer")
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil012-seller-{suffix}")],
            )
            .await
            .expect("insert seller")
            .get(0);
        let provider_account_id: String = client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key, account_scope, account_scope_id, account_name,
                   settlement_subject_type, settlement_subject_id, jurisdiction_code,
                   account_mode, status, config_json
                 ) VALUES (
                   'mock_payment', 'tenant', $1::text::uuid, $2,
                   'organization', $1::text::uuid, 'SG',
                   'sandbox', 'active', '{}'::jsonb
                 ) RETURNING provider_account_id::text",
                &[&seller_org_id, &format!("bil012-account-{suffix}")],
            )
            .await
            .expect("insert provider account")
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil012-asset-{suffix}"),
                    &format!("bil012 asset {suffix}"),
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
                   1024, 'SG', ARRAY['SG']::text[], false,
                   '{"payment_mode":"online"}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text"#,
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
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_download',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil012-product-{suffix}"),
                    &format!("bil012 product {suffix}"),
                    &format!("bil012 summary {suffix}"),
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
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("BIL012-SKU-{suffix}")],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status,
                   dispute_status, payment_mode, amount, currency_code, price_snapshot_json,
                   delivery_route_snapshot, trust_boundary_snapshot, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'accepted', 'paid', 'delivered', 'accepted', 'pending_settlement',
                   'none', 'online', 88.00, 'SGD', '{}'::jsonb,
                   'file_download', '{}'::jsonb, 'bil012_seed'
                 ) RETURNING order_id::text"#,
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                ],
            )
            .await
            .expect("insert order")
            .get(0);
        let payment_intent_id: String = client
            .query_one(
                r#"INSERT INTO payment.payment_intent (
                   order_id, intent_type, provider_key, provider_account_id, payer_subject_type, payer_subject_id,
                   payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
                   launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
                   status, request_id, idempotency_key, capability_snapshot, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_payment', 'mock_payment', $2::text::uuid, 'organization', $3::text::uuid,
                   'organization', $4::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'SGD',
                   'succeeded', $5, $6, '{}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &provider_account_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil012-pay-req-{suffix}"),
                    &format!("pay:bil012:{suffix}"),
                ],
            )
            .await
            .expect("insert payment intent")
            .get(0);
        let settlement_id: String = client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id, settlement_type, settlement_status, settlement_mode,
                   payable_amount, platform_fee_amount, channel_fee_amount,
                   net_receivable_amount, refund_amount, compensation_amount,
                   reason_code, settled_at
                 ) VALUES (
                   $1::text::uuid, 'order_settlement', 'pending', 'manual',
                   88.00000000, 2.00000000, 1.00000000,
                   85.00000000, 0.00000000, 0.00000000,
                   'bil012_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);
        SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
            settlement_id,
            provider_account_id,
        }
    }

    async fn cleanup(client: &db::Client, seed: &SeedGraph, statement_id: &str) {
        let _ = client
            .execute(
                "DELETE FROM payment.reconciliation_statement WHERE reconciliation_statement_id = $1::text::uuid",
                &[&statement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE settlement_id = $1::text::uuid",
                &[&seed.settlement_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.provider_account WHERE provider_account_id = $1::text::uuid",
                &[&seed.provider_account_id],
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
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid OR org_id = $2::text::uuid",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
