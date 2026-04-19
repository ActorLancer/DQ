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
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        asset_object_id: String,
        connector_id: String,
        environment_id: String,
        query_surface_id: String,
        foreign_query_surface_id: String,
        template_v1_id: String,
        template_v2_id: String,
        foreign_template_id: String,
    }

    #[tokio::test]
    async fn dlv011_template_grant_db_smoke() {
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

        let create_request_id = format!("req-dlv011-create-{suffix}");
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-grants", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &create_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_surface_id": seed.query_surface_id,
                            "allowed_template_ids": [seed.template_v1_id, seed.template_v2_id],
                            "execution_rule_snapshot": {
                                "entrypoint": "template_query_lite",
                                "grant_source": "manual_open"
                            },
                            "output_boundary_json": {
                                "allowed_formats": ["json"],
                                "max_rows": 50,
                                "max_cells": 500
                            },
                            "run_quota_json": {
                                "max_runs": 20,
                                "daily_limit": 5,
                                "monthly_limit": 50
                            }
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
        let template_query_grant_id = create_json["data"]["data"]["template_query_grant_id"]
            .as_str()
            .expect("template_query_grant_id")
            .to_string();
        assert_eq!(
            create_json["data"]["data"]["operation"].as_str(),
            Some("granted")
        );
        assert_eq!(
            create_json["data"]["data"]["current_state"].as_str(),
            Some("template_authorized")
        );
        assert_eq!(
            create_json["data"]["data"]["grant_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            create_json["data"]["data"]["allowed_template_ids"],
            json!([seed.template_v1_id, seed.template_v2_id])
        );

        let update_request_id = format!("req-dlv011-update-{suffix}");
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-grants", seed.order_id))
                    .header("x-role", "tenant_developer")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &update_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "template_query_grant_id": template_query_grant_id,
                            "allowed_template_ids": [seed.template_v2_id],
                            "execution_rule_snapshot": {
                                "entrypoint": "template_query_lite",
                                "grant_source": "buyer_update",
                                "requires_audit": true
                            },
                            "output_boundary_json": {
                                "allowed_formats": ["json"],
                                "max_rows": 25,
                                "max_cells": 250
                            },
                            "run_quota_json": {
                                "max_runs": 10,
                                "daily_limit": 3,
                                "monthly_limit": 30
                            }
                        })
                        .to_string(),
                    ))
                    .expect("update request"),
            )
            .await
            .expect("update response");
        let update_status = update_response.status();
        let update_body = to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("update body");
        assert_eq!(
            update_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&update_body)
        );
        let update_json: Value = serde_json::from_slice(&update_body).expect("update json");
        assert_eq!(
            update_json["data"]["data"]["template_query_grant_id"].as_str(),
            Some(template_query_grant_id.as_str())
        );
        assert_eq!(
            update_json["data"]["data"]["operation"].as_str(),
            Some("updated")
        );
        assert_eq!(
            update_json["data"]["data"]["allowed_template_ids"],
            json!([seed.template_v2_id])
        );
        assert_eq!(
            update_json["data"]["data"]["run_quota_json"]["max_runs"].as_i64(),
            Some(10)
        );

        let invalid_request_id = format!("req-dlv011-invalid-{suffix}");
        let invalid_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/template-grants", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &invalid_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "query_surface_id": seed.query_surface_id,
                            "allowed_template_ids": [seed.foreign_template_id],
                            "run_quota_json": {
                                "max_runs": 1,
                                "daily_limit": 1,
                                "monthly_limit": 1
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

        let grant_row = client
            .query_one(
                "SELECT query_surface_id::text,
                        asset_object_id::text,
                        environment_id::text,
                        template_type,
                        template_digest,
                        allowed_template_ids,
                        execution_rule_snapshot,
                        output_boundary_json,
                        run_quota_json,
                        grant_status
                 FROM delivery.template_query_grant
                 WHERE template_query_grant_id = $1::text::uuid",
                &[&template_query_grant_id],
            )
            .await
            .expect("query template grant row");
        assert_eq!(
            grant_row.get::<_, String>(0),
            seed.query_surface_id,
            "query_surface_id should match seed surface"
        );
        assert_eq!(grant_row.get::<_, String>(1), seed.asset_object_id);
        assert_eq!(
            grant_row.get::<_, Option<String>>(2).as_deref(),
            Some(seed.environment_id.as_str())
        );
        assert_eq!(grant_row.get::<_, String>(3), "sql_template");
        assert_eq!(grant_row.get::<_, Value>(5), json!([seed.template_v2_id]));
        assert_eq!(
            grant_row.get::<_, Value>(6)["grant_source"].as_str(),
            Some("buyer_update")
        );
        assert_eq!(grant_row.get::<_, Value>(7)["max_rows"].as_i64(), Some(25));
        assert_eq!(
            grant_row.get::<_, Value>(8)["daily_limit"].as_i64(),
            Some(3)
        );
        assert_eq!(grant_row.get::<_, String>(9), "active");
        let template_digest = grant_row.get::<_, String>(4);

        let delivery_row = client
            .query_one(
                "SELECT status,
                        delivery_type,
                        delivery_route,
                        delivery_commit_hash,
                        receipt_hash
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "committed");
        assert_eq!(delivery_row.get::<_, String>(1), "template_grant");
        assert_eq!(delivery_row.get::<_, String>(2), "template_query");
        assert_eq!(
            delivery_row.get::<_, Option<String>>(3).as_deref(),
            Some(template_digest.as_str())
        );
        assert_eq!(
            delivery_row.get::<_, Option<String>>(4).as_deref(),
            Some(template_digest.as_str())
        );

        let order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order row");
        assert_eq!(order_row.get::<_, String>(0), "template_authorized");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "in_progress");
        assert_eq!(order_row.get::<_, String>(3), "not_started");
        assert_eq!(order_row.get::<_, String>(4), "pending_settlement");

        let delivery_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.template_query.enable'
                   AND request_id IN ($1, $2)",
                &[&create_request_id, &update_request_id],
            )
            .await
            .expect("delivery audit count")
            .get(0);
        assert_eq!(delivery_audit_count, 2);

        let trade_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'trade.order.qry_lite.transition'
                   AND request_id IN ($1, $2)",
                &[&create_request_id, &update_request_id],
            )
            .await
            .expect("trade audit count")
            .get(0);
        assert_eq!(trade_audit_count, 2);

        let outbox_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE event_type = 'delivery.committed'
                   AND request_id IN ($1, $2)
                   AND payload ->> 'delivery_branch' = 'template'
                   AND target_topic = 'dtp.outbox.domain-events'",
                &[&create_request_id, &update_request_id],
            )
            .await
            .expect("template outbox count")
            .get(0);
        assert_eq!(outbox_count, 2);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv011-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv011-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'low', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("dlv011-asset-{suffix}"),
                    &format!("dlv011 asset {suffix}"),
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
                   '{"query_mode":"controlled"}'::jsonb, 'active'
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
                   '{"fields":[{"name":"city"},{"name":"amount"},{"name":"confidence"}]}'::jsonb,
                   '{"fields":[{"name":"city"},{"name":"total_amount"},{"name":"confidence"}]}'::jsonb,
                   '{}'::jsonb,
                   '{"preview":false}'::jsonb,
                   '{"zone":"curated"}'::jsonb
                 )
                 RETURNING asset_object_id::text"#,
                &[
                    &asset_version_id,
                    &format!("dlv011-structured-{suffix}"),
                    &format!("warehouse://curated/{suffix}/sales"),
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
                   $5, 'listed', 'usage', 48.00, 'CNY', 'template_query',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv011-product-{suffix}"),
                    &format!("dlv011 product {suffix}"),
                    &format!("dlv011 search {suffix}"),
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
                   $1::text::uuid, $2, 'QRY_LITE', '次', 'usage', 'query_service',
                   'template_grant', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV011-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                r#"INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json, delivery_route_snapshot, trust_boundary_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 48.00, 'CNY',
                   '{"query_surface_type":"template_query_lite"}'::jsonb,
                   'template_query',
                   '{"query_delivery":"controlled"}'::jsonb
                 )
                 RETURNING order_id::text"#,
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
                &[&order_id, &format!("sha256:dlv011:{suffix}")],
            )
            .await?;

        let connector_id: String = client
            .query_one(
                r#"INSERT INTO core.connector (
                   org_id, connector_name, connector_type, status, version,
                   network_zone, health_status, endpoint_ref, metadata
                 ) VALUES (
                   $1::text::uuid, $2, 'query_runtime', 'active', 'v1',
                   'private', 'healthy', $3, '{"provider":"local"}'::jsonb
                 )
                 RETURNING connector_id::text"#,
                &[
                    &seller_org_id,
                    &format!("dlv011-connector-{suffix}"),
                    &format!("https://connector.example.com/{suffix}"),
                ],
            )
            .await?
            .get(0);

        let environment_id: String = client
            .query_one(
                r#"INSERT INTO core.execution_environment (
                   org_id, connector_id, environment_name, environment_type,
                   status, network_zone, region_code, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'query_runtime',
                   'active', 'private', 'cn-east-1', '{"mode":"template_query"}'::jsonb
                 )
                 RETURNING environment_id::text"#,
                &[
                    &seller_org_id,
                    &connector_id,
                    &format!("dlv011-env-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let query_surface_id: String = client
            .query_one(
                r#"INSERT INTO catalog.query_surface_definition (
                   asset_version_id, asset_object_id, environment_id, surface_type, binding_mode,
                   execution_scope, input_contract_json, output_boundary_json, query_policy_json,
                   quota_policy_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, 'template_query_lite', 'managed_surface',
                   'curated_zone',
                   '{"source_zones":["curated_zone"]}'::jsonb,
                   '{"max_rows":100,"max_cells":1000,"allow_raw_export":false,"allowed_formats":["json","csv"]}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb,
                   '{"daily_limit":60,"monthly_limit":600}'::jsonb,
                   'active',
                   '{"owner":"seller"}'::jsonb
                 )
                 RETURNING query_surface_id::text"#,
                &[&asset_version_id, &asset_object_id, &environment_id],
            )
            .await?
            .get(0);

        let foreign_query_surface_id: String = client
            .query_one(
                r#"INSERT INTO catalog.query_surface_definition (
                   asset_version_id, asset_object_id, environment_id, surface_type, binding_mode,
                   execution_scope, input_contract_json, output_boundary_json, query_policy_json,
                   quota_policy_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, 'template_query_lite', 'managed_surface',
                   'curated_zone',
                   '{"source_zones":["curated_zone"]}'::jsonb,
                   '{"max_rows":100,"max_cells":1000,"allow_raw_export":false,"allowed_formats":["json"]}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb,
                   '{"daily_limit":60,"monthly_limit":600}'::jsonb,
                   'active',
                   '{"owner":"seller","scope":"foreign"}'::jsonb
                 )
                 RETURNING query_surface_id::text"#,
                &[&asset_version_id, &asset_object_id, &environment_id],
            )
            .await?
            .get(0);

        let template_v1_id =
            insert_template(client, &query_surface_id, suffix, "sales_overview", 1).await?;
        let template_v2_id =
            insert_template(client, &query_surface_id, suffix, "sales_overview", 2).await?;
        let foreign_template_id = insert_template(
            client,
            &foreign_query_surface_id,
            suffix,
            "foreign_summary",
            1,
        )
        .await?;

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            asset_object_id,
            connector_id,
            environment_id,
            query_surface_id,
            foreign_query_surface_id,
            template_v1_id,
            template_v2_id,
            foreign_template_id,
        })
    }

    async fn insert_template(
        client: &Client,
        query_surface_id: &str,
        suffix: &str,
        template_name: &str,
        version_no: i32,
    ) -> Result<String, db::Error> {
        client
            .query_one(
                r#"INSERT INTO delivery.query_template_definition (
                   query_surface_id, template_name, template_type, template_body_ref,
                   parameter_schema_json, analysis_rule_json, result_schema_json,
                   export_policy_json, risk_guard_json, status, version_no
                 ) VALUES (
                   $1::text::uuid, $2, 'sql_template', $3,
                   '{"type":"object","properties":{"start_date":{"type":"string"},"limit":{"type":"integer"}}}'::jsonb,
                   '{"analysis_rule":"whitelist_only","template_review_status":"approved"}'::jsonb,
                   '{"fields":[{"name":"city","type":"string"},{"name":"total_amount","type":"number"},{"name":"confidence","type":"number"}]}'::jsonb,
                   '{"allow_raw_export":false,"allowed_formats":["json"],"max_export_rows":100,"max_export_cells":1000}'::jsonb,
                   '{"risk_mode":"strict"}'::jsonb,
                   'active',
                   $4
                 )
                 RETURNING query_template_id::text"#,
                &[
                    &query_surface_id,
                    &template_name,
                    &format!(
                        "minio://delivery-objects/templates/{suffix}/{template_name}_v{version_no}.sql"
                    ),
                    &version_no,
                ],
            )
            .await
            .map(|row| row.get(0))
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM delivery.query_template_definition
                 WHERE query_surface_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.query_surface_id, &seed.foreign_query_surface_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.query_surface_definition
                 WHERE query_surface_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.query_surface_id, &seed.foreign_query_surface_id],
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
                "DELETE FROM catalog.asset_object_binding WHERE asset_object_id = $1::text::uuid",
                &[&seed.asset_object_id],
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
