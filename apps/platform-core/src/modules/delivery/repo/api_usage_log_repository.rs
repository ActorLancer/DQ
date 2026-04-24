use crate::modules::delivery::dto::{
    ApiUsageLogAppData, ApiUsageLogEntryData, ApiUsageLogListResponseData, ApiUsageLogSummaryData,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::ErrorResponse;
use serde_json::{Value, json};

const DELIVERY_API_LOG_READ_EVENT: &str = "delivery.api.log.read";

pub async fn get_api_usage_log(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ApiUsageLogListResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_usage_context(&tx, order_id, request_id).await?;

    enforce_subject_status(&context, request_id)?;
    enforce_read_scope(actor_role, tenant_id, &context.buyer_org_id, request_id)?;
    enforce_api_sku(&context, request_id)?;

    let app_binding =
        load_latest_api_binding(&tx, order_id, &context.buyer_org_id, request_id).await?;
    let summary = load_usage_summary(&tx, order_id, &context.buyer_org_id).await?;
    let rows = tx
        .query(
            "SELECT l.api_usage_log_id::text,
                    l.api_credential_id::text,
                    l.request_id,
                    l.response_code,
                    l.usage_units::text,
                    to_char(l.occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.api_usage_log l
             JOIN core.application app ON app.app_id = l.app_id
             WHERE l.order_id = $1::text::uuid
               AND app.org_id = $2::text::uuid
             ORDER BY l.occurred_at DESC, l.api_usage_log_id DESC",
            &[&order_id, &context.buyer_org_id],
        )
        .await
        .map_err(map_db_error)?;

    let logs = rows.iter().map(map_usage_row).collect::<Vec<_>>();

    write_delivery_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        DELIVERY_API_LOG_READ_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "buyer_org_id": context.buyer_org_id,
            "app_id": app_binding.app_id,
            "api_credential_id": app_binding.api_credential_id,
            "log_count": logs.len(),
            "has_logs": !logs.is_empty(),
            "minimal_disclosure": true,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(ApiUsageLogListResponseData {
        order_id: order_id.to_string(),
        sku_id: context.sku_id,
        sku_type: context.sku_type,
        current_state: context.current_state,
        payment_status: context.payment_status,
        app: app_binding,
        summary,
        logs,
    })
}

struct ApiUsageContext {
    buyer_org_id: String,
    buyer_org_status: String,
    seller_org_status: String,
    buyer_org_metadata: Value,
    seller_org_metadata: Value,
    sku_id: String,
    sku_type: String,
    current_state: String,
    payment_status: String,
}

async fn load_usage_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<ApiUsageContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.buyer_org_id::text,
                    o.seller_org_id::text,
                    buyer.status,
                    seller.status,
                    buyer.metadata,
                    seller.metadata,
                    sku.sku_id::text,
                    sku.sku_type,
                    o.status,
                    o.payment_status
             FROM trade.order_main o
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    Ok(ApiUsageContext {
        buyer_org_id: row.get(0),
        buyer_org_status: row.get(2),
        seller_org_status: row.get(3),
        buyer_org_metadata: row.get(4),
        seller_org_metadata: row.get(5),
        sku_id: row.get(6),
        sku_type: row.get(7),
        current_state: row.get(8),
        payment_status: row.get(9),
    })
}

fn enforce_subject_status(
    context: &ApiUsageContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.buyer_org_status != "active" || context.seller_org_status != "active" {
        return Err(conflict(
            "API_USAGE_LOG_FORBIDDEN: buyer/seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&context.buyer_org_metadata)
        || !is_subject_deliverable(&context.seller_org_metadata)
    {
        return Err(conflict(
            "API_USAGE_LOG_FORBIDDEN: buyer/seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_read_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(
        actor_role,
        "buyer_operator"
            | "procurement_manager"
            | "tenant_developer"
            | "tenant_audit_readonly"
            | "tenant_admin"
    ) && tenant_id == Some(buyer_org_id)
    {
        return Ok(());
    }
    Err(forbidden(
        "api usage log read is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_api_sku(
    context: &ApiUsageContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(context.sku_type.as_str(), "API_SUB" | "API_PPU") {
        return Ok(());
    }
    Err(conflict(
        &format!(
            "API_USAGE_LOG_FORBIDDEN: order sku_type `{}` is not API_SUB/API_PPU",
            context.sku_type
        ),
        request_id,
    ))
}

async fn load_latest_api_binding(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<ApiUsageLogAppData, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT ac.api_credential_id::text,
                    app.app_id::text,
                    app.app_name,
                    app.client_id,
                    ac.status,
                    ac.upstream_mode
             FROM delivery.api_credential ac
             JOIN core.application app ON app.app_id = ac.app_id
             WHERE ac.order_id = $1::text::uuid
               AND app.org_id = $2::text::uuid
             ORDER BY ac.created_at DESC, ac.api_credential_id DESC
             LIMIT 1",
            &[&order_id, &buyer_org_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(conflict(
            "API_USAGE_LOG_FORBIDDEN: api delivery is not enabled for current order",
            request_id,
        ));
    };

    Ok(ApiUsageLogAppData {
        app_id: row.get(1),
        app_name: row.get(2),
        client_id: row.get(3),
        api_credential_id: row.get(0),
        credential_status: row.get(4),
        upstream_mode: row.get(5),
    })
}

async fn load_usage_summary(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    buyer_org_id: &str,
) -> Result<ApiUsageLogSummaryData, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_one(
            "SELECT COUNT(*)::bigint,
                    COUNT(*) FILTER (
                      WHERE COALESCE(response_code, 0) >= 200
                        AND COALESCE(response_code, 0) < 400
                    )::bigint,
                    COUNT(*) FILTER (
                      WHERE response_code IS NULL
                         OR response_code < 200
                         OR response_code >= 400
                    )::bigint,
                    COALESCE(SUM(usage_units), 0)::text,
                    to_char(MAX(occurred_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.api_usage_log l
             JOIN core.application app ON app.app_id = l.app_id
             WHERE l.order_id = $1::text::uuid
               AND app.org_id = $2::text::uuid",
            &[&order_id, &buyer_org_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(ApiUsageLogSummaryData {
        total_calls: row.get(0),
        successful_calls: row.get(1),
        failed_calls: row.get(2),
        total_usage_units: row.get(3),
        last_occurred_at: row.get(4),
    })
}

fn map_usage_row(row: &Row) -> ApiUsageLogEntryData {
    let request_id = row.get::<_, Option<String>>(2);
    let response_code = row.get::<_, Option<i32>>(3);
    ApiUsageLogEntryData {
        api_usage_log_id: row.get(0),
        api_credential_id: row.get(1),
        request_ref: request_id.as_deref().map(mask_request_ref),
        response_code,
        response_class: response_code.map(response_class),
        usage_units: row.get(4),
        occurred_at: row.get(5),
    }
}

fn response_class(response_code: i32) -> String {
    format!("{}xx", response_code / 100)
}

fn mask_request_ref(request_id: &str) -> String {
    let trimmed = request_id.trim();
    if trimmed.is_empty() {
        return "***".to_string();
    }
    let visible = 4usize.min(trimmed.len());
    format!("***{}", &trimmed[trimmed.len() - visible..])
}

fn is_subject_deliverable(metadata: &Value) -> bool {
    !matches!(
        metadata
            .get("subject_risk")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        "blocked" | "suspended" | "frozen"
    ) && !metadata
        .get("risk_flags")
        .and_then(Value::as_array)
        .is_some_and(|flags| {
            flags.iter().any(|flag| {
                matches!(
                    flag.as_str().unwrap_or_default(),
                    "blocked" | "suspended" | "frozen" | "risk_hold"
                )
            })
        })
}

fn forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: "API_USAGE_LOG_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
