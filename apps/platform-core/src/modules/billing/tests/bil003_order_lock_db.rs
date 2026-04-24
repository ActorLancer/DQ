#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn bil003_order_lock_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil003-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil003-seller-{suffix}")).await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let order_graph = seed_order(&client, &buyer_org_id, &seller_org_id, &suffix).await;
        let order_id = order_graph.order_id.clone();
        let other_order_graph = seed_order(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &format!("{suffix}-other"),
        )
        .await;
        let other_order_id = other_order_graph.order_id.clone();
        let payment_intent_id = seed_payment_intent(
            &client,
            &order_id,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &suffix,
        )
        .await;
        let mismatch_intent_id = seed_payment_intent(
            &client,
            &other_order_id,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &format!("{suffix}-mismatch"),
        )
        .await;

        let app = crate::with_live_test_state(router()).await;
        let request_ok = format!("req-bil003-ok-{suffix}");
        let request_replay = format!("req-bil003-replay-{suffix}");
        let request_mismatch = format!("req-bil003-mismatch-{suffix}");

        let lock_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/lock"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_ok)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"buyer_deposit_lock"}}"#
                    )))
                    .expect("lock request build"),
            )
            .await
            .expect("lock response");
        let lock_status = lock_response.status();
        let lock_body = to_bytes(lock_response.into_body(), usize::MAX)
            .await
            .expect("lock body");
        assert_eq!(
            lock_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&lock_body)
        );
        let lock_json: Value = serde_json::from_slice(&lock_body).expect("lock json");
        assert_eq!(lock_json["code"].as_str(), Some("OK"));
        assert_eq!(lock_json["message"].as_str(), Some("success"));
        assert_eq!(lock_json["request_id"].as_str(), Some(request_ok.as_str()));
        assert_eq!(
            lock_json["data"]["current_state"].as_str(),
            Some("buyer_locked")
        );
        assert_eq!(lock_json["data"]["payment_status"].as_str(), Some("locked"));

        let replay_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/lock"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_replay)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"payment_intent_id":"{payment_intent_id}","lock_reason":"buyer_deposit_lock"}}"#
                    )))
                    .expect("replay request build"),
            )
            .await
            .expect("replay response");
        let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .expect("replay body");
        let replay_json: Value = serde_json::from_slice(&replay_body).expect("replay json");
        assert_eq!(replay_json["code"].as_str(), Some("OK"));
        assert_eq!(replay_json["message"].as_str(), Some("success"));
        assert_eq!(
            replay_json["request_id"].as_str(),
            Some(request_replay.as_str())
        );
        assert_eq!(
            replay_json["data"]["current_state"].as_str(),
            Some("buyer_locked")
        );
        assert_eq!(
            replay_json["data"]["payment_intent_id"].as_str(),
            Some(payment_intent_id.as_str())
        );

        let mismatch_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/lock"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_mismatch)
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"payment_intent_id":"{mismatch_intent_id}","lock_reason":"buyer_deposit_lock"}}"#
                    )))
                    .expect("mismatch request build"),
            )
            .await
            .expect("mismatch response");
        let mismatch_status = mismatch_response.status();
        let mismatch_body = to_bytes(mismatch_response.into_body(), usize::MAX)
            .await
            .expect("mismatch body");
        assert_eq!(
            mismatch_status,
            StatusCode::CONFLICT,
            "{}",
            String::from_utf8_lossy(&mismatch_body)
        );
        let mismatch_json: Value = serde_json::from_slice(&mismatch_body).expect("mismatch json");
        assert_eq!(mismatch_json["code"].as_str(), Some("BIL_PROVIDER_FAILED"));
        assert_eq!(
            mismatch_json["request_id"].as_str(),
            Some(request_mismatch.as_str())
        );

        let lock_db: (String, String, String, String) = {
            let row = client
                .query_one(
                    "SELECT payment_status,
                            COALESCE(payment_channel_snapshot ->> 'payment_intent_id', ''),
                            COALESCE(payment_channel_snapshot ->> 'provider_key', ''),
                            COALESCE(payment_channel_snapshot ->> 'lock_reason', '')
                     FROM trade.order_main
                     WHERE order_id = $1::text::uuid",
                    &[&order_id],
                )
                .await
                .expect("query order lock state");
            (row.get(0), row.get(1), row.get(2), row.get(3))
        };
        assert_eq!(lock_db.0, "locked");
        assert_eq!(lock_db.1, payment_intent_id);
        assert_eq!(lock_db.2, "mock_payment");
        assert_eq!(lock_db.3, "buyer_deposit_lock");

        let audit_summary: (i64, i64) = {
            let row = client
                .query_one(
                    "SELECT
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'order.payment.lock'),
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'order.payment.lock.idempotent_replay')",
                    &[&request_ok, &request_replay],
                )
                .await
                .expect("query audit summary");
            (row.get(0), row.get(1))
        };
        assert_eq!(audit_summary.0, 1);
        assert_eq!(audit_summary.1, 1);

        cleanup(
            &client,
            &[&payment_intent_id, &mismatch_intent_id],
            &[&order_graph, &other_order_graph],
            &[&buyer_org_id, &seller_org_id],
            &[&request_ok, &request_replay, &request_mismatch],
        )
        .await;
    }

    struct SeedOrderGraph {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    async fn seed_org(client: &db::Client, code: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (
                   org_name, org_type, status, metadata
                 ) VALUES (
                   $1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb
                 )
                 RETURNING org_id::text",
                &[&format!("{code}-name")],
            )
            .await
            .expect("seed org")
            .get(0)
    }

    async fn seed_provider_account(
        client: &db::Client,
        seller_org_id: &str,
        suffix: &str,
        provider_key: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key,
                   account_scope,
                   account_scope_id,
                   account_name,
                   settlement_subject_type,
                   settlement_subject_id,
                   jurisdiction_code,
                   account_mode,
                   status,
                   config_json
                 ) VALUES (
                   $1, 'tenant', $2::text::uuid, $3, 'organization', $2::text::uuid, 'SG', 'sandbox', 'active', '{}'::jsonb
                 )
                 RETURNING provider_account_id::text",
                &[
                    &provider_key,
                    &seller_org_id,
                    &format!("acct-{suffix}"),
                ],
            )
            .await
            .expect("seed provider account")
            .get(0)
    }

    async fn seed_order(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> SeedOrderGraph {
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil003-asset-{suffix}"),
                    &format!("bil003 asset {suffix}"),
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
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_download',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil003-product-{suffix}"),
                    &format!("bil003 product {suffix}"),
                    &format!("bil003 summary {suffix}"),
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
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("BIL003-SKU-{suffix}")],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status,
                   payment_status,
                   delivery_status,
                   acceptance_status,
                   settlement_status,
                   dispute_status,
                   payment_mode,
                   amount,
                   currency_code,
                   fee_preview_snapshot,
                   price_snapshot_json,
                   last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'contract_effective',
                   'unpaid',
                   'not_started',
                   'not_started',
                   'pending_settlement',
                   'none',
                   'online',
                   88.00,
                   'SGD',
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD',
                     'captured_at', '1776659000000',
                     'source', 'bil003-db-smoke'
                   ),
                   $6
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("bil003-order-{suffix}"),
                ],
            )
            .await
            .expect("seed order")
            .get(0);
        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn seed_payment_intent(
        client: &db::Client,
        order_id: &str,
        buyer_org_id: &str,
        seller_org_id: &str,
        provider_account_id: &str,
        suffix: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.payment_intent (
                   order_id,
                   intent_type,
                   provider_key,
                   provider_account_id,
                   payer_subject_type,
                   payer_subject_id,
                   payee_subject_type,
                   payee_subject_id,
                   payer_jurisdiction_code,
                   payee_jurisdiction_code,
                   launch_jurisdiction_code,
                   amount,
                   price_currency_code,
                   currency_code,
                   payment_method,
                   status,
                   idempotency_key,
                   capability_snapshot,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   'order_payment',
                   'mock_payment',
                   $2::text::uuid,
                   'organization',
                   $3::text::uuid,
                   'organization',
                   $4::text::uuid,
                   'SG',
                   'SG',
                   'SG',
                   88.00,
                   'USD',
                   'SGD',
                   'wallet',
                   'created',
                   $5,
                   '{}'::jsonb,
                   jsonb_build_object('source', 'bil003-db-smoke')
                 )
                 RETURNING payment_intent_id::text",
                &[
                    &order_id,
                    &provider_account_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("idem-bil003-{suffix}"),
                ],
            )
            .await
            .expect("seed payment intent")
            .get(0)
    }

    async fn cleanup(
        client: &db::Client,
        payment_intent_ids: &[&str],
        order_graphs: &[&SeedOrderGraph],
        org_ids: &[&str],
        request_ids: &[&str],
    ) {
        for payment_intent_id in payment_intent_ids {
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                    &[payment_intent_id],
                )
                .await;
        }
        for order_graph in order_graphs {
            let _ = client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[&order_graph.order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                    &[&order_graph.sku_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                    &[&order_graph.product_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                    &[&order_graph.asset_version_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                    &[&order_graph.asset_id],
                )
                .await;
        }
        for org_id in org_ids {
            let _ = client
                .execute(
                    "DELETE FROM payment.provider_account WHERE account_scope_id = $1::text::uuid",
                    &[org_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                    &[org_id],
                )
                .await;
        }
        for request_id in request_ids {
            let _ = client
                .execute(
                    "DELETE FROM audit.audit_event WHERE request_id = $1",
                    &[request_id],
                )
                .await;
        }
    }
}
