use crate::modules::authorization::domain::{
    build_authorization_model_snapshot, extract_or_build_authorization_model,
};
use crate::modules::order::dto::{
    AcceptanceLifecycleSnapshot, AuthorizationLifecycleSnapshot, ContractLifecycleSnapshot,
    DeliveryLifecycleSnapshot, DisputeLifecycleSnapshot, GetOrderLifecycleSnapshotsResponseData,
    OrderLifecycleSnapshot, PaymentLifecycleSnapshot, SettlementLifecycleSnapshot,
};
use crate::modules::order::repo::pre_request_repository::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::ErrorResponse;
use serde_json::Value;

pub async fn load_order_lifecycle_snapshots(
    client: &Client,
    order_id: &str,
) -> Result<Option<GetOrderLifecycleSnapshotsResponseData>, (StatusCode, Json<ErrorResponse>)> {
    let order_row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status,
               payment_mode,
               amount::text,
               currency_code,
               acceptance_status,
               settlement_status,
               dispute_status,
               last_reason_code,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(buyer_locked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(order_row) = order_row else {
        return Ok(None);
    };

    let contract = load_contract_snapshot(client, order_id).await?;
    let authorization = load_authorization_snapshot(client, order_id).await?;
    let delivery = load_delivery_snapshot(client, order_id).await?;

    let payment_status: String = order_row.get(4);
    let acceptance_status: Option<String> = order_row.get(8);
    let settlement_status: Option<String> = order_row.get(9);
    let dispute_status: Option<String> = order_row.get(10);

    Ok(Some(GetOrderLifecycleSnapshotsResponseData {
        order: OrderLifecycleSnapshot {
            order_id: order_row.get(0),
            buyer_org_id: order_row.get(1),
            seller_org_id: order_row.get(2),
            current_state: order_row.get(3),
            payment: PaymentLifecycleSnapshot {
                current_status: payment_status,
                payment_mode: order_row.get(5),
                amount: order_row.get(6),
                currency_code: order_row.get(7),
                buyer_locked_at: order_row.get(14),
            },
            acceptance: AcceptanceLifecycleSnapshot {
                current_status: acceptance_status.unwrap_or_else(|| "not_started".to_string()),
                accepted_at: order_row.get(15),
            },
            settlement: SettlementLifecycleSnapshot {
                current_status: settlement_status.unwrap_or_else(|| "not_started".to_string()),
                settled_at: order_row.get(16),
                closed_at: order_row.get(17),
            },
            dispute: DisputeLifecycleSnapshot {
                current_status: dispute_status.unwrap_or_else(|| "none".to_string()),
                last_reason_code: order_row.get(11),
            },
            created_at: order_row.get(12),
            updated_at: order_row.get(13),
        },
        contract,
        authorization,
        delivery,
    }))
}

async fn load_contract_snapshot(
    client: &Client,
    order_id: &str,
) -> Result<Option<ContractLifecycleSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               contract_id::text,
               status,
               contract_digest,
               to_char(signed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               variables_json,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM contract.digital_contract
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    row.map(|row| {
        Ok(ContractLifecycleSnapshot {
            contract_id: row.get(0),
            contract_status: row.get(1),
            contract_digest: row.get(2),
            signed_at: row.get(3),
            variables_json: row.get::<_, Value>(4),
            updated_at: row.get(5),
        })
    })
    .transpose()
}

async fn load_authorization_snapshot(
    client: &Client,
    order_id: &str,
) -> Result<Option<AuthorizationLifecycleSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               ag.authorization_grant_id::text,
               ag.status,
               ag.grant_type,
               ag.granted_to_type,
               ag.granted_to_id::text,
               to_char(ag.valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ag.valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ag.policy_snapshot,
               to_char(ag.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               o.product_id::text,
               o.sku_id::text,
               s.sku_type
             FROM trade.authorization_grant ag
             JOIN trade.order_main o ON o.order_id = ag.order_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE ag.order_id = $1::text::uuid
             ORDER BY ag.updated_at DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    row.map(|row| {
        let policy_snapshot: Value = row.get(7);
        let fallback = build_authorization_model_snapshot(
            order_id,
            row.get::<_, String>(9).as_str(),
            row.get::<_, String>(10).as_str(),
            row.get::<_, String>(11).as_str(),
            policy_snapshot
                .get("policy_id")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            row.get::<_, String>(3).as_str(),
            row.get::<_, String>(4).as_str(),
            row.get::<_, String>(2).as_str(),
            policy_snapshot
                .get("subject_constraints")
                .unwrap_or(&Value::Null),
            policy_snapshot
                .get("usage_constraints")
                .unwrap_or(&Value::Null),
            policy_snapshot
                .get("exportable")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        Ok(AuthorizationLifecycleSnapshot {
            authorization_id: row.get(0),
            current_status: row.get(1),
            grant_type: row.get(2),
            granted_to_type: row.get(3),
            granted_to_id: row.get(4),
            valid_from: row.get(5),
            valid_to: row.get(6),
            authorization_model: extract_or_build_authorization_model(&policy_snapshot, fallback),
            policy_snapshot,
            updated_at: row.get(8),
        })
    })
    .transpose()
}

async fn load_delivery_snapshot(
    client: &Client,
    order_id: &str,
) -> Result<Option<DeliveryLifecycleSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               delivery_id::text,
               delivery_type,
               delivery_route,
               status,
               to_char(committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               receipt_hash,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.delivery_record
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    row.map(|row| {
        Ok(DeliveryLifecycleSnapshot {
            delivery_id: row.get(0),
            delivery_type: row.get(1),
            delivery_route: row.get(2),
            current_status: row.get(3),
            committed_at: row.get(4),
            expires_at: row.get(5),
            receipt_hash: row.get(6),
            updated_at: row.get(7),
        })
    })
    .transpose()
}
