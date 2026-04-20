#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        buyer_user_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        asset_object_id: String,
        connector_id: String,
        environment_id: String,
        query_surface_id: String,
    }

    #[tokio::test]
    async fn dlv014_sandbox_workspace_db_smoke() {
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
        let app = crate::with_live_test_state(delivery_router()).await;

        let create_request_id = format!("req-dlv014-create-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/sandbox-workspaces",
                        seed.order_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_surface_id": seed.query_surface_id,
                            "workspace_name": format!("dlv014-workspace-{suffix}"),
                            "seat_user_id": seed.buyer_user_id,
                            "expire_at": "2027-01-01T00:00:00Z",
                            "export_policy_json": {
                                "allow_export": false,
                                "allowed_formats": ["json"],
                                "max_exports": 0,
                                "network_access": "deny"
                            },
                            "clean_room_mode": "lite",
                            "data_residency_mode": "seller_self_hosted"
                        })
                        .to_string(),
                    ))
                    .expect("create request"),
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
        let data = &create_json["data"]["data"];
        let sandbox_workspace_id = data["sandbox_workspace_id"]
            .as_str()
            .expect("sandbox_workspace_id")
            .to_string();
        let sandbox_session_id = data["sandbox_session_id"]
            .as_str()
            .expect("sandbox_session_id")
            .to_string();
        assert_eq!(data["operation"].as_str(), Some("created"));
        assert_eq!(data["workspace_status"].as_str(), Some("active"));
        assert_eq!(data["session_status"].as_str(), Some("active"));
        assert_eq!(data["current_state"].as_str(), Some("seat_issued"));
        assert_eq!(data["environment_type"].as_str(), Some("sandbox"));
        assert_eq!(
            data["export_policy_json"]["allow_export"].as_bool(),
            Some(false)
        );

        let invalid_request_id = format!("req-dlv014-invalid-{suffix}");
        let invalid_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/sandbox-workspaces",
                        seed.order_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &invalid_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_surface_id": seed.query_surface_id,
                            "seat_user_id": seed.buyer_user_id,
                            "expire_at": "2027-01-01T00:00:00Z",
                            "export_policy_json": {
                                "allow_export": true,
                                "allowed_formats": ["csv"],
                                "max_exports": 5
                            }
                        })
                        .to_string(),
                    ))
                    .expect("invalid request"),
            )
            .await
            .expect("invalid response");
        let invalid_status = invalid_response.status();
        let invalid_body = to_bytes(invalid_response.into_body(), usize::MAX)
            .await
            .expect("invalid body");
        assert_eq!(
            invalid_status,
            StatusCode::CONFLICT,
            "{}",
            String::from_utf8_lossy(&invalid_body)
        );
        let invalid_json: Value = serde_json::from_slice(&invalid_body).expect("invalid json");
        let invalid_msg = invalid_json["message"]
            .as_str()
            .or_else(|| invalid_json["error"]["message"].as_str())
            .unwrap_or_default()
            .to_string();
        assert!(invalid_msg.contains("SANDBOX_WORKSPACE_FORBIDDEN"));

        let workspace_row = client
            .query_one(
                "SELECT query_surface_id::text,
                        environment_id::text,
                        workspace_name,
                        status,
                        clean_room_mode,
                        data_residency_mode,
                        export_policy ->> 'allow_export',
                        output_boundary_json ->> 'allow_export'
                 FROM delivery.sandbox_workspace
                 WHERE sandbox_workspace_id = $1::text::uuid",
                &[&sandbox_workspace_id],
            )
            .await
            .expect("workspace row");
        assert_eq!(workspace_row.get::<_, String>(0), seed.query_surface_id);
        assert_eq!(workspace_row.get::<_, String>(1), seed.environment_id);
        assert_eq!(workspace_row.get::<_, String>(3), "active");
        assert_eq!(workspace_row.get::<_, String>(4), "lite");
        assert_eq!(workspace_row.get::<_, String>(5), "seller_self_hosted");
        assert_eq!(
            workspace_row.get::<_, Option<String>>(6).as_deref(),
            Some("false")
        );
        assert_eq!(
            workspace_row.get::<_, Option<String>>(7).as_deref(),
            Some("false")
        );

        let session_row = client
            .query_one(
                "SELECT sandbox_workspace_id::text,
                        user_id::text,
                        session_status,
                        query_count,
                        export_attempt_count,
                        to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM delivery.sandbox_session
                 WHERE sandbox_session_id = $1::text::uuid",
                &[&sandbox_session_id],
            )
            .await
            .expect("session row");
        assert_eq!(session_row.get::<_, String>(0), sandbox_workspace_id);
        assert_eq!(session_row.get::<_, String>(1), seed.buyer_user_id);
        assert_eq!(session_row.get::<_, String>(2), "active");
        assert_eq!(session_row.get::<_, i32>(3), 0);
        assert_eq!(session_row.get::<_, i32>(4), 0);
        assert_eq!(
            session_row.get::<_, Option<String>>(5).as_deref(),
            Some("2027-01-01T00:00:00.000Z")
        );

        let delivery_row = client
            .query_one(
                "SELECT delivery_type,
                        delivery_route,
                        status,
                        receipt_hash,
                        delivery_commit_hash,
                        expires_at IS NOT NULL,
                        trust_boundary_snapshot -> 'sandbox_workspace' ->> 'sandbox_workspace_id'
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "sandbox_workspace");
        assert_eq!(
            delivery_row.get::<_, Option<String>>(1).as_deref(),
            Some("sandbox_query")
        );
        assert_eq!(delivery_row.get::<_, String>(2), "committed");
        assert!(delivery_row.get::<_, String>(3).contains(&seed.order_id));
        assert!(
            delivery_row
                .get::<_, String>(4)
                .contains(&sandbox_workspace_id)
        );
        assert!(delivery_row.get::<_, bool>(5));
        assert_eq!(
            delivery_row.get::<_, Option<String>>(6).as_deref(),
            Some(sandbox_workspace_id.as_str())
        );

        let order_row = client
            .query_one(
                "SELECT status,
                        payment_status,
                        delivery_status,
                        acceptance_status,
                        settlement_status,
                        dispute_status,
                        last_reason_code
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("order row");
        assert_eq!(order_row.get::<_, String>(0), "seat_issued");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "in_progress");
        assert_eq!(order_row.get::<_, String>(3), "not_started");
        assert_eq!(order_row.get::<_, String>(4), "pending_settlement");
        assert_eq!(order_row.get::<_, String>(5), "none");
        assert_eq!(
            order_row.get::<_, String>(6),
            "delivery_sbx_std_seat_issued"
        );

        let delivery_audit_row = client
            .query_one(
                "SELECT action_name, ref_type, ref_id::text
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'delivery.sandbox.enable'
                 ORDER BY event_time DESC, audit_id DESC
                 LIMIT 1",
                &[&create_request_id],
            )
            .await
            .expect("delivery audit row");
        assert_eq!(
            delivery_audit_row.get::<_, String>(0),
            "delivery.sandbox.enable"
        );
        assert_eq!(delivery_audit_row.get::<_, String>(1), "sandbox_workspace");
        assert_eq!(delivery_audit_row.get::<_, String>(2), sandbox_workspace_id);

        let outbox_row = client
            .query_one(
                "SELECT target_topic,
                        payload ->> 'delivery_branch',
                        payload ->> 'order_id'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'delivery.committed'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&create_request_id],
            )
            .await
            .expect("sandbox outbox row");
        assert_eq!(
            outbox_row.get::<_, Option<String>>(0).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(1).as_deref(),
            Some("sandbox")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(2).as_deref(),
            Some(seed.order_id.as_str())
        );
        let billing_bridge_row = client
            .query_one(
                "SELECT target_topic,
                        payload ->> 'delivery_branch',
                        payload ->> 'trigger_stage',
                        payload -> 'billing_trigger_matrix' ->> 'billing_trigger'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'billing.trigger.bridge'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&create_request_id],
            )
            .await
            .expect("sandbox billing bridge row");
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(0).as_deref(),
            Some("billing.events")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(1).as_deref(),
            Some("sandbox")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(2).as_deref(),
            Some("delivery_committed")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(3).as_deref(),
            Some("bill_after_workspace_activation_acceptance")
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                 RETURNING org_id::text"#,
                &[&format!("dlv014-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                r#"INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{"risk_status":"normal"}'::jsonb)
                 RETURNING org_id::text"#,
                &[&format!("dlv014-seller-{suffix}")],
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
                    &format!("dlv014-buyer-user-{suffix}"),
                    &format!("DLV014 Buyer {suffix}"),
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
                 )
                 RETURNING asset_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv014-asset-{suffix}"),
                    &format!("dlv014 asset {suffix}"),
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
                   4096, 'CN', ARRAY['CN']::text[], true,
                   '{"query_mode":"sandbox"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);

        let asset_object_id: String = client
            .query_one(
                r#"INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'structured_dataset', $2, $3,
                   '{"fields":[{"name":"region"},{"name":"amount"}]}'::jsonb,
                   '{"fields":[{"name":"region"},{"name":"amount"}]}'::jsonb,
                   '{}'::jsonb,
                   '{"preview":false}'::jsonb,
                   '{"zone":"curated"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv014-object-{suffix}"),
                    &format!("warehouse://sandbox/{suffix}/orders"),
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
                   $5, 'listed', 'subscription', 88.00, 'CNY', 'sandbox_workspace',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv014-product-{suffix}"),
                    &format!("dlv014 product {suffix}"),
                    &format!("dlv014 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SBX_STD', '席位月', 'subscription', 'query_service',
                   'sandbox_workspace', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV014-SKU-{suffix}")],
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
                 )
                 RETURNING connector_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv014-connector-{suffix}"),
                    &format!("sandbox://{}", suffix),
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
                 )
                 RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv014-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let query_surface_id: String = client
            .query_one(
                r#"INSERT INTO catalog.query_surface_definition (
                   asset_version_id, asset_object_id, environment_id, surface_type, binding_mode,
                   execution_scope, input_contract_json, output_boundary_json,
                   query_policy_json, quota_policy_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, 'sandbox_query', 'managed_surface',
                   'curated_zone',
                   '{"source_zones":["curated"],"seat_required":true}'::jsonb,
                   '{"allow_export":false,"allowed_formats":["json"],"max_exports":0}'::jsonb,
                   '{"network_boundary":"deny","clipboard":"masked_only"}'::jsonb,
                   '{"seat_limit":1}'::jsonb,
                   'active',
                   '{"owner":"seller"}'::jsonb
                 )
                 RETURNING query_surface_id::text"#,
                &[&asset_version_id, &asset_object_id, &environment_id],
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
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 88.00, 'CNY', '{}'::jsonb
                 )
                 RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        client
            .execute(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&order_id, &format!("sha256:dlv014:{suffix}")],
            )
            .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
            connector_id,
            environment_id,
            query_surface_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM delivery.sandbox_session WHERE sandbox_workspace_id IN (
                   SELECT sandbox_workspace_id FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid
                 )",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.sandbox_workspace WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
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
                "DELETE FROM catalog.query_surface_definition WHERE query_surface_id = $1::text::uuid",
                &[&seed.query_surface_id],
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
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
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
