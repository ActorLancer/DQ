#[cfg(test)]
mod tests {
    use super::super::super::api::router as billing_router;
    use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug, Clone)]
    struct SeedOrder {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        outbox_event_id: String,
        publish_attempt_id: String,
        sku_type: String,
    }

    #[tokio::test]
    async fn bil024_billing_trigger_bridge_db_smoke() {
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
            .expect("clock")
            .as_millis()
            .to_string();

        let cases = vec![
            seed_order(
                &client,
                &suffix,
                "FILE_STD",
                "accepted",
                "one_time",
                "one_time",
                "delivery.accept",
                "acceptance_passed",
                "file",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "FILE_SUB",
                "accepted",
                "subscription",
                "subscription",
                "delivery.accept",
                "acceptance_passed",
                "file",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "SHARE_RO",
                "share_enabled",
                "one_time",
                "subscription",
                "delivery.share.manage",
                "delivery_committed",
                "share",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "API_SUB",
                "api_enabled",
                "subscription",
                "subscription",
                "delivery.api.enable",
                "delivery_committed",
                "api",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "API_PPU",
                "api_enabled",
                "usage_metered",
                "usage_metered",
                "delivery.api.enable",
                "delivery_committed",
                "api",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "QRY_LITE",
                "result_available",
                "one_time",
                "one_time",
                "delivery.template.run",
                "delivery_committed",
                "query",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "SBX_STD",
                "workspace_enabled",
                "subscription",
                "subscription",
                "delivery.sandbox.enable",
                "delivery_committed",
                "sandbox",
            )
            .await,
            seed_order(
                &client,
                &suffix,
                "RPT_STD",
                "accepted",
                "one_time",
                "one_time",
                "delivery.accept",
                "acceptance_passed",
                "report",
            )
            .await,
        ];

        let app = crate::with_live_test_state(billing_router()).await;
        for seed in &cases {
            let response =
                process_bridge(&app, &seed.order_id, &seed.outbox_event_id, &suffix).await;
            let data = &response["data"];
            match seed.sku_type.as_str() {
                "API_SUB" | "API_PPU" => {
                    assert_eq!(data["processed_count"].as_u64(), Some(0));
                    assert_eq!(data["ignored_count"].as_u64(), Some(1));
                    assert_eq!(
                        data["ignored_outbox_event_ids"][0].as_str(),
                        Some(seed.outbox_event_id.as_str())
                    );
                }
                "FILE_SUB" | "SBX_STD" => {
                    assert_eq!(data["processed_count"].as_u64(), Some(1));
                    assert_eq!(data["replayed_count"].as_u64(), Some(0));
                    let billing_event_id = data["processed_billing_event_ids"][0]
                        .as_str()
                        .expect("processed billing event id");
                    let row = client
                        .query_one(
                            "SELECT
                                event_type,
                                event_source,
                                metadata ->> 'bridge_outbox_event_id',
                                metadata ->> 'bridge_publish_attempt_id'
                             FROM billing.billing_event
                             WHERE billing_event_id = $1::text::uuid",
                            &[&billing_event_id],
                        )
                        .await
                        .expect("query recurring billing event");
                    assert_eq!(row.get::<_, String>(0), "recurring_charge");
                    assert_eq!(
                        row.get::<_, Option<String>>(2).as_deref(),
                        Some(seed.outbox_event_id.as_str())
                    );
                    assert_eq!(
                        row.get::<_, Option<String>>(3).as_deref(),
                        Some(seed.publish_attempt_id.as_str())
                    );
                }
                _ => {
                    assert_eq!(data["processed_count"].as_u64(), Some(1));
                    let billing_event_id = data["processed_billing_event_ids"][0]
                        .as_str()
                        .expect("processed billing event id");
                    let row = client
                        .query_one(
                            "SELECT
                                event_type,
                                metadata ->> 'bridge_outbox_event_id',
                                metadata ->> 'bridge_publish_attempt_id'
                             FROM billing.billing_event
                             WHERE billing_event_id = $1::text::uuid",
                            &[&billing_event_id],
                        )
                        .await
                        .expect("query one-time billing event");
                    assert_eq!(row.get::<_, String>(0), "one_time_charge");
                    assert_eq!(
                        row.get::<_, Option<String>>(1).as_deref(),
                        Some(seed.outbox_event_id.as_str())
                    );
                    assert_eq!(
                        row.get::<_, Option<String>>(2).as_deref(),
                        Some(seed.publish_attempt_id.as_str())
                    );
                }
            }
            let outbox_row = client
                .query_one(
                    "SELECT status, published_at IS NOT NULL
                       FROM ops.outbox_event
                      WHERE outbox_event_id = $1::text::uuid",
                    &[&seed.outbox_event_id],
                )
                .await
                .expect("query outbox status");
            assert_eq!(outbox_row.get::<_, String>(0), "published");
            assert!(outbox_row.get::<_, bool>(1));
        }

        let unpublished_seed = seed_order_without_publish_attempt(
            &client,
            &suffix,
            "FILE_STD",
            "accepted",
            "one_time",
            "one_time",
            "delivery.accept",
            "acceptance_passed",
            "file",
        )
        .await;
        let unpublished_response = process_bridge(
            &app,
            &unpublished_seed.order_id,
            &unpublished_seed.outbox_event_id,
            &format!("{suffix}-unpublished"),
        )
        .await;
        let unpublished_data = &unpublished_response["data"];
        assert_eq!(unpublished_data["processed_count"].as_u64(), Some(0));
        assert_eq!(unpublished_data["ignored_count"].as_u64(), Some(0));
        let unpublished_billing_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM billing.billing_event
                 WHERE order_id = $1::text::uuid",
                &[&unpublished_seed.order_id],
            )
            .await
            .expect("count unpublished billing events")
            .get(0);
        assert_eq!(unpublished_billing_count, 0);

        let processed_settlement_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM billing.settlement_record
                 WHERE order_id = ANY($1::text[]::uuid[])",
                &[&cases
                    .iter()
                    .filter(|seed| !matches!(seed.sku_type.as_str(), "API_SUB" | "API_PPU"))
                    .map(|seed| seed.order_id.clone())
                    .collect::<Vec<_>>()],
            )
            .await
            .expect("count settlement records")
            .get(0);
        assert_eq!(processed_settlement_count, 6);

        let mut cleanup_orders = cases.clone();
        cleanup_orders.push(unpublished_seed);
        cleanup_seed_orders(&client, &cleanup_orders).await;
    }

    async fn process_bridge(
        app: &Router,
        order_id: &str,
        outbox_event_id: &str,
        suffix: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/billing/{order_id}/bridge-events/process"))
                    .header("x-role", "platform_finance_operator")
                    .header(
                        "x-request-id",
                        format!("req-bil024-process-{suffix}-{order_id}"),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "outbox_event_id": outbox_event_id
                        })
                        .to_string(),
                    ))
                    .expect("bridge process request"),
            )
            .await
            .expect("bridge process response");
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

    #[allow(clippy::too_many_arguments)]
    async fn seed_order(
        client: &Client,
        suffix: &str,
        sku_type: &str,
        order_status: &str,
        pricing_mode: &str,
        billing_mode: &str,
        trigger_action: &str,
        trigger_stage: &str,
        delivery_branch: &str,
    ) -> SeedOrder {
        seed_order_with_publish_attempt(
            client,
            suffix,
            sku_type,
            order_status,
            pricing_mode,
            billing_mode,
            trigger_action,
            trigger_stage,
            delivery_branch,
            true,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn seed_order_without_publish_attempt(
        client: &Client,
        suffix: &str,
        sku_type: &str,
        order_status: &str,
        pricing_mode: &str,
        billing_mode: &str,
        trigger_action: &str,
        trigger_stage: &str,
        delivery_branch: &str,
    ) -> SeedOrder {
        seed_order_with_publish_attempt(
            client,
            suffix,
            sku_type,
            order_status,
            pricing_mode,
            billing_mode,
            trigger_action,
            trigger_stage,
            delivery_branch,
            false,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn seed_order_with_publish_attempt(
        client: &Client,
        suffix: &str,
        sku_type: &str,
        order_status: &str,
        pricing_mode: &str,
        billing_mode: &str,
        trigger_action: &str,
        trigger_stage: &str,
        delivery_branch: &str,
        include_publish_attempt: bool,
    ) -> SeedOrder {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil024-buyer-{sku_type}-{suffix}")],
            )
            .await
            .expect("insert buyer")
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil024-seller-{sku_type}-{suffix}")],
            )
            .await
            .expect("insert seller")
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'active', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("bil024-asset-{sku_type}-{suffix}"),
                    &format!("bil024 asset {sku_type} {suffix}"),
                ],
            )
            .await
            .expect("insert asset")
            .get(0);
        let asset_version_id: String = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status
                 ) VALUES (
                   $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
                   1024, 'SG', ARRAY['SG']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id],
            )
            .await
            .expect("insert asset version")
            .get(0);
        let product_id: String = client
            .query_one(
                "INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type,
                   description, status, price_mode, price, currency_code, delivery_type,
                   allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', $6, 66.00, 'SGD', $7,
                   ARRAY['billing_use']::text[], $8, '{\"review_status\":\"approved\"}'::jsonb
                 )
                 RETURNING product_id::text",
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("bil024-product-{sku_type}-{suffix}"),
                    &format!("bil024 product {sku_type} {suffix}"),
                    &pricing_mode,
                    &delivery_type_for_sku(sku_type),
                    &format!("bil024 searchable {sku_type} {suffix}"),
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
                   $1::text::uuid, $2, $3, '份', $4, 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("BIL024-SKU-{sku_type}-{suffix}"),
                    &sku_type,
                    &billing_mode,
                ],
            )
            .await
            .expect("insert sku")
            .get(0);

        let acceptance_status = match trigger_stage {
            "acceptance_passed" => "accepted",
            _ => "accepted",
        };
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, 'paid', 'delivered', $7, 'pending_settlement', 'none',
                   'online', 66.00, 'SGD',
                   jsonb_build_object(
                     'sku_type', $8,
                     'selected_sku_type', $8,
                     'pricing_mode', $9,
                     'billing_mode', $10,
                     'settlement_basis', CASE WHEN $10 = 'usage_metered' THEN 'usage' WHEN $10 = 'subscription' THEN 'periodic' ELSE 'one_time' END,
                     'refund_mode', 'manual_refund',
                     'refund_template_code', format('REFUND_%s_V1', $8)
                   )
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &order_status,
                    &acceptance_status,
                    &sku_type,
                    &pricing_mode,
                    &billing_mode,
                ],
            )
            .await
            .expect("insert order")
            .get(0);

        let bridge_request_id = format!("req-bil024-bridge-{sku_type}-{suffix}");
        let bridge_idempotency_key =
            format!("billing-trigger:{order_id}:{trigger_stage}:{trigger_action}");
        let bridge_payload = json!({
            "event_schema_version": "v1",
            "authority_scope": "business",
            "source_of_truth": "database",
            "proof_commit_policy": "async_evidence",
            "order_id": order_id,
            "sku_type": sku_type,
            "buyer_org_id": buyer_org_id,
            "seller_org_id": seller_org_id,
            "current_state": order_status,
            "payment_status": "paid",
            "delivery_status": "delivered",
            "acceptance_status": acceptance_status,
            "settlement_status": "pending_settlement",
            "dispute_status": "none",
            "amount": "66.00000000",
            "currency_code": "SGD",
            "pricing_mode": pricing_mode,
            "billing_mode": billing_mode,
            "refund_mode": "manual_refund",
            "trigger_stage": trigger_stage,
            "trigger_ref_type": "delivery_record",
            "trigger_ref_id": order_id,
            "trigger_action": trigger_action,
            "delivery_branch": delivery_branch,
            "billing_trigger_matrix": {
                "billing_trigger": match sku_type {
                    "FILE_SUB" | "SBX_STD" | "API_SUB" => "recurring_charge",
                    "API_PPU" => "usage_charge",
                    _ => "one_time_charge"
                }
            }
        });
        write_canonical_outbox_event(
            client,
            CanonicalOutboxWrite {
                aggregate_type: "trade.order_main",
                aggregate_id: &order_id,
                event_type: "billing.trigger.bridge",
                producer_service: "platform-core.delivery",
                request_id: Some(bridge_request_id.as_str()),
                trace_id: None,
                idempotency_key: Some(bridge_idempotency_key.as_str()),
                occurred_at: None,
                business_payload: &bridge_payload,
                deduplicate_by_idempotency_key: true,
            },
        )
        .await
        .expect("insert bridge outbox");
        let outbox_event_id: String = client
            .query_one(
                "SELECT outbox_event_id::text
                 FROM ops.outbox_event
                 WHERE idempotency_key = $1
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&bridge_idempotency_key],
            )
            .await
            .expect("query bridge outbox")
            .get(0);
        let publish_attempt_id = if include_publish_attempt {
            client
                .query_one(
                    "INSERT INTO ops.outbox_publish_attempt (
                       outbox_event_id,
                       worker_id,
                       target_bus,
                       target_topic,
                       attempt_no,
                       result_code,
                       attempted_at,
                       completed_at
                     ) VALUES (
                       $1::text::uuid,
                       'seed-bil024-outbox-publisher',
                       'kafka',
                       'dtp.outbox.domain-events',
                       1,
                       'published',
                       now(),
                       now()
                     )
                     RETURNING outbox_publish_attempt_id::text",
                    &[&outbox_event_id],
                )
                .await
                .expect("insert bridge publish attempt")
                .get(0)
        } else {
            format!("missing-publish-attempt-{outbox_event_id}")
        };
        client
            .execute(
                "UPDATE ops.outbox_event
                    SET status = 'published',
                        published_at = now()
                  WHERE outbox_event_id = $1::text::uuid",
                &[&outbox_event_id],
            )
            .await
            .expect("mark bridge outbox published");

        SeedOrder {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            outbox_event_id,
            publish_attempt_id,
            sku_type: sku_type.to_string(),
        }
    }

    async fn cleanup_seed_orders(client: &Client, orders: &[SeedOrder]) {
        let order_ids = orders
            .iter()
            .map(|seed| seed.order_id.clone())
            .collect::<Vec<_>>();
        let sku_ids = orders
            .iter()
            .map(|seed| seed.sku_id.clone())
            .collect::<Vec<_>>();
        let product_ids = orders
            .iter()
            .map(|seed| seed.product_id.clone())
            .collect::<Vec<_>>();
        let version_ids = orders
            .iter()
            .map(|seed| seed.asset_version_id.clone())
            .collect::<Vec<_>>();
        let asset_ids = orders
            .iter()
            .map(|seed| seed.asset_id.clone())
            .collect::<Vec<_>>();
        let org_ids = orders
            .iter()
            .flat_map(|seed| [seed.buyer_org_id.clone(), seed.seller_org_id.clone()])
            .collect::<Vec<_>>();

        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = ANY($1::text[]::uuid[])",
                &[&order_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = ANY($1::text[]::uuid[])",
                &[&sku_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = ANY($1::text[]::uuid[])",
                &[&product_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = ANY($1::text[]::uuid[])",
                &[&version_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = ANY($1::text[]::uuid[])",
                &[&asset_ids],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&org_ids],
            )
            .await;
    }

    fn delivery_type_for_sku(sku_type: &str) -> &'static str {
        match sku_type {
            "FILE_STD" | "FILE_SUB" => "file_package",
            "SHARE_RO" => "read_only_share",
            "API_SUB" | "API_PPU" => "api_access",
            "QRY_LITE" => "template_query",
            "SBX_STD" => "sandbox_workspace",
            "RPT_STD" => "report_delivery",
            _ => "file_package",
        }
    }
}
