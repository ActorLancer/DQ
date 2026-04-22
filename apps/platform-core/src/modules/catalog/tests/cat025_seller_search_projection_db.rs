use crate::modules::catalog::api::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::Value;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    product_ids: Vec<String>,
    asset_ids: Vec<String>,
    asset_version_ids: Vec<String>,
}

#[derive(Debug)]
struct SellerProjectionRow {
    seller_name: String,
    country_code: Option<String>,
    region_code: Option<String>,
    industry_tags: Vec<String>,
    certification_tags: Vec<String>,
    featured_products: Value,
    rating_summary: Value,
    document_version: i64,
    index_sync_status: String,
}

async fn seed_seller_projection_graph(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org_row = client
        .query_one(
            "INSERT INTO core.organization (
               org_name,
               org_type,
               status,
               real_name_status,
               compliance_level,
               country_code,
               region_code,
               industry_tags,
               metadata
             ) VALUES (
               $1,
               'enterprise',
               'active',
               'verified',
               'L2',
               'CN',
               'SH',
               ARRAY['industrial_manufacturing','supply_chain']::text[],
               jsonb_build_object(
                 'description', $2,
                 'certification_level', 'enhanced',
                 'certification_tags', jsonb_build_array('iso27001', 'trusted_partner')
               )
             )
             RETURNING org_id::text",
            &[
                &format!("cat025-org-{suffix}"),
                &format!("seller projection {suffix}"),
            ],
        )
        .await?;
    let org_id: String = org_row.get(0);

    client
        .execute(
            "INSERT INTO risk.reputation_snapshot (
               subject_type,
               subject_id,
               score,
               risk_level,
               credit_level,
               effective_at,
               metadata
             ) VALUES (
               'organization',
               $1::text::uuid,
               0.88,
               1,
               4,
               now(),
               jsonb_build_object(
                 'rating_count', 12,
                 'average_rating', 4.70,
                 'last_rating_at', '2026-04-22T00:00:00.000Z'
               )
             )",
            &[&org_id],
        )
        .await?;

    let product_specs = [
        ("featured-alpha", 1_i32, "0.95", "行研精选", "listed"),
        ("featured-beta", 2_i32, "0.87", "供应链画像", "listed"),
        ("featured-gamma", 3_i32, "0.76", "制造洞察", "listed"),
        ("draft-hidden", 9_i32, "0.99", "草稿不应入榜", "draft"),
    ];

    let mut product_ids = Vec::new();
    let mut asset_ids = Vec::new();
    let mut asset_version_ids = Vec::new();

    for (idx, (label, featured_rank, quality_score, subtitle, status)) in
        product_specs.into_iter().enumerate()
    {
        let asset_row = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description
                 ) VALUES (
                   $1::text::uuid, $2, 'manufacturing', 'internal', 'draft', $3
                 )
                 RETURNING asset_id::text",
                &[
                    &org_id,
                    &format!("cat025-asset-{label}-{suffix}"),
                    &format!("asset {label} {suffix}"),
                ],
            )
            .await?;
        let asset_id: String = asset_row.get(0);
        asset_ids.push(asset_id.clone());

        let asset_version_row = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id,
                   version_no,
                   schema_version,
                   schema_hash,
                   sample_hash,
                   full_hash,
                   data_size_bytes,
                   origin_region,
                   allowed_region,
                   requires_controlled_execution,
                   trust_boundary_snapshot,
                   status
                 ) VALUES (
                   $1::text::uuid,
                   1,
                   'v1',
                   $2,
                   $3,
                   $4,
                   2048,
                   'CN',
                   ARRAY['CN']::text[],
                   false,
                   '{}'::jsonb,
                   'active'
                 )
                 RETURNING asset_version_id::text",
                &[
                    &asset_id,
                    &format!("schema-{label}-{suffix}"),
                    &format!("sample-{label}-{suffix}"),
                    &format!("full-{label}-{suffix}"),
                ],
            )
            .await?;
        let asset_version_id: String = asset_version_row.get(0);
        asset_version_ids.push(asset_version_id.clone());

        let price_value = 90 + idx as i32 * 10;
        let product_row = client
            .query_one(
                "INSERT INTO catalog.product (
                   asset_id,
                   asset_version_id,
                   seller_org_id,
                   title,
                   category,
                   product_type,
                   description,
                   status,
                   price_mode,
                   price,
                   currency_code,
                   delivery_type,
                   allowed_usage,
                   searchable_text,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4,
                   'manufacturing',
                   'data_product',
                   $5,
                   $6,
                   'one_time',
                   $7,
                   'CNY',
                   'file_download',
                   ARRAY['internal_use']::text[],
                   $8,
                   jsonb_build_object(
                     'subtitle', $9,
                     'industry', 'industrial_manufacturing',
                     'quality_score', $10,
                     'is_featured', CASE WHEN $11::int < 4 THEN true ELSE false END,
                     'featured_rank', $11
                   )
                 )
                 RETURNING product_id::text",
                &[
                    &asset_id,
                    &asset_version_id,
                    &org_id,
                    &format!("cat025-product-{label}-{suffix}"),
                    &format!("product {label} {suffix}"),
                    &status,
                    &price_value,
                    &format!("seller projection keyword {label} {suffix}"),
                    &subtitle,
                    &quality_score,
                    &featured_rank,
                ],
            )
            .await?;
        product_ids.push(product_row.get(0));
    }

    client
        .execute(
            "SELECT search.refresh_seller_search_document_by_id($1::text::uuid)",
            &[&org_id],
        )
        .await?;

    Ok(SeedIds {
        org_id,
        product_ids,
        asset_ids,
        asset_version_ids,
    })
}

async fn load_seller_projection(
    client: &Client,
    org_id: &str,
) -> Result<SellerProjectionRow, Error> {
    let row = client
        .query_one(
            "SELECT
               seller_name,
               country_code,
               region_code,
               COALESCE(industry_tags, '{}')::text[],
               COALESCE(certification_tags, '{}')::text[],
               COALESCE(featured_products, '[]'::jsonb),
               COALESCE(rating_summary, '{}'::jsonb),
               document_version,
               index_sync_status
             FROM search.seller_search_document
             WHERE org_id = $1::text::uuid",
            &[&org_id],
        )
        .await?;
    Ok(SellerProjectionRow {
        seller_name: row.get(0),
        country_code: row.get(1),
        region_code: row.get(2),
        industry_tags: row.get(3),
        certification_tags: row.get(4),
        featured_products: row.get(5),
        rating_summary: row.get(6),
        document_version: row.get(7),
        index_sync_status: row.get(8),
    })
}

async fn cleanup_seller_projection_graph(client: &Client, seed: &SeedIds) {
    let _ = client
        .execute(
            "DELETE FROM search.seller_search_document WHERE org_id = $1::text::uuid",
            &[&seed.org_id],
        )
        .await;
    let _ = client
        .execute(
            "DELETE FROM risk.reputation_snapshot
             WHERE subject_type = 'organization'
               AND subject_id = $1::text::uuid",
            &[&seed.org_id],
        )
        .await;
    for product_id in &seed.product_ids {
        let _ = client
            .execute(
                "DELETE FROM search.product_search_document WHERE product_id = $1::text::uuid",
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
    for asset_version_id in &seed.asset_version_ids {
        let _ = client
            .execute(
                "DELETE FROM catalog.asset_version WHERE asset_version_id = $1::text::uuid",
                &[asset_version_id],
            )
            .await;
    }
    for asset_id in &seed.asset_ids {
        let _ = client
            .execute(
                "DELETE FROM catalog.data_asset WHERE asset_id = $1::text::uuid",
                &[asset_id],
            )
            .await;
    }
    let _ = client
        .execute(
            "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
            &[&seed.org_id],
        )
        .await;
}

#[tokio::test]
async fn cat025_seller_search_projection_db_smoke() {
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
    let seed = seed_seller_projection_graph(&client, &suffix)
        .await
        .expect("seed seller projection graph");
    let request_id = format!("req-cat025-seller-profile-{suffix}");

    let outcome: Result<(), String> = async {
        let projection = load_seller_projection(&client, &seed.org_id)
            .await
            .map_err(|err| format!("load seller projection failed: {err}"))?;

        if projection.country_code.as_deref() != Some("CN") {
            return Err("seller projection country_code mismatch".to_string());
        }
        if projection.region_code.as_deref() != Some("SH") {
            return Err("seller projection region_code mismatch".to_string());
        }
        if !projection
            .industry_tags
            .iter()
            .any(|tag| tag == "industrial_manufacturing")
        {
            return Err(
                "seller projection industry_tags missing industrial_manufacturing".to_string(),
            );
        }
        for expected_tag in [
            "real_name_verified",
            "compliance:l2",
            "certification:enhanced",
            "iso27001",
            "trusted_partner",
        ] {
            if !projection
                .certification_tags
                .iter()
                .any(|tag| tag == expected_tag)
            {
                return Err(format!(
                    "seller projection certification tag missing: {expected_tag}"
                ));
            }
        }
        let featured_products = projection
            .featured_products
            .as_array()
            .ok_or_else(|| "seller projection featured_products is not array".to_string())?;
        if featured_products.len() != 3 {
            return Err(format!(
                "seller projection featured_products length mismatch: expected 3, got {}",
                featured_products.len()
            ));
        }
        let featured_titles: Vec<&str> = featured_products
            .iter()
            .filter_map(|item| item["title"].as_str())
            .collect();
        let expected_titles = [
            format!("cat025-product-featured-alpha-{suffix}"),
            format!("cat025-product-featured-beta-{suffix}"),
            format!("cat025-product-featured-gamma-{suffix}"),
        ];
        if featured_titles
            != expected_titles
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        {
            return Err(format!(
                "seller projection featured_titles mismatch: expected {:?}, got {:?}",
                expected_titles, featured_titles
            ));
        }
        if projection.rating_summary["rating_count"].as_i64() != Some(12) {
            return Err("seller projection rating_count mismatch".to_string());
        }
        if projection.rating_summary["average_rating"].as_f64() != Some(4.7) {
            return Err("seller projection average_rating mismatch".to_string());
        }
        if projection.rating_summary["reputation_score"].as_f64() != Some(0.88) {
            return Err("seller projection reputation_score mismatch".to_string());
        }
        if projection.rating_summary["credit_level"].as_i64() != Some(4) {
            return Err("seller projection credit_level mismatch".to_string());
        }
        if projection.rating_summary["risk_level"].as_i64() != Some(1) {
            return Err("seller projection risk_level mismatch".to_string());
        }
        if projection.document_version < 1 {
            return Err("seller projection document_version missing".to_string());
        }
        if projection.index_sync_status != "pending" {
            return Err(format!(
                "seller projection index_sync_status mismatch: expected pending, got {}",
                projection.index_sync_status
            ));
        }

        let app = crate::with_live_test_state(router()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/sellers/{}/profile", seed.org_id))
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build seller profile request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call seller profile endpoint: {err}"))?;
        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read seller profile error body: {err}"))?;
            return Err(format!(
                "seller profile status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read seller profile body: {err}"))?;
        let json: Value = serde_json::from_slice(&body)
            .map_err(|err| format!("decode seller profile json: {err}"))?;
        if json["data"]["org_id"].as_str() != Some(seed.org_id.as_str()) {
            return Err("seller profile org_id mismatch".to_string());
        }
        if json["data"]["org_name"].as_str() != Some(projection.seller_name.as_str()) {
            return Err("seller profile org_name mismatch".to_string());
        }
        let api_cert_tags = json["data"]["certification_tags"]
            .as_array()
            .ok_or_else(|| "seller profile certification_tags missing".to_string())?;
        if api_cert_tags.len() < 5 {
            return Err("seller profile certification_tags too short".to_string());
        }
        let api_featured_products = json["data"]["featured_products"]
            .as_array()
            .ok_or_else(|| "seller profile featured_products missing".to_string())?;
        if api_featured_products.len() != 3 {
            return Err("seller profile featured_products length mismatch".to_string());
        }
        if json["data"]["rating_summary"]["average_rating"].as_f64() != Some(4.7) {
            return Err("seller profile average_rating mismatch".to_string());
        }

        let audit_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'catalog.seller.profile.read'
                   AND ref_type = 'seller'
                   AND ref_id = $2::text::uuid",
                &[&request_id, &seed.org_id],
            )
            .await
            .map_err(|err| format!("query seller profile audit event failed: {err}"))?
            .get(0);
        if audit_count < 1 {
            return Err("seller profile audit event missing".to_string());
        }

        Ok(())
    }
    .await;

    cleanup_seller_projection_graph(&client, &seed).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
