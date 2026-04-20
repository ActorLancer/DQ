use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::models::{
    BillingSplitInstructionView, CreateManualPayoutRequest, ManualPayoutExecutionView,
};
use crate::modules::billing::repo::billing_adjustment_repository::release_provisional_dispute_hold_in_tx;
use crate::modules::billing::repo::settlement_aggregate_repository::recompute_settlement_for_order;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
struct PayoutContext {
    order_id: String,
    settlement_id: String,
    seller_org_id: String,
    net_receivable_amount: String,
    net_receivable_amount_numeric: f64,
    currency_code: String,
    payment_status: String,
    settlement_status: String,
    provider_key: String,
    provider_account_id: Option<String>,
    provider_supports_payout: bool,
    payout_preference_id: Option<String>,
    beneficiary_subject_type: String,
    beneficiary_subject_id: String,
    destination_jurisdiction_code: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MockPayoutProviderResponse {
    status: String,
    provider_transfer_id: String,
    message: Option<String>,
}

pub async fn execute_manual_payout(
    client: &Client,
    payload: &CreateManualPayoutRequest,
    idempotency_key: &str,
    actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ManualPayoutExecutionView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;

    if let Some(existing) =
        find_existing_payout(&tx, &payload.settlement_id, idempotency_key, request_id).await?
    {
        write_audit_event(
            &tx,
            "billing",
            "payout",
            &existing.payout_instruction_id,
            actor_role,
            "billing.payout.execute_manual.idempotent_replay",
            "idempotent_replay",
            request_id,
            trace_id,
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(existing);
    }

    let context = load_payout_context(
        &tx,
        &payload.order_id,
        &payload.settlement_id,
        payload.payout_preference_id.as_deref(),
        request_id,
    )
    .await?;
    ensure_settlement_not_already_paid_out(&tx, &payload.settlement_id, request_id).await?;

    let amount = parse_positive_amount(
        &payload.amount,
        "manual payout amount must be a positive decimal string",
        request_id,
    )?;
    if amount > context.net_receivable_amount_numeric {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "manual payout amount exceeds settlement net receivable: {} > {}",
                payload.amount, context.net_receivable_amount
            ),
            request_id,
        ));
    }
    if context.payment_status != "paid" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "manual payout is not allowed from payment_status `{}`",
                context.payment_status
            ),
            request_id,
        ));
    }
    if matches!(context.settlement_status.as_str(), "closed" | "refunded") {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "manual payout is not allowed from settlement_status `{}`",
                context.settlement_status
            ),
            request_id,
        ));
    }

    let currency_code = payload
        .currency_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(context.currency_code.as_str())
        .to_string();
    if currency_code != context.currency_code {
        return Err(billing_error(
            StatusCode::BAD_REQUEST,
            &format!(
                "manual payout currency must match settlement currency: {}",
                context.currency_code
            ),
            request_id,
        ));
    }

    let provider_key = payload
        .provider_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(context.provider_key.as_str())
        .to_string();
    let provider_supports_payout = if provider_key == context.provider_key {
        context.provider_supports_payout
    } else {
        load_provider_supports_payout(&tx, &provider_key, request_id).await?
    };
    if !provider_supports_payout {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "payment provider does not support manual payout",
            request_id,
        ));
    }

    let provider_account_id = payload
        .provider_account_id
        .as_deref()
        .and_then(parse_uuid_text)
        .or_else(|| context.provider_account_id.clone());
    let payout_preference_id = payload
        .payout_preference_id
        .as_deref()
        .and_then(parse_uuid_text)
        .or_else(|| context.payout_preference_id.clone());
    let payout_mode = payload
        .payout_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("manual")
        .to_string();
    if payout_mode != "manual" {
        return Err(billing_error(
            StatusCode::BAD_REQUEST,
            "manual payout execute currently supports payout_mode `manual` only",
            request_id,
        ));
    }

    let provider_result =
        execute_provider_manual_payout(&provider_key, &payload.amount, &currency_code, request_id)
            .await?;
    let executed_by_param = actor_user_id.and_then(parse_uuid_text);
    let provider_message = provider_result.message.unwrap_or_default();
    let payout_metadata = json!({
        "provider_key": provider_key,
        "provider_status": provider_result.status,
        "provider_payout_no": provider_result.provider_transfer_id,
        "provider_message": provider_message,
        "step_up_bound": true,
        "idempotency_key": idempotency_key,
        "payout_mode": payout_mode,
        "settlement_direction": "payable",
        "request_metadata": payload.metadata,
    });

    let payout_row = tx
        .query_one(
            r#"INSERT INTO payment.payout_instruction (
               settlement_id,
               provider_key,
               provider_account_id,
               payout_preference_id,
               beneficiary_subject_type,
               beneficiary_subject_id,
               destination_jurisdiction_code,
               amount,
               currency_code,
               payout_mode,
               status,
               provider_payout_no,
               reviewed_by,
               executed_by,
               executed_at,
               idempotency_key,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3::text::uuid,
               $4::text::uuid,
               $5,
               $6::text::uuid,
               $7,
               $8::text::numeric,
               $9,
               $10,
               'succeeded',
               $11,
               $12::text::uuid,
               $12::text::uuid,
               now(),
               $13,
               $14::jsonb
             )
             RETURNING
               payout_instruction_id::text,
               settlement_id::text,
               provider_key,
               provider_account_id::text,
               payout_preference_id::text,
               beneficiary_subject_type,
               beneficiary_subject_id::text,
               destination_jurisdiction_code,
               amount::text,
               currency_code,
               payout_mode,
               status,
               provider_payout_no,
               to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &payload.settlement_id,
                &provider_key,
                &provider_account_id,
                &payout_preference_id,
                &context.beneficiary_subject_type,
                &context.beneficiary_subject_id,
                &context.destination_jurisdiction_code,
                &payload.amount,
                &currency_code,
                &payout_mode,
                &provider_result.provider_transfer_id,
                &executed_by_param,
                &idempotency_key,
                &payout_metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let payout_instruction_id: String = payout_row.get(0);
    let executed_at: Option<String> = payout_row.get(13);
    let updated_at: String = payout_row.get(14);

    let sub_merchant_binding_id = match provider_account_id.as_deref() {
        Some(provider_account_id) => Some(
            ensure_sub_merchant_binding(
                &tx,
                provider_account_id,
                &context.beneficiary_subject_type,
                &context.beneficiary_subject_id,
            )
            .await?,
        ),
        None => None,
    };
    let split_placeholder = upsert_split_placeholder(
        &tx,
        &payload.settlement_id,
        provider_account_id.as_deref(),
        sub_merchant_binding_id.as_deref(),
        &payload.amount,
        &currency_code,
    )
    .await?;

    release_provisional_dispute_hold_in_tx(
        &tx,
        &payload.order_id,
        "manual_payout_execute",
        &payout_instruction_id,
        actor_role,
        request_id,
        trace_id,
    )
    .await?;

    let billing_event_metadata = build_billing_event_metadata(
        &payout_metadata,
        &payout_instruction_id,
        &split_placeholder.split_instruction_id,
    );
    let event_row = tx
        .query_one(
            r#"INSERT INTO billing.billing_event (
               order_id,
               event_type,
               event_source,
               amount,
               currency_code,
               units,
               occurred_at,
               metadata
             ) VALUES (
               $1::text::uuid,
               'manual_settlement',
               'manual_payout_execute',
               $2::text::numeric,
               $3,
               NULL,
               now(),
               $4::jsonb
             )
             RETURNING
               billing_event_id::text,
               to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &payload.order_id,
                &payload.amount,
                &currency_code,
                &billing_event_metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let billing_event_id: String = event_row.get(0);
    let _billing_event_occurred_at: String = event_row.get(1);

    write_payout_outbox(
        &tx,
        &billing_event_id,
        &payload.order_id,
        &payload.amount,
        &currency_code,
        &payout_instruction_id,
        &split_placeholder.split_instruction_id,
        idempotency_key,
        &billing_event_metadata,
        request_id,
        trace_id,
    )
    .await?;

    let _ =
        recompute_settlement_for_order(&tx, &payload.order_id, actor_role, request_id, trace_id)
            .await?;
    let _ = tx
        .execute(
            "UPDATE trade.order_main
             SET updated_at = now(),
                 last_reason_code = 'billing_manual_payout_succeeded'
             WHERE order_id = $1::text::uuid",
            &[&payload.order_id],
        )
        .await
        .map_err(map_db_error)?;

    write_audit_event(
        &tx,
        "billing",
        "payout",
        &payout_instruction_id,
        actor_role,
        "billing.payout.execute_manual",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    write_audit_event(
        &tx,
        "billing",
        "billing_event",
        &billing_event_id,
        actor_role,
        "billing.event.generated",
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(ManualPayoutExecutionView {
        payout_instruction_id,
        order_id: context.order_id,
        settlement_id: context.settlement_id,
        beneficiary_subject_type: context.beneficiary_subject_type,
        beneficiary_subject_id: context.beneficiary_subject_id,
        destination_jurisdiction_code: context.destination_jurisdiction_code,
        amount: payload.amount.clone(),
        currency_code,
        payout_mode,
        current_status: "succeeded".to_string(),
        provider_key,
        provider_account_id,
        payout_preference_id,
        provider_payout_no: Some(provider_result.provider_transfer_id),
        step_up_bound: true,
        idempotent_replay: false,
        executed_at,
        updated_at,
        metadata: payout_metadata,
        split_placeholder,
    })
}

async fn find_existing_payout(
    client: &impl GenericClient,
    settlement_id: &str,
    idempotency_key: &str,
    request_id: Option<&str>,
) -> Result<Option<ManualPayoutExecutionView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"SELECT
               p.payout_instruction_id::text,
               s.order_id::text,
               p.settlement_id::text,
               p.beneficiary_subject_type,
               p.beneficiary_subject_id::text,
               p.destination_jurisdiction_code,
               p.amount::text,
               p.currency_code,
               p.payout_mode,
               p.status,
               p.provider_key,
               p.provider_account_id::text,
               p.payout_preference_id::text,
               p.provider_payout_no,
               p.metadata,
               to_char(p.executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
               to_char(p.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
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
               to_char(si.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
             FROM payment.payout_instruction p
             JOIN billing.settlement_record s ON s.settlement_id = p.settlement_id
             LEFT JOIN LATERAL (
               SELECT
                 split_instruction_id,
                 settlement_id,
                 reward_id,
                 provider_account_id,
                 sub_merchant_binding_id,
                 split_mode,
                 amount,
                 currency_code,
                 status,
                 provider_split_no,
                 updated_at
               FROM payment.split_instruction
               WHERE settlement_id = p.settlement_id
               ORDER BY updated_at DESC, split_instruction_id DESC
               LIMIT 1
             ) si ON true
             WHERE p.settlement_id = $1::text::uuid
               AND p.idempotency_key = $2
             ORDER BY p.updated_at DESC, p.payout_instruction_id DESC
             LIMIT 1"#,
            &[&settlement_id, &idempotency_key],
        )
        .await
        .map_err(map_db_error)?;

    row.map(|row| {
        let split_instruction_id: String = row.get(17);
        Ok(ManualPayoutExecutionView {
            payout_instruction_id: row.get(0),
            order_id: row.get(1),
            settlement_id: row.get(2),
            beneficiary_subject_type: row.get(3),
            beneficiary_subject_id: row.get(4),
            destination_jurisdiction_code: row.get(5),
            amount: row.get(6),
            currency_code: row.get(7),
            payout_mode: row.get(8),
            current_status: row.get(9),
            provider_key: row.get(10),
            provider_account_id: row.get(11),
            payout_preference_id: row.get(12),
            provider_payout_no: row.get(13),
            step_up_bound: row
                .get::<_, Value>(14)
                .get("step_up_bound")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            idempotent_replay: true,
            metadata: row.get(14),
            executed_at: row.get(15),
            updated_at: row.get(16),
            split_placeholder: BillingSplitInstructionView {
                split_instruction_id,
                settlement_id: row.get(18),
                reward_id: row.get(19),
                provider_account_id: row.get(20),
                sub_merchant_binding_id: row.get(21),
                split_mode: row.get(22),
                amount: row.get(23),
                currency_code: row.get(24),
                current_status: row.get(25),
                provider_split_no: row.get(26),
                updated_at: row.get(27),
            },
        })
    })
    .transpose()
}

async fn load_payout_context(
    client: &impl GenericClient,
    order_id: &str,
    settlement_id: &str,
    payout_preference_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<PayoutContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"WITH selected_preference AS (
               SELECT
                 payout_preference_id::text,
                 beneficiary_subject_type,
                 beneficiary_subject_id::text,
                 destination_jurisdiction_code,
                 preferred_provider_key,
                 preferred_provider_account_id::text
               FROM payment.payout_preference
               WHERE beneficiary_subject_type = 'organization'
                 AND beneficiary_subject_id = (
                   SELECT seller_org_id FROM trade.order_main WHERE order_id = $1::text::uuid
                 )
                 AND status = 'active'
                 AND (
                   $3::text IS NULL OR payout_preference_id = $3::text::uuid
                 )
               ORDER BY
                 CASE
                   WHEN $3::text IS NOT NULL AND payout_preference_id = $3::text::uuid THEN 0
                   WHEN is_default THEN 1
                   ELSE 2
                 END,
                 updated_at DESC,
                 payout_preference_id DESC
               LIMIT 1
             )
             SELECT
               o.order_id::text,
               s.settlement_id::text,
               o.seller_org_id::text,
               GREATEST(
                 s.payable_amount
                 - s.platform_fee_amount
                 - s.channel_fee_amount
                 - GREATEST(
                     s.refund_amount - COALESCE(provisional_hold.outstanding_amount, 0),
                     0
                   )
                 - s.compensation_amount,
                 0
               )::text,
               o.currency_code,
               o.payment_status,
               s.settlement_status,
               COALESCE(sp.preferred_provider_key, pi.provider_key, 'mock_payment') AS provider_key,
               COALESCE(sp.preferred_provider_account_id, pi.provider_account_id::text) AS provider_account_id,
               COALESCE(p.supports_payout, false),
               sp.payout_preference_id::text,
               COALESCE(sp.beneficiary_subject_type, 'organization'),
               COALESCE(sp.beneficiary_subject_id, o.seller_org_id::text),
               COALESCE(sp.destination_jurisdiction_code, pi.payee_jurisdiction_code, pi.launch_jurisdiction_code, 'SG')
             FROM billing.settlement_record s
             JOIN trade.order_main o ON o.order_id = s.order_id
             LEFT JOIN selected_preference sp ON true
             LEFT JOIN LATERAL (
               SELECT
                 provider_key,
                 provider_account_id,
                 payee_jurisdiction_code,
                 launch_jurisdiction_code
               FROM payment.payment_intent
               WHERE order_id = o.order_id
               ORDER BY updated_at DESC, payment_intent_id DESC
               LIMIT 1
             ) pi ON true
             LEFT JOIN LATERAL (
               SELECT COALESCE(SUM(amount), 0) AS outstanding_amount
               FROM billing.billing_event
               WHERE order_id = o.order_id
                 AND event_type = 'refund_adjustment'
                 AND COALESCE(metadata ->> 'adjustment_class', '') = 'provisional_dispute_hold'
             ) provisional_hold ON true
             LEFT JOIN payment.provider p
               ON p.provider_key = COALESCE(sp.preferred_provider_key, pi.provider_key, 'mock_payment')
             WHERE o.order_id = $1::text::uuid
               AND s.settlement_id = $2::text::uuid"#,
            &[&order_id, &settlement_id, &payout_preference_id],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            billing_error(
                StatusCode::NOT_FOUND,
                "manual payout order/settlement context not found",
                request_id,
            )
        })?;

    let net_receivable_amount: String = row.get(3);
    let net_receivable_amount_numeric = parse_positive_amount(
        &net_receivable_amount,
        "settlement net receivable must be a positive decimal string",
        request_id,
    )?;

    Ok(PayoutContext {
        order_id: row.get(0),
        settlement_id: row.get(1),
        seller_org_id: row.get(2),
        net_receivable_amount,
        net_receivable_amount_numeric,
        currency_code: row.get(4),
        payment_status: row.get(5),
        settlement_status: row.get(6),
        provider_key: row.get(7),
        provider_account_id: row.get(8),
        provider_supports_payout: row.get(9),
        payout_preference_id: row.get(10),
        beneficiary_subject_type: row.get(11),
        beneficiary_subject_id: row.get(12),
        destination_jurisdiction_code: row.get(13),
    })
}

async fn ensure_settlement_not_already_paid_out(
    client: &impl GenericClient,
    settlement_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let existing_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM payment.payout_instruction
             WHERE settlement_id = $1::text::uuid
               AND status = 'succeeded'",
            &[&settlement_id],
        )
        .await
        .map_err(map_db_error)?
        .get(0);
    if existing_count > 0 {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "manual payout already executed for settlement",
            request_id,
        ));
    }
    Ok(())
}

async fn load_provider_supports_payout(
    client: &impl GenericClient,
    provider_key: &str,
    request_id: Option<&str>,
) -> Result<bool, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT supports_payout FROM payment.provider WHERE provider_key = $1",
            &[&provider_key],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            billing_error(
                StatusCode::NOT_FOUND,
                &format!("payment provider not found: {provider_key}"),
                request_id,
            )
        })?;
    Ok(row.get(0))
}

async fn ensure_sub_merchant_binding(
    client: &impl GenericClient,
    provider_account_id: &str,
    beneficiary_type: &str,
    beneficiary_id: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    if let Some(existing) = client
        .query_opt(
            "SELECT sub_merchant_binding_id::text
             FROM payment.sub_merchant_binding
             WHERE provider_account_id = $1::text::uuid
               AND beneficiary_type = $2
               AND beneficiary_id = $3::text::uuid",
            &[&provider_account_id, &beneficiary_type, &beneficiary_id],
        )
        .await
        .map_err(map_db_error)?
    {
        return Ok(existing.get(0));
    }

    let row = client
        .query_one(
            "INSERT INTO payment.sub_merchant_binding (
               provider_account_id,
               beneficiary_type,
               beneficiary_id,
               external_sub_merchant_id,
               status,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3::text::uuid,
               $4,
               'active',
               jsonb_build_object('binding_mode', 'v1_placeholder', 'source', 'manual_payout')
             )
             RETURNING sub_merchant_binding_id::text",
            &[
                &provider_account_id,
                &beneficiary_type,
                &beneficiary_id,
                &format!("v1-placeholder:{beneficiary_id}"),
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(row.get(0))
}

async fn upsert_split_placeholder(
    client: &impl GenericClient,
    settlement_id: &str,
    provider_account_id: Option<&str>,
    sub_merchant_binding_id: Option<&str>,
    amount: &str,
    currency_code: &str,
) -> Result<BillingSplitInstructionView, (StatusCode, Json<ErrorResponse>)> {
    if let Some(row) = client
        .query_opt(
            r#"UPDATE payment.split_instruction
               SET provider_account_id = COALESCE($2::text::uuid, provider_account_id),
                   sub_merchant_binding_id = COALESCE($3::text::uuid, sub_merchant_binding_id),
                   split_mode = 'platform_ledger_then_payout',
                   amount = $4::text::numeric,
                   currency_code = $5,
                   status = 'succeeded',
                   updated_at = now()
             WHERE split_instruction_id = (
               SELECT split_instruction_id
               FROM payment.split_instruction
               WHERE settlement_id = $1::text::uuid
               ORDER BY updated_at DESC, split_instruction_id DESC
               LIMIT 1
             )
             RETURNING
               split_instruction_id::text,
               settlement_id::text,
               reward_id::text,
               provider_account_id::text,
               sub_merchant_binding_id::text,
               split_mode,
               amount::text,
               currency_code,
               status,
               provider_split_no,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &settlement_id,
                &provider_account_id,
                &sub_merchant_binding_id,
                &amount,
                &currency_code,
            ],
        )
        .await
        .map_err(map_db_error)?
    {
        return Ok(parse_split_row(&row));
    }

    let row = client
        .query_one(
            r#"INSERT INTO payment.split_instruction (
               settlement_id,
               provider_account_id,
               sub_merchant_binding_id,
               split_mode,
               amount,
               currency_code,
               status,
               provider_split_no
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               'platform_ledger_then_payout',
               $4::text::numeric,
               $5,
               'succeeded',
               NULL
             )
             RETURNING
               split_instruction_id::text,
               settlement_id::text,
               reward_id::text,
               provider_account_id::text,
               sub_merchant_binding_id::text,
               split_mode,
               amount::text,
               currency_code,
               status,
               provider_split_no,
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &settlement_id,
                &provider_account_id,
                &sub_merchant_binding_id,
                &amount,
                &currency_code,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(parse_split_row(&row))
}

fn parse_split_row(row: &db::Row) -> BillingSplitInstructionView {
    BillingSplitInstructionView {
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
    }
}

fn parse_positive_amount(
    raw: &str,
    error_message: &str,
    request_id: Option<&str>,
) -> Result<f64, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim().parse::<f64>() {
        Ok(value) if value > 0.0 => Ok(value),
        _ => Err(billing_error(
            StatusCode::BAD_REQUEST,
            error_message,
            request_id,
        )),
    }
}

fn parse_uuid_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() == 36 {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn build_billing_event_metadata(
    metadata: &Value,
    payout_instruction_id: &str,
    split_instruction_id: &str,
) -> Value {
    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }
    if let Some(obj) = metadata.as_object_mut() {
        obj.insert(
            "payout_instruction_id".to_string(),
            Value::String(payout_instruction_id.to_string()),
        );
        obj.insert(
            "split_instruction_id".to_string(),
            Value::String(split_instruction_id.to_string()),
        );
        obj.insert("split_placeholder".to_string(), Value::Bool(true));
        obj.insert(
            "model_name".to_string(),
            Value::String("BillingEvent".to_string()),
        );
        obj.insert(
            "event_type_canonical".to_string(),
            Value::String("manual_settlement".to_string()),
        );
    }
    metadata
}

async fn write_payout_outbox(
    client: &impl GenericClient,
    billing_event_id: &str,
    order_id: &str,
    amount: &str,
    currency_code: &str,
    payout_instruction_id: &str,
    split_instruction_id: &str,
    idempotency_key: &str,
    metadata: &Value,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let payload = json!({
        "event_name": "billing.event.recorded",
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "pending_fabric_anchor",
        "billing_event_id": billing_event_id,
        "order_id": order_id,
        "event_type": "manual_settlement",
        "event_source": "manual_payout_execute",
        "amount": amount,
        "currency_code": currency_code,
        "payout_instruction_id": payout_instruction_id,
        "split_instruction_id": split_instruction_id,
        "metadata": metadata,
    });
    let request_id = request_id.map(str::to_string);
    let trace_id = trace_id.map(str::to_string);
    client
        .query_one(
            "INSERT INTO ops.outbox_event (
               aggregate_type,
               aggregate_id,
               event_type,
               payload,
               status,
               request_id,
               trace_id,
               idempotency_key,
               event_schema_version,
               authority_scope,
               source_of_truth,
               proof_commit_policy,
               target_bus,
               target_topic,
               partition_key,
               ordering_key,
               payload_hash
             ) VALUES (
               'payment.payout_instruction',
               $1::text::uuid,
               'billing.event.recorded',
               $2::jsonb,
               'pending',
               $3,
               $4,
               $5,
               'v1',
               'business',
               'database',
               'pending_fabric_anchor',
               'kafka',
               'billing.events',
               $6,
               $6,
               encode(digest(($2::jsonb)::text, 'sha256'), 'hex')
             )
             ON CONFLICT DO NOTHING
             RETURNING outbox_event_id::text",
            &[
                &payout_instruction_id,
                &payload,
                &request_id,
                &trace_id,
                &idempotency_key,
                &order_id,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

async fn execute_provider_manual_payout(
    provider_key: &str,
    amount: &str,
    currency_code: &str,
    request_id: Option<&str>,
) -> Result<MockPayoutProviderResponse, (StatusCode, Json<ErrorResponse>)> {
    if provider_key != "mock_payment" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "manual payout execution currently supports provider `mock_payment` only",
            request_id,
        ));
    }
    let mode = std::env::var("MOCK_PAYMENT_ADAPTER_MODE")
        .unwrap_or_else(|_| "stub".to_string())
        .to_ascii_lowercase();
    if mode != "live" {
        return Ok(MockPayoutProviderResponse {
            status: "MANUAL_TRANSFER_SUCCESS".to_string(),
            provider_transfer_id: format!(
                "mock-mtf-stub-{}",
                kernel::new_external_readable_id("payout")
            ),
            message: Some("Manual transfer success (stub)".to_string()),
        });
    }
    let base_url = std::env::var("MOCK_PAYMENT_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8089".to_string());
    let response = reqwest::Client::new()
        .post(format!(
            "{}/mock/payment/manual-transfer/success",
            base_url.trim_end_matches('/')
        ))
        .json(&json!({
            "transfer_amount": amount,
            "currency": currency_code,
        }))
        .send()
        .await
        .map_err(|err| {
            billing_error(
                StatusCode::BAD_GATEWAY,
                &format!("mock payout provider call failed: {err}"),
                request_id,
            )
        })?;
    if !response.status().is_success() {
        return Err(billing_error(
            StatusCode::BAD_GATEWAY,
            &format!("mock payout provider returned HTTP {}", response.status()),
            request_id,
        ));
    }
    response
        .json::<MockPayoutProviderResponse>()
        .await
        .map_err(|err| {
            billing_error(
                StatusCode::BAD_GATEWAY,
                &format!("mock payout provider payload parse failed: {err}"),
                request_id,
            )
        })
}

fn billing_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: if status == StatusCode::FORBIDDEN {
                ErrorCode::IamUnauthorized.as_str().to_string()
            } else {
                ErrorCode::BilProviderFailed.as_str().to_string()
            },
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
