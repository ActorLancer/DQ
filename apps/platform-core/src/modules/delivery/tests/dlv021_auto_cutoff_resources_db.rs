#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::delivery::repo::load_download_ticket_cache;
    use crate::modules::order::api::router as order_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        file_sku_id: String,
        share_sku_id: String,
        api_sku_id: String,
        sbx_sku_id: String,
        asset_object_id: String,
        storage_namespace_id: String,
        object_id: String,
        envelope_id: String,
        app_id: String,
        connector_id: String,
        environment_id: String,
        file_order_id: String,
        file_delivery_id: String,
        file_ticket_id: String,
        share_expire_order_id: String,
        share_dispute_order_id: String,
        api_order_id: String,
        sandbox_order_id: String,
    }

    #[tokio::test]
    async fn dlv021_auto_cutoff_resources_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_millis()
            .to_string();
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let app = crate::with_live_test_state(delivery_router().merge(order_router())).await;

        let ticket_req_id = format!("req-dlv021-{suffix}-ticket");
        let ticket_resp = call(
            &app,
            "GET",
            format!("/api/v1/orders/{}/download-ticket", seed.file_order_id),
            &seed.buyer_org_id,
            &ticket_req_id,
            None,
        )
        .await;
        assert_eq!(ticket_resp.status, StatusCode::OK, "{}", ticket_resp.body);
        let ticket_json: Value = serde_json::from_str(&ticket_resp.body).expect("ticket json");
        let download_token = ticket_json["data"]["data"]["download_token"]
            .as_str()
            .expect("download token")
            .to_string();
        assert!(
            load_download_ticket_cache(&seed.file_ticket_id)
                .await
                .expect("load cache")
                .is_some(),
            "download ticket cache should exist before cutoff"
        );

        let file_req_id = format!("req-dlv021-{suffix}-file-refund");
        let file_resp = call(
            &app,
            "POST",
            format!("/api/v1/orders/{}/file-std/transition", seed.file_order_id),
            &seed.buyer_org_id,
            &file_req_id,
            Some(r#"{"action":"request_refund"}"#),
        )
        .await;
        assert_eq!(file_resp.status, StatusCode::OK, "{}", file_resp.body);

        let file_ticket_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_ticket WHERE ticket_id = $1::text::uuid",
                &[&seed.file_ticket_id],
            )
            .await
            .expect("file ticket row");
        assert_eq!(file_ticket_row.get::<_, String>(0), "revoked");
        let file_delivery_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_record WHERE delivery_id = $1::text::uuid",
                &[&seed.file_delivery_id],
            )
            .await
            .expect("file delivery row");
        assert_eq!(file_delivery_row.get::<_, String>(0), "revoked");
        assert!(
            load_download_ticket_cache(&seed.file_ticket_id)
                .await
                .expect("load cache after cutoff")
                .is_none(),
            "download ticket cache should be removed after cutoff"
        );

        let download_resp = call(
            &app,
            "GET",
            format!(
                "/api/v1/orders/{}/download?ticket={}",
                seed.file_order_id, download_token
            ),
            &seed.buyer_org_id,
            &format!("req-dlv021-{suffix}-download-after-cutoff"),
            None,
        )
        .await;
        assert_eq!(
            download_resp.status,
            StatusCode::CONFLICT,
            "{}",
            download_resp.body
        );
        assert!(
            download_resp
                .body
                .contains("ticket cache not found or expired")
        );

        let share_expire_req_id = format!("req-dlv021-{suffix}-share-expire");
        let share_expire_resp = call(
            &app,
            "POST",
            format!(
                "/api/v1/orders/{}/share-ro/transition",
                seed.share_expire_order_id
            ),
            &seed.buyer_org_id,
            &share_expire_req_id,
            Some(r#"{"action":"expire_share"}"#),
        )
        .await;
        assert_eq!(
            share_expire_resp.status,
            StatusCode::OK,
            "{}",
            share_expire_resp.body
        );
        assert_share_status(&client, &seed.share_expire_order_id, "expired", "expired").await;

        let share_dispute_req_id = format!("req-dlv021-{suffix}-share-dispute");
        let share_dispute_resp = call(
            &app,
            "POST",
            format!(
                "/api/v1/orders/{}/share-ro/transition",
                seed.share_dispute_order_id
            ),
            &seed.buyer_org_id,
            &share_dispute_req_id,
            Some(r#"{"action":"interrupt_dispute"}"#),
        )
        .await;
        assert_eq!(
            share_dispute_resp.status,
            StatusCode::OK,
            "{}",
            share_dispute_resp.body
        );
        assert_share_status(
            &client,
            &seed.share_dispute_order_id,
            "suspended",
            "suspended",
        )
        .await;

        let api_req_id = format!("req-dlv021-{suffix}-api-risk");
        let api_resp = call(
            &app,
            "POST",
            format!("/api/v1/orders/{}/api-ppu/transition", seed.api_order_id),
            &seed.buyer_org_id,
            &api_req_id,
            Some(r#"{"action":"disable_access"}"#),
        )
        .await;
        assert_eq!(api_resp.status, StatusCode::OK, "{}", api_resp.body);
        let api_row = client
            .query_one(
                "SELECT status, to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM delivery.api_credential
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, api_credential_id DESC
                 LIMIT 1",
                &[&seed.api_order_id],
            )
            .await
            .expect("api credential row");
        assert_eq!(api_row.get::<_, String>(0), "suspended");
        assert!(api_row.get::<_, Option<String>>(1).is_some());
        let api_delivery_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid AND delivery_type = 'api_access'
                 ORDER BY updated_at DESC, delivery_id DESC LIMIT 1",
                &[&seed.api_order_id],
            )
            .await
            .expect("api delivery row");
        assert_eq!(api_delivery_row.get::<_, String>(0), "suspended");

        let sandbox_req_id = format!("req-dlv021-{suffix}-sandbox-expire");
        let sandbox_resp = call(
            &app,
            "POST",
            format!(
                "/api/v1/orders/{}/sbx-std/transition",
                seed.sandbox_order_id
            ),
            &seed.buyer_org_id,
            &sandbox_req_id,
            Some(r#"{"action":"expire_sandbox"}"#),
        )
        .await;
        assert_eq!(sandbox_resp.status, StatusCode::OK, "{}", sandbox_resp.body);
        let sandbox_workspace_row = client
            .query_one(
                "SELECT status FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid",
                &[&seed.sandbox_order_id],
            )
            .await
            .expect("sandbox workspace row");
        assert_eq!(sandbox_workspace_row.get::<_, String>(0), "expired");
        let sandbox_session_row = client
            .query_one(
                "SELECT session_status, to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM delivery.sandbox_session
                 WHERE sandbox_workspace_id IN (
                   SELECT sandbox_workspace_id FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid
                 )",
                &[&seed.sandbox_order_id],
            )
            .await
            .expect("sandbox session row");
        assert_eq!(sandbox_session_row.get::<_, String>(0), "expired");
        assert!(sandbox_session_row.get::<_, Option<String>>(1).is_some());
        let sandbox_delivery_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid AND delivery_type = 'sandbox_workspace'
                 ORDER BY updated_at DESC, delivery_id DESC LIMIT 1",
                &[&seed.sandbox_order_id],
            )
            .await
            .expect("sandbox delivery row");
        assert_eq!(sandbox_delivery_row.get::<_, String>(0), "expired");

        assert_audit_count(
            &client,
            &file_req_id,
            "delivery.file.auto_cutoff.revoked",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &share_expire_req_id,
            "delivery.share.auto_cutoff.expired",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &share_dispute_req_id,
            "delivery.share.auto_cutoff.suspended",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &api_req_id,
            "delivery.api.auto_cutoff.suspended",
            1,
        )
        .await;
        assert_audit_count(
            &client,
            &sandbox_req_id,
            "delivery.sandbox.auto_cutoff.expired",
            1,
        )
        .await;

        cleanup_seed_graph(&client, &seed).await;
    }

    struct CallResponse {
        status: StatusCode,
        body: String,
    }

    async fn call(
        app: &axum::Router,
        method: &str,
        uri: String,
        buyer_org_id: &str,
        request_id: &str,
        payload: Option<&str>,
    ) -> CallResponse {
        let builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-role", "buyer_operator")
            .header("x-tenant-id", buyer_org_id)
            .header("x-request-id", request_id);
        let req = if let Some(payload) = payload {
            builder
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("request")
        } else {
            builder.body(Body::empty()).expect("request")
        };
        let resp = app.clone().oneshot(req).await.expect("response");
        let status = resp.status();
        let body = to_bytes(resp.into_body(), usize::MAX).await.expect("body");
        CallResponse {
            status,
            body: String::from_utf8_lossy(&body).to_string(),
        }
    }

    async fn assert_share_status(
        client: &Client,
        order_id: &str,
        expected_grant_status: &str,
        expected_delivery_status: &str,
    ) {
        let share_row = client
            .query_one(
                "SELECT grant_status,
                        to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM delivery.data_share_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, data_share_grant_id DESC
                 LIMIT 1",
                &[&order_id],
            )
            .await
            .expect("share row");
        assert_eq!(share_row.get::<_, String>(0), expected_grant_status);
        assert!(share_row.get::<_, Option<String>>(1).is_some());

        let delivery_row = client
            .query_one(
                "SELECT status FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid AND delivery_type = 'share_grant'
                 ORDER BY updated_at DESC, delivery_id DESC LIMIT 1",
                &[&order_id],
            )
            .await
            .expect("share delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), expected_delivery_status);
    }

    async fn assert_audit_count(client: &Client, request_id: &str, action_name: &str, min: i64) {
        let count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM audit.audit_event
                 WHERE request_id = $1 AND action_name = $2",
                &[&request_id, &action_name],
            )
            .await
            .expect("audit count")
            .get(0);
        assert!(count >= min, "missing audit {action_name}");
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                   VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                   RETURNING org_id::text"#,
                &[&format!("dlv021-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                   VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                   RETURNING org_id::text"#,
                &[&format!("dlv021-seller-{suffix}")],
            )
            .await?
            .get(0);

        let buyer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status)
                 VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'disabled')
                 RETURNING user_id::text",
                &[
                    &buyer_org_id,
                    &format!("dlv021-user-{suffix}"),
                    &format!("DLV021 Buyer {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                r#"INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'active', $3
                 ) RETURNING asset_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv021-asset-{suffix}"),
                    &format!("dlv021 asset {suffix}"),
                ],
            )
            .await?
            .get(0);

        let asset_version_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   4096, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 ) RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);

        let asset_object_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator, share_protocol,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'share_object', $2, $3, 'share_grant',
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb
                 ) RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv021-share-object-{suffix}"),
                    &format!("share://seller/{suffix}/dataset"),
                ],
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
                   $5, 'listed', 'subscription', 188.00, 'CNY', 'composite_delivery',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","sellable_status":"active"}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv021-product-{suffix}"),
                    &format!("dlv021 product {suffix}"),
                    &format!("dlv021 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let file_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("DLV021-FILE-{suffix}")],
            )
            .await?
            .get(0);

        let share_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '次', 'subscription', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("DLV021-SHARE-{suffix}")],
            )
            .await?
            .get(0);

        let api_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'API_PPU', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("DLV021-API-{suffix}")],
            )
            .await?
            .get(0);

        let sbx_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SBX_STD', '席位月', 'subscription', 'query_service',
                   'sandbox_workspace', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("DLV021-SBX-{suffix}")],
            )
            .await?
            .get(0);

        let storage_namespace_id: String = client
            .query_one(
                "INSERT INTO catalog.storage_namespace (
                   owner_org_id, namespace_name, provider_type, namespace_kind, bucket_name, prefix_rule, status
                 ) VALUES (
                   $1::text::uuid, $2, 's3_compatible', 'product', 'delivery-objects', 'orders/{order_id}', 'active'
                 ) RETURNING storage_namespace_id::text",
                &[&seller_org_id, &format!("dlv021-ns-{suffix}")],
            )
            .await?
            .get(0);

        let object_id: String = client
            .query_one(
                "INSERT INTO delivery.storage_object (
                   org_id, object_type, object_uri, location_type, managed_by_org_id,
                   content_type, size_bytes, content_hash, encryption_algo, plaintext_visible_to_platform,
                   storage_namespace_id, storage_zone, storage_class
                 ) VALUES (
                   $1::text::uuid, 'delivery_object', $2, 'platform_object_storage', $1::text::uuid,
                   'application/octet-stream', 1024, $3, 'AES-GCM', false,
                   $4::text::uuid, 'delivery', 'standard'
                 ) RETURNING object_id::text",
                &[
                    &seller_org_id,
                    &format!("s3://delivery-objects/orders/{suffix}/payload.enc"),
                    &format!("sha256:content:{suffix}"),
                    &storage_namespace_id,
                ],
            )
            .await?
            .get(0);

        let app_id: String = client
            .query_one(
                "INSERT INTO core.application (
                   org_id, app_name, app_type, status, client_id, metadata
                 ) VALUES (
                   $1::text::uuid, $2, 'api_client', 'active', $3, '{}'::jsonb
                 ) RETURNING app_id::text",
                &[
                    &buyer_org_id,
                    &format!("dlv021-app-{suffix}"),
                    &format!("dlv021-client-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let connector_id: String = client
            .query_one(
                r#"INSERT INTO core.connector (
                   org_id, connector_name, connector_type, status, version,
                   network_zone, health_status, endpoint_ref, metadata
                 ) VALUES (
                   $1::text::uuid, $2, 'sandbox_bridge', 'active', 'v1',
                   'seller_vpc', 'healthy', $3, '{"driver":"container"}'::jsonb
                 ) RETURNING connector_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv021-connector-{suffix}"),
                    &format!("sandbox://{suffix}"),
                ],
            )
            .await?
            .get(0);

        let environment_id: String = client
            .query_one(
                r#"INSERT INTO core.execution_environment (
                   org_id, connector_id, environment_name, environment_type, status, network_zone, region_code, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'sandbox', 'active', 'seller_vpc', 'CN',
                   '{"egress":"deny","clipboard":"masked_only"}'::jsonb
                 ) RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv021-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let file_order_id = insert_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &file_sku_id,
            "delivered",
            "paid",
            "delivered",
            "pending_acceptance",
            "pending_settlement",
            "none",
            "signed_url",
            "delivery_file_committed",
        )
        .await?;
        let envelope_id: String = client
            .query_one(
                "INSERT INTO delivery.key_envelope (
                   order_id, recipient_type, recipient_id, key_cipher, key_control_mode, unwrap_policy_json, key_version
                 ) VALUES (
                   $1::text::uuid, 'organization', $2::text::uuid, $3, 'seller_managed',
                   jsonb_build_object('kms', 'local-mock', 'buyer_org_id', $2::text), 'v1'
                 ) RETURNING envelope_id::text",
                &[&file_order_id, &buyer_org_id, &format!("cipher-{suffix}")],
            )
            .await?
            .get(0);
        let file_delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, object_id, delivery_type, delivery_route, status, delivery_commit_hash, envelope_id,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, 'file_download', 'signed_url', 'committed', $3, $4::text::uuid,
                   jsonb_build_object('watermark_rule', 'buyer_bound'),
                   $5, NOW() - INTERVAL '1 hour', NOW() + INTERVAL '7 days', 'standard', 'not_required'
                 ) RETURNING delivery_id::text",
                &[
                    &file_order_id,
                    &object_id,
                    &format!("commit-{suffix}"),
                    &envelope_id,
                    &format!("receipt-{suffix}"),
                ],
            )
            .await?
            .get(0);
        let file_ticket_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_ticket (
                   order_id, buyer_org_id, token_hash, expire_at, download_limit, download_count, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, NOW() + INTERVAL '6 days', 5, 1, 'active'
                 ) RETURNING ticket_id::text",
                &[&file_order_id, &buyer_org_id, &format!("ticket-{suffix}")],
            )
            .await?
            .get(0);

        let share_expire_order_id = insert_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &share_sku_id,
            "shared_active",
            "paid",
            "in_progress",
            "not_started",
            "pending_settlement",
            "none",
            "share_link",
            "share_ro_first_query_confirmed",
        )
        .await?;
        insert_share_delivery_bundle(
            client,
            &share_expire_order_id,
            &asset_object_id,
            &format!("expire-{suffix}"),
        )
        .await?;

        let share_dispute_order_id = insert_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &share_sku_id,
            "shared_active",
            "paid",
            "in_progress",
            "not_started",
            "pending_settlement",
            "none",
            "share_link",
            "share_ro_first_query_confirmed",
        )
        .await?;
        insert_share_delivery_bundle(
            client,
            &share_dispute_order_id,
            &asset_object_id,
            &format!("dispute-{suffix}"),
        )
        .await?;

        let api_order_id = insert_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &api_sku_id,
            "usage_active",
            "paid",
            "in_progress",
            "not_started",
            "pending_settlement",
            "none",
            "seller_gateway",
            "api_ppu_success_call_billed",
        )
        .await?;
        insert_api_delivery_bundle(client, &api_order_id, &app_id, &format!("api-{suffix}"))
            .await?;

        let sandbox_order_id = insert_order(
            client,
            &product_id,
            &asset_version_id,
            &buyer_org_id,
            &seller_org_id,
            &sbx_sku_id,
            "result_limited_exported",
            "paid",
            "in_progress",
            "not_started",
            "pending_settlement",
            "none",
            "sandbox_query",
            "sbx_std_limited_result_exported",
        )
        .await?;
        insert_sandbox_delivery_bundle(
            client,
            &sandbox_order_id,
            &environment_id,
            &buyer_user_id,
            &format!("sandbox-{suffix}"),
        )
        .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            asset_id,
            asset_version_id,
            product_id,
            file_sku_id,
            share_sku_id,
            api_sku_id,
            sbx_sku_id,
            asset_object_id,
            storage_namespace_id,
            object_id,
            envelope_id,
            app_id,
            connector_id,
            environment_id,
            file_order_id,
            file_delivery_id,
            file_ticket_id,
            share_expire_order_id,
            share_dispute_order_id,
            api_order_id,
            sandbox_order_id,
        })
    }

    async fn insert_order(
        client: &Client,
        product_id: &str,
        asset_version_id: &str,
        buyer_org_id: &str,
        seller_org_id: &str,
        sku_id: &str,
        status: &str,
        payment_status: &str,
        delivery_status: &str,
        acceptance_status: &str,
        settlement_status: &str,
        dispute_status: &str,
        delivery_route_snapshot: &str,
        last_reason_code: &str,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, $7, 'online', 188.00, 'CNY',
                   $8, $9, $10, $11,
                   jsonb_build_object('product_id', $1::text, 'sku_id', $5::text),
                   '{}'::jsonb,
                   $12,
                   $13
                 ) RETURNING order_id::text",
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
                    &delivery_route_snapshot,
                    &last_reason_code,
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn insert_share_delivery_bundle(
        client: &Client,
        order_id: &str,
        asset_object_id: &str,
        tag: &str,
    ) -> Result<(), db::Error> {
        client
            .execute(
                "INSERT INTO delivery.data_share_grant (
                   order_id, asset_object_id, recipient_ref, share_protocol, access_locator,
                   grant_status, read_only, receipt_hash, granted_at, expires_at, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'share_grant', $4,
                   'active', true, $5, now() - INTERVAL '1 day', NOW() + INTERVAL '7 days', '{}'::jsonb
                 )",
                &[
                    &order_id,
                    &asset_object_id,
                    &format!("warehouse://buyer/{tag}"),
                    &format!("share://seller/{tag}/dataset"),
                    &format!("share-receipt-{tag}"),
                ],
            )
            .await?;
        client
            .execute(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at,
                   sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'share_grant', 'share_grant', 'committed', $2,
                   '{}'::jsonb, $3, now() - INTERVAL '1 day', NOW() + INTERVAL '7 days',
                   'standard', 'not_required'
                 )",
                &[
                    &order_id,
                    &format!("share-commit-{tag}"),
                    &format!("share-receipt-{tag}"),
                ],
            )
            .await?;
        Ok(())
    }

    async fn insert_api_delivery_bundle(
        client: &Client,
        order_id: &str,
        app_id: &str,
        tag: &str,
    ) -> Result<(), db::Error> {
        client
            .execute(
                r#"INSERT INTO delivery.api_credential (
                   order_id, app_id, source_binding_id, api_key_hash, upstream_mode,
                   quota_json, status, valid_from, valid_to
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, NULL, $3, 'seller_gateway',
                   '{"requests_per_minute":30}'::jsonb, 'active', now() - INTERVAL '1 day', NOW() + INTERVAL '7 days'
                 )"#,
                &[&order_id, &app_id, &format!("api-key-hash-{tag}")],
            )
            .await?;
        client
            .execute(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at,
                   sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'api_access', 'seller_gateway', 'committed', $2,
                   '{}'::jsonb, $3, now() - INTERVAL '1 day', NOW() + INTERVAL '7 days',
                   'standard', 'not_required'
                 )",
                &[
                    &order_id,
                    &format!("api-commit-{tag}"),
                    &format!("api-receipt-{tag}"),
                ],
            )
            .await?;
        Ok(())
    }

    async fn insert_sandbox_delivery_bundle(
        client: &Client,
        order_id: &str,
        environment_id: &str,
        buyer_user_id: &str,
        tag: &str,
    ) -> Result<(), db::Error> {
        let sandbox_workspace_id: String = client
            .query_one(
                r#"INSERT INTO delivery.sandbox_workspace (
                   order_id, environment_id, query_surface_id, workspace_name, status,
                   clean_room_mode, data_residency_mode, export_policy, output_boundary_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, NULL, $3, 'active',
                   'lite', 'seller_self_hosted', '{"allow_export":false}'::jsonb, '{"allow_export":false}'::jsonb
                 ) RETURNING sandbox_workspace_id::text"#,
                &[&order_id, &environment_id, &format!("workspace-{tag}")],
            )
            .await?
            .get(0);
        client
            .execute(
                "INSERT INTO delivery.sandbox_session (
                   sandbox_workspace_id, user_id, started_at, ended_at, session_status, query_count, export_attempt_count
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, now() - INTERVAL '1 day', NOW() + INTERVAL '7 days', 'active', 3, 0
                 )",
                &[&sandbox_workspace_id, &buyer_user_id],
            )
            .await?;
        client
            .execute(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash,
                   trust_boundary_snapshot, receipt_hash, committed_at, expires_at,
                   sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'sandbox_workspace', 'sandbox_query', 'committed', $2,
                   jsonb_build_object('sandbox_workspace', jsonb_build_object('tag', $3)),
                   $4, now() - INTERVAL '1 day', NOW() + INTERVAL '7 days',
                   'standard', 'not_required'
                 )",
                &[
                    &order_id,
                    &format!("sandbox-commit-{tag}"),
                    &tag,
                    &format!("sandbox-receipt-{tag}"),
                ],
            )
            .await?;
        Ok(())
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        for order_id in [
            &seed.file_order_id,
            &seed.share_expire_order_id,
            &seed.share_dispute_order_id,
            &seed.api_order_id,
            &seed.sandbox_order_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_receipt WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_ticket WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.api_usage_log WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.api_credential WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.data_share_grant WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.sandbox_session WHERE sandbox_workspace_id IN (
                       SELECT sandbox_workspace_id FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid
                     )",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM delivery.key_envelope WHERE order_id = $1::text::uuid",
                    &[order_id],
                )
                .await;
            let _ = client
                .execute(
                    "DELETE FROM trade.authorization_grant WHERE order_id = $1::text::uuid",
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

        let _ = client
            .execute(
                "DELETE FROM core.application WHERE app_id = $1::text::uuid",
                &[&seed.app_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.execution_environment WHERE environment_id = $1::text::uuid",
                &[&seed.environment_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.connector WHERE connector_id = $1::text::uuid",
                &[&seed.connector_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.storage_object WHERE object_id = $1::text::uuid",
                &[&seed.object_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
            )
            .await;
        for sku_id in [
            &seed.file_sku_id,
            &seed.share_sku_id,
            &seed.api_sku_id,
            &seed.sbx_sku_id,
        ] {
            let _ = client
                .execute(
                    "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                    &[sku_id],
                )
                .await;
        }
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
                &[&seed.buyer_user_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
