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
    struct FileSeed {
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

    #[derive(Debug)]
    struct ReportSeed {
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
    async fn dlv019_watermark_policy_db_smoke() {
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
        let file_seed = seed_file_order(&client, &format!("{suffix}-file"))
            .await
            .expect("seed file order");
        let report_seed = seed_report_order(&client, &format!("{suffix}-report"))
            .await
            .expect("seed report order");
        let app = crate::with_live_test_state(delivery_router().merge(order_router())).await;

        let file_request_id = format!("req-dlv019-file-{suffix}");
        let file_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", file_seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &file_seed.seller_org_id)
                    .header("x-request-id", &file_request_id)
                    .body(Body::from(
                        json!({
                            "branch": "file",
                            "object_uri": format!("s3://delivery-objects/orders/{suffix}/payload.enc"),
                            "content_type": "application/octet-stream",
                            "size_bytes": 1024,
                            "content_hash": format!("sha256:file:{suffix}"),
                            "encryption_algo": "AES-GCM",
                            "key_cipher": format!("cipher-{suffix}"),
                            "key_control_mode": "seller_managed",
                            "unwrap_policy_json": {
                                "kms": "local-mock",
                                "buyer_org_id": file_seed.buyer_org_id,
                            },
                            "key_version": "v1",
                            "expire_at": "2026-05-20T10:00:00Z",
                            "download_limit": 3,
                            "delivery_commit_hash": format!("commit-{suffix}"),
                            "receipt_hash": format!("receipt-{suffix}")
                        })
                        .to_string(),
                    ))
                    .expect("file request build"),
            )
            .await
            .expect("file deliver response");
        let file_status = file_response.status();
        let file_body = to_bytes(file_response.into_body(), usize::MAX)
            .await
            .expect("file deliver body");
        assert_eq!(
            file_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&file_body)
        );

        let report_request_id = format!("req-dlv019-report-{suffix}");
        let report_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/orders/{}/deliver", report_seed.order_id))
                    .header("content-type", "application/json")
                    .header("x-role", "seller_operator")
                    .header("x-tenant-id", &report_seed.seller_org_id)
                    .header("x-request-id", &report_request_id)
                    .body(Body::from(
                        json!({
                            "branch": "report",
                            "object_uri": format!("s3://report-results/orders/{suffix}/monthly-report.pdf"),
                            "content_type": "application/pdf",
                            "size_bytes": 2048,
                            "content_hash": format!("sha256:report:{suffix}"),
                            "report_type": "pdf_report",
                            "storage_namespace_id": report_seed.storage_namespace_id,
                            "delivery_commit_hash": format!("report-commit-{suffix}"),
                            "receipt_hash": format!("report-receipt-{suffix}"),
                            "metadata": {
                                "title": format!("Monthly report {suffix}"),
                                "template_code": "DELIVERY_REPORT_V1"
                            }
                        })
                        .to_string(),
                    ))
                    .expect("report request build"),
            )
            .await
            .expect("report deliver response");
        let report_status = report_response.status();
        let report_body = to_bytes(report_response.into_body(), usize::MAX)
            .await
            .expect("report deliver body");
        assert_eq!(
            report_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&report_body)
        );

        let file_detail_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", file_seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &file_seed.buyer_org_id)
                    .header("x-request-id", format!("{file_request_id}-detail"))
                    .body(Body::empty())
                    .expect("file detail request build"),
            )
            .await
            .expect("file detail response");
        let file_detail_status = file_detail_response.status();
        let file_detail_body = to_bytes(file_detail_response.into_body(), usize::MAX)
            .await
            .expect("file detail body");
        assert_eq!(
            file_detail_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&file_detail_body)
        );
        let file_detail_json: Value =
            serde_json::from_slice(&file_detail_body).expect("file detail json");
        assert_eq!(
            file_detail_json["data"]["relations"]["deliveries"][0]["storage_gateway"]
                ["watermark_policy"]["rule"]["delivery_branch"]
                .as_str(),
            Some("file")
        );
        assert_eq!(
            file_detail_json["data"]["relations"]["deliveries"][0]["storage_gateway"]
                ["watermark_policy"]["rule"]["pipeline"]["stage"]
                .as_str(),
            Some("post_delivery_commit")
        );

        let report_detail_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", report_seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &report_seed.buyer_org_id)
                    .header("x-request-id", format!("{report_request_id}-detail"))
                    .body(Body::empty())
                    .expect("report detail request build"),
            )
            .await
            .expect("report detail response");
        let report_detail_status = report_detail_response.status();
        let report_detail_body = to_bytes(report_detail_response.into_body(), usize::MAX)
            .await
            .expect("report detail body");
        assert_eq!(
            report_detail_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&report_detail_body)
        );
        let report_detail_json: Value =
            serde_json::from_slice(&report_detail_body).expect("report detail json");
        assert_eq!(
            report_detail_json["data"]["relations"]["deliveries"][0]["storage_gateway"]
                ["watermark_policy"]["rule"]["delivery_branch"]
                .as_str(),
            Some("report")
        );
        assert_eq!(
            report_detail_json["data"]["relations"]["deliveries"][0]["storage_gateway"]
                ["watermark_policy"]["rule"]["policy"]
                .as_str(),
            Some("result_attested")
        );

        let file_row = client
            .query_one(
                "SELECT trust_boundary_snapshot ->> 'watermark_mode',
                        trust_boundary_snapshot ->> 'watermark_rule',
                        trust_boundary_snapshot -> 'watermark_policy' ->> 'delivery_branch',
                        trust_boundary_snapshot -> 'watermark_policy' -> 'pipeline' ->> 'status',
                        jsonb_array_length(COALESCE(trust_boundary_snapshot -> 'fingerprint_fields', '[]'::jsonb))
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&file_seed.delivery_id],
            )
            .await
            .expect("query file delivery row");
        assert_eq!(
            file_row.get::<_, Option<String>>(0).as_deref(),
            Some("rule_bound")
        );
        assert_eq!(
            file_row.get::<_, Option<String>>(1).as_deref(),
            Some("buyer_bound")
        );
        assert_eq!(
            file_row.get::<_, Option<String>>(2).as_deref(),
            Some("file")
        );
        assert_eq!(
            file_row.get::<_, Option<String>>(3).as_deref(),
            Some("reserved")
        );
        assert_eq!(file_row.get::<_, i32>(4), 2);

        let report_row = client
            .query_one(
                "SELECT trust_boundary_snapshot ->> 'watermark_mode',
                        trust_boundary_snapshot ->> 'watermark_rule',
                        trust_boundary_snapshot -> 'watermark_policy' ->> 'delivery_branch',
                        trust_boundary_snapshot -> 'watermark_policy' -> 'pipeline' ->> 'status',
                        jsonb_array_length(COALESCE(trust_boundary_snapshot -> 'fingerprint_fields', '[]'::jsonb))
                 FROM delivery.delivery_record
                 WHERE delivery_id = $1::text::uuid",
                &[&report_seed.delivery_id],
            )
            .await
            .expect("query report delivery row");
        assert_eq!(
            report_row.get::<_, Option<String>>(0).as_deref(),
            Some("rule_bound")
        );
        assert_eq!(
            report_row.get::<_, Option<String>>(1).as_deref(),
            Some("result_attested")
        );
        assert_eq!(
            report_row.get::<_, Option<String>>(2).as_deref(),
            Some("report")
        );
        assert_eq!(
            report_row.get::<_, Option<String>>(3).as_deref(),
            Some("reserved")
        );
        assert_eq!(report_row.get::<_, i32>(4), 0);

        cleanup_file_seed(&client, &file_seed).await;
        cleanup_report_seed(&client, &report_seed).await;
    }

    async fn seed_file_order(client: &Client, suffix: &str) -> Result<FileSeed, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv019-file-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv019-file-seller-{suffix}")],
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
                    &format!("dlv019-file-asset-{suffix}"),
                    &format!("dlv019 file asset {suffix}"),
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
                &[&asset_id, &format!("wmk-file-{suffix}")],
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
                    &format!("dlv019-file-product-{suffix}"),
                    &format!("dlv019 file product {suffix}"),
                    &format!("dlv019 file search {suffix}"),
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
                &[&product_id, &format!("DLV019-FILE-SKU-{suffix}")],
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
                    &format!("DLV019-FILE-SKU-{suffix}"),
                    &format!("wmk-file-{suffix}"),
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
                &[&order_id, &format!("sha256:dlv019-file:{suffix}")],
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
                &[&seller_org_id, &format!("dlv019-file-ns-{suffix}")],
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
                &[&order_id, &format!("wmk-file-{suffix}")],
            )
            .await?
            .get(0);

        Ok(FileSeed {
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

    async fn seed_report_order(client: &Client, suffix: &str) -> Result<ReportSeed, db::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv019-report-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("dlv019-report-seller-{suffix}")],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'analytics', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("dlv019-report-asset-{suffix}"),
                    &format!("dlv019 report asset {suffix}"),
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
                   jsonb_build_object('delivery_mode', 'report_delivery', 'watermark_rule', 'result_attested'),
                   'active'
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'analytics', 'service_product',
                   $5, 'listed', 'one_time', 88.00, 'CNY', 'report_delivery',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved"}'::jsonb
                 )
                 RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("dlv019-report-product-{suffix}"),
                    &format!("dlv019 report product {suffix}"),
                    &format!("dlv019 report search {suffix}"),
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
                &[&product_id, &format!("DLV019-RPT-SKU-{suffix}")],
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
                    &format!("DLV019-RPT-SKU-{suffix}"),
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
                &[&order_id, &format!("sha256:dlv019-report:{suffix}")],
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
                &[&seller_org_id, &format!("dlv019-report-ns-{suffix}")],
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

        Ok(ReportSeed {
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

    async fn cleanup_file_seed(client: &Client, seed: &FileSeed) {
        client
            .execute(
                "DELETE FROM delivery.delivery_receipt WHERE delivery_id = $1::text::uuid",
                &[&seed.delivery_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.delivery_ticket WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.key_envelope WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.storage_object WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
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
                "UPDATE trade.order_main SET contract_id = NULL WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM contract.digital_contract WHERE contract_id = $1::text::uuid",
                &[&seed.contract_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await
            .ok();
    }

    async fn cleanup_report_seed(client: &Client, seed: &ReportSeed) {
        client
            .execute(
                "DELETE FROM delivery.report_artifact WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.delivery_record WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM delivery.storage_object WHERE storage_namespace_id = $1::text::uuid",
                &[&seed.storage_namespace_id],
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
                "UPDATE trade.order_main SET contract_id = NULL WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM contract.digital_contract WHERE contract_id = $1::text::uuid",
                &[&seed.contract_id],
            )
            .await
            .ok();
        client
            .execute(
                "DELETE FROM trade.order_main WHERE order_id = $1::text::uuid",
                &[&seed.order_id],
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
                "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
                &[&seed.buyer_org_id, &seed.seller_org_id],
            )
            .await
            .ok();
    }
}
