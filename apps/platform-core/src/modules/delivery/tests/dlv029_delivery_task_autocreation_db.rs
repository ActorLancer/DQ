#[cfg(test)]
mod tests {
    use crate::modules::{billing, order};
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        file_std_product_id: String,
        file_std_sku_id: String,
        file_std_order_id: String,
        file_sub_product_id: String,
        file_sub_sku_id: String,
        file_sub_order_id: String,
        api_sub_product_id: String,
        api_sub_sku_id: String,
        api_sub_order_id: String,
        api_sub_payment_intent_id: String,
    }

    #[tokio::test]
    async fn dlv029_delivery_task_autocreation_db_smoke() {
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
        let app = app().await;

        let file_std_req = format!("req-dlv029-{suffix}-file-std");
        let file_std_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-std/transition",
                        seed.file_std_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &file_std_req)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"lock_funds"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("file std response");
        assert_eq!(file_std_response.status(), StatusCode::OK);

        let file_std_delivery = latest_delivery_record(&client, &seed.file_std_order_id)
            .await
            .expect("file std delivery");
        assert_eq!(
            file_std_delivery["delivery_type"].as_str(),
            Some("file_download")
        );
        assert_eq!(
            file_std_delivery["delivery_route"].as_str(),
            Some("signed_url")
        );
        assert_eq!(file_std_delivery["status"].as_str(), Some("prepared"));
        assert_eq!(
            file_std_delivery["executor_type"].as_str(),
            Some("seller_org")
        );
        assert_eq!(
            file_std_delivery["executor_ref_id"].as_str(),
            Some(seed.seller_org_id.as_str())
        );
        assert_eq!(
            file_std_delivery["trust_boundary_snapshot"]["delivery_task"]["creation_source"]
                .as_str(),
            Some("file_std_lock_funds")
        );
        assert_eq!(
            file_std_delivery["trust_boundary_snapshot"]["delivery_task"]["responsible_subject_id"]
                .as_str(),
            Some(seed.seller_org_id.as_str())
        );
        assert_eq!(
            file_std_delivery["trust_boundary_snapshot"]["delivery_task"]["retry_count"].as_i64(),
            Some(0)
        );
        assert_eq!(
            file_std_delivery["trust_boundary_snapshot"]["delivery_task"]["manual_takeover"]
                .as_bool(),
            Some(false)
        );
        assert_trade_audit(
            &client,
            &file_std_req,
            &seed.file_std_order_id,
            "trade.order.delivery_task.auto_created",
            "success",
        )
        .await;
        assert_delivery_task_outbox(
            &client,
            file_std_delivery["delivery_id"]
                .as_str()
                .expect("delivery id"),
            &seed.file_std_order_id,
            "FILE_STD",
            "file_std_lock_funds",
            "seller_org",
        )
        .await;

        let file_sub_req = format!("req-dlv029-{suffix}-file-sub-renew");
        let file_sub_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/file-sub/transition",
                        seed.file_sub_order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &file_sub_req)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"renew_subscription"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("file sub response");
        assert_eq!(file_sub_response.status(), StatusCode::OK);

        let file_sub_rows = delivery_records(&client, &seed.file_sub_order_id).await;
        assert_eq!(
            file_sub_rows.len(),
            2,
            "renewal should append a new prepared delivery task"
        );
        assert_eq!(file_sub_rows[0]["status"].as_str(), Some("prepared"));
        assert_eq!(
            file_sub_rows[0]["delivery_type"].as_str(),
            Some("revision_push")
        );
        assert_eq!(
            file_sub_rows[0]["delivery_route"].as_str(),
            Some("revision_event")
        );
        assert_eq!(file_sub_rows[1]["status"].as_str(), Some("committed"));
        assert_eq!(
            file_sub_rows[0]["trust_boundary_snapshot"]["delivery_task"]["creation_source"]
                .as_str(),
            Some("file_sub_subscription_lock")
        );
        assert_eq!(
            file_sub_rows[0]["executor_ref_id"].as_str(),
            Some(seed.seller_org_id.as_str())
        );
        assert_trade_audit(
            &client,
            &file_sub_req,
            &seed.file_sub_order_id,
            "trade.order.delivery_task.auto_created",
            "success",
        )
        .await;

        let webhook_req = format!("req-dlv029-{suffix}-api-sub-webhook");
        let occurred_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis() as i64;
        let webhook_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payments/webhooks/mock_payment")
                    .header("content-type", "application/json")
                    .header("x-provider-signature", "mock-signature")
                    .header("x-webhook-timestamp", occurred_at_ms.to_string())
                    .header("x-request-id", &webhook_req)
                    .body(Body::from(format!(
                        r#"{{
                          "provider_event_id":"evt-dlv029-api-sub-{suffix}",
                          "event_type":"payment.succeeded",
                          "payment_intent_id":"{}",
                          "provider_status":"succeeded",
                          "occurred_at_ms":{}
                        }}"#,
                        seed.api_sub_payment_intent_id, occurred_at_ms
                    )))
                    .expect("request should build"),
            )
            .await
            .expect("webhook response");
        assert_eq!(webhook_response.status(), StatusCode::OK);
        let webhook_body = to_bytes(webhook_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let webhook_json: Value = serde_json::from_slice(&webhook_body).expect("json");
        assert_eq!(
            webhook_json["data"]["processed_status"].as_str(),
            Some("processed")
        );
        assert_eq!(
            webhook_json["data"]["applied_payment_status"].as_str(),
            Some("succeeded")
        );

        let api_sub_delivery = latest_delivery_record(&client, &seed.api_sub_order_id)
            .await
            .expect("api sub delivery");
        assert_eq!(
            api_sub_delivery["delivery_type"].as_str(),
            Some("api_access")
        );
        assert_eq!(
            api_sub_delivery["delivery_route"].as_str(),
            Some("api_gateway")
        );
        assert_eq!(api_sub_delivery["status"].as_str(), Some("prepared"));
        assert_eq!(
            api_sub_delivery["executor_type"].as_str(),
            Some("buyer_org")
        );
        assert_eq!(
            api_sub_delivery["executor_ref_id"].as_str(),
            Some(seed.buyer_org_id.as_str())
        );
        assert_eq!(
            api_sub_delivery["trust_boundary_snapshot"]["delivery_task"]["creation_source"]
                .as_str(),
            Some("payment_result_orchestrator")
        );
        assert_eq!(
            api_sub_delivery["trust_boundary_snapshot"]["delivery_task"]["responsible_scope"]
                .as_str(),
            Some("buyer_api_binding")
        );
        assert_trade_audit(
            &client,
            &webhook_req,
            &seed.api_sub_order_id,
            "trade.order.delivery_task.auto_created",
            "success",
        )
        .await;
        assert_delivery_task_outbox(
            &client,
            api_sub_delivery["delivery_id"]
                .as_str()
                .expect("delivery id"),
            &seed.api_sub_order_id,
            "API_SUB",
            "payment_result_orchestrator",
            "buyer_org",
        )
        .await;

        cleanup_seed_graph(
            &client,
            &seed,
            &[&file_std_req, &file_sub_req, &webhook_req],
        )
        .await;
    }

    async fn app() -> Router {
        crate::with_live_test_state(
            Router::new()
                .merge(billing::api::router())
                .merge(order::api::router()),
        )
        .await
    }

    async fn latest_delivery_record(client: &Client, order_id: &str) -> Option<Value> {
        client
            .query_opt(
                "SELECT delivery_id::text,
                        delivery_type,
                        delivery_route,
                        executor_type,
                        executor_ref_id::text,
                        status,
                        trust_boundary_snapshot
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&order_id],
            )
            .await
            .expect("query latest delivery")
            .map(|row| {
                json!({
                    "delivery_id": row.get::<_, String>(0),
                    "delivery_type": row.get::<_, String>(1),
                    "delivery_route": row.get::<_, Option<String>>(2),
                    "executor_type": row.get::<_, String>(3),
                    "executor_ref_id": row.get::<_, Option<String>>(4),
                    "status": row.get::<_, String>(5),
                    "trust_boundary_snapshot": row.get::<_, Value>(6),
                })
            })
    }

    async fn delivery_records(client: &Client, order_id: &str) -> Vec<Value> {
        client
            .query(
                "SELECT delivery_id::text,
                        delivery_type,
                        delivery_route,
                        executor_type,
                        executor_ref_id::text,
                        status,
                        trust_boundary_snapshot
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, delivery_id DESC",
                &[&order_id],
            )
            .await
            .expect("query deliveries")
            .into_iter()
            .map(|row| {
                json!({
                    "delivery_id": row.get::<_, String>(0),
                    "delivery_type": row.get::<_, String>(1),
                    "delivery_route": row.get::<_, Option<String>>(2),
                    "executor_type": row.get::<_, String>(3),
                    "executor_ref_id": row.get::<_, Option<String>>(4),
                    "status": row.get::<_, String>(5),
                    "trust_boundary_snapshot": row.get::<_, Value>(6),
                })
            })
            .collect()
    }

    async fn assert_trade_audit(
        client: &Client,
        request_id: &str,
        order_id: &str,
        action_name: &str,
        result_code: &str,
    ) {
        let row = client
            .query_one(
                "SELECT action_name, result_code
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND ref_type = 'order'
                   AND ref_id = $2::text::uuid
                   AND action_name = $3
                 ORDER BY event_time DESC
                 LIMIT 1",
                &[&request_id, &order_id, &action_name],
            )
            .await
            .expect("query trade audit");
        assert_eq!(row.get::<_, String>(0), action_name);
        assert_eq!(row.get::<_, String>(1), result_code);
    }

    async fn assert_delivery_task_outbox(
        client: &Client,
        delivery_id: &str,
        order_id: &str,
        sku_type: &str,
        creation_source: &str,
        executor_type: &str,
    ) {
        let row = client
            .query_one(
                "SELECT event_type, target_topic, payload
                 FROM ops.outbox_event
                 WHERE aggregate_id = $1::text::uuid
                   AND event_type = 'delivery.task.auto_created'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&delivery_id],
            )
            .await
            .expect("query outbox row");
        assert_eq!(row.get::<_, String>(0), "delivery.task.auto_created");
        assert_eq!(
            row.get::<_, Option<String>>(1).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        let payload: Value = row.get(2);
        assert_eq!(payload["order_id"].as_str(), Some(order_id));
        assert_eq!(payload["sku_type"].as_str(), Some(sku_type));
        assert_eq!(payload["creation_source"].as_str(), Some(creation_source));
        assert_eq!(payload["executor_type"].as_str(), Some(executor_type));
        assert_eq!(payload["initial_status"].as_str(), Some("prepared"));
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id = insert_org(client, &format!("dlv029-buyer-{suffix}")).await?;
        let seller_org_id = insert_org(client, &format!("dlv029-seller-{suffix}")).await?;
        let asset_id = insert_asset(client, &seller_org_id, suffix).await?;
        let asset_version_id = insert_asset_version(client, &asset_id).await?;

        let file_std_product_id = insert_product(
            client,
            &asset_id,
            &asset_version_id,
            &seller_org_id,
            suffix,
            "FILE_STD",
            "file_download",
            "dlv029 file std",
        )
        .await?;
        let file_std_sku_id = insert_sku(client, &file_std_product_id, suffix, "FILE_STD").await?;
        let file_std_order_id = insert_order(
            client,
            &file_std_product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &file_std_sku_id,
            &format!("DLV029-FILE-STD-{suffix}"),
            "FILE_STD",
            "S2",
            "行业原始数据文件下载",
            "contract_effective",
            "unpaid",
            "not_started",
            "not_started",
            "not_started",
            "none",
            Some("dlv029_seed_contract_effective"),
        )
        .await?;
        insert_signed_contract(client, &file_std_order_id, suffix).await?;

        let file_sub_product_id = insert_product(
            client,
            &asset_id,
            &asset_version_id,
            &seller_org_id,
            suffix,
            "FILE_SUB",
            "file_download",
            "dlv029 file sub",
        )
        .await?;
        let file_sub_sku_id = insert_sku(client, &file_sub_product_id, suffix, "FILE_SUB").await?;
        let file_sub_order_id = insert_order(
            client,
            &file_sub_product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &file_sub_sku_id,
            &format!("DLV029-FILE-SUB-{suffix}"),
            "FILE_SUB",
            "S2",
            "行业原始数据订阅更新",
            "expired",
            "paid",
            "expired",
            "expired",
            "expired",
            "none",
            Some("dlv029_seed_expired"),
        )
        .await?;
        insert_signed_contract(client, &file_sub_order_id, suffix).await?;
        insert_committed_delivery_record(
            client,
            &file_sub_order_id,
            "revision_push",
            "revision_event",
            suffix,
        )
        .await?;

        let api_sub_product_id = insert_product(
            client,
            &asset_id,
            &asset_version_id,
            &seller_org_id,
            suffix,
            "API_SUB",
            "api_service",
            "dlv029 api sub",
        )
        .await?;
        let api_sub_sku_id = insert_sku(client, &api_sub_product_id, suffix, "API_SUB").await?;
        let api_sub_order_id = insert_order(
            client,
            &api_sub_product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &api_sub_sku_id,
            &format!("DLV029-API-SUB-{suffix}"),
            "API_SUB",
            "S1",
            "工业设备运行指标 API 订阅",
            "contract_effective",
            "unpaid",
            "not_started",
            "not_started",
            "not_started",
            "none",
            Some("dlv029_seed_contract_effective"),
        )
        .await?;
        insert_signed_contract(client, &api_sub_order_id, suffix).await?;
        let api_sub_payment_intent_id = insert_payment_intent(
            client,
            &api_sub_order_id,
            &buyer_org_id,
            &seller_org_id,
            suffix,
        )
        .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            file_std_product_id,
            file_std_sku_id,
            file_std_order_id,
            file_sub_product_id,
            file_sub_sku_id,
            file_sub_order_id,
            api_sub_product_id,
            api_sub_sku_id,
            api_sub_order_id,
            api_sub_payment_intent_id,
        })
    }

    async fn insert_org(client: &Client, org_name: &str) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_asset(
        client: &Client,
        seller_org_id: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("dlv029-asset-{suffix}"),
                    &format!("dlv029 asset {suffix}"),
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_asset_version(client: &Client, asset_id: &str) -> Result<String, db::Error> {
        client
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
            .await
            .map(|row| row.get(0))
    }

    async fn insert_product(
        client: &Client,
        asset_id: &str,
        asset_version_id: &str,
        seller_org_id: &str,
        suffix: &str,
        _sku_type: &str,
        delivery_type: &str,
        label: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 88.00, 'CNY', $6,
                   ARRAY['internal_use']::text[], $7,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("{label}-{suffix}"),
                    &format!("{label} {suffix}"),
                    &delivery_type,
                    &format!("{label} search {suffix}"),
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_sku(
        client: &Client,
        product_id: &str,
        suffix: &str,
        sku_type: &str,
    ) -> Result<String, db::Error> {
        let (billing_mode, acceptance_mode, refund_mode, unit_name) = match sku_type {
            "FILE_SUB" => ("subscription", "manual_accept", "manual_refund", "期"),
            "API_SUB" => ("subscription", "auto_accept", "manual_refund", "月"),
            _ => ("one_time", "manual_accept", "manual_refund", "份"),
        };
        client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6, $7, 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("{sku_type}-{suffix}"),
                    &sku_type,
                    &unit_name,
                    &billing_mode,
                    &acceptance_mode,
                    &refund_mode,
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_order(
        client: &Client,
        product_id: &str,
        asset_version_id: &str,
        buyer_org_id: &str,
        seller_org_id: &str,
        sku_id: &str,
        sku_code: &str,
        sku_type: &str,
        scenario_code: &str,
        scenario_name: &str,
        status: &str,
        payment_status: &str,
        delivery_status: &str,
        acceptance_status: &str,
        settlement_status: &str,
        dispute_status: &str,
        last_reason_code: Option<&str>,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, $7, $8, $9, $10, $11,
                   'online', 88.00, 'CNY', $12::jsonb, $13
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &status,
                    &payment_status,
                    &delivery_status,
                    &acceptance_status,
                    &settlement_status,
                    &dispute_status,
                    &price_snapshot(sku_id, sku_code, sku_type, scenario_code, scenario_name),
                    &last_reason_code,
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_signed_contract(
        client: &Client,
        order_id: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{}'::jsonb
                 )
                 RETURNING contract_id::text",
                &[
                    &order_id,
                    &format!("sha256:dlv029-contract:{suffix}:{order_id}"),
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_committed_delivery_record(
        client: &Client,
        order_id: &str,
        delivery_type: &str,
        delivery_route: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, executor_type, status,
                   delivery_commit_hash, receipt_hash, committed_at, expires_at, trust_boundary_snapshot
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'seller_org', 'committed',
                   $4, $5, now() - interval '1 day', now() + interval '1 day',
                   '{\"legacy_cycle\":true}'::jsonb
                 )
                 RETURNING delivery_id::text",
                &[
                    &order_id,
                    &delivery_type,
                    &delivery_route,
                    &format!("dlv029-commit-{suffix}-{order_id}"),
                    &format!("dlv029-receipt-{suffix}-{order_id}"),
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_payment_intent(
        client: &Client,
        order_id: &str,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO payment.payment_intent (
                   order_id, intent_type, provider_key, payer_subject_type, payer_subject_id,
                   payee_subject_type, payee_subject_id, payer_jurisdiction_code, payee_jurisdiction_code,
                   launch_jurisdiction_code, amount, payment_method, currency_code, price_currency_code,
                 status, request_id, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_payment', 'mock_payment', 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'CNY', 'CNY',
                   'processing', $4, '{}'::jsonb
                 )
                 RETURNING payment_intent_id::text",
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("req-dlv029-intent-{suffix}"),
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    fn price_snapshot(
        sku_id: &str,
        sku_code: &str,
        sku_type: &str,
        scenario_code: &str,
        scenario_name: &str,
    ) -> Value {
        let (
            primary_sku,
            supplementary_skus,
            selected_role,
            contract_template,
            acceptance_template,
            refund_template,
        ) = match sku_type {
            "API_SUB" => (
                "API_SUB",
                json!(["API_PPU"]),
                "primary",
                "CONTRACT_API_SUB_V1",
                "ACCEPT_API_SUB_V1",
                "REFUND_API_SUB_V1",
            ),
            "FILE_SUB" => (
                "FILE_SUB",
                json!([]),
                "primary",
                "CONTRACT_FILE_SUB_V1",
                "ACCEPT_FILE_SUB_V1",
                "REFUND_FILE_SUB_V1",
            ),
            _ => (
                "FILE_STD",
                json!([]),
                "primary",
                "CONTRACT_FILE_STD_V1",
                "ACCEPT_FILE_STD_V1",
                "REFUND_FILE_STD_V1",
            ),
        };
        json!({
            "product_id": "00000000-0000-0000-0000-000000000001",
            "sku_id": sku_id,
            "sku_code": sku_code,
            "sku_type": sku_type,
            "pricing_mode": if sku_type == "FILE_SUB" || sku_type == "API_SUB" { "subscription" } else { "one_time" },
            "unit_price": "88.00",
            "currency_code": "CNY",
            "billing_mode": if sku_type == "FILE_SUB" || sku_type == "API_SUB" { "subscription" } else { "one_time" },
            "refund_mode": "manual_refund",
            "settlement_terms": {
                "settlement_basis": if sku_type == "API_SUB" { "subscription_cycle" } else { "manual_acceptance" },
                "settlement_mode": "manual_v1"
            },
            "tax_terms": {
                "tax_policy": "platform_default",
                "tax_code": "VAT",
                "tax_inclusive": false
            },
            "scenario_snapshot": {
                "scenario_code": scenario_code,
                "scenario_name": scenario_name,
                "selected_sku_id": sku_id,
                "selected_sku_code": sku_code,
                "selected_sku_type": sku_type,
                "selected_sku_role": selected_role,
                "primary_sku": primary_sku,
                "supplementary_skus": supplementary_skus,
                "contract_template": contract_template,
                "acceptance_template": acceptance_template,
                "refund_template": refund_template,
                "per_sku_snapshot_required": true,
                "multi_sku_requires_independent_contract_authorization_settlement": true
            },
            "captured_at": "2026-01-01T00:00:00.000Z",
            "source": "catalog_quote_snapshot"
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph, request_ids: &[&str]) {
        let request_id_params = request_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();
        for delivery_id in [
            latest_delivery_record(client, &seed.file_std_order_id)
                .await
                .and_then(|row| row["delivery_id"].as_str().map(str::to_string)),
            latest_delivery_record(client, &seed.file_sub_order_id)
                .await
                .and_then(|row| row["delivery_id"].as_str().map(str::to_string)),
            latest_delivery_record(client, &seed.api_sub_order_id)
                .await
                .and_then(|row| row["delivery_id"].as_str().map(str::to_string)),
        ]
        .into_iter()
        .flatten()
        {
            let _ = client
                .execute(
                    "DELETE FROM ops.outbox_event WHERE aggregate_id = $1::text::uuid",
                    &[&delivery_id],
                )
                .await;
        }

        let _ = client
            .execute(
                "DELETE FROM payment.payment_webhook_event WHERE payment_intent_id = $1::text::uuid",
                &[&seed.api_sub_payment_intent_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.payment_intent WHERE payment_intent_id = $1::text::uuid",
                &[&seed.api_sub_payment_intent_id],
            )
            .await;

        for order_id in [
            &seed.file_std_order_id,
            &seed.file_sub_order_id,
            &seed.api_sub_order_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM contract.digital_contract WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
        }

        for product_id in [
            &seed.file_std_product_id,
            &seed.file_sub_product_id,
            &seed.api_sub_product_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM catalog.product_sku WHERE product_id = $1::text::uuid",
                    &[product_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                    &[product_id],
                )
                .await;
        }

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
                "DELETE FROM audit.audit_event WHERE request_id = ANY($1::text[])",
                &[&request_id_params],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::text::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
