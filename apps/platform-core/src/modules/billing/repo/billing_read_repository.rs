use crate::modules::billing::db::map_db_error;
use crate::modules::billing::models::{
    BillingCompensationView, BillingInvoicePlaceholderView, BillingInvoiceView,
    BillingOrderDetailView, BillingPayoutView, BillingRefundView, BillingSettlementSummaryView,
    BillingSettlementView, BillingSplitInstructionView, BillingTaxPlaceholderView,
};
use crate::modules::billing::repo::billing_event_repository::list_billing_events_for_order;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};

struct BillingOrderContext {
    order_id: String,
    buyer_org_id: String,
    seller_org_id: String,
    order_status: String,
    payment_status: String,
    settlement_status: String,
    dispute_status: String,
    order_amount: String,
    currency_code: String,
}

pub async fn get_billing_order_detail(
    client: &Client,
    order_id: &str,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<BillingOrderDetailView>, (StatusCode, Json<ErrorResponse>)> {
    let Some(context) = load_billing_order_context(client, order_id).await? else {
        return Ok(None);
    };
    enforce_order_scope(&context, tenant_scope_id, request_id)?;

    let billing_events =
        list_billing_events_for_order(client, order_id, tenant_scope_id, request_id).await?;
    let settlements = load_settlements(client, order_id).await?;
    let refunds = load_refunds(client, order_id).await?;
    let compensations = load_compensations(client, order_id).await?;
    let payouts = load_payouts(client, order_id).await?;
    let split_placeholders = load_split_placeholders(client, order_id).await?;
    let invoices = load_invoices(client, order_id).await?;
    let settlement_summary = build_settlement_summary(&settlements);

    let tax_placeholder = build_tax_placeholder(&context, &invoices);
    let invoice_placeholder = build_invoice_placeholder(&settlements, &invoices);

    Ok(Some(BillingOrderDetailView {
        order_id: context.order_id,
        order_status: context.order_status,
        payment_status: context.payment_status,
        settlement_status: context.settlement_status,
        dispute_status: context.dispute_status,
        order_amount: context.order_amount,
        currency_code: context.currency_code,
        billing_events,
        settlements,
        settlement_summary,
        refunds,
        compensations,
        payouts,
        split_placeholders,
        invoices,
        tax_placeholder,
        invoice_placeholder,
    }))
}

async fn load_billing_order_context(
    client: &Client,
    order_id: &str,
) -> Result<Option<BillingOrderContext>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               order_id::text,
               buyer_org_id::text,
               seller_org_id::text,
               status,
               payment_status,
               settlement_status,
               dispute_status,
               amount::text,
               currency_code
             FROM trade.order_main
             WHERE order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| BillingOrderContext {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        order_status: row.get(3),
        payment_status: row.get(4),
        settlement_status: row.get(5),
        dispute_status: row.get(6),
        order_amount: row.get(7),
        currency_code: row.get(8),
    }))
}

fn enforce_order_scope(
    context: &BillingOrderContext,
    tenant_scope_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };
    if tenant_scope_id == context.buyer_org_id || tenant_scope_id == context.seller_org_id {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "tenant scope does not match billing order".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

async fn load_settlements(
    client: &Client,
    order_id: &str,
) -> Result<Vec<BillingSettlementView>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               settlement_id::text,
               settlement_type,
               settlement_status,
               settlement_mode,
               payable_amount::text,
               platform_fee_amount::text,
               channel_fee_amount::text,
               net_receivable_amount::text,
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
        .map(|row| BillingSettlementView {
            settlement_id: row.get(0),
            settlement_type: row.get(1),
            settlement_status: row.get(2),
            settlement_mode: row.get(3),
            payable_amount: row.get(4),
            platform_fee_amount: row.get(5),
            channel_fee_amount: row.get(6),
            net_receivable_amount: row.get(7),
            refund_amount: row.get(8),
            compensation_amount: row.get(9),
            reason_code: row.get(10),
            settled_at: row.get(11),
            updated_at: row.get(12),
        })
        .collect())
}

async fn load_refunds(
    client: &Client,
    order_id: &str,
) -> Result<Vec<BillingRefundView>, (StatusCode, Json<ErrorResponse>)> {
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
        .map(|row| BillingRefundView {
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
) -> Result<Vec<BillingCompensationView>, (StatusCode, Json<ErrorResponse>)> {
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
        .map(|row| BillingCompensationView {
            compensation_id: row.get(0),
            amount: row.get(1),
            currency_code: row.get(2),
            current_status: row.get(3),
            executed_at: row.get(4),
            updated_at: row.get(5),
        })
        .collect())
}

async fn load_payouts(
    client: &Client,
    order_id: &str,
) -> Result<Vec<BillingPayoutView>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               p.payout_instruction_id::text,
               p.settlement_id::text,
               p.provider_key,
               p.provider_account_id::text,
               p.payout_preference_id::text,
               p.beneficiary_subject_type,
               p.beneficiary_subject_id::text,
               p.destination_jurisdiction_code,
               p.amount::text,
               p.currency_code,
               p.payout_mode,
               p.status,
               p.provider_payout_no,
               to_char(p.executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(p.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.payout_instruction p
             JOIN billing.settlement_record s ON s.settlement_id = p.settlement_id
             WHERE s.order_id = $1::text::uuid
             ORDER BY p.updated_at DESC, p.payout_instruction_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| BillingPayoutView {
            payout_instruction_id: row.get(0),
            settlement_id: row.get(1),
            provider_key: row.get(2),
            provider_account_id: row.get(3),
            payout_preference_id: row.get(4),
            beneficiary_subject_type: row.get(5),
            beneficiary_subject_id: row.get(6),
            destination_jurisdiction_code: row.get(7),
            amount: row.get(8),
            currency_code: row.get(9),
            payout_mode: row.get(10),
            current_status: row.get(11),
            provider_payout_no: row.get(12),
            executed_at: row.get(13),
            updated_at: row.get(14),
        })
        .collect())
}

async fn load_split_placeholders(
    client: &Client,
    order_id: &str,
) -> Result<Vec<BillingSplitInstructionView>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               si.split_instruction_id::text,
               si.settlement_id::text,
               si.reward_id::text,
               si.provider_account_id::text,
               si.sub_merchant_binding_id::text,
               si.split_mode,
               si.amount::text,
               si.currency_code,
               si.status,
               si.provider_split_no,
               to_char(si.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.split_instruction si
             JOIN billing.settlement_record s ON s.settlement_id = si.settlement_id
             WHERE s.order_id = $1::text::uuid
             ORDER BY si.updated_at DESC, si.split_instruction_id DESC",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|row| BillingSplitInstructionView {
            split_instruction_id: row.get(0),
            settlement_id: row.get(1),
            reward_id: row.get(2),
            provider_account_id: row.get(3),
            sub_merchant_binding_id: row.get(4),
            split_mode: row.get(5),
            amount: row.get(6),
            currency_code: row.get(7),
            current_status: row.get(8),
            provider_split_no: row.get(9),
            updated_at: row.get(10),
        })
        .collect())
}

async fn load_invoices(
    client: &Client,
    order_id: &str,
) -> Result<Vec<BillingInvoiceView>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT
               invoice_request_id::text,
               settlement_id::text,
               requester_org_id::text,
               invoice_title,
               tax_no,
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
        .map(|row| BillingInvoiceView {
            invoice_request_id: row.get(0),
            settlement_id: row.get(1),
            requester_org_id: row.get(2),
            invoice_title: row.get(3),
            tax_no: row.get(4),
            amount: row.get(5),
            currency_code: row.get(6),
            current_status: row.get(7),
            updated_at: row.get(8),
        })
        .collect())
}

fn build_tax_placeholder(
    context: &BillingOrderContext,
    invoices: &[BillingInvoiceView],
) -> BillingTaxPlaceholderView {
    let latest_invoice = invoices.first();
    BillingTaxPlaceholderView {
        tax_engine_status: "placeholder".to_string(),
        tax_rule_code: "tax-placeholder-v1".to_string(),
        currency_code: context.currency_code.clone(),
        latest_invoice_title: latest_invoice.map(|invoice| invoice.invoice_title.clone()),
        latest_tax_no: latest_invoice.and_then(|invoice| invoice.tax_no.clone()),
        tax_breakdown_ready: false,
    }
}

fn build_invoice_placeholder(
    settlements: &[BillingSettlementView],
    invoices: &[BillingInvoiceView],
) -> BillingInvoicePlaceholderView {
    let latest_invoice = invoices.first();
    let pending_invoice_count = invoices
        .iter()
        .filter(|invoice| matches!(invoice.current_status.as_str(), "pending" | "processing"))
        .count() as i64;
    BillingInvoicePlaceholderView {
        invoice_mode: "manual_placeholder".to_string(),
        invoice_required: !settlements.is_empty(),
        latest_invoice_request_id: latest_invoice.map(|invoice| invoice.invoice_request_id.clone()),
        latest_invoice_status: latest_invoice.map(|invoice| invoice.current_status.clone()),
        latest_invoice_title: latest_invoice.map(|invoice| invoice.invoice_title.clone()),
        pending_invoice_count,
    }
}

fn build_settlement_summary(
    settlements: &[BillingSettlementView],
) -> Option<BillingSettlementSummaryView> {
    let latest = settlements.first()?;
    Some(BillingSettlementSummaryView {
        gross_amount: latest.payable_amount.clone(),
        platform_commission_amount: latest.platform_fee_amount.clone(),
        channel_fee_amount: latest.channel_fee_amount.clone(),
        refund_adjustment_amount: latest.refund_amount.clone(),
        compensation_adjustment_amount: latest.compensation_amount.clone(),
        supplier_receivable_amount: latest.net_receivable_amount.clone(),
        summary_state: format!(
            "{}:{}:{}",
            latest.settlement_type, latest.settlement_status, latest.settlement_mode
        ),
        proof_commit_state: "pending_anchor".to_string(),
    })
}
