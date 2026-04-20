use crate::modules::delivery::dto::{
    ManageSensitiveExecutionPolicyRequest, SensitiveExecutionPolicyModel,
    SensitiveExecutionPolicyResponseData,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

const DELIVERY_SENSITIVE_EXECUTION_MANAGE_EVENT: &str = "delivery.sensitive_execution.manage";

pub async fn manage_sensitive_execution_policy(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageSensitiveExecutionPolicyRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<SensitiveExecutionPolicyResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_order_context(&tx, order_id, request_id).await?;

    enforce_manage_scope(actor_role, tenant_id, &context.seller_org_id, request_id)?;

    if context.payment_status != "paid" {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: order payment_status must be `paid`",
            request_id,
        ));
    }

    let execution_mode = resolve_execution_mode(&context, payload, request_id)?;
    let policy_scope = resolve_policy_scope(payload, &execution_mode, request_id)?;

    if matches!(
        execution_mode.as_str(),
        "seller_hosted_api" | "report_result"
    ) && (payload.query_surface_id.as_deref().is_some()
        || payload.template_query_grant_id.as_deref().is_some()
        || payload.sandbox_workspace_id.as_deref().is_some())
    {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: API/report policies do not bind query_surface_id, template_query_grant_id, or sandbox_workspace_id in V1",
            request_id,
        ));
    }

    let template_grant = load_template_grant_ref(&tx, order_id, payload, request_id).await?;
    let sandbox_workspace = load_sandbox_workspace_ref(&tx, order_id, payload, request_id).await?;
    let query_surface_id = resolve_query_surface_id(
        payload,
        &execution_mode,
        &template_grant,
        &sandbox_workspace,
        request_id,
    )?;
    let query_surface = if let Some(query_surface_id) = query_surface_id.as_deref() {
        Some(
            load_query_surface_ref(&tx, query_surface_id, &context.asset_version_id, request_id)
                .await?,
        )
    } else {
        None
    };

    if let (Some(template_grant), Some(query_surface)) = (&template_grant, &query_surface)
        && template_grant.query_surface_id != query_surface.query_surface_id
    {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: template grant query_surface_id does not match payload query_surface_id",
            request_id,
        ));
    }
    if let (Some(sandbox_workspace), Some(query_surface)) = (&sandbox_workspace, &query_surface)
        && sandbox_workspace.query_surface_id != query_surface.query_surface_id
    {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: sandbox workspace query_surface_id does not match payload query_surface_id",
            request_id,
        ));
    }

    let output_boundary_json = resolve_output_boundary_json(
        payload.output_boundary_json.clone(),
        query_surface.as_ref(),
        template_grant.as_ref(),
        sandbox_workspace.as_ref(),
        request_id,
    )?;
    let export_control_json = resolve_export_control_json(
        payload.export_control_json.clone(),
        &execution_mode,
        query_surface.as_ref(),
        &output_boundary_json,
        sandbox_workspace.as_ref(),
        request_id,
    )?;

    let step_up_required = payload.step_up_required.unwrap_or_else(|| {
        query_surface
            .as_ref()
            .and_then(|surface| {
                surface
                    .query_policy_json
                    .get("step_up_required")
                    .and_then(Value::as_bool)
            })
            .unwrap_or(false)
    });
    let attestation_required = payload.attestation_required.unwrap_or_else(|| {
        query_surface
            .as_ref()
            .and_then(|surface| {
                surface
                    .query_policy_json
                    .get("attestation_required")
                    .and_then(Value::as_bool)
                    .or_else(|| {
                        surface
                            .query_policy_json
                            .get("requires_attestation")
                            .and_then(Value::as_bool)
                    })
            })
            .unwrap_or(false)
    });
    let approval_ticket_id = payload
        .approval_ticket_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let policy_snapshot_json = build_policy_snapshot_json(
        &context,
        &policy_scope,
        &execution_mode,
        query_surface.as_ref(),
        template_grant.as_ref(),
        sandbox_workspace.as_ref(),
        &output_boundary_json,
        &export_control_json,
        step_up_required,
        attestation_required,
        approval_ticket_id.as_deref(),
    );

    let existing_policy_id = load_existing_policy_id(
        &tx,
        order_id,
        payload.sensitive_execution_policy_id.as_deref(),
        &policy_scope,
        &execution_mode,
        template_grant
            .as_ref()
            .map(|value| value.template_query_grant_id.as_str()),
        sandbox_workspace
            .as_ref()
            .map(|value| value.sandbox_workspace_id.as_str()),
        request_id,
    )
    .await?;

    let (row, operation) = if let Some(existing_policy_id) = existing_policy_id.as_deref() {
        (
            tx.query_one(
                "UPDATE delivery.sensitive_execution_policy
                 SET query_surface_id = $2::text::uuid,
                     template_query_grant_id = $3::text::uuid,
                     sandbox_workspace_id = $4::text::uuid,
                     policy_scope = $5,
                     execution_mode = $6,
                     output_boundary_json = $7::jsonb,
                     export_control_json = $8::jsonb,
                     step_up_required = $9,
                     attestation_required = $10,
                     approval_ticket_id = $11::text::uuid,
                     policy_snapshot = $12::jsonb,
                     status = 'active',
                     updated_at = now()
                 WHERE sensitive_execution_policy_id = $1::text::uuid
                 RETURNING sensitive_execution_policy_id::text,
                           order_id::text,
                           query_surface_id::text,
                           template_query_grant_id::text,
                           sandbox_workspace_id::text,
                           policy_scope,
                           execution_mode,
                           status,
                           output_boundary_json,
                           export_control_json,
                           policy_snapshot,
                           step_up_required,
                           attestation_required,
                           approval_ticket_id::text,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &existing_policy_id,
                    &query_surface_id,
                    &template_grant
                        .as_ref()
                        .map(|value| value.template_query_grant_id.clone()),
                    &sandbox_workspace
                        .as_ref()
                        .map(|value| value.sandbox_workspace_id.clone()),
                    &policy_scope,
                    &execution_mode,
                    &output_boundary_json,
                    &export_control_json,
                    &step_up_required,
                    &attestation_required,
                    &approval_ticket_id,
                    &policy_snapshot_json,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "updated",
        )
    } else {
        (
            tx.query_one(
                "INSERT INTO delivery.sensitive_execution_policy (
                   order_id,
                   query_surface_id,
                   template_query_grant_id,
                   sandbox_workspace_id,
                   policy_scope,
                   execution_mode,
                   output_boundary_json,
                   export_control_json,
                   step_up_required,
                   attestation_required,
                   approval_ticket_id,
                   policy_snapshot,
                   status
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4::text::uuid,
                   $5,
                   $6,
                   $7::jsonb,
                   $8::jsonb,
                   $9,
                   $10,
                   $11::text::uuid,
                   $12::jsonb,
                   'active'
                 )
                 RETURNING sensitive_execution_policy_id::text,
                           order_id::text,
                           query_surface_id::text,
                           template_query_grant_id::text,
                           sandbox_workspace_id::text,
                           policy_scope,
                           execution_mode,
                           status,
                           output_boundary_json,
                           export_control_json,
                           policy_snapshot,
                           step_up_required,
                           attestation_required,
                           approval_ticket_id::text,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &order_id,
                    &query_surface_id,
                    &template_grant
                        .as_ref()
                        .map(|value| value.template_query_grant_id.clone()),
                    &sandbox_workspace
                        .as_ref()
                        .map(|value| value.sandbox_workspace_id.clone()),
                    &policy_scope,
                    &execution_mode,
                    &output_boundary_json,
                    &export_control_json,
                    &step_up_required,
                    &attestation_required,
                    &approval_ticket_id,
                    &policy_snapshot_json,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "created",
        )
    };

    let sensitive_execution_policy_id: String = row.get(0);
    let response = SensitiveExecutionPolicyResponseData {
        sensitive_execution_policy_id: sensitive_execution_policy_id.clone(),
        order_id: row.get(1),
        sku_id: context.sku_id.clone(),
        sku_type: context.sku_type.clone(),
        current_state: context.current_state.clone(),
        payment_status: context.payment_status.clone(),
        delivery_status: context.delivery_status.clone(),
        acceptance_status: context.acceptance_status.clone(),
        settlement_status: context.settlement_status.clone(),
        dispute_status: context.dispute_status.clone(),
        operation: operation.to_string(),
        created_at: row.get(14),
        updated_at: row.get(15),
        policy: SensitiveExecutionPolicyModel {
            sensitive_execution_policy_id,
            query_surface_id: row.get(2),
            template_query_grant_id: row.get(3),
            sandbox_workspace_id: row.get(4),
            policy_scope: row.get(5),
            execution_mode: row.get(6),
            policy_status: row.get(7),
            output_boundary_json: row.get(8),
            export_control_json: row.get(9),
            policy_snapshot_json: row.get(10),
            step_up_required: row.get(11),
            attestation_required: row.get(12),
            approval_ticket_id: row.get(13),
        },
    };

    write_delivery_audit_event(
        &tx,
        "sensitive_execution_policy",
        &response.sensitive_execution_policy_id,
        actor_role,
        DELIVERY_SENSITIVE_EXECUTION_MANAGE_EVENT,
        operation,
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "sku_type": context.sku_type.clone(),
            "policy_scope": response.policy.policy_scope.clone(),
            "execution_mode": response.policy.execution_mode.clone(),
            "query_surface_id": response.policy.query_surface_id.clone(),
            "template_query_grant_id": response.policy.template_query_grant_id.clone(),
            "sandbox_workspace_id": response.policy.sandbox_workspace_id.clone(),
            "step_up_required": response.policy.step_up_required,
            "attestation_required": response.policy.attestation_required,
            "output_boundary_json": response.policy.output_boundary_json.clone(),
            "export_control_json": response.policy.export_control_json.clone(),
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(response)
}

#[derive(Debug)]
struct OrderContext {
    order_id: String,
    product_id: String,
    asset_version_id: String,
    seller_org_id: String,
    sku_id: String,
    sku_type: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    delivery_route_snapshot: Option<String>,
}

#[derive(Debug)]
struct QuerySurfaceRef {
    query_surface_id: String,
    output_boundary_json: Value,
    query_policy_json: Value,
}

#[derive(Debug)]
struct TemplateGrantRef {
    template_query_grant_id: String,
    query_surface_id: String,
    output_boundary_json: Value,
    run_quota_json: Value,
}

#[derive(Debug)]
struct SandboxWorkspaceRef {
    sandbox_workspace_id: String,
    query_surface_id: String,
    output_boundary_json: Value,
    export_policy_json: Value,
}

async fn load_order_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.order_id::text,
                    o.product_id::text,
                    o.asset_version_id::text,
                    o.seller_org_id::text,
                    o.sku_id::text,
                    s.sku_type,
                    o.status,
                    o.payment_status,
                    o.delivery_status,
                    o.acceptance_status,
                    o.settlement_status,
                    o.dispute_status,
                    o.delivery_route_snapshot
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };
    Ok(OrderContext {
        order_id: row.get(0),
        product_id: row.get(1),
        asset_version_id: row.get(2),
        seller_org_id: row.get(3),
        sku_id: row.get(4),
        sku_type: row.get(5),
        current_state: row.get(6),
        payment_status: row.get(7),
        delivery_status: row.get(8),
        acceptance_status: row.get(9),
        settlement_status: row.get(10),
        dispute_status: row.get(11),
        delivery_route_snapshot: row.get(12),
    })
}

async fn load_query_surface_ref(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    asset_version_id: &str,
    request_id: Option<&str>,
) -> Result<QuerySurfaceRef, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT query_surface_id::text,
                    output_boundary_json,
                    query_policy_json,
                    asset_version_id::text
             FROM catalog.query_surface_definition
             WHERE query_surface_id = $1::text::uuid",
            &[&query_surface_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(query_surface_id, request_id));
    };
    let surface_asset_version_id: String = row.get(3);
    if surface_asset_version_id != asset_version_id {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: query_surface does not belong to order asset_version",
            request_id,
        ));
    }
    Ok(QuerySurfaceRef {
        query_surface_id: row.get(0),
        output_boundary_json: row.get(1),
        query_policy_json: row.get(2),
    })
}

async fn load_template_grant_ref(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    payload: &ManageSensitiveExecutionPolicyRequest,
    request_id: Option<&str>,
) -> Result<Option<TemplateGrantRef>, (StatusCode, Json<ErrorResponse>)> {
    let Some(template_query_grant_id) = payload
        .template_query_grant_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let row = client
        .query_opt(
            "SELECT template_query_grant_id::text,
                    query_surface_id::text,
                    output_boundary_json,
                    run_quota_json,
                    grant_status
             FROM delivery.template_query_grant
             WHERE template_query_grant_id = $1::text::uuid
               AND order_id = $2::text::uuid",
            &[&template_query_grant_id, &order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(template_query_grant_id, request_id));
    };
    let grant_status: String = row.get(4);
    if grant_status != "active" {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: template query grant must be active",
            request_id,
        ));
    }
    Ok(Some(TemplateGrantRef {
        template_query_grant_id: row.get(0),
        query_surface_id: row.get(1),
        output_boundary_json: row.get(2),
        run_quota_json: row.get(3),
    }))
}

async fn load_sandbox_workspace_ref(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    payload: &ManageSensitiveExecutionPolicyRequest,
    request_id: Option<&str>,
) -> Result<Option<SandboxWorkspaceRef>, (StatusCode, Json<ErrorResponse>)> {
    let Some(sandbox_workspace_id) = payload
        .sandbox_workspace_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let row = client
        .query_opt(
            "SELECT sandbox_workspace_id::text,
                    query_surface_id::text,
                    output_boundary_json,
                    export_policy,
                    status
             FROM delivery.sandbox_workspace
             WHERE sandbox_workspace_id = $1::text::uuid
               AND order_id = $2::text::uuid",
            &[&sandbox_workspace_id, &order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(sandbox_workspace_id, request_id));
    };
    let status: String = row.get(4);
    if !matches!(status.as_str(), "active" | "provisioning" | "suspended") {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: sandbox workspace status does not allow policy management",
            request_id,
        ));
    }
    Ok(Some(SandboxWorkspaceRef {
        sandbox_workspace_id: row.get(0),
        query_surface_id: row.get(1),
        output_boundary_json: row.get(2),
        export_policy_json: row.get(3),
    }))
}

async fn load_existing_policy_id(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    explicit_policy_id: Option<&str>,
    policy_scope: &str,
    execution_mode: &str,
    template_query_grant_id: Option<&str>,
    sandbox_workspace_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if let Some(explicit_policy_id) = explicit_policy_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let row = client
            .query_opt(
                "SELECT sensitive_execution_policy_id::text
                 FROM delivery.sensitive_execution_policy
                 WHERE sensitive_execution_policy_id = $1::text::uuid
                   AND order_id = $2::text::uuid",
                &[&explicit_policy_id, &order_id],
            )
            .await
            .map_err(map_db_error)?;
        return row
            .map(|row| row.get::<_, String>(0))
            .ok_or_else(|| not_found(explicit_policy_id, request_id))
            .map(Some);
    }

    client
        .query_opt(
            "SELECT sensitive_execution_policy_id::text
             FROM delivery.sensitive_execution_policy
             WHERE order_id = $1::text::uuid
               AND policy_scope = $2
               AND execution_mode = $3
               AND (($4::text IS NULL AND template_query_grant_id IS NULL) OR template_query_grant_id = $4::text::uuid)
               AND (($5::text IS NULL AND sandbox_workspace_id IS NULL) OR sandbox_workspace_id = $5::text::uuid)
             ORDER BY updated_at DESC, sensitive_execution_policy_id DESC
             LIMIT 1",
            &[
                &order_id,
                &policy_scope,
                &execution_mode,
                &template_query_grant_id,
                &sandbox_workspace_id,
            ],
        )
        .await
        .map_err(map_db_error)
        .map(|row| row.map(|value| value.get::<_, String>(0)))
}

fn validate_request(
    payload: &ManageSensitiveExecutionPolicyRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(output_boundary_json) = payload.output_boundary_json.as_ref()
        && !output_boundary_json.is_object()
    {
        return Err(bad_request(
            "output_boundary_json must be an object",
            request_id,
        ));
    }
    if let Some(export_control_json) = payload.export_control_json.as_ref()
        && !export_control_json.is_object()
    {
        return Err(bad_request(
            "export_control_json must be an object",
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
    if matches!(
        actor_role,
        "seller_operator"
            | "seller_storage_operator"
            | "sandbox_operator"
            | "tenant_developer"
            | "tenant_admin"
    ) && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    Err(conflict(
        "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: tenant scope does not match seller org",
        request_id,
    ))
}

fn resolve_execution_mode(
    context: &OrderContext,
    payload: &ManageSensitiveExecutionPolicyRequest,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let derived = match context.sku_type.as_str() {
        "QRY_LITE" => Some("template_query_lite"),
        "SBX_STD" => Some("sandbox_query"),
        "API_SUB" | "API_PPU" => Some("seller_hosted_api"),
        "RPT_STD" => Some("report_result"),
        _ => None,
    }
    .ok_or_else(|| {
        conflict(
            &format!(
                "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: sku_type `{}` is not a V1 controlled execution path",
                context.sku_type
            ),
            request_id,
        )
    })?;

    let route = context
        .delivery_route_snapshot
        .as_deref()
        .unwrap_or_default();
    let route_ok = match derived {
        "template_query_lite" => route == "template_query",
        "sandbox_query" => route == "sandbox_query",
        "seller_hosted_api" => route == "seller_gateway",
        "report_result" => route == "report_delivery",
        _ => false,
    };
    if !route_ok {
        return Err(conflict(
            &format!(
                "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: delivery_route_snapshot `{route}` is not legal for execution_mode `{derived}`"
            ),
            request_id,
        ));
    }

    if let Some(requested) = payload
        .execution_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && requested != derived
    {
        return Err(conflict(
            &format!(
                "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: execution_mode `{requested}` does not match order path `{derived}`"
            ),
            request_id,
        ));
    }

    Ok(derived.to_string())
}

fn resolve_policy_scope(
    payload: &ManageSensitiveExecutionPolicyRequest,
    execution_mode: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let default_scope = if payload
        .sandbox_workspace_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        "sandbox_workspace"
    } else if payload
        .template_query_grant_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        "template_query_grant"
    } else {
        "order"
    };
    let scope = payload
        .policy_scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default_scope);

    let valid = match execution_mode {
        "template_query_lite" => matches!(scope, "order" | "template_query_grant"),
        "sandbox_query" => matches!(scope, "order" | "sandbox_workspace"),
        "seller_hosted_api" | "report_result" => scope == "order",
        _ => false,
    };
    if !valid {
        return Err(conflict(
            &format!(
                "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: policy_scope `{scope}` is not legal for execution_mode `{execution_mode}`"
            ),
            request_id,
        ));
    }
    Ok(scope.to_string())
}

fn resolve_query_surface_id(
    payload: &ManageSensitiveExecutionPolicyRequest,
    execution_mode: &str,
    template_grant: &Option<TemplateGrantRef>,
    sandbox_workspace: &Option<SandboxWorkspaceRef>,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let requested = payload
        .query_surface_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let derived = requested
        .clone()
        .or_else(|| {
            template_grant
                .as_ref()
                .map(|value| value.query_surface_id.clone())
        })
        .or_else(|| {
            sandbox_workspace
                .as_ref()
                .map(|value| value.query_surface_id.clone())
        });

    if matches!(execution_mode, "template_query_lite" | "sandbox_query") && derived.is_none() {
        return Err(conflict(
            "SENSITIVE_EXECUTION_POLICY_FORBIDDEN: query_surface_id is required for template_query_lite or sandbox_query",
            request_id,
        ));
    }

    Ok(derived)
}

fn resolve_output_boundary_json(
    payload_output_boundary_json: Option<Value>,
    query_surface: Option<&QuerySurfaceRef>,
    template_grant: Option<&TemplateGrantRef>,
    sandbox_workspace: Option<&SandboxWorkspaceRef>,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let json = payload_output_boundary_json
        .or_else(|| template_grant.map(|value| value.output_boundary_json.clone()))
        .or_else(|| sandbox_workspace.map(|value| value.output_boundary_json.clone()))
        .or_else(|| query_surface.map(|value| value.output_boundary_json.clone()))
        .unwrap_or_else(|| {
            json!({
                "allow_export": false,
                "allow_raw_export": false,
                "allowed_formats": [],
                "requires_disclosure_review": false,
            })
        });
    normalize_json_object(json, "output_boundary_json", request_id)
}

fn resolve_export_control_json(
    payload_export_control_json: Option<Value>,
    execution_mode: &str,
    query_surface: Option<&QuerySurfaceRef>,
    output_boundary_json: &Value,
    sandbox_workspace: Option<&SandboxWorkspaceRef>,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let default_network_access = query_surface
        .and_then(|surface| {
            surface
                .query_policy_json
                .get("network_access")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| match execution_mode {
            "seller_hosted_api" => "seller_gateway".to_string(),
            "sandbox_query" => "seller_vpc".to_string(),
            _ => "deny".to_string(),
        });
    let json = payload_export_control_json
        .or_else(|| sandbox_workspace.map(|value| value.export_policy_json.clone()))
        .unwrap_or_else(|| {
            json!({
                "allow_export": output_boundary_json
                    .get("allow_export")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                "allow_raw_export": output_boundary_json
                    .get("allow_raw_export")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                "allowed_formats": output_boundary_json
                    .get("allowed_formats")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(vec![])),
                "max_exports": 0,
                "network_access": default_network_access,
                "copy_control": "deny",
            })
        });
    normalize_json_object(json, "export_control_json", request_id)
}

fn build_policy_snapshot_json(
    context: &OrderContext,
    policy_scope: &str,
    execution_mode: &str,
    query_surface: Option<&QuerySurfaceRef>,
    template_grant: Option<&TemplateGrantRef>,
    sandbox_workspace: Option<&SandboxWorkspaceRef>,
    output_boundary_json: &Value,
    export_control_json: &Value,
    step_up_required: bool,
    attestation_required: bool,
    approval_ticket_id: Option<&str>,
) -> Value {
    let mut snapshot = Map::new();
    snapshot.insert(
        "order".to_string(),
        json!({
            "order_id": context.order_id,
            "sku_id": context.sku_id,
            "sku_type": context.sku_type,
            "current_state": context.current_state,
            "payment_status": context.payment_status,
            "delivery_status": context.delivery_status,
            "acceptance_status": context.acceptance_status,
            "settlement_status": context.settlement_status,
            "dispute_status": context.dispute_status,
            "delivery_route_snapshot": context.delivery_route_snapshot,
        }),
    );
    snapshot.insert(
        "policy_scope".to_string(),
        Value::String(policy_scope.to_string()),
    );
    snapshot.insert(
        "execution_mode".to_string(),
        Value::String(execution_mode.to_string()),
    );
    snapshot.insert(
        "output_boundary_json".to_string(),
        output_boundary_json.clone(),
    );
    snapshot.insert(
        "export_control_json".to_string(),
        export_control_json.clone(),
    );
    snapshot.insert(
        "step_up_required".to_string(),
        Value::Bool(step_up_required),
    );
    snapshot.insert(
        "attestation_required".to_string(),
        Value::Bool(attestation_required),
    );
    snapshot.insert(
        "approval_ticket_id".to_string(),
        approval_ticket_id
            .map(|value| Value::String(value.to_string()))
            .unwrap_or(Value::Null),
    );
    if let Some(query_surface) = query_surface {
        snapshot.insert(
            "query_surface".to_string(),
            json!({
                "query_surface_id": query_surface.query_surface_id,
                "query_policy_json": query_surface.query_policy_json,
            }),
        );
    }
    if let Some(template_grant) = template_grant {
        snapshot.insert(
            "template_query_grant".to_string(),
            json!({
                "template_query_grant_id": template_grant.template_query_grant_id,
                "query_surface_id": template_grant.query_surface_id,
                "run_quota_json": template_grant.run_quota_json,
            }),
        );
    }
    if let Some(sandbox_workspace) = sandbox_workspace {
        snapshot.insert(
            "sandbox_workspace".to_string(),
            json!({
                "sandbox_workspace_id": sandbox_workspace.sandbox_workspace_id,
                "query_surface_id": sandbox_workspace.query_surface_id,
            }),
        );
    }
    Value::Object(snapshot)
}

fn normalize_json_object(
    value: Value,
    field: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    if value.is_object() {
        Ok(value)
    } else {
        Err(bad_request(
            &format!("{field} must be an object"),
            request_id,
        ))
    }
}
