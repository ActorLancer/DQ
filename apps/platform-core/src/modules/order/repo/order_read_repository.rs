use crate::modules::order::domain::OrderPriceSnapshot;
use crate::modules::order::dto::GetOrderDetailResponseData;
use crate::modules::order::repo::pre_request_repository::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::Value;

pub async fn load_order_detail(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Option<GetOrderDetailResponseData>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               product_id::text,
               sku_id::text,
               status,
               payment_status,
               amount::text,
               currency_code,
               price_snapshot_json,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    row.map(parse_order_row).transpose()
}

fn parse_order_row(
    row: tokio_postgres::Row,
) -> Result<GetOrderDetailResponseData, (StatusCode, Json<ErrorResponse>)> {
    let current_state: String = row.get(5);
    let payment_status: String = row.get(6);
    let price_snapshot_value: Value = row.get(9);
    let price_snapshot = if price_snapshot_value
        .as_object()
        .is_some_and(|v| !v.is_empty())
    {
        Some(
            serde_json::from_value::<OrderPriceSnapshot>(price_snapshot_value).map_err(|err| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        code: ErrorCode::TrdStateConflict.as_str().to_string(),
                        message: format!("invalid order snapshot payload: {err}"),
                        request_id: None,
                    }),
                )
            })?,
        )
    } else {
        None
    };

    let (delivery_status, acceptance_status, settlement_status, dispute_status) =
        derive_layered_status(&current_state, &payment_status);

    Ok(GetOrderDetailResponseData {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        product_id: row.get(3),
        sku_id: row.get(4),
        current_state,
        payment_status,
        delivery_status,
        acceptance_status,
        settlement_status,
        dispute_status,
        amount: row.get(7),
        currency_code: row.get(8),
        price_snapshot,
        created_at: row.get(10),
        updated_at: row.get(11),
    })
}

fn derive_layered_status(
    current_state: &str,
    payment_status: &str,
) -> (String, String, String, String) {
    let delivery_status = match current_state {
        "created"
        | "buyer_locked"
        | "payment_failed_pending_resolution"
        | "payment_timeout_pending_compensation_cancel" => "pending_delivery",
        "seller_delivering" => "in_progress",
        "delivered" | "accepted" | "settled" | "closed" => "delivered",
        _ => "pending_delivery",
    };
    let acceptance_status = match current_state {
        "delivered" => "pending_acceptance",
        "accepted" | "settled" | "closed" => "accepted",
        _ => "not_started",
    };
    let settlement_status = match current_state {
        "settled" => "settled",
        "closed" => "closed",
        _ => {
            if payment_status == "paid" {
                "pending_settlement"
            } else {
                "not_started"
            }
        }
    };
    let dispute_status = "none";
    (
        delivery_status.to_string(),
        acceptance_status.to_string(),
        settlement_status.to_string(),
        dispute_status.to_string(),
    )
}
