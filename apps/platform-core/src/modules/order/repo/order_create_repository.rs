use crate::modules::order::domain::{
    OrderPriceSnapshot, SettlementTermsSnapshot, TaxTermsSnapshot, derive_layered_status,
    derive_settlement_basis, resolve_standard_scenario_snapshot,
};
use crate::modules::order::dto::{CreateOrderRequest, CreateOrderResponseData};
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

struct ProductSkuContext {
    product_id: String,
    asset_version_id: String,
    seller_org_id: String,
    product_status: String,
    unit_price: String,
    currency_code: String,
    pricing_mode: String,
    metadata: Value,
    sku_id: String,
    sku_code: String,
    sku_type: String,
    billing_mode: String,
    refund_mode: String,
    sku_status: String,
    risk_blocked: bool,
}

pub async fn find_order_by_idempotency(
    client: &Client,
    idempotency_key: &str,
) -> Result<Option<CreateOrderResponseData>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               product_id::text,
               sku_id::text,
               status,
               payment_status,
               amount::text,
               currency_code,
               price_snapshot_json,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main
             WHERE idempotency_key = $1",
            &[&idempotency_key],
        )
        .await
        .map_err(map_db_error)?;
    row.map(|r| parse_created_order_row(&r)).transpose()
}

pub async fn create_order_with_snapshot(
    client: &mut Client,
    payload: &CreateOrderRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<CreateOrderResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let ctx = load_product_sku_context(&tx, &payload.product_id, &payload.sku_id).await?;
    validate_order_create_preconditions(&tx, &ctx, &payload.buyer_org_id).await?;

    let price_snapshot = build_price_snapshot(&ctx, payload.scenario_code.as_deref())?;
    let price_snapshot_json = serde_json::to_value(&price_snapshot).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("snapshot serialization failed: {err}"),
                request_id: request_id.map(|s| s.to_string()),
            }),
        )
    })?;
    let fee_preview_snapshot = serde_json::json!({
        "pricing_mode": ctx.pricing_mode,
        "unit_price": ctx.unit_price,
        "currency_code": ctx.currency_code,
        "captured_at": price_snapshot.captured_at
    });
    let layered_status = derive_layered_status("created", "unpaid");

    let row = tx
        .query_one(
            "INSERT INTO trade.order_main (
               inquiry_id,
               product_id,
               asset_version_id,
               buyer_org_id,
               seller_org_id,
               sku_id,
               status,
               payment_status,
               delivery_status,
               acceptance_status,
               settlement_status,
               dispute_status,
               payment_mode,
               amount,
               currency_code,
               fee_preview_snapshot,
               price_snapshot_json,
               idempotency_key,
               last_reason_code
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4::text::uuid,
               $5::text::uuid,
               $6::text::uuid,
               'created',
               'unpaid',
               $12,
               $13,
               $14,
               $15,
               'online',
               $7::text::numeric,
               $8,
               $9::jsonb,
               $10::jsonb,
               $11,
               'TRADE-003'
             )
             RETURNING
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               product_id::text,
               sku_id::text,
               status,
               payment_status,
               amount::text,
               currency_code,
               price_snapshot_json,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &payload.inquiry_id,
                &ctx.product_id,
                &ctx.asset_version_id,
                &payload.buyer_org_id,
                &ctx.seller_org_id,
                &ctx.sku_id,
                &ctx.unit_price,
                &ctx.currency_code,
                &fee_preview_snapshot,
                &price_snapshot_json,
                &idempotency_key,
                &layered_status.delivery_status,
                &layered_status.acceptance_status,
                &layered_status.settlement_status,
                &layered_status.dispute_status,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let created = parse_created_order_row(&row)?;

    write_trade_audit_event(
        &tx,
        "order",
        &created.order_id,
        actor_role,
        "trade.order.create",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    write_order_create_outbox_event(&tx, &created, request_id, trace_id, idempotency_key, &ctx)
        .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(created)
}

async fn load_product_sku_context(
    client: &(impl GenericClient + Sync),
    product_id: &str,
    sku_id: &str,
) -> Result<ProductSkuContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               p.product_id::text,
               p.asset_version_id::text,
               p.seller_org_id::text,
               p.status,
               p.price::text,
               p.currency_code,
               p.price_mode,
               p.metadata,
               s.sku_id::text,
               s.sku_code,
               s.sku_type,
               s.billing_mode,
               s.refund_mode,
               s.status,
               CASE
                 WHEN lower(coalesce(p.metadata->>'risk_blocked', 'false')) IN ('true', '1') THEN true
                 WHEN lower(coalesce(p.metadata#>>'{risk_flags,block_submit}', 'false')) IN ('true', '1') THEN true
                 ELSE false
               END AS risk_blocked
             FROM catalog.product p
             JOIN catalog.product_sku s ON s.product_id = p.product_id
             WHERE p.product_id = $1::text::uuid
               AND s.sku_id = $2::text::uuid",
            &[&product_id, &sku_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product or sku not found".to_string(),
                request_id: None,
            }),
        ));
    };

    Ok(ProductSkuContext {
        product_id: row.get(0),
        asset_version_id: row.get(1),
        seller_org_id: row.get(2),
        product_status: row.get(3),
        unit_price: row.get(4),
        currency_code: row.get(5),
        pricing_mode: row.get(6),
        metadata: row.get(7),
        sku_id: row.get(8),
        sku_code: row.get(9),
        sku_type: row.get(10),
        billing_mode: row.get(11),
        refund_mode: row.get(12),
        sku_status: row.get(13),
        risk_blocked: row.get(14),
    })
}

async fn validate_order_create_preconditions(
    client: &(impl GenericClient + Sync),
    context: &ProductSkuContext,
    buyer_org_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.product_status != "listed" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "ORDER_CREATE_FORBIDDEN".to_string(),
                message: "ORDER_CREATE_FORBIDDEN: product is not listed".to_string(),
                request_id: None,
            }),
        ));
    }
    if !matches!(context.sku_status.as_str(), "active" | "listed") {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: "ORDER_CREATE_FORBIDDEN".to_string(),
                message: "ORDER_CREATE_FORBIDDEN: sku is not purchasable".to_string(),
                request_id: None,
            }),
        ));
    }
    if context.risk_blocked {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: "ORDER_CREATE_FORBIDDEN".to_string(),
                message: "ORDER_CREATE_FORBIDDEN: product is blocked by risk policy".to_string(),
                request_id: None,
            }),
        ));
    }
    ensure_org_active(client, buyer_org_id, "buyer").await?;
    ensure_org_active(client, &context.seller_org_id, "seller").await?;
    Ok(())
}

async fn ensure_org_active(
    client: &(impl GenericClient + Sync),
    org_id: &str,
    org_label: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status
             FROM core.organization
             WHERE org_id = $1::text::uuid",
            &[&org_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: "ORDER_CREATE_FORBIDDEN".to_string(),
                message: format!("ORDER_CREATE_FORBIDDEN: {org_label} organization not found"),
                request_id: None,
            }),
        ));
    };
    let status: String = row.get(0);
    if status != "active" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: "ORDER_CREATE_FORBIDDEN".to_string(),
                message: format!(
                    "ORDER_CREATE_FORBIDDEN: {org_label} organization status is not active"
                ),
                request_id: None,
            }),
        ));
    }
    Ok(())
}

fn build_price_snapshot(
    context: &ProductSkuContext,
    scenario_code: Option<&str>,
) -> Result<OrderPriceSnapshot, (StatusCode, Json<ErrorResponse>)> {
    let scenario_snapshot = resolve_standard_scenario_snapshot(
        &context.sku_id,
        &context.sku_code,
        &context.sku_type,
        scenario_code,
        &context.metadata,
    )
    .map_err(|err| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: err.message(),
                request_id: None,
            }),
        )
    })?;

    Ok(OrderPriceSnapshot {
        product_id: context.product_id.clone(),
        sku_id: context.sku_id.clone(),
        sku_code: context.sku_code.clone(),
        sku_type: context.sku_type.clone(),
        pricing_mode: context.pricing_mode.clone(),
        unit_price: context.unit_price.clone(),
        currency_code: context.currency_code.clone(),
        billing_mode: context.billing_mode.clone(),
        refund_mode: context.refund_mode.clone(),
        settlement_terms: SettlementTermsSnapshot {
            settlement_basis: derive_settlement_basis(&context.billing_mode, &context.pricing_mode),
            settlement_mode: "manual_v1".to_string(),
        },
        tax_terms: parse_tax_terms(&context.metadata),
        scenario_snapshot: Some(scenario_snapshot),
        captured_at: kernel::UtcTimestampMs::now().0.to_string(),
        source: "catalog.product + catalog.product_sku".to_string(),
    })
}

fn parse_tax_terms(metadata: &Value) -> TaxTermsSnapshot {
    let tax = metadata
        .get("tax")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    TaxTermsSnapshot {
        tax_policy: tax
            .get("policy")
            .and_then(Value::as_str)
            .unwrap_or("platform_default")
            .to_string(),
        tax_code: tax
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("UNSPECIFIED")
            .to_string(),
        tax_inclusive: tax
            .get("inclusive")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn parse_created_order_row(
    row: &Row,
) -> Result<CreateOrderResponseData, (StatusCode, Json<ErrorResponse>)> {
    let snapshot: Value = row.get(9);
    let price_snapshot = serde_json::from_value::<OrderPriceSnapshot>(snapshot).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("invalid stored order snapshot: {err}"),
                request_id: None,
            }),
        )
    })?;

    Ok(CreateOrderResponseData {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        product_id: row.get(3),
        sku_id: row.get(4),
        current_state: row.get(5),
        payment_status: row.get(6),
        order_amount: row.get(7),
        currency_code: row.get(8),
        price_snapshot,
        created_at: row.get(10),
    })
}

async fn write_order_create_outbox_event(
    client: &(impl GenericClient + Sync),
    created: &CreateOrderResponseData,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
    context: &ProductSkuContext,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let payload = json!({
        "order_id": created.order_id,
        "buyer_org_id": created.buyer_org_id,
        "seller_org_id": created.seller_org_id,
        "product_id": created.product_id,
        "sku_id": created.sku_id,
        "sku_code": context.sku_code,
        "sku_type": context.sku_type,
        "current_state": created.current_state,
        "payment_status": created.payment_status,
        "order_amount": created.order_amount,
        "currency_code": created.currency_code,
        "scenario_snapshot": created.price_snapshot.scenario_snapshot.clone(),
        "created_at": created.created_at
    });
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "trade.order",
            aggregate_id: &created.order_id,
            event_type: "trade.order.created",
            producer_service: "platform-core.order",
            request_id,
            trace_id,
            idempotency_key,
            occurred_at: Some(created.created_at.as_str()),
            business_payload: &payload,
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}
