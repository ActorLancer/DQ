#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::delivery::domain::expected_acceptance_status_for_state;
    use crate::modules::order::api::router as order_router;
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
    }

    #[tokio::test]
    async fn dlv006_share_grant_db_smoke() {
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

        let grant_request_id = format!("req-dlv006-grant-{suffix}");
        let grant_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/share-grants", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &grant_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "operation": "grant",
                            "asset_object_id": seed.asset_object_id,
                            "recipient_ref": format!("warehouse://buyer/{suffix}"),
                            "subscriber_ref": format!("sub-{suffix}"),
                            "share_protocol": "share_grant",
                            "access_locator": format!("share://seller/{suffix}/dataset"),
                            "scope_json": {"schema": "analytics", "tables": ["orders", "inventory"]},
                            "expires_at": "2026-06-01T00:00:00Z",
                            "receipt_hash": format!("share-receipt-{suffix}"),
                            "metadata": {"channel": "manual-open"}
                        })
                        .to_string(),
                    ))
                    .expect("grant request"),
            )
            .await
            .expect("grant response");
        let grant_status = grant_response.status();
        let grant_body = to_bytes(grant_response.into_body(), usize::MAX)
            .await
            .expect("grant body");
        assert_eq!(
            grant_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&grant_body)
        );
        let grant_json: Value = serde_json::from_slice(&grant_body).expect("grant json");
        assert_eq!(
            grant_json["data"]["data"]["grant_status"].as_str(),
            Some("active")
        );
        assert_eq!(
            grant_json["data"]["data"]["operation"].as_str(),
            Some("granted")
        );
        assert_eq!(
            grant_json["data"]["data"]["current_state"].as_str(),
            Some("share_granted")
        );
        assert_eq!(
            grant_json["data"]["data"]["delivery_status"].as_str(),
            Some("delivered")
        );
        assert_eq!(
            grant_json["data"]["data"]["subscriber_ref"].as_str(),
            Some(format!("sub-{suffix}").as_str())
        );

        let get_request_id = format!("req-dlv006-get-{suffix}");
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}/share-grants", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &get_request_id)
                    .body(Body::empty())
                    .expect("get request"),
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
            get_json["data"]["data"]["grants"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            get_json["data"]["data"]["grants"][0]["grant_status"].as_str(),
            Some("active")
        );

        let granted_order_row = client
            .query_one(
                "SELECT status, payment_status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query granted order row");
        assert_eq!(granted_order_row.get::<_, String>(0), "share_granted");
        assert_eq!(granted_order_row.get::<_, String>(1), "paid");
        assert_eq!(granted_order_row.get::<_, String>(2), "delivered");
        assert_eq!(
            granted_order_row.get::<_, String>(3),
            expected_acceptance_status_for_state("SHARE_RO", "share_granted")
                .expect("share grant acceptance status")
        );
        assert_eq!(granted_order_row.get::<_, String>(4), "pending_settlement");

        let revoke_request_id = format!("req-dlv006-revoke-{suffix}");
        let revoke_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/share-grants", seed.order_id))
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &revoke_request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "operation": "revoke",
                            "receipt_hash": format!("share-revoke-{suffix}"),
                            "metadata": {"reason": "manual revoke"}
                        })
                        .to_string(),
                    ))
                    .expect("revoke request"),
            )
            .await
            .expect("revoke response");
        let revoke_status = revoke_response.status();
        let revoke_body = to_bytes(revoke_response.into_body(), usize::MAX)
            .await
            .expect("revoke body");
        assert_eq!(
            revoke_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&revoke_body)
        );
        let revoke_json: Value = serde_json::from_slice(&revoke_body).expect("revoke json");
        assert_eq!(
            revoke_json["data"]["data"]["grant_status"].as_str(),
            Some("revoked")
        );
        assert_eq!(
            revoke_json["data"]["data"]["operation"].as_str(),
            Some("revoked")
        );
        assert_eq!(
            revoke_json["data"]["data"]["current_state"].as_str(),
            Some("revoked")
        );

        let grant_row = client
            .query_one(
                "SELECT grant_status,
                        recipient_ref,
                        share_protocol,
                        access_locator,
                        receipt_hash,
                        metadata ->> 'subscriber_ref'
                 FROM delivery.data_share_grant
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, data_share_grant_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query grant row");
        assert_eq!(grant_row.get::<_, String>(0), "revoked");
        assert_eq!(
            grant_row.get::<_, String>(1),
            format!("warehouse://buyer/{suffix}")
        );
        assert_eq!(grant_row.get::<_, String>(2), "share_grant");
        assert_eq!(
            grant_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("share://seller/{suffix}/dataset").as_str())
        );
        assert_eq!(
            grant_row.get::<_, Option<String>>(4).as_deref(),
            Some(format!("share-revoke-{suffix}").as_str())
        );
        assert_eq!(
            grant_row.get::<_, Option<String>>(5).as_deref(),
            Some(format!("sub-{suffix}").as_str())
        );

        let delivery_row = client
            .query_one(
                "SELECT status, delivery_type, delivery_route, receipt_hash
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                 ORDER BY updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "revoked");
        assert_eq!(delivery_row.get::<_, String>(1), "share_grant");
        assert_eq!(delivery_row.get::<_, String>(2), "share_grant");
        assert_eq!(
            delivery_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("share-revoke-{suffix}").as_str())
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
        assert_eq!(order_row.get::<_, String>(0), "revoked");
        assert_eq!(order_row.get::<_, String>(1), "paid");
        assert_eq!(order_row.get::<_, String>(2), "closed");
        assert_eq!(order_row.get::<_, String>(3), "closed");
        assert_eq!(order_row.get::<_, String>(4), "closed");

        let manage_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.share.enable'
                   AND request_id IN ($1, $2)",
                &[&grant_request_id, &revoke_request_id],
            )
            .await
            .expect("manage audit count")
            .get(0);
        assert_eq!(manage_audit_count, 2);

        let read_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.share.read'
                   AND request_id = $1",
                &[&get_request_id],
            )
            .await
            .expect("read audit count")
            .get(0);
        assert_eq!(read_audit_count, 1);

        let trade_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'trade.order.share_ro.transition'
                   AND request_id IN ($1, $2)",
                &[&grant_request_id, &revoke_request_id],
            )
            .await
            .expect("trade audit count")
            .get(0);
        assert_eq!(trade_audit_count, 2);

        let outbox_row = client
            .query_one(
                "SELECT target_topic,
                        payload ->> 'delivery_branch',
                        payload ->> 'order_id',
                        payload ->> 'receipt_hash'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'delivery.committed'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&grant_request_id],
            )
            .await
            .expect("share outbox row");
        assert_eq!(
            outbox_row.get::<_, Option<String>>(0).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(1).as_deref(),
            Some("share")
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(2).as_deref(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            outbox_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("share-receipt-{suffix}").as_str())
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
                &[&grant_request_id],
            )
            .await
            .expect("share billing bridge row");
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(0).as_deref(),
            Some("billing.events")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(1).as_deref(),
            Some("share")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(2).as_deref(),
            Some("delivery_committed")
        );
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(3).as_deref(),
            Some("bill_once_on_grant_effective")
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv006-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv006-seller-{suffix}")],
            )
            .await?
            .get(0);

        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("dlv006-asset-{suffix}"),
                    &format!("dlv006 asset {suffix}"),
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
                   1024, 'CN', ARRAY['CN']::text[], false,
                   '{"share_mode":"readonly"}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text"#,
                &[&asset_id],
            )
            .await?
            .get(0);

        let asset_object_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator, share_protocol,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, 'share_object', $2, $3, 'share_grant',
                   '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb
                 )
                 RETURNING asset_object_id::text",
                &[
                    &asset_version_id,
                    &format!("dlv006-share-{suffix}"),
                    &format!("share://seller/{suffix}/source"),
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
                   $5, 'listed', 'subscription', 166.00, 'CNY', 'read_only_share',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","share_protocol":"share_grant"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv006-product-{suffix}"),
                    &format!("dlv006 product {suffix}"),
                    &format!("dlv006 search {suffix}"),
                ],
            )
            .await?
            .get(0);

        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, share_protocol, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '月', 'subscription', 'subscription_access',
                   'share_grant', 'share_grant', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV006-SKU-{suffix}")],
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
                   'online', 166.00, 'CNY',
                   '{"share_protocol":"share_grant"}'::jsonb,
                   'share_link',
                   '{"share_delivery":"readonly"}'::jsonb
                 )
                 RETURNING order_id::text"#,
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
            )
            .await?
            .get(0);

        client
            .execute(
                r#"INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{"term_days":30}'::jsonb
                 )"#,
                &[&order_id, &format!("sha256:dlv006:{suffix}")],
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
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
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
