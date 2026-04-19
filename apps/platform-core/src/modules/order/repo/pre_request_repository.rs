use crate::modules::authorization::domain::AuthorizationExpectationSnapshot;
use crate::modules::contract::domain::ContractExpectationSnapshot;
use crate::modules::order::domain::{
    InquiryStatus, PreOrderRequestKind, PreOrderRequestPayload, TradePreRequest,
    TradePreRequestDetails,
};
use crate::modules::order::dto::CreateTradePreRequestRequest;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InquiryPayloadV1 {
    request_kind: PreOrderRequestKind,
    details: TradePreRequestDetails,
    contract_expectation: ContractExpectationSnapshot,
    authorization_expectation: AuthorizationExpectationSnapshot,
}

pub async fn insert_trade_pre_request(
    client: &Client,
    payload: &CreateTradePreRequestRequest,
) -> Result<TradePreRequest, (StatusCode, Json<ErrorResponse>)> {
    let stored_payload = InquiryPayloadV1 {
        request_kind: payload.request_kind,
        details: payload.details.clone(),
        contract_expectation: payload.contract_expectation.clone().unwrap_or_default(),
        authorization_expectation: payload
            .authorization_expectation
            .clone()
            .unwrap_or_default(),
    };
    let payload_text = serde_json::to_string(&stored_payload).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("failed to serialize pre-request payload: {err}"),
                request_id: None,
            }),
        )
    })?;

    let row = client
        .query_one(
            "INSERT INTO trade.inquiry (
               buyer_org_id,
               product_id,
               status,
               message_text,
               created_by
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               'open',
               $3,
               $4::text::uuid
             )
             RETURNING
               inquiry_id::text,
               buyer_org_id::text,
               product_id::text,
               created_by::text,
               status,
               message_text,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &payload.buyer_org_id,
                &payload.product_id,
                &payload_text,
                &payload.created_by,
            ],
        )
        .await
        .map_err(map_db_error)?;
    parse_pre_request_row(&row)
}

pub async fn load_trade_pre_request(
    client: &Client,
    inquiry_id: &str,
) -> Result<Option<TradePreRequest>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               inquiry_id::text,
               buyer_org_id::text,
               product_id::text,
               created_by::text,
               status,
               message_text,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.inquiry
             WHERE inquiry_id = $1::text::uuid",
            &[&inquiry_id],
        )
        .await
        .map_err(map_db_error)?;
    row.map(|r| parse_pre_request_row(&r)).transpose()
}

fn parse_pre_request_row(row: &Row) -> Result<TradePreRequest, (StatusCode, Json<ErrorResponse>)> {
    let payload_text: Option<String> = row.get(5);
    let payload = payload_text
        .as_deref()
        .and_then(|raw| serde_json::from_str::<InquiryPayloadV1>(raw).ok())
        .unwrap_or_else(default_rfq_payload);

    Ok(TradePreRequest {
        inquiry_id: row.get::<_, String>(0),
        buyer_org_id: row.get::<_, String>(1),
        product_id: row.get::<_, Option<String>>(2),
        created_by: row.get::<_, Option<String>>(3),
        status: InquiryStatus::parse(&row.get::<_, String>(4)),
        request_payload: PreOrderRequestPayload {
            request_kind: payload.request_kind,
            details: payload.details,
            contract_expectation: payload.contract_expectation,
            authorization_expectation: payload.authorization_expectation,
        },
        created_at: row.get::<_, String>(6),
        updated_at: row.get::<_, String>(7),
    })
}

fn default_rfq_payload() -> InquiryPayloadV1 {
    InquiryPayloadV1 {
        request_kind: PreOrderRequestKind::Rfq,
        details: TradePreRequestDetails::default(),
        contract_expectation: ContractExpectationSnapshot::default(),
        authorization_expectation: AuthorizationExpectationSnapshot::default(),
    }
}

pub async fn write_trade_audit_event(
    client: &(impl GenericClient + Sync),
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let metadata = serde_json::json!({
        "actor_role": actor_role,
        "request_kind_object": ref_type,
    });
    client
        .execute(
            "INSERT INTO audit.audit_event (
               domain_name,
               ref_type,
               ref_id,
               actor_type,
               action_name,
               result_code,
               request_id,
               trace_id,
               metadata
             ) VALUES (
               'trade',
               $2,
               $1::text::uuid,
               'role',
               $3,
               $4,
               $5,
               $6,
               $7::jsonb
             )",
            &[
                &ref_id,
                &ref_type,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

pub fn map_db_error(err: Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("trade persistence failed: {err}"),
            request_id: None,
        }),
    )
}
