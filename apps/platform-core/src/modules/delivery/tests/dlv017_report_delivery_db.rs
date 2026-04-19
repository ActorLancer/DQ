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
        contract_id: String,
    }

    #[tokio::test]
    async fn dlv017_report_delivery_db_smoke() {
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
        let request_id = format!("req-dlv017-{suffix}");
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
                            "branch": "report",
                            "object_uri": format!("s3://report-results/orders/{suffix}/monthly-report.pdf"),
                            "content_type": "application/pdf",
                            "size_bytes": 2048,
                            "content_hash": format!("sha256:report:{suffix}"),
                            "report_type": "pdf_report",
                            "storage_namespace_id": seed.storage_namespace_id,
                            "delivery_commit_hash": format!("report-commit-{suffix}"),
                            "receipt_hash": format!("report-receipt-{suffix}"),
                            "metadata": {
                                "title": format!("Monthly report {suffix}"),
                                "template_code": "DELIVERY_REPORT_V1"
                            }
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
        let data = &json["data"]["data"];
        assert_eq!(data["current_state"].as_str(), Some("report_delivered"));
        assert_eq!(data["delivery_status"].as_str(), Some("delivered"));
        assert_eq!(data["acceptance_status"].as_str(), Some("in_progress"));
        assert_eq!(data["bucket_name"].as_str(), Some("report-results"));
        assert_eq!(
            data["object_key"].as_str(),
            Some(format!("orders/{suffix}/monthly-report.pdf").as_str())
        );
        assert_eq!(data["report_type"].as_str(), Some("pdf_report"));
        assert_eq!(data["report_version_no"].as_i64(), Some(1));
        assert_eq!(
            data["report_hash"].as_str(),
            Some(format!("sha256:report:{suffix}").as_str())
        );
        let report_artifact_id = data["report_artifact_id"]
            .as_str()
            .expect("report_artifact_id")
            .to_string();

        let replay_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &seed.seller_org_id)
                    .header("x-request-id", format!("{request_id}-replay"))
                    .body(Body::from(
                        json!({
                            "branch": "report",
                            "object_uri": format!("s3://report-results/orders/{suffix}/monthly-report.pdf"),
                            "content_type": "application/pdf",
                            "size_bytes": 2048,
                            "content_hash": format!("sha256:report:{suffix}"),
                            "report_type": "pdf_report",
                            "storage_namespace_id": seed.storage_namespace_id,
                            "delivery_commit_hash": format!("report-commit-{suffix}"),
                            "receipt_hash": format!("report-receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("replay build"),
            )
            .await
            .expect("replay response");
        let replay_status = replay_response.status();
        let replay_body = to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .expect("replay body");
        assert_eq!(
            replay_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&replay_body)
        );
        let replay_json: Value = serde_json::from_slice(&replay_body).expect("replay json");
        assert_eq!(
            replay_json["data"]["data"]["operation"].as_str(),
            Some("already_committed")
        );
        assert_eq!(
            replay_json["data"]["data"]["report_artifact_id"].as_str(),
            Some(report_artifact_id.as_str())
        );

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
            detail_json["data"]["data"]["relations"]["deliveries"][0]["storage_gateway"]
                ["object_locator"]["bucket_name"]
                .as_str(),
            Some("report-results")
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
        assert_eq!(order_row.get::<_, String>(0), "report_delivered");
        assert_eq!(order_row.get::<_, String>(1), "delivered");
        assert_eq!(order_row.get::<_, String>(2), "in_progress");
        assert_eq!(order_row.get::<_, String>(3), "pending_settlement");

        let delivery_row = client
            .query_one(
                "SELECT status, object_id::text, delivery_commit_hash, receipt_hash
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await
            .expect("query delivery row");
        assert_eq!(delivery_row.get::<_, String>(0), "committed");
        let object_id = delivery_row
            .get::<_, Option<String>>(1)
            .expect("delivery object id");
        assert_eq!(
            delivery_row.get::<_, Option<String>>(2).as_deref(),
            Some(format!("report-commit-{suffix}").as_str())
        );
        assert_eq!(
            delivery_row.get::<_, Option<String>>(3).as_deref(),
            Some(format!("report-receipt-{suffix}").as_str())
        );

        let artifact_row = client
            .query_one(
                "SELECT object_id::text, report_type, version_no, status
                 FROM delivery.report_artifact
                 WHERE report_artifact_id = $1::text::uuid",
                &[&report_artifact_id],
            )
            .await
            .expect("query report artifact");
        assert_eq!(
            artifact_row.get::<_, Option<String>>(0).as_deref(),
            Some(object_id.as_str())
        );
        assert_eq!(artifact_row.get::<_, String>(1), "pdf_report");
        assert_eq!(artifact_row.get::<_, i32>(2), 1);
        assert_eq!(artifact_row.get::<_, String>(3), "delivered");

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE action_name = 'delivery.report.commit'
                   AND request_id = $1",
                &[&request_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert_eq!(audit_count, 1);

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv017-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv017-seller-{suffix}")],
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
                    &format!("dlv017-asset-{suffix}"),
                    &format!("dlv017 asset {suffix}"),
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
                   jsonb_build_object('report_delivery', 'result_package'), 'active'
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
                   $5, 'listed', 'one_time', 88.00, 'CNY', 'report_delivery',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv017-product-{suffix}"),
                    &format!("dlv017 product {suffix}"),
                    &format!("dlv017 search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'RPT_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 )
                 RETURNING sku_id::text",
                &[&product_id, &format!("DLV017-SKU-{suffix}")],
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
                   'report_generated', 'paid', 'online', 88.00, 'CNY',
                   'in_progress', 'not_started', 'pending_settlement', 'none',
                   jsonb_build_object(
                     'product_id', $1::text,
                     'sku_id', $5::text,
                     'sku_code', $6::text,
                     'sku_type', 'RPT_STD',
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
                   jsonb_build_object('delivery_mode', 'report_delivery', 'watermark_rule', 'result_attested'),
                   'result_package'
                 )
                 RETURNING order_id::text",
                &[
                    &product_id,
                    &asset_version_id,
                    &buyer_org_id,
                    &seller_org_id,
                    &sku_id,
                    &format!("DLV017-SKU-{suffix}"),
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
                &[&order_id, &format!("sha256:dlv017:{suffix}")],
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
                   $1::text::uuid, $2, 's3_compatible', 'product', 'report-results', 'orders/{order_id}', 'active'
                 )
                 RETURNING storage_namespace_id::text",
                &[&seller_org_id, &format!("dlv017-ns-{suffix}")],
            )
            .await?
            .get(0);
        let delivery_id: String = client
            .query_one(
                "INSERT INTO delivery.delivery_record (
                   order_id, delivery_type, delivery_route, status, trust_boundary_snapshot, sensitive_delivery_mode, disclosure_review_status
                 ) VALUES (
                   $1::text::uuid, 'report_delivery', 'result_package', 'prepared',
                   jsonb_build_object('delivery_mode', 'report_delivery', 'result_package', true),
                   'standard', 'not_required'
                 )
                 RETURNING delivery_id::text",
                &[&order_id],
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
            contract_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM delivery.report_artifact WHERE order_id = $1::text::uuid",
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
                "DELETE FROM delivery.storage_object WHERE org_id = $1::text::uuid",
                &[&seed.seller_org_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.storage_namespace WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.digital_contract WHERE contract_id = $1::text::uuid",
                &[&seed.contract_id],
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
