#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
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
        storage_namespace_id: String,
        delivery_id: String,
    }

    #[tokio::test]
    async fn dlv002_file_delivery_commit_db_smoke() {
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
        let request_id = format!("req-dlv002-{suffix}");
        let app = crate::with_live_test_state(delivery_router().merge(order_router())).await;

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", &request_id)
                    .body(Body::from(
                        json!({
                            "branch": "file",
                            "object_uri": format!("s3://delivery-objects/orders/{suffix}/payload.enc"),
                            "content_type": "application/octet-stream",
                            "size_bytes": 1024,
                            "content_hash": format!("sha256:content:{suffix}"),
                            "encryption_algo": "AES-GCM",
                            "key_cipher": format!("cipher-{suffix}"),
                            "key_control_mode": "seller_managed",
                            "unwrap_policy_json": {
                                "kms": "local-mock",
                                "buyer_org_id": seed.buyer_org_id,
                            },
                            "key_version": "v1",
                            "expire_at": "2026-05-20T10:00:00Z",
                            "download_limit": 5,
                            "delivery_commit_hash": format!("commit-{suffix}"),
                            "receipt_hash": format!("receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("request build"),
            )
            .await
            .expect("deliver response");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("deliver body");
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let json: Value = serde_json::from_slice(&body).expect("deliver json");
        assert_eq!(
            json["data"]["data"]["current_state"].as_str(),
            Some("delivered")
        );
        assert_eq!(json["data"]["data"]["ticket_id"].as_str().is_some(), true);
        assert_eq!(
            json["data"]["data"]["bucket_name"].as_str(),
            Some("delivery-objects")
        );
        assert_eq!(
            json["data"]["data"]["object_key"].as_str(),
            Some(format!("orders/{suffix}/payload.enc").as_str())
        );
        assert_eq!(json["data"]["data"]["download_limit"].as_i64(), Some(5));

        let detail_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", format!("{request_id}-detail"))
                    .body(Body::empty())
                    .expect("detail request build"),
            )
            .await
            .expect("detail response");
        let detail_status = detail_response.status();
        let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .expect("detail body");
        assert_eq!(
            detail_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&detail_body)
        );
        let detail_json: Value = serde_json::from_slice(&detail_body).expect("detail json");
        assert_eq!(
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]["object_locator"]["bucket_name"].as_str(),
            Some("delivery-objects")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]["watermark_policy"]["mode"].as_str(),
            Some("rule_bound")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]["watermark_policy"]["rule"]["delivery_branch"].as_str(),
            Some("file")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]["watermark_policy"]["rule"]["pipeline"]["status"].as_str(),
            Some("reserved")
        );
        assert_eq!(
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]["watermark_policy"]["rule"]["fingerprint_strategy"].as_str(),
            Some("field_bound")
        );

        let order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, settlement_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .expect("query order row");
        assert_eq!(order_row.get::<_, String>(0), "delivered");
        assert_eq!(order_row.get::<_, String>(1), "delivered");
        assert_eq!(order_row.get::<_, String>(2), "pending_acceptance");
        assert_eq!(order_row.get::<_, String>(3), "pending_settlement");

        let delivery_row = client
            .query_one(
                "SELECT status,
                        object_id::text,
                        envelope_id::text,
                        delivery_commit_hash,
                        receipt_hash,
                        trust_boundary_snapshot -> 'watermark_policy' ->> 'delivery_branch',
                        trust_boundary_snapshot -> 'watermark_policy' -> 'pipeline' ->> 'status'
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await
            .expect("query delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "committed");
        assert!(delivery_row.get::<_, Option<String>>(1).is_some());
        assert!(delivery_row.get::<_, Option<String>>(2).is_some());
        assert_eq!(
            delivery_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("commit-{suffix}").as_str())
        );
        assert_eq!(
            delivery_row.get::<_, Option<String>>(4).as_deref(),
            Some(format!("receipt-{suffix}").as_str())
        );
        assert_eq!(
            delivery_row.get::<_, Option<String>>(5).as_deref(),
            Some("file")
        );
        assert_eq!(
            delivery_row.get::<_, Option<String>>(6).as_deref(),
            Some("reserved")
        );

        let ticket_row = client
            .query_one(
                "SELECT download_limit, download_count, status
                 FROM delivery.delivery_ticket
                 WHERE order_id = $1::text::uuid
                 ORDER BY created_at DESC, ticket_id DESC
                 LIMIT 1",
                &[&seed.order_id],
            )
            .await
            .expect("query ticket row");
        assert_eq!(ticket_row.get::<_, i32>(0), 5);
        assert_eq!(ticket_row.get::<_, i32>(1), 0);
        assert_eq!(ticket_row.get::<_, String>(2), "active");

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.file.commit'
                   AND request_id = $1",
                &[&request_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert_eq!(audit_count, 1);

        let outbox_row = client
            .query_one(
                "SELECT target_topic, payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'delivery.committed'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&request_id],
            )
            .await
            .expect("query outbox row");
        assert_eq!(
            outbox_row.get::<_, Option<String>>(0).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        let outbox_payload: Value = outbox_row.get(1);
        assert_eq!(
            outbox_payload["event_type"].as_str(),
            Some("delivery.committed")
        );
        assert_eq!(
            outbox_payload["producer_service"].as_str(),
            Some("platform-core.delivery")
        );
        assert_eq!(outbox_payload["delivery_branch"].as_str(), Some("file"));
        assert_eq!(
            outbox_payload["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            outbox_payload["receipt_hash"].as_str(),
            Some(format!("receipt-{suffix}").as_str())
        );
        assert_eq!(
            outbox_payload["payload"]["delivery_branch"].as_str(),
            Some("file")
        );
        let billing_bridge_row = client
            .query_one(
                "SELECT target_topic, payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'billing.trigger.bridge'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&request_id],
            )
            .await
            .expect("query billing bridge row");
        assert_eq!(
            billing_bridge_row.get::<_, Option<String>>(0).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        let billing_bridge_payload: Value = billing_bridge_row.get(1);
        assert_eq!(
            billing_bridge_payload["event_type"].as_str(),
            Some("billing.trigger.bridge")
        );
        assert_eq!(
            billing_bridge_payload["producer_service"].as_str(),
            Some("platform-core.delivery")
        );
        assert_eq!(
            billing_bridge_payload["delivery_branch"].as_str(),
            Some("file")
        );
        assert_eq!(
            billing_bridge_payload["trigger_stage"].as_str(),
            Some("delivery_committed")
        );
        assert_eq!(
            billing_bridge_payload["billing_trigger_matrix"]["billing_trigger"].as_str(),
            Some("bill_once_after_acceptance")
        );
        assert_eq!(
            billing_bridge_payload["payload"]["delivery_branch"].as_str(),
            Some("file")
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv002-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv002-seller-{suffix}")],
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
                    &format!("dlv002-asset-{suffix}"),
                    &format!("dlv002 asset {suffix}"),
                ],
            )
            .await?
            .get(0);
        let asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   2048, 'CN', ARRAY['CN']::text[], false,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $2),
                   'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id, &format!("wmk-{suffix}")],
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
                   $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv002-product-{suffix}"),
                    &format!("dlv002 product {suffix}"),
                    &format!("dlv002 search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV002-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'seller_delivering', 'paid', 'online', 88.00, 'CNY',
                   'in_progress', 'not_started', 'pending_settlement', 'none',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '88.00',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776510000000',
                     'source', 'seed'
                   )::jsonb,
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $7),
                   'signed_url'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("DLV002-SKU-{suffix}"),
                    &format!("wmk-{suffix}"),
                ],
            )
            .await?
            .get(0);
        let contract_id: String = client
            .query_one(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, 'signed', now(), '{}'::jsonb
                 )
                 RETURNING contract_id::text",
                &[&order_id, &format!("sha256:dlv002:{suffix}")],
            )
            .await?
            .get(0);
        client
            .execute(
                "UPDATE trade.order_main
                 SET contract_id = $2::text::uuid,
                     updated_at = now()
                 WHERE order_id = $1::text::uuid",
                &[&order_id, &contract_id],
            )
            .await?;
        let storage_namespace_id: String = client
            .query_one(
                "INSERT INTO catalog.storage_namespace (
                   owner_org_id, namespace_name, provider_type, namespace_kind, bucket_name, prefix_rule, status
                 ) VALUES (
                   $1::text::uuid, $2, 's3_compatible', 'product', 'delivery-objects', 'orders/{order_id}', 'active'
                 )
                 RETURNING storage_namespace_id::text",
                &[&seller_org_id, &format!("dlv002-ns-{suffix}")],
            )
            .await?
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'prepared',
                   jsonb_build_object('watermark_rule', 'buyer_bound', 'fingerprint_fields', jsonb_build_array('buyer_org_id','request_id'), 'watermark_hash', $2),
                   'standard', 'not_required'
                 )
                 RETURNING delivery_id::text",
                &[&order_id, &format!("wmk-{suffix}")],
            )
            .await?
            .get(0);

        Ok(SeedGraph {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            storage_namespace_id,
            delivery_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&seed.sku_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&seed.product_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&seed.asset_version_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&seed.asset_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await
            .ok();
    }
}
