use crate::modules::delivery::dto::{
    DestructionAttestationResponseData, ManageDestructionAttestationRequest,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use crate::modules::storage::domain::resolve_storage_object_location;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

const DELIVERY_DESTRUCTION_ATTEST_EVENT: &str = "delivery.destruction.attest";

pub async fn manage_destruction_attestation(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageDestructionAttestationRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<DestructionAttestationResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_order_context(&tx, order_id, request_id).await?;
    enforce_manage_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;

    let existing = load_existing_attestation(
        &tx,
        order_id,
        payload.destruction_attestation_id.as_deref(),
        request_id,
    )
    .await?;
    let object_id = payload
        .object_id
        .as_deref()
        .or(existing
            .as_ref()
            .and_then(|value| value.object_id.as_deref()))
        .ok_or_else(|| {
            conflict(
                "DESTRUCTION_ATTESTATION_FORBIDDEN: object_id is required",
                request_id,
            )
        })?;
    let object = load_object_context(&tx, order_id, object_id, request_id).await?;

    let retention_action = normalize_retention_action(
        payload.retention_action.as_deref().or(existing
            .as_ref()
            .map(|value| value.retention_action.as_str())),
        request_id,
    )?;
    let legal_hold_active = payload
        .metadata
        .as_ref()
        .and_then(|value| value.get("legal_hold_status"))
        .and_then(Value::as_str)
        == Some("active");
    enforce_triggered(&context, &object, legal_hold_active, request_id)?;

    let status = normalize_status(
        payload.status.as_deref(),
        Some(retention_action.as_str()),
        request_id,
    )?;
    validate_status_action_alignment(&retention_action, &status, request_id)?;

    let approval_ticket_id = payload.approval_ticket_id.as_deref().or(existing
        .as_ref()
        .and_then(|value| value.approval_ticket_id.as_deref()));
    if retention_action == "legal_hold" || legal_hold_active {
        validate_approval_ticket(&tx, approval_ticket_id, true, request_id).await?;
    } else {
        validate_approval_ticket(&tx, approval_ticket_id, false, request_id).await?;
    }

    let attestation_uri = normalize_optional_text(
        payload.attestation_uri.as_deref().or(existing
            .as_ref()
            .and_then(|value| value.attestation_uri.as_deref())),
    );
    let attestation_hash = normalize_optional_text(
        payload.attestation_hash.as_deref().or(existing
            .as_ref()
            .and_then(|value| value.attestation_hash.as_deref())),
    );
    if status != "pending" && attestation_uri.is_none() && attestation_hash.is_none() {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: completed or retained proofs require attestation_uri or attestation_hash",
            request_id,
        ));
    }

    let executed_by_type =
        normalize_executed_by_type(payload.executed_by_type.as_deref(), actor_role, request_id)?;
    let executed_by_id = normalize_optional_text(
        payload.executed_by_id.as_deref().or(existing
            .as_ref()
            .and_then(|value| value.executed_by_id.as_deref())),
    );
    let ref_type = normalize_ref_type(
        payload.ref_type.as_deref(),
        Some(object.link_type.as_str()),
        request_id,
    )?;
    let metadata = build_metadata(
        payload.metadata.as_ref(),
        actor_role,
        &context,
        &object,
        &retention_action,
        &status,
    )?;

    let (row, operation) = if let Some(existing_id) = existing
        .as_ref()
        .map(|value| value.destruction_attestation_id.as_str())
    {
        (
            tx.query_one(
                "UPDATE delivery.destruction_attestation
                 SET object_id = $2::text::uuid,
                     ref_type = $3,
                     retention_action = $4,
                     attestation_uri = $5,
                     attestation_hash = $6,
                     executed_by_type = $7,
                     executed_by_id = $8::text::uuid,
                     approval_ticket_id = $9::text::uuid,
                     executed_at = COALESCE($10::timestamptz, CASE WHEN $11 = 'pending' THEN NULL ELSE now() END),
                     status = $11,
                     metadata = $12::jsonb,
                     updated_at = now()
                 WHERE destruction_attestation_id = $1::text::uuid
                 RETURNING destruction_attestation_id::text,
                           order_id::text,
                           object_id::text,
                           ref_type,
                           retention_action,
                           attestation_uri,
                           attestation_hash,
                           executed_by_type,
                           executed_by_id::text,
                           approval_ticket_id::text,
                           to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           status,
                           metadata,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &existing_id,
                    &object_id,
                    &ref_type,
                    &retention_action,
                    &attestation_uri,
                    &attestation_hash,
                    &executed_by_type,
                    &executed_by_id,
                    &approval_ticket_id,
                    &payload.executed_at,
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
                "INSERT INTO delivery.destruction_attestation (
                   order_id,
                   object_id,
                   ref_type,
                   retention_action,
                   attestation_uri,
                   attestation_hash,
                   executed_by_type,
                   executed_by_id,
                   approval_ticket_id,
                   executed_at,
                   status,
                   metadata
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3,
                   $4,
                   $5,
                   $6,
                   $7,
                   $8::text::uuid,
                   $9::text::uuid,
                   COALESCE($10::timestamptz, CASE WHEN $11 = 'pending' THEN NULL ELSE now() END),
                   $11,
                   $12::jsonb
                 )
                 RETURNING destruction_attestation_id::text,
                           order_id::text,
                           object_id::text,
                           ref_type,
                           retention_action,
                           attestation_uri,
                           attestation_hash,
                           executed_by_type,
                           executed_by_id::text,
                           approval_ticket_id::text,
                           to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           status,
                           metadata,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &order_id,
                    &object_id,
                    &ref_type,
                    &retention_action,
                    &attestation_uri,
                    &attestation_hash,
                    &executed_by_type,
                    &executed_by_id,
                    &approval_ticket_id,
                    &payload.executed_at,
                    &status,
                    &metadata,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "created",
        )
    };

    write_delivery_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        DELIVERY_DESTRUCTION_ATTEST_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "destruction_attestation_id": row.get::<_, String>(0),
            "object_id": row.get::<_, Option<String>>(2),
            "ref_type": row.get::<_, String>(3),
            "retention_action": row.get::<_, String>(4),
            "status": row.get::<_, String>(11),
            "operation": operation,
            "current_state": context.current_state,
            "payment_status": context.payment_status,
            "delivery_status": context.delivery_status,
            "object_link_type": object.link_type,
            "object_link_status": object.link_status,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    let resolved_object = object
        .object_uri
        .as_deref()
        .map(|uri| resolve_storage_object_location(uri, None))
        .unwrap_or_default();

    Ok(DestructionAttestationResponseData {
        destruction_attestation_id: row.get(0),
        order_id: row.get(1),
        object_id: row.get(2),
        ref_type: row.get(3),
        retention_action: row.get(4),
        attestation_uri: row.get(5),
        attestation_hash: row.get(6),
        executed_by_type: row.get(7),
        executed_by_id: row.get(8),
        approval_ticket_id: row.get(9),
        executed_at: row.get(10),
        status: row.get(11),
        metadata: row.get(12),
        object_bucket_name: resolved_object.bucket_name,
        object_key: resolved_object.object_key,
        object_link_type: object.link_type,
        object_link_status: object.link_status,
        operation: operation.to_string(),
        current_state: context.current_state,
        payment_status: context.payment_status,
        delivery_status: context.delivery_status,
        created_at: row.get(13),
        updated_at: row.get(14),
    })
}

#[derive(Debug)]
struct OrderContext {
    buyer_org_id: String,
    seller_org_id: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
    dispute_status: String,
}

#[derive(Debug)]
struct ExistingDestructionAttestation {
    destruction_attestation_id: String,
    object_id: Option<String>,
    retention_action: String,
    attestation_uri: Option<String>,
    attestation_hash: Option<String>,
    executed_by_id: Option<String>,
    approval_ticket_id: Option<String>,
}

#[derive(Debug)]
struct ObjectContext {
    object_uri: Option<String>,
    link_type: String,
    link_status: Option<String>,
}

fn validate_request(
    payload: &ManageDestructionAttestationRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(metadata) = payload.metadata.as_ref()
        && !metadata.is_object()
    {
        return Err(bad_request("metadata must be a JSON object", request_id));
    }
    Ok(())
}

async fn load_order_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT buyer_org_id::text,
                    seller_org_id::text,
                    status,
                    payment_status,
                    delivery_status,
                    dispute_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };
    Ok(OrderContext {
        buyer_org_id: row.get(0),
        seller_org_id: row.get(1),
        current_state: row.get(2),
        payment_status: row.get(3),
        delivery_status: row.get(4),
        dispute_status: row.get(5),
    })
}

async fn load_existing_attestation(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    destruction_attestation_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<ExistingDestructionAttestation>, (StatusCode, Json<ErrorResponse>)> {
    let Some(destruction_attestation_id) = destruction_attestation_id else {
        return Ok(None);
    };
    let row = client
        .query_opt(
            "SELECT destruction_attestation_id::text,
                    object_id::text,
                    retention_action,
                    attestation_uri,
                    attestation_hash,
                    executed_by_id::text,
                    approval_ticket_id::text
             FROM delivery.destruction_attestation
             WHERE destruction_attestation_id = $1::text::uuid
               AND order_id = $2::text::uuid",
            &[&destruction_attestation_id, &order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: destruction_attestation_id does not belong to current order",
            request_id,
        ));
    };
    Ok(Some(ExistingDestructionAttestation {
        destruction_attestation_id: row.get(0),
        object_id: row.get(1),
        retention_action: row.get(2),
        attestation_uri: row.get(3),
        attestation_hash: row.get(4),
        executed_by_id: row.get(5),
        approval_ticket_id: row.get(6),
    }))
}

async fn load_object_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    object_id: &str,
    request_id: Option<&str>,
) -> Result<ObjectContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT so.object_uri,
                    dr.status,
                    qr.status
             FROM delivery.storage_object so
             LEFT JOIN LATERAL (
               SELECT status
               FROM delivery.delivery_record
               WHERE order_id = $1::text::uuid
                 AND object_id = so.object_id
               ORDER BY updated_at DESC, delivery_id DESC
               LIMIT 1
             ) dr ON true
             LEFT JOIN LATERAL (
               SELECT status
               FROM delivery.query_execution_run
               WHERE order_id = $1::text::uuid
                 AND result_object_id = so.object_id
               ORDER BY updated_at DESC, query_run_id DESC
               LIMIT 1
             ) qr ON true
             WHERE so.object_id = $2::text::uuid",
            &[&order_id, &object_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: object_id not found",
            request_id,
        ));
    };

    let delivery_status: Option<String> = row.get(1);
    let query_run_status: Option<String> = row.get(2);
    let Some(link_status) = delivery_status.clone().or(query_run_status.clone()) else {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: object_id is not linked to current order",
            request_id,
        ));
    };

    Ok(ObjectContext {
        object_uri: row.get(0),
        link_type: if delivery_status.is_some() {
            "delivery_object".to_string()
        } else {
            "query_result".to_string()
        },
        link_status: Some(link_status),
    })
}

fn enforce_manage_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(
        actor_role,
        "platform_admin"
            | "platform_audit_security"
            | "platform_risk_settlement"
            | "audit_admin"
            | "retention_admin"
    ) {
        return Ok(());
    }

    let Some(tenant_id) = tenant_id else {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: tenant scope is required",
            request_id,
        ));
    };

    let allowed = match actor_role {
        "seller_operator" | "seller_storage_operator" => tenant_id == seller_org_id,
        "buyer_operator" | "procurement_manager" => tenant_id == buyer_org_id,
        "tenant_admin" => tenant_id == buyer_org_id || tenant_id == seller_org_id,
        _ => false,
    };

    if allowed {
        return Ok(());
    }

    Err(conflict(
        "DESTRUCTION_ATTESTATION_FORBIDDEN: tenant scope does not match order participants",
        request_id,
    ))
}

fn enforce_triggered(
    context: &OrderContext,
    object: &ObjectContext,
    legal_hold_active: bool,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let order_triggered = matches!(
        context.current_state.as_str(),
        "expired"
            | "closed"
            | "payment_failed_pending_resolution"
            | "payment_timeout_pending_compensation_cancel"
    ) || matches!(
        context.delivery_status.as_str(),
        "expired" | "revoked" | "suspended"
    ) || context.dispute_status != "none";
    let object_triggered = matches!(
        object.link_status.as_deref(),
        Some("expired" | "revoked" | "suspended")
    );

    if order_triggered || object_triggered || legal_hold_active {
        return Ok(());
    }

    Err(conflict(
        "DESTRUCTION_ATTESTATION_FORBIDDEN: order has not reached expiry, revocation, dispute, or legal-hold stage",
        request_id,
    ))
}

fn normalize_retention_action(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let action = value.unwrap_or("destroy").trim().to_ascii_lowercase();
    if matches!(action.as_str(), "destroy" | "retain" | "legal_hold") {
        return Ok(action);
    }
    Err(bad_request(
        "retention_action must be one of: destroy, retain, legal_hold",
        request_id,
    ))
}

fn normalize_status(
    value: Option<&str>,
    retention_action: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let status = value
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_else(|| match retention_action.unwrap_or("destroy") {
            "retain" | "legal_hold" => "retained".to_string(),
            _ => "completed".to_string(),
        });
    if matches!(
        status.as_str(),
        "pending" | "completed" | "retained" | "failed"
    ) {
        return Ok(status);
    }
    Err(bad_request(
        "status must be one of: pending, completed, retained, failed",
        request_id,
    ))
}

fn validate_status_action_alignment(
    retention_action: &str,
    status: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if retention_action == "destroy" && status == "retained" {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: destroy action cannot use retained status",
            request_id,
        ));
    }
    if matches!(retention_action, "retain" | "legal_hold") && status == "completed" {
        return Err(conflict(
            "DESTRUCTION_ATTESTATION_FORBIDDEN: retain or legal_hold action must use pending, retained, or failed status",
            request_id,
        ));
    }
    Ok(())
}

async fn validate_approval_ticket(
    client: &(impl GenericClient + Sync),
    approval_ticket_id: Option<&str>,
    required: bool,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(approval_ticket_id) = approval_ticket_id else {
        if required {
            return Err(conflict(
                "DESTRUCTION_ATTESTATION_FORBIDDEN: approval_ticket_id is required for legal hold or explicit retention proof",
                request_id,
            ));
        }
        return Ok(());
    };

    let exists = client
        .query_opt(
            "SELECT 1
             FROM ops.approval_ticket
             WHERE approval_ticket_id = $1::text::uuid",
            &[&approval_ticket_id],
        )
        .await
        .map_err(map_db_error)?
        .is_some();
    if exists {
        return Ok(());
    }
    Err(conflict(
        "DESTRUCTION_ATTESTATION_FORBIDDEN: approval_ticket_id not found",
        request_id,
    ))
}

fn normalize_executed_by_type(
    value: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let default_type = if matches!(
        actor_role,
        "platform_admin"
            | "platform_audit_security"
            | "platform_risk_settlement"
            | "audit_admin"
            | "retention_admin"
    ) {
        "platform"
    } else if matches!(actor_role, "seller_operator" | "seller_storage_operator") {
        "seller"
    } else if matches!(actor_role, "buyer_operator" | "procurement_manager") {
        "buyer"
    } else {
        "tenant"
    };
    let executed_by_type = value.unwrap_or(default_type).trim().to_ascii_lowercase();
    if matches!(
        executed_by_type.as_str(),
        "platform" | "seller" | "buyer" | "tenant" | "partner" | "regulator"
    ) {
        return Ok(executed_by_type);
    }
    Err(bad_request(
        "executed_by_type must be one of: platform, seller, buyer, tenant, partner, regulator",
        request_id,
    ))
}

fn normalize_ref_type(
    value: Option<&str>,
    default_value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let ref_type = value
        .or(default_value)
        .unwrap_or("delivery_object")
        .trim()
        .to_ascii_lowercase();
    if matches!(
        ref_type.as_str(),
        "delivery_object" | "query_result" | "report_result" | "sandbox_artifact"
    ) {
        return Ok(ref_type);
    }
    Err(bad_request(
        "ref_type must be one of: delivery_object, query_result, report_result, sandbox_artifact",
        request_id,
    ))
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn build_metadata(
    client_metadata: Option<&Value>,
    actor_role: &str,
    context: &OrderContext,
    object: &ObjectContext,
    retention_action: &str,
    status: &str,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut object_map = match client_metadata.cloned().unwrap_or_else(|| json!({})) {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    object_map.insert(
        "order_snapshot".to_string(),
        json!({
            "current_state": context.current_state,
            "payment_status": context.payment_status,
            "delivery_status": context.delivery_status,
            "dispute_status": context.dispute_status,
        }),
    );
    object_map.insert(
        "object_snapshot".to_string(),
        json!({
            "link_type": object.link_type,
            "link_status": object.link_status,
            "object_uri": object.object_uri,
        }),
    );
    object_map.insert(
        "proof_snapshot".to_string(),
        json!({
            "retention_action": retention_action,
            "status": status,
            "actor_role": actor_role,
            "placeholder_v1": true,
        }),
    );
    Ok(Value::Object(object_map))
}
