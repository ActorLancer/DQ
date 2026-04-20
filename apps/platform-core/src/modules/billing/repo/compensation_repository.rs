use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::models::{CompensationExecutionView, CreateCompensationRequest};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
struct CompensationContext {
    order_id: String,
    buyer_org_id: String,
    seller_org_id: String,
    order_amount: String,
    order_amount_numeric: f64,
    currency_code: String,
    payment_status: String,
    settlement_status: String,
    dispute_status: String,
    compensation_mode: String,
    compensation_template: Option<String>,
    provider_key: Option<String>,
    provider_supports_payout: bool,
}

#[derive(Debug, Clone)]
struct DisputeDecisionContext {
    case_id: String,
    order_id: String,
    status: String,
    decision_code: String,
    penalty_code: Option<String>,
    decision_id: String,
    liability_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MockCompensationProviderResponse {
    status: String,
    provider_transfer_id: String,
    message: Option<String>,
}

pub async fn execute_compensation(
    client: &Client,
    payload: &CreateCompensationRequest,
    idempotency_key: &str,
    tenant_scope_id: Option<&str>,
    actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<CompensationExecutionView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;

    if let Some(existing) =
        find_existing_compensation(&tx, &payload.order_id, idempotency_key, request_id).await?
    {
        write_audit_event(
            &tx,
            "billing",
            "compensation",
            &existing.compensation_id,
            actor_role,
            "billing.compensation.execute.idempotent_replay",
            "idempotent_replay",
            request_id,
            trace_id,
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(existing);
    }

    let context = load_compensation_context(&tx, &payload.order_id, request_id).await?;
    enforce_compensation_scope(tenant_scope_id, &context, request_id)?;
    let amount = parse_positive_amount(
        &payload.amount,
        "compensation amount must be a positive decimal string",
        request_id,
    )?;
    if amount > context.order_amount_numeric {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "compensation amount exceeds order amount: {} > {}",
                payload.amount, context.order_amount
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
                "compensation currency must match order currency: {}",
                context.currency_code
            ),
            request_id,
        ));
    }
    if !context.provider_supports_payout {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "payment provider does not support compensation payout",
            request_id,
        ));
    }
    if !matches!(
        context.payment_status.as_str(),
        "paid" | "buyer_locked" | "refunded"
    ) {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!(
                "compensation is not allowed from payment_status `{}`",
                context.payment_status
            ),
            request_id,
        ));
    }
    let dispute =
        load_dispute_decision(&tx, &payload.case_id, &payload.order_id, request_id).await?;
    if dispute.decision_code != payload.decision_code.trim() {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "payload decision_code does not match stored dispute decision",
            request_id,
        ));
    }

    let provider_key = context
        .provider_key
        .clone()
        .unwrap_or_else(|| "mock_payment".to_string());
    let provider_result =
        execute_provider_compensation(&provider_key, &payload.amount, &currency_code, request_id)
            .await?;

    let executed_by_param = actor_user_id.and_then(|value| parse_uuid_text(value));
    let penalty_code = payload
        .penalty_code
        .clone()
        .or(dispute.penalty_code.clone());
    let provider_message = provider_result.message.unwrap_or_default();
    let compensation_metadata = json!({
        "case_id": payload.case_id,
        "decision_id": dispute.decision_id,
        "decision_code": dispute.decision_code,
        "penalty_code": penalty_code,
        "liability_type": dispute.liability_type,
        "reason_code": payload.reason_code,
        "compensation_mode": payload.compensation_mode.as_deref().unwrap_or(context.compensation_mode.as_str()),
        "compensation_template": payload.compensation_template.as_deref().or(context.compensation_template.as_deref()),
        "provider_key": provider_key,
        "provider_status": provider_result.status,
        "provider_transfer_id": provider_result.provider_transfer_id,
        "provider_message": provider_message,
        "step_up_bound": true,
        "idempotency_key": idempotency_key,
        "request_metadata": payload.metadata,
    });
    let row = tx
        .query_one(
            r#"INSERT INTO billing.compensation_record (
               order_id,
               amount,
               currency_code,
               status,
               executed_by,
               executed_at
             ) VALUES (
               $1::text::uuid,
               $2::text::numeric,
               $3,
               'succeeded',
               $4::text::uuid,
               now()
             )
             RETURNING
               compensation_id::text,
               order_id::text,
               amount::text,
               currency_code,
               status,
               to_char(executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &payload.order_id,
                &payload.amount,
                &currency_code,
                &executed_by_param,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let compensation_id: String = row.get(0);

    let billing_event_metadata =
        build_billing_event_metadata(&compensation_metadata, &compensation_id);
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
               'compensation',
               'compensation_execute',
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
    let billing_event_occurred_at: String = event_row.get(1);

    write_compensation_outbox(
        &tx,
        &billing_event_id,
        &payload.order_id,
        &payload.amount,
        &currency_code,
        &compensation_id,
        idempotency_key,
        &billing_event_metadata,
        request_id,
        trace_id,
    )
    .await?;

    let _ = tx
        .execute(
            "UPDATE billing.settlement_record
             SET compensation_amount = COALESCE(compensation_amount, 0) + $2::text::numeric,
                 updated_at = now()
             WHERE settlement_id = (
               SELECT settlement_id
               FROM billing.settlement_record
               WHERE order_id = $1::text::uuid
               ORDER BY created_at DESC, settlement_id DESC
               LIMIT 1
             )",
            &[&payload.order_id, &payload.amount],
        )
        .await
        .map_err(map_db_error)?;

    let _ = tx
        .execute(
            "UPDATE trade.order_main
             SET dispute_status = 'resolved',
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[&payload.order_id],
        )
        .await
        .map_err(map_db_error)?;

    let _ = tx
        .execute(
            "UPDATE support.dispute_case
             SET status = 'resolved',
                 decision_code = $2,
                 penalty_code = COALESCE($3, penalty_code),
                 resolved_at = COALESCE(resolved_at, now()),
                 updated_at = now()
             WHERE case_id = $1::text::uuid",
            &[&payload.case_id, &payload.decision_code, &penalty_code],
        )
        .await
        .map_err(map_db_error)?;

    write_audit_event(
        &tx,
        "billing",
        "compensation",
        &compensation_id,
        actor_role,
        "billing.compensation.execute",
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

    Ok(CompensationExecutionView {
        compensation_id,
        order_id: payload.order_id.clone(),
        case_id: payload.case_id.clone(),
        decision_code: payload.decision_code.clone(),
        penalty_code,
        amount: payload.amount.clone(),
        currency_code,
        current_status: "succeeded".to_string(),
        provider_key,
        provider_transfer_id: Some(provider_result.provider_transfer_id),
        provider_status: Some(provider_result.status),
        step_up_bound: true,
        idempotent_replay: false,
        executed_at: Some(row.get(5)),
        updated_at: row.get(6),
        metadata: json!({
            "compensation_mode": payload.compensation_mode.as_deref().unwrap_or(context.compensation_mode.as_str()),
            "compensation_template": payload.compensation_template.as_deref().or(context.compensation_template.as_deref()),
            "reason_code": payload.reason_code,
            "billing_event_id": billing_event_id,
            "billing_event_occurred_at": billing_event_occurred_at,
            "provider_message": provider_message,
        }),
    })
}

async fn find_existing_compensation(
    client: &impl GenericClient,
    order_id: &str,
    idempotency_key: &str,
    _request_id: Option<&str>,
) -> Result<Option<CompensationExecutionView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"SELECT
               r.compensation_id::text,
               r.order_id::text,
               COALESCE(be.metadata ->> 'case_id', ''),
               COALESCE(be.metadata ->> 'decision_code', ''),
               NULLIF(be.metadata ->> 'penalty_code', ''),
               r.amount::text,
               r.currency_code,
               r.status,
               COALESCE(be.metadata ->> 'provider_key', 'mock_payment'),
               NULLIF(be.metadata ->> 'provider_transfer_id', ''),
               NULLIF(be.metadata ->> 'provider_status', ''),
               to_char(r.executed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
               to_char(r.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
               COALESCE(be.metadata, '{}'::jsonb)
             FROM billing.compensation_record r
             JOIN billing.billing_event be
               ON COALESCE(be.metadata ->> 'compensation_id', '') = r.compensation_id::text
             WHERE r.order_id = $1::text::uuid
               AND be.event_type = 'compensation'
               AND COALESCE(be.metadata ->> 'idempotency_key', '') = $2
             ORDER BY r.updated_at DESC, r.compensation_id DESC
             LIMIT 1"#,
            &[&order_id, &idempotency_key],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(|row| CompensationExecutionView {
        compensation_id: row.get(0),
        order_id: row.get(1),
        case_id: row.get(2),
        decision_code: row.get(3),
        penalty_code: row.get(4),
        amount: row.get(5),
        currency_code: row.get(6),
        current_status: row.get(7),
        provider_key: row.get(8),
        provider_transfer_id: row.get(9),
        provider_status: row.get(10),
        step_up_bound: true,
        idempotent_replay: true,
        executed_at: row.get(11),
        updated_at: row.get(12),
        metadata: row.get(13),
    }))
}

async fn load_compensation_context(
    client: &impl GenericClient,
    order_id: &str,
    request_id: Option<&str>,
) -> Result<CompensationContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               o.amount::text,
               o.currency_code,
               o.payment_status,
               o.settlement_status,
               o.dispute_status,
               COALESCE(o.price_snapshot_json ->> 'compensation_mode', 'manual_transfer'),
               COALESCE(o.price_snapshot_json ->> 'compensation_template', ''),
               pi.provider_key,
               COALESCE(p.supports_payout, false)
             FROM trade.order_main o
             LEFT JOIN LATERAL (
               SELECT provider_key
               FROM payment.payment_intent
               WHERE order_id = o.order_id
               ORDER BY created_at DESC, payment_intent_id DESC
               LIMIT 1
             ) pi ON true
             LEFT JOIN payment.provider p ON p.provider_key = pi.provider_key
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_error(
            StatusCode::NOT_FOUND,
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };
    let order_amount: String = row.get(3);
    Ok(CompensationContext {
        order_id: row.get(0),
        buyer_org_id: row.get(1),
        seller_org_id: row.get(2),
        order_amount_numeric: order_amount.parse::<f64>().map_err(|_| {
            billing_error(
                StatusCode::BAD_REQUEST,
                "order amount is not a valid decimal",
                request_id,
            )
        })?,
        order_amount,
        currency_code: row.get(4),
        payment_status: row.get(5),
        settlement_status: row.get(6),
        dispute_status: row.get(7),
        compensation_mode: row.get(8),
        compensation_template: match row.get::<_, String>(9) {
            v if v.is_empty() => None,
            v => Some(v),
        },
        provider_key: row.get(10),
        provider_supports_payout: row.get(11),
    })
}

fn enforce_compensation_scope(
    tenant_scope_id: Option<&str>,
    context: &CompensationContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(tenant_scope_id) = tenant_scope_id else {
        return Ok(());
    };
    if tenant_scope_id == context.buyer_org_id || tenant_scope_id == context.seller_org_id {
        return Ok(());
    }
    Err(billing_error(
        StatusCode::FORBIDDEN,
        "tenant scope does not match compensation order",
        request_id,
    ))
}

async fn load_dispute_decision(
    client: &impl GenericClient,
    case_id: &str,
    order_id: &str,
    request_id: Option<&str>,
) -> Result<DisputeDecisionContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               c.case_id::text,
               c.order_id::text,
               c.status,
               c.decision_code,
               c.penalty_code,
               d.decision_id::text,
               d.liability_type
             FROM support.dispute_case c
             JOIN support.decision_record d ON d.case_id = c.case_id
             WHERE c.case_id = $1::text::uuid",
            &[&case_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(billing_error(
            StatusCode::NOT_FOUND,
            &format!("dispute decision not found for case: {case_id}"),
            request_id,
        ));
    };
    let record = DisputeDecisionContext {
        case_id: row.get(0),
        order_id: row.get(1),
        status: row.get(2),
        decision_code: row.get(3),
        penalty_code: row.get(4),
        decision_id: row.get(5),
        liability_type: row.get(6),
    };
    if record.order_id != order_id {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "dispute case does not belong to compensation order",
            request_id,
        ));
    }
    if record.decision_code.trim().is_empty() {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "dispute case is missing decision_code",
            request_id,
        ));
    }
    Ok(record)
}

fn parse_positive_amount(
    raw: &str,
    message: &str,
    request_id: Option<&str>,
) -> Result<f64, (StatusCode, Json<ErrorResponse>)> {
    match raw.trim().parse::<f64>() {
        Ok(value) if value > 0.0 => Ok(value),
        _ => Err(billing_error(StatusCode::BAD_REQUEST, message, request_id)),
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

fn build_billing_event_metadata(metadata: &Value, compensation_id: &str) -> Value {
    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }
    if let Some(obj) = metadata.as_object_mut() {
        obj.insert(
            "compensation_id".to_string(),
            Value::String(compensation_id.to_string()),
        );
        obj.insert(
            "model_name".to_string(),
            Value::String("BillingEvent".to_string()),
        );
        obj.insert(
            "event_type_canonical".to_string(),
            Value::String("compensation".to_string()),
        );
    }
    metadata
}

async fn write_compensation_outbox(
    client: &impl GenericClient,
    billing_event_id: &str,
    order_id: &str,
    amount: &str,
    currency_code: &str,
    compensation_id: &str,
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
        "event_type": "compensation",
        "event_source": "compensation_execute",
        "amount": amount,
        "currency_code": currency_code,
        "compensation_id": compensation_id,
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
               'billing.compensation_record',
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
                &compensation_id,
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

async fn execute_provider_compensation(
    provider_key: &str,
    amount: &str,
    currency_code: &str,
    request_id: Option<&str>,
) -> Result<MockCompensationProviderResponse, (StatusCode, Json<ErrorResponse>)> {
    if provider_key != "mock_payment" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            "compensation execution currently supports provider `mock_payment` only",
            request_id,
        ));
    }
    let mode = std::env::var("MOCK_PAYMENT_ADAPTER_MODE")
        .unwrap_or_else(|_| "stub".to_string())
        .to_ascii_lowercase();
    if mode != "live" {
        return Ok(MockCompensationProviderResponse {
            status: "MANUAL_TRANSFER_SUCCESS".to_string(),
            provider_transfer_id: format!(
                "mock-mtf-stub-{}",
                kernel::new_external_readable_id("mtf")
            ),
            message: Some("Compensation success (stub)".to_string()),
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
                &format!("mock compensation provider call failed: {err}"),
                request_id,
            )
        })?;
    if !response.status().is_success() {
        return Err(billing_error(
            StatusCode::BAD_GATEWAY,
            &format!(
                "mock compensation provider returned HTTP {}",
                response.status()
            ),
            request_id,
        ));
    }
    response
        .json::<MockCompensationProviderResponse>()
        .await
        .map_err(|err| {
            billing_error(
                StatusCode::BAD_GATEWAY,
                &format!("mock compensation provider payload parse failed: {err}"),
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
