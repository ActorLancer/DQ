use super::outbox_repository::{
    build_delivery_receipt_outbox_payload, write_delivery_receipt_outbox_event,
};
use crate::modules::delivery::dto::{ManageTemplateGrantRequest, TemplateGrantResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, write_delivery_audit_event,
};
use crate::modules::order::repo::{
    ensure_order_deliverable_and_prepare_delivery, map_db_error, write_trade_audit_event,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

const DELIVERY_TEMPLATE_GRANT_EVENT: &str = "delivery.template_query.enable";
const DEFAULT_TEMPLATE_TYPE: &str = "sql_template";

pub async fn manage_template_grant(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageTemplateGrantRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<TemplateGrantResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_template_grant_context(&tx, order_id, request_id).await?;

    enforce_manage_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_qry_lite_state(&context, request_id)?;

    let existing = load_existing_grant(&tx, order_id, payload, &context, request_id).await?;
    let prepared = ensure_order_deliverable_and_prepare_delivery(
        &tx, order_id, actor_role, request_id, trace_id,
    )
    .await?;

    let query_surface_id = payload
        .query_surface_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| existing.query_surface_id.clone())
        .ok_or_else(|| bad_request("query_surface_id is required", request_id))?;
    let query_surface = load_query_surface(&tx, &query_surface_id, request_id).await?;
    enforce_query_surface_matches_order(&query_surface, &context, request_id)?;

    let asset_object_id = payload
        .asset_object_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| query_surface.asset_object_id.clone())
        .or_else(|| existing.asset_object_id.clone())
        .ok_or_else(|| {
            conflict(
                "TEMPLATE_GRANT_FORBIDDEN: asset_object_id is required for template grant",
                request_id,
            )
        })?;
    let environment_id = payload
        .environment_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| query_surface.environment_id.clone())
        .or_else(|| existing.environment_id.clone());

    let allowed_template_ids = normalize_allowed_template_ids(
        payload.allowed_template_ids.as_ref(),
        existing.allowed_template_ids.as_ref(),
        request_id,
    )?;
    let templates =
        load_allowed_templates(&tx, &query_surface_id, &allowed_template_ids, request_id).await?;
    let template_type = resolve_template_type(
        payload.template_type.as_deref(),
        existing.template_type.as_deref(),
        &templates,
        request_id,
    )?;
    let output_boundary_json = resolve_output_boundary(
        payload
            .output_boundary_json
            .clone()
            .or(existing.output_boundary_json.clone()),
        &query_surface.output_boundary_json,
        &templates,
        request_id,
    )?;
    let run_quota_json = normalize_run_quota(
        payload
            .run_quota_json
            .clone()
            .or(existing.run_quota_json.clone()),
        request_id,
    )?;
    let execution_rule_snapshot = build_execution_rule_snapshot(
        payload
            .execution_rule_snapshot
            .clone()
            .or(existing.execution_rule_snapshot.clone()),
        &query_surface_id,
        &templates,
        &allowed_template_ids,
        &output_boundary_json,
        &run_quota_json,
        actor_role,
        request_id,
    )?;
    let template_digest = build_template_digest(&query_surface_id, &allowed_template_ids);

    let operation = if existing.template_query_grant_id.is_some() {
        "updated"
    } else {
        "granted"
    };
    let row = if let Some(template_query_grant_id) = existing.template_query_grant_id.as_deref() {
        tx.query_one(
            "UPDATE delivery.template_query_grant
             SET asset_object_id = $2::text::uuid,
                 environment_id = $3::text::uuid,
                 query_surface_id = $4::text::uuid,
                 template_type = $5,
                 template_digest = $6,
                 allowed_template_ids = $7::jsonb,
                 execution_rule_snapshot = $8::jsonb,
                 output_boundary_json = $9::jsonb,
                 run_quota_json = $10::jsonb,
                 grant_status = 'active',
                 updated_at = now()
             WHERE template_query_grant_id = $1::text::uuid
             RETURNING template_query_grant_id::text,
                       order_id::text,
                       asset_object_id::text,
                       environment_id::text,
                       query_surface_id::text,
                       template_type,
                       template_digest,
                       allowed_template_ids,
                       execution_rule_snapshot,
                       output_boundary_json,
                       run_quota_json,
                       grant_status,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &template_query_grant_id,
                &asset_object_id,
                &environment_id,
                &query_surface_id,
                &template_type,
                &template_digest,
                &json!(allowed_template_ids),
                &execution_rule_snapshot,
                &output_boundary_json,
                &run_quota_json,
            ],
        )
        .await
        .map_err(map_db_error)?
    } else {
        tx.query_one(
            "INSERT INTO delivery.template_query_grant (
               order_id,
               asset_object_id,
               environment_id,
               query_surface_id,
               template_type,
               template_digest,
               allowed_template_ids,
               execution_rule_snapshot,
               output_boundary_json,
               run_quota_json,
               grant_status
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4::text::uuid,
               $5,
               $6,
               $7::jsonb,
               $8::jsonb,
               $9::jsonb,
               $10::jsonb,
               'active'
             )
             RETURNING template_query_grant_id::text,
                       order_id::text,
                       asset_object_id::text,
                       environment_id::text,
                       query_surface_id::text,
                       template_type,
                       template_digest,
                       allowed_template_ids,
                       execution_rule_snapshot,
                       output_boundary_json,
                       run_quota_json,
                       grant_status,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &asset_object_id,
                &environment_id,
                &query_surface_id,
                &template_type,
                &template_digest,
                &json!(allowed_template_ids),
                &execution_rule_snapshot,
                &output_boundary_json,
                &run_quota_json,
            ],
        )
        .await
        .map_err(map_db_error)?
    };

    let target_state = if context.current_state == "buyer_locked" {
        "template_authorized"
    } else {
        "template_authorized"
    };
    let delivery_status = update_qry_lite_order_state(
        &tx,
        order_id,
        target_state,
        &context.payment_status,
        if operation == "updated" {
            "qry_lite_template_grant_updated"
        } else {
            "qry_lite_template_authorized"
        },
    )
    .await?;
    update_template_delivery_record(
        &tx,
        order_id,
        Some(prepared.delivery_id.as_str()),
        context.committed_delivery_id.as_deref(),
        &template_digest,
    )
    .await?;

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
    write_delivery_audit_event(
        &tx,
        "template_query_grant",
        &row.get::<_, String>(0),
        actor_role,
        DELIVERY_TEMPLATE_GRANT_EVENT,
        operation,
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "query_surface_id": query_surface_id,
            "allowed_template_ids": allowed_template_ids,
            "template_type": template_type,
            "current_state": target_state,
            "delivery_status": delivery_status,
        }),
    )
    .await?;
    let delivery_id = context
        .committed_delivery_id
        .clone()
        .unwrap_or_else(|| prepared.delivery_id.clone());
    let acceptance_status = "not_started";
    let settlement_status = if context.payment_status == "paid" {
        "pending_settlement"
    } else {
        "not_started"
    };
    write_delivery_receipt_outbox_event(
        &tx,
        &delivery_id,
        &build_delivery_receipt_outbox_payload(
            "template",
            order_id,
            &delivery_id,
            &context.sku_type,
            actor_role,
            &context.buyer_org_id,
            &context.seller_org_id,
            target_state,
            &context.payment_status,
            &delivery_status,
            acceptance_status,
            settlement_status,
            "none",
            Some(template_digest.as_str()),
            Some(template_digest.as_str()),
            Some("template_grant"),
            Some("template_query"),
            None,
            json!({
                "template_query_grant_id": row.get::<_, String>(0),
                "query_surface_id": query_surface_id,
                "template_type": template_type,
                "template_digest": template_digest,
                "allowed_template_ids": allowed_template_ids,
                "operation": operation,
            }),
        ),
        request_id,
        trace_id,
        idempotency_key,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(map_template_grant_response(
        &row,
        &context.sku_id,
        &context.sku_type,
        target_state,
        &context.payment_status,
        &delivery_status,
        operation,
    ))
}

#[derive(Debug)]
struct TemplateGrantContext {
    buyer_org_id: String,
    seller_org_id: String,
    asset_version_id: String,
    sku_id: String,
    sku_type: String,
    current_state: String,
    payment_status: String,
    committed_delivery_id: Option<String>,
    active_grant_id: Option<String>,
    active_query_surface_id: Option<String>,
    active_asset_object_id: Option<String>,
    active_environment_id: Option<String>,
    active_template_type: Option<String>,
    active_allowed_template_ids: Option<Value>,
    active_execution_rule_snapshot: Option<Value>,
    active_output_boundary_json: Option<Value>,
    active_run_quota_json: Option<Value>,
}

#[derive(Debug, Default)]
struct ExistingGrant {
    template_query_grant_id: Option<String>,
    query_surface_id: Option<String>,
    asset_object_id: Option<String>,
    environment_id: Option<String>,
    template_type: Option<String>,
    allowed_template_ids: Option<Vec<String>>,
    execution_rule_snapshot: Option<Value>,
    output_boundary_json: Option<Value>,
    run_quota_json: Option<Value>,
}

#[derive(Debug)]
struct QuerySurfaceContext {
    asset_version_id: String,
    asset_object_id: Option<String>,
    environment_id: Option<String>,
    surface_type: String,
    output_boundary_json: Value,
    status: String,
}

#[derive(Debug)]
struct TemplateDefinition {
    query_template_id: String,
    template_name: String,
    template_type: String,
    version_no: i32,
    parameter_schema_json: Value,
    analysis_rule_json: Value,
    result_schema_json: Value,
    export_policy_json: Value,
}

async fn load_template_grant_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<TemplateGrantContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.buyer_org_id::text,
                    o.seller_org_id::text,
                    o.asset_version_id::text,
                    o.sku_id::text,
                    s.sku_type,
                    o.status,
                    o.payment_status,
                    committed.delivery_id::text,
                    active.template_query_grant_id::text,
                    active.query_surface_id::text,
                    active.asset_object_id::text,
                    active.environment_id::text,
                    active.template_type,
                    active.allowed_template_ids,
                    active.execution_rule_snapshot,
                    active.output_boundary_json,
                    active.run_quota_json
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             LEFT JOIN LATERAL (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
                 AND status = 'committed'
               ORDER BY committed_at DESC NULLS LAST, created_at DESC, delivery_id DESC
               LIMIT 1
             ) committed ON true
             LEFT JOIN LATERAL (
               SELECT template_query_grant_id,
                      query_surface_id::text,
                      asset_object_id::text,
                      environment_id::text,
                      template_type,
                      allowed_template_ids,
                      execution_rule_snapshot,
                      output_boundary_json,
                      run_quota_json
               FROM delivery.template_query_grant
               WHERE order_id = o.order_id
                 AND grant_status = 'active'
               ORDER BY updated_at DESC, template_query_grant_id DESC
               LIMIT 1
             ) active ON true
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };

    Ok(TemplateGrantContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        asset_version_id: row.get(2),
        sku_id: row.get(3),
        sku_type: row.get(4),
        current_state: row.get(5),
        payment_status: row.get(6),
        committed_delivery_id: row.get(7),
        active_grant_id: row.get(8),
        active_query_surface_id: row.get(9),
        active_asset_object_id: row.get(10),
        active_environment_id: row.get(11),
        active_template_type: row.get(12),
        active_allowed_template_ids: row.get(13),
        active_execution_rule_snapshot: row.get(14),
        active_output_boundary_json: row.get(15),
        active_run_quota_json: row.get(16),
    })
}

async fn load_existing_grant(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    payload: &ManageTemplateGrantRequest,
    context: &TemplateGrantContext,
    request_id: Option<&str>,
) -> Result<ExistingGrant, (StatusCode, Json<ErrorResponse>)> {
    if let Some(grant_id) = payload
        .template_query_grant_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let row = client
            .query_opt(
                "SELECT template_query_grant_id::text,
                        query_surface_id::text,
                        asset_object_id::text,
                        environment_id::text,
                        template_type,
                        allowed_template_ids,
                        execution_rule_snapshot,
                        output_boundary_json,
                        run_quota_json
                 FROM delivery.template_query_grant
                 WHERE template_query_grant_id = $1::text::uuid
                   AND order_id = $2::text::uuid",
                &[&grant_id, &order_id],
            )
            .await
            .map_err(map_db_error)?;
        let Some(row) = row else {
            return Err(conflict(
                "TEMPLATE_GRANT_FORBIDDEN: template_query_grant_id does not belong to current order",
                request_id,
            ));
        };
        return Ok(ExistingGrant {
            template_query_grant_id: row.get(0),
            query_surface_id: row.get(1),
            asset_object_id: row.get(2),
            environment_id: row.get(3),
            template_type: row.get(4),
            allowed_template_ids: row.get::<_, Option<Value>>(5).and_then(parse_string_array),
            execution_rule_snapshot: row.get(6),
            output_boundary_json: row.get(7),
            run_quota_json: row.get(8),
        });
    }

    if context.active_grant_id.is_some() {
        return Ok(ExistingGrant {
            template_query_grant_id: context.active_grant_id.clone(),
            query_surface_id: context.active_query_surface_id.clone(),
            asset_object_id: context.active_asset_object_id.clone(),
            environment_id: context.active_environment_id.clone(),
            template_type: context.active_template_type.clone(),
            allowed_template_ids: context
                .active_allowed_template_ids
                .clone()
                .and_then(parse_string_array),
            execution_rule_snapshot: context.active_execution_rule_snapshot.clone(),
            output_boundary_json: context.active_output_boundary_json.clone(),
            run_quota_json: context.active_run_quota_json.clone(),
        });
    }

    Ok(ExistingGrant::default())
}

async fn load_query_surface(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    request_id: Option<&str>,
) -> Result<QuerySurfaceContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT asset_version_id::text,
                    asset_object_id::text,
                    environment_id::text,
                    surface_type,
                    output_boundary_json,
                    status
             FROM catalog.query_surface_definition
             WHERE query_surface_id = $1::text::uuid",
            &[&query_surface_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(
            &format!("query surface not found: {query_surface_id}"),
            request_id,
        ));
    };

    Ok(QuerySurfaceContext {
        asset_version_id: row.get(0),
        asset_object_id: row.get(1),
        environment_id: row.get(2),
        surface_type: row.get(3),
        output_boundary_json: row.get(4),
        status: row.get(5),
    })
}

async fn load_allowed_templates(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    allowed_template_ids: &[String],
    request_id: Option<&str>,
) -> Result<Vec<TemplateDefinition>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT query_template_id::text,
                    template_name,
                    template_type,
                    version_no,
                    parameter_schema_json,
                    analysis_rule_json,
                    result_schema_json,
                    export_policy_json,
                    status
             FROM delivery.query_template_definition
             WHERE query_surface_id = $1::text::uuid
               AND query_template_id::text = ANY($2::text[])",
            &[&query_surface_id, &allowed_template_ids],
        )
        .await
        .map_err(map_db_error)?;

    if rows.len() != allowed_template_ids.len() {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: allowed_template_ids contains template outside current query surface",
            request_id,
        ));
    }

    let mut templates = Vec::with_capacity(rows.len());
    for row in rows {
        let status: String = row.get(8);
        if status != "active" {
            return Err(conflict(
                "TEMPLATE_GRANT_FORBIDDEN: allowed templates must be active",
                request_id,
            ));
        }
        templates.push(TemplateDefinition {
            query_template_id: row.get(0),
            template_name: row.get(1),
            template_type: row.get(2),
            version_no: row.get(3),
            parameter_schema_json: row.get(4),
            analysis_rule_json: row.get(5),
            result_schema_json: row.get(6),
            export_policy_json: row.get(7),
        });
    }
    templates.sort_by(|left, right| left.query_template_id.cmp(&right.query_template_id));
    Ok(templates)
}

fn enforce_manage_scope(
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
        "seller_operator" | "seller_storage_operator" | "sandbox_operator"
    ) && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    if actor_role == "tenant_developer" && tenant_id == Some(buyer_org_id) {
        return Ok(());
    }
    if actor_role == "tenant_admin"
        && (tenant_id == Some(buyer_org_id) || tenant_id == Some(seller_org_id))
    {
        return Ok(());
    }
    Err(forbidden(
        "template query enable is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_qry_lite_state(
    context: &TemplateGrantContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type != "QRY_LITE" {
        return Err(conflict(
            &format!(
                "TEMPLATE_GRANT_FORBIDDEN: order sku_type `{}` is not QRY_LITE",
                context.sku_type
            ),
            request_id,
        ));
    }
    if !matches!(
        context.current_state.as_str(),
        "buyer_locked" | "template_authorized"
    ) {
        return Err(conflict(
            &format!(
                "TEMPLATE_GRANT_FORBIDDEN: current_state `{}` does not allow template grant enablement",
                context.current_state
            ),
            request_id,
        ));
    }
    Ok(())
}

fn enforce_query_surface_matches_order(
    query_surface: &QuerySurfaceContext,
    context: &TemplateGrantContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if query_surface.status != "active" {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: query surface is not active",
            request_id,
        ));
    }
    if query_surface.surface_type != "template_query_lite" {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: query surface is not template_query_lite",
            request_id,
        ));
    }
    if query_surface.asset_version_id != context.asset_version_id {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: query surface does not belong to current order asset version",
            request_id,
        ));
    }
    Ok(())
}

fn normalize_allowed_template_ids(
    payload: Option<&Vec<String>>,
    existing: Option<&Vec<String>>,
    request_id: Option<&str>,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
    let ids = payload
        .cloned()
        .or_else(|| existing.cloned())
        .unwrap_or_default();
    let normalized = ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        return Err(bad_request("allowed_template_ids is required", request_id));
    }
    if normalized.len() != normalized.iter().collect::<BTreeSet<_>>().len() {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: allowed_template_ids contains duplicate template ids",
            request_id,
        ));
    }
    Ok(normalized)
}

fn resolve_template_type(
    payload_template_type: Option<&str>,
    existing_template_type: Option<&str>,
    templates: &[TemplateDefinition],
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let inferred = templates
        .first()
        .map(|template| template.template_type.as_str())
        .unwrap_or(DEFAULT_TEMPLATE_TYPE);
    if templates
        .iter()
        .any(|template| template.template_type.as_str() != inferred)
    {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: allowed_template_ids must share the same template_type",
            request_id,
        ));
    }

    let template_type = payload_template_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or(existing_template_type)
        .unwrap_or(inferred)
        .to_string();
    if template_type != inferred {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: template_type does not match selected templates",
            request_id,
        ));
    }
    Ok(template_type)
}

fn resolve_output_boundary(
    requested: Option<Value>,
    surface_boundary_json: &Value,
    templates: &[TemplateDefinition],
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut boundary = normalize_object(
        requested.or(Some(surface_boundary_json.clone())),
        "output_boundary_json",
        request_id,
    )?;
    if boundary
        .get("allow_raw_export")
        .and_then(Value::as_bool)
        .is_some_and(|value| value)
    {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: allow_raw_export cannot be true",
            request_id,
        ));
    }

    let surface_formats = extract_string_array(surface_boundary_json, "allowed_formats")?;
    let mut allowed_formats = extract_string_array(&boundary, "allowed_formats")?;
    if allowed_formats.is_empty() {
        allowed_formats = surface_formats.clone();
    }
    if !surface_formats.is_empty()
        && allowed_formats
            .iter()
            .any(|format| !surface_formats.contains(format))
    {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: output formats exceed query surface boundary",
            request_id,
        ));
    }
    for template in templates {
        let template_formats =
            extract_string_array(&template.export_policy_json, "allowed_formats")?;
        if !template_formats.is_empty()
            && allowed_formats
                .iter()
                .any(|format| !template_formats.contains(format))
        {
            return Err(conflict(
                "TEMPLATE_GRANT_FORBIDDEN: output formats exceed template export policy",
                request_id,
            ));
        }
        if template
            .export_policy_json
            .get("allow_raw_export")
            .and_then(Value::as_bool)
            .is_some_and(|value| value)
        {
            return Err(conflict(
                "TEMPLATE_GRANT_FORBIDDEN: selected template cannot allow raw export",
                request_id,
            ));
        }
    }

    let max_rows_limit = min_positive_i64(
        parse_positive_i64(surface_boundary_json, "max_rows", request_id)?,
        templates
            .iter()
            .map(|template| {
                parse_positive_i64(&template.export_policy_json, "max_export_rows", request_id)
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    let requested_max_rows = parse_positive_i64(&boundary, "max_rows", request_id)?;
    if let (Some(limit), Some(value)) = (max_rows_limit, requested_max_rows)
        && value > limit
    {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: max_rows exceeds query surface/template boundary",
            request_id,
        ));
    }
    let max_cells_limit = min_positive_i64(
        parse_positive_i64(surface_boundary_json, "max_cells", request_id)?,
        templates
            .iter()
            .map(|template| {
                parse_positive_i64(&template.export_policy_json, "max_export_cells", request_id)
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    let requested_max_cells = parse_positive_i64(&boundary, "max_cells", request_id)?;
    if let (Some(limit), Some(value)) = (max_cells_limit, requested_max_cells)
        && value > limit
    {
        return Err(conflict(
            "TEMPLATE_GRANT_FORBIDDEN: max_cells exceeds query surface/template boundary",
            request_id,
        ));
    }

    let object = boundary
        .as_object_mut()
        .ok_or_else(|| bad_request("output_boundary_json must be an object", request_id))?;
    object.insert("allow_raw_export".to_string(), Value::Bool(false));
    object.insert(
        "allowed_formats".to_string(),
        Value::Array(allowed_formats.into_iter().map(Value::String).collect()),
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
    Ok(boundary)
}

fn normalize_run_quota(
    value: Option<Value>,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let value = normalize_object(value, "run_quota_json", request_id)?;
    for field in ["max_runs", "daily_limit", "monthly_limit"] {
        parse_positive_i64(&value, field, request_id)?;
    }
    Ok(value)
}

fn build_execution_rule_snapshot(
    requested: Option<Value>,
    query_surface_id: &str,
    templates: &[TemplateDefinition],
    allowed_template_ids: &[String],
    output_boundary_json: &Value,
    run_quota_json: &Value,
    actor_role: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut snapshot = normalize_object(requested, "execution_rule_snapshot", request_id)?;
    let object = snapshot
        .as_object_mut()
        .ok_or_else(|| bad_request("execution_rule_snapshot must be an object", request_id))?;
    object.insert(
        "query_surface_id".to_string(),
        Value::String(query_surface_id.to_string()),
    );
    object.insert(
        "allowed_template_ids".to_string(),
        Value::Array(
            allowed_template_ids
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );
    object.insert(
        "template_summaries".to_string(),
        Value::Array(
            templates
                .iter()
                .map(|template| {
                    json!({
                        "query_template_id": template.query_template_id,
                        "template_name": template.template_name,
                        "template_type": template.template_type,
                        "version_no": template.version_no,
                        "parameter_schema_json": template.parameter_schema_json,
                        "result_schema_json": template.result_schema_json,
                        "analysis_rule_json": template.analysis_rule_json,
                    })
                })
                .collect(),
        ),
    );
    object.insert(
        "output_boundary_json".to_string(),
        output_boundary_json.clone(),
    );
    object.insert("run_quota_json".to_string(), run_quota_json.clone());
    object.insert(
        "granted_by_role".to_string(),
        Value::String(actor_role.to_string()),
    );
    Ok(snapshot)
}

fn build_template_digest(query_surface_id: &str, allowed_template_ids: &[String]) -> String {
    let mut normalized = allowed_template_ids.to_vec();
    normalized.sort();
    format!(
        "sha256:template-grant:{query_surface_id}:{}",
        normalized.join(",")
    )
}

async fn update_template_delivery_record(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    prepared_delivery_id: Option<&str>,
    committed_delivery_id: Option<&str>,
    template_digest: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(delivery_id) = committed_delivery_id {
        client
            .execute(
                "UPDATE delivery.delivery_record
                 SET status = 'committed',
                     delivery_type = 'template_grant',
                     delivery_route = 'template_query',
                     delivery_commit_hash = $2,
                     receipt_hash = $2,
                     committed_at = COALESCE(committed_at, now()),
                     updated_at = now()
                 WHERE delivery_id = $1::text::uuid",
                &[&delivery_id, &template_digest],
            )
            .await
            .map_err(map_db_error)?;
        return Ok(());
    }

    let prepared_delivery_id = prepared_delivery_id.ok_or_else(|| {
        conflict(
            &format!(
                "TEMPLATE_GRANT_FORBIDDEN: prepared delivery record not found for order `{order_id}`"
            ),
            None,
        )
    })?;
    client
        .execute(
            "UPDATE delivery.delivery_record
             SET status = 'committed',
                 delivery_type = 'template_grant',
                 delivery_route = 'template_query',
                 delivery_commit_hash = $2,
                 receipt_hash = $2,
                 committed_at = now(),
                 updated_at = now()
             WHERE delivery_id = $1::text::uuid",
            &[&prepared_delivery_id, &template_digest],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

async fn update_qry_lite_order_state(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    target_state: &str,
    payment_status: &str,
    reason_code: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let delivery_status = "in_progress".to_string();
    let acceptance_status = "not_started".to_string();
    let settlement_status = if payment_status == "paid" {
        "pending_settlement".to_string()
    } else {
        "not_started".to_string()
    };
    client
        .execute(
            "UPDATE trade.order_main
             SET status = $2,
                 payment_status = $3,
                 delivery_status = $4,
                 acceptance_status = $5,
                 settlement_status = $6,
                 dispute_status = 'none',
                 last_reason_code = $7,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &target_state,
                &payment_status,
                &delivery_status,
                &acceptance_status,
                &settlement_status,
                &reason_code,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(delivery_status)
}

fn map_template_grant_response(
    row: &db::Row,
    sku_id: &str,
    sku_type: &str,
    current_state: &str,
    payment_status: &str,
    delivery_status: &str,
    operation: &str,
) -> TemplateGrantResponseData {
    TemplateGrantResponseData {
        template_query_grant_id: row.get(0),
        order_id: row.get(1),
        asset_object_id: row.get(2),
        environment_id: row.get(3),
        query_surface_id: row.get(4),
        sku_id: sku_id.to_string(),
        sku_type: sku_type.to_string(),
        template_type: row.get(5),
        template_digest: row.get(6),
        allowed_template_ids: parse_string_array(row.get::<_, Value>(7)).unwrap_or_default(),
        execution_rule_snapshot: row.get(8),
        output_boundary_json: row.get(9),
        run_quota_json: row.get(10),
        grant_status: row.get(11),
        operation: operation.to_string(),
        current_state: current_state.to_string(),
        payment_status: payment_status.to_string(),
        delivery_status: delivery_status.to_string(),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn parse_string_array(value: Value) -> Option<Vec<String>> {
    value.as_array().map(|values| {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>()
    })
}

fn normalize_object(
    value: Option<Value>,
    field: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let value = value.unwrap_or_else(|| Value::Object(Map::new()));
    if value.is_object() {
        return Ok(value);
    }
    Err(bad_request(
        &format!("{field} must be a JSON object"),
        request_id,
    ))
}

fn extract_string_array(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, (StatusCode, Json<ErrorResponse>)> {
    let Some(raw_values) = value.get(field) else {
        return Ok(Vec::new());
    };
    let Some(raw_values) = raw_values.as_array() else {
        return Err(bad_request(
            &format!("{field} must be an array of strings"),
            None,
        ));
    };
    Ok(raw_values
        .iter()
        .filter_map(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect())
}

fn parse_positive_i64(
    value: &Value,
    field: &str,
    request_id: Option<&str>,
) -> Result<Option<i64>, (StatusCode, Json<ErrorResponse>)> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let parsed = raw
        .as_i64()
        .or_else(|| raw.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| raw.as_str().and_then(|value| value.parse::<i64>().ok()));
    match parsed {
        Some(parsed) if parsed > 0 => Ok(Some(parsed)),
        Some(_) => Err(conflict(
            &format!("TEMPLATE_GRANT_FORBIDDEN: {field} must be > 0"),
            request_id,
        )),
        None => Err(bad_request(
            &format!("{field} must be a positive integer"),
            request_id,
        )),
    }
}

fn min_positive_i64(base: Option<i64>, values: Vec<Option<i64>>) -> Option<i64> {
    values.into_iter().flatten().fold(base, |current, value| {
        Some(current.map_or(value, |existing| existing.min(value)))
    })
}

fn not_found(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: message.to_string(),
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
