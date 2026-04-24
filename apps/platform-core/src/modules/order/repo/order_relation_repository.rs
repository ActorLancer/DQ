use crate::modules::authorization::domain::{
    build_authorization_model_snapshot, extract_or_build_authorization_model,
};
use crate::modules::delivery::repo::load_storage_gateway_snapshots;
use crate::modules::order::dto::{
    OrderAuthorizationRelation, OrderBillingEventRelation, OrderBillingRelations,
    OrderCompensationRelation, OrderContractRelation, OrderDeliveryRelation, OrderDisputeRelation,
    OrderInvoiceRelation, OrderRefundRelation, OrderRelations, OrderSettlementRelation,
};
use crate::modules::order::repo::pre_request_repository::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::ErrorResponse;
use serde_json::Value;

pub async fn load_order_relations(
    client: &Client,
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
    client: &Client,
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
    client: &Client,
    order_id: &str,
) -> Result<Vec<OrderAuthorizationRelation>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
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
             ORDER BY ag.updated_at DESC, ag.authorization_grant_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| {
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
            OrderAuthorizationRelation {
                authorization_id: row.get(0),
                current_status: row.get(1),
                grant_type: row.get(2),
                granted_to_type: row.get(3),
                granted_to_id: row.get(4),
                valid_from: row.get(5),
                valid_to: row.get(6),
                authorization_model: extract_or_build_authorization_model(
                    &policy_snapshot,
                    fallback,
                ),
                policy_snapshot,
                updated_at: row.get(8),
            }
        })
        .collect())
}

async fn load_delivery_relations(
    client: &Client,
    order_id: &str,
) -> Result<Vec<OrderDeliveryRelation>, (StatusCode, Json<ErrorResponse>)> {
    let gateway_snapshots = load_storage_gateway_snapshots(client, order_id).await?;
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
        .map(|row| {
            let delivery_id: String = row.get(0);
            OrderDeliveryRelation {
                storage_gateway: gateway_snapshots.get(&delivery_id).cloned(),
                delivery_id,
                delivery_type: row.get(1),
                delivery_route: row.get(2),
                current_status: row.get(3),
                delivery_commit_hash: row.get(4),
                receipt_hash: row.get(5),
                committed_at: row.get(6),
                expires_at: row.get(7),
                updated_at: row.get(8),
            }
        })
        .collect())
}

async fn load_billing_relations(
    client: &Client,
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
    client: &Client,
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
            metered_quantity: row.get(5),
            occurred_at: row.get(6),
            metadata: row.get(7),
        })
        .collect())
}

async fn load_settlements(
    client: &Client,
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
    client: &Client,
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
    client: &Client,
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
    client: &Client,
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
    client: &Client,
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
