use crate::modules::order::domain::{
    OrderPriceSnapshot, SettlementTermsSnapshot, TaxTermsSnapshot, derive_settlement_basis,
    resolve_standard_scenario_snapshot,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

pub async fn freeze_order_price_snapshot(
    client: &Client,
    order_id: &str,
) -> Result<Option<OrderPriceSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               o.sku_id::text,
               o.price_snapshot_json,
               p.price_mode,
               p.price::text,
               p.currency_code,
               p.metadata,
               s.sku_code,
               s.sku_type,
               s.billing_mode,
               s.refund_mode
             FROM trade.order_main o
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Ok(None);
    };

    let current_snapshot: Value = row.get(3);
    let metadata: Value = row.get(7);
    let tax_terms = parse_tax_terms(&metadata);
    let pricing_mode: String = row.get(4);
    let billing_mode: String = row.get(10);
    let scenario_code_hint = current_snapshot
        .get("scenario_snapshot")
        .and_then(|value| value.get("scenario_code"))
        .and_then(Value::as_str);
    let scenario_snapshot = resolve_standard_scenario_snapshot(
        row.get::<_, String>(2).as_str(),
        row.get::<_, String>(8).as_str(),
        row.get::<_, String>(9).as_str(),
        scenario_code_hint,
        &metadata,
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
    let snapshot = OrderPriceSnapshot {
        product_id: row.get(1),
        sku_id: row.get(2),
        sku_code: row.get(8),
        sku_type: row.get(9),
        pricing_mode: pricing_mode.clone(),
        unit_price: row.get(5),
        currency_code: row.get(6),
        billing_mode: billing_mode.clone(),
        refund_mode: row.get(11),
        settlement_terms: SettlementTermsSnapshot {
            settlement_basis: derive_settlement_basis(&billing_mode, &pricing_mode),
            settlement_mode: "manual_v1".to_string(),
        },
        tax_terms,
        scenario_snapshot: Some(scenario_snapshot),
        captured_at: kernel::UtcTimestampMs::now().0.to_string(),
        source: "catalog.product + catalog.product_sku".to_string(),
    };
    let snapshot_json = serde_json::to_value(&snapshot).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("snapshot serialization failed: {err}"),
                request_id: None,
            }),
        )
    })?;

    let _ = client
        .execute(
            "UPDATE trade.order_main
             SET
               price_snapshot_json = $2::jsonb,
               fee_preview_snapshot = fee_preview_snapshot || jsonb_build_object(
                 'pricing_mode', $3::text,
                 'unit_price', $4::text,
                 'currency_code', $5::text,
                 'captured_at', $6::text
               ),
               updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &snapshot_json,
                &snapshot.pricing_mode,
                &snapshot.unit_price,
                &snapshot.currency_code,
                &snapshot.captured_at,
            ],
        )
        .await
        .map_err(map_db_error)?;

    Ok(Some(snapshot))
}

fn parse_tax_terms(metadata: &Value) -> TaxTermsSnapshot {
    let tax = metadata
        .get("tax")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    let tax_policy = tax
        .get("policy")
        .and_then(Value::as_str)
        .unwrap_or("platform_default")
        .to_string();
    let tax_code = tax
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("UNSPECIFIED")
        .to_string();
    let tax_inclusive = tax
        .get("inclusive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    TaxTermsSnapshot {
        tax_policy,
        tax_code,
        tax_inclusive,
    }
}

fn map_db_error(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("trade persistence failed: {err}"),
            request_id: None,
        }),
    )
}
