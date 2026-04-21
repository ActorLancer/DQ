use crate::modules::order::repo::map_db_error;
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Error as DbError, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

pub(crate) const DELIVERY_RECEIPT_EVENT_TYPE: &str = "delivery.committed";
const DELIVERY_RECEIPT_AGGREGATE_TYPE: &str = "delivery.delivery_record";
pub(crate) const BILLING_TRIGGER_BRIDGE_EVENT_TYPE: &str = "billing.trigger.bridge";
const BILLING_TRIGGER_BRIDGE_AGGREGATE_TYPE: &str = "trade.order_main";

#[derive(Clone, Copy)]
struct BillingTriggerMatrixSnapshot {
    payment_trigger: &'static str,
    delivery_trigger: &'static str,
    acceptance_trigger: &'static str,
    billing_trigger: &'static str,
    settlement_cycle: &'static str,
    refund_entry: &'static str,
    compensation_entry: &'static str,
    dispute_freeze_trigger: &'static str,
    resume_settlement_trigger: &'static str,
    metadata: &'static str,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_delivery_receipt_outbox_payload(
    delivery_branch: &str,
    order_id: &str,
    delivery_id: &str,
    sku_type: &str,
    actor_role: &str,
    buyer_org_id: &str,
    seller_org_id: &str,
    current_state: &str,
    payment_status: &str,
    delivery_status: &str,
    acceptance_status: &str,
    settlement_status: &str,
    dispute_status: &str,
    receipt_hash: Option<&str>,
    delivery_commit_hash: Option<&str>,
    delivery_type: Option<&str>,
    delivery_route: Option<&str>,
    committed_at: Option<&str>,
    extra: Value,
) -> Value {
    let mut payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "async_evidence",
        "delivery_branch": delivery_branch,
        "order_id": order_id,
        "delivery_id": delivery_id,
        "sku_type": sku_type,
        "actor_role": actor_role,
        "buyer_org_id": buyer_org_id,
        "seller_org_id": seller_org_id,
        "current_state": current_state,
        "payment_status": payment_status,
        "delivery_status": delivery_status,
        "acceptance_status": acceptance_status,
        "settlement_status": settlement_status,
        "dispute_status": dispute_status,
        "receipt_hash": receipt_hash,
        "delivery_commit_hash": delivery_commit_hash,
        "delivery_type": delivery_type,
        "delivery_route": delivery_route,
        "committed_at": committed_at,
    });

    let object = payload
        .as_object_mut()
        .expect("delivery receipt payload must be object");
    if let Some(extra_object) = extra.as_object() {
        merge_object(object, extra_object);
    } else if !extra.is_null() {
        object.insert("details".to_string(), extra);
    }
    payload
}

pub(crate) async fn write_delivery_receipt_outbox_event(
    client: &(impl GenericClient + Sync),
    delivery_id: &str,
    payload: &Value,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: DELIVERY_RECEIPT_AGGREGATE_TYPE,
            aggregate_id: delivery_id,
            event_type: DELIVERY_RECEIPT_EVENT_TYPE,
            producer_service: "platform-core.delivery",
            request_id,
            trace_id,
            idempotency_key,
            occurred_at: payload.get("committed_at").and_then(Value::as_str),
            business_payload: payload,
            deduplicate_by_idempotency_key: false,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn write_billing_trigger_bridge_event(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    trigger_stage: &str,
    trigger_ref_type: &str,
    trigger_ref_id: &str,
    trigger_action: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: &str,
    extra: Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               s.sku_type,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               o.amount::text,
               o.currency_code,
               o.price_snapshot_json
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Ok(());
    };

    let price_snapshot: Value = row.get(11);
    let sku_type = row.get::<_, String>(0);
    let billing_trigger_matrix = load_billing_trigger_matrix(client, sku_type.as_str()).await?;
    let mut payload = json!({
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "async_evidence",
        "order_id": order_id,
        "sku_type": sku_type,
        "buyer_org_id": row.get::<_, String>(1),
        "seller_org_id": row.get::<_, String>(2),
        "current_state": row.get::<_, String>(3),
        "payment_status": row.get::<_, String>(4),
        "delivery_status": row.get::<_, String>(5),
        "acceptance_status": row.get::<_, String>(6),
        "settlement_status": row.get::<_, String>(7),
        "dispute_status": row.get::<_, String>(8),
        "amount": row.get::<_, String>(9),
        "currency_code": row.get::<_, String>(10),
        "pricing_mode": price_snapshot.get("pricing_mode").cloned(),
        "billing_mode": price_snapshot.get("billing_mode").cloned(),
        "refund_mode": price_snapshot.get("refund_mode").cloned(),
        "scenario_snapshot": price_snapshot.get("scenario_snapshot").cloned(),
        "trigger_stage": trigger_stage,
        "trigger_ref_type": trigger_ref_type,
        "trigger_ref_id": trigger_ref_id,
        "trigger_action": trigger_action,
        "actor_role": actor_role,
        "billing_trigger_matrix": billing_trigger_matrix,
    });
    let object = payload
        .as_object_mut()
        .expect("billing trigger bridge payload must be object");
    if let Some(extra_object) = extra.as_object() {
        merge_object(object, extra_object);
    } else if !extra.is_null() {
        object.insert("details".to_string(), extra);
    }

    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: BILLING_TRIGGER_BRIDGE_AGGREGATE_TYPE,
            aggregate_id: order_id,
            event_type: BILLING_TRIGGER_BRIDGE_EVENT_TYPE,
            producer_service: "platform-core.delivery",
            request_id,
            trace_id,
            idempotency_key: Some(idempotency_key),
            occurred_at: None,
            business_payload: &payload,
            deduplicate_by_idempotency_key: true,
        },
    )
    .await
    .map_err(map_db_error)?;
    Ok(())
}

fn merge_object(target: &mut Map<String, Value>, extra: &Map<String, Value>) {
    for (key, value) in extra {
        target.insert(key.clone(), value.clone());
    }
}

async fn load_billing_trigger_matrix(
    client: &(impl GenericClient + Sync),
    sku_type: &str,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let matrix_relation_exists = client
        .query_one(
            "SELECT to_regclass('billing.sku_billing_trigger_matrix') IS NOT NULL",
            &[],
        )
        .await
        .map_err(map_db_error)?
        .get::<_, bool>(0);
    if !matrix_relation_exists {
        return Ok(billing_trigger_matrix_fallback(sku_type));
    }

    match client
        .query_opt(
            "SELECT
               payment_trigger,
               delivery_trigger,
               acceptance_trigger,
               billing_trigger,
               settlement_cycle,
               refund_entry,
               compensation_entry,
               dispute_freeze_trigger,
               resume_settlement_trigger,
               metadata
             FROM billing.sku_billing_trigger_matrix
             WHERE sku_code = $1",
            &[&sku_type],
        )
        .await
    {
        Ok(Some(row)) => Ok(json!({
            "payment_trigger": row.get::<_, String>(0),
            "delivery_trigger": row.get::<_, String>(1),
            "acceptance_trigger": row.get::<_, String>(2),
            "billing_trigger": row.get::<_, String>(3),
            "settlement_cycle": row.get::<_, String>(4),
            "refund_entry": row.get::<_, String>(5),
            "compensation_entry": row.get::<_, String>(6),
            "dispute_freeze_trigger": row.get::<_, String>(7),
            "resume_settlement_trigger": row.get::<_, String>(8),
            "metadata": row.get::<_, Value>(9),
        })),
        Ok(None) => Ok(billing_trigger_matrix_fallback(sku_type)),
        Err(err) if is_missing_billing_trigger_matrix_relation(&err) => {
            Ok(billing_trigger_matrix_fallback(sku_type))
        }
        Err(err) => Err(map_db_error(err)),
    }
}

fn is_missing_billing_trigger_matrix_relation(err: &DbError) -> bool {
    match err {
        DbError::Sqlx(sqlx::Error::Database(db_err)) => db_err.code().as_deref() == Some("42P01"),
        _ => false,
    }
}

fn billing_trigger_matrix_fallback(sku_type: &str) -> Value {
    let snapshot = billing_trigger_matrix_snapshot(sku_type);
    json!({
        "payment_trigger": snapshot.payment_trigger,
        "delivery_trigger": snapshot.delivery_trigger,
        "acceptance_trigger": snapshot.acceptance_trigger,
        "billing_trigger": snapshot.billing_trigger,
        "settlement_cycle": snapshot.settlement_cycle,
        "refund_entry": snapshot.refund_entry,
        "compensation_entry": snapshot.compensation_entry,
        "dispute_freeze_trigger": snapshot.dispute_freeze_trigger,
        "resume_settlement_trigger": snapshot.resume_settlement_trigger,
        "metadata": serde_json::from_str::<Value>(snapshot.metadata)
            .expect("frozen billing trigger matrix metadata must be valid json"),
    })
}

fn billing_trigger_matrix_snapshot(sku_type: &str) -> BillingTriggerMatrixSnapshot {
    match sku_type {
        "FILE_STD" => BillingTriggerMatrixSnapshot {
            payment_trigger: "order_contract_effective_lock_once",
            delivery_trigger: "seller_publish_single_package",
            acceptance_trigger: "buyer_manual_accept_or_timeout",
            billing_trigger: "bill_once_after_acceptance",
            settlement_cycle: "t_plus_1_once",
            refund_entry: "pre_acceptance_cancel_or_acceptance_failed",
            compensation_entry: "delivery_defect_or_delay",
            dispute_freeze_trigger: "freeze_on_dispute_opened",
            resume_settlement_trigger: "resume_on_dispute_closed_with_ruling",
            metadata: r#"{"seed":"db034","sku":"FILE_STD","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "FILE_SUB" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_before_each_subscription_cycle",
            delivery_trigger: "generate_delivery_batch_each_cycle",
            acceptance_trigger: "cycle_window_manual_acceptance",
            billing_trigger: "bill_per_cycle_after_acceptance",
            settlement_cycle: "monthly_cycle",
            refund_entry: "refund_current_cycle_if_not_delivered",
            compensation_entry: "compensate_on_repeated_missing_delivery",
            dispute_freeze_trigger: "freeze_future_cycles_on_dispute_opened",
            resume_settlement_trigger: "resume_after_dispute_closed_for_cycle",
            metadata: r#"{"seed":"db034","sku":"FILE_SUB","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "SHARE_RO" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_before_share_grant_activation",
            delivery_trigger: "readonly_share_grant_enabled",
            acceptance_trigger: "accessibility_check_passed",
            billing_trigger: "bill_once_on_grant_effective",
            settlement_cycle: "t_plus_1_once",
            refund_entry: "refund_if_grant_not_effective",
            compensation_entry: "compensate_on_scope_or_access_violation",
            dispute_freeze_trigger: "freeze_on_share_dispute_opened",
            resume_settlement_trigger: "resume_on_dispute_closed_after_fix",
            metadata: r#"{"seed":"db034","sku":"SHARE_RO","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "API_SUB" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_before_subscription_cycle_start",
            delivery_trigger: "api_key_and_quota_provisioned",
            acceptance_trigger: "first_success_call_or_cycle_acceptance",
            billing_trigger: "bill_cycle_after_enable_and_acceptance",
            settlement_cycle: "monthly_cycle",
            refund_entry: "refund_current_cycle_if_unavailable",
            compensation_entry: "compensate_on_sla_breach",
            dispute_freeze_trigger: "freeze_current_cycle_on_sla_dispute",
            resume_settlement_trigger: "resume_on_sla_dispute_closed",
            metadata: r#"{"seed":"db034","sku":"API_SUB","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "API_PPU" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_prepaid_quota_or_minimum_commit",
            delivery_trigger: "api_key_enabled_for_metering",
            acceptance_trigger: "usage_reconciliation_window_confirmed",
            billing_trigger: "bill_by_metered_usage",
            settlement_cycle: "daily_with_monthly_statement",
            refund_entry: "refund_failed_batch_or_unused_quota",
            compensation_entry: "compensate_on_metering_or_throttling_fault",
            dispute_freeze_trigger: "freeze_metered_settlement_on_dispute",
            resume_settlement_trigger: "resume_after_metering_reconcile",
            metadata: r#"{"seed":"db034","sku":"API_PPU","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "QRY_LITE" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_before_query_job_execution",
            delivery_trigger: "query_job_succeeded_result_available",
            acceptance_trigger: "result_integrity_and_download_check",
            billing_trigger: "bill_once_after_task_acceptance",
            settlement_cycle: "t_plus_1_once",
            refund_entry: "refund_if_task_failed_or_unavailable",
            compensation_entry: "compensate_on_execution_unavailability",
            dispute_freeze_trigger: "freeze_on_query_result_dispute",
            resume_settlement_trigger: "resume_after_result_recheck_closed",
            metadata: r#"{"seed":"db034","sku":"QRY_LITE","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "SBX_STD" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_before_workspace_provision",
            delivery_trigger: "workspace_account_quota_ready",
            acceptance_trigger: "login_and_probe_check_passed",
            billing_trigger: "bill_after_workspace_activation_acceptance",
            settlement_cycle: "monthly_resource_cycle",
            refund_entry: "refund_if_workspace_not_ready",
            compensation_entry: "compensate_on_resource_or_isolation_fault",
            dispute_freeze_trigger: "freeze_on_security_or_isolation_dispute",
            resume_settlement_trigger: "resume_after_risk_cleared_dispute_closed",
            metadata: r#"{"seed":"db034","sku":"SBX_STD","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        "RPT_STD" => BillingTriggerMatrixSnapshot {
            payment_trigger: "lock_after_report_order_created",
            delivery_trigger: "report_generated_and_downloadable",
            acceptance_trigger: "buyer_accept_or_timeout_acceptance",
            billing_trigger: "bill_once_after_report_acceptance",
            settlement_cycle: "t_plus_1_once",
            refund_entry: "refund_if_report_not_generated_or_rejected",
            compensation_entry: "compensate_on_critical_report_defect",
            dispute_freeze_trigger: "freeze_on_report_quality_dispute",
            resume_settlement_trigger: "resume_on_review_passed_dispute_closed",
            metadata: r#"{"seed":"db034","sku":"RPT_STD","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
        _ => BillingTriggerMatrixSnapshot {
            payment_trigger: "unknown",
            delivery_trigger: "unknown",
            acceptance_trigger: "unknown",
            billing_trigger: "unknown",
            settlement_cycle: "unknown",
            refund_entry: "unknown",
            compensation_entry: "unknown",
            dispute_freeze_trigger: "unknown",
            resume_settlement_trigger: "unknown",
            metadata: r#"{"seed":"db034","sku":"UNKNOWN","source":"docs/03-db/sku-billing-trigger-matrix.md","fallback":"code"}"#,
        },
    }
}
