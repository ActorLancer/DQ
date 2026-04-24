use crate::modules::delivery::dto::{
    QueryRunAuditReferenceData, QueryRunListResponseData, QueryRunResponseData,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Value, json};

const DELIVERY_TEMPLATE_QUERY_RUN_READ_EVENT: &str = "delivery.template_query.run.read";

pub async fn get_query_runs(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<QueryRunListResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_order_context(&tx, order_id, request_id).await?;

    enforce_subject_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_read_context(&context, request_id)?;

    let rows = tx
        .query(
            "SELECT r.query_run_id::text,
                    r.template_query_grant_id::text,
                    r.query_surface_id::text,
                    r.query_template_id::text,
                    t.template_name,
                    t.version_no,
                    r.requester_user_id::text,
                    r.execution_mode,
                    COALESCE(r.request_payload_json, '{}'::jsonb),
                    COALESCE(r.result_summary_json, '{}'::jsonb),
                    r.result_object_id::text,
                    so.object_uri,
                    COALESCE(r.result_row_count, 0),
                    COALESCE(r.billed_units, 0)::text,
                    r.export_attempt_count,
                    r.status,
                    r.masked_level,
                    r.export_scope,
                    r.approval_ticket_id::text,
                    COALESCE(r.sensitive_policy_snapshot, '{}'::jsonb),
                    to_char(r.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(r.started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(r.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.query_execution_run r
             LEFT JOIN delivery.query_template_definition t
               ON t.query_template_id = r.query_template_id
             LEFT JOIN delivery.storage_object so
               ON so.object_id = r.result_object_id
             WHERE r.order_id = $1::text::uuid
             ORDER BY r.created_at DESC, r.query_run_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let mut query_runs = Vec::with_capacity(rows.len());
    let mut query_run_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let query_run_id: String = row.get(0);
        let result_object_uri: Option<String> = row.get(11);
        let resolved_location = result_object_uri
            .as_deref()
            .map(|uri| resolve_storage_object_location(uri, None))
            .unwrap_or_default();
        let request_payload_json: Value = row.get(8);
        let result_summary_json: Value = row.get(9);
        let parameter_summary_json = request_payload_json
            .get("parameter_summary")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let (audit_refs, policy_hits) = load_audit_refs(&tx, &query_run_id).await?;
        let policy_hits = if policy_hits.is_empty() {
            extract_string_array(result_summary_json.get("policy_hits"))
        } else {
            policy_hits
        };

        query_run_ids.push(query_run_id.clone());
        query_runs.push(QueryRunResponseData {
            query_run_id,
            order_id: order_id.to_string(),
            template_query_grant_id: row.get(1),
            query_surface_id: row.get(2),
            query_template_id: row.get(3),
            query_template_name: row
                .get::<_, Option<String>>(4)
                .unwrap_or_else(|| "unknown".to_string()),
            query_template_version: row.get::<_, Option<i32>>(5).unwrap_or_default(),
            requester_user_id: row.get(6),
            execution_mode: row.get(7),
            request_payload_json,
            parameter_summary_json,
            result_summary_json,
            result_object_id: row.get(10),
            result_object_uri,
            bucket_name: resolved_location.bucket_name,
            object_key: resolved_location.object_key,
            result_row_count: row.get(12),
            billed_units: row.get(13),
            export_attempt_count: row.get(14),
            status: row.get(15),
            masked_level: row.get(16),
            export_scope: row.get(17),
            approval_ticket_id: row.get(18),
            sensitive_policy_snapshot: row.get(19),
            policy_hits,
            audit_refs,
            operation: "read".to_string(),
            current_state: context.current_state.clone(),
            payment_status: context.payment_status.clone(),
            delivery_status: context.delivery_status.clone(),
            created_at: row.get(20),
            started_at: row.get(21),
            completed_at: row.get(22),
        });
    }

    write_delivery_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        DELIVERY_TEMPLATE_QUERY_RUN_READ_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "query_run_ids": query_run_ids,
            "query_run_count": query_runs.len(),
            "current_state": context.current_state,
            "payment_status": context.payment_status,
            "delivery_status": context.delivery_status,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(QueryRunListResponseData {
        order_id: order_id.to_string(),
        current_state: context.current_state,
        payment_status: context.payment_status,
        delivery_status: context.delivery_status,
        query_runs,
    })
}

#[derive(Debug)]
struct QueryRunOrderContext {
    buyer_org_id: String,
    seller_org_id: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
    sku_type: String,
    buyer_status: String,
    buyer_metadata: Value,
    seller_status: String,
    seller_metadata: Value,
}

async fn load_order_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<QueryRunOrderContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.buyer_org_id::text,
                    o.seller_org_id::text,
                    o.status,
                    o.payment_status,
                    o.delivery_status,
                    s.sku_type,
                    buyer.status,
                    buyer.metadata,
                    seller.status,
                    seller.metadata
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };
    Ok(QueryRunOrderContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        current_state: row.get(2),
        payment_status: row.get(3),
        delivery_status: row.get(4),
        sku_type: row.get(5),
        buyer_status: row.get(6),
        buyer_metadata: row.get(7),
        seller_status: row.get(8),
        seller_metadata: row.get(9),
    })
}

async fn load_audit_refs(
    client: &(impl GenericClient + Sync),
    query_run_id: &str,
) -> Result<(Vec<QueryRunAuditReferenceData>, Vec<String>), (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT audit_id::text,
                    action_name,
                    result_code,
                    request_id,
                    trace_id,
                    to_char(event_time AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    COALESCE(metadata, '{}'::jsonb)
             FROM audit.audit_event
             WHERE ref_type = 'query_execution_run'
               AND ref_id = $1::text::uuid
             ORDER BY event_time DESC, audit_id DESC",
            &[&query_run_id],
        )
        .await
        .map_err(map_db_error)?;

    let mut audit_refs = Vec::with_capacity(rows.len());
    let mut policy_hits = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let metadata: Value = row.get(6);
        if index == 0 {
            policy_hits = extract_string_array(metadata.get("policy_hits"));
        }
        audit_refs.push(QueryRunAuditReferenceData {
            audit_id: row.get(0),
            action_name: row.get(1),
            result_code: row.get(2),
            request_id: row.get(3),
            trace_id: row.get(4),
            event_time: row.get(5),
        });
    }
    Ok((audit_refs, policy_hits))
}

fn enforce_subject_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(
        actor_role,
        "buyer_operator" | "tenant_developer" | "business_analyst"
    ) && tenant_id == Some(buyer_org_id)
    {
        return Ok(());
    }
    if matches!(actor_role, "tenant_admin") && tenant_id == Some(buyer_org_id) {
        return Ok(());
    }
    if matches!(
        actor_role,
        "seller_operator" | "seller_storage_operator" | "sandbox_operator"
    ) && tenant_id == Some(seller_org_id)
    {
        return Err(conflict(
            "QUERY_RUN_READ_FORBIDDEN: seller-side roles cannot read buyer query runs",
            request_id,
        ));
    }
    Err(forbidden(
        "template query run read is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_read_context(
    context: &QueryRunOrderContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type != "QRY_LITE" {
        return Err(conflict(
            &format!(
                "QUERY_RUN_READ_FORBIDDEN: order sku_type `{}` is not QRY_LITE",
                context.sku_type
            ),
            request_id,
        ));
    }
    if context.buyer_status != "active" || context.seller_status != "active" {
        return Err(conflict(
            "QUERY_RUN_READ_FORBIDDEN: buyer/seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_readable(&context.buyer_metadata)
        || !is_subject_readable(&context.seller_metadata)
    {
        return Err(conflict(
            "QUERY_RUN_READ_FORBIDDEN: buyer/seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    Ok(())
}

fn is_subject_readable(metadata: &Value) -> bool {
    metadata
        .get("risk_status")
        .and_then(Value::as_str)
        .map(|status| !matches!(status.trim(), "blocked" | "frozen" | "risk_hold"))
        .unwrap_or(true)
        && metadata
            .get("risk_flags")
            .and_then(Value::as_array)
            .map(|flags| {
                !flags.iter().any(|flag| {
                    matches!(
                        flag.as_str().unwrap_or_default(),
                        "blocked" | "suspended" | "frozen" | "risk_hold"
                    )
                })
            })
            .unwrap_or(true)
}

fn extract_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: "QUERY_RUN_READ_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
