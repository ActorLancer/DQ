#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use super::super::super::repo::billing_event_repository::{
        RecordBillingEventRequest, record_billing_event,
    };
    use super::super::super::repo::settlement_aggregate_repository::recompute_settlement_for_order;
    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    struct SeedOrderGraph {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn bil016_settlement_summary_outbox_db_smoke() {
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
        let buyer_org_id = seed_org(&client, &format!("bil016-buyer-{suffix}")).await;
        let seller_org_id = seed_org(&client, &format!("bil016-seller-{suffix}")).await;
        let order =
            seed_order_without_settlement(&client, &buyer_org_id, &seller_org_id, &suffix).await;

        record_event(
            &client,
            &order.order_id,
            &buyer_org_id,
            "one_time_charge",
            "payment_webhook",
            Some("88.00000000"),
            json!({
                "idempotency_key": format!("billing_event:{}:charge", order.order_id),
                "provider_event_id": format!("bil016-success-{suffix}"),
            }),
        )
        .await;

        let first = recompute_settlement_for_order(
            &client,
            &order.order_id,
            "platform_finance_operator",
            Some(&format!("req-bil016-recompute-1-{suffix}")),
            None,
        )
        .await
        .expect("first recompute");
        assert_eq!(first.settlement_status, "pending");

        let created_rows = client
            .query(
                "SELECT event_type, target_topic, payload -> 'summary' ->> 'gross_amount',
                        payload -> 'summary' ->> 'summary_state', payload ->> 'proof_commit_state'
                 FROM ops.outbox_event
                 WHERE aggregate_id = $1::text::uuid
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&first.settlement_id],
            )
            .await
            .expect("query created outbox");
        assert_eq!(created_rows.len(), 1);
        let created_event_type: String = created_rows[0].get(0);
        let created_target_topic: Option<String> = created_rows[0].get(1);
        let created_gross_amount: Option<String> = created_rows[0].get(2);
        let created_summary_state: Option<String> = created_rows[0].get(3);
        let created_proof_commit_state: Option<String> = created_rows[0].get(4);
        assert_eq!(created_event_type, "settlement.created");
        assert_eq!(
            created_target_topic.as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(created_gross_amount.as_deref(), Some("88.00000000"));
        assert_eq!(
            created_summary_state.as_deref(),
            Some("order_settlement:pending:manual")
        );
        assert_eq!(
            created_proof_commit_state.as_deref(),
            Some("pending_anchor")
        );

        let _ = recompute_settlement_for_order(
            &client,
            &order.order_id,
            "platform_finance_operator",
            Some(&format!("req-bil016-recompute-1b-{suffix}")),
            None,
        )
        .await
        .expect("repeat pending recompute");
        let created_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE aggregate_id = $1::text::uuid AND event_type = 'settlement.created'",
                &[&first.settlement_id],
            )
            .await
            .expect("count created outbox")
            .get(0);
        assert_eq!(created_count, 1);

        record_event(
            &client,
            &order.order_id,
            &buyer_org_id,
            "manual_settlement",
            "manual_payout_execute",
            Some("85.00000000"),
            json!({
                "idempotency_key": format!("billing_event:{}:manual", order.order_id),
                "settlement_direction": "receivable",
            }),
        )
        .await;

        let completed = recompute_settlement_for_order(
            &client,
            &order.order_id,
            "platform_finance_operator",
            Some(&format!("req-bil016-recompute-2-{suffix}")),
            None,
        )
        .await
        .expect("completed recompute");
        assert_eq!(completed.settlement_id, first.settlement_id);
        assert_eq!(completed.settlement_status, "settled");
        assert_eq!(completed.net_receivable_amount, "85.00000000");

        let _ = recompute_settlement_for_order(
            &client,
            &order.order_id,
            "platform_finance_operator",
            Some(&format!("req-bil016-recompute-2b-{suffix}")),
            None,
        )
        .await
        .expect("repeat completed recompute");

        let completed_rows = client
            .query(
                "SELECT event_type, payload -> 'summary' ->> 'summary_state'
                 FROM ops.outbox_event
                 WHERE aggregate_id = $1::text::uuid
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&completed.settlement_id],
            )
            .await
            .expect("query completed outbox");
        assert_eq!(completed_rows.len(), 2);
        let second_event_type: String = completed_rows[1].get(0);
        let second_summary_state: Option<String> = completed_rows[1].get(1);
        assert_eq!(second_event_type, "settlement.completed");
        assert_eq!(
            second_summary_state.as_deref(),
            Some("order_settlement:settled:manual")
        );

        let completed_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE aggregate_id = $1::text::uuid AND event_type = 'settlement.completed'",
                &[&completed.settlement_id],
            )
            .await
            .expect("count completed outbox")
            .get(0);
        assert_eq!(completed_count, 1);

        let outbox_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'billing.settlement.summary.outbox'
                   AND ref_id = $1::text::uuid",
                &[&completed.settlement_id],
            )
            .await
            .expect("count outbox audit")
            .get(0);
        assert_eq!(outbox_audit_count, 2);

        let app = crate::with_live_test_state(router()).await;
        let detail = get_billing_order(&app, &order.order_id, &buyer_org_id, &suffix).await;
        assert_eq!(
            detail["data"]["settlement_summary"]["summary_state"].as_str(),
            Some("order_settlement:settled:manual")
        );
        assert_eq!(
            detail["data"]["settlement_summary"]["proof_commit_state"].as_str(),
            Some("pending_anchor")
        );

        cleanup(&client, &order).await;
    }

    async fn record_event(
        client: &Client,
        order_id: &str,
        tenant_scope_id: &str,
        event_type: &str,
        event_source: &str,
        amount: Option<&str>,
        metadata: Value,
    ) {
        let payload = RecordBillingEventRequest {
            order_id: order_id.to_string(),
            event_type: event_type.to_string(),
            event_source: event_source.to_string(),
            amount: amount.map(str::to_string),
            currency_code: Some("SGD".to_string()),
            units: None,
            occurred_at: None,
            metadata,
        };
        let _ = record_billing_event(
            client,
            &payload,
            Some(tenant_scope_id),
            "tenant_admin",
            "billing.event.record",
            Some(&format!("req-bil016-{event_type}-{order_id}")),
            None,
        )
        .await
        .expect("record billing event");
    }

    async fn get_billing_order(
        app: &Router,
        order_id: &str,
        tenant_id: &str,
        suffix: &str,
    ) -> Value {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/billing/{order_id}"))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", tenant_id)
                    .header("x-request-id", format!("req-bil016-read-{suffix}"))
                    .body(Body::empty())
                    .expect("billing request should build"),
            )
            .await
            .expect("billing response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("billing body");
        assert_eq!(
            status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).expect("billing json")
    }

    async fn seed_org(client: &Client, org_name: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&org_name],
            )
            .await
            .expect("insert org")
            .get(0)
    }

    async fn seed_order_without_settlement(
        client: &Client,
        buyer_org_id: &str,
        seller_org_id: &str,
        suffix: &str,
    ) -> SeedOrderGraph {
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
                    &format!("bil016-asset-{suffix}"),
                    &format!("bil016 asset {suffix}"),
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
                    &format!("bil016-product-{suffix}"),
                    &format!("bil016 product {suffix}"),
                    &format!("bil016 summary {suffix}"),
                ],
            )
            .await
            .expect("insert product")
            .get(0);
        let sku_code = format!("BIL016-SKU-{suffix}");
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'auto_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &sku_code],
            )
            .await
            .expect("insert sku")
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, fee_preview_snapshot, price_snapshot_json, last_reason_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'buyer_locked', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 88.00, 'SGD',
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'platform_fee_amount', '2.00',
                     'channel_fee_amount', '1.00',
                     'payable_total_amount', '88.00',
                     'currency_code', 'SGD'
                   ),
                   jsonb_build_object(
                     'pricing_mode', 'one_time',
                     'billing_mode', 'one_time',
                     'settlement_basis', 'one_time',
                     'price_currency_code', 'USD',
                     'currency_code', 'SGD'
                   ),
                   'bil016_seed'
                 )
                 RETURNING order_id::text",
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

        SeedOrderGraph {
            buyer_org_id: buyer_org_id.to_string(),
            seller_org_id: seller_org_id.to_string(),
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
        }
    }

    async fn cleanup(client: &Client, order: &SeedOrderGraph) {
        client
            .execute(
                "DELETE FROM ops.outbox_event WHERE aggregate_id = (
                   SELECT settlement_id FROM billing.settlement_record WHERE order_id = $1::text::uuid
                 )",
                &[&order.order_id],
            )
            .await
            .expect("delete outbox");
        client
            .execute(
                "DELETE FROM billing.billing_event WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("delete billing events");
        client
            .execute(
                "DELETE FROM billing.settlement_record WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("delete settlement");
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&order.order_id],
            )
            .await
            .expect("delete order");
        client
            .execute(
                "DELETE FROM catalog.product_sku WHERE sku_id = $1::text::uuid",
                &[&order.sku_id],
            )
            .await
            .expect("delete sku");
        client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[&order.product_id],
            )
            .await
            .expect("delete product");
        client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[&order.asset_version_id],
            )
            .await
            .expect("delete asset version");
        client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[&order.asset_id],
            )
            .await
            .expect("delete asset");
        client
            .execute(
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&order.buyer_org_id, &order.seller_org_id],
            )
            .await
            .expect("delete orgs");
    }
}
