use crate::modules::delivery::domain::merge_snapshot_patch;
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

pub struct PreparedDeliveryRecord {
    pub delivery_id: String,
    pub current_status: String,
    pub created: bool,
}

pub struct PreparedDeliveryOptions<'a> {
    pub creation_source: &'a str,
    pub executor_type: &'a str,
    pub executor_ref_id: Option<&'a str>,
    pub responsible_scope: &'a str,
    pub audit_action_name: &'a str,
}

pub async fn ensure_order_deliverable_and_prepare_delivery(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<PreparedDeliveryRecord, (StatusCode, Json<ErrorResponse>)> {
    ensure_order_deliverable_and_prepare_delivery_with_options(
        client,
        order_id,
        actor_role,
        request_id,
        trace_id,
        &PreparedDeliveryOptions {
            creation_source: "deliverability_gate",
            executor_type: "platform",
            executor_ref_id: None,
            responsible_scope: "platform_delivery",
            audit_action_name: "trade.order.delivery_gate.prepared",
        },
    )
    .await
}

pub async fn ensure_order_deliverable_and_prepare_delivery_with_options(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    options: &PreparedDeliveryOptions<'_>,
) -> Result<PreparedDeliveryRecord, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_status,
               o.trust_boundary_snapshot,
               o.delivery_route_snapshot,
               o.buyer_org_id::text,
               o.seller_org_id::text,
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

    let order_status: String = row.get(0);
    let payment_status: String = row.get(1);
    let delivery_status: String = row.get(2);
    let trust_boundary_snapshot: Value = row.get(3);
    let delivery_route_snapshot: Option<String> = row.get(4);
    let buyer_org_id: String = row.get(5);
    let seller_org_id: String = row.get(6);
    let buyer_status: String = row.get(7);
    let buyer_metadata: Value = row.get(8);
    let seller_status: String = row.get(9);
    let seller_metadata: Value = row.get(10);
    let product_status: String = row.get(11);
    let product_metadata: Value = row.get(12);
    let asset_version_status: String = row.get(13);
    let sku_status: String = row.get(14);
    let sku_type: String = row.get(15);
    let contract_id: Option<String> = row.get(16);
    let contract_status: Option<String> = row.get(17);

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
        .map(|existing| existing.get::<_, String>(0))
    {
        write_trade_audit_event(
            client,
            "order",
            order_id,
            actor_role,
            options.audit_action_name,
            "already_exists",
            request_id,
            trace_id,
        )
        .await?;
        return Ok(PreparedDeliveryRecord {
            delivery_id: existing_id,
            current_status: "prepared".to_string(),
            created: false,
        });
    }

    if delivery_status != "pending_delivery" {
        if let Some(existing_id) = client
            .query_opt(
                "SELECT delivery_id::text
                 FROM delivery.delivery_record
                 WHERE order_id = $1::text::uuid
                   AND status = 'committed'
                 ORDER BY committed_at DESC NULLS LAST, updated_at DESC, delivery_id DESC
                 LIMIT 1",
                &[&order_id],
            )
            .await
            .map_err(map_db_error)?
            .map(|existing| existing.get::<_, String>(0))
        {
            write_trade_audit_event(
                client,
                "order",
                order_id,
                actor_role,
                options.audit_action_name,
                "already_exists",
                request_id,
                trace_id,
            )
            .await?;
            return Ok(PreparedDeliveryRecord {
                delivery_id: existing_id,
                current_status: "committed".to_string(),
                created: false,
            });
        }
    }

    let (delivery_type, default_route) = default_delivery_shape(&sku_type);
    let delivery_trust_boundary_snapshot = merge_snapshot_patch(
        &trust_boundary_snapshot,
        &build_delivery_task_snapshot(
            &order_status,
            options.creation_source,
            options.executor_type,
            options.executor_ref_id,
            options.responsible_scope,
            resolve_responsible_subject_id(
                options.executor_type,
                &buyer_org_id,
                &seller_org_id,
                options.executor_ref_id,
            ),
        ),
    );
    let delivery_id: String = client
        .query_one(
            "INSERT INTO delivery.delivery_record (
               order_id,
               delivery_type,
               delivery_route,
               executor_type,
               executor_ref_id,
               status,
               trust_boundary_snapshot
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4,
               $5::text::uuid,
               'prepared',
               $6::jsonb
             )
             RETURNING delivery_id::text",
            &[
                &order_id,
                &delivery_type,
                &delivery_route_snapshot.as_deref().unwrap_or(default_route),
                &options.executor_type,
                &options.executor_ref_id,
                &delivery_trust_boundary_snapshot,
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
        options.audit_action_name,
        "success",
        request_id,
        trace_id,
    )
    .await?;

    Ok(PreparedDeliveryRecord {
        delivery_id,
        current_status: "prepared".to_string(),
        created: true,
    })
}

fn build_delivery_task_snapshot(
    order_status: &str,
    creation_source: &str,
    executor_type: &str,
    executor_ref_id: Option<&str>,
    responsible_scope: &str,
    responsible_subject_id: Option<String>,
) -> Value {
    let responsible_subject_type = responsible_subject_id.as_ref().map(|_| "organization");
    json!({
        "delivery_task": {
            "auto_created": true,
            "creation_source": creation_source,
            "origin_order_state": order_status,
            "retry_count": 0,
            "manual_takeover": false,
            "responsible_scope": responsible_scope,
            "responsible_subject_type": responsible_subject_type,
            "responsible_subject_id": responsible_subject_id,
            "executor_type": executor_type,
            "executor_ref_id": executor_ref_id,
        }
    })
}

fn resolve_responsible_subject_id(
    executor_type: &str,
    buyer_org_id: &str,
    seller_org_id: &str,
    executor_ref_id: Option<&str>,
) -> Option<String> {
    match executor_type {
        "buyer_org" => Some(executor_ref_id.unwrap_or(buyer_org_id).to_string()),
        "seller_org" => Some(executor_ref_id.unwrap_or(seller_org_id).to_string()),
        _ => executor_ref_id.map(str::to_string),
    }
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
            code: "ORDER_DELIVERABILITY_CHECK_FAILED".to_string(),
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
    use super::{
        build_delivery_task_snapshot, default_delivery_shape, is_subject_deliverable,
        resolve_responsible_subject_id,
    };
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

    #[test]
    fn resolves_responsible_subject_by_executor() {
        assert_eq!(
            resolve_responsible_subject_id("buyer_org", "buyer-1", "seller-1", Some("buyer-2")),
            Some("buyer-2".to_string())
        );
        assert_eq!(
            resolve_responsible_subject_id("seller_org", "buyer-1", "seller-1", None),
            Some("seller-1".to_string())
        );
    }

    #[test]
    fn builds_delivery_task_snapshot_with_required_flags() {
        let snapshot = build_delivery_task_snapshot(
            "buyer_locked",
            "payment_webhook",
            "seller_org",
            Some("seller-1"),
            "seller_delivery_operator",
            Some("seller-1".to_string()),
        );
        assert_eq!(
            snapshot["delivery_task"]["creation_source"].as_str(),
            Some("payment_webhook")
        );
        assert_eq!(snapshot["delivery_task"]["retry_count"].as_i64(), Some(0));
        assert_eq!(
            snapshot["delivery_task"]["manual_takeover"].as_bool(),
            Some(false)
        );
        assert_eq!(
            snapshot["delivery_task"]["responsible_subject_id"].as_str(),
            Some("seller-1")
        );
    }
}
