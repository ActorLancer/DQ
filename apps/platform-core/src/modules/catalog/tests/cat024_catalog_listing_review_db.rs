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
    user_id: String,
    asset_id: String,
    asset_version_id: String,
    template_id: String,
}

async fn seed_base(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1::text, 'enterprise', 'active', '{}'::jsonb
             )
             RETURNING org_id::text",
            &[&format!("cat024-org-{suffix}")],
        )
        .await?;
    let org_id: String = org.get(0);

    let user = client
        .query_one(
            "INSERT INTO core.user_account (
               org_id, login_id, display_name, user_type, status, mfa_status
             ) VALUES (
               $1::text::uuid, $2::text, $3::text, 'human', 'active', 'verified'
             )
             RETURNING user_id::text",
            &[
                &org_id,
                &format!("cat024-user-{suffix}@example.com"),
                &format!("cat024-user-{suffix}"),
            ],
        )
        .await?;
    let user_id: String = user.get(0);

    let asset = client
        .query_one(
            "INSERT INTO catalog.data_asset (
               owner_org_id, title, category, sensitivity_level, status, metadata
             ) VALUES (
               $1::text::uuid, $2::text, 'manufacturing', 'internal', 'draft', '{}'::jsonb
             )
             RETURNING asset_id::text",
            &[&org_id, &format!("cat024-asset-{suffix}")],
        )
        .await?;
    let asset_id: String = asset.get(0);

    let asset_version = client
        .query_one(
            "INSERT INTO catalog.asset_version (
               asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
               data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
               trust_boundary_snapshot, status
             ) VALUES (
               $1::text::uuid, 1, 'v1', 'schema-hash', 'sample-hash', 'full-hash',
               4096, 'CN', ARRAY['CN']::text[], false, '{}'::jsonb, 'active'
             )
             RETURNING asset_version_id::text",
            &[&asset_id],
        )
        .await?;
    let asset_version_id: String = asset_version.get(0);

    let template = client
        .query_one(
            "INSERT INTO contract.template_definition (
               template_type, template_name, applicable_sku_types, status
             ) VALUES (
               'contract', $1::text, ARRAY['FILE_STD']::text[], 'active'
             )
             RETURNING template_id::text",
            &[&format!("CONTRACT_FILE_STD_CAT024_{suffix}")],
        )
        .await?;
    let template_id: String = template.get(0);

    Ok(SeedIds {
        org_id,
        user_id,
        asset_id,
        asset_version_id,
        template_id,
    })
}

async fn call_api_ok(
    app: axum::Router,
    request: Request<Body>,
    action: &str,
) -> Result<Value, String> {
    let resp = app
        .oneshot(request)
        .await
        .map_err(|err| format!("call {action}: {err}"))?;
    if resp.status() != StatusCode::OK {
        let status = resp.status();
        let body = to_bytes(resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read {action} error body: {err}"))?;
        return Err(format!(
            "{action} status mismatch: expected 200, got {status}; body={}",
            String::from_utf8_lossy(&body)
        ));
    }
    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .map_err(|err| format!("read {action} body: {err}"))?;
    serde_json::from_slice(&body).map_err(|err| format!("decode {action} response json: {err}"))
}

async fn cleanup(
    client: &Client,
    ids: &SeedIds,
    product_ids: &[String],
    sku_ids: &[String],
    request_ids: &[String],
) {
    for request_id in request_ids {
        let _ = client
            .execute(
                "DELETE FROM ops.outbox_event WHERE request_id = $1",
                &[request_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM audit.audit_event WHERE request_id = $1",
                &[request_id],
            )
            .await;
    }

    for product_id in product_ids {
        let _ = client
            .execute(
                "DELETE FROM review.review_step
                 WHERE review_task_id IN (
                   SELECT review_task_id
                   FROM review.review_task
                   WHERE ref_type = 'product'
                     AND ref_id = $1::text::uuid
                 )",
                &[product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM review.review_task
                 WHERE ref_type = 'product'
                   AND ref_id = $1::text::uuid",
                &[product_id],
            )
            .await;
    }

    for sku_id in sku_ids {
        let _ = client
            .execute(
                "DELETE FROM contract.data_contract WHERE sku_id = $1::text::uuid",
                &[sku_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM contract.template_binding
                 WHERE ref_type = 'sku'
                   AND ref_id = $1::text::uuid",
                &[sku_id],
            )
            .await;
    }

    for product_id in product_ids {
        let _ = client
            .execute(
                "DELETE FROM contract.template_binding
                 WHERE ref_type = 'product'
                   AND ref_id = $1::text::uuid",
                &[product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product_metadata_profile WHERE product_id = $1::text::uuid",
                &[product_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM catalog.product WHERE product_id = $1::text::uuid",
                &[product_id],
            )
            .await;
    }

    let _ = client
        .execute(
            "DELETE FROM catalog.asset_quality_report WHERE asset_version_id = $1::text::uuid",
            &[&ids.asset_version_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM iam.step_up_challenge WHERE user_id = $1::text::uuid",
            &[&ids.user_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM contract.template_definition WHERE template_id = $1::text::uuid",
            &[&ids.template_id],
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
            "DELETE FROM core.user_account WHERE user_id = $1::text::uuid",
            &[&ids.user_id],
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
async fn cat024_catalog_listing_review_end_to_end_db_smoke() {
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
    let ids = seed_base(&client, &suffix).await.expect("seed base graph");
    let mut created_products: Vec<String> = Vec::new();
    let mut created_skus: Vec<String> = Vec::new();
    let mut request_ids: Vec<String> = Vec::new();

    let outcome: Result<(), String> = async {
        let app = crate::with_live_test_state(router()).await;

        let create_product_req = format!("req-cat024-product-create-a-{suffix}");
        request_ids.push(create_product_req.clone());
        let product_a = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri("/api/v1/products")
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &create_product_req)
                .body(Body::from(
                    json!({
                        "asset_id": ids.asset_id,
                        "asset_version_id": ids.asset_version_id,
                        "seller_org_id": ids.org_id,
                        "title": format!("cat024-product-a-{suffix}"),
                        "category": "manufacturing",
                        "product_type": "data_product",
                        "description": "cat024 product a",
                        "delivery_type": "file_download",
                        "searchable_text": "cat024 product a searchable text"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build create product a request: {err}"))?,
            "create product a",
        )
        .await?;
        let product_a_id = product_a["data"]["product_id"]
            .as_str()
            .ok_or_else(|| "missing product_id in create product a response".to_string())?
            .to_string();
        created_products.push(product_a_id.clone());

        let create_sku_req = format!("req-cat024-sku-create-a-{suffix}");
        request_ids.push(create_sku_req.clone());
        let sku_a = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/products/{product_a_id}/skus"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &create_sku_req)
                .body(Body::from(
                    json!({
                        "sku_code": format!("CAT024-SKU-A-{suffix}"),
                        "sku_type": "FILE_STD",
                        "billing_mode": "one_time",
                        "template_id": ids.template_id,
                        "acceptance_mode": "manual_accept",
                        "refund_mode": "manual_refund"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build create sku a request: {err}"))?,
            "create sku a",
        )
        .await?;
        let sku_a_id = sku_a["data"]["sku_id"]
            .as_str()
            .ok_or_else(|| "missing sku_id in create sku a response".to_string())?
            .to_string();
        created_skus.push(sku_a_id.clone());

        let put_metadata_req = format!("req-cat024-metadata-put-a-{suffix}");
        request_ids.push(put_metadata_req.clone());
        let _ = call_api_ok(
            app.clone(),
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/products/{product_a_id}/metadata-profile"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &put_metadata_req)
                .body(Body::from(
                    json!({
                        "business_description_json": {"product_name":"cat024 product a"},
                        "data_content_json": {"object_type":"table"},
                        "structure_description_json": {"schema_version":"v1"},
                        "quality_description_json": {"missing_rate":0.01},
                        "compliance_description_json": {"contains_pi":false},
                        "delivery_description_json": {"delivery_modes":["file_download"]},
                        "version_description_json": {"version_no":1},
                        "authorization_description_json": {"license_scope":"internal_use"},
                        "responsibility_description_json": {"owner":"seller"},
                        "processing_overview_json": {"processing_mode":"seller_self_managed"}
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build metadata profile request: {err}"))?,
            "put product metadata profile",
        )
        .await?;

        let quality_req = format!("req-cat024-quality-report-a-{suffix}");
        request_ids.push(quality_req.clone());
        let _ = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/assets/{}/quality-reports",
                    ids.asset_version_id
                ))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &quality_req)
                .body(Body::from(
                    json!({
                        "report_no": 1,
                        "report_type": "seller_declared",
                        "missing_rate": 0.01,
                        "duplicate_rate": 0.02,
                        "anomaly_rate": 0.03,
                        "metrics_json": {"completeness":0.99}
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build quality report request: {err}"))?,
            "create asset quality report",
        )
        .await?;

        let contract_req = format!("req-cat024-data-contract-a-{suffix}");
        request_ids.push(contract_req.clone());
        let _ = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/skus/{sku_a_id}/data-contracts"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &contract_req)
                .body(Body::from(
                    json!({
                        "asset_version_id": ids.asset_version_id,
                        "product_id": product_a_id,
                        "contract_name": format!("cat024-contract-a-{suffix}"),
                        "contract_scope": "sku",
                        "business_terms_json": {"delivery":"T+0"},
                        "rights_terms_json": {"allowed":["internal_use"]}
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build data contract request: {err}"))?,
            "create data contract",
        )
        .await?;

        let submit_req = format!("req-cat024-submit-a-{suffix}");
        request_ids.push(submit_req.clone());
        let submit_a = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/products/{product_a_id}/submit"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &submit_req)
                .body(Body::from(
                    json!({ "submission_note": "submit for review" }).to_string(),
                ))
                .map_err(|err| format!("build submit product a request: {err}"))?,
            "submit product a",
        )
        .await?;
        if submit_a["data"]["status"].as_str() != Some("pending_review") {
            return Err("product a submit status is not pending_review".to_string());
        }

        let review_approve_req = format!("req-cat024-review-approve-a-{suffix}");
        request_ids.push(review_approve_req.clone());
        let review_a = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/review/products/{product_a_id}"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &review_approve_req)
                .body(Body::from(
                    json!({
                        "action_name": "approve",
                        "action_reason": "cat024 approve path"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build review approve request: {err}"))?,
            "review approve product a",
        )
        .await?;
        if review_a["data"]["status"].as_str() != Some("listed") {
            return Err("product a review approve status is not listed".to_string());
        }

        let step_up_id_row = client
            .query_one(
                "INSERT INTO iam.step_up_challenge (
                   user_id, challenge_type, target_action, target_ref_type, target_ref_id,
                   challenge_status, expires_at, completed_at, metadata
                 ) VALUES (
                   $1::text::uuid, 'mfa', 'risk.product.freeze', 'product', $2::text::uuid,
                   'verified', now() + interval '30 minutes', now(), '{}'::jsonb
                 )
                 RETURNING step_up_challenge_id::text",
                &[&ids.user_id, &product_a_id],
            )
            .await
            .map_err(|err| format!("insert verified step-up challenge: {err}"))?;
        let step_up_id: String = step_up_id_row.get(0);

        let freeze_req = format!("req-cat024-freeze-a-{suffix}");
        request_ids.push(freeze_req.clone());
        let freeze_a = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/products/{product_a_id}/suspend"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-user-id", &ids.user_id)
                .header("x-step-up-challenge-id", &step_up_id)
                .header("x-request-id", &freeze_req)
                .body(Body::from(
                    json!({
                        "suspend_mode": "freeze",
                        "reason": "cat024 freeze path"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build freeze request: {err}"))?,
            "freeze product a",
        )
        .await?;
        if freeze_a["data"]["status"].as_str() != Some("frozen") {
            return Err("product a freeze status is not frozen".to_string());
        }

        let create_product_b_req = format!("req-cat024-product-create-b-{suffix}");
        request_ids.push(create_product_b_req.clone());
        let product_b = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri("/api/v1/products")
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &create_product_b_req)
                .body(Body::from(
                    json!({
                        "asset_id": ids.asset_id,
                        "asset_version_id": ids.asset_version_id,
                        "seller_org_id": ids.org_id,
                        "title": format!("cat024-product-b-{suffix}"),
                        "category": "manufacturing",
                        "product_type": "data_product",
                        "delivery_type": "file_download"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build create product b request: {err}"))?,
            "create product b",
        )
        .await?;
        let product_b_id = product_b["data"]["product_id"]
            .as_str()
            .ok_or_else(|| "missing product_id in create product b response".to_string())?
            .to_string();
        created_products.push(product_b_id.clone());

        let create_sku_b_req = format!("req-cat024-sku-create-b-{suffix}");
        request_ids.push(create_sku_b_req.clone());
        let sku_b = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/products/{product_b_id}/skus"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &create_sku_b_req)
                .body(Body::from(
                    json!({
                        "sku_code": format!("CAT024-SKU-B-{suffix}"),
                        "sku_type": "FILE_STD",
                        "billing_mode": "one_time",
                        "template_id": ids.template_id,
                        "acceptance_mode": "manual_accept",
                        "refund_mode": "manual_refund"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build create sku b request: {err}"))?,
            "create sku b",
        )
        .await?;
        let sku_b_id = sku_b["data"]["sku_id"]
            .as_str()
            .ok_or_else(|| "missing sku_id in create sku b response".to_string())?
            .to_string();
        created_skus.push(sku_b_id.clone());

        let metadata_b_req = format!("req-cat024-metadata-put-b-{suffix}");
        request_ids.push(metadata_b_req.clone());
        let _ = call_api_ok(
            app.clone(),
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/products/{product_b_id}/metadata-profile"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &metadata_b_req)
                .body(Body::from(
                    json!({
                        "business_description_json": {"product_name":"cat024 product b"},
                        "data_content_json": {"object_type":"table"},
                        "structure_description_json": {"schema_version":"v1"},
                        "quality_description_json": {"missing_rate":0.01},
                        "compliance_description_json": {"contains_pi":false},
                        "delivery_description_json": {"delivery_modes":["file_download"]},
                        "version_description_json": {"version_no":1},
                        "authorization_description_json": {"license_scope":"internal_use"},
                        "responsibility_description_json": {"owner":"seller"},
                        "processing_overview_json": {"processing_mode":"seller_self_managed"}
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build metadata profile b request: {err}"))?,
            "put product metadata profile b",
        )
        .await?;

        let submit_b_req = format!("req-cat024-submit-b-{suffix}");
        request_ids.push(submit_b_req.clone());
        let submit_b = call_api_ok(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/products/{product_b_id}/submit"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &submit_b_req)
                .body(Body::from(
                    json!({ "submission_note": "submit for reject path" }).to_string(),
                ))
                .map_err(|err| format!("build submit product b request: {err}"))?,
            "submit product b",
        )
        .await?;
        if submit_b["data"]["status"].as_str() != Some("pending_review") {
            return Err("product b submit status is not pending_review".to_string());
        }

        let review_reject_req = format!("req-cat024-review-reject-b-{suffix}");
        request_ids.push(review_reject_req.clone());
        let review_b = call_api_ok(
            app,
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/review/products/{product_b_id}"))
                .header("content-type", "application/json")
                .header("x-role", "platform_admin")
                .header("x-request-id", &review_reject_req)
                .body(Body::from(
                    json!({
                        "action_name": "reject",
                        "action_reason": "cat024 reject path"
                    })
                    .to_string(),
                ))
                .map_err(|err| format!("build review reject request: {err}"))?,
            "review reject product b",
        )
        .await?;
        if review_b["data"]["status"].as_str() != Some("draft") {
            return Err("product b review reject status is not draft".to_string());
        }

        let quality_count_row = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM catalog.asset_quality_report
                 WHERE asset_version_id = $1::text::uuid",
                &[&ids.asset_version_id],
            )
            .await
            .map_err(|err| format!("query quality report count: {err}"))?;
        let quality_count: i64 = quality_count_row.get(0);
        if quality_count < 1 {
            return Err("quality report not persisted".to_string());
        }

        let contract_count_row = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM contract.data_contract
                 WHERE sku_id = $1::text::uuid",
                &[&sku_a_id],
            )
            .await
            .map_err(|err| format!("query data contract count: {err}"))?;
        let contract_count: i64 = contract_count_row.get(0);
        if contract_count < 1 {
            return Err("data contract not persisted".to_string());
        }

        for action_name in [
            "catalog.product.create",
            "catalog.sku.create",
            "catalog.asset_quality_report.create",
            "catalog.data_contract.create",
            "catalog.product.submit",
            "catalog.review.product",
            "catalog.product.suspend",
        ] {
            let audit_row = client
                .query_one(
                    "SELECT count(*)::bigint
                     FROM audit.audit_event
                     WHERE action_name = $1
                       AND request_id = ANY($2::text[])",
                    &[&action_name, &request_ids],
                )
                .await
                .map_err(|err| format!("query audit action {action_name}: {err}"))?;
            let count: i64 = audit_row.get(0);
            if count < 1 {
                return Err(format!("audit event missing for action={action_name}"));
            }
        }

        let status_event_row = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM ops.outbox_event
                 WHERE event_type = 'catalog.product.status.changed'
                   AND aggregate_id = $1::text::uuid",
                &[&product_a_id],
            )
            .await
            .map_err(|err| format!("query status changed outbox events: {err}"))?;
        let status_event_count: i64 = status_event_row.get(0);
        if status_event_count < 2 {
            return Err(
                "catalog.product.status.changed outbox events are insufficient".to_string(),
            );
        }

        Ok(())
    }
    .await;

    cleanup(
        &client,
        &ids,
        &created_products,
        &created_skus,
        &request_ids,
    )
    .await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
