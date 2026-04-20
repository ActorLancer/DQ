use super::outbox_repository::write_billing_trigger_bridge_event;
use crate::modules::delivery::dto::{ExecuteTemplateRunRequest, QueryRunResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::{map_db_error, write_trade_audit_event};
use crate::modules::storage::application::put_object_bytes;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

const DELIVERY_TEMPLATE_QUERY_USE_EVENT: &str = "delivery.template_query.use";
const RESULT_BUCKET_ENV: &str = "BUCKET_REPORT_RESULTS";
const DEFAULT_RESULT_BUCKET: &str = "report-results";

pub async fn execute_template_run(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ExecuteTemplateRunRequest,
    actor_role: &str,
    header_user_id: Option<&str>,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<QueryRunResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_template_run_context(&tx, order_id, request_id).await?;

    enforce_subject_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_run_context(&context, request_id)?;

    let grant = load_template_query_grant(
        &tx,
        order_id,
        payload.template_query_grant_id.as_deref(),
        request_id,
    )
    .await?;
    let template = load_query_template(
        &tx,
        &grant.query_surface_id,
        payload.query_template_id.trim(),
        request_id,
    )
    .await?;
    enforce_template_allowed(&grant, &template, request_id)?;

    let requester_user_id = resolve_requester_user_id(
        &tx,
        payload.requester_user_id.as_deref().or(header_user_id),
        &context.buyer_org_id,
        request_id,
    )
    .await?;

    let request_payload_json = normalize_object(
        Some(payload.request_payload_json.clone()),
        "request_payload_json",
        request_id,
    )?;
    validate_parameter_schema(
        &template.parameter_schema_json,
        &request_payload_json,
        request_id,
    )?;
    let output_boundary_json = resolve_run_output_boundary(
        payload.output_boundary_json.clone(),
        &grant.output_boundary_json,
        &template.export_policy_json,
        request_id,
    )?;
    let masked_level = resolve_masked_level(payload.masked_level.as_deref(), request_id)?;
    let export_scope = resolve_export_scope(payload.export_scope.as_deref(), request_id)?;
    validate_template_risk_guard(
        &template.analysis_rule_json,
        &template.risk_guard_json,
        &request_payload_json,
        payload.approval_ticket_id.as_deref(),
        &masked_level,
        &export_scope,
        request_id,
    )?;
    enforce_run_quota(
        &tx,
        &grant.template_query_grant_id,
        order_id,
        &grant.run_quota_json,
        request_id,
    )
    .await?;

    let selected_fields = resolve_result_fields(
        &template.result_schema_json,
        &template.analysis_rule_json,
        &template.export_policy_json,
    );
    let row_count =
        derive_result_row_count(&request_payload_json, &output_boundary_json, request_id)?;
    enforce_max_cells(
        &output_boundary_json,
        row_count,
        selected_fields.len(),
        request_id,
    )?;
    let selected_format = resolve_selected_format(&output_boundary_json)?;
    let billed_units = format!("{:.8}", 1.0f64);
    let request_summary = build_request_payload_record(
        &request_payload_json,
        &output_boundary_json,
        payload.execution_metadata_json.clone(),
    );
    let parameter_summary_json = request_summary
        .get("parameter_summary")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let sensitive_policy_snapshot = build_sensitive_policy_snapshot(
        &masked_level,
        &export_scope,
        &output_boundary_json,
        &template.risk_guard_json,
        payload.approval_ticket_id.as_deref(),
    );

    let run_row = tx
        .query_one(
            "INSERT INTO delivery.query_execution_run (
               order_id,
               template_query_grant_id,
               query_template_id,
               query_surface_id,
               requester_user_id,
               execution_mode,
               request_payload_json,
               billed_units,
               status,
               started_at,
               masked_level,
               export_scope,
               approval_ticket_id,
               sensitive_policy_snapshot
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4::text::uuid,
               $5::text::uuid,
               'template_query',
               $6::jsonb,
               $7::numeric,
               'running',
               now(),
               $8,
               $9,
               $10::text::uuid,
               $11::jsonb
             )
             RETURNING query_run_id::text,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &grant.template_query_grant_id,
                &template.query_template_id,
                &grant.query_surface_id,
                &requester_user_id,
                &request_summary,
                &billed_units,
                &masked_level,
                &export_scope,
                &payload.approval_ticket_id,
                &sensitive_policy_snapshot,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let query_run_id: String = run_row.get(0);
    let created_at: String = run_row.get(1);
    let started_at: Option<String> = run_row.get(2);

    let bucket_name = default_result_bucket_name();
    let object_extension = if selected_format == "csv" {
        "csv"
    } else {
        "json"
    };
    let object_key = format!("query-runs/{order_id}/{query_run_id}/result.{object_extension}");
    let object_uri = format!("s3://{bucket_name}/{object_key}");
    let result_payload = build_result_payload(
        &query_run_id,
        order_id,
        &template,
        &selected_fields,
        row_count,
        &selected_format,
        &request_payload_json,
        &output_boundary_json,
        payload.execution_metadata_json.as_ref(),
    );
    let result_bytes = render_result_payload(&result_payload, &selected_fields, &selected_format)?;
    put_object_bytes(
        &bucket_name,
        &object_key,
        result_bytes.clone(),
        Some(content_type_for_format(&selected_format)),
    )
    .await?;

    let content_hash = build_result_hash(&query_run_id, row_count, selected_fields.len());
    let object_id: String = tx
        .query_one(
            "INSERT INTO delivery.storage_object (
               org_id,
               object_type,
               object_uri,
               location_type,
               managed_by_org_id,
               environment_id,
               content_type,
               size_bytes,
               content_hash,
               encryption_algo,
               plaintext_visible_to_platform,
               storage_zone,
               storage_class
             ) VALUES (
               $1::text::uuid,
               'result_object',
               $2,
               'platform_object_storage',
               $1::text::uuid,
               $3::text::uuid,
               $4,
               $5,
               $6,
               'managed_masked',
               true,
               'result',
               'standard'
             )
             RETURNING object_id::text",
            &[
                &context.seller_org_id,
                &object_uri,
                &grant.environment_id,
                &content_type_for_format(&selected_format),
                &(result_bytes.len() as i64),
                &content_hash,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    let result_summary_json = build_result_summary(
        &content_hash,
        &object_uri,
        &selected_fields,
        row_count,
        &selected_format,
        &masked_level,
        &export_scope,
        payload.approval_ticket_id.as_deref(),
        &request_payload_json,
    );

    let completed_row = tx
        .query_one(
            "UPDATE delivery.query_execution_run
             SET result_summary_json = $2::jsonb,
                 result_object_id = $3::text::uuid,
                 result_row_count = $4,
                 billed_units = $5::numeric,
                 export_attempt_count = 0,
                 status = 'completed',
                 completed_at = now(),
                 updated_at = now()
             WHERE query_run_id = $1::text::uuid
             RETURNING to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &query_run_id,
                &result_summary_json,
                &object_id,
                &row_count,
                &billed_units,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let completed_at: Option<String> = completed_row.get(0);

    let target_state = derive_target_state(&context.current_state);
    let delivery_status = derive_qry_lite_delivery_status(target_state, &context.payment_status);
    if target_state != context.current_state {
        tx.execute(
            "UPDATE trade.order_main
             SET status = $2,
                 payment_status = $3,
                 delivery_status = $4,
                 acceptance_status = $5,
                 settlement_status = $6,
                 dispute_status = $7,
                 last_reason_code = $8,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &target_state,
                &context.payment_status,
                &delivery_status,
                &derive_qry_lite_acceptance_status(target_state),
                &derive_qry_lite_settlement_status(target_state, &context.payment_status),
                &"none",
                &"qry_lite_query_executed",
            ],
        )
        .await
        .map_err(map_db_error)?;
        write_trade_audit_event(
            &tx,
            "order",
            order_id,
            actor_role,
            "trade.order.qry_lite.transition",
            "success",
            request_id,
            trace_id,
        )
        .await?;
    }

    write_delivery_audit_event(
        &tx,
        "query_execution_run",
        &query_run_id,
        actor_role,
        DELIVERY_TEMPLATE_QUERY_USE_EVENT,
        "completed",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "template_query_grant_id": grant.template_query_grant_id,
            "query_template_id": template.query_template_id,
            "query_surface_id": grant.query_surface_id,
            "selected_format": selected_format,
            "result_row_count": row_count,
            "policy_hits": ["template_whitelist_passed", "parameter_schema_passed", "output_boundary_passed", "risk_guard_passed"],
            "result_object_id": object_id,
            "result_object_uri": object_uri,
        }),
    )
    .await?;
    let billing_bridge_idempotency_key = format!("billing-trigger:query-run:{query_run_id}");
    write_billing_trigger_bridge_event(
        &tx,
        order_id,
        "execution_completed",
        "query_execution_run",
        &query_run_id,
        DELIVERY_TEMPLATE_QUERY_USE_EVENT,
        actor_role,
        request_id,
        trace_id,
        billing_bridge_idempotency_key.as_str(),
        json!({
            "delivery_branch": "query_run",
            "query_run_id": query_run_id,
            "query_surface_id": grant.query_surface_id,
            "query_template_id": template.query_template_id,
            "query_template_name": template.template_name,
            "query_template_version": template.version_no,
            "result_object_id": object_id,
            "result_object_uri": object_uri,
            "result_row_count": row_count,
            "billed_units": billed_units,
            "selected_format": selected_format,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(QueryRunResponseData {
        query_run_id,
        order_id: order_id.to_string(),
        template_query_grant_id: grant.template_query_grant_id,
        query_surface_id: grant.query_surface_id,
        query_template_id: template.query_template_id,
        query_template_name: template.template_name,
        query_template_version: template.version_no,
        requester_user_id,
        execution_mode: "template_query".to_string(),
        request_payload_json: request_summary,
        parameter_summary_json,
        result_summary_json,
        result_object_id: Some(object_id),
        result_object_uri: Some(object_uri),
        bucket_name: Some(bucket_name),
        object_key: Some(object_key),
        result_row_count: row_count,
        billed_units,
        export_attempt_count: 0,
        status: "completed".to_string(),
        masked_level,
        export_scope,
        approval_ticket_id: payload.approval_ticket_id.clone(),
        sensitive_policy_snapshot,
        policy_hits: vec![
            "template_whitelist_passed".to_string(),
            "parameter_schema_passed".to_string(),
            "output_boundary_passed".to_string(),
            "risk_guard_passed".to_string(),
        ],
        audit_refs: vec![],
        operation: "executed".to_string(),
        current_state: target_state.to_string(),
        payment_status: context.payment_status,
        delivery_status,
        created_at,
        started_at,
        completed_at,
    })
}

#[derive(Debug)]
struct TemplateRunContext {
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

#[derive(Debug)]
struct TemplateGrantContext {
    template_query_grant_id: String,
    query_surface_id: String,
    environment_id: Option<String>,
    template_type: String,
    allowed_template_ids: Vec<String>,
    output_boundary_json: Value,
    run_quota_json: Value,
    grant_status: String,
}

#[derive(Debug)]
struct QueryTemplateContext {
    query_template_id: String,
    template_name: String,
    template_type: String,
    version_no: i32,
    parameter_schema_json: Value,
    analysis_rule_json: Value,
    result_schema_json: Value,
    export_policy_json: Value,
    risk_guard_json: Value,
    status: String,
}

async fn load_template_run_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<TemplateRunContext, (StatusCode, Json<ErrorResponse>)> {
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
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };
    Ok(TemplateRunContext {
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

async fn load_template_query_grant(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    grant_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<TemplateGrantContext, (StatusCode, Json<ErrorResponse>)> {
    let row = if let Some(grant_id) = grant_id.map(str::trim).filter(|value| !value.is_empty()) {
        client
            .query_opt(
                "SELECT template_query_grant_id::text,
                        query_surface_id::text,
                        environment_id::text,
                        template_type,
                        allowed_template_ids,
                        output_boundary_json,
                        run_quota_json,
                        grant_status
                 FROM delivery.template_query_grant
                 WHERE template_query_grant_id = $1::text::uuid
                   AND order_id = $2::text::uuid",
                &[&grant_id, &order_id],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_opt(
                "SELECT template_query_grant_id::text,
                        query_surface_id::text,
                        environment_id::text,
                        template_type,
                        allowed_template_ids,
                        output_boundary_json,
                        run_quota_json,
                        grant_status
                 FROM delivery.template_query_grant
                 WHERE order_id = $1::text::uuid
                   AND grant_status = 'active'
                 ORDER BY updated_at DESC, template_query_grant_id DESC
                 LIMIT 1",
                &[&order_id],
            )
            .await
            .map_err(map_db_error)?
    };

    let Some(row) = row else {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: active template_query_grant not found for order",
            request_id,
        ));
    };
    let allowed_template_ids = row
        .get::<_, Option<Value>>(4)
        .and_then(parse_string_array)
        .unwrap_or_default();
    Ok(TemplateGrantContext {
        template_query_grant_id: row.get(0),
        query_surface_id: row.get(1),
        environment_id: row.get(2),
        template_type: row.get(3),
        allowed_template_ids,
        output_boundary_json: row.get(5),
        run_quota_json: row.get(6),
        grant_status: row.get(7),
    })
}

async fn load_query_template(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    query_template_id: &str,
    request_id: Option<&str>,
) -> Result<QueryTemplateContext, (StatusCode, Json<ErrorResponse>)> {
    if query_template_id.is_empty() {
        return Err(bad_request("query_template_id is required", request_id));
    }
    let row = client
        .query_opt(
            "SELECT query_template_id::text,
                    template_name,
                    template_type,
                    version_no,
                    parameter_schema_json,
                    analysis_rule_json,
                    result_schema_json,
                    export_policy_json,
                    risk_guard_json,
                    status
             FROM delivery.query_template_definition
             WHERE query_template_id = $1::text::uuid
               AND query_surface_id = $2::text::uuid",
            &[&query_template_id, &query_surface_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(query_template_id, request_id));
    };
    Ok(QueryTemplateContext {
        query_template_id: row.get(0),
        template_name: row.get(1),
        template_type: row.get(2),
        version_no: row.get(3),
        parameter_schema_json: row.get(4),
        analysis_rule_json: row.get(5),
        result_schema_json: row.get(6),
        export_policy_json: row.get(7),
        risk_guard_json: row.get(8),
        status: row.get(9),
    })
}

async fn resolve_requester_user_id(
    client: &(impl GenericClient + Sync),
    user_id: Option<&str>,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(user_id) = user_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let row = client
        .query_opt(
            "SELECT user_id::text, org_id::text, status
             FROM core.user_account
             WHERE user_id = $1::text::uuid",
            &[&user_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: requester_user_id does not exist",
            request_id,
        ));
    };
    let resolved_user_id: String = row.get(0);
    let org_id: String = row.get(1);
    let status: String = row.get(2);
    if org_id != buyer_org_id {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: requester_user_id is outside current buyer tenant",
            request_id,
        ));
    }
    if status != "active" {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: requester_user_id is not active",
            request_id,
        ));
    }
    Ok(Some(resolved_user_id))
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
            "TEMPLATE_RUN_FORBIDDEN: seller-side roles cannot execute buyer query runs",
            request_id,
        ));
    }
    Err(forbidden(
        "template query use is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_run_context(
    context: &TemplateRunContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type != "QRY_LITE" {
        return Err(conflict(
            &format!(
                "TEMPLATE_RUN_FORBIDDEN: order sku_type `{}` is not QRY_LITE",
                context.sku_type
            ),
            request_id,
        ));
    }
    if !matches!(
        context.current_state.as_str(),
        "template_authorized" | "params_validated" | "query_executed" | "result_available"
    ) {
        return Err(conflict(
            &format!(
                "TEMPLATE_RUN_FORBIDDEN: current_state `{}` does not allow template execution",
                context.current_state
            ),
            request_id,
        ));
    }
    if context.payment_status != "paid" {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: payment status is not paid",
            request_id,
        ));
    }
    if context.buyer_status != "active" || context.seller_status != "active" {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: buyer/seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&context.buyer_metadata)
        || !is_subject_deliverable(&context.seller_metadata)
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: buyer/seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_template_allowed(
    grant: &TemplateGrantContext,
    template: &QueryTemplateContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if grant.grant_status != "active" {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: template query grant is not active",
            request_id,
        ));
    }
    if template.status != "active" {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: query template is not active",
            request_id,
        ));
    }
    if template.template_type != grant.template_type {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: query template type does not match template grant",
            request_id,
        ));
    }
    if !grant
        .allowed_template_ids
        .iter()
        .any(|value| value == &template.query_template_id)
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: query template is not granted for current order",
            request_id,
        ));
    }
    if template
        .analysis_rule_json
        .get("template_review_status")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().eq_ignore_ascii_case("approved"))
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: query template review_status is not approved",
            request_id,
        ));
    }
    Ok(())
}

fn normalize_object(
    value: Option<Value>,
    field: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let value = value.unwrap_or_else(|| json!({}));
    if value.is_object() {
        return Ok(value);
    }
    Err(bad_request(
        &format!("{field} must be a JSON object"),
        request_id,
    ))
}

fn validate_parameter_schema(
    schema: &Value,
    payload: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let payload_object = payload
        .as_object()
        .ok_or_else(|| bad_request("request_payload_json must be a JSON object", request_id))?;

    let schema_properties = extract_schema_properties(schema);
    let required_fields = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    for field in required_fields {
        if !payload_object.contains_key(&field) {
            return Err(conflict(
                &format!(
                    "TEMPLATE_RUN_FORBIDDEN: request_payload_json is missing required field `{field}`"
                ),
                request_id,
            ));
        }
    }

    let additional_allowed = schema
        .get("additionalProperties")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !schema_properties.is_empty() && !additional_allowed {
        for key in payload_object.keys() {
            if !schema_properties.contains_key(key) {
                return Err(conflict(
                    &format!(
                        "TEMPLATE_RUN_FORBIDDEN: request parameter `{key}` is not declared in parameter_schema_json"
                    ),
                    request_id,
                ));
            }
        }
    }

    for (key, value) in payload_object {
        if let Some(field_schema) = schema_properties.get(key) {
            validate_value_against_schema(key, value, field_schema, request_id)?;
        }
    }
    Ok(())
}

fn extract_schema_properties(schema: &Value) -> Map<String, Value> {
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        return properties.clone();
    }
    if let Some(fields) = schema.get("fields").and_then(Value::as_array) {
        let mut properties = Map::new();
        for field in fields {
            if let Some(name) = field.get("name").and_then(Value::as_str) {
                let mut normalized = Map::new();
                if let Some(field_type) = field.get("type") {
                    normalized.insert("type".to_string(), field_type.clone());
                }
                if let Some(enum_values) = field.get("enum") {
                    normalized.insert("enum".to_string(), enum_values.clone());
                }
                properties.insert(name.to_string(), Value::Object(normalized));
            }
        }
        return properties;
    }
    Map::new()
}

fn validate_value_against_schema(
    field: &str,
    value: &Value,
    schema: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        let valid = match expected_type {
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            _ => true,
        };
        if !valid {
            return Err(conflict(
                &format!(
                    "TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` does not match declared type `{expected_type}`"
                ),
                request_id,
            ));
        }
    }

    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array)
        && !enum_values.iter().any(|candidate| candidate == value)
    {
        return Err(conflict(
            &format!(
                "TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` is outside declared enum"
            ),
            request_id,
        ));
    }

    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64)
        && value.as_f64().is_some_and(|numeric| numeric < minimum)
    {
        return Err(conflict(
            &format!("TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` is lower than minimum"),
            request_id,
        ));
    }
    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64)
        && value.as_f64().is_some_and(|numeric| numeric > maximum)
    {
        return Err(conflict(
            &format!("TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` is greater than maximum"),
            request_id,
        ));
    }
    if let Some(min_length) = schema.get("minLength").and_then(Value::as_u64)
        && value
            .as_str()
            .is_some_and(|string| string.chars().count() < min_length as usize)
    {
        return Err(conflict(
            &format!(
                "TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` is shorter than minLength"
            ),
            request_id,
        ));
    }
    if let Some(max_length) = schema.get("maxLength").and_then(Value::as_u64)
        && value
            .as_str()
            .is_some_and(|string| string.chars().count() > max_length as usize)
    {
        return Err(conflict(
            &format!(
                "TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` is longer than maxLength"
            ),
            request_id,
        ));
    }
    if let Some(items_schema) = schema.get("items")
        && let Some(items) = value.as_array()
    {
        if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64)
            && items.len() < min_items as usize
        {
            return Err(conflict(
                &format!(
                    "TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` has fewer than minItems"
                ),
                request_id,
            ));
        }
        if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64)
            && items.len() > max_items as usize
        {
            return Err(conflict(
                &format!("TEMPLATE_RUN_FORBIDDEN: request parameter `{field}` exceeds maxItems"),
                request_id,
            ));
        }
        for (idx, item) in items.iter().enumerate() {
            validate_value_against_schema(
                &format!("{field}[{idx}]"),
                item,
                items_schema,
                request_id,
            )?;
        }
    }
    Ok(())
}

fn resolve_run_output_boundary(
    requested: Option<Value>,
    grant_boundary_json: &Value,
    template_export_policy_json: &Value,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut boundary = normalize_object(
        requested.or(Some(grant_boundary_json.clone())),
        "output_boundary_json",
        request_id,
    )?;
    if boundary
        .get("allow_raw_export")
        .and_then(Value::as_bool)
        .is_some_and(|value| value)
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: allow_raw_export cannot be true",
            request_id,
        ));
    }

    let grant_formats = extract_string_array(grant_boundary_json, "allowed_formats")?;
    let template_formats = extract_string_array(template_export_policy_json, "allowed_formats")?;
    let mut allowed_formats = extract_string_array(&boundary, "allowed_formats")?;
    if allowed_formats.is_empty() {
        allowed_formats = if !grant_formats.is_empty() {
            grant_formats.clone()
        } else {
            template_formats.clone()
        };
    }
    if !grant_formats.is_empty()
        && allowed_formats
            .iter()
            .any(|format| !grant_formats.contains(format))
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: output formats exceed template grant boundary",
            request_id,
        ));
    }
    if !template_formats.is_empty()
        && allowed_formats
            .iter()
            .any(|format| !template_formats.contains(format))
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: output formats exceed query template export policy",
            request_id,
        ));
    }

    let grant_max_rows = parse_positive_i64(grant_boundary_json, "max_rows", request_id)?;
    let template_max_rows =
        parse_positive_i64(template_export_policy_json, "max_export_rows", request_id)?;
    let requested_max_rows = parse_positive_i64(&boundary, "max_rows", request_id)?;
    let max_rows_limit = min_positive_i64(grant_max_rows, template_max_rows);
    if let (Some(limit), Some(value)) = (max_rows_limit, requested_max_rows)
        && value > limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: max_rows exceeds template grant/template boundary",
            request_id,
        ));
    }

    let grant_max_cells = parse_positive_i64(grant_boundary_json, "max_cells", request_id)?;
    let template_max_cells =
        parse_positive_i64(template_export_policy_json, "max_export_cells", request_id)?;
    let requested_max_cells = parse_positive_i64(&boundary, "max_cells", request_id)?;
    let max_cells_limit = min_positive_i64(grant_max_cells, template_max_cells);
    if let (Some(limit), Some(value)) = (max_cells_limit, requested_max_cells)
        && value > limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: max_cells exceeds template grant/template boundary",
            request_id,
        ));
    }

    let object = boundary
        .as_object_mut()
        .ok_or_else(|| bad_request("output_boundary_json must be an object", request_id))?;
    object.insert("allow_raw_export".to_string(), Value::Bool(false));
    object.insert(
        "allowed_formats".to_string(),
        Value::Array(allowed_formats.iter().cloned().map(Value::String).collect()),
    );
    if object.get("max_rows").is_none() {
        if let Some(limit) = max_rows_limit {
            object.insert("max_rows".to_string(), Value::from(limit));
        }
    }
    if object.get("max_cells").is_none() {
        if let Some(limit) = max_cells_limit {
            object.insert("max_cells".to_string(), Value::from(limit));
        }
    }
    if object.get("selected_format").is_none() {
        if let Some(first_format) = allowed_formats.first() {
            object.insert(
                "selected_format".to_string(),
                Value::String(first_format.clone()),
            );
        }
    }
    Ok(boundary)
}

fn resolve_masked_level(
    requested: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let masked_level = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("masked")
        .to_ascii_lowercase();
    if !matches!(masked_level.as_str(), "masked" | "summary" | "restricted") {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: masked_level must be masked/summary/restricted",
            request_id,
        ));
    }
    Ok(masked_level)
}

fn resolve_export_scope(
    requested: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let export_scope = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("none")
        .to_ascii_lowercase();
    if !matches!(
        export_scope.as_str(),
        "none" | "summary" | "restricted_object"
    ) {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: export_scope must be none/summary/restricted_object",
            request_id,
        ));
    }
    Ok(export_scope)
}

fn validate_template_risk_guard(
    analysis_rule_json: &Value,
    risk_guard_json: &Value,
    request_payload_json: &Value,
    approval_ticket_id: Option<&str>,
    masked_level: &str,
    export_scope: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if analysis_rule_json
        .get("analysis_rule")
        .and_then(Value::as_str)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("free_sql"))
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: free_sql analysis rule is not allowed",
            request_id,
        ));
    }
    if risk_guard_json
        .get("risk_mode")
        .and_then(Value::as_str)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("bypass"))
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: risk_guard_json cannot bypass risk evaluation",
            request_id,
        ));
    }
    if risk_guard_json
        .get("approval_required")
        .and_then(Value::as_bool)
        == Some(true)
        && approval_ticket_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: approval_ticket_id is required by risk_guard_json",
            request_id,
        ));
    }
    if risk_guard_json
        .get("strict_masking")
        .and_then(Value::as_bool)
        == Some(true)
        && masked_level == "restricted"
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: strict_masking blocks restricted masked_level",
            request_id,
        ));
    }
    if export_scope == "restricted_object"
        && risk_guard_json
            .get("allow_restricted_object")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: export_scope restricted_object is not allowed by risk_guard_json",
            request_id,
        ));
    }
    let blocked_parameter_keys = risk_guard_json
        .get("blocked_parameter_keys")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if let Some(payload_object) = request_payload_json.as_object() {
        for key in payload_object.keys() {
            if blocked_parameter_keys.contains(key) {
                return Err(conflict(
                    &format!(
                        "TEMPLATE_RUN_FORBIDDEN: request parameter `{key}` is blocked by risk_guard_json"
                    ),
                    request_id,
                ));
            }
        }
    }
    Ok(())
}

async fn enforce_run_quota(
    client: &(impl GenericClient + Sync),
    template_query_grant_id: &str,
    order_id: &str,
    run_quota_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_one(
            "SELECT COUNT(*)::bigint,
                    COUNT(*) FILTER (WHERE created_at >= date_trunc('day', now()))::bigint,
                    COUNT(*) FILTER (WHERE created_at >= date_trunc('month', now()))::bigint
             FROM delivery.query_execution_run
             WHERE order_id = $1::text::uuid
               AND template_query_grant_id = $2::text::uuid",
            &[&order_id, &template_query_grant_id],
        )
        .await
        .map_err(map_db_error)?;
    let total_runs: i64 = row.get(0);
    let daily_runs: i64 = row.get(1);
    let monthly_runs: i64 = row.get(2);

    let max_runs = parse_positive_i64(run_quota_json, "max_runs", request_id)?;
    let daily_limit = parse_positive_i64(run_quota_json, "daily_limit", request_id)?;
    let monthly_limit = parse_positive_i64(run_quota_json, "monthly_limit", request_id)?;
    if let Some(limit) = max_runs
        && total_runs >= limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: template grant max_runs has been exhausted",
            request_id,
        ));
    }
    if let Some(limit) = daily_limit
        && daily_runs >= limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: template grant daily_limit has been exhausted",
            request_id,
        ));
    }
    if let Some(limit) = monthly_limit
        && monthly_runs >= limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: template grant monthly_limit has been exhausted",
            request_id,
        ));
    }
    Ok(())
}

fn resolve_result_fields(
    result_schema_json: &Value,
    analysis_rule_json: &Value,
    export_policy_json: &Value,
) -> Vec<ResultField> {
    let whitelist = analysis_rule_json
        .get("whitelist_fields")
        .and_then(Value::as_array)
        .or_else(|| {
            export_policy_json
                .get("whitelist_fields")
                .and_then(Value::as_array)
        })
        .map(|fields| {
            fields
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut fields = Vec::new();
    if let Some(properties) = result_schema_json
        .get("properties")
        .and_then(Value::as_object)
    {
        for (name, schema) in properties {
            if !whitelist.is_empty() && !whitelist.contains(name) {
                continue;
            }
            fields.push(ResultField {
                name: name.to_string(),
                field_type: schema
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("string")
                    .to_string(),
            });
        }
    }
    if let Some(field_array) = result_schema_json.get("fields").and_then(Value::as_array) {
        for field in field_array {
            let Some(name) = field.get("name").and_then(Value::as_str) else {
                continue;
            };
            if !whitelist.is_empty() && !whitelist.contains(name) {
                continue;
            }
            if fields.iter().any(|existing| existing.name == name) {
                continue;
            }
            fields.push(ResultField {
                name: name.to_string(),
                field_type: field
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("string")
                    .to_string(),
            });
        }
    }
    if fields.is_empty() {
        fields.push(ResultField {
            name: "result_summary".to_string(),
            field_type: "string".to_string(),
        });
    }
    fields
}

fn derive_result_row_count(
    request_payload_json: &Value,
    output_boundary_json: &Value,
    request_id: Option<&str>,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    let requested_limit = request_payload_json
        .get("limit")
        .and_then(Value::as_i64)
        .or_else(|| {
            request_payload_json
                .get("limit")
                .and_then(Value::as_u64)
                .map(|v| v as i64)
        });
    let boundary_max_rows = parse_positive_i64(output_boundary_json, "max_rows", request_id)?;
    let row_count = requested_limit.or(boundary_max_rows).unwrap_or(1);
    if row_count <= 0 {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: request limit must be greater than zero",
            request_id,
        ));
    }
    if let Some(limit) = boundary_max_rows
        && row_count > limit
    {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: request limit exceeds output boundary max_rows",
            request_id,
        ));
    }
    Ok(row_count)
}

fn enforce_max_cells(
    output_boundary_json: &Value,
    row_count: i64,
    field_count: usize,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let max_cells = parse_positive_i64(output_boundary_json, "max_cells", request_id)?;
    if let Some(limit) = max_cells {
        let total_cells = row_count.saturating_mul(field_count as i64);
        if total_cells > limit {
            return Err(conflict(
                "TEMPLATE_RUN_FORBIDDEN: result cell count exceeds output boundary max_cells",
                request_id,
            ));
        }
    }
    Ok(())
}

fn build_request_payload_record(
    request_payload_json: &Value,
    output_boundary_json: &Value,
    execution_metadata_json: Option<Value>,
) -> Value {
    let parameter_keys = request_payload_json
        .as_object()
        .map(|items| items.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    json!({
        "parameters": request_payload_json,
        "parameter_summary": {
            "parameter_keys": parameter_keys,
            "parameter_count": parameter_keys.len(),
            "limit": request_payload_json.get("limit"),
        },
        "output_boundary_json": output_boundary_json,
        "execution_metadata_json": execution_metadata_json.unwrap_or_else(|| json!({})),
    })
}

fn build_sensitive_policy_snapshot(
    masked_level: &str,
    export_scope: &str,
    output_boundary_json: &Value,
    risk_guard_json: &Value,
    approval_ticket_id: Option<&str>,
) -> Value {
    json!({
        "masked_level": masked_level,
        "export_scope": export_scope,
        "output_boundary_json": output_boundary_json,
        "risk_guard_json": risk_guard_json,
        "approval_ticket_id": approval_ticket_id,
    })
}

#[derive(Debug)]
struct ResultField {
    name: String,
    field_type: String,
}

fn build_result_payload(
    query_run_id: &str,
    order_id: &str,
    template: &QueryTemplateContext,
    selected_fields: &[ResultField],
    row_count: i64,
    selected_format: &str,
    request_payload_json: &Value,
    output_boundary_json: &Value,
    execution_metadata_json: Option<&Value>,
) -> Value {
    let preview_rows = (0..row_count.min(3))
        .map(|idx| {
            let mut row = Map::new();
            for field in selected_fields {
                row.insert(
                    field.name.clone(),
                    sample_value_for_field(&field.field_type, idx),
                );
            }
            Value::Object(row)
        })
        .collect::<Vec<_>>();
    json!({
        "query_run_id": query_run_id,
        "order_id": order_id,
        "query_template_id": template.query_template_id,
        "query_template_name": template.template_name,
        "query_template_version": template.version_no,
        "selected_format": selected_format,
        "row_count": row_count,
        "columns": selected_fields.iter().map(|field| json!({"name": field.name, "type": field.field_type})).collect::<Vec<_>>(),
        "preview_rows": preview_rows,
        "request_payload_json": request_payload_json,
        "output_boundary_json": output_boundary_json,
        "execution_metadata_json": execution_metadata_json.cloned().unwrap_or_else(|| json!({})),
    })
}

fn render_result_payload(
    payload: &Value,
    selected_fields: &[ResultField],
    selected_format: &str,
) -> Result<Vec<u8>, (StatusCode, Json<ErrorResponse>)> {
    match selected_format {
        "csv" => {
            let mut csv = String::new();
            csv.push_str(
                &selected_fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            csv.push('\n');
            if let Some(rows) = payload.get("preview_rows").and_then(Value::as_array) {
                for row in rows {
                    let values = selected_fields
                        .iter()
                        .map(|field| {
                            row.get(&field.name)
                                .map(csv_value)
                                .unwrap_or_else(|| "".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    csv.push_str(&values);
                    csv.push('\n');
                }
            }
            Ok(csv.into_bytes())
        }
        _ => serde_json::to_vec_pretty(payload).map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: ErrorCode::OpsInternal.as_str().to_string(),
                    message: format!("render query result payload failed: {err}"),
                    request_id: None,
                }),
            )
        }),
    }
}

fn build_result_summary(
    result_hash: &str,
    object_uri: &str,
    selected_fields: &[ResultField],
    row_count: i64,
    selected_format: &str,
    masked_level: &str,
    export_scope: &str,
    approval_ticket_id: Option<&str>,
    request_payload_json: &Value,
) -> Value {
    json!({
        "result_hash": result_hash,
        "result_object_uri": object_uri,
        "selected_format": selected_format,
        "row_count": row_count,
        "column_count": selected_fields.len(),
        "columns": selected_fields.iter().map(|field| field.name.clone()).collect::<Vec<_>>(),
        "masked_level": masked_level,
        "export_scope": export_scope,
        "approval_ticket_id": approval_ticket_id,
        "policy_hits": ["template_whitelist_passed", "parameter_schema_passed", "output_boundary_passed", "risk_guard_passed"],
        "request_parameter_keys": request_payload_json.as_object().map(|items| items.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
    })
}

fn sample_value_for_field(field_type: &str, idx: i64) -> Value {
    match field_type {
        "integer" => Value::from(idx + 1),
        "number" => Value::from((idx + 1) as f64),
        "boolean" => Value::Bool(idx % 2 == 0),
        "array" => Value::Array(vec![]),
        "object" => json!({}),
        _ => Value::String(format!("masked_{idx}")),
    }
}

fn csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => {
            if string.contains(',') || string.contains('"') || string.contains('\n') {
                format!("\"{}\"", string.replace('"', "\"\""))
            } else {
                string.clone()
            }
        }
        other => other.to_string(),
    }
}

fn build_result_hash(query_run_id: &str, row_count: i64, field_count: usize) -> String {
    format!("sha256:query-run:{query_run_id}:{row_count}:{field_count}")
}

fn resolve_selected_format(
    output_boundary_json: &Value,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let selected_format = output_boundary_json
        .get("selected_format")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            output_boundary_json
                .get("allowed_formats")
                .and_then(Value::as_array)
                .and_then(|formats| formats.first())
                .and_then(Value::as_str)
                .map(|value| value.trim().to_ascii_lowercase())
        })
        .unwrap_or_else(|| "json".to_string());
    let allowed_formats = output_boundary_json
        .get("allowed_formats")
        .and_then(Value::as_array)
        .map(|formats| {
            formats
                .iter()
                .filter_map(Value::as_str)
                .map(|value| value.trim().to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["json".to_string()]);
    if !allowed_formats.contains(&selected_format) {
        return Err(conflict(
            "TEMPLATE_RUN_FORBIDDEN: selected_format is outside output boundary allowed_formats",
            None,
        ));
    }
    Ok(selected_format)
}

fn content_type_for_format(selected_format: &str) -> &'static str {
    match selected_format {
        "csv" => "text/csv",
        _ => "application/json",
    }
}

fn derive_target_state(current_state: &str) -> &str {
    match current_state {
        "result_available" => "result_available",
        _ => "query_executed",
    }
}

fn derive_qry_lite_delivery_status(target_state: &str, _payment_status: &str) -> String {
    match target_state {
        "template_authorized" | "params_validated" => "in_progress".to_string(),
        "query_executed" | "result_available" => "delivered".to_string(),
        "closed" => "closed".to_string(),
        _ => "pending_delivery".to_string(),
    }
}

fn derive_qry_lite_acceptance_status(target_state: &str) -> String {
    match target_state {
        "query_executed" | "result_available" => "accepted".to_string(),
        "closed" => "closed".to_string(),
        _ => "not_started".to_string(),
    }
}

fn derive_qry_lite_settlement_status(target_state: &str, payment_status: &str) -> String {
    match target_state {
        "template_authorized" | "params_validated" | "query_executed" | "result_available" => {
            if payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            }
        }
        "closed" => "closed".to_string(),
        _ => "not_started".to_string(),
    }
}

fn extract_string_array(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
    let array = value
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut normalized = Vec::with_capacity(array.len());
    for item in array {
        let Some(item) = item.as_str() else {
            return Err(bad_request(
                &format!("{field} must be an array of strings"),
                None,
            ));
        };
        let item = item.trim().to_ascii_lowercase();
        if !item.is_empty() {
            normalized.push(item);
        }
    }
    Ok(normalized)
}

fn parse_string_array(value: Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>()
    })
}

fn parse_positive_i64(
    value: &Value,
    field: &str,
    request_id: Option<&str>,
) -> Result<Option<i64>, (StatusCode, Json<ErrorResponse>)> {
    let Some(field_value) = value.get(field) else {
        return Ok(None);
    };
    let parsed = field_value
        .as_i64()
        .or_else(|| field_value.as_u64().map(|value| value as i64))
        .ok_or_else(|| bad_request(&format!("{field} must be a positive integer"), request_id))?;
    if parsed <= 0 {
        return Err(bad_request(
            &format!("{field} must be a positive integer"),
            request_id,
        ));
    }
    Ok(Some(parsed))
}

fn min_positive_i64(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn default_result_bucket_name() -> String {
    std::env::var(RESULT_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_RESULT_BUCKET.to_string())
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

    metadata
        .get("risk_flags")
        .and_then(Value::as_array)
        .is_none_or(|flags| {
            !flags.iter().any(|flag| {
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
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
