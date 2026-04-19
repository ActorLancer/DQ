use crate::modules::order::repo::pre_request_repository::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

pub async fn ensure_pre_payment_lock_checks(
    client: &(impl tokio_postgres::GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               buyer.status,
               seller.status,
               p.status,
               v.status,
               s.status,
               p.metadata,
               o.price_snapshot_json
             FROM trade.order_main o
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.asset_version v ON v.asset_version_id = o.asset_version_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };

    let buyer_status: String = row.get(0);
    let seller_status: String = row.get(1);
    let product_status: String = row.get(2);
    let asset_version_status: String = row.get(3);
    let sku_status: String = row.get(4);
    let product_metadata: Value = row.get(5);
    let snapshot: Value = row.get(6);

    if buyer_status != "active" || seller_status != "active" {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: buyer/seller organization is not active",
            request_id,
        ));
    }
    if product_status != "listed" {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: product status is not listed",
            request_id,
        ));
    }
    if !matches!(asset_version_status.as_str(), "active" | "published") {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: asset version status is not active/published",
            request_id,
        ));
    }
    if !matches!(sku_status.as_str(), "active" | "listed") {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: sku status is not purchasable",
            request_id,
        ));
    }
    if !is_review_status_approved(&product_metadata) {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: product review status is not approved",
            request_id,
        ));
    }
    if is_risk_blocked(&product_metadata) {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: product is blocked by risk policy",
            request_id,
        ));
    }
    if !is_price_snapshot_complete(&snapshot) {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: price snapshot is incomplete",
            request_id,
        ));
    }
    if !is_template_snapshot_complete(&snapshot) {
        return Err(conflict(
            "ORDER_PRE_LOCK_CHECK_FAILED: template snapshot is incomplete",
            request_id,
        ));
    }

    Ok(())
}

fn is_price_snapshot_complete(snapshot: &Value) -> bool {
    let required = [
        "product_id",
        "sku_id",
        "sku_code",
        "sku_type",
        "pricing_mode",
        "unit_price",
        "currency_code",
        "billing_mode",
        "refund_mode",
        "captured_at",
        "source",
    ];
    required
        .iter()
        .all(|key| snapshot.get(*key).is_some_and(|v| !v.is_null()))
}

fn is_template_snapshot_complete(snapshot: &Value) -> bool {
    let settlement_ok = snapshot
        .get("settlement_terms")
        .and_then(Value::as_object)
        .is_some_and(|obj| {
            obj.get("settlement_basis").is_some_and(|v| !v.is_null())
                && obj.get("settlement_mode").is_some_and(|v| !v.is_null())
        });
    let tax_ok = snapshot
        .get("tax_terms")
        .and_then(Value::as_object)
        .is_some_and(|obj| {
            obj.get("tax_policy").is_some_and(|v| !v.is_null())
                && obj.get("tax_code").is_some_and(|v| !v.is_null())
                && obj.get("tax_inclusive").is_some_and(|v| !v.is_null())
        });
    settlement_ok && tax_ok
}

fn is_review_status_approved(product_metadata: &Value) -> bool {
    let Some(status) = product_metadata
        .get("review_status")
        .and_then(Value::as_str)
    else {
        return true;
    };
    matches!(status, "approved" | "auto_approved" | "passed")
}

fn is_risk_blocked(product_metadata: &Value) -> bool {
    let risk_blocked = product_metadata
        .get("risk_blocked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let block_submit = product_metadata
        .get("risk_flags")
        .and_then(Value::as_object)
        .and_then(|flags| flags.get("block_submit"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    risk_blocked || block_submit
}

fn conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
