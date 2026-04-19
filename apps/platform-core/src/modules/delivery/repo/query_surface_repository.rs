use crate::modules::delivery::dto::{ManageQuerySurfaceRequest, QuerySurfaceResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

const DELIVERY_QUERY_SURFACE_MANAGE_EVENT: &str = "delivery.query_surface.manage";
const DEFAULT_SURFACE_TYPE: &str = "template_query_lite";
const DEFAULT_BINDING_MODE: &str = "managed_surface";
const DEFAULT_EXECUTION_SCOPE: &str = "curated_zone";
const DEFAULT_STATUS: &str = "draft";

pub async fn manage_query_surface(
    client: &mut Client,
    product_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageQuerySurfaceRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<QuerySurfaceResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_product_context(&tx, product_id, request_id).await?;

    enforce_seller_subject_status(&context, request_id)?;
    enforce_manage_scope(actor_role, tenant_id, &context.seller_org_id, request_id)?;

    let existing =
        load_existing_surface(&tx, &context.asset_version_id, payload, request_id).await?;

    let surface_type = normalized_enum(
        payload
            .surface_type
            .as_deref()
            .or(existing.surface_type.as_deref()),
        DEFAULT_SURFACE_TYPE,
        &["template_query_lite", "sandbox_query", "report_result"],
        "surface_type",
        request_id,
    )?;
    let binding_mode = normalized_enum(
        payload
            .binding_mode
            .as_deref()
            .or(existing.binding_mode.as_deref()),
        DEFAULT_BINDING_MODE,
        &["managed_surface", "seller_managed"],
        "binding_mode",
        request_id,
    )?;
    let execution_scope = normalize_execution_scope(
        payload
            .execution_scope
            .as_deref()
            .or(existing.execution_scope.as_deref()),
        request_id,
    )?;
    let status = normalized_enum(
        payload.status.as_deref().or(existing.status.as_deref()),
        DEFAULT_STATUS,
        &["draft", "active", "disabled"],
        "status",
        request_id,
    )?;

    let environment_id = payload
        .environment_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_request("environment_id is required", request_id))?
        .to_string();
    load_execution_environment(&tx, &environment_id, &context.seller_org_id, request_id).await?;

    let asset_object_id = match payload
        .asset_object_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(asset_object_id) => {
            load_asset_object(&tx, &context.asset_version_id, asset_object_id, request_id).await?;
            Some(asset_object_id.to_string())
        }
        None => existing.asset_object_id.clone(),
    };

    let input_contract_json = normalize_json_object(
        payload
            .input_contract_json
            .clone()
            .or(existing.input_contract_json.clone()),
        "input_contract_json",
        request_id,
    )?;
    let output_boundary_json = normalize_json_object(
        payload
            .output_boundary_json
            .clone()
            .or(existing.output_boundary_json.clone()),
        "output_boundary_json",
        request_id,
    )?;
    let query_policy_json = normalize_json_object(
        payload
            .query_policy_json
            .clone()
            .or(existing.query_policy_json.clone()),
        "query_policy_json",
        request_id,
    )?;
    let quota_policy_json = normalize_json_object(
        payload
            .quota_policy_json
            .clone()
            .or(existing.quota_policy_json.clone()),
        "quota_policy_json",
        request_id,
    )?;
    let metadata = normalize_json_object(
        payload.metadata.clone().or(existing.metadata.clone()),
        "metadata",
        request_id,
    )?;

    validate_read_zones(&execution_scope, &input_contract_json, request_id)?;
    validate_output_boundary(&output_boundary_json, request_id)?;

    let (row, operation) = if let Some(query_surface_id) = existing.query_surface_id.as_deref() {
        (
            tx.query_one(
                "UPDATE catalog.query_surface_definition
                 SET asset_object_id = $2::text::uuid,
                     environment_id = $3::text::uuid,
                     surface_type = $4,
                     binding_mode = $5,
                     execution_scope = $6,
                     input_contract_json = $7::jsonb,
                     output_boundary_json = $8::jsonb,
                     query_policy_json = $9::jsonb,
                     quota_policy_json = $10::jsonb,
                     status = $11,
                     metadata = $12::jsonb,
                     updated_at = now()
                 WHERE query_surface_id = $1::text::uuid
                 RETURNING query_surface_id::text,
                           asset_version_id::text,
                           asset_object_id::text,
                           environment_id::text,
                           surface_type,
                           binding_mode,
                           execution_scope,
                           input_contract_json,
                           output_boundary_json,
                           query_policy_json,
                           quota_policy_json,
                           status,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &query_surface_id,
                    &asset_object_id,
                    &environment_id,
                    &surface_type,
                    &binding_mode,
                    &execution_scope,
                    &input_contract_json,
                    &output_boundary_json,
                    &query_policy_json,
                    &quota_policy_json,
                    &status,
                    &metadata,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "updated",
        )
    } else {
        (
            tx.query_one(
                "INSERT INTO catalog.query_surface_definition (
                   asset_version_id,
                   asset_object_id,
                   environment_id,
                   surface_type,
                   binding_mode,
                   execution_scope,
                   input_contract_json,
                   output_boundary_json,
                   query_policy_json,
                   quota_policy_json,
                   status,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4,
                   $5,
                   $6,
                   $7::jsonb,
                   $8::jsonb,
                   $9::jsonb,
                   $10::jsonb,
                   $11,
                   $12::jsonb
                 )
                 RETURNING query_surface_id::text,
                           asset_version_id::text,
                           asset_object_id::text,
                           environment_id::text,
                           surface_type,
                           binding_mode,
                           execution_scope,
                           input_contract_json,
                           output_boundary_json,
                           query_policy_json,
                           quota_policy_json,
                           status,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &context.asset_version_id,
                    &asset_object_id,
                    &environment_id,
                    &surface_type,
                    &binding_mode,
                    &execution_scope,
                    &input_contract_json,
                    &output_boundary_json,
                    &query_policy_json,
                    &quota_policy_json,
                    &status,
                    &metadata,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "created",
        )
    };

    tx.execute(
        "UPDATE catalog.asset_version
         SET query_surface_type = $2,
             updated_at = now()
         WHERE asset_version_id = $1::text::uuid",
        &[&context.asset_version_id, &surface_type],
    )
    .await
    .map_err(map_db_error)?;

    let query_surface_id: String = row.get(0);
    write_delivery_audit_event(
        &tx,
        "query_surface",
        &query_surface_id,
        actor_role,
        DELIVERY_QUERY_SURFACE_MANAGE_EVENT,
        operation,
        request_id,
        trace_id,
        json!({
            "product_id": product_id,
            "asset_version_id": context.asset_version_id,
            "seller_org_id": context.seller_org_id,
            "asset_object_id": asset_object_id,
            "environment_id": environment_id,
            "surface_type": surface_type,
            "binding_mode": binding_mode,
            "execution_scope": execution_scope,
            "status": status,
            "read_zones": extract_read_zones(&input_contract_json),
            "output_boundary": output_boundary_json,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(QuerySurfaceResponseData {
        query_surface_id,
        product_id: product_id.to_string(),
        asset_version_id: row.get(1),
        asset_object_id: row.get(2),
        environment_id: row.get(3),
        surface_type: row.get(4),
        binding_mode: row.get(5),
        execution_scope: row.get(6),
        input_contract_json: row.get(7),
        output_boundary_json: row.get(8),
        query_policy_json: row.get(9),
        quota_policy_json: row.get(10),
        status: row.get(11),
        operation: operation.to_string(),
        created_at: row.get(12),
        updated_at: row.get(13),
    })
}

struct ProductContext {
    asset_version_id: String,
    seller_org_id: String,
    seller_status: String,
    seller_metadata: Value,
    asset_version_status: String,
}

#[derive(Default)]
struct ExistingSurface {
    query_surface_id: Option<String>,
    asset_object_id: Option<String>,
    environment_id: Option<String>,
    surface_type: Option<String>,
    binding_mode: Option<String>,
    execution_scope: Option<String>,
    input_contract_json: Option<Value>,
    output_boundary_json: Option<Value>,
    query_policy_json: Option<Value>,
    quota_policy_json: Option<Value>,
    status: Option<String>,
    metadata: Option<Value>,
}

async fn load_product_context(
    client: &(impl GenericClient + Sync),
    product_id: &str,
    request_id: Option<&str>,
) -> Result<ProductContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT p.asset_version_id::text,
                    p.seller_org_id::text,
                    seller.status,
                    COALESCE(seller.metadata, '{}'::jsonb),
                    v.status
             FROM catalog.product p
             JOIN catalog.asset_version v ON v.asset_version_id = p.asset_version_id
             JOIN core.organization seller ON seller.org_id = p.seller_org_id
             WHERE p.product_id = $1::text::uuid
             FOR UPDATE OF p",
            &[&product_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(product_not_found(product_id, request_id));
    };

    let asset_version_status: String = row.get(4);
    if !matches!(asset_version_status.as_str(), "active" | "published") {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: asset version status is not active/published",
            request_id,
        ));
    }

    Ok(ProductContext {
        asset_version_id: row.get(0),
        seller_org_id: row.get(1),
        seller_status: row.get(2),
        seller_metadata: row.get(3),
        asset_version_status,
    })
}

async fn load_existing_surface(
    client: &(impl GenericClient + Sync),
    asset_version_id: &str,
    payload: &ManageQuerySurfaceRequest,
    request_id: Option<&str>,
) -> Result<ExistingSurface, (StatusCode, Json<ErrorResponse>)> {
    let target_surface_type = payload
        .surface_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_SURFACE_TYPE);

    let row = if let Some(query_surface_id) = payload
        .query_surface_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        client
            .query_opt(
                "SELECT query_surface_id::text,
                        asset_object_id::text,
                        environment_id::text,
                        surface_type,
                        binding_mode,
                        execution_scope,
                        input_contract_json,
                        output_boundary_json,
                        query_policy_json,
                        quota_policy_json,
                        status,
                        metadata
                 FROM catalog.query_surface_definition
                 WHERE query_surface_id = $1::text::uuid
                   AND asset_version_id = $2::text::uuid",
                &[&query_surface_id, &asset_version_id],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_opt(
                "SELECT query_surface_id::text,
                        asset_object_id::text,
                        environment_id::text,
                        surface_type,
                        binding_mode,
                        execution_scope,
                        input_contract_json,
                        output_boundary_json,
                        query_policy_json,
                        quota_policy_json,
                        status,
                        metadata
                 FROM catalog.query_surface_definition
                 WHERE asset_version_id = $1::text::uuid
                   AND surface_type = $2
                 ORDER BY updated_at DESC, query_surface_id DESC
                 LIMIT 1",
                &[&asset_version_id, &target_surface_type],
            )
            .await
            .map_err(map_db_error)?
    };

    if payload
        .query_surface_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
        && row.is_none()
    {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: query_surface_id does not belong to current product asset version",
            request_id,
        ));
    }

    Ok(
        row.map_or_else(ExistingSurface::default, |row| ExistingSurface {
            query_surface_id: row.get(0),
            asset_object_id: row.get(1),
            environment_id: row.get(2),
            surface_type: row.get(3),
            binding_mode: row.get(4),
            execution_scope: row.get(5),
            input_contract_json: Some(row.get(6)),
            output_boundary_json: Some(row.get(7)),
            query_policy_json: Some(row.get(8)),
            quota_policy_json: Some(row.get(9)),
            status: row.get(10),
            metadata: Some(row.get(11)),
        }),
    )
}

async fn load_execution_environment(
    client: &(impl GenericClient + Sync),
    environment_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT org_id::text, status
             FROM core.execution_environment
             WHERE environment_id = $1::text::uuid",
            &[&environment_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: execution environment not found",
            request_id,
        ));
    };

    let environment_org_id: Option<String> = row.get(0);
    let status: String = row.get(1);
    if !matches!(status.as_str(), "active" | "draft") {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: execution environment status is not active/draft",
            request_id,
        ));
    }
    if environment_org_id
        .as_deref()
        .is_some_and(|value| value != seller_org_id)
    {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: execution environment does not belong to current seller scope",
            request_id,
        ));
    }
    Ok(())
}

async fn load_asset_object(
    client: &(impl GenericClient + Sync),
    asset_version_id: &str,
    asset_object_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let exists = client
        .query_opt(
            "SELECT asset_object_id::text
             FROM catalog.asset_object_binding
             WHERE asset_object_id = $1::text::uuid
               AND asset_version_id = $2::text::uuid",
            &[&asset_object_id, &asset_version_id],
        )
        .await
        .map_err(map_db_error)?
        .is_some();

    if !exists {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: asset_object_id does not belong to current asset version",
            request_id,
        ));
    }
    Ok(())
}

fn validate_request(
    payload: &ManageQuerySurfaceRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload
        .environment_id
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("environment_id is required", request_id));
    }
    Ok(())
}

fn enforce_seller_subject_status(
    context: &ProductContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.seller_status != "active" {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: seller organization is not active",
            request_id,
        ));
    }
    if !is_subject_deliverable(&context.seller_metadata) {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: seller organization is blocked by subject risk policy",
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
        "query surface management is forbidden for current tenant scope",
        request_id,
    ))
}

fn normalized_enum(
    value: Option<&str>,
    default_value: &str,
    allowed: &[&str],
    field: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default_value)
        .to_ascii_lowercase();
    if allowed.iter().any(|allowed| *allowed == normalized) {
        return Ok(normalized);
    }
    Err(bad_request(
        &format!("{field} must be one of {}", allowed.join(", ")),
        request_id,
    ))
}

fn normalize_execution_scope(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_EXECUTION_SCOPE)
        .to_ascii_lowercase();
    if matches!(normalized.as_str(), "raw" | "raw_zone") {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: execution_scope cannot target raw zone",
            request_id,
        ));
    }
    if matches!(
        normalized.as_str(),
        "curated_zone" | "product_zone" | "result_zone"
    ) {
        return Ok(normalized);
    }
    Err(bad_request(
        "execution_scope must be one of curated_zone, product_zone, result_zone",
        request_id,
    ))
}

fn normalize_json_object(
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

fn validate_read_zones(
    execution_scope: &str,
    input_contract_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(execution_scope, "raw" | "raw_zone") {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: execution_scope cannot target raw zone",
            request_id,
        ));
    }

    let read_zones = extract_read_zones(input_contract_json);
    let allowed = [
        "curated",
        "product",
        "result",
        "curated_zone",
        "product_zone",
        "result_zone",
    ];
    for zone in read_zones {
        let normalized = zone.trim().to_ascii_lowercase();
        if matches!(normalized.as_str(), "raw" | "raw_zone") {
            return Err(conflict(
                "QUERY_SURFACE_MANAGE_FORBIDDEN: raw zone is not allowed in query surface read scope",
                request_id,
            ));
        }
        if !allowed
            .iter()
            .any(|allowed_zone| *allowed_zone == normalized)
        {
            return Err(conflict(
                &format!(
                    "QUERY_SURFACE_MANAGE_FORBIDDEN: unsupported read zone `{}`",
                    zone
                ),
                request_id,
            ));
        }
    }
    Ok(())
}

fn extract_read_zones(input_contract_json: &Value) -> Vec<String> {
    ["source_zones", "read_zones"]
        .into_iter()
        .filter_map(|key| input_contract_json.get(key).and_then(Value::as_array))
        .flat_map(|values| values.iter())
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn validate_output_boundary(
    output_boundary_json: &Value,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if output_boundary_json
        .get("allow_raw_export")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return Err(conflict(
            "QUERY_SURFACE_MANAGE_FORBIDDEN: allow_raw_export cannot be true",
            request_id,
        ));
    }

    if output_boundary_json
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
            "QUERY_SURFACE_MANAGE_FORBIDDEN: raw format export is not allowed",
            request_id,
        ));
    }

    for key in ["max_rows", "max_cells"] {
        if output_boundary_json
            .get(key)
            .and_then(Value::as_i64)
            .is_some_and(|value| value <= 0)
        {
            return Err(conflict(
                &format!("QUERY_SURFACE_MANAGE_FORBIDDEN: {key} must be > 0"),
                request_id,
            ));
        }
    }

    Ok(())
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

fn product_not_found(
    product_id: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("product not found: {product_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
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
