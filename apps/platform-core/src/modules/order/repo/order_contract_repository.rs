use crate::modules::order::domain::derive_layered_status;
use crate::modules::order::dto::{ConfirmOrderContractRequest, ConfirmOrderContractResponseData};
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::Client;

pub struct ContractConfirmContext {
    pub buyer_org_id: String,
    pub seller_org_id: String,
}

pub async fn load_order_contract_confirm_context(
    client: &Client,
    order_id: &str,
) -> Result<Option<ContractConfirmContext>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT buyer_org_id::text, seller_org_id::text
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    Ok(row.map(|v| ContractConfirmContext {
        buyer_org_id: v.get(0),
        seller_org_id: v.get(1),
    }))
}

pub async fn confirm_order_contract(
    client: &mut Client,
    order_id: &str,
    payload: &ConfirmOrderContractRequest,
    signer_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ConfirmOrderContractResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT status, payment_status
             FROM trade.order_main
             WHERE order_id = $1::text::uuid
             FOR UPDATE",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    let current_status: String = row.get(0);
    let current_payment_status: String = row.get(1);
    if !matches!(
        current_status.as_str(),
        "created" | "contract_pending" | "contract_effective"
    ) {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!(
                    "ORDER_CONTRACT_CONFIRM_FORBIDDEN: current state `{current_status}` is not confirmable"
                ),
                request_id: request_id.map(str::to_string),
            }),
        ));
    }

    let upserted = tx
        .query_one(
            "INSERT INTO contract.digital_contract (
               order_id,
               contract_template_id,
               data_contract_id,
               contract_digest,
               data_contract_digest,
               status,
               signed_at,
               variables_json
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4,
               $5,
               'signed',
               now(),
               $6::jsonb
             )
             ON CONFLICT (order_id) DO UPDATE
             SET contract_template_id = EXCLUDED.contract_template_id,
                 data_contract_id = EXCLUDED.data_contract_id,
                 contract_digest = EXCLUDED.contract_digest,
                 data_contract_digest = EXCLUDED.data_contract_digest,
                 status = 'signed',
                 signed_at = now(),
                 variables_json = EXCLUDED.variables_json
             RETURNING
               contract_id::text,
               contract_template_id::text,
               contract_digest,
               data_contract_id::text,
               data_contract_digest,
               status,
               to_char(signed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               variables_json",
            &[
                &order_id,
                &payload.contract_template_id,
                &payload.data_contract_id,
                &payload.contract_digest,
                &payload.data_contract_digest,
                &payload.variables_json,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let contract_id: String = upserted.get(0);
    let contract_template_id: String = upserted.get(1);
    let contract_digest: String = upserted.get(2);
    let data_contract_id: Option<String> = upserted.get(3);
    let data_contract_digest: Option<String> = upserted.get(4);
    let contract_status: String = upserted.get(5);
    let signed_at: String = upserted.get(6);
    let variables_json: serde_json::Value = upserted.get(7);

    let signer_exists = tx
        .query_opt(
            "SELECT contract_signer_id::text
             FROM contract.contract_signer
             WHERE contract_id = $1::text::uuid
               AND signer_id = $2::text::uuid
               AND signer_role = $3
             LIMIT 1",
            &[&contract_id, &signer_id, &payload.signer_role],
        )
        .await
        .map_err(map_db_error)?
        .is_some();
    if !signer_exists {
        tx.execute(
            "INSERT INTO contract.contract_signer (
               contract_id,
               signer_type,
               signer_id,
               signer_role,
               signature_digest,
               signed_at
             ) VALUES (
               $1::text::uuid,
               'user',
               $2::text::uuid,
               $3,
               $4,
               now()
             )",
            &[
                &contract_id,
                &signer_id,
                &payload.signer_role,
                &payload.contract_digest,
            ],
        )
        .await
        .map_err(map_db_error)?;
    }

    let order_status = "contract_effective".to_string();
    let layered_status = derive_layered_status(&order_status, &current_payment_status);
    tx.execute(
        "UPDATE trade.order_main
         SET contract_id = $2::text::uuid,
             status = CASE
                        WHEN status IN ('created', 'contract_pending') THEN 'contract_effective'
                        ELSE status
                      END,
             delivery_status = $3,
             acceptance_status = $4,
             settlement_status = $5,
             dispute_status = $6,
             last_reason_code = 'TRADE-006',
             updated_at = now()
         WHERE order_id = $1::text::uuid",
        &[
            &order_id,
            &contract_id,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
        ],
    )
    .await
    .map_err(map_db_error)?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        "trade.contract.confirm",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ConfirmOrderContractResponseData {
        order_id: order_id.to_string(),
        contract_id,
        contract_template_id,
        contract_digest: contract_digest.clone(),
        data_contract_id,
        data_contract_digest,
        contract_status,
        order_status,
        signer_id: signer_id.to_string(),
        signer_type: "user".to_string(),
        signer_role: payload.signer_role.clone(),
        signed_at,
        variables_json,
        onchain_digest_ref: contract_digest,
    })
}
