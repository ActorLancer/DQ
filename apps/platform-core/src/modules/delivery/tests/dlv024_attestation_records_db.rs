#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::storage::application::{delete_object, put_object_bytes};
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
        buyer_login_id: String,
        buyer_display_name: String,
        approval_ticket_id: String,
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
    async fn dlv024_attestation_records_db_smoke() {
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

        let create_request_id = format!("req-dlv024-sbx-{suffix}");
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
                            "workspace_name": format!("dlv024-workspace-{suffix}"),
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
        let create_data = &create_json["data"]["data"];
        let attestation_record_id = create_data["attestation"]["attestation_record_id"]
            .as_str()
            .expect("attestation_record_id")
            .to_string();
        assert_eq!(
            create_data["attestation"]["status"].as_str(),
            Some("pending")
        );

        let read_request_id = format!("req-dlv024-read-{suffix}");
        let read_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/attestations", seed.order_id))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &read_request_id)
                    .body(Body::empty())
                    .expect("read request"),
            )
            .await
            .expect("read response");
        let read_status = read_response.status();
        let read_body = to_bytes(read_response.into_body(), usize::MAX)
            .await
            .expect("read body");
        assert_eq!(
            read_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&read_body)
        );
        let read_json: Value = serde_json::from_slice(&read_body).expect("read json");
        let attestations = read_json["data"]["data"]["attestations"]
            .as_array()
            .expect("attestations array");
        assert_eq!(attestations.len(), 1);
        assert_eq!(
            attestations[0]["attestation_record_id"].as_str(),
            Some(attestation_record_id.as_str())
        );
        assert_eq!(
            attestations[0]["source_type"].as_str(),
            Some("sandbox_session")
        );

        let bucket_name = std::env::var("BUCKET_DELIVERY_OBJECTS")
            .unwrap_or_else(|_| "delivery-objects".to_string());
        let object_key = format!("dlv024/{suffix}/sandbox-result.json");
        let proof_key = format!("dlv024/{suffix}/destruction-proof.json");
        put_object_bytes(
            &bucket_name,
            &object_key,
            br#"{"rows":2,"status":"expired"}"#.to_vec(),
            Some("application/json"),
        )
        .await
        .expect("put result object");
        put_object_bytes(
            &bucket_name,
            &proof_key,
            br#"{"proof":"retained","version":"v1"}"#.to_vec(),
            Some("application/json"),
        )
        .await
        .expect("put proof object");

        let object_uri = format!("s3://{bucket_name}/{object_key}");
        let object_id: String = client
            .query_one(
                "INSERT INTO delivery.storage_object (
                   org_id,
                   object_type,
                   object_uri,
                   location_type,
                   managed_by_org_id,
                   connector_id,
                   environment_id,
                   content_type,
                   size_bytes,
                   content_hash
                 ) VALUES (
                   $1::text::uuid,
                   'sandbox_artifact',
                   $2,
                   'platform_object_storage',
                   $1::text::uuid,
                   $3::text::uuid,
                   $4::text::uuid,
                   'application/json',
                   31,
                   $5
                 )
                 RETURNING object_id::text",
                &[
                    &seed.seller_org_id,
                    &object_uri,
                    &seed.connector_id,
                    &seed.environment_id,
                    &format!("sha256:dlv024-result-{suffix}"),
                ],
            )
            .await
            .expect("insert storage object")
            .get(0);

        client
            .execute(
                r#"INSERT INTO delivery.delivery_record (
                   order_id,
                   object_id,
                   delivery_type,
                   delivery_route,
                   executor_type,
                   status,
                   trust_boundary_snapshot,
                   committed_at,
                   expires_at
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   'sandbox_artifact',
                   'sandbox_query',
                   'platform',
                   'expired',
                   '{"sandbox":"expired"}'::jsonb,
                   now() - interval '2 days',
                   now() - interval '1 day'
                 )"#,
                &[&seed.order_id, &object_id],
            )
            .await
            .expect("insert expired delivery record");
        client
            .execute(
                "UPDATE trade.order_main
                 SET status = 'expired',
                     delivery_status = 'expired',
                     updated_at = now()
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("expire order");

        let proof_uri = format!("s3://{bucket_name}/{proof_key}");
        let create_proof_request_id = format!("req-dlv024-proof-create-{suffix}");
        let create_proof_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/destruction-attestations",
                        seed.order_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_proof_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "object_id": object_id,
                            "retention_action": "retain",
                            "attestation_uri": proof_uri,
                            "attestation_hash": format!("sha256:dlv024-proof-{suffix}"),
                            "approval_ticket_id": seed.approval_ticket_id,
                            "metadata": {
                                "reason_code": "dispute_hold",
                                "legal_hold_status": "active",
                                "retention_until": "2027-06-30T00:00:00Z"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("create proof request"),
            )
            .await
            .expect("create proof response");
        let create_proof_status = create_proof_response.status();
        let create_proof_body = to_bytes(create_proof_response.into_body(), usize::MAX)
            .await
            .expect("create proof body");
        assert_eq!(
            create_proof_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&create_proof_body)
        );
        let create_proof_json: Value =
            serde_json::from_slice(&create_proof_body).expect("create proof json");
        let create_proof_data = &create_proof_json["data"]["data"];
        let destruction_attestation_id = create_proof_data["destruction_attestation_id"]
            .as_str()
            .expect("destruction_attestation_id")
            .to_string();
        assert_eq!(create_proof_data["operation"].as_str(), Some("created"));
        assert_eq!(
            create_proof_data["retention_action"].as_str(),
            Some("retain")
        );
        assert_eq!(create_proof_data["status"].as_str(), Some("retained"));
        assert_eq!(
            create_proof_data["object_bucket_name"].as_str(),
            Some(bucket_name.as_str())
        );
        assert_eq!(
            create_proof_data["object_key"].as_str(),
            Some(object_key.as_str())
        );

        let update_proof_request_id = format!("req-dlv024-proof-update-{suffix}");
        let update_proof_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/destruction-attestations",
                        seed.order_id
                    ))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &update_proof_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "destruction_attestation_id": destruction_attestation_id,
                            "object_id": object_id,
                            "retention_action": "destroy",
                            "attestation_uri": proof_uri,
                            "attestation_hash": format!("sha256:dlv024-proof-updated-{suffix}"),
                            "status": "completed",
                            "metadata": {
                                "reason_code": "retention_window_elapsed"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("update proof request"),
            )
            .await
            .expect("update proof response");
        let update_proof_status = update_proof_response.status();
        let update_proof_body = to_bytes(update_proof_response.into_body(), usize::MAX)
            .await
            .expect("update proof body");
        assert_eq!(
            update_proof_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&update_proof_body)
        );
        let update_proof_json: Value =
            serde_json::from_slice(&update_proof_body).expect("update proof json");
        let update_proof_data = &update_proof_json["data"]["data"];
        assert_eq!(update_proof_data["operation"].as_str(), Some("updated"));
        assert_eq!(
            update_proof_data["retention_action"].as_str(),
            Some("destroy")
        );
        assert_eq!(update_proof_data["status"].as_str(), Some("completed"));

        let attestation_row = client
            .query_one(
                "SELECT attestation_type,
                        status,
                        metadata ->> 'sandbox_workspace_id'
                 FROM delivery.attestation_record
                 WHERE attestation_record_id = $1::text::uuid",
                &[&attestation_record_id],
            )
            .await
            .expect("attestation row");
        assert_eq!(attestation_row.get::<_, String>(0), "execution_receipt");
        assert_eq!(attestation_row.get::<_, String>(1), "pending");

        let destruction_row = client
            .query_one(
                "SELECT retention_action,
                        status,
                        attestation_uri,
                        attestation_hash,
                        approval_ticket_id::text,
                        metadata -> 'order_snapshot' ->> 'delivery_status',
                        metadata -> 'proof_snapshot' ->> 'retention_action'
                 FROM delivery.destruction_attestation
                 WHERE destruction_attestation_id = $1::text::uuid",
                &[&destruction_attestation_id],
            )
            .await
            .expect("destruction row");
        assert_eq!(destruction_row.get::<_, String>(0), "destroy");
        assert_eq!(destruction_row.get::<_, String>(1), "completed");
        assert_eq!(
            destruction_row.get::<_, Option<String>>(2).as_deref(),
            Some(proof_uri.as_str())
        );
        assert_eq!(
            destruction_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("sha256:dlv024-proof-updated-{suffix}").as_str())
        );
        assert_eq!(
            destruction_row.get::<_, Option<String>>(4).as_deref(),
            Some(seed.approval_ticket_id.as_str())
        );
        assert_eq!(
            destruction_row.get::<_, Option<String>>(5).as_deref(),
            Some("expired")
        );
        assert_eq!(
            destruction_row.get::<_, Option<String>>(6).as_deref(),
            Some("destroy")
        );

        let audit_counts = client
            .query_one(
                "SELECT COUNT(*) FILTER (WHERE request_id = $1),
                        COUNT(*) FILTER (WHERE request_id = $2),
                        COUNT(*) FILTER (WHERE request_id = $3)
                 FROM audit.audit_event
                 WHERE action_name IN (
                   'delivery.attestation.read',
                   'delivery.destruction.attest'
                 )",
                &[
                    &read_request_id,
                    &create_proof_request_id,
                    &update_proof_request_id,
                ],
            )
            .await
            .expect("audit counts");
        assert_eq!(audit_counts.get::<_, i64>(0), 1);
        assert_eq!(audit_counts.get::<_, i64>(1), 1);
        assert_eq!(audit_counts.get::<_, i64>(2), 1);

        let _ = delete_object(&bucket_name, &object_key).await;
        let _ = delete_object(&bucket_name, &proof_key).await;
        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(
        client: &Client,
        suffix: &str,
    ) -> Result<SeedGraph, Box<dyn std::error::Error + Send + Sync>> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv024-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{\"risk_status\":\"normal\"}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv024-seller-{suffix}")],
            )
            .await?
            .get(0);

        let buyer_login_id = format!("dlv024-buyer-user-{suffix}");
        let buyer_display_name = format!("DLV024 Buyer {suffix}");
        let buyer_user_id: String = client
            .query_one(
                "INSERT INTO core.user_account (org_id, login_id, display_name, user_type, status, mfa_status)
                 VALUES ($1::text::uuid, $2, $3, 'person', 'active', 'disabled')
                 RETURNING user_id::text",
                &[&buyer_org_id, &buyer_login_id, &buyer_display_name],
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
                    &format!("dlv024-asset-{suffix}"),
                    &format!("dlv024 asset {suffix}"),
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
                    &format!("dlv024-object-{suffix}"),
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
                    &format!("dlv024-product-{suffix}"),
                    &format!("dlv024 product {suffix}"),
                    &format!("dlv024 search {suffix}"),
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
                &[&product_id, &format!("DLV024-SKU-{suffix}")],
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
                    &format!("dlv024-connector-{suffix}"),
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
                   '{"egress":"deny","clipboard":"masked_only","attestation_required":true,"verifier_ref":"sandbox-verifier"}'::jsonb
                 )
                 RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv024-env-{suffix}"),
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
                   '{"network_access":"seller_vpc_only","clipboard":"masked_only","step_up_required":true,"requires_attestation":true,"seat_limit":3}'::jsonb,
                   '{"seat_limit":3}'::jsonb,
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
                &[&order_id, &format!("sha256:dlv024:{suffix}")],
            )
            .await?;

        let approval_ticket_id: String = client
            .query_one(
                "INSERT INTO ops.approval_ticket (ticket_type, ref_type, ref_id, requested_by, status, requires_second_review)
                 VALUES ('destruction_attestation', 'order', $1::text::uuid, $2::text::uuid, 'approved', false)
                 RETURNING approval_ticket_id::text",
                &[&order_id, &buyer_user_id],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            buyer_user_id,
            buyer_login_id,
            buyer_display_name,
            approval_ticket_id,
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
                "DELETE FROM delivery.destruction_attestation WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.attestation_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.sensitive_execution_policy WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
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
                "DELETE FROM delivery.storage_object WHERE connector_id = $1::text::uuid",
                &[&seed.connector_id],
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
                "DELETE FROM ops.approval_ticket WHERE approval_ticket_id = $1::text::uuid",
                &[&seed.approval_ticket_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await;
    }
}
