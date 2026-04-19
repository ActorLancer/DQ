#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tokio_postgres::{Client, NoTls};
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
        contract_id: String,
        authorization_ids: Vec<String>,
        delivery_ids: Vec<String>,
        billing_event_ids: Vec<String>,
        settlement_id: String,
        refund_id: String,
        compensation_id: String,
        invoice_request_id: String,
        dispute_case_ids: Vec<String>,
    }

    #[tokio::test]
    async fn trade022_order_relations_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
            .await
            .expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_millis()
        );
        let seed = seed_graph(&client, &suffix).await.expect("seed graph");
        let request_id = format!("req-trade022-{suffix}");

        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");

        assert_eq!(
            json["data"]["data"]["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["contract"]["contract_id"].as_str(),
            Some(seed.contract_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["contract"]["contract_status"].as_str(),
            Some("signed")
        );
        assert_eq!(
            json["data"]["data"]["relations"]["authorizations"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["authorizations"][0]["authorization_id"].as_str(),
            Some(seed.authorization_ids[1].as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["authorizations"][0]["authorization_model"]["scope"]
                ["order_id"]
                .as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["authorizations"][0]["authorization_model"]["resource"]["sku_id"].as_str(),
            Some(seed.sku_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["deliveries"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["deliveries"][0]["delivery_id"].as_str(),
            Some(seed.delivery_ids[1].as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["billing_events"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["billing_events"][0]["billing_event_id"]
                .as_str(),
            Some(seed.billing_event_ids[1].as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["settlements"][0]["settlement_id"]
                .as_str(),
            Some(seed.settlement_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["refunds"][0]["refund_id"].as_str(),
            Some(seed.refund_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["compensations"][0]["compensation_id"]
                .as_str(),
            Some(seed.compensation_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["billing"]["invoices"][0]["invoice_request_id"]
                .as_str(),
            Some(seed.invoice_request_id.as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"][0]["case_id"].as_str(),
            Some(seed.dispute_case_ids[1].as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"][0]["evidence_count"].as_i64(),
            Some(0)
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"][1]["case_id"].as_str(),
            Some(seed.dispute_case_ids[0].as_str())
        );
        assert_eq!(
            json["data"]["data"]["relations"]["disputes"][1]["evidence_count"].as_i64(),
            Some(1)
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.read'",
                &[&request_id],
            )
            .await
            .expect("query audit")
            .get(0);
        assert!(audit_count >= 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade022-buyer-{suffix}")],
            )
            .await?
            .get(0);

        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade022-seller-{suffix}")],
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
                    &format!("trade022-asset-{suffix}"),
                    &format!("trade022 asset {suffix}"),
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
                   2048, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
                   $5, 'listed', 'one_time', 128.00, 'CNY', 'file_download',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade022-product-{suffix}"),
                    &format!("trade022 product {suffix}"),
                    &format!("trade022 search {suffix}"),
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
                &[&product_id, &format!("TRADE022-SKU-{suffix}")],
            )
            .await?
            .get(0);

        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'accepted', 'paid', 'online', 128.00, 'CNY',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'FILE_STD',
                     'pricing_mode', 'one_time',
                     'unit_price', '128.00',
                     'currency_code', 'CNY',
                     'billing_mode', 'one_time',
                     'refund_mode', 'manual_refund',
                     'settlement_terms', jsonb_build_object('settlement_basis', 'one_time_final', 'settlement_mode', 'manual_v1'),
                     'tax_terms', jsonb_build_object('tax_policy', 'platform_default', 'tax_code', 'VAT', 'tax_inclusive', false),
                     'captured_at', '1776510000000',
                     'source', 'seed'
                   )::jsonb
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("TRADE022-SKU-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let contract_id: String = client
            .query_one(
                "INSERT INTO contract.digital_contract (
                   order_id, contract_digest, data_contract_digest, status, signed_at, variables_json
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'signed', NOW() - INTERVAL '2 days',
                   jsonb_build_object('region', 'CN', 'plan', 'detail-read')
                 )
                 RETURNING contract_id::text",
                &[
                    &order_id,
                    &format!("sha256:trade022-contract:{suffix}"),
                    &format!("sha256:trade022-data-contract:{suffix}"),
                ],
            )
            .await?
            .get(0);
        client
            .execute(
                "UPDATE trade.order_main
                 SET contract_id = $2::text::uuid
                 WHERE order_id = $1::text::uuid",
                &[&order_id, &contract_id],
            )
            .await?;

        let auth_old_id: String = client
            .query_one(
                "INSERT INTO trade.authorization_grant (
                   order_id, grant_type, granted_to_type, granted_to_id, policy_snapshot, valid_from, status
                 ) VALUES (
                   $1::text::uuid, 'download', 'organization', $2::text::uuid,
                   jsonb_build_object('scope', 'download_once', 'version', 1),
                   NOW() - INTERVAL '3 days', 'expired'
                 )
                 RETURNING authorization_grant_id::text",
                &[&order_id, &buyer_org_id],
            )
            .await?
            .get(0);

        let auth_new_id: String = client
            .query_one(
                "INSERT INTO trade.authorization_grant (
                   order_id, grant_type, granted_to_type, granted_to_id, policy_snapshot, valid_from, status
                 ) VALUES (
                   $1::text::uuid, 'download', 'organization', $2::text::uuid,
                   jsonb_build_object('scope', 'download_once', 'version', 2),
                   NOW() - INTERVAL '1 day', 'active'
                 )
                 RETURNING authorization_grant_id::text",
                &[&order_id, &buyer_org_id],
            )
            .await?
            .get(0);

        let delivery_old_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash, receipt_hash,
                   committed_at, expires_at
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'prepared', $2, $3,
                   NOW() - INTERVAL '2 days', NOW() + INTERVAL '5 days'
                 )
                 RETURNING delivery_id::text",
                &[
                    &order_id,
                    &format!("commit-old-{suffix}"),
                    &format!("receipt-old-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let delivery_new_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, delivery_commit_hash, receipt_hash,
                   committed_at, expires_at
                 ) VALUES (
                   $1::text::uuid, 'file_download', 'signed_url', 'committed', $2, $3,
                   NOW() - INTERVAL '1 day', NOW() + INTERVAL '6 days'
                 )
                 RETURNING delivery_id::text",
                &[
                    &order_id,
                    &format!("commit-new-{suffix}"),
                    &format!("receipt-new-{suffix}"),
                ],
            )
            .await?
            .get(0);

        let billing_event_old_id: String = client
            .query_one(
                "INSERT INTO billing.billing_event (
                   order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
                 ) VALUES (
                   $1::text::uuid, 'order_created', 'trade.order', 128.00, 'CNY', 1,
                   NOW() - INTERVAL '2 days', jsonb_build_object('phase', 'create')
                 )
                 RETURNING billing_event_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        let billing_event_new_id: String = client
            .query_one(
                "INSERT INTO billing.billing_event (
                   order_id, event_type, event_source, amount, currency_code, units, occurred_at, metadata
                 ) VALUES (
                   $1::text::uuid, 'accepted_billable', 'trade.acceptance', 128.00, 'CNY', 1,
                   NOW() - INTERVAL '1 day', jsonb_build_object('phase', 'acceptance')
                 )
                 RETURNING billing_event_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        let settlement_id: String = client
            .query_one(
                "INSERT INTO billing.settlement_record (
                   order_id, settlement_type, settlement_status, settlement_mode, payable_amount,
                   refund_amount, compensation_amount, reason_code, settled_at
                 ) VALUES (
                   $1::text::uuid, 'manual_settlement', 'settled', 'manual', 120.00,
                   8.00, 0.00, 'trade022_settled', NOW() - INTERVAL '12 hours'
                 )
                 RETURNING settlement_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        let refund_id: String = client
            .query_one(
                "INSERT INTO billing.refund_record (
                   order_id, amount, currency_code, status, executed_at
                 ) VALUES (
                   $1::text::uuid, 8.00, 'CNY', 'completed', NOW() - INTERVAL '10 hours'
                 )
                 RETURNING refund_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        let compensation_id: String = client
            .query_one(
                "INSERT INTO billing.compensation_record (
                   order_id, amount, currency_code, status, executed_at
                 ) VALUES (
                   $1::text::uuid, 2.00, 'CNY', 'completed', NOW() - INTERVAL '8 hours'
                 )
                 RETURNING compensation_id::text",
                &[&order_id],
            )
            .await?
            .get(0);

        let invoice_request_id: String = client
            .query_one(
                "INSERT INTO billing.invoice_request (
                   order_id, settlement_id, requester_org_id, invoice_title, amount, currency_code, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 120.00, 'CNY', 'issued'
                 )
                 RETURNING invoice_request_id::text",
                &[
                    &order_id,
                    &settlement_id,
                    &buyer_org_id,
                    &format!("trade022 invoice {suffix}"),
                ],
            )
            .await?
            .get(0);

        let dispute_open_id: String = client
            .query_one(
                "INSERT INTO support.dispute_case (
                   order_id, complainant_type, complainant_id, reason_code, status, opened_at
                 ) VALUES (
                   $1::text::uuid, 'buyer_org', $2::text::uuid, 'missing_field', 'opened',
                   NOW() - INTERVAL '6 hours'
                 )
                 RETURNING case_id::text",
                &[&order_id, &buyer_org_id],
            )
            .await?
            .get(0);
        client
            .execute(
                "INSERT INTO support.evidence_object (
                   case_id, object_type, object_uri, object_hash, metadata
                 ) VALUES (
                   $1::text::uuid, 'audit_export', $2, $3, '{}'::jsonb
                 )",
                &[
                    &dispute_open_id,
                    &format!("s3://evidence/trade022/{suffix}"),
                    &format!("sha256:evidence:{suffix}"),
                ],
            )
            .await?;

        let dispute_closed_id: String = client
            .query_one(
                "INSERT INTO support.dispute_case (
                   order_id, complainant_type, complainant_id, reason_code, status, decision_code,
                   penalty_code, opened_at, resolved_at
                 ) VALUES (
                   $1::text::uuid, 'seller_org', $2::text::uuid, 'delayed_delivery', 'closed',
                   'refund_partial', 'seller_warning',
                   NOW() - INTERVAL '4 hours', NOW() - INTERVAL '1 hour'
                 )
                 RETURNING case_id::text",
                &[&order_id, &seller_org_id],
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
            contract_id,
            authorization_ids: vec![auth_old_id, auth_new_id],
            delivery_ids: vec![delivery_old_id, delivery_new_id],
            billing_event_ids: vec![billing_event_old_id, billing_event_new_id],
            settlement_id,
            refund_id,
            compensation_id,
            invoice_request_id,
            dispute_case_ids: vec![dispute_open_id, dispute_closed_id],
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM billing.invoice_request WHERE invoice_request_id = $1::text::uuid",
                &[&seed.invoice_request_id],
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
                "DELETE FROM core.organization WHERE org_id = ANY($1::text[]::uuid[])",
                &[&vec![seed.buyer_org_id.clone(), seed.seller_org_id.clone()]],
            )
            .await;
    }
}
