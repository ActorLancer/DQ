#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn bil002_payment_intent_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil002-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil002-seller-{suffix}")).await;
        let provider_account_id =
            seed_provider_account(&client, &seller_org_id, &suffix, "mock_payment").await;
        let corridor_policy_id = active_corridor_policy_id(&client).await;
        let order = seed_order(&client, &buyer_org_id, &seller_org_id, &suffix).await;
        let order_id = order.order_id.clone();

        let request_create = format!("req-bil002-create-{suffix}");
        let request_get = format!("req-bil002-read-{suffix}");
        let request_cancel = format!("req-bil002-cancel-{suffix}");
        let request_cancel_replay = format!("req-bil002-cancel-replay-{suffix}");
        let request_create_replay = format!("req-bil002-replay-{suffix}");
        let idempotency_key = format!("pay:{order_id}:order_payment:1");

        let app = crate::with_live_test_state(router()).await;
        let create_payload = format!(
            r#"{{
              "order_id":"{order_id}",
              "intent_type":"order_payment",
              "provider_key":"mock_payment",
              "provider_account_id":"{provider_account_id}",
              "payer_subject_type":"organization",
              "payer_subject_id":"{buyer_org_id}",
              "payee_subject_type":"organization",
              "payee_subject_id":"{seller_org_id}",
              "payer_jurisdiction_code":"SG",
              "payee_jurisdiction_code":"SG",
              "launch_jurisdiction_code":"SG",
              "corridor_policy_id":"{corridor_policy_id}",
              "payment_amount":"88.00",
              "price_currency_code":"USD",
              "currency_code":"SGD",
              "payment_method":"wallet",
              "expire_at":"2026-04-20T12:00:00Z",
              "metadata":{{"source":"bil002-db-smoke"}}
            }}"#
        );

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/intents")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-step-up-token", "bil002-stepup")
                    .header("x-request-id", &request_create)
                    .header("x-idempotency-key", &idempotency_key)
                    .header("content-type", "application/json")
                    .body(Body::from(create_payload.clone()))
                    .expect("create request should build"),
            )
            .await
            .expect("create response");
        let create_status = create_response.status();
        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create body");
        assert_eq!(
            create_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_body)
        );
        let create_json: Value = serde_json::from_slice(&create_body).expect("create json");
        let payment_intent_id = create_json["data"]["payment_intent_id"]
            .as_str()
            .expect("payment intent id")
            .to_string();
        assert_eq!(
            create_json["data"]["provider_account_id"].as_str(),
            Some(provider_account_id.as_str())
        );
        assert_eq!(
            create_json["data"]["payment_status"].as_str(),
            Some("created")
        );

        let replay_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/intents")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-step-up-token", "bil002-stepup")
                    .header("x-request-id", &request_create_replay)
                    .header("x-idempotency-key", &idempotency_key)
                    .header("content-type", "application/json")
                    .body(Body::from(create_payload))
                    .expect("replay request should build"),
            )
            .await
            .expect("replay response");
        let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .expect("replay body");
        let replay_json: Value = serde_json::from_slice(&replay_body).expect("replay json");
        assert_eq!(
            replay_json["data"]["payment_intent_id"].as_str(),
            Some(payment_intent_id.as_str())
        );

        seed_transaction_and_webhook(&client, &payment_intent_id, &suffix).await;

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/payments/intents/{payment_intent_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_get)
                    .body(Body::empty())
                    .expect("get request should build"),
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
            get_json["data"]["payment_intent"]["payment_intent_id"].as_str(),
            Some(payment_intent_id.as_str())
        );
        assert_eq!(
            get_json["data"]["latest_transaction_summary"]["transaction_type"].as_str(),
            Some("payin")
        );
        assert_eq!(
            get_json["data"]["webhook_summary"]["event_type"].as_str(),
            Some("payment.succeeded")
        );

        let cancel_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/payments/intents/{payment_intent_id}/cancel"
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_cancel)
                    .body(Body::empty())
                    .expect("cancel request should build"),
            )
            .await
            .expect("cancel response");
        let cancel_status = cancel_response.status();
        let cancel_body = to_bytes(cancel_response.into_body(), usize::MAX)
            .await
            .expect("cancel body");
        assert_eq!(
            cancel_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&cancel_body)
        );
        let cancel_json: Value = serde_json::from_slice(&cancel_body).expect("cancel json");
        assert_eq!(
            cancel_json["data"]["payment_status"].as_str(),
            Some("canceled")
        );

        let cancel_replay_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/payments/intents/{payment_intent_id}/cancel"
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &buyer_org_id)
                    .header("x-request-id", &request_cancel_replay)
                    .body(Body::empty())
                    .expect("cancel replay request should build"),
            )
            .await
            .expect("cancel replay response");
        let cancel_replay_body = to_bytes(cancel_replay_response.into_body(), usize::MAX)
            .await
            .expect("cancel replay body");
        let cancel_replay_json: Value =
            serde_json::from_slice(&cancel_replay_body).expect("cancel replay json");
        assert_eq!(
            cancel_replay_json["data"]["payment_status"].as_str(),
            Some("canceled")
        );

        let intent_db: (String, String, String, String) = {
            let row = client
                .query_one(
                    "SELECT status, provider_account_id::text, corridor_policy_id::text, COALESCE(metadata ->> 'source', '')
                     FROM payment.payment_intent
                     WHERE payment_intent_id = $1::text::uuid",
                    &[&payment_intent_id],
                )
                .await
                .expect("query payment intent");
            (row.get(0), row.get(1), row.get(2), row.get(3))
        };
        assert_eq!(intent_db.0, "canceled");
        assert_eq!(intent_db.1, provider_account_id);
        assert_eq!(intent_db.2, corridor_policy_id);
        assert_eq!(intent_db.3, "bil002-db-smoke");

        let audit_summary: (i64, i64, i64, i64, i64) = {
            let row = client
                .query_one(
                    "SELECT
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'payment.intent.create'),
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE action_name = 'payment.intent.create.idempotent_replay' AND ref_id = $2::text::uuid),
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'payment.intent.read'),
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $4 AND action_name = 'payment.intent.cancel'),
                       (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $5 AND action_name = 'payment.intent.cancel.idempotent_replay')",
                    &[&request_create, &payment_intent_id, &request_get, &request_cancel, &request_cancel_replay],
                )
                .await
                .expect("query audit summary");
            (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
        };
        assert!(audit_summary.0 >= 1);
        assert!(audit_summary.1 >= 1);
        assert!(audit_summary.2 >= 1);
        assert!(audit_summary.3 >= 1);
        assert!(audit_summary.4 >= 1);

        cleanup_seed(
            &client,
            &buyer_org_id,
            &seller_org_id,
            &provider_account_id,
            &payment_intent_id,
            &order,
            &[
                request_create.as_str(),
                request_get.as_str(),
                request_cancel.as_str(),
                request_cancel_replay.as_str(),
                request_create_replay.as_str(),
            ],
        )
        .await;
    }

    async fn seed_org(client: &db::Client, org_name: &str) -> String {
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

    async fn seed_provider_account(
        client: &db::Client,
        subject_id: &str,
        suffix: &str,
        provider_key: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key, account_scope, account_scope_id, account_name,
                   settlement_subject_type, settlement_subject_id, jurisdiction_code,
                   account_mode, status, config_json
                 ) VALUES (
                   $1, 'tenant', $2::text::uuid, $3,
                   'organization', $2::text::uuid, 'SG',
                   'sandbox', 'active', '{}'::jsonb
                 )
                 RETURNING provider_account_id::text",
                &[
                    &provider_key,
                    &subject_id,
                    &format!("bil002-account-{suffix}"),
                ],
            )
            .await
            .expect("insert provider account")
            .get(0)
    }

    async fn active_corridor_policy_id(client: &db::Client) -> String {
        client
            .query_one(
                "SELECT corridor_policy_id::text
                 FROM payment.corridor_policy
                 WHERE status = 'active'
                   AND COALESCE((policy_snapshot ->> 'real_payment_enabled')::boolean, false) = true
                 ORDER BY effective_from DESC NULLS LAST, created_at DESC
                 LIMIT 1",
                &[],
            )
            .await
            .expect("query active corridor")
            .get(0)
    }

    async fn seed_order(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> SeedOrderGraph {
        let buyer_org_id_owned = buyer_org_id.to_string();
        let seller_org_id_owned = seller_org_id.to_string();
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id_owned,
                    &format!("bil002-asset-{suffix}"),
                    &format!("bil002 asset {suffix}"),
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
                    &seller_org_id_owned,
                    &format!("bil002-product-{suffix}"),
                    &format!("bil002 product {suffix}"),
                    &format!("bil002 summary {suffix}"),
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
                &[&product_id, &format!("BIL002-SKU-{suffix}")],
            )
            .await
            .expect("insert sku")
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'contract_effective', 'unpaid', 'not_started', 'not_started', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
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
                     'source', 'bil002-db-smoke'
                   ),
                   $6
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id_owned,
                    &seller_org_id_owned,
                    &sku_id,
                    &format!("bil002-order-{suffix}"),
                ],
            )
            .await
            .expect("insert order")
            .get(0);

        SeedOrderGraph {
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn seed_transaction_and_webhook(
        client: &db::Client,
        payment_intent_id: &str,
        suffix: &str,
    ) {
        let payment_transaction_id: String = client
            .query_one(
                "INSERT INTO payment.payment_transaction (
                   payment_intent_id, transaction_type, direction, provider_transaction_no,
                   provider_status, amount, currency_code, channel_fee_amount, settled_amount,
                   raw_payload
                 ) VALUES (
                   $1::text::uuid, 'payin', 'inbound', $2, 'SUCCESS', 88.00, 'SGD', 1.00, 87.00,
                   jsonb_build_object('source', 'bil002-db-smoke')
                 )
                 RETURNING payment_transaction_id::text",
                &[&payment_intent_id, &format!("txn-bil002-{suffix}")],
            )
            .await
            .expect("insert payment transaction")
            .get(0);

        let _ = client
            .execute(
                "INSERT INTO payment.payment_webhook_event (
                   provider_key, provider_event_id, event_type, signature_verified,
                   payment_intent_id, payment_transaction_id, payload, processed_status,
                   duplicate_flag, processed_at
                 ) VALUES (
                   'mock_payment', $1, 'payment.succeeded', true,
                   $2::text::uuid, $3::text::uuid, jsonb_build_object('source', 'bil002-db-smoke'),
                   'processed', false, now()
                 )",
                &[
                    &format!("evt-bil002-{suffix}"),
                    &payment_intent_id,
                    &payment_transaction_id,
                ],
            )
            .await
            .expect("insert webhook event");
    }

    async fn cleanup_seed(
        client: &db::Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        provider_account_id: &str,
        payment_intent_id: &str,
        order: &SeedOrderGraph,
        request_ids: &[&str],
    ) {
        for request_id in request_ids {
            let _ = client
                .execute(
                    "DELETE FROM audit.audit_event WHERE request_id = $1",
                    &[request_id],
                )
                .await;
        }
        let _ = client
            .execute(
                "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_transaction WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.provider_account WHERE provider_account_id = $1::text::uuid",
                &[&provider_account_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&order.sku_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&order.product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&order.asset_version_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&order.asset_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&buyer_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&seller_org_id],
            )
            .await;
    }
}
