use crate::modules::delivery::dto::{
    ManageRevisionSubscriptionRequest, RevisionSubscriptionResponseData,
};
use crate::modules::order::domain::derive_layered_status;
use crate::modules::order::repo::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Value, json};

const DELIVERY_SUBSCRIPTION_MANAGE_EVENT: &str = "delivery.subscription.manage";
const DELIVERY_SUBSCRIPTION_READ_EVENT: &str = "delivery.subscription.read";

pub async fn manage_revision_subscription(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &ManageRevisionSubscriptionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<RevisionSubscriptionResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_subscription_context(&tx, order_id, request_id).await?;

    enforce_subject_status(&context, request_id)?;
    enforce_manage_scope(actor_role, tenant_id, &context.seller_org_id, request_id)?;
    enforce_file_sub(&context, request_id)?;
    enforce_version_release_status(&context, request_id)?;
    enforce_paid_or_renewable(&context, request_id)?;

    let cadence = normalize_cadence(
        payload
            .cadence
            .as_deref()
            .or(context.default_cadence.as_deref()),
        request_id,
    )?;
    let delivery_channel =
        normalize_delivery_channel(payload.delivery_channel.as_deref(), request_id)?;

    let start_version_no = payload
        .start_version_no
        .unwrap_or_else(|| context.current_version_no.max(1));
    validate_version_window(
        start_version_no,
        payload.last_delivered_version_no,
        context.current_version_no,
        request_id,
    )?;

    let (target_state, reason_code, operation) = match context.current_state.as_str() {
        "paused" | "expired" => (Some("buyer_locked"), Some("file_sub_renewed"), "renewed"),
        _ if context.existing_subscription_id.is_some() => (None, None, "updated"),
        _ => (None, None, "created"),
    };

    if let Some(target_state) = target_state {
        let layered_status = derive_layered_status(target_state, "paid");
        tx.execute(
            "UPDATE trade.order_main
             SET status = $2,
                 payment_status = 'paid',
                 delivery_status = $3,
                 acceptance_status = $4,
                 settlement_status = $5,
                 dispute_status = $6,
                 last_reason_code = $7,
                 updated_at = now()
             WHERE order_id = $1::text::uuid",
            &[
                &order_id,
                &target_state,
                &layered_status.delivery_status,
                &layered_status.acceptance_status,
                &layered_status.settlement_status,
                &layered_status.dispute_status,
                &reason_code.unwrap_or("file_sub_subscription_managed"),
            ],
        )
        .await
        .map_err(map_db_error)?;

        write_trade_audit_event(
            &tx,
            "order",
            order_id,
            actor_role,
            "trade.order.file_sub.transition",
            "success",
            request_id,
            trace_id,
        )
        .await?;
    }

    let subscription_status = "active";
    let metadata = merged_metadata(
        context.existing_metadata.as_ref(),
        payload.metadata.as_ref(),
        actor_role,
        operation,
        request_id,
        idempotency_key,
    );

    let row = if let Some(existing_id) = context.existing_subscription_id.as_deref() {
        tx.query_one(
            "UPDATE delivery.revision_subscription
             SET cadence = $2,
                 delivery_channel = $3,
                 start_version_no = $4,
                 last_delivered_version_no = COALESCE($5, last_delivered_version_no),
                 next_delivery_at = COALESCE(
                   $6::timestamptz,
                   CASE
                     WHEN $2 = 'weekly' THEN now() + INTERVAL '7 days'
                     WHEN $2 = 'monthly' THEN now() + INTERVAL '1 month'
                     WHEN $2 = 'quarterly' THEN now() + INTERVAL '3 months'
                     WHEN $2 = 'yearly' THEN now() + INTERVAL '1 year'
                     ELSE now() + INTERVAL '1 month'
                   END
                 ),
                 subscription_status = $7,
                 metadata = $8::jsonb,
                 updated_at = now()
             WHERE revision_subscription_id = $1::text::uuid
             RETURNING revision_subscription_id::text,
                       order_id::text,
                       asset_id::text,
                       sku_id::text,
                       cadence,
                       delivery_channel,
                       start_version_no,
                       last_delivered_version_no,
                       $9::int4,
                       to_char(next_delivery_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       subscription_status,
                       metadata,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &existing_id,
                &cadence,
                &delivery_channel,
                &start_version_no,
                &payload.last_delivered_version_no,
                &payload.next_delivery_at,
                &subscription_status,
                &metadata,
                &context.current_version_no,
            ],
        )
        .await
        .map_err(map_db_error)?
    } else {
        tx.query_one(
            "INSERT INTO delivery.revision_subscription (
               order_id,
               asset_id,
               sku_id,
               cadence,
               delivery_channel,
               start_version_no,
               last_delivered_version_no,
               next_delivery_at,
               subscription_status,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               $3::text::uuid,
               $4,
               $5,
               $6,
               $7,
               COALESCE(
                 $8::timestamptz,
                 CASE
                   WHEN $4 = 'weekly' THEN now() + INTERVAL '7 days'
                   WHEN $4 = 'monthly' THEN now() + INTERVAL '1 month'
                   WHEN $4 = 'quarterly' THEN now() + INTERVAL '3 months'
                   WHEN $4 = 'yearly' THEN now() + INTERVAL '1 year'
                   ELSE now() + INTERVAL '1 month'
                 END
               ),
               $9,
               $10::jsonb
             )
             RETURNING revision_subscription_id::text,
                       order_id::text,
                       asset_id::text,
                       sku_id::text,
                       cadence,
                       delivery_channel,
                       start_version_no,
                       last_delivered_version_no,
                       $11::int4,
                       to_char(next_delivery_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       subscription_status,
                       metadata,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &order_id,
                &context.asset_id,
                &context.sku_id,
                &cadence,
                &delivery_channel,
                &start_version_no,
                &payload.last_delivered_version_no,
                &payload.next_delivery_at,
                &subscription_status,
                &metadata,
                &context.current_version_no,
            ],
        )
        .await
        .map_err(map_db_error)?
    };

    write_delivery_subscription_audit_event(
        &tx,
        order_id,
        actor_role,
        DELIVERY_SUBSCRIPTION_MANAGE_EVENT,
        operation,
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "seller_org_id": context.seller_org_id,
            "buyer_org_id": context.buyer_org_id,
            "asset_id": context.asset_id,
            "sku_id": context.sku_id,
            "sku_type": context.sku_type,
            "cadence": cadence,
            "delivery_channel": delivery_channel,
            "start_version_no": start_version_no,
            "subscription_status": subscription_status,
            "idempotency_key": idempotency_key,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(map_revision_subscription_response(
        &row,
        context.sku_type,
        target_state.unwrap_or(context.current_state.as_str()),
        "paid",
        Some(operation),
    ))
}

pub async fn get_revision_subscription(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<RevisionSubscriptionResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let context = load_subscription_context(&tx, order_id, request_id).await?;

    enforce_subject_status(&context, request_id)?;
    enforce_read_scope(
        actor_role,
        tenant_id,
        &context.buyer_org_id,
        &context.seller_org_id,
        request_id,
    )?;
    enforce_file_sub(&context, request_id)?;

    let Some(existing_id) = context.existing_subscription_id.as_deref() else {
        return Err(not_found(
            &format!("revision subscription not found for order: {order_id}"),
            request_id,
        ));
    };

    let effective_status = effective_subscription_status(
        context.current_state.as_str(),
        context.existing_subscription_status.as_deref(),
    );

    let row = if context.existing_subscription_status.as_deref() != Some(effective_status) {
        tx.query_one(
            "UPDATE delivery.revision_subscription
             SET subscription_status = $2,
                 updated_at = now()
             WHERE revision_subscription_id = $1::text::uuid
             RETURNING revision_subscription_id::text,
                       order_id::text,
                       asset_id::text,
                       sku_id::text,
                       cadence,
                       delivery_channel,
                       start_version_no,
                       last_delivered_version_no,
                       $3::int4,
                       to_char(next_delivery_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       subscription_status,
                       metadata,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&existing_id, &effective_status, &context.current_version_no],
        )
        .await
        .map_err(map_db_error)?
    } else {
        tx.query_one(
            "SELECT revision_subscription_id::text,
                    order_id::text,
                    asset_id::text,
                    sku_id::text,
                    cadence,
                    delivery_channel,
                    start_version_no,
                    last_delivered_version_no,
                    $2::int4,
                    to_char(next_delivery_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    subscription_status,
                    metadata,
                    to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM delivery.revision_subscription
             WHERE revision_subscription_id = $1::text::uuid",
            &[&existing_id, &context.current_version_no],
        )
        .await
        .map_err(map_db_error)?
    };

    write_delivery_subscription_audit_event(
        &tx,
        order_id,
        actor_role,
        DELIVERY_SUBSCRIPTION_READ_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "buyer_org_id": context.buyer_org_id,
            "seller_org_id": context.seller_org_id,
            "asset_id": context.asset_id,
            "sku_id": context.sku_id,
            "sku_type": context.sku_type,
            "subscription_status": effective_status,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(map_revision_subscription_response(
        &row,
        context.sku_type,
        &context.current_state,
        &context.payment_status,
        None,
    ))
}

struct SubscriptionContext {
    current_state: String,
    payment_status: String,
    buyer_org_id: String,
    seller_org_id: String,
    buyer_org_status: String,
    seller_org_status: String,
    asset_id: String,
    current_version_no: i32,
    asset_version_status: String,
    product_status: String,
    product_review_status: String,
    sku_id: String,
    sku_type: String,
    default_cadence: Option<String>,
    existing_subscription_id: Option<String>,
    existing_subscription_status: Option<String>,
    existing_metadata: Option<Value>,
}

async fn load_subscription_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<SubscriptionContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT o.status,
                    o.payment_status,
                    o.buyer_org_id::text,
                    o.seller_org_id::text,
                    buyer.status,
                    seller.status,
                    av.asset_id::text,
                    av.version_no,
                    av.status,
                    p.status,
                    COALESCE(p.metadata ->> 'review_status', ''),
                    s.sku_id::text,
                    s.sku_type,
                    COALESCE(
                      NULLIF(o.price_snapshot_json ->> 'subscription_cadence', ''),
                      NULLIF(p.metadata ->> 'subscription_cadence', ''),
                      'monthly'
                    ),
                    rs.revision_subscription_id::text,
                    rs.subscription_status,
                    rs.metadata
             FROM trade.order_main o
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             JOIN catalog.asset_version av ON av.asset_version_id = o.asset_version_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             JOIN catalog.product p ON p.product_id = s.product_id
             LEFT JOIN delivery.revision_subscription rs ON rs.order_id = o.order_id
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(
            &format!("order not found: {order_id}"),
            request_id,
        ));
    };

    Ok(SubscriptionContext {
        current_state: row.get(0),
        payment_status: row.get(1),
        buyer_org_id: row.get(2),
        seller_org_id: row.get(3),
        buyer_org_status: row.get(4),
        seller_org_status: row.get(5),
        asset_id: row.get(6),
        current_version_no: row.get(7),
        asset_version_status: row.get(8),
        product_status: row.get(9),
        product_review_status: row.get(10),
        sku_id: row.get(11),
        sku_type: row.get(12),
        default_cadence: row.get(13),
        existing_subscription_id: row.get(14),
        existing_subscription_status: row.get(15),
        existing_metadata: row.get(16),
    })
}

fn enforce_subject_status(
    context: &SubscriptionContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.buyer_org_status != "active" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: buyer organization status `{}` is not active",
                context.buyer_org_status
            ),
            request_id,
        ));
    }
    if context.seller_org_status != "active" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: seller organization status `{}` is not active",
                context.seller_org_status
            ),
            request_id,
        ));
    }
    Ok(())
}

fn enforce_file_sub(
    context: &SubscriptionContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.sku_type == "FILE_SUB" {
        return Ok(());
    }
    Err(conflict(
        &format!(
            "REVISION_SUBSCRIPTION_FORBIDDEN: order sku_type `{}` is not FILE_SUB",
            context.sku_type
        ),
        request_id,
    ))
}

fn enforce_version_release_status(
    context: &SubscriptionContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.asset_version_status != "active" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: asset_version status `{}` is not active",
                context.asset_version_status
            ),
            request_id,
        ));
    }
    if context.product_status != "listed" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: product status `{}` is not listed",
                context.product_status
            ),
            request_id,
        ));
    }
    if context.product_review_status != "approved" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: product review status `{}` is not approved",
                context.product_review_status
            ),
            request_id,
        ));
    }
    Ok(())
}

fn enforce_paid_or_renewable(
    context: &SubscriptionContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.payment_status != "paid" {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: payment_status `{}` is not paid",
                context.payment_status
            ),
            request_id,
        ));
    }
    if matches!(
        context.current_state.as_str(),
        "buyer_locked"
            | "accepted"
            | "seller_delivering"
            | "delivered"
            | "paused"
            | "expired"
            | "dispute_opened"
    ) {
        return Ok(());
    }
    Err(conflict(
        &format!(
            "REVISION_SUBSCRIPTION_FORBIDDEN: current_state `{}` does not allow subscription management",
            context.current_state
        ),
        request_id,
    ))
}

fn normalize_cadence(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value.unwrap_or("monthly").trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "weekly" | "monthly" | "quarterly" | "yearly"
    ) {
        return Ok(normalized);
    }
    Err(bad_request(
        "cadence must be one of weekly/monthly/quarterly/yearly",
        request_id,
    ))
}

fn normalize_delivery_channel(
    value: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value.unwrap_or("file_ticket").trim().to_ascii_lowercase();
    if normalized == "file_ticket" {
        return Ok(normalized);
    }
    Err(bad_request(
        "delivery_channel must be `file_ticket` for FILE_SUB in V1",
        request_id,
    ))
}

fn validate_version_window(
    start_version_no: i32,
    last_delivered_version_no: Option<i32>,
    current_version_no: i32,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if start_version_no <= 0 {
        return Err(bad_request("start_version_no must be > 0", request_id));
    }
    if start_version_no > current_version_no {
        return Err(conflict(
            &format!(
                "REVISION_SUBSCRIPTION_FORBIDDEN: start_version_no `{start_version_no}` exceeds current_version_no `{current_version_no}`"
            ),
            request_id,
        ));
    }
    if let Some(last_delivered_version_no) = last_delivered_version_no {
        if last_delivered_version_no < start_version_no - 1 {
            return Err(conflict(
                &format!(
                    "REVISION_SUBSCRIPTION_FORBIDDEN: last_delivered_version_no `{last_delivered_version_no}` is before allowed range for start_version_no `{start_version_no}`"
                ),
                request_id,
            ));
        }
        if last_delivered_version_no > current_version_no {
            return Err(conflict(
                &format!(
                    "REVISION_SUBSCRIPTION_FORBIDDEN: last_delivered_version_no `{last_delivered_version_no}` exceeds current_version_no `{current_version_no}`"
                ),
                request_id,
            ));
        }
    }
    Ok(())
}

fn effective_subscription_status<'a>(
    current_state: &str,
    existing_status: Option<&'a str>,
) -> &'a str {
    match current_state {
        "paused" => "paused",
        "expired" => "expired",
        "closed" => "closed",
        _ => existing_status.unwrap_or("active"),
    }
}

fn merged_metadata(
    existing: Option<&Value>,
    incoming: Option<&Value>,
    actor_role: &str,
    operation: &str,
    request_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Value {
    let mut metadata = existing.cloned().unwrap_or_else(|| json!({}));
    if let Some(incoming) = incoming {
        merge_json(&mut metadata, incoming.clone());
    }
    merge_json(
        &mut metadata,
        json!({
            "managed_by": actor_role,
            "last_operation": operation,
            "last_request_id": request_id,
            "last_idempotency_key": idempotency_key,
        }),
    );
    metadata
}

fn merge_json(target: &mut Value, incoming: Value) {
    match (target, incoming) {
        (Value::Object(target_map), Value::Object(incoming_map)) => {
            for (key, value) in incoming_map {
                match target_map.get_mut(&key) {
                    Some(existing) => merge_json(existing, value),
                    None => {
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (target, incoming) => *target = incoming,
    }
}

fn map_revision_subscription_response(
    row: &db::Row,
    sku_type: String,
    current_state: &str,
    payment_status: &str,
    operation: Option<&str>,
) -> RevisionSubscriptionResponseData {
    RevisionSubscriptionResponseData {
        revision_subscription_id: row.get(0),
        order_id: row.get(1),
        asset_id: row.get(2),
        sku_id: row.get(3),
        sku_type,
        cadence: row.get(4),
        delivery_channel: row.get(5),
        start_version_no: row.get(6),
        last_delivered_version_no: row.get(7),
        current_version_no: row.get(8),
        next_delivery_at: row.get(9),
        subscription_status: row.get(10),
        current_state: current_state.to_string(),
        payment_status: payment_status.to_string(),
        operation: operation.map(str::to_string),
        metadata: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn enforce_manage_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(
        actor_role,
        "seller_operator" | "seller_storage_operator" | "tenant_admin"
    ) && tenant_id == Some(seller_org_id)
    {
        return Ok(());
    }
    Err(forbidden(
        "revision subscription management is forbidden for current tenant scope",
        request_id,
    ))
}

fn enforce_read_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    if matches!(
        actor_role,
        "buyer_operator"
            | "seller_operator"
            | "seller_storage_operator"
            | "procurement_manager"
            | "tenant_admin"
    ) && (tenant_id == Some(buyer_org_id) || tenant_id == Some(seller_org_id))
    {
        return Ok(());
    }
    Err(forbidden(
        "revision subscription read is forbidden for current tenant scope",
        request_id,
    ))
}

async fn write_delivery_subscription_audit_event(
    client: &(impl GenericClient + Sync),
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    metadata: Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
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
               'delivery',
               'revision_subscription',
               $1::text::uuid,
               'role',
               $2,
               $3,
               $4,
               $5,
               $6::jsonb || jsonb_build_object('actor_role', $7)
             )",
            &[
                &ref_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &metadata,
                &actor_role,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn not_found(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: "REVISION_SUBSCRIPTION_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn forbidden(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: "REVISION_SUBSCRIPTION_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn bad_request(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "REVISION_SUBSCRIPTION_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}

fn conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: "REVISION_SUBSCRIPTION_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
