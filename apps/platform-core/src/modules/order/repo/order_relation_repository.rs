use crate::modules::order::dto::{
    OrderAuthorizationRelation, OrderBillingEventRelation, OrderBillingRelations,
    OrderCompensationRelation, OrderContractRelation, OrderDeliveryRelation, OrderDisputeRelation,
    OrderInvoiceRelation, OrderRefundRelation, OrderRelations, OrderSettlementRelation,
};
use crate::modules::order::repo::pre_request_repository::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use kernel::ErrorResponse;
use serde_json::Value;

pub async fn load_order_relations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<OrderRelations, (StatusCode, Json<ErrorResponse>)> {
    Ok(OrderRelations {
        contract: load_contract_relation(client, order_id).await?,
        authorizations: load_authorization_relations(client, order_id).await?,
        deliveries: load_delivery_relations(client, order_id).await?,
        billing: load_billing_relations(client, order_id).await?,
        disputes: load_dispute_relations(client, order_id).await?,
    })
}

async fn load_contract_relation(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Option<OrderContractRelation>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               contract_id::text,
               contract_template_id::text,
               status,
               contract_digest,
               data_contract_id::text,
               data_contract_digest,
               to_char(signed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               variables_json,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM contract.digital_contract
             WHERE order_id = $1::text::uuid
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    row.map(|row| {
        Ok(OrderContractRelation {
            contract_id: row.get(0),
            contract_template_id: row.get(1),
            contract_status: row.get(2),
            contract_digest: row.get(3),
            data_contract_id: row.get(4),
            data_contract_digest: row.get(5),
            signed_at: row.get(6),
            variables_json: row.get::<_, Value>(7),
            updated_at: row.get(8),
        })
    })
    .transpose()
}

async fn load_authorization_relations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderAuthorizationRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               authorization_grant_id::text,
               status,
               grant_type,
               granted_to_type,
               granted_to_id::text,
               to_char(valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               policy_snapshot,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM trade.authorization_grant
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, authorization_grant_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderAuthorizationRelation {
            authorization_id: row.get(0),
            current_status: row.get(1),
            grant_type: row.get(2),
            granted_to_type: row.get(3),
            granted_to_id: row.get(4),
            valid_from: row.get(5),
            valid_to: row.get(6),
            policy_snapshot: row.get(7),
            updated_at: row.get(8),
        })
        .collect())
}

async fn load_delivery_relations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderDeliveryRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               delivery_id::text,
               delivery_type,
               delivery_route,
               status,
               delivery_commit_hash,
               receipt_hash,
               to_char(committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.delivery_record
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, delivery_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderDeliveryRelation {
            delivery_id: row.get(0),
            delivery_type: row.get(1),
            delivery_route: row.get(2),
            current_status: row.get(3),
            delivery_commit_hash: row.get(4),
            receipt_hash: row.get(5),
            committed_at: row.get(6),
            expires_at: row.get(7),
            updated_at: row.get(8),
        })
        .collect())
}

async fn load_billing_relations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<OrderBillingRelations, (StatusCode, Json<ErrorResponse>)> {
    Ok(OrderBillingRelations {
        billing_events: load_billing_events(client, order_id).await?,
        settlements: load_settlements(client, order_id).await?,
        refunds: load_refunds(client, order_id).await?,
        compensations: load_compensations(client, order_id).await?,
        invoices: load_invoices(client, order_id).await?,
    })
}

async fn load_billing_events(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderBillingEventRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               billing_event_id::text,
               event_type,
               event_source,
               amount::text,
               currency_code,
               units::text,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               metadata
             FROM billing.billing_event
             WHERE order_id = $1::text::uuid
             ORDER BY occurred_at DESC, billing_event_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderBillingEventRelation {
            billing_event_id: row.get(0),
            event_type: row.get(1),
            event_source: row.get(2),
            amount: row.get(3),
            currency_code: row.get(4),
            units: row.get(5),
            occurred_at: row.get(6),
            metadata: row.get(7),
        })
        .collect())
}

async fn load_settlements(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderSettlementRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               settlement_id::text,
               settlement_type,
               settlement_status,
               settlement_mode,
               payable_amount::text,
               refund_amount::text,
               compensation_amount::text,
               reason_code,
               to_char(settled_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM billing.settlement_record
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, settlement_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderSettlementRelation {
            settlement_id: row.get(0),
            settlement_type: row.get(1),
            settlement_status: row.get(2),
            settlement_mode: row.get(3),
            payable_amount: row.get(4),
            refund_amount: row.get(5),
            compensation_amount: row.get(6),
            reason_code: row.get(7),
            settled_at: row.get(8),
            updated_at: row.get(9),
        })
        .collect())
}

async fn load_refunds(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderRefundRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               refund_id::text,
               amount::text,
               currency_code,
               status,
               to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM billing.refund_record
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, refund_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderRefundRelation {
            refund_id: row.get(0),
            amount: row.get(1),
            currency_code: row.get(2),
            current_status: row.get(3),
            executed_at: row.get(4),
            updated_at: row.get(5),
        })
        .collect())
}

async fn load_compensations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderCompensationRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               compensation_id::text,
               amount::text,
               currency_code,
               status,
               to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM billing.compensation_record
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, compensation_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderCompensationRelation {
            compensation_id: row.get(0),
            amount: row.get(1),
            currency_code: row.get(2),
            current_status: row.get(3),
            executed_at: row.get(4),
            updated_at: row.get(5),
        })
        .collect())
}

async fn load_invoices(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderInvoiceRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               invoice_request_id::text,
               settlement_id::text,
               requester_org_id::text,
               invoice_title,
               amount::text,
               currency_code,
               status,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM billing.invoice_request
             WHERE order_id = $1::text::uuid
             ORDER BY updated_at DESC, invoice_request_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderInvoiceRelation {
            invoice_request_id: row.get(0),
            settlement_id: row.get(1),
            requester_org_id: row.get(2),
            invoice_title: row.get(3),
            amount: row.get(4),
            currency_code: row.get(5),
            current_status: row.get(6),
            updated_at: row.get(7),
        })
        .collect())
}

async fn load_dispute_relations(
    client: &tokio_postgres::Client,
    order_id: &str,
) -> Result<Vec<OrderDisputeRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               dc.case_id::text,
               dc.complainant_type,
               dc.complainant_id::text,
               dc.reason_code,
               dc.status,
               dc.decision_code,
               dc.penalty_code,
               COALESCE(eo.evidence_count, 0)::bigint,
               to_char(dc.opened_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(dc.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(dc.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM support.dispute_case dc
             LEFT JOIN (
               SELECT case_id, COUNT(*) AS evidence_count
               FROM support.evidence_object
               GROUP BY case_id
             ) eo ON eo.case_id = dc.case_id
             WHERE dc.order_id = $1::text::uuid
             ORDER BY dc.updated_at DESC, dc.case_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| OrderDisputeRelation {
            case_id: row.get(0),
            complainant_type: row.get(1),
            complainant_id: row.get(2),
            reason_code: row.get(3),
            current_status: row.get(4),
            decision_code: row.get(5),
            penalty_code: row.get(6),
            evidence_count: row.get(7),
            opened_at: row.get(8),
            resolved_at: row.get(9),
            updated_at: row.get(10),
        })
        .collect())
}
