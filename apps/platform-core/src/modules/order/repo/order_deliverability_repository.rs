use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

pub struct PreparedDeliveryRecord {
    pub delivery_id: String,
}

pub async fn ensure_order_deliverable_and_prepare_delivery(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<PreparedDeliveryRecord, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.payment_status,
               o.trust_boundary_snapshot,
               o.delivery_route_snapshot,
               buyer.status,
               buyer.metadata,
               seller.status,
               seller.metadata,
               p.status,
               p.metadata,
               v.status,
               s.status,
               s.sku_type,
               dc.contract_id::text,
               dc.status
             FROM trade.order_main o
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.asset_version v ON v.asset_version_id = o.asset_version_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN contract.digital_contract dc ON dc.order_id = o.order_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    let payment_status: String = row.get(0);
    let trust_boundary_snapshot: Value = row.get(1);
    let delivery_route_snapshot: Option<String> = row.get(2);
    let buyer_status: String = row.get(3);
    let buyer_metadata: Value = row.get(4);
    let seller_status: String = row.get(5);
    let seller_metadata: Value = row.get(6);
    let product_status: String = row.get(7);
    let product_metadata: Value = row.get(8);
    let asset_version_status: String = row.get(9);
    let sku_status: String = row.get(10);
    let sku_type: String = row.get(11);
    let contract_id: Option<String> = row.get(12);
    let contract_status: Option<String> = row.get(13);

    if payment_status != "paid" {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: payment status is not paid",
            request_id,
        ));
    }
    if buyer_status != "active" || seller_status != "active" {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: buyer/seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&buyer_metadata) || !is_subject_deliverable(&seller_metadata) {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: buyer/seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    if product_status != "listed" {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: product status is not listed",
            request_id,
        ));
    }
    if !matches!(asset_version_status.as_str(), "active" | "published") {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: asset version status is not active/published",
            request_id,
        ));
    }
    if !matches!(sku_status.as_str(), "active" | "listed") {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: sku status is not deliverable",
            request_id,
        ));
    }
    if !is_review_status_approved(&product_metadata) {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: product review status is not approved",
            request_id,
        ));
    }
    if is_product_risk_blocked(&product_metadata) {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: product is blocked by risk policy",
            request_id,
        ));
    }
    if contract_id.is_none() || contract_status.as_deref() != Some("signed") {
        return Err(conflict(
            "ORDER_DELIVERABILITY_CHECK_FAILED: contract is not signed/effective",
            request_id,
        ));
    }

    if let Some(existing_id) = client
        .query_opt(
            "SELECT delivery_id::text
             FROM delivery.delivery_record
             WHERE order_id = $1::text::uuid
               AND status = 'prepared'
             ORDER BY created_at DESC, delivery_id DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?
        .map(|row| row.get::<_, String>(0))
    {
        return Ok(PreparedDeliveryRecord {
            delivery_id: existing_id,
        });
    }

    let (delivery_type, default_route) = default_delivery_shape(&sku_type);
    let delivery_id: String = client
        .query_one(
            "INSERT INTO delivery.delivery_record (
               order_id,
               delivery_type,
               delivery_route,
               status,
               trust_boundary_snapshot
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               'prepared',
               $4::jsonb
             )
             RETURNING delivery_id::text",
            &[
                &order_id,
                &delivery_type,
                &delivery_route_snapshot.as_deref().unwrap_or(default_route),
                &trust_boundary_snapshot,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    write_trade_audit_event(
        client,
        "order",
        order_id,
        actor_role,
        "trade.order.delivery_gate.prepared",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    Ok(PreparedDeliveryRecord { delivery_id })
}

fn default_delivery_shape(sku_type: &str) -> (&'static str, &'static str) {
    match sku_type {
        "FILE_STD" => ("file_download", "signed_url"),
        "FILE_SUB" => ("revision_push", "revision_event"),
        "API_SUB" | "API_PPU" => ("api_access", "api_gateway"),
        "SHARE_RO" => ("share_grant", "share_link"),
        "QRY_LITE" => ("template_grant", "template_query"),
        "SBX_STD" => ("sandbox_workspace", "sandbox_portal"),
        "RPT_STD" => ("report_delivery", "result_package"),
        _ => ("delivery", "platform_delivery"),
    }
}

fn is_subject_deliverable(metadata: &Value) -> bool {
    let risk_status = metadata
        .get("risk_status")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase());
    if matches!(
        risk_status.as_deref(),
        Some("blocked" | "frozen" | "high" | "high_risk" | "deny")
    ) {
        return false;
    }

    let sellable_status = metadata
        .get("sellable_status")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase());
    if matches!(
        sellable_status.as_deref(),
        Some("blocked" | "disabled" | "frozen" | "suspended")
    ) {
        return false;
    }

    let freeze_reason = metadata
        .get("freeze_reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    freeze_reason.is_empty()
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

fn is_product_risk_blocked(product_metadata: &Value) -> bool {
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

fn not_found(order_id: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("order not found: {order_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{default_delivery_shape, is_subject_deliverable};
    use serde_json::json;

    #[test]
    fn maps_standard_sku_to_expected_delivery_shape() {
        assert_eq!(
            default_delivery_shape("FILE_STD"),
            ("file_download", "signed_url")
        );
        assert_eq!(
            default_delivery_shape("QRY_LITE"),
            ("template_grant", "template_query")
        );
    }

    #[test]
    fn subject_metadata_with_freeze_or_block_is_not_deliverable() {
        assert!(!is_subject_deliverable(&json!({
            "risk_status": "blocked"
        })));
        assert!(!is_subject_deliverable(&json!({
            "sellable_status": "frozen"
        })));
        assert!(!is_subject_deliverable(&json!({
            "freeze_reason": "manual_hold"
        })));
        assert!(is_subject_deliverable(&json!({
            "risk_status": "normal",
            "sellable_status": "enabled"
        })));
    }
}
