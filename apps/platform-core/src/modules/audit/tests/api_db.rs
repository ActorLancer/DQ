use crate::modules::audit::api::router;
use crate::modules::order::repo::write_trade_audit_event;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::Value;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("AUD_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedGraph {
    buyer_org_id: String,
    seller_org_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
    sku_id: String,
    order_id: String,
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[tokio::test]
    async fn rejects_audit_trace_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/traces")
            .header("x-request-id", "req-aud003-route-forbidden")
            .header("x-role", "buyer_operator")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn audit_trace_requires_request_id() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/audit/traces")
            .header("x-role", "platform_audit_security")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn audit_trace_api_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let dsn = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string());
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let suffix = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
    );
    let seed = seed_order_graph(&client, &suffix)
        .await
        .expect("seed order graph");
    let app = crate::with_live_test_state(router()).await;
    let order_request_id = format!("req-aud003-order-{suffix}");
    let trace_request_id = format!("req-aud003-traces-{suffix}");
    let tenant_request_id = format!("req-aud003-tenant-{suffix}");
    let trace_id = format!("trace-aud003-{suffix}");

    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "buyer_operator",
        "trade.order.create",
        "accepted",
        Some(&order_request_id),
        Some(&trace_id),
    )
    .await
    .expect("write order create audit");
    write_trade_audit_event(
        &client,
        "order",
        &seed.order_id,
        "buyer_operator",
        "trade.order.lock",
        "accepted",
        Some(&order_request_id),
        Some(&trace_id),
    )
    .await
    .expect("write order lock audit");

    let order_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/orders/{}", seed.order_id))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &order_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("order request"),
        )
        .await
        .expect("call order audit");
    assert_eq!(order_resp.status(), StatusCode::OK);
    let order_body = to_bytes(order_resp.into_body(), usize::MAX)
        .await
        .expect("read order body");
    let order_json: Value = serde_json::from_slice(&order_body).expect("decode order body");
    assert_eq!(
        order_json["data"]["order_id"].as_str(),
        Some(seed.order_id.as_str())
    );
    assert_eq!(
        order_json["data"]["buyer_org_id"].as_str(),
        Some(seed.buyer_org_id.as_str())
    );
    assert_eq!(
        order_json["data"]["seller_org_id"].as_str(),
        Some(seed.seller_org_id.as_str())
    );
    assert_eq!(order_json["data"]["total"].as_i64(), Some(2));
    assert_eq!(
        order_json["data"]["traces"][0]["trace_id"].as_str(),
        Some(trace_id.as_str())
    );

    let traces_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/audit/traces?order_id={}&trace_id={}",
                    seed.order_id, trace_id
                ))
                .header("x-role", "platform_audit_security")
                .header("x-request-id", &trace_request_id)
                .header("x-trace-id", &trace_id)
                .body(Body::empty())
                .expect("trace request"),
        )
        .await
        .expect("call trace audit");
    assert_eq!(traces_resp.status(), StatusCode::OK);
    let traces_body = to_bytes(traces_resp.into_body(), usize::MAX)
        .await
        .expect("read traces body");
    let traces_json: Value = serde_json::from_slice(&traces_body).expect("decode traces body");
    assert_eq!(traces_json["data"]["total"].as_i64(), Some(2));
    assert_eq!(
        traces_json["data"]["items"][0]["ref_type"].as_str(),
        Some("order")
    );

    let tenant_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/traces?order_id={}", seed.order_id))
                .header("x-role", "tenant_audit_readonly")
                .header("x-tenant-id", &seed.buyer_org_id)
                .header("x-request-id", &tenant_request_id)
                .body(Body::empty())
                .expect("tenant trace request"),
        )
        .await
        .expect("call tenant trace audit");
    assert_eq!(tenant_resp.status(), StatusCode::OK);

    let foreign_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/audit/orders/{}", seed.order_id))
                .header("x-role", "tenant_audit_readonly")
                .header("x-tenant-id", "00000000-0000-0000-0000-000000000999")
                .header("x-request-id", format!("req-aud003-foreign-{suffix}"))
                .body(Body::empty())
                .expect("foreign order request"),
        )
        .await
        .expect("call foreign order audit");
    assert_eq!(foreign_resp.status(), StatusCode::FORBIDDEN);

    let access_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.access_audit
             WHERE request_id = ANY($1::text[])",
            &[&vec![
                order_request_id.clone(),
                trace_request_id.clone(),
                tenant_request_id.clone(),
            ]],
        )
        .await
        .expect("count access audit")
        .get(0);
    assert_eq!(access_count, 3);

    let log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = ANY($1::text[])
               AND message_text LIKE 'audit lookup executed:%'",
            &[&vec![
                order_request_id.clone(),
                trace_request_id.clone(),
                tenant_request_id.clone(),
            ]],
        )
        .await
        .expect("count system logs")
        .get(0);
    assert_eq!(log_count, 3);

    cleanup_business_rows(&client, &seed).await;
}

async fn seed_order_graph(client: &Client, suffix: &str) -> Result<SeedGraph, Error> {
    let buyer_org_id: String = client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&format!("aud003-buyer-{suffix}")],
        )
        .await?
        .get(0);
    let seller_org_id: String = client
        .query_one(
            "INSERT INTO core.organization (org_name, org_type, status, metadata)
             VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
             RETURNING org_id::text",
            &[&format!("aud003-seller-{suffix}")],
        )
        .await?
        .get(0);
    let asset_id: String = client
        .query_one(
            r#"INSERT INTO catalog.data_asset (
                 owner_org_id, title, category, sensitivity_level, status, description
               ) VALUES (
                 $1::text::uuid, $2, 'manufacturing', 'low', 'active', $3
               )
               RETURNING asset_id::text"#,
            &[
                &seller_org_id,
                &format!("aud003-asset-{suffix}"),
                &format!("audit trace asset {suffix}"),
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
                 1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
               )
               RETURNING asset_version_id::text"#,
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
                 $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing',
                 'data_product', $5, 'listed', 'one_time', 88.00, 'CNY', 'file_download',
                 ARRAY['internal_use']::text[], $6, '{"review_status":"approved"}'::jsonb
               )
               RETURNING product_id::text"#,
            &[
                &asset_id,
                &asset_version_id,
                &seller_org_id,
                &format!("aud003-product-{suffix}"),
                &format!("audit trace product {suffix}"),
                &format!("audit trace search {suffix}"),
            ],
        )
        .await?
        .get(0);
    let sku_id: String = client
        .query_one(
            "INSERT INTO catalog.product_sku (
               product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
               delivery_object_kind, acceptance_mode, refund_mode, status
             ) VALUES (
               $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'file_download',
               'download_file', 'manual_accept', 'manual_refund', 'active'
             )
             RETURNING sku_id::text",
            &[&product_id, &format!("AUD003-FILE-STD-{suffix}")],
        )
        .await?
        .get(0);
    let order_id: String = client
        .query_one(
            "INSERT INTO trade.order_main (
               product_id, asset_version_id, buyer_org_id, seller_org_id, sku_id,
               status, payment_status, delivery_status, acceptance_status,
               settlement_status, dispute_status, payment_mode, amount, currency_code,
               price_snapshot_json
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::text::uuid, $5::text::uuid,
               'buyer_locked', 'paid', 'pending_delivery', 'not_started',
               'pending_settlement', 'none', 'online', 88.00, 'CNY',
               jsonb_build_object('sku_type', 'FILE_STD', 'audit_seed', $6)
             )
             RETURNING order_id::text",
            &[
                &product_id,
                &asset_version_id,
                &buyer_org_id,
                &seller_org_id,
                &sku_id,
                &suffix,
            ],
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
    })
}

async fn cleanup_business_rows(client: &Client, seed: &SeedGraph) {
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
            "DELETE FROM core.organization WHERE org_id IN ($1::text::uuid, $2::text::uuid)",
            &[&seed.buyer_org_id, &seed.seller_org_id],
        )
        .await;
}
