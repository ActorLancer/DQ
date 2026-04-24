use crate::modules::delivery::dto::{ManageQueryTemplateRequest, QueryTemplateResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const DELIVERY_QUERY_TEMPLATE_MANAGE_EVENT: &str = "delivery.query_template.manage";
const DEFAULT_TEMPLATE_TYPE: &str = "sql_template";
const DEFAULT_TEMPLATE_STATUS: &str = "draft";

pub async fn manage_query_template(
    client: &mut Client,
    query_surface_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageQueryTemplateRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<QueryTemplateResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_query_surface_context(&tx, query_surface_id, request_id).await?;

    enforce_seller_subject_status(&context, request_id)?;
    enforce_manage_scope(actor_role, tenant_id, &context.seller_org_id, request_id)?;
    enforce_query_surface_status(&context, request_id)?;

    let existing = load_existing_template(&tx, query_surface_id, payload, request_id).await?;

    let template_name = payload
        .template_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| existing.template_name.clone())
        .ok_or_else(|| bad_request("template_name is required", request_id))?;
    let template_type = payload
        .template_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| existing.template_type.clone())
        .unwrap_or_else(|| DEFAULT_TEMPLATE_TYPE.to_string());
    let template_body_ref = payload
        .template_body_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| existing.template_body_ref.clone());
    let status = normalize_status(
        payload.status.as_deref().or(existing.status.as_deref()),
        request_id,
    )?;

    if payload.query_template_id.is_none() && payload.template_name.is_none() {
        return Err(bad_request(
            "template_name is required when query_template_id is absent",
            request_id,
        ));
    }

    let parameter_schema_json = normalize_schema_object(
        payload
            .parameter_schema_json
            .clone()
            .or(existing.parameter_schema_json.clone()),
        "parameter_schema_json",
        request_id,
    )?;
    let result_schema_json = normalize_schema_object(
        payload
            .result_schema_json
            .clone()
            .or(existing.result_schema_json.clone()),
        "result_schema_json",
        request_id,
    )?;
    if existing.query_template_id.is_none() {
        if payload.parameter_schema_json.is_none() {
            return Err(bad_request(
                "parameter_schema_json is required when creating a template version",
                request_id,
            ));
        }
        if payload.result_schema_json.is_none() {
            return Err(bad_request(
                "result_schema_json is required when creating a template version",
                request_id,
            ));
        }
    }

    let mut analysis_rule_json = normalize_object(
        payload
            .analysis_rule_json
            .clone()
            .or(existing.analysis_rule_json.clone()),
        "analysis_rule_json",
        request_id,
    )?;
    let mut export_policy_json = normalize_object(
        payload
            .export_policy_json
            .clone()
            .or(existing.export_policy_json.clone()),
        "export_policy_json",
        request_id,
    )?;
    let risk_guard_json = normalize_object(
        payload
            .risk_guard_json
            .clone()
            .or(existing.risk_guard_json.clone()),
        "risk_guard_json",
        request_id,
    )?;

    let whitelist_fields = resolve_whitelist_fields(
        payload.whitelist_fields.as_ref(),
        &analysis_rule_json,
        &export_policy_json,
        &result_schema_json,
        request_id,
    )?;
    merge_whitelist_fields(
        &mut analysis_rule_json,
        &mut export_policy_json,
        &whitelist_fields,
    );

    validate_export_policy(&export_policy_json, request_id)?;
    validate_analysis_rule(&analysis_rule_json, request_id)?;
    validate_risk_guard(&risk_guard_json, request_id)?;

    let version_no = resolve_version_no(
        &tx,
        query_surface_id,
        &template_name,
        payload,
        &existing,
        request_id,
    )
    .await?;

    let (row, operation) = if let Some(query_template_id) = existing.query_template_id.as_deref() {
        (
            tx.query_one(
                "UPDATE delivery.query_template_definition
                 SET template_name = $2,
                     template_type = $3,
                     template_body_ref = $4,
                     parameter_schema_json = $5::jsonb,
                     analysis_rule_json = $6::jsonb,
                     result_schema_json = $7::jsonb,
                     export_policy_json = $8::jsonb,
                     risk_guard_json = $9::jsonb,
                     status = $10,
                     updated_at = now()
                 WHERE query_template_id = $1::text::uuid
                 RETURNING query_template_id::text,
                           query_surface_id::text,
                           template_name,
                           template_type,
                           template_body_ref,
                           version_no,
                           parameter_schema_json,
                           analysis_rule_json,
                           result_schema_json,
                           export_policy_json,
                           risk_guard_json,
                           status,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &query_template_id,
                    &template_name,
                    &template_type,
                    &template_body_ref,
                    &parameter_schema_json,
                    &analysis_rule_json,
                    &result_schema_json,
                    &export_policy_json,
                    &risk_guard_json,
                    &status,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "updated",
        )
    } else {
        (
            tx.query_one(
                "INSERT INTO delivery.query_template_definition (
                   query_surface_id,
                   template_name,
                   template_type,
                   template_body_ref,
                   parameter_schema_json,
                   analysis_rule_json,
                   result_schema_json,
                   export_policy_json,
                   risk_guard_json,
                   status,
                   version_no
                 ) VALUES (
                   $1::text::uuid,
                   $2,
                   $3,
                   $4,
                   $5::jsonb,
                   $6::jsonb,
                   $7::jsonb,
                   $8::jsonb,
                   $9::jsonb,
                   $10,
                   $11
                 )
                 RETURNING query_template_id::text,
                           query_surface_id::text,
                           template_name,
                           template_type,
                           template_body_ref,
                           version_no,
                           parameter_schema_json,
                           analysis_rule_json,
                           result_schema_json,
                           export_policy_json,
                           risk_guard_json,
                           status,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &query_surface_id,
                    &template_name,
                    &template_type,
                    &template_body_ref,
                    &parameter_schema_json,
                    &analysis_rule_json,
                    &result_schema_json,
                    &export_policy_json,
                    &risk_guard_json,
                    &status,
                    &version_no,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "created",
        )
    };

    let query_template_id: String = row.get(0);
    write_delivery_audit_event(
        &tx,
        "query_template",
        &query_template_id,
        actor_role,
        DELIVERY_QUERY_TEMPLATE_MANAGE_EVENT,
        operation,
        request_id,
        trace_id,
        json!({
            "query_surface_id": query_surface_id,
            "asset_version_id": context.asset_version_id,
            "seller_org_id": context.seller_org_id,
            "surface_type": context.surface_type,
            "template_name": template_name,
            "template_type": template_type,
            "version_no": version_no,
            "status": status,
            "whitelist_fields": whitelist_fields.clone(),
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(QueryTemplateResponseData {
        query_template_id,
        query_surface_id: row.get(1),
        template_name: row.get(2),
        template_type: row.get(3),
        template_body_ref: row.get(4),
        version_no: row.get(5),
        parameter_schema_json: row.get(6),
        analysis_rule_json: row.get(7),
        result_schema_json: row.get(8),
        export_policy_json: row.get(9),
        risk_guard_json: row.get(10),
        whitelist_fields,
        status: row.get(11),
        operation: operation.to_string(),
        created_at: row.get(12),
        updated_at: row.get(13),
    })
}

struct QuerySurfaceContext {
    query_surface_id: String,
    surface_type: String,
    surface_status: String,
    asset_version_id: String,
    seller_org_id: String,
    seller_status: String,
    seller_metadata: Value,
    asset_version_status: String,
}

#[derive(Default)]
struct ExistingTemplate {
    query_template_id: Option<String>,
    template_name: Option<String>,
    template_type: Option<String>,
    template_body_ref: Option<String>,
    version_no: Option<i32>,
    parameter_schema_json: Option<Value>,
    analysis_rule_json: Option<Value>,
    result_schema_json: Option<Value>,
    export_policy_json: Option<Value>,
    risk_guard_json: Option<Value>,
    status: Option<String>,
}

async fn load_query_surface_context(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    request_id: Option<&str>,
) -> Result<QuerySurfaceContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT qs.query_surface_id::text,
                    qs.surface_type,
                    qs.status,
                    qs.asset_version_id::text,
                    asset.owner_org_id::text,
                    seller.status,
                    COALESCE(seller.metadata, '{}'::jsonb),
                    v.status
             FROM catalog.query_surface_definition qs
             JOIN catalog.asset_version v ON v.asset_version_id = qs.asset_version_id
             JOIN catalog.data_asset asset ON asset.asset_id = v.asset_id
             JOIN core.organization seller ON seller.org_id = asset.owner_org_id
             WHERE qs.query_surface_id = $1::text::uuid
             FOR UPDATE OF qs",
            &[&query_surface_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(query_surface_not_found(query_surface_id, request_id));
    };

    Ok(QuerySurfaceContext {
        query_surface_id: row.get(0),
        surface_type: row.get(1),
        surface_status: row.get(2),
        asset_version_id: row.get(3),
        seller_org_id: row.get(4),
        seller_status: row.get(5),
        seller_metadata: row.get(6),
        asset_version_status: row.get(7),
    })
}

async fn load_existing_template(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    payload: &ManageQueryTemplateRequest,
    request_id: Option<&str>,
) -> Result<ExistingTemplate, (StatusCode, Json<ErrorResponse>)> {
    let row = if let Some(query_template_id) = payload
        .query_template_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        client
            .query_opt(
                "SELECT query_template_id::text,
                        template_name,
                        template_type,
                        template_body_ref,
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
            .map_err(map_db_error)?
    } else if let (Some(template_name), Some(version_no)) = (
        payload
            .template_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        payload.version_no,
    ) {
        client
            .query_opt(
                "SELECT query_template_id::text,
                        template_name,
                        template_type,
                        template_body_ref,
                        version_no,
                        parameter_schema_json,
                        analysis_rule_json,
                        result_schema_json,
                        export_policy_json,
                        risk_guard_json,
                        status
                 FROM delivery.query_template_definition
                 WHERE query_surface_id = $1::text::uuid
                   AND template_name = $2
                   AND version_no = $3",
                &[&query_surface_id, &template_name, &version_no],
            )
            .await
            .map_err(map_db_error)?
    } else {
        None
    };

    if payload
        .query_template_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
        && row.is_none()
    {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: query_template_id does not belong to current query surface",
            request_id,
        ));
    }

    Ok(
        row.map_or_else(ExistingTemplate::default, |row| ExistingTemplate {
            query_template_id: row.get(0),
            template_name: row.get(1),
            template_type: row.get(2),
            template_body_ref: row.get(3),
            version_no: row.get(4),
            parameter_schema_json: Some(row.get(5)),
            analysis_rule_json: Some(row.get(6)),
            result_schema_json: Some(row.get(7)),
            export_policy_json: Some(row.get(8)),
            risk_guard_json: Some(row.get(9)),
            status: row.get(10),
        }),
    )
}

fn enforce_seller_subject_status(
    context: &QuerySurfaceContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.seller_status != "active" {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&context.seller_metadata) {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: seller organization is blocked by subject risk policy",
            request_id,
        ));
    }
    if !matches!(
        context.asset_version_status.as_str(),
        "active" | "published"
    ) {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: asset version status is not active/published",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_manage_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(actor_role, "seller_operator" | "tenant_admin") && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    Err(forbidden(
        "query template management is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_query_surface_status(
    context: &QuerySurfaceContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.surface_status == "disabled" {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: query surface is disabled",
            request_id,
        ));
    }
    Ok(())
}

fn normalize_status(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TEMPLATE_STATUS)
        .to_ascii_lowercase();
    if matches!(normalized.as_str(), "draft" | "active" | "disabled") {
        return Ok(normalized);
    }
    Err(bad_request(
        "status must be one of draft, active, disabled",
        request_id,
    ))
}

fn normalize_schema_object(
    value: Option<Value>,
    field: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let value = value.unwrap_or_else(|| json!({}));
    if !value.is_object() {
        return Err(bad_request(
            &format!("{field} must be a JSON object"),
            request_id,
        ));
    }
    Ok(value)
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

fn resolve_whitelist_fields(
    payload_whitelist_fields: Option<&Vec<String>>,
    analysis_rule_json: &Value,
    export_policy_json: &Value,
    result_schema_json: &Value,
    request_id: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
    let whitelist_fields = if let Some(fields) = payload_whitelist_fields {
        fields.clone()
    } else if let Some(fields) = analysis_rule_json
        .get("whitelist_fields")
        .and_then(Value::as_array)
    {
        fields
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    } else if let Some(fields) = export_policy_json
        .get("whitelist_fields")
        .and_then(Value::as_array)
    {
        fields
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };

    let normalized = whitelist_fields
        .into_iter()
        .map(|field| field.trim().to_string())
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>();
    if normalized.len() != normalized.iter().collect::<BTreeSet<_>>().len() {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: whitelist_fields contains duplicate field names",
            request_id,
        ));
    }

    let result_fields = extract_schema_field_names(result_schema_json);
    if !normalized.is_empty() && result_fields.is_empty() {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: result_schema_json must declare fields before whitelist_fields can be used",
            request_id,
        ));
    }
    for field in &normalized {
        if !result_fields.contains(field) {
            return Err(conflict(
                &format!(
                    "QUERY_TEMPLATE_MANAGE_FORBIDDEN: whitelist field `{field}` is not declared in result_schema_json"
                ),
                request_id,
            ));
        }
    }
    Ok(normalized)
}

fn extract_schema_field_names(result_schema_json: &Value) -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    if let Some(properties) = result_schema_json
        .get("properties")
        .and_then(Value::as_object)
    {
        fields.extend(properties.keys().cloned());
    }
    if let Some(field_array) = result_schema_json.get("fields").and_then(Value::as_array) {
        fields.extend(field_array.iter().filter_map(|field| {
            field
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
        }));
    }
    fields
}

fn merge_whitelist_fields(
    analysis_rule_json: &mut Value,
    export_policy_json: &mut Value,
    whitelist_fields: &[String],
) {
    if let Some(object) = analysis_rule_json.as_object_mut() {
        object.insert(
            "whitelist_fields".to_string(),
            Value::Array(
                whitelist_fields
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }
    if let Some(object) = export_policy_json.as_object_mut() {
        object.insert(
            "whitelist_fields".to_string(),
            Value::Array(
                whitelist_fields
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
    }
}

fn validate_export_policy(
    export_policy_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if export_policy_json
        .get("allow_raw_export")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: allow_raw_export cannot be true",
            request_id,
        ));
    }
    if export_policy_json
        .get("allowed_formats")
        .and_then(Value::as_array)
        .is_some_and(|formats| {
            formats.iter().any(|format| {
                matches!(
                    format
                        .as_str()
                        .unwrap_or_default()
                        .trim()
                        .to_ascii_lowercase()
                        .as_str(),
                    "raw" | "raw_table" | "raw_export"
                )
            })
        })
    {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: raw format export is not allowed",
            request_id,
        ));
    }
    Ok(())
}

fn validate_analysis_rule(
    analysis_rule_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if analysis_rule_json
        .get("analysis_rule")
        .and_then(Value::as_str)
        .is_some_and(|rule| rule.trim().eq_ignore_ascii_case("free_sql"))
    {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: free_sql analysis rule is not allowed in V1",
            request_id,
        ));
    }
    Ok(())
}

fn validate_risk_guard(
    risk_guard_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if risk_guard_json
        .get("risk_mode")
        .and_then(Value::as_str)
        .is_some_and(|mode| mode.trim().eq_ignore_ascii_case("bypass"))
    {
        return Err(conflict(
            "QUERY_TEMPLATE_MANAGE_FORBIDDEN: risk_guard_json cannot bypass risk evaluation",
            request_id,
        ));
    }
    Ok(())
}

async fn resolve_version_no(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    template_name: &str,
    payload: &ManageQueryTemplateRequest,
    existing: &ExistingTemplate,
    request_id: Option<&str>,
) -> Result<i32, (StatusCode, Json<ErrorResponse>)> {
    if let Some(existing_version_no) = existing.version_no {
        if let Some(payload_version_no) = payload.version_no
            && payload_version_no != existing_version_no
        {
            return Err(conflict(
                "QUERY_TEMPLATE_MANAGE_FORBIDDEN: version_no cannot change for an existing query_template_id",
                request_id,
            ));
        }
        return Ok(existing_version_no);
    }

    if let Some(version_no) = payload.version_no {
        if version_no <= 0 {
            return Err(bad_request("version_no must be > 0", request_id));
        }
        return Ok(version_no);
    }

    let next_version_no: i32 = client
        .query_one(
            "SELECT COALESCE(MAX(version_no), 0) + 1
             FROM delivery.query_template_definition
             WHERE query_surface_id = $1::text::uuid
               AND template_name = $2",
            &[&query_surface_id, &template_name],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    Ok(next_version_no)
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

fn query_surface_not_found(
    query_surface_id: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("query surface not found: {query_surface_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: "QUERY_TEMPLATE_MANAGE_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
