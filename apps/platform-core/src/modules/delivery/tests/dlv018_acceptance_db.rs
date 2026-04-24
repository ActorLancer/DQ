#[cfg(test)]
mod tests {
    use crate::modules::delivery::api::router as delivery_router;
    use crate::modules::delivery::domain::{
        is_manual_acceptance_state, manual_acceptance_delivery_branch,
    };
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{Client, GenericClient, NoTls, connect};
    use serde_json::{Value, json};
    use tower::util::ServiceExt;

    #[derive(Debug)]
    struct SeedOrder {
        buyer_org_id: String,
        seller_org_id: String,
        asset_id: String,
        asset_version_id: String,
        product_id: String,
        sku_id: String,
        order_id: String,
        delivery_id: String,
    }

    #[tokio::test]
    async fn dlv018_acceptance_db_smoke() {
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
        let file_seed = seed_order(&client, &suffix, "FILE_STD", "delivered", "file_download")
            .await
            .expect("seed file order");
        let report_seed = seed_order(
            &client,
            &suffix,
            "RPT_STD",
            "report_delivered",
            "report_delivery",
        )
        .await
        .expect("seed report order");

        let app = crate::with_live_test_state(delivery_router()).await;
        assert!(is_manual_acceptance_state("FILE_STD", "delivered"));
        assert!(is_manual_acceptance_state("RPT_STD", "report_delivered"));
        assert_eq!(manual_acceptance_delivery_branch("FILE_STD"), Some("file"));
        assert_eq!(manual_acceptance_delivery_branch("RPT_STD"), Some("report"));

        let accept_request_id = format!("req-dlv018-accept-{suffix}");
        let accept_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/accept", file_seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &file_seed.buyer_org_id)
                    .header("x-request-id", &accept_request_id)
                    .header("x-idempotency-key", format!("idem-dlv018-accept-{suffix}"))
                    .body(Body::from(
                        json!({
                            "note": "hash verified and contract matched",
                            "verification_summary": {
                                "hash_match": true,
                                "contract_template_match": true
                            }
                        })
                        .to_string(),
                    ))
                    .expect("accept request"),
            )
            .await
            .expect("accept response");
        let accept_status = accept_response.status();
        let accept_body = to_bytes(accept_response.into_body(), usize::MAX)
            .await
            .expect("accept body");
        assert_eq!(
            accept_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&accept_body)
        );
        let accept_json: Value = serde_json::from_slice(&accept_body).expect("accept json");
        let accept_data = &accept_json["data"];
        assert_eq!(accept_data["current_state"].as_str(), Some("accepted"));
        assert_eq!(accept_data["delivery_branch"].as_str(), Some("file"));
        assert_eq!(accept_data["acceptance_status"].as_str(), Some("accepted"));
        assert_eq!(
            accept_data["settlement_status"].as_str(),
            Some("pending_settlement")
        );
        assert_eq!(
            accept_data["reason_code"].as_str(),
            Some("delivery_accept_passed")
        );
        assert!(accept_data["accepted_at"].as_str().is_some());

        let replay_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/accept", file_seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &file_seed.buyer_org_id)
                    .header("x-request-id", format!("{accept_request_id}-replay"))
                    .header(
                        "x-idempotency-key",
                        format!("idem-dlv018-accept-replay-{suffix}"),
                    )
                    .body(Body::from("{}".to_string()))
                    .expect("accept replay request"),
            )
            .await
            .expect("accept replay response");
        let replay_status = replay_response.status();
        let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .expect("accept replay body");
        assert_eq!(
            replay_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&replay_body)
        );
        let replay_json: Value = serde_json::from_slice(&replay_body).expect("accept replay json");
        assert_eq!(
            replay_json["data"]["operation"].as_str(),
            Some("already_accepted")
        );

        let reject_request_id = format!("req-dlv018-reject-{suffix}");
        let reject_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/reject", report_seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &report_seed.buyer_org_id)
                    .header("x-request-id", &reject_request_id)
                    .header("x-idempotency-key", format!("idem-dlv018-reject-{suffix}"))
                    .body(Body::from(
                        json!({
                            "reason_code": "report_quality_failed",
                            "reason_detail": "sample section mismatched template",
                            "verification_summary": {
                                "hash_match": true,
                                "report_section_check": false
                            }
                        })
                        .to_string(),
                    ))
                    .expect("reject request"),
            )
            .await
            .expect("reject response");
        let reject_status = reject_response.status();
        let reject_body = to_bytes(reject_response.into_body(), usize::MAX)
            .await
            .expect("reject body");
        assert_eq!(
            reject_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&reject_body)
        );
        let reject_json: Value = serde_json::from_slice(&reject_body).expect("reject json");
        let reject_data = &reject_json["data"];
        assert_eq!(reject_data["current_state"].as_str(), Some("rejected"));
        assert_eq!(reject_data["delivery_branch"].as_str(), Some("report"));
        assert_eq!(reject_data["acceptance_status"].as_str(), Some("rejected"));
        assert_eq!(reject_data["settlement_status"].as_str(), Some("blocked"));
        assert_eq!(reject_data["dispute_status"].as_str(), Some("open"));
        assert_eq!(
            reject_data["reason_code"].as_str(),
            Some("report_quality_failed")
        );

        let file_order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, settlement_status,
                        dispute_status, accepted_at IS NOT NULL
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&file_seed.order_id],
            )
            .await
            .expect("query file order");
        assert_eq!(file_order_row.get::<_, String>(0), "accepted");
        assert_eq!(file_order_row.get::<_, String>(1), "delivered");
        assert_eq!(file_order_row.get::<_, String>(2), "accepted");
        assert_eq!(file_order_row.get::<_, String>(3), "pending_settlement");
        assert_eq!(file_order_row.get::<_, String>(4), "none");
        assert!(file_order_row.get::<_, bool>(5));

        let file_delivery_row = client
            .query_one(
                "SELECT status, trust_boundary_snapshot->'acceptance'->>'decision'
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&file_seed.delivery_id],
            )
            .await
            .expect("query file delivery");
        assert_eq!(file_delivery_row.get::<_, String>(0), "accepted");
        assert_eq!(
            file_delivery_row.get::<_, Option<String>>(1).as_deref(),
            Some("accepted")
        );

        let report_order_row = client
            .query_one(
                "SELECT status, delivery_status, acceptance_status, settlement_status, dispute_status
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&report_seed.order_id],
            )
            .await
            .expect("query report order");
        assert_eq!(report_order_row.get::<_, String>(0), "rejected");
        assert_eq!(report_order_row.get::<_, String>(1), "delivered");
        assert_eq!(report_order_row.get::<_, String>(2), "rejected");
        assert_eq!(report_order_row.get::<_, String>(3), "blocked");
        assert_eq!(report_order_row.get::<_, String>(4), "open");

        let report_delivery_row = client
            .query_one(
                "SELECT status,
                        trust_boundary_snapshot->'acceptance'->>'decision',
                        trust_boundary_snapshot->'acceptance'->>'reason_code'
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&report_seed.delivery_id],
            )
            .await
            .expect("query report delivery");
        assert_eq!(report_delivery_row.get::<_, String>(0), "rejected");
        assert_eq!(
            report_delivery_row.get::<_, Option<String>>(1).as_deref(),
            Some("rejected")
        );
        assert_eq!(
            report_delivery_row.get::<_, Option<String>>(2).as_deref(),
            Some("report_quality_failed")
        );

        let accept_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'delivery.accept'",
                &[&accept_request_id],
            )
            .await
            .expect("query accept audit count")
            .get(0);
        assert_eq!(accept_audit_count, 1);
        let accept_billing_bridge_row = client
            .query_one(
                "SELECT target_topic,
                        payload ->> 'trigger_stage',
                        payload -> 'billing_trigger_matrix' ->> 'billing_trigger'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'billing.trigger.bridge'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&accept_request_id],
            )
            .await
            .expect("query accept billing bridge row");
        assert_eq!(
            accept_billing_bridge_row
                .get::<_, Option<String>>(0)
                .as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            accept_billing_bridge_row
                .get::<_, Option<String>>(1)
                .as_deref(),
            Some("acceptance_passed")
        );
        assert_eq!(
            accept_billing_bridge_row
                .get::<_, Option<String>>(2)
                .as_deref(),
            Some("bill_once_after_acceptance")
        );
        let accept_event_row = client
            .query_one(
                "SELECT aggregate_type,
                        target_topic,
                        payload ->> 'reason_code'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'acceptance.passed'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&accept_request_id],
            )
            .await
            .expect("query accept outbox row");
        assert_eq!(
            accept_event_row.get::<_, String>(0),
            "trade.acceptance_record"
        );
        assert_eq!(
            accept_event_row.get::<_, Option<String>>(1).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            accept_event_row.get::<_, Option<String>>(2).as_deref(),
            Some("delivery_accept_passed")
        );
        let accept_notification_rows = client
            .query(
                "SELECT payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&accept_request_id],
            )
            .await
            .expect("query acceptance passed notifications");
        assert_eq!(accept_notification_rows.len(), 3);
        let accept_notification_payloads = accept_notification_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let accept_buyer = find_notification_payload(&accept_notification_payloads, "buyer");
        assert_eq!(
            accept_buyer["payload"]["notification_code"].as_str(),
            Some("acceptance.passed")
        );
        assert_eq!(
            accept_buyer["payload"]["template_code"].as_str(),
            Some("NOTIFY_ACCEPTANCE_PASSED_V1")
        );
        assert_eq!(
            accept_buyer["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/trade/orders/{}", file_seed.order_id).as_str())
        );
        let accept_ops = find_notification_payload(&accept_notification_payloads, "ops");
        assert_eq!(
            accept_ops["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );
        assert_eq!(
            accept_ops["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/billing?order_id={}", file_seed.order_id).as_str())
        );
        let accept_replay_notification_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'",
                &[&format!("{accept_request_id}-replay")],
            )
            .await
            .expect("query acceptance replay notification count")
            .get(0);
        assert_eq!(accept_replay_notification_count, 0);

        let reject_audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'delivery.reject'",
                &[&reject_request_id],
            )
            .await
            .expect("query reject audit count")
            .get(0);
        assert_eq!(reject_audit_count, 1);
        let reject_event_row = client
            .query_one(
                "SELECT aggregate_type,
                        target_topic,
                        payload ->> 'reason_code'
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND event_type = 'acceptance.rejected'
                 ORDER BY created_at DESC, outbox_event_id DESC
                 LIMIT 1",
                &[&reject_request_id],
            )
            .await
            .expect("query reject outbox row");
        assert_eq!(
            reject_event_row.get::<_, String>(0),
            "trade.acceptance_record"
        );
        assert_eq!(
            reject_event_row.get::<_, Option<String>>(1).as_deref(),
            Some("dtp.outbox.domain-events")
        );
        assert_eq!(
            reject_event_row.get::<_, Option<String>>(2).as_deref(),
            Some("report_quality_failed")
        );
        let reject_notification_rows = client
            .query(
                "SELECT payload
                 FROM ops.outbox_event
                 WHERE request_id = $1
                   AND target_topic = 'dtp.notification.dispatch'
                 ORDER BY created_at ASC, outbox_event_id ASC",
                &[&reject_request_id],
            )
            .await
            .expect("query acceptance rejected notifications");
        assert_eq!(reject_notification_rows.len(), 3);
        let reject_notification_payloads = reject_notification_rows
            .iter()
            .map(|row| row.get::<_, Value>(0))
            .collect::<Vec<_>>();
        let reject_buyer = find_notification_payload(&reject_notification_payloads, "buyer");
        assert_eq!(
            reject_buyer["payload"]["notification_code"].as_str(),
            Some("acceptance.rejected")
        );
        assert_eq!(
            reject_buyer["payload"]["template_code"].as_str(),
            Some("NOTIFY_ACCEPTANCE_REJECTED_V1")
        );
        assert_eq!(
            reject_buyer["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/support/cases/new?order_id={}", report_seed.order_id).as_str())
        );
        let reject_ops = find_notification_payload(&reject_notification_payloads, "ops");
        assert_eq!(
            reject_ops["payload"]["variables"]["show_ops_context"].as_bool(),
            Some(true)
        );
        assert_eq!(
            reject_ops["payload"]["variables"]["action_href"].as_str(),
            Some(format!("/support/cases/new?order_id={}", report_seed.order_id).as_str())
        );

        cleanup_seed(&client, &file_seed).await;
        cleanup_seed(&client, &report_seed).await;
    }

    fn find_notification_payload<'a>(payloads: &'a [Value], audience_scope: &str) -> &'a Value {
        payloads
            .iter()
            .find(|payload| payload["payload"]["audience_scope"].as_str() == Some(audience_scope))
            .unwrap_or_else(|| panic!("missing payload for audience {audience_scope}"))
    }

    async fn seed_order(
        client: &Client,
        suffix: &str,
        sku_type: &str,
        status: &str,
        delivery_type: &str,
    ) -> Result<SeedOrder, db::Error> {
        let lower = sku_type.to_ascii_lowercase();
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv018-{lower}-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv018-{lower}-seller-{suffix}")],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'analysis', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("dlv018-{lower}-asset-{suffix}"),
                    &format!("dlv018 {lower} asset {suffix}"),
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
                   1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
                 )
                 RETURNING asset_version_id::text",
                &[&asset_id],
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'analysis', 'data_product',
                   $5, 'listed', 'one_time', 66.60, 'CNY', $6,
                   ARRAY['internal_use']::text[], $7,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv018-{lower}-product-{suffix}"),
                    &format!("dlv018 {lower} product {suffix}"),
                    &delivery_type,
                    &format!("dlv018 {lower} search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let acceptance_mode = if sku_type == "RPT_STD" {
            "manual_accept"
        } else {
            "manual_accept"
        };
        let refund_mode = "manual_refund";
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, '次', 'one_time', $4, $5, 'active'
                 )
                 RETURNING sku_id::text",
                &[
                    &product_id,
                    &format!("DLV018-{}-{suffix}", sku_type),
                    &sku_type,
                    &acceptance_mode,
                    &refund_mode,
                ],
            )
            .await?
            .get(0);
        let (delivery_status, acceptance_status, settlement_status, dispute_status) =
            if status == "report_delivered" {
                ("delivered", "in_progress", "pending_settlement", "none")
            } else {
                ("delivered", "in_progress", "pending_settlement", "none")
            };
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code,
                   delivery_status, acceptance_status, settlement_status, dispute_status,
                   price_snapshot_json, trust_boundary_snapshot, delivery_route_snapshot
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   $6, 'paid', 'online', 66.60, 'CNY',
                   $7, $8, $9, $10,
                   '{}'::jsonb,
                   jsonb_build_object('delivery_mode', $11),
                   'result_package'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &status,
                    &delivery_status,
                    &acceptance_status,
                    &settlement_status,
                    &dispute_status,
                    &delivery_type,
                ],
            )
            .await?
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot,
                   sensitive_delivery_mode, disclosure_review_status, committed_at
                 ) VALUES (
                   $1::text::uuid, $2, 'result_package', 'committed',
                   jsonb_build_object('delivery_mode', $2),
                   'standard', 'not_required', now()
                 )
                 RETURNING delivery_id::text",
                &[&order_id, &delivery_type],
            )
            .await?
            .get(0);

        Ok(SeedOrder {
            buyer_org_id,
            seller_org_id,
            asset_id,
            asset_version_id,
            product_id,
            sku_id,
            order_id,
            delivery_id,
        })
    }

    async fn cleanup_seed(client: &Client, seed: &SeedOrder) {
        let _ = client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
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
