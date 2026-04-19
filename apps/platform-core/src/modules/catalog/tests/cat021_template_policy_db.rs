use crate::modules::catalog::router::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::{Value, json};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    asset_id: String,
    asset_version_id: String,
    product_id: String,
    sku_id: String,
    template_product_id: String,
    template_sku_id: String,
    policy_id: String,
}

async fn seed_catalog_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', '{}'::jsonb
             )
             RETURNING org_id::text",
            &[&format!("cat021-org-{suffix}")],
        )
        .await?;
    let org_id: String = org.get(0);

    let asset = client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status
             ) VALUES (
               $1::text::uuid, $2, 'manufacturing', 'internal', 'draft'
             )
             RETURNING asset_id::text",
            &[&org_id, &format!("cat021-asset-{suffix}")],
        )
        .await?;
    let asset_id: String = asset.get(0);

    let asset_version = client
        .query_one(
            "INSERT INTO catalog.asset_version (
               asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
               data_size_bytes, origin_region, allowed_region, requires_controlled_execution, trust_boundary_snapshot, status
             ) VALUES (
               $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
               1024, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
             )
             RETURNING asset_version_id::text",
            &[&asset_id],
        )
        .await?;
    let asset_version_id: String = asset_version.get(0);

    let product = client
        .query_one(
            "INSERT INTO catalog.product (
               asset_id, asset_version_id, seller_org_id, title, category, product_type,
               description, status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, 'manufacturing', 'data_product',
               'cat021 product', 'draft', 'one_time', 99, 'CNY', 'file_download', ARRAY['internal_use']::text[], 'cat021', '{}'::jsonb
             )
             RETURNING product_id::text",
            &[
                &asset_id,
                &asset_version_id,
                &org_id,
                &format!("cat021-product-{suffix}"),
            ],
        )
        .await?;
    let product_id: String = product.get(0);

    let sku = client
        .query_one(
            "INSERT INTO catalog.product_sku (
               product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode, refund_mode, status
             ) VALUES (
               $1::text::uuid, $2, 'FILE_STD', '份', 'one_time', 'manual_accept', 'manual_refund', 'draft'
             )
             RETURNING sku_id::text",
            &[&product_id, &format!("CAT021-SKU-{suffix}")],
        )
        .await?;
    let sku_id: String = sku.get(0);

    let template_product = client
        .query_one(
            "INSERT INTO contract.template_definition (
               template_type, template_name, applicable_sku_types, status
             ) VALUES (
               'contract', $1, ARRAY['FILE_STD']::text[], 'active'
             )
             RETURNING template_id::text",
            &[&format!("CONTRACT_FILE_STD_V1_{suffix}")],
        )
        .await?;
    let template_product_id: String = template_product.get(0);

    let template_sku = client
        .query_one(
            "INSERT INTO contract.template_definition (
               template_type, template_name, applicable_sku_types, status
             ) VALUES (
               'license', $1, ARRAY['FILE_STD']::text[], 'active'
             )
             RETURNING template_id::text",
            &[&format!("LICENSE_FILE_STD_V1_{suffix}")],
        )
        .await?;
    let template_sku_id: String = template_sku.get(0);

    let policy = client
        .query_one(
            "INSERT INTO contract.usage_policy (
               owner_org_id, policy_name, status
             ) VALUES (
               $1::text::uuid, $2, 'draft'
             )
             RETURNING policy_id::text",
            &[&org_id, &format!("cat021-policy-{suffix}")],
        )
        .await?;
    let policy_id: String = policy.get(0);

    Ok(SeedIds {
        org_id,
        asset_id,
        asset_version_id,
        product_id,
        sku_id,
        template_product_id,
        template_sku_id,
        policy_id,
    })
}

async fn cleanup_catalog_graph(client: &Client, ids: &SeedIds) {
    let _ = client
        .execute(
            "DELETE FROM contract.usage_policy WHERE policy_id = $1::text::uuid",
            &[&ids.policy_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM contract.template_definition WHERE template_id IN ($1::text::uuid, $2::text::uuid)",
            &[&ids.template_product_id, &ids.template_sku_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
            &[&ids.product_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
            &[&ids.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
            &[&ids.asset_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
            &[&ids.org_id],
        )
        .await;
}

#[tokio::test]
async fn cat021_template_bind_and_policy_patch_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
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
    let ids = seed_catalog_graph(&client, &suffix)
        .await
        .expect("seed catalog graph");
    let req_product_bind = format!("req-cat021-product-bind-{suffix}");
    let req_sku_bind = format!("req-cat021-sku-bind-{suffix}");
    let req_policy_patch = format!("req-cat021-policy-patch-{suffix}");

    let outcome: Result<(), String> = async {
        let app = crate::with_live_test_state(router()).await;

        let product_bind_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/products/{}/bind-template", ids.product_id))
                    .header("content-type", "application/json")
                    .header("x-role", "platform_admin")
                    .header("x-request-id", &req_product_bind)
                    .body(Body::from(
                        json!({ "template_id": ids.template_product_id }).to_string(),
                    ))
                    .map_err(|err| format!("build product bind request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call product bind endpoint: {err}"))?;
        if product_bind_resp.status() != StatusCode::OK {
            let status = product_bind_resp.status();
            let body = to_bytes(product_bind_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read product bind error body: {err}"))?;
            return Err(format!(
                "product bind status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let body = to_bytes(product_bind_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read product bind body: {err}"))?;
        let json: Value = serde_json::from_slice(&body)
            .map_err(|err| format!("decode product bind json: {err}"))?;
        if json["data"]["binding_scope"].as_str() != Some("product") {
            return Err("product bind response binding_scope mismatch".to_string());
        }
        if json["data"]["bound_sku_count"].as_i64() != Some(1) {
            return Err("product bind response bound_sku_count mismatch".to_string());
        }

        let sku_bind_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/skus/{}/bind-template", ids.sku_id))
                    .header("content-type", "application/json")
                    .header("x-role", "platform_admin")
                    .header("x-request-id", &req_sku_bind)
                    .body(Body::from(
                        json!({
                            "template_id": ids.template_sku_id,
                            "binding_type": "license"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build sku bind request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call sku bind endpoint: {err}"))?;
        if sku_bind_resp.status() != StatusCode::OK {
            let status = sku_bind_resp.status();
            let body = to_bytes(sku_bind_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read sku bind error body: {err}"))?;
            return Err(format!(
                "sku bind status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let body = to_bytes(sku_bind_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read sku bind body: {err}"))?;
        let json: Value =
            serde_json::from_slice(&body).map_err(|err| format!("decode sku bind json: {err}"))?;
        if json["data"]["binding_scope"].as_str() != Some("sku") {
            return Err("sku bind response binding_scope mismatch".to_string());
        }
        if json["data"]["binding_type"].as_str() != Some("license") {
            return Err("sku bind response binding_type mismatch".to_string());
        }

        let policy_patch_resp = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/policies/{}", ids.policy_id))
                    .header("content-type", "application/json")
                    .header("x-role", "platform_admin")
                    .header("x-request-id", &req_policy_patch)
                    .body(Body::from(
                        json!({
                            "policy_name": format!("cat021-policy-updated-{suffix}"),
                            "subject_constraints": {"tenant_scope":"self"},
                            "usage_constraints": {"allow":["access","result_get"]},
                            "exportable": false,
                            "status": "active"
                        })
                        .to_string(),
                    ))
                    .map_err(|err| format!("build policy patch request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call policy patch endpoint: {err}"))?;
        if policy_patch_resp.status() != StatusCode::OK {
            let status = policy_patch_resp.status();
            let body = to_bytes(policy_patch_resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read policy patch error body: {err}"))?;
            return Err(format!(
                "policy patch status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let body = to_bytes(policy_patch_resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read policy patch body: {err}"))?;
        let json: Value = serde_json::from_slice(&body)
            .map_err(|err| format!("decode policy patch json: {err}"))?;
        if json["data"]["status"].as_str() != Some("active") {
            return Err("policy patch response status mismatch".to_string());
        }

        let binding_count = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM contract.template_binding
                 WHERE sku_id = $1::text::uuid
                   AND template_id IN ($2::text::uuid, $3::text::uuid)",
                &[&ids.sku_id, &ids.template_product_id, &ids.template_sku_id],
            )
            .await
            .map_err(|err| format!("query template binding rows: {err}"))?;
        let binding_count: i64 = binding_count.get(0);
        if binding_count < 2 {
            return Err("template bindings were not fully persisted".to_string());
        }

        let sku_meta = client
            .query_one(
                "SELECT metadata->>'draft_template_id'
                 FROM catalog.product_sku
                 WHERE sku_id = $1::text::uuid",
                &[&ids.sku_id],
            )
            .await
            .map_err(|err| format!("query sku metadata: {err}"))?;
        let draft_template_id: Option<String> = sku_meta.get(0);
        if draft_template_id.as_deref() != Some(ids.template_sku_id.as_str()) {
            return Err("sku draft_template_id was not updated by sku bind".to_string());
        }

        let policy_row = client
            .query_one(
                "SELECT policy_name, status, exportable
                 FROM contract.usage_policy
                 WHERE policy_id = $1::text::uuid",
                &[&ids.policy_id],
            )
            .await
            .map_err(|err| format!("query patched policy: {err}"))?;
        let policy_name: String = policy_row.get(0);
        let policy_status: String = policy_row.get(1);
        let policy_exportable: bool = policy_row.get(2);
        if !policy_name.starts_with("cat021-policy-updated-") {
            return Err("usage_policy.policy_name was not updated".to_string());
        }
        if policy_status != "active" || policy_exportable {
            return Err("usage_policy status/exportable mismatch after patch".to_string());
        }

        for (request_id, action_name, ref_type, ref_id) in [
            (
                req_product_bind.as_str(),
                "template.product.bind",
                "product",
                ids.product_id.as_str(),
            ),
            (
                req_sku_bind.as_str(),
                "template.sku.bind",
                "sku",
                ids.sku_id.as_str(),
            ),
            (
                req_policy_patch.as_str(),
                "template.policy.update",
                "usage_policy",
                ids.policy_id.as_str(),
            ),
        ] {
            let row = client
                .query_one(
                    "SELECT count(*)::bigint
                     FROM audit.audit_event
                     WHERE request_id = $1
                       AND action_name = $2
                       AND ref_type = $3
                       AND ref_id = $4::text::uuid",
                    &[&request_id, &action_name, &ref_type, &ref_id],
                )
                .await
                .map_err(|err| format!("query audit event ({action_name}): {err}"))?;
            let count: i64 = row.get(0);
            if count < 1 {
                return Err(format!("audit event missing for action {action_name}"));
            }
        }

        Ok(())
    }
    .await;

    cleanup_catalog_graph(&client, &ids).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
