#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use crate::modules::storage::application::{delete_object, fetch_object_bytes};
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        platform_org_id: String,
        buyer_user_id: String,
        platform_user_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        payment_intent_id: String,
        settlement_id: String,
    }

    #[tokio::test]
    async fn bil013_dispute_case_db_smoke() {
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

        let create_request_id = format!("req-bil013-create-{suffix}");
        let dispute_case = create_case(&app, &seed, &create_request_id).await;
        let case_id = dispute_case["data"]["case_id"]
            .as_str()
            .expect("case id")
            .to_string();
        assert_eq!(
            dispute_case["data"]["current_status"].as_str(),
            Some("opened")
        );
        assert_eq!(
            dispute_case["data"]["reason_code"].as_str(),
            Some("delivery_failed")
        );

        let evidence_request_id = format!("req-bil013-evidence-{suffix}");
        let evidence = upload_evidence(&app, &seed, &case_id, &evidence_request_id).await;
        let evidence_id = evidence["data"]["evidence_id"]
            .as_str()
            .expect("evidence id")
            .to_string();
        let object_uri = evidence["data"]["object_uri"]
            .as_str()
            .expect("object uri")
            .to_string();
        assert_eq!(
            evidence["data"]["object_type"].as_str(),
            Some("delivery_receipt")
        );
        assert_eq!(evidence["data"]["idempotent_replay"].as_bool(), Some(false));

        let replay = upload_evidence(
            &app,
            &seed,
            &case_id,
            &format!("{evidence_request_id}-replay"),
        )
        .await;
        assert_eq!(
            replay["data"]["evidence_id"],
            evidence["data"]["evidence_id"]
        );
        assert_eq!(replay["data"]["idempotent_replay"].as_bool(), Some(true));

        let (bucket_name, object_key) = parse_s3_uri(&object_uri);
        let fetched = fetch_object_bytes(bucket_name.as_str(), object_key.as_str())
            .await
            .expect("fetch object bytes");
        assert_eq!(fetched.bytes, b"evidence-bil013");

        let resolve_request_id = format!("req-bil013-resolve-{suffix}");
        let resolution = resolve_case(&app, &seed, &case_id, &resolve_request_id).await;
        assert_eq!(
            resolution["data"]["current_status"].as_str(),
            Some("resolved")
        );
        assert_eq!(
            resolution["data"]["decision_code"].as_str(),
            Some("refund_full")
        );
        assert_eq!(resolution["data"]["step_up_bound"].as_bool(), Some(true));
        assert_eq!(
            resolution["data"]["idempotent_replay"].as_bool(),
            Some(false)
        );

        let resolution_replay = resolve_case(
            &app,
            &seed,
            &case_id,
            &format!("{resolve_request_id}-replay"),
        )
        .await;
        assert_eq!(
            resolution_replay["data"]["decision_id"],
            resolution["data"]["decision_id"]
        );
        assert_eq!(
            resolution_replay["data"]["idempotent_replay"].as_bool(),
            Some(true)
        );

        let case_row = client
            .query_one(
                "SELECT status, decision_code, penalty_code FROM support.dispute_case WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await
            .expect("query dispute case");
        let case_status: String = case_row.get(0);
        let decision_code: Option<String> = case_row.get(1);
        let penalty_code: Option<String> = case_row.get(2);
        assert_eq!(case_status, "resolved");
        assert_eq!(decision_code.as_deref(), Some("refund_full"));
        assert_eq!(penalty_code.as_deref(), Some("seller_warning"));

        let evidence_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM support.evidence_object WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await
            .expect("count evidence")
            .get(0);
        assert_eq!(evidence_count, 1);

        let order_dispute_status: String = client
            .query_one(
                "SELECT dispute_status FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order dispute status")
            .get(0);
        assert_eq!(order_dispute_status, "resolved");

        let outbox_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE aggregate_type = 'support.dispute_case'
                   AND aggregate_id = $1::text::uuid
                   AND event_type = ANY($2::text[])",
                &[
                    &case_id,
                    &vec![
                        "dispute.created".to_string(),
                        "dispute.resolved".to_string(),
                    ],
                ],
            )
            .await
            .expect("query outbox")
            .get(0);
        assert_eq!(outbox_count, 2);

        let audit_row = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1 AND action_name = 'dispute.case.create'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $2 AND action_name = 'dispute.evidence.upload'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'dispute.case.resolve')",
                &[&create_request_id, &evidence_request_id, &resolve_request_id],
            )
            .await
            .expect("query audit counts");
        let create_audit: i64 = audit_row.get(0);
        let upload_audit: i64 = audit_row.get(1);
        let resolve_audit: i64 = audit_row.get(2);
        assert_eq!(create_audit, 1);
        assert_eq!(upload_audit, 1);
        assert_eq!(resolve_audit, 1);

        cleanup(
            &client,
            &seed,
            &case_id,
            &evidence_id,
            bucket_name.as_str(),
            object_key.as_str(),
        )
        .await;
    }

    async fn create_case(app: &Router, seed: &SeedGraph, request_id: &str) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cases")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "order_id": seed.order_id,
                            "reason_code": "delivery_failed",
                            "requested_resolution": "refund_full",
                            "claimed_amount": "88.00000000",
                            "evidence_scope": "delivery_receipt,download_log",
                            "blocking_effect": "freeze_settlement",
                            "metadata": {
                                "entry": "bil013"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("create case request should build"),
            )
            .await
            .expect("create case response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create case body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("create case json")
    }

    async fn upload_evidence(
        app: &Router,
        seed: &SeedGraph,
        case_id: &str,
        request_id: &str,
    ) -> Value {
        let boundary = format!("BIL013-{}", request_id.replace(':', "-"));
        let body = [
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"object_type\"".to_string(),
            String::new(),
            "delivery_receipt".to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"metadata_json\"".to_string(),
            String::new(),
            json!({"entry":"bil013","kind":"upload"}).to_string(),
            format!("--{boundary}"),
            "Content-Disposition: form-data; name=\"file\"; filename=\"receipt.txt\"".to_string(),
            "Content-Type: text/plain".to_string(),
            String::new(),
            "evidence-bil013".to_string(),
            format!("--{boundary}--"),
            String::new(),
        ]
        .join("\r\n");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/cases/{case_id}/evidence"))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-user-id", &seed.buyer_user_id)
                    .header("x-request-id", request_id)
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("upload evidence request should build"),
            )
            .await
            .expect("upload evidence response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("upload evidence body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("upload evidence json")
    }

    async fn resolve_case(
        app: &Router,
        seed: &SeedGraph,
        case_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/cases/{case_id}/resolve"))
                    .header("x-role", "platform_risk_settlement")
                    .header("x-user-id", &seed.platform_user_id)
                    .header("x-step-up-token", "bil013-stepup")
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "decision_type": "manual_resolution",
                            "decision_code": "refund_full",
                            "liability_type": "seller",
                            "penalty_code": "seller_warning",
                            "decision_text": "refund approved after evidence review",
                            "metadata": {
                                "entry": "bil013"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("resolve request should build"),
            )
            .await
            .expect("resolve response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("resolve body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("resolve json")
    }

    async fn seed_graph(client: &db::Client, suffix: &str) -> SeedGraph {
        let buyer_org_id = seed_org(client, &format!("bil013-buyer-{suffix}"), "enterprise").await;
        let seller_org_id =
            seed_org(client, &format!("bil013-seller-{suffix}"), "enterprise").await;
        let platform_org_id =
            seed_org(client, &format!("bil013-platform-{suffix}"), "platform").await;
        let buyer_user_id = seed_user(client, &buyer_org_id, &format!("buyer-{suffix}")).await;
        let platform_user_id =
            seed_user(client, &platform_org_id, &format!("platform-{suffix}")).await;
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (owner_org_id, title, category, sensitivity_level, status, description)
                 VALUES ($1::text::uuid, $2, 'finance', 'internal', 'active', $3)
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil013-asset-{suffix}"),
                    &format!("bil013 asset {suffix}"),
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
                    &format!("bil013-product-{suffix}"),
                    &format!("bil013 product {suffix}"),
                    &format!("bil013 summary {suffix}"),
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
                &[&product_id, &format!("BIL013-SKU-{suffix}")],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id,
                   asset_version_id,
                   buyer_org_id,
                   seller_org_id,
                   sku_id,
                   status,
                   payment_status,
                   delivery_status,
                   acceptance_status,
                   settlement_status,
                   dispute_status,
                   payment_mode,
                   amount,
                   currency_code,
                   price_snapshot_json,
                   delivery_route_snapshot,
                   trust_boundary_snapshot,
                   last_reason_code
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4::text::uuid,
                   $5::text::uuid,
                   'buyer_locked',
                   'paid',
                   'delivered',
                   'accepted',
                   'pending_settlement',
                   'none',
                   'online',
                   88.00,
                   'SGD',
                   jsonb_build_object(
                     'product_id', $1,
                     'sku_id', $5,
                     'sku_type', 'FILE_STD',
                     'selected_sku_type', 'FILE_STD',
                     'billing_mode', 'one_time',
                     'pricing_mode', 'one_time',
                     'settlement_basis', 'gross_amount',
                     'refund_mode', 'manual_refund',
                     'refund_template', 'REFUND_FILE_V1',
                     'price_currency_code', 'SGD'
                   ),
                   'file_download',
                   '{"delivery_mode":"file_download"}'::jsonb,
                   'bil013_seed_dispute'
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
                   $1::text::uuid, 'order_payment', 'mock_payment', NULL, 'organization', $2::text::uuid,
                   'organization', $3::text::uuid, 'SG', 'SG', 'SG', 88.00, 'wallet', 'SGD', 'SGD',
                   'succeeded', $4, $5, '{"supports_refund":true}'::jsonb, '{}'::jsonb
                 ) RETURNING payment_intent_id::text"#,
                &[
                    &order_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &format!("bil013-pay-req-{suffix}"),
                    &format!("pay:bil013:{suffix}"),
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
                   'bil013_seed', NULL
                 ) RETURNING settlement_id::text",
                &[&order_id],
            )
            .await
            .expect("insert settlement")
            .get(0);

        SeedGraph {
            buyer_org_id,
            seller_org_id,
            platform_org_id,
            buyer_user_id,
            platform_user_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            payment_intent_id,
            settlement_id,
        }
    }

    async fn seed_org(client: &db::Client, org_name: &str, org_type: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, $2, 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name, &org_type],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn seed_user(client: &db::Client, org_id: &str, suffix: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status, email, attrs)
                 VALUES ($1::text::uuid, $2, $3, 'human', 'active', 'verified', $4, '{}'::jsonb)
                 RETURNING user_id::text",
                &[
                    &org_id,
                    &format!("bil013-user-{suffix}"),
                    &format!("BIL013 User {suffix}"),
                    &format!("bil013-{suffix}@example.com"),
                ],
            )
            .await
            .expect("insert user")
            .get(0)
    }

    async fn cleanup(
        client: &db::Client,
        seed: &SeedGraph,
        case_id: &str,
        evidence_id: &str,
        bucket_name: &str,
        object_key: &str,
    ) {
        let _ = delete_object(bucket_name, object_key).await;
        let _ = client
            .execute(
                "DELETE FROM support.decision_record WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM support.evidence_object WHERE evidence_id = $1::text::uuid",
                &[&evidence_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM support.dispute_case WHERE case_id = $1::text::uuid",
                &[&case_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id = $1::text::uuid OR ordering_key = $2",
                &[&case_id, &seed.order_id],
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
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.user_account WHERE user_id = ANY($1::uuid[])",
                &[&vec![
                    seed.buyer_user_id.clone(),
                    seed.platform_user_id.clone(),
                ]],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::uuid[])",
                &[&vec![
                    seed.buyer_org_id.clone(),
                    seed.seller_org_id.clone(),
                    seed.platform_org_id.clone(),
                ]],
            )
            .await;
    }

    fn parse_s3_uri(uri: &str) -> (String, String) {
        let without_scheme = uri.trim_start_matches("s3://");
        let mut parts = without_scheme.splitn(2, '/');
        let bucket = parts.next().unwrap_or_default().to_string();
        let key = parts.next().unwrap_or_default().to_string();
        (bucket, key)
    }
}
