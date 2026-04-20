use crate::modules::delivery::dto::{
    OrderAttestationListResponseData, OrderAttestationResponseData,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::json;

const DELIVERY_ATTESTATION_READ_EVENT: &str = "delivery.attestation.read";

pub async fn get_order_attestations(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<OrderAttestationListResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_order_context(&tx, order_id, request_id).await?;
    enforce_read_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;

    let rows = tx
        .query(
            "SELECT a.attestation_record_id::text,
                    a.order_id::text,
                    a.query_run_id::text,
                    a.sandbox_session_id::text,
                    a.environment_id::text,
                    env.environment_name,
                    env.environment_type,
                    a.attestation_type,
                    a.attestation_uri,
                    a.attestation_hash,
                    a.verifier_ref,
                    to_char(a.verified_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    a.status,
                    COALESCE(a.metadata, '{}'::jsonb),
                    qr.status,
                    ss.session_status,
                    to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.attestation_record a
             LEFT JOIN core.execution_environment env
               ON env.environment_id = a.environment_id
             LEFT JOIN delivery.query_execution_run qr
               ON qr.query_run_id = a.query_run_id
             LEFT JOIN delivery.sandbox_session ss
               ON ss.sandbox_session_id = a.sandbox_session_id
             WHERE a.order_id = $1::text::uuid
             ORDER BY a.updated_at DESC, a.attestation_record_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    if rows.is_empty() {
        return Err(conflict(
            "ORDER_ATTESTATION_FORBIDDEN: no execution attestation exists for current order",
            request_id,
        ));
    }

    let mut attestations = Vec::with_capacity(rows.len());
    let mut attestation_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let attestation_record_id: String = row.get(0);
        let query_run_id: Option<String> = row.get(2);
        let sandbox_session_id: Option<String> = row.get(3);
        attestation_ids.push(attestation_record_id.clone());
        attestations.push(OrderAttestationResponseData {
            attestation_record_id,
            order_id: row.get(1),
            query_run_id: query_run_id.clone(),
            sandbox_session_id: sandbox_session_id.clone(),
            environment_id: row.get(4),
            environment_name: row.get(5),
            environment_type: row.get(6),
            attestation_type: row.get(7),
            attestation_uri: row.get(8),
            attestation_hash: row.get(9),
            verifier_ref: row.get(10),
            verified_at: row.get(11),
            status: row.get(12),
            metadata_json: row.get(13),
            source_type: if query_run_id.is_some() {
                "query_run".to_string()
            } else if sandbox_session_id.is_some() {
                "sandbox_session".to_string()
            } else {
                "execution_environment".to_string()
            },
            query_run_status: row.get(14),
            sandbox_session_status: row.get(15),
            created_at: row.get(16),
            updated_at: row.get(17),
        });
    }

    write_delivery_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        DELIVERY_ATTESTATION_READ_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "attestation_ids": attestation_ids,
            "attestation_count": attestations.len(),
            "current_state": context.current_state,
            "payment_status": context.payment_status,
            "delivery_status": context.delivery_status,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(OrderAttestationListResponseData {
        order_id: order_id.to_string(),
        current_state: context.current_state,
        payment_status: context.payment_status,
        delivery_status: context.delivery_status,
        attestations,
    })
}

#[derive(Debug)]
struct OrderContext {
    buyer_org_id: String,
    seller_org_id: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
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
                    delivery_status
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
    })
}

fn enforce_read_scope(
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
            | "compliance_reviewer"
    ) {
        return Ok(());
    }

    let Some(tenant_id) = tenant_id else {
        return Err(conflict(
            "ORDER_ATTESTATION_FORBIDDEN: tenant scope is required",
            request_id,
        ));
    };

    let allowed = match actor_role {
        "seller_operator" | "seller_storage_operator" | "sandbox_operator" => {
            tenant_id == seller_org_id
        }
        "buyer_operator" | "procurement_manager" => tenant_id == buyer_org_id,
        "tenant_admin" | "tenant_audit_readonly" => {
            tenant_id == buyer_org_id || tenant_id == seller_org_id
        }
        _ => false,
    };

    if allowed {
        return Ok(());
    }

    Err(conflict(
        "ORDER_ATTESTATION_FORBIDDEN: tenant scope does not match order participants",
        request_id,
    ))
}
