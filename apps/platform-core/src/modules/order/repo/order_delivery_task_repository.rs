use super::deliverability_gate::{
    PreparedDeliveryOptions, ensure_order_deliverable_and_prepare_delivery_with_options,
};
use axum::Json;
use axum::http::StatusCode;
use db::GenericClient;
use kernel::{ErrorCode, ErrorResponse};
use serde_json::json;

pub struct AutoCreatedDeliveryTask {
    pub delivery_id: String,
    pub current_status: String,
    pub created: bool,
    pub executor_type: String,
    pub executor_ref_id: Option<String>,
    pub responsible_scope: String,
}

pub async fn auto_create_delivery_task_if_needed(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    creation_source: &str,
) -> Result<Option<AutoCreatedDeliveryTask>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_status,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               s.sku_type
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    let order_status: String = row.get(0);
    let payment_status: String = row.get(1);
    let delivery_status: String = row.get(2);
    let buyer_org_id: String = row.get(3);
    let seller_org_id: String = row.get(4);
    let sku_type: String = row.get(5);

    if order_status != "buyer_locked"
        || payment_status != "paid"
        || delivery_status != "pending_delivery"
    {
        return Ok(None);
    }

    let routing = delivery_task_routing(&sku_type, &buyer_org_id, &seller_org_id);
    let prepared = match ensure_order_deliverable_and_prepare_delivery_with_options(
        client,
        order_id,
        actor_role,
        request_id,
        trace_id,
        &PreparedDeliveryOptions {
            creation_source,
            executor_type: routing.executor_type,
            executor_ref_id: Some(routing.executor_ref_id.as_str()),
            responsible_scope: routing.responsible_scope,
            audit_action_name: "trade.order.delivery_task.auto_created",
        },
    )
    .await
    {
        Ok(prepared) => prepared,
        Err((status, Json(error)))
            if status == StatusCode::CONFLICT
                && error
                    .message
                    .starts_with("ORDER_DELIVERABILITY_CHECK_FAILED:") =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    if prepared.created {
        write_delivery_task_outbox_event(
            client,
            order_id,
            &prepared.delivery_id,
            &sku_type,
            actor_role,
            request_id,
            trace_id,
            creation_source,
            routing.executor_type,
            &routing.executor_ref_id,
            routing.responsible_scope,
        )
        .await?;
    }

    Ok(Some(AutoCreatedDeliveryTask {
        delivery_id: prepared.delivery_id,
        current_status: prepared.current_status,
        created: prepared.created,
        executor_type: routing.executor_type.to_string(),
        executor_ref_id: Some(routing.executor_ref_id),
        responsible_scope: routing.responsible_scope.to_string(),
    }))
}

struct DeliveryTaskRouting<'a> {
    executor_type: &'a str,
    executor_ref_id: String,
    responsible_scope: &'a str,
}

fn delivery_task_routing<'a>(
    sku_type: &'a str,
    buyer_org_id: &str,
    seller_org_id: &str,
) -> DeliveryTaskRouting<'a> {
    match sku_type {
        "FILE_STD" => DeliveryTaskRouting {
            executor_type: "seller_org",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "seller_file_delivery",
        },
        "FILE_SUB" => DeliveryTaskRouting {
            executor_type: "seller_org",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "seller_revision_delivery",
        },
        "API_SUB" => DeliveryTaskRouting {
            executor_type: "buyer_org",
            executor_ref_id: buyer_org_id.to_string(),
            responsible_scope: "buyer_api_binding",
        },
        "API_PPU" => DeliveryTaskRouting {
            executor_type: "buyer_org",
            executor_ref_id: buyer_org_id.to_string(),
            responsible_scope: "buyer_api_authorization",
        },
        "SHARE_RO" => DeliveryTaskRouting {
            executor_type: "seller_org",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "seller_share_grant",
        },
        "QRY_LITE" => DeliveryTaskRouting {
            executor_type: "seller_org",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "seller_template_grant",
        },
        "SBX_STD" => DeliveryTaskRouting {
            executor_type: "buyer_org",
            executor_ref_id: buyer_org_id.to_string(),
            responsible_scope: "buyer_sandbox_enablement",
        },
        "RPT_STD" => DeliveryTaskRouting {
            executor_type: "seller_org",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "seller_report_delivery",
        },
        _ => DeliveryTaskRouting {
            executor_type: "platform",
            executor_ref_id: seller_org_id.to_string(),
            responsible_scope: "platform_delivery",
        },
    }
}

async fn write_delivery_task_outbox_event(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    delivery_id: &str,
    sku_type: &str,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
    creation_source: &str,
    executor_type: &str,
    executor_ref_id: &str,
    responsible_scope: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let payload = json!({
        "event_name": "delivery.task.auto_created",
        "event_schema_version": "v1",
        "authority_scope": "business",
        "source_of_truth": "database",
        "proof_commit_policy": "async_evidence",
        "order_id": order_id,
        "delivery_id": delivery_id,
        "sku_type": sku_type,
        "actor_role": actor_role,
        "creation_source": creation_source,
        "executor_type": executor_type,
        "executor_ref_id": executor_ref_id,
        "responsible_scope": responsible_scope,
        "initial_status": "prepared"
    });
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
               'delivery.delivery_record',
               $1::text::uuid,
               'delivery.task.auto_created',
               $2::jsonb,
               'pending',
               $3,
               $4,
               COALESCE($2::jsonb ->> 'event_schema_version', 'v1'),
               COALESCE($2::jsonb ->> 'authority_scope', 'business'),
               COALESCE($2::jsonb ->> 'source_of_truth', 'database'),
               COALESCE($2::jsonb ->> 'proof_commit_policy', 'async_evidence'),
               'kafka',
               'dtp.outbox.domain-events',
               $5,
               $5,
               encode(digest(($2::jsonb)::text, 'sha256'), 'hex')
             )
             RETURNING outbox_event_id::text",
            &[&delivery_id, &payload, &request_id, &trace_id, &order_id],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn map_db_error(err: db::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("delivery task auto-creation failed: {err}"),
            request_id: None,
        }),
    )
}

fn not_found(order_id: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("order not found: {order_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::delivery_task_routing;

    #[test]
    fn routes_seller_side_delivery_tasks() {
        let file_std = delivery_task_routing("FILE_STD", "buyer-1", "seller-1");
        assert_eq!(file_std.executor_type, "seller_org");
        assert_eq!(file_std.executor_ref_id, "seller-1");
        assert_eq!(file_std.responsible_scope, "seller_file_delivery");

        let report = delivery_task_routing("RPT_STD", "buyer-1", "seller-1");
        assert_eq!(report.executor_type, "seller_org");
        assert_eq!(report.responsible_scope, "seller_report_delivery");
    }

    #[test]
    fn routes_buyer_side_delivery_tasks() {
        let api = delivery_task_routing("API_SUB", "buyer-1", "seller-1");
        assert_eq!(api.executor_type, "buyer_org");
        assert_eq!(api.executor_ref_id, "buyer-1");
        assert_eq!(api.responsible_scope, "buyer_api_binding");

        let sandbox = delivery_task_routing("SBX_STD", "buyer-1", "seller-1");
        assert_eq!(sandbox.executor_type, "buyer_org");
        assert_eq!(sandbox.responsible_scope, "buyer_sandbox_enablement");
    }
}
