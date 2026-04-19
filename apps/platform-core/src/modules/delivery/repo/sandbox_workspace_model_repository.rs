use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

#[derive(Debug, Clone)]
pub(crate) struct SensitiveExecutionPolicyRecord {
    pub sensitive_execution_policy_id: String,
    pub policy_scope: String,
    pub execution_mode: String,
    pub policy_status: String,
    pub export_control_json: Value,
    pub output_boundary_json: Value,
    pub policy_snapshot_json: Value,
    pub step_up_required: bool,
    pub attestation_required: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AttestationReferenceRecord {
    pub attestation_record_id: String,
    pub attestation_type: String,
    pub status: String,
    pub attestation_uri: Option<String>,
    pub attestation_hash: Option<String>,
    pub verifier_ref: Option<String>,
    pub verified_at: Option<String>,
    pub metadata_json: Value,
}

pub(crate) async fn upsert_sensitive_execution_policy(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    sandbox_workspace_id: &str,
    query_surface_id: &str,
    output_boundary_json: &Value,
    export_control_json: &Value,
    step_up_required: bool,
    attestation_required: bool,
    policy_snapshot_json: &Value,
) -> Result<SensitiveExecutionPolicyRecord, (StatusCode, Json<ErrorResponse>)> {
    let existing_policy_id = client
        .query_opt(
            "SELECT sensitive_execution_policy_id::text
             FROM delivery.sensitive_execution_policy
             WHERE sandbox_workspace_id = $1::text::uuid
             ORDER BY updated_at DESC, sensitive_execution_policy_id DESC
             LIMIT 1",
            &[&sandbox_workspace_id],
        )
        .await
        .map_err(map_db_error)?
        .map(|row| row.get::<_, String>(0));

    let row = if let Some(policy_id) = existing_policy_id.as_ref() {
        client
            .query_one(
                "UPDATE delivery.sensitive_execution_policy
                 SET order_id = $2::text::uuid,
                     query_surface_id = $3::text::uuid,
                     policy_scope = 'sandbox_workspace',
                     execution_mode = 'sandbox_query',
                     output_boundary_json = $4::jsonb,
                     export_control_json = $5::jsonb,
                     step_up_required = $6,
                     attestation_required = $7,
                     approval_ticket_id = NULL,
                     policy_snapshot = $8::jsonb,
                     status = 'active',
                     updated_at = now()
                 WHERE sensitive_execution_policy_id = $1::text::uuid
                 RETURNING sensitive_execution_policy_id::text,
                           policy_scope,
                           execution_mode,
                           status,
                           export_control_json,
                           output_boundary_json,
                           policy_snapshot,
                           step_up_required,
                           attestation_required",
                &[
                    policy_id,
                    &order_id,
                    &query_surface_id,
                    output_boundary_json,
                    export_control_json,
                    &step_up_required,
                    &attestation_required,
                    policy_snapshot_json,
                ],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_one(
                "INSERT INTO delivery.sensitive_execution_policy (
                   order_id,
                   query_surface_id,
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
                   'sandbox_workspace',
                   'sandbox_query',
                   $4::jsonb,
                   $5::jsonb,
                   $6,
                   $7,
                   NULL,
                   $8::jsonb,
                   'active'
                 )
                 RETURNING sensitive_execution_policy_id::text,
                           policy_scope,
                           execution_mode,
                           status,
                           export_control_json,
                           output_boundary_json,
                           policy_snapshot,
                           step_up_required,
                           attestation_required",
                &[
                    &order_id,
                    &query_surface_id,
                    &sandbox_workspace_id,
                    output_boundary_json,
                    export_control_json,
                    &step_up_required,
                    &attestation_required,
                    policy_snapshot_json,
                ],
            )
            .await
            .map_err(map_db_error)?
    };

    Ok(SensitiveExecutionPolicyRecord {
        sensitive_execution_policy_id: row.get(0),
        policy_scope: row.get(1),
        execution_mode: row.get(2),
        policy_status: row.get(3),
        export_control_json: row.get(4),
        output_boundary_json: row.get(5),
        policy_snapshot_json: row.get(6),
        step_up_required: row.get(7),
        attestation_required: row.get(8),
    })
}

pub(crate) async fn upsert_attestation_reference(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    sandbox_session_id: &str,
    environment_id: &str,
    verifier_ref: Option<&str>,
    attestation_required: bool,
    metadata_json: &Value,
) -> Result<Option<AttestationReferenceRecord>, (StatusCode, Json<ErrorResponse>)> {
    if !attestation_required {
        return Ok(None);
    }

    let existing_attestation_id = client
        .query_opt(
            "SELECT attestation_record_id::text
             FROM delivery.attestation_record
             WHERE sandbox_session_id = $1::text::uuid
             ORDER BY updated_at DESC, attestation_record_id DESC
             LIMIT 1",
            &[&sandbox_session_id],
        )
        .await
        .map_err(map_db_error)?
        .map(|row| row.get::<_, String>(0));

    let row = if let Some(attestation_id) = existing_attestation_id.as_ref() {
        client
            .query_one(
                "UPDATE delivery.attestation_record
                 SET order_id = $2::text::uuid,
                     query_run_id = NULL,
                     sandbox_session_id = $3::text::uuid,
                     environment_id = $4::text::uuid,
                     attestation_type = 'execution_receipt',
                     attestation_uri = attestation_uri,
                     attestation_hash = attestation_hash,
                     verifier_ref = $5,
                     verified_at = NULL,
                     status = 'pending',
                     metadata = $6::jsonb,
                     updated_at = now()
                 WHERE attestation_record_id = $1::text::uuid
                 RETURNING attestation_record_id::text,
                           attestation_type,
                           status,
                           attestation_uri,
                           attestation_hash,
                           verifier_ref,
                           to_char(verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           metadata",
                &[
                    attestation_id,
                    &order_id,
                    &sandbox_session_id,
                    &environment_id,
                    &verifier_ref,
                    metadata_json,
                ],
            )
            .await
            .map_err(map_db_error)?
    } else {
        client
            .query_one(
                "INSERT INTO delivery.attestation_record (
                   order_id,
                   query_run_id,
                   sandbox_session_id,
                   environment_id,
                   attestation_type,
                   attestation_uri,
                   attestation_hash,
                   verifier_ref,
                   verified_at,
                   status,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   NULL,
                   $2::text::uuid,
                   $3::text::uuid,
                   'execution_receipt',
                   NULL,
                   NULL,
                   $4,
                   NULL,
                   'pending',
                   $5::jsonb
                 )
                 RETURNING attestation_record_id::text,
                           attestation_type,
                           status,
                           attestation_uri,
                           attestation_hash,
                           verifier_ref,
                           to_char(verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           metadata",
                &[
                    &order_id,
                    &sandbox_session_id,
                    &environment_id,
                    &verifier_ref,
                    metadata_json,
                ],
            )
            .await
            .map_err(map_db_error)?
    };

    Ok(Some(AttestationReferenceRecord {
        attestation_record_id: row.get(0),
        attestation_type: row.get(1),
        status: row.get(2),
        attestation_uri: row.get(3),
        attestation_hash: row.get(4),
        verifier_ref: row.get(5),
        verified_at: row.get(6),
        metadata_json: row.get(7),
    }))
}

pub(crate) fn build_export_control_json(
    export_policy_json: &Value,
    query_policy_json: &Value,
    output_boundary_json: &Value,
    seat_limit: i32,
    session_expire_at: &str,
) -> Value {
    let allowed_formats = export_policy_json
        .get("allowed_formats")
        .cloned()
        .unwrap_or_else(|| Value::Array(vec![]));
    let allow_export = export_policy_json
        .get("allow_export")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let max_exports = export_policy_json
        .get("max_exports")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let network_access = export_policy_json
        .get("network_access")
        .and_then(Value::as_str)
        .or_else(|| {
            query_policy_json
                .get("network_access")
                .and_then(Value::as_str)
        })
        .unwrap_or("deny");
    let allow_raw_export = output_boundary_json
        .get("allow_raw_export")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let copy_control = query_policy_json
        .get("copy_control")
        .cloned()
        .unwrap_or_else(|| Value::String("deny".to_string()));
    let concurrent_session_limit = query_policy_json
        .get("concurrent_session_limit")
        .and_then(Value::as_i64)
        .unwrap_or(1);

    json!({
        "allow_export": allow_export,
        "allow_raw_export": allow_raw_export,
        "allowed_formats": allowed_formats,
        "max_exports": max_exports,
        "network_access": network_access,
        "copy_control": copy_control,
        "seat_limit": seat_limit,
        "concurrent_session_limit": concurrent_session_limit,
        "session_expire_at": session_expire_at,
    })
}

pub(crate) fn build_sandbox_policy_snapshot(
    workspace_json: &Value,
    session_json: &Value,
    seat_json: &Value,
    environment_limits_json: &Value,
    export_control_json: &Value,
    attestation_json: Option<&Value>,
) -> Value {
    let mut object = Map::new();
    object.insert("workspace".to_string(), workspace_json.clone());
    object.insert("session".to_string(), session_json.clone());
    object.insert("seat".to_string(), seat_json.clone());
    object.insert(
        "environment_limits_json".to_string(),
        environment_limits_json.clone(),
    );
    object.insert("export_control".to_string(), export_control_json.clone());
    if let Some(attestation_json) = attestation_json {
        object.insert("attestation".to_string(), attestation_json.clone());
    }
    Value::Object(object)
}
