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
        policy_id: String,
        order_id: String,
    }

    #[tokio::test]
    async fn trade025_authorization_min_structure_db_smoke() {
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
        let request_id = format!("req-trade025-{suffix}");

        let app = router();
        let transition_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/orders/{}/authorization/transition",
                        seed.order_id
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", &request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"action":"grant"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("transition response");
        let transition_status = transition_response.status();
        let transition_body = to_bytes(transition_response.into_body(), usize::MAX)
            .await
            .expect("transition body");
        assert_eq!(
            transition_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&transition_body)
        );
        let transition_json: Value = serde_json::from_slice(&transition_body).expect("json");
        assert_eq!(
            transition_json["data"]["data"]["authorization_model"]["scope"]["order_id"].as_str(),
            Some(seed.order_id.as_str())
        );
        assert_eq!(
            transition_json["data"]["data"]["authorization_model"]["resource"]["product_id"]
                .as_str(),
            Some(seed.product_id.as_str())
        );
        assert_eq!(
            transition_json["data"]["data"]["authorization_model"]["action"]["allowed_usage"][0]
                .as_str(),
            Some("internal_use")
        );

        let detail_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/orders/{}", seed.order_id))
                    .header("x-role", "buyer_operator")
                    .header("x-tenant-id", &seed.buyer_org_id)
                    .header("x-request-id", format!("req-trade025-detail-{suffix}"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("detail response");
        assert_eq!(detail_response.status(), StatusCode::OK);
        let detail_body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .expect("detail body");
        let detail_json: Value = serde_json::from_slice(&detail_body).expect("detail json");
        assert_eq!(
            detail_json["data"]["data"]["relations"]["authorizations"][0]["authorization_model"]
                ["subject"]["subject_id"]
                .as_str(),
            Some(seed.buyer_org_id.as_str())
        );

        cleanup_seed_graph(&client, &seed).await;
    }

    async fn seed_graph(client: &Client, suffix: &str) -> Result<SeedGraph, tokio_postgres::Error> {
        let buyer_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1::text, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade025-buyer-{suffix}")],
            )
            .await?
            .get(0);
        let seller_org_id: String = client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1::text, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("trade025-seller-{suffix}")],
            )
            .await?
            .get(0);
        let asset_id: String = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'finance', 'internal', 'draft', $3
                 ) RETURNING asset_id::text",
                &[
                    &seller_org_id,
                    &format!("trade025-asset-{suffix}"),
                    &format!("trade025 asset {suffix}"),
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
                 ) RETURNING asset_version_id::text",
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
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'finance', 'data_product',
                   $5, 'listed', 'one_time', 188.00, 'CNY', 'share_grant',
                   ARRAY['internal_use']::text[], $6,
                   '{"review_status":"approved","tax":{"policy":"platform_default","code":"VAT","inclusive":false}}'::jsonb
                 ) RETURNING product_id::text"#,
                &[
                    &asset_id,
                    &asset_version_id,
                    &seller_org_id,
                    &format!("trade025-product-{suffix}"),
                    &format!("trade025 product {suffix}"),
                    &format!("trade025 search {suffix}"),
                ],
            )
            .await?
            .get(0);
        let sku_id: String = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
                 ) VALUES (
                   $1::text::uuid, $2, 'SHARE_RO', '次', 'one_time', 'manual_accept', 'manual_refund', 'active'
                 ) RETURNING sku_id::text",
                &[&product_id, &format!("TRADE025-SKU-{suffix}")],
            )
            .await?
            .get(0);
        let policy_id: String = client
            .query_one(
                r#"INSERT INTO contract.usage_policy (
                   owner_org_id, policy_name, stage_from,
                   subject_constraints, usage_constraints, time_constraints,
                   region_constraints, output_constraints, exportable, status
                 ) VALUES (
                   $1::text::uuid, $2, 'V1',
                   '{"principal_type":"org"}'::jsonb,
                   '{"allowed_usage":["internal_use"]}'::jsonb,
                   '{"ttl_days":30}'::jsonb,
                   '{"allow_regions":["CN"]}'::jsonb,
                   '{"allow_export":false}'::jsonb,
                   false,
                   'active'
                 ) RETURNING policy_id::text"#,
                &[&seller_org_id, &format!("TRADE025-POL-{suffix}")],
            )
            .await?
            .get(0);
        let _: String = client
            .query_one(
                "INSERT INTO contract.policy_binding (policy_id, product_id, sku_id, binding_scope)
                 VALUES ($1::text::uuid, $2::text::uuid, $3::text::uuid, 'sku')
                 RETURNING policy_binding_id::text",
                &[&policy_id, &product_id, &sku_id],
            )
            .await?
            .get(0);
        let order_id: String = client
            .query_one(
                "INSERT INTO trade.order_main (
                   product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
                   status, payment_status, delivery_status, acceptance_status, settlement_status, dispute_status,
                   payment_mode, amount, currency_code, price_snapshot_json
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
                   'shared_active', 'paid', 'delivered', 'accepted', 'pending_settlement', 'none',
                   'online', 188.00, 'CNY', '{}'::jsonb
                 ) RETURNING order_id::text",
                &[&product_id, &asset_version_id, &buyer_org_id, &seller_org_id, &sku_id],
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
            policy_id,
            order_id,
        })
    }

    async fn cleanup_seed_graph(client: &Client, seed: &SeedGraph) {
        let _ = client
            .execute(
                "DELETE FROM trade.authorization_grant WHERE order_id = $1::text::uuid",
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
                "DELETE FROM contract.policy_binding WHERE policy_id = $1::text::uuid",
                &[&seed.policy_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.usage_policy WHERE policy_id = $1::text::uuid",
                &[&seed.policy_id],
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
