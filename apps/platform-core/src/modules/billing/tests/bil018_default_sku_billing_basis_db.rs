#[cfg(test)]
mod tests {
    use super::super::super::api::router as billing_router;
    use crate::modules::order::api::router as order_router;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedGraph {
        buyer_org_id: String,
        seller_org_id: String,
        file_asset_id: String,
        file_asset_version_id: String,
        file_product_id: String,
        file_sku_id: String,
        file_order_id: String,
        share_asset_id: String,
        share_asset_version_id: String,
        share_product_id: String,
        share_sku_id: String,
        share_order_id: String,
    }

    #[tokio::test]
    async fn bil018_default_sku_billing_basis_db_smoke() {
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
        let app = crate::with_live_test_state(billing_router().merge(order_router())).await;

        let file_billing = get_billing_order(
            &app,
            &seed.file_order_id,
            &seed.buyer_org_id,
            &format!("req-bil018-file-{suffix}"),
        )
        .await;
        assert_eq!(
            file_billing["data"]["sku_billing_basis"]["sku_type"].as_str(),
            Some("FILE_STD")
        );
        assert_eq!(
            file_billing["data"]["sku_billing_basis"]["default_event_type"].as_str(),
            Some("one_time_charge")
        );
        assert_eq!(
            file_billing["data"]["sku_billing_basis"]["refund_entry"].as_str(),
            Some("pre_acceptance_cancel_or_acceptance_failed")
        );
        assert_eq!(
            file_billing["data"]["sku_billing_basis"]["refund_mode"].as_str(),
            Some("manual_refund")
        );
        assert_eq!(
            file_billing["data"]["sku_billing_basis"]["refund_template_code"].as_str(),
            Some("REFUND_FILE_STD_V1")
        );

        let share_billing_before = get_billing_order(
            &app,
            &seed.share_order_id,
            &seed.buyer_org_id,
            &format!("req-bil018-share-before-{suffix}"),
        )
        .await;
        assert_eq!(
            share_billing_before["data"]["sku_billing_basis"]["sku_type"].as_str(),
            Some("SHARE_RO")
        );
        assert_eq!(
            share_billing_before["data"]["sku_billing_basis"]["default_event_type"].as_str(),
            Some("one_time_charge")
        );
        assert_eq!(
            share_billing_before["data"]["sku_billing_basis"]["billing_trigger"].as_str(),
            Some("bill_once_on_grant_effective")
        );
        assert_eq!(
            share_billing_before["data"]["sku_billing_basis"]["refund_entry"].as_str(),
            Some("refund_if_grant_not_effective")
        );
        assert_eq!(
            share_billing_before["data"]["billing_events"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );

        let transition = transition_share_enable(
            &app,
            &seed.share_order_id,
            &seed.buyer_org_id,
            &format!("req-bil018-share-enable-{suffix}"),
        )
        .await;
        assert_eq!(
            transition["data"]["data"]["current_state"].as_str(),
            Some("share_enabled")
        );
        assert_eq!(
            transition["data"]["data"]["billing_event_type"].as_str(),
            Some("one_time_charge")
        );
        assert_eq!(
            transition["data"]["data"]["billing_event_replayed"].as_bool(),
            Some(false)
        );
        let billing_event_id = transition["data"]["data"]["billing_event_id"]
            .as_str()
            .expect("share billing event id")
            .to_string();

        let share_billing_after = get_billing_order(
            &app,
            &seed.share_order_id,
            &seed.buyer_org_id,
            &format!("req-bil018-share-after-{suffix}"),
        )
        .await;
        assert_eq!(
            share_billing_after["data"]["billing_events"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            share_billing_after["data"]["billing_events"][0]["event_type"].as_str(),
            Some("one_time_charge")
        );

        let counts = client
            .query_one(
                "SELECT
                   (SELECT COUNT(*)::bigint FROM billing.billing_event WHERE order_id = $1::text::uuid AND event_type = 'one_time_charge'),
                   (SELECT COUNT(*)::bigint FROM ops.outbox_event WHERE aggregate_id = $2::text::uuid AND target_topic = 'dtp.outbox.domain-events'),
                   (SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $3 AND action_name = 'billing.event.record.share_ro_enable')
                 ",
                &[
                    &seed.share_order_id,
                    &billing_event_id,
                    &format!("req-bil018-share-enable-{suffix}"),
                ],
            )
            .await
            .expect("count billing artifacts");
        assert_eq!(counts.get::<_, i64>(0), 1);
        assert_eq!(counts.get::<_, i64>(1), 1);
        assert_eq!(counts.get::<_, i64>(2), 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn get_billing_order(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .body(Body::empty())
                    .expect("billing read request"),
            )
            .await
            .expect("billing read response");
        json_response(response, StatusCode::OK).await
    }

    async fn transition_share_enable(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        request_id: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{order_id}/share-ro/transition"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"enable_share"}"#))
                    .expect("share enable request"),
            )
            .await
            .expect("share enable response");
        json_response(response, StatusCode::OK).await
    }

    async fn json_response(response: axum::response::Response, expected: StatusCode) -> Value {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        assert_eq!(status, expected, "{}", String::from_utf8_lossy(&body));
        serde_json::from_slice(&body).expect("json body")
    }

    async fn seed_graph(client: &Client, suffix: &str) -> SeedGraph {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil018-buyer-{suffix}")],
            )
            .await
            .expect("insert buyer org")
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil018-seller-{suffix}")],
            )
            .await
            .expect("insert seller org")
            .get(0);

        let file_asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil018-file-asset-{suffix}"),
                    &format!("bil018 file asset {suffix}"),
                ],
            )
            .await
            .expect("insert file asset")
            .get(0);
        let file_asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'file-schema-hash', 'file-sample-hash', 'file-full-hash',
                   2048, 'SG', ARRAY['SG']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&file_asset_id],
            )
            .await
            .expect("insert file asset version")
            .get(0);
        let file_product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 88.00, 'SGD', 'file_package',
                   ARRAY['billing_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &file_asset_id,
                    &file_asset_version_id,
                    &seller_org_id,
                    &format!("bil018-file-product-{suffix}"),
                    &format!("bil018 file product {suffix}"),
                    &format!("bil018 file summary {suffix}"),
                ],
            )
            .await
            .expect("insert file product")
            .get(0);
        let file_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&file_product_id, &format!("BIL018-FILE-SKU-{suffix}")],
            )
            .await
            .expect("insert file sku")
            .get(0);
        let file_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'delivered', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   jsonb_build_object(
                     'sku_type', 'FILE_STD',
                     'selected_sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'billing_mode', 'one_time',
                     'settlement_basis', 'one_time',
                     'refund_mode', 'manual_refund',
                     'refund_template_code', 'REFUND_FILE_STD_V1'
                   )
                 )
                 RETURNING order_id::text",
                &[&file_product_id, &file_asset_version_id, &buyer_org_id, &seller_org_id, &file_sku_id],
            )
            .await
            .expect("insert file order")
            .get(0);

        let share_asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil018-share-asset-{suffix}"),
                    &format!("bil018 share asset {suffix}"),
                ],
            )
            .await
            .expect("insert share asset")
            .get(0);
        let share_asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'share-schema-hash', 'share-sample-hash', 'share-full-hash',
                   1024, 'SG', ARRAY['SG']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&share_asset_id],
            )
            .await
            .expect("insert share asset version")
            .get(0);
        let share_product_id: String = client
            .query_one(
                r#"INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 21.50, 'SGD', 'read_only_share',
                   ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &share_asset_id,
                    &share_asset_version_id,
                    &seller_org_id,
                    &format!("bil018-share-product-{suffix}"),
                    &format!("bil018 share product {suffix}"),
                    &format!("bil018 share summary {suffix}"),
                ],
            )
            .await
            .expect("insert share product")
            .get(0);
        let share_sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '月', 'subscription', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&share_product_id, &format!("BIL018-SHARE-SKU-{suffix}")],
            )
            .await
            .expect("insert share sku")
            .get(0);
        let share_order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'pending_delivery', 'not_started', 'pending_settlement', 'none',
                   'online', 21.50, 'SGD',
                   jsonb_build_object(
                     'sku_type', 'SHARE_RO',
                     'selected_sku_type', 'SHARE_RO',
                     'pricing_mode', 'one_time',
                     'billing_mode', 'subscription',
                     'settlement_basis', 'one_time',
                     'refund_mode', 'manual_refund',
                     'refund_template_code', 'REFUND_SHARE_RO_V1'
                   )
                 )
                 RETURNING order_id::text",
                &[&share_product_id, &share_asset_version_id, &buyer_org_id, &seller_org_id, &share_sku_id],
            )
            .await
            .expect("insert share order")
            .get(0);
        client
            .execute(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{\"term_days\":30}'::jsonb
                 )",
                &[&share_order_id, &format!("sha256:bil018:{suffix}")],
            )
            .await
            .expect("insert share contract");

        SeedGraph {
            buyer_org_id,
            seller_org_id,
            file_asset_id,
            file_asset_version_id,
            file_product_id,
            file_sku_id,
            file_order_id,
            share_asset_id,
            share_asset_version_id,
            share_product_id,
            share_sku_id,
            share_order_id,
        }
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id IN (
                   SELECT billing_event_id FROM billing.billing_event WHERE order_id = ANY($1::text[]::uuid[])
                 )",
                &[&vec![seed.file_order_id.clone(), seed.share_order_id.clone()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_order_id.clone(),
                    seed.share_order_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_order_id.clone(),
                    seed.share_order_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE order_id = $1::text::uuid",
                &[&seed.share_order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_order_id.clone(),
                    seed.share_order_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.file_sku_id.clone(), seed.share_sku_id.clone()]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_product_id.clone(),
                    seed.share_product_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_asset_version_id.clone(),
                    seed.share_asset_version_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = ANY($1::text[]::uuid[])",
                &[&vec![
                    seed.file_asset_id.clone(),
                    seed.share_asset_id.clone(),
                ]],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
