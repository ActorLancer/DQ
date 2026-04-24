use super::outbox_repository::{
    build_delivery_receipt_outbox_payload, write_billing_trigger_bridge_event,
    write_delivery_receipt_outbox_event,
};
use crate::modules::delivery::domain::is_accepted_state;
use crate::modules::delivery::dto::{
    ManageSandboxWorkspaceRequest, SandboxAttestationRefModel, SandboxExecutionEnvironmentModel,
    SandboxExportControlModel, SandboxRuntimeIsolationModel, SandboxSeatModel, SandboxSessionModel,
    SandboxWorkspaceModel, SandboxWorkspaceResponseData,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use crate::modules::delivery::repo::sandbox_workspace_model_repository::{
    build_export_control_json, build_sandbox_policy_snapshot, upsert_attestation_reference,
    upsert_sensitive_execution_policy,
};
use crate::modules::integration::application::{
    DeliveryCompletionNotificationDispatchInput, queue_delivery_completion_notifications,
};
use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::repo::{
    ensure_order_deliverable_and_prepare_delivery, map_db_error, write_trade_audit_event,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

const DELIVERY_SANDBOX_ENABLE_EVENT: &str = "delivery.sandbox.enable";
const DEFAULT_CLEAN_ROOM_MODE: &str = "lite";
const DEFAULT_DATA_RESIDENCY_MODE: &str = "seller_self_hosted";

pub async fn manage_sandbox_workspace(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageSandboxWorkspaceRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<SandboxWorkspaceResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_sandbox_context(&tx, order_id, request_id).await?;

    enforce_manage_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_sbx_std_state(&context, request_id)?;

    let prepared = ensure_order_deliverable_and_prepare_delivery(
        &tx, order_id, actor_role, request_id, trace_id,
    )
    .await?;

    let existing_workspace = load_existing_workspace(&tx, order_id, request_id).await?;
    let existing_session = if let Some(workspace) = existing_workspace.as_ref() {
        load_existing_session(&tx, &workspace.sandbox_workspace_id, request_id).await?
    } else {
        None
    };

    let query_surface_id = payload
        .query_surface_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            existing_workspace
                .as_ref()
                .map(|workspace| workspace.query_surface_id.clone())
        })
        .ok_or_else(|| bad_request("query_surface_id is required", request_id))?;
    let query_surface = load_query_surface(&tx, &query_surface_id, request_id).await?;
    enforce_query_surface_matches_order(&query_surface, &context, request_id)?;

    let environment_id = query_surface.environment_id.clone().ok_or_else(|| {
        conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: query surface is missing execution environment",
            request_id,
        )
    })?;
    let environment =
        load_execution_environment(&tx, &environment_id, &context.seller_org_id, request_id)
            .await?;

    let seat_user = resolve_seat_user_id(
        &tx,
        payload.seat_user_id.as_deref().or(existing_session
            .as_ref()
            .map(|session| session.user_id.as_str())),
        &context.buyer_org_id,
        request_id,
    )
    .await?;
    let workspace_name = normalize_workspace_name(
        payload.workspace_name.as_deref(),
        existing_workspace
            .as_ref()
            .map(|workspace| workspace.workspace_name.as_str()),
        order_id,
    );
    let expire_at = normalize_expire_at(
        &tx,
        payload.expire_at.as_deref().or(existing_session
            .as_ref()
            .map(|session| session.ended_at.as_str())),
        request_id,
    )
    .await?;
    let clean_room_mode = normalize_clean_room_mode(
        payload.clean_room_mode.as_deref().or(existing_workspace
            .as_ref()
            .map(|workspace| workspace.clean_room_mode.as_str())),
        request_id,
    )?;
    let data_residency_mode = normalize_data_residency_mode(
        payload.data_residency_mode.as_deref().or(existing_workspace
            .as_ref()
            .map(|workspace| workspace.data_residency_mode.as_str())),
    );
    let export_policy_json = resolve_export_policy(
        payload.export_policy_json.clone(),
        existing_workspace
            .as_ref()
            .map(|workspace| workspace.export_policy_json.clone()),
        &query_surface.output_boundary_json,
        request_id,
    )?;
    let output_boundary_json =
        derive_workspace_output_boundary(&query_surface.output_boundary_json, &export_policy_json);
    let execution_environment_model_json = build_execution_environment_model_json(&environment);
    let environment_limits_json = build_environment_limits_json(
        &query_surface,
        &environment,
        &execution_environment_model_json,
    );

    let operation = if existing_workspace.is_some() {
        "updated"
    } else {
        "created"
    };
    let workspace_row = if let Some(workspace) = existing_workspace.as_ref() {
        tx.query_one(
            "UPDATE delivery.sandbox_workspace
             SET environment_id = $2::text::uuid,
                 query_surface_id = $3::text::uuid,
                 workspace_name = $4,
                 status = 'active',
                 clean_room_mode = $5,
                 data_residency_mode = $6,
                 export_policy = $7::jsonb,
                 output_boundary_json = $8::jsonb,
                 updated_at = now()
             WHERE sandbox_workspace_id = $1::text::uuid
             RETURNING sandbox_workspace_id::text,
                       status,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &workspace.sandbox_workspace_id,
                &environment.environment_id,
                &query_surface_id,
                &workspace_name,
                &clean_room_mode,
                &data_residency_mode,
                &export_policy_json,
                &output_boundary_json,
            ],
        )
        .await
        .map_err(map_db_error)?
    } else {
        tx.query_one(
            "INSERT INTO delivery.sandbox_workspace (
               order_id,
               environment_id,
               query_surface_id,
               workspace_name,
               status,
               clean_room_mode,
               data_residency_mode,
               export_policy,
               output_boundary_json
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4,
               'active',
               $5,
               $6,
               $7::jsonb,
               $8::jsonb
             )
             RETURNING sandbox_workspace_id::text,
                       status,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &environment.environment_id,
                &query_surface_id,
                &workspace_name,
                &clean_room_mode,
                &data_residency_mode,
                &export_policy_json,
                &output_boundary_json,
            ],
        )
        .await
        .map_err(map_db_error)?
    };
    let sandbox_workspace_id: String = workspace_row.get(0);
    let workspace_status: String = workspace_row.get(1);
    let created_at: String = workspace_row.get(2);
    let updated_at: String = workspace_row.get(3);

    let session_row = if let Some(session) = existing_session.as_ref() {
        tx.query_one(
            "UPDATE delivery.sandbox_session
             SET user_id = $2::text::uuid,
                 started_at = COALESCE(started_at, now()),
                 ended_at = $3::timestamptz,
                 session_status = 'active'
             WHERE sandbox_session_id = $1::text::uuid
             RETURNING sandbox_session_id::text,
                       user_id::text,
                       session_status,
                       query_count,
                       export_attempt_count,
                       to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&session.sandbox_session_id, &seat_user.user_id, &expire_at],
        )
        .await
        .map_err(map_db_error)?
    } else {
        tx.query_one(
            "INSERT INTO delivery.sandbox_session (
               sandbox_workspace_id,
               user_id,
               started_at,
               ended_at,
               session_status,
               query_count,
               export_attempt_count
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               now(),
               $3::timestamptz,
               'active',
               0,
               0
             )
             RETURNING sandbox_session_id::text,
                       user_id::text,
                       session_status,
                       query_count,
                       export_attempt_count,
                       to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&sandbox_workspace_id, &seat_user.user_id, &expire_at],
        )
        .await
        .map_err(map_db_error)?
    };
    let sandbox_session_id: String = session_row.get(0);
    let session_user_id: String = session_row.get(1);
    let session_status: String = session_row.get(2);
    let session_query_count: i32 = session_row.get(3);
    let export_attempt_count: i32 = session_row.get(4);
    let session_started_at: String = session_row.get(5);
    let session_expire_at: String = session_row.get(6);

    let seat_limit = derive_seat_limit(&query_surface.query_policy_json);
    let step_up_required = derive_step_up_required(&query_surface.query_policy_json);
    let attestation_required =
        derive_attestation_required(&query_surface.query_policy_json, &environment.metadata);
    let export_control_json = build_export_control_json(
        &export_policy_json,
        &query_surface.query_policy_json,
        &output_boundary_json,
        seat_limit,
        &session_expire_at,
    );
    let workspace_model_json = json!({
        "sandbox_workspace_id": sandbox_workspace_id.clone(),
        "query_surface_id": query_surface_id.clone(),
        "environment_id": environment.environment_id.clone(),
        "workspace_name": workspace_name.clone(),
        "workspace_status": workspace_status.clone(),
        "clean_room_mode": clean_room_mode.clone(),
        "data_residency_mode": data_residency_mode.clone(),
        "created_at": created_at.clone(),
        "updated_at": updated_at.clone(),
    });
    let session_model_json = json!({
        "sandbox_session_id": sandbox_session_id.clone(),
        "session_status": session_status.clone(),
        "session_started_at": session_started_at.clone(),
        "expire_at": session_expire_at.clone(),
        "session_query_count": session_query_count,
        "export_attempt_count": export_attempt_count,
    });
    let seat_model_json = json!({
        "seat_user_id": seat_user.user_id.clone(),
        "login_id": seat_user.login_id.clone(),
        "display_name": seat_user.display_name.clone(),
        "email": seat_user.email.clone(),
        "seat_status": session_status.clone(),
        "seat_limit": seat_limit,
    });
    let attestation_metadata = json!({
        "order_id": order_id,
        "sandbox_workspace_id": sandbox_workspace_id.clone(),
        "sandbox_session_id": sandbox_session_id.clone(),
        "query_surface_id": query_surface_id.clone(),
        "environment_id": environment.environment_id.clone(),
        "seat_user_id": seat_user.user_id.clone(),
        "seat_login_id": seat_user.login_id.clone(),
    });
    let attestation_reference = upsert_attestation_reference(
        &tx,
        order_id,
        &sandbox_session_id,
        &environment.environment_id,
        environment
            .metadata
            .get("verifier_ref")
            .and_then(Value::as_str),
        attestation_required,
        &attestation_metadata,
    )
    .await?;
    let attestation_snapshot_json = attestation_reference.as_ref().map(|attestation| {
        json!({
            "attestation_record_id": attestation.attestation_record_id,
            "attestation_type": attestation.attestation_type,
            "status": attestation.status,
            "attestation_uri": attestation.attestation_uri,
            "attestation_hash": attestation.attestation_hash,
            "verifier_ref": attestation.verifier_ref,
            "verified_at": attestation.verified_at,
            "metadata_json": attestation.metadata_json,
        })
    });
    let policy_snapshot_json = build_sandbox_policy_snapshot(
        &workspace_model_json,
        &session_model_json,
        &seat_model_json,
        &execution_environment_model_json,
        &environment_limits_json,
        &export_control_json,
        attestation_snapshot_json.as_ref(),
    );
    let sensitive_execution_policy = upsert_sensitive_execution_policy(
        &tx,
        order_id,
        &sandbox_workspace_id,
        &query_surface_id,
        &output_boundary_json,
        &export_control_json,
        step_up_required,
        attestation_required,
        &policy_snapshot_json,
    )
    .await?;

    let target_state = "seat_issued";
    let layered_status = derive_sbx_std_layered_status(target_state, &context.payment_status);
    let reason_code = if operation == "created" {
        "delivery_sbx_std_seat_issued"
    } else {
        "delivery_sbx_std_workspace_updated"
    };
    let delivery_commit_hash = format!(
        "sandbox-workspace:{}:{}:{}",
        sandbox_workspace_id, sandbox_session_id, environment.environment_id
    );
    let receipt_hash = format!(
        "sandbox-seat:{}:{}:{}",
        order_id, sandbox_workspace_id, sandbox_session_id
    );
    let trust_boundary_patch = json!({
        "delivery_mode": "sandbox_workspace",
        "sandbox_workspace": {
            "sandbox_workspace_id": sandbox_workspace_id.clone(),
            "sandbox_session_id": sandbox_session_id.clone(),
            "query_surface_id": query_surface_id.clone(),
            "environment_id": environment.environment_id.clone(),
            "environment_type": environment.environment_type.clone(),
            "seat_user_id": session_user_id.clone(),
            "seat_login_id": seat_user.login_id.clone(),
            "seat_display_name": seat_user.display_name.clone(),
            "clean_room_mode": clean_room_mode.clone(),
            "data_residency_mode": data_residency_mode.clone(),
            "execution_environment": execution_environment_model_json.clone(),
            "export_policy": export_policy_json.clone(),
            "output_boundary_json": output_boundary_json.clone(),
            "environment_limits_json": environment_limits_json.clone(),
            "export_control_json": sensitive_execution_policy.export_control_json.clone(),
            "sensitive_execution_policy_id": sensitive_execution_policy.sensitive_execution_policy_id.clone(),
            "attestation": attestation_snapshot_json,
            "session_expire_at": session_expire_at.clone(),
        }
    });

    tx.execute(
        "UPDATE delivery.delivery_record
         SET delivery_type = 'sandbox_workspace',
             delivery_route = 'sandbox_query',
             status = 'committed',
             delivery_commit_hash = $2,
             trust_boundary_snapshot = trust_boundary_snapshot || $3::jsonb,
             receipt_hash = $4,
             committed_at = COALESCE(committed_at, now()),
             expires_at = $5::timestamptz,
             updated_at = now()
         WHERE delivery_id = $1::text::uuid",
        &[
            &prepared.delivery_id,
            &delivery_commit_hash,
            &trust_boundary_patch,
            &receipt_hash,
            &session_expire_at,
        ],
    )
    .await
    .map_err(map_db_error)?;

    tx.execute(
        "UPDATE trade.order_main
         SET status = $2,
             delivery_status = $3,
             acceptance_status = $4,
             settlement_status = $5,
             dispute_status = $6,
             last_reason_code = $7,
             updated_at = now()
         WHERE order_id = $1::text::uuid",
        &[
            &order_id,
            &target_state,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
            &reason_code,
        ],
    )
    .await
    .map_err(map_db_error)?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        "trade.order.sbx_std.transition",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    write_delivery_audit_event(
        &tx,
        "sandbox_workspace",
        &sandbox_workspace_id,
        actor_role,
        DELIVERY_SANDBOX_ENABLE_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "sandbox_session_id": sandbox_session_id.clone(),
            "query_surface_id": query_surface_id.clone(),
            "environment_id": environment.environment_id.clone(),
            "seat_user_id": session_user_id.clone(),
            "runtime_provider": execution_environment_model_json["runtime_isolation"]["runtime_provider"].clone(),
            "runtime_mode": execution_environment_model_json["runtime_isolation"]["runtime_mode"].clone(),
            "runtime_class": execution_environment_model_json["runtime_isolation"]["runtime_class"].clone(),
            "sensitive_execution_policy_id": sensitive_execution_policy
                .sensitive_execution_policy_id
                .clone(),
            "attestation_record_id": attestation_reference
                .as_ref()
                .map(|attestation| attestation.attestation_record_id.clone()),
            "operation": operation.to_string(),
            "delivery_id": prepared.delivery_id,
        }),
    )
    .await?;
    write_delivery_receipt_outbox_event(
        &tx,
        &prepared.delivery_id,
        &build_delivery_receipt_outbox_payload(
            "sandbox",
            order_id,
            &prepared.delivery_id,
            &context.sku_type,
            actor_role,
            &context.buyer_org_id,
            &context.seller_org_id,
            &target_state,
            &context.payment_status,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
            Some(receipt_hash.as_str()),
            Some(delivery_commit_hash.as_str()),
            Some("sandbox_workspace"),
            Some("sandbox_query"),
            None,
            json!({
                "sandbox_workspace_id": sandbox_workspace_id,
                "sandbox_session_id": sandbox_session_id,
                "query_surface_id": query_surface_id,
                "environment_id": environment.environment_id,
                "seat_user_id": session_user_id,
                "runtime_provider": execution_environment_model_json["runtime_isolation"]["runtime_provider"].clone(),
                "runtime_mode": execution_environment_model_json["runtime_isolation"]["runtime_mode"].clone(),
                "runtime_class": execution_environment_model_json["runtime_isolation"]["runtime_class"].clone(),
                "sensitive_execution_policy_id": sensitive_execution_policy.sensitive_execution_policy_id,
                "attestation_record_id": attestation_reference
                    .as_ref()
                    .map(|attestation| attestation.attestation_record_id.clone()),
                "session_expire_at": session_expire_at,
                "operation": operation,
            }),
        ),
        request_id,
        trace_id,
        idempotency_key,
    )
    .await?;
    let billing_bridge_idempotency_key =
        format!("billing-trigger:sandbox-enable:{}", prepared.delivery_id);
    write_billing_trigger_bridge_event(
        &tx,
        order_id,
        "delivery_committed",
        "delivery_record",
        &prepared.delivery_id,
        DELIVERY_SANDBOX_ENABLE_EVENT,
        actor_role,
        request_id,
        trace_id,
        billing_bridge_idempotency_key.as_str(),
        json!({
            "delivery_branch": "sandbox",
            "delivery_id": prepared.delivery_id,
            "sandbox_workspace_id": sandbox_workspace_id,
            "sandbox_session_id": sandbox_session_id,
            "query_surface_id": query_surface_id,
            "environment_id": environment.environment_id,
            "operation": operation,
        }),
    )
    .await?;
    let _ = queue_delivery_completion_notifications(
        &tx,
        DeliveryCompletionNotificationDispatchInput {
            order_id,
            delivery_branch: "sandbox",
            result_ref_type: "delivery_record",
            result_ref_id: &prepared.delivery_id,
            source_event_aggregate_type: "delivery.delivery_record",
            source_event_event_type: "delivery.committed",
            source_event_occurred_at: None,
            delivery_type: Some("sandbox_workspace"),
            delivery_route: Some("sandbox_query"),
            receipt_hash: Some(receipt_hash.as_str()),
            delivery_commit_hash: Some(delivery_commit_hash.as_str()),
            request_id,
            trace_id,
        },
    )
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(SandboxWorkspaceResponseData {
        sandbox_workspace_id,
        sandbox_session_id,
        sensitive_execution_policy_id: sensitive_execution_policy
            .sensitive_execution_policy_id
            .clone(),
        order_id: order_id.to_string(),
        query_surface_id,
        environment_id: environment.environment_id,
        environment_name: environment.environment_name,
        environment_type: environment.environment_type,
        network_zone: environment.network_zone,
        region_code: environment.region_code,
        sku_id: context.sku_id,
        sku_type: context.sku_type,
        workspace_name,
        workspace_status,
        session_status: session_status.clone(),
        seat_user_id: session_user_id,
        clean_room_mode,
        data_residency_mode,
        export_policy_json,
        output_boundary_json,
        environment_limits_json,
        session_started_at,
        expire_at: session_expire_at,
        session_query_count,
        export_attempt_count,
        operation: operation.to_string(),
        current_state: target_state.to_string(),
        payment_status: context.payment_status,
        delivery_status: layered_status.delivery_status,
        acceptance_status: layered_status.acceptance_status,
        settlement_status: layered_status.settlement_status,
        dispute_status: layered_status.dispute_status,
        created_at,
        updated_at,
        workspace: SandboxWorkspaceModel {
            sandbox_workspace_id: workspace_model_json["sandbox_workspace_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            query_surface_id: workspace_model_json["query_surface_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            environment_id: workspace_model_json["environment_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            workspace_name: workspace_model_json["workspace_name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            workspace_status: workspace_model_json["workspace_status"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            clean_room_mode: workspace_model_json["clean_room_mode"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            data_residency_mode: workspace_model_json["data_residency_mode"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            created_at: workspace_model_json["created_at"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            updated_at: workspace_model_json["updated_at"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        },
        session: SandboxSessionModel {
            sandbox_session_id: session_model_json["sandbox_session_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            session_status: session_model_json["session_status"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            session_started_at: session_model_json["session_started_at"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            expire_at: session_model_json["expire_at"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            session_query_count: session_query_count,
            export_attempt_count,
        },
        seat: SandboxSeatModel {
            seat_user_id: seat_user.user_id.clone(),
            login_id: seat_user.login_id.clone(),
            display_name: seat_user.display_name.clone(),
            email: seat_user.email.clone(),
            seat_status: session_status.clone(),
            seat_limit,
        },
        execution_environment: SandboxExecutionEnvironmentModel {
            environment_id: execution_environment_model_json["environment_id"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            environment_name: execution_environment_model_json["environment_name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            environment_type: execution_environment_model_json["environment_type"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            network_zone: execution_environment_model_json["network_zone"]
                .as_str()
                .map(str::to_string),
            region_code: execution_environment_model_json["region_code"]
                .as_str()
                .map(str::to_string),
            environment_status: execution_environment_model_json["environment_status"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            isolation_level: execution_environment_model_json["isolation_level"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            export_policy_json: execution_environment_model_json["export_policy_json"].clone(),
            audit_policy_json: execution_environment_model_json["audit_policy_json"].clone(),
            trusted_attestation_flag: execution_environment_model_json["trusted_attestation_flag"]
                .as_bool()
                .unwrap_or(false),
            supported_product_types: execution_environment_model_json["supported_product_types"]
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            current_capacity_json: execution_environment_model_json["current_capacity_json"]
                .clone(),
            runtime_isolation: SandboxRuntimeIsolationModel {
                runtime_provider:
                    execution_environment_model_json["runtime_isolation"]["runtime_provider"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                runtime_mode: execution_environment_model_json["runtime_isolation"]["runtime_mode"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                runtime_class:
                    execution_environment_model_json["runtime_isolation"]["runtime_class"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                profile_name: execution_environment_model_json["runtime_isolation"]["profile_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                rootfs_mode: execution_environment_model_json["runtime_isolation"]["rootfs_mode"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                network_mode: execution_environment_model_json["runtime_isolation"]["network_mode"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                seccomp_profile:
                    execution_environment_model_json["runtime_isolation"]["seccomp_profile"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                status: execution_environment_model_json["runtime_isolation"]["status"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            },
        },
        export_control: SandboxExportControlModel {
            sensitive_execution_policy_id: sensitive_execution_policy
                .sensitive_execution_policy_id
                .clone(),
            policy_scope: sensitive_execution_policy.policy_scope,
            execution_mode: sensitive_execution_policy.execution_mode,
            policy_status: sensitive_execution_policy.policy_status,
            export_control_json: sensitive_execution_policy.export_control_json,
            output_boundary_json: sensitive_execution_policy.output_boundary_json,
            policy_snapshot_json: sensitive_execution_policy.policy_snapshot_json,
            step_up_required: sensitive_execution_policy.step_up_required,
            attestation_required: sensitive_execution_policy.attestation_required,
        },
        attestation: attestation_reference.map(|attestation| SandboxAttestationRefModel {
            attestation_record_id: attestation.attestation_record_id,
            attestation_type: attestation.attestation_type,
            status: attestation.status,
            attestation_uri: attestation.attestation_uri,
            attestation_hash: attestation.attestation_hash,
            verifier_ref: attestation.verifier_ref,
            verified_at: attestation.verified_at,
            metadata_json: attestation.metadata_json,
        }),
    })
}

#[derive(Debug)]
struct SandboxWorkspaceContext {
    buyer_org_id: String,
    seller_org_id: String,
    asset_version_id: String,
    sku_id: String,
    sku_type: String,
    current_state: String,
    payment_status: String,
}

#[derive(Debug, Clone)]
struct ExistingWorkspace {
    sandbox_workspace_id: String,
    query_surface_id: String,
    workspace_name: String,
    clean_room_mode: String,
    data_residency_mode: String,
    export_policy_json: Value,
}

#[derive(Debug, Clone)]
struct ExistingSession {
    sandbox_session_id: String,
    user_id: String,
    ended_at: String,
}

#[derive(Debug, Clone)]
struct SeatUserContext {
    user_id: String,
    login_id: String,
    display_name: String,
    email: Option<String>,
}

#[derive(Debug)]
struct QuerySurfaceContext {
    asset_version_id: String,
    environment_id: Option<String>,
    surface_type: String,
    execution_scope: String,
    output_boundary_json: Value,
    query_policy_json: Value,
    status: String,
}

#[derive(Debug)]
struct ExecutionEnvironmentContext {
    environment_id: String,
    environment_name: String,
    environment_type: String,
    status: String,
    network_zone: Option<String>,
    region_code: Option<String>,
    metadata: Value,
}

async fn load_sandbox_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<SandboxWorkspaceContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.buyer_org_id::text,
                    o.seller_org_id::text,
                    o.asset_version_id::text,
                    o.status,
                    o.payment_status,
                    s.sku_id::text,
                    s.sku_type
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

    Ok(SandboxWorkspaceContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        asset_version_id: row.get(2),
        current_state: row.get(3),
        payment_status: row.get(4),
        sku_id: row.get(5),
        sku_type: row.get(6),
    })
}

async fn load_existing_workspace(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<Option<ExistingWorkspace>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT sandbox_workspace_id::text,
                    query_surface_id::text,
                    workspace_name,
                    clean_room_mode,
                    data_residency_mode,
                    export_policy
             FROM delivery.sandbox_workspace
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, sandbox_workspace_id DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| ExistingWorkspace {
        sandbox_workspace_id: row.get(0),
        query_surface_id: row.get(1),
        workspace_name: row.get(2),
        clean_room_mode: row.get(3),
        data_residency_mode: row.get(4),
        export_policy_json: row.get(5),
    }))
}

async fn load_existing_session(
    client: &(impl GenericClient + Sync),
    sandbox_workspace_id: &str,
    request_id: Option<&str>,
) -> Result<Option<ExistingSession>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT sandbox_session_id::text,
                    user_id::text,
                    to_char(ended_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.sandbox_session
             WHERE sandbox_workspace_id = $1::text::uuid
             ORDER BY started_at DESC, sandbox_session_id DESC
             LIMIT 1",
            &[&sandbox_workspace_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| ExistingSession {
        sandbox_session_id: row.get(0),
        user_id: row.get(1),
        ended_at: row.get::<_, Option<String>>(2).unwrap_or_default(),
    }))
}

async fn load_query_surface(
    client: &(impl GenericClient + Sync),
    query_surface_id: &str,
    request_id: Option<&str>,
) -> Result<QuerySurfaceContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT asset_version_id::text,
                    environment_id::text,
                    surface_type,
                    execution_scope,
                    output_boundary_json,
                    query_policy_json,
                    status
             FROM catalog.query_surface_definition
             WHERE query_surface_id = $1::text::uuid",
            &[&query_surface_id],
        )
        .await
        .map_err(|_| bad_request("query_surface_id must be a valid uuid", request_id))?;

    let Some(row) = row else {
        return Err(not_found(
            &format!("query surface not found: {query_surface_id}"),
            request_id,
        ));
    };

    Ok(QuerySurfaceContext {
        asset_version_id: row.get(0),
        environment_id: row.get(1),
        surface_type: row.get(2),
        execution_scope: row.get(3),
        output_boundary_json: row.get(4),
        query_policy_json: row.get(5),
        status: row.get(6),
    })
}

async fn load_execution_environment(
    client: &(impl GenericClient + Sync),
    environment_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<ExecutionEnvironmentContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT environment_id::text,
                    org_id::text,
                    environment_name,
                    environment_type,
                    status,
                    network_zone,
                    region_code,
                    metadata
             FROM core.execution_environment
             WHERE environment_id = $1::text::uuid",
            &[&environment_id],
        )
        .await
        .map_err(|_| bad_request("environment_id must be a valid uuid", request_id))?;

    let Some(row) = row else {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: execution environment not found",
            request_id,
        ));
    };

    let environment_org_id: Option<String> = row.get(1);
    let environment_name: String = row.get(2);
    let environment_type: String = row.get(3);
    let status: String = row.get(4);
    if status != "active" {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: execution environment is not active",
            request_id,
        ));
    }
    if environment_type != "sandbox" {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: execution environment is not sandbox type",
            request_id,
        ));
    }
    if environment_org_id
        .as_deref()
        .is_some_and(|value| value != seller_org_id)
    {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: execution environment is outside current seller scope",
            request_id,
        ));
    }

    Ok(ExecutionEnvironmentContext {
        environment_id: row.get(0),
        environment_name,
        environment_type,
        status,
        network_zone: row.get(5),
        region_code: row.get(6),
        metadata: row.get(7),
    })
}

async fn resolve_seat_user_id(
    client: &(impl GenericClient + Sync),
    seat_user_id: Option<&str>,
    buyer_org_id: &str,
    request_id: Option<&str>,
) -> Result<SeatUserContext, (StatusCode, Json<ErrorResponse>)> {
    let seat_user_id = seat_user_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_request("seat_user_id is required", request_id))?;

    let row = client
        .query_opt(
            "SELECT user_id::text,
                    org_id::text,
                    status,
                    login_id,
                    display_name,
                    email::text
             FROM core.user_account
             WHERE user_id = $1::text::uuid",
            &[&seat_user_id],
        )
        .await
        .map_err(|_| bad_request("seat_user_id must be a valid uuid", request_id))?;

    let Some(row) = row else {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: seat_user_id does not exist",
            request_id,
        ));
    };

    let resolved_user_id: String = row.get(0);
    let org_id: String = row.get(1);
    let status: String = row.get(2);
    if org_id != buyer_org_id {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: seat_user_id is outside current buyer tenant",
            request_id,
        ));
    }
    if status != "active" {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: seat_user_id is not active",
            request_id,
        ));
    }
    Ok(SeatUserContext {
        user_id: resolved_user_id,
        login_id: row.get(3),
        display_name: row.get(4),
        email: row.get(5),
    })
}

async fn normalize_expire_at(
    client: &(impl GenericClient + Sync),
    expire_at: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let expire_at = expire_at
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| bad_request("expire_at is required", request_id))?;

    let row = client
        .query_one(
            "SELECT to_char($1::timestamptz AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    ($1::timestamptz > now())",
            &[&expire_at],
        )
        .await
        .map_err(|_| bad_request("expire_at must be a valid RFC3339 timestamp", request_id))?;
    let normalized: String = row.get(0);
    let is_future: bool = row.get(1);
    if !is_future {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: expire_at must be in the future",
            request_id,
        ));
    }
    Ok(normalized)
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
        "sandbox workspace enable is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_sbx_std_state(
    context: &SandboxWorkspaceContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type != "SBX_STD" {
        return Err(conflict(
            &format!(
                "SANDBOX_WORKSPACE_FORBIDDEN: order sku_type `{}` is not SBX_STD",
                context.sku_type
            ),
            request_id,
        ));
    }
    if !matches!(
        context.current_state.as_str(),
        "buyer_locked" | "workspace_enabled" | "seat_issued"
    ) {
        return Err(conflict(
            &format!(
                "SANDBOX_WORKSPACE_FORBIDDEN: current_state `{}` does not allow sandbox provisioning",
                context.current_state
            ),
            request_id,
        ));
    }
    Ok(())
}

fn enforce_query_surface_matches_order(
    query_surface: &QuerySurfaceContext,
    context: &SandboxWorkspaceContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if query_surface.status != "active" {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: query surface is not active",
            request_id,
        ));
    }
    if query_surface.surface_type != "sandbox_query" {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: query surface is not sandbox_query",
            request_id,
        ));
    }
    if query_surface.asset_version_id != context.asset_version_id {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: query surface does not belong to current order asset version",
            request_id,
        ));
    }
    Ok(())
}

fn normalize_workspace_name(
    workspace_name: Option<&str>,
    existing_workspace_name: Option<&str>,
    order_id: &str,
) -> String {
    workspace_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| existing_workspace_name.map(str::to_string))
        .unwrap_or_else(|| {
            let suffix = order_id.get(0..8).unwrap_or(order_id);
            format!("sandbox-{suffix}")
        })
}

fn normalize_clean_room_mode(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_CLEAN_ROOM_MODE)
        .to_ascii_lowercase();
    if normalized != DEFAULT_CLEAN_ROOM_MODE {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: clean_room_mode must be `lite` in V1",
            request_id,
        ));
    }
    Ok(normalized)
}

fn normalize_data_residency_mode(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_DATA_RESIDENCY_MODE)
        .to_string()
}

fn resolve_export_policy(
    candidate: Option<Value>,
    existing: Option<Value>,
    surface_output_boundary: &Value,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut policy = normalize_json_object(
        candidate.or(existing).or_else(|| {
            Some(json!({
                "allow_export": false,
                "allowed_formats": [],
                "max_exports": 0,
                "network_access": "deny"
            }))
        }),
        "export_policy_json",
        request_id,
    )?;

    let surface_allowed_formats = string_set(surface_output_boundary, "allowed_formats");
    let policy_allowed_formats = string_set(&policy, "allowed_formats");
    if !surface_allowed_formats.is_empty()
        && !policy_allowed_formats.is_empty()
        && !policy_allowed_formats.is_subset(&surface_allowed_formats)
    {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: export allowed_formats exceed query surface boundary",
            request_id,
        ));
    }

    let surface_allow_export = surface_output_boundary
        .get("allow_export")
        .or_else(|| surface_output_boundary.get("allow_raw_export"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let allow_export = policy
        .get("allow_export")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if allow_export && !surface_allow_export {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: export policy exceeds query surface boundary",
            request_id,
        ));
    }

    let surface_max_exports = surface_output_boundary
        .get("max_exports")
        .and_then(Value::as_i64);
    let policy_max_exports = policy
        .get("max_exports")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if surface_max_exports.is_some_and(|limit| policy_max_exports > limit) {
        return Err(conflict(
            "SANDBOX_WORKSPACE_FORBIDDEN: max_exports exceed query surface boundary",
            request_id,
        ));
    }

    if let Some(object) = policy.as_object_mut() {
        if !surface_allowed_formats.is_empty() && policy_allowed_formats.is_empty() {
            object.insert(
                "allowed_formats".to_string(),
                Value::Array(
                    surface_allowed_formats
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        object.insert("allow_export".to_string(), Value::Bool(allow_export));
        if !object.contains_key("max_exports") {
            object.insert(
                "max_exports".to_string(),
                Value::from(surface_max_exports.unwrap_or(0)),
            );
        }
    }

    Ok(policy)
}

fn derive_workspace_output_boundary(
    surface_output_boundary: &Value,
    export_policy: &Value,
) -> Value {
    let mut boundary = surface_output_boundary
        .as_object()
        .cloned()
        .unwrap_or_else(Map::new);
    if let Some(allowed_formats) = export_policy.get("allowed_formats") {
        boundary.insert("allowed_formats".to_string(), allowed_formats.clone());
    }
    if let Some(max_exports) = export_policy.get("max_exports") {
        boundary.insert("max_exports".to_string(), max_exports.clone());
    }
    if let Some(allow_export) = export_policy.get("allow_export") {
        boundary.insert("allow_export".to_string(), allow_export.clone());
    }
    Value::Object(boundary)
}

fn build_environment_limits_json(
    query_surface: &QuerySurfaceContext,
    environment: &ExecutionEnvironmentContext,
    execution_environment_model_json: &Value,
) -> Value {
    json!({
        "execution_scope": query_surface.execution_scope,
        "query_policy_json": query_surface.query_policy_json,
        "environment": {
            "environment_id": environment.environment_id,
            "environment_name": environment.environment_name,
            "environment_type": environment.environment_type,
            "status": environment.status,
            "network_zone": environment.network_zone,
            "region_code": environment.region_code,
            "metadata": environment.metadata,
        },
        "execution_environment": execution_environment_model_json,
    })
}

fn build_execution_environment_model_json(environment: &ExecutionEnvironmentContext) -> Value {
    let metadata = environment.metadata.as_object();
    let runtime_isolation = metadata
        .and_then(|map| map.get("runtime_isolation"))
        .cloned()
        .or_else(|| metadata.and_then(|map| map.get("gvisor")).cloned())
        .unwrap_or_else(|| json!({}));

    let trusted_attestation_flag = metadata
        .and_then(|map| map.get("trusted_attestation_flag"))
        .and_then(Value::as_bool)
        .or_else(|| {
            metadata
                .and_then(|map| map.get("attestation_required"))
                .and_then(Value::as_bool)
        })
        .or_else(|| {
            metadata
                .and_then(|map| map.get("verifier_ref"))
                .and_then(Value::as_str)
                .map(|_| true)
        })
        .unwrap_or(false);

    let supported_product_types = metadata
        .and_then(|map| map.get("supported_product_types"))
        .and_then(Value::as_array)
        .map(|values| {
            Value::Array(
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|value| Value::String(value.to_string()))
                    .collect(),
            )
        })
        .unwrap_or_else(|| json!(["SBX_STD"]));

    json!({
        "environment_id": environment.environment_id,
        "environment_name": environment.environment_name,
        "environment_type": environment.environment_type,
        "network_zone": environment.network_zone,
        "region_code": environment.region_code,
        "environment_status": environment.status,
        "isolation_level": metadata
            .and_then(|map| map.get("isolation_level"))
            .and_then(Value::as_str)
            .unwrap_or("container_sandbox"),
        "export_policy_json": metadata
            .and_then(|map| map.get("export_policy"))
            .cloned()
            .unwrap_or_else(|| json!({
                "allow_export": false,
                "network_access": environment.network_zone.clone().unwrap_or_else(|| "seller_vpc".to_string()),
                "policy_source": "environment_placeholder"
            })),
        "audit_policy_json": metadata
            .and_then(|map| map.get("audit_policy"))
            .cloned()
            .unwrap_or_else(|| json!({
                "required_events": ["query_log", "session_log", "policy_hit", "export_attempt"],
                "policy_source": "environment_placeholder"
            })),
        "trusted_attestation_flag": trusted_attestation_flag,
        "supported_product_types": supported_product_types,
        "current_capacity_json": metadata
            .and_then(|map| map.get("current_capacity"))
            .cloned()
            .unwrap_or_else(|| json!({})),
        "runtime_isolation": {
            "runtime_provider": runtime_isolation
                .get("runtime_provider")
                .and_then(Value::as_str)
                .or_else(|| runtime_isolation.get("preferred_runtime").and_then(Value::as_str))
                .unwrap_or("gvisor"),
            "runtime_mode": runtime_isolation
                .get("runtime_mode")
                .and_then(Value::as_str)
                .unwrap_or("local_placeholder"),
            "runtime_class": runtime_isolation
                .get("runtime_class")
                .and_then(Value::as_str)
                .unwrap_or("runsc"),
            "profile_name": runtime_isolation
                .get("profile_name")
                .and_then(Value::as_str)
                .unwrap_or("sbx-std-default"),
            "rootfs_mode": runtime_isolation
                .get("rootfs_mode")
                .and_then(Value::as_str)
                .unwrap_or("read_only"),
            "network_mode": runtime_isolation
                .get("network_mode")
                .and_then(Value::as_str)
                .or(environment.network_zone.as_deref())
                .unwrap_or("seller_vpc"),
            "seccomp_profile": runtime_isolation
                .get("seccomp_profile")
                .and_then(Value::as_str)
                .unwrap_or("platform/default"),
            "status": runtime_isolation
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("reserved"),
        }
    })
}

fn derive_seat_limit(query_policy_json: &Value) -> i32 {
    query_policy_json
        .get("seat_limit")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .clamp(1, i64::from(i32::MAX)) as i32
}

fn derive_step_up_required(query_policy_json: &Value) -> bool {
    query_policy_json
        .get("step_up_required")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn derive_attestation_required(query_policy_json: &Value, environment_metadata: &Value) -> bool {
    query_policy_json
        .get("attestation_required")
        .and_then(Value::as_bool)
        .or_else(|| {
            query_policy_json
                .get("requires_attestation")
                .and_then(Value::as_bool)
        })
        .or_else(|| {
            environment_metadata
                .get("attestation_required")
                .and_then(Value::as_bool)
        })
        .unwrap_or(false)
}

fn derive_sbx_std_layered_status(target_state: &str, payment_status: &str) -> LayeredOrderStatus {
    LayeredOrderStatus {
        delivery_status: if is_accepted_state("SBX_STD", target_state) {
            "delivered".to_string()
        } else {
            "in_progress".to_string()
        },
        acceptance_status: if is_accepted_state("SBX_STD", target_state) {
            "accepted".to_string()
        } else {
            "not_started".to_string()
        },
        settlement_status: if payment_status == "paid" {
            "pending_settlement".to_string()
        } else {
            "not_started".to_string()
        },
        dispute_status: "none".to_string(),
    }
}

fn normalize_json_object(
    value: Option<Value>,
    field: &str,
    request_id: Option<&str>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let value = value.unwrap_or_else(|| Value::Object(Map::new()));
    if value.is_object() {
        Ok(value)
    } else {
        Err(bad_request(
            &format!("{field} must be a JSON object"),
            request_id,
        ))
    }
}

fn string_set(value: &Value, key: &str) -> BTreeSet<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

fn forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: "SANDBOX_WORKSPACE_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
