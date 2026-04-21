use crate::modules::integration::events::{
    BuildNotificationRequestInput, NotificationActionLink, NotificationAudience,
    NotificationRecipient, NotificationRequestedPayload, NotificationScene,
    NotificationSourceEvent, NotificationSubjectRef, build_notification_idempotency_key,
    build_notification_request_payload,
};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use db::{Error, GenericClient};
use serde_json::{Value, json};
use std::str::FromStr;

pub struct QueueNotificationRequest<'a> {
    pub aggregate_id: &'a str,
    pub payload: &'a NotificationRequestedPayload,
    pub idempotency_key: &'a str,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

pub async fn queue_notification_request(
    client: &(impl GenericClient + Sync),
    request: QueueNotificationRequest<'_>,
) -> Result<bool, Error> {
    let payload = serde_json::to_value(request.payload).map_err(|err| {
        Error::Bind(format!(
            "encode notification requested payload failed: {err}"
        ))
    })?;
    write_canonical_outbox_event(
        client,
        CanonicalOutboxWrite {
            aggregate_type: "notification.dispatch_request",
            aggregate_id: request.aggregate_id,
            event_type: "notification.requested",
            producer_service: "platform-core.integration",
            request_id: request.request_id,
            trace_id: request.trace_id,
            idempotency_key: Some(request.idempotency_key),
            occurred_at: request.payload.source_event.occurred_at.as_deref(),
            business_payload: &payload,
            deduplicate_by_idempotency_key: true,
        },
    )
    .await
}

pub fn prepare_notification_request(
    input: BuildNotificationRequestInput,
) -> (NotificationRequestedPayload, String) {
    let scene = input.scene;
    let audience = input.audience;
    let payload = build_notification_request_payload(input);
    let idempotency_key = build_notification_idempotency_key(
        scene,
        audience,
        &payload.source_event,
        &payload.recipient,
    );
    (payload, idempotency_key)
}

pub struct PaymentSuccessNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub billing_event_id: &'a str,
    pub payment_intent_id: &'a str,
    pub provider_reference_id: &'a str,
    pub provider_result_source: &'a str,
    pub provider_status: &'a str,
    pub occurred_at: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PaymentSuccessNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct PaymentSuccessNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    buyer_locked_at: Option<String>,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    delivery_type: String,
    delivery_route: String,
}

#[derive(Debug, Clone)]
struct NotificationRecipientCandidate {
    user_id: String,
    address: String,
    display_name: String,
    persona: Option<String>,
}

pub async fn queue_payment_success_notifications(
    client: &(impl GenericClient + Sync),
    input: PaymentSuccessNotificationDispatchInput<'_>,
) -> Result<PaymentSuccessNotificationDispatchResult, Error> {
    let context = load_payment_success_notification_context(client, input.order_id).await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;
    let source_event = NotificationSourceEvent {
        aggregate_type: "billing.billing_event".to_string(),
        aggregate_id: input.billing_event_id.to_string(),
        event_type: "billing.event.recorded".to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.occurred_at.map(str::to_string),
    };

    let dispatches = vec![
        PaymentSuccessNotificationDispatch::buyer(&context, &buyer_recipient, &source_event),
        PaymentSuccessNotificationDispatch::seller(
            &context,
            &seller_recipient,
            &source_event,
            input.provider_status,
        ),
        PaymentSuccessNotificationDispatch::ops(
            &context,
            &ops_recipient,
            &source_event,
            input.payment_intent_id,
            input.provider_reference_id,
            input.provider_result_source,
            input.provider_status,
        ),
    ];

    let mut result = PaymentSuccessNotificationDispatchResult::default();
    for dispatch in dispatches {
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: input.billing_event_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

#[derive(Debug, Clone)]
struct PaymentSuccessNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

impl PaymentSuccessNotificationDispatch {
    fn buyer(
        context: &PaymentSuccessNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = format!("/portal/orders/{}", context.order_id);
        Self {
            scene: NotificationScene::PaymentSucceeded,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "付款成功，订单进入待交付",
                "headline": format!(
                    "订单 {} 已完成付款锁定，卖方 {} 将开始交付。",
                    context.order_id, context.seller_org_name
                ),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "buyer_locked_at": context.buyer_locked_at,
                "action_label": "查看订单详情",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-004",
                "transition_code": "payment_succeeded_to_pending_delivery",
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: "order_detail".to_string(),
                href: format!("/portal/orders/{}", context.order_id),
            }],
        }
    }

    fn seller(
        context: &PaymentSuccessNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        provider_status: &str,
    ) -> Self {
        Self {
            scene: NotificationScene::PendingDelivery,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "收到新待交付订单",
                "headline": format!(
                    "订单 {} 已完成收款锁定，请开始向买方 {} 交付。",
                    context.order_id, context.buyer_org_name
                ),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "show_ops_context": false,
                "action_label": "进入交付工作台",
                "action_href": format!("/portal/orders/{}/deliveries", context.order_id),
            }),
            metadata: json!({
                "task_id": "NOTIF-004",
                "transition_code": "payment_succeeded_to_pending_delivery",
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "provider_status": provider_status,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: "delivery_console".to_string(),
                href: format!("/portal/orders/{}/deliveries", context.order_id),
            }],
        }
    }

    fn ops(
        context: &PaymentSuccessNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        payment_intent_id: &str,
        provider_reference_id: &str,
        provider_result_source: &str,
        provider_status: &str,
    ) -> Self {
        Self {
            scene: NotificationScene::PendingDelivery,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "订单进入待交付（运营联查）",
                "headline": format!(
                    "订单 {} 已进入待交付，请关注交付责任方与通知链路状态。",
                    context.order_id
                ),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "show_ops_context": true,
                "billing_event_id": source_event.aggregate_id,
                "payment_intent_id": payment_intent_id,
                "provider_reference_id": provider_reference_id,
                "provider_result_source": provider_result_source,
                "provider_status": provider_status,
                "action_label": "查看运营联查页",
                "action_href": format!("/ops/orders/{}", context.order_id),
            }),
            metadata: json!({
                "task_id": "NOTIF-004",
                "transition_code": "payment_succeeded_to_pending_delivery",
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "payment_intent_id": payment_intent_id,
                "provider_reference_id": provider_reference_id,
                "provider_result_source": provider_result_source,
                "provider_status": provider_status,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "billing_event".to_string(),
                    ref_id: source_event.aggregate_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "payment_intent".to_string(),
                    ref_id: payment_intent_id.to_string(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: "ops_order_detail".to_string(),
                href: format!("/ops/orders/{}", context.order_id),
            }],
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

async fn load_payment_success_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<PaymentSuccessNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               CASE
                 WHEN o.buyer_locked_at IS NULL THEN NULL
                 ELSE to_char(o.buyer_locked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
               END,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               COALESCE(delivery.delivery_type, 'unknown'),
               COALESCE(delivery.delivery_route, 'unknown')
             FROM trade.order_main o
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             LEFT JOIN LATERAL (
               SELECT delivery_type, delivery_route
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) delivery ON TRUE
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing order context for notification dispatch: {order_id}"
        )));
    };
    Ok(PaymentSuccessNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        buyer_locked_at: row.get(10),
        buyer_org_id: row.get(11),
        buyer_org_name: row.get(12),
        seller_org_id: row.get(13),
        seller_org_name: row.get(14),
        delivery_type: row.get(15),
        delivery_route: row.get(16),
    })
}

pub struct DeliveryCompletionNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub delivery_branch: &'a str,
    pub result_ref_type: &'a str,
    pub result_ref_id: &'a str,
    pub source_event_aggregate_type: &'a str,
    pub source_event_event_type: &'a str,
    pub source_event_occurred_at: Option<&'a str>,
    pub delivery_type: Option<&'a str>,
    pub delivery_route: Option<&'a str>,
    pub receipt_hash: Option<&'a str>,
    pub delivery_commit_hash: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DeliveryCompletionNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct DeliveryCompletionNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    delivery_type: String,
    delivery_route: String,
}

#[derive(Debug, Clone)]
struct DeliveryCompletionNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

pub async fn queue_delivery_completion_notifications(
    client: &(impl GenericClient + Sync),
    input: DeliveryCompletionNotificationDispatchInput<'_>,
) -> Result<DeliveryCompletionNotificationDispatchResult, Error> {
    let context = load_delivery_completion_notification_context(
        client,
        input.order_id,
        input.delivery_type,
        input.delivery_route,
    )
    .await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;
    let source_event = NotificationSourceEvent {
        aggregate_type: input.source_event_aggregate_type.to_string(),
        aggregate_id: input.result_ref_id.to_string(),
        event_type: input.source_event_event_type.to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.source_event_occurred_at.map(str::to_string),
    };
    let manual_acceptance = requires_manual_acceptance_follow_up(&context.acceptance_status);

    let dispatches = vec![
        DeliveryCompletionNotificationDispatch::buyer(
            &context,
            &buyer_recipient,
            &source_event,
            input.delivery_branch,
            manual_acceptance,
        ),
        DeliveryCompletionNotificationDispatch::seller(
            &context,
            &seller_recipient,
            &source_event,
            input.delivery_branch,
            manual_acceptance,
        ),
        DeliveryCompletionNotificationDispatch::ops(
            &context,
            &ops_recipient,
            &source_event,
            input.delivery_branch,
            input.result_ref_type,
            input.receipt_hash,
            input.delivery_commit_hash,
            manual_acceptance,
        ),
    ];

    let mut result = DeliveryCompletionNotificationDispatchResult::default();
    for dispatch in dispatches {
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: input.result_ref_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

impl DeliveryCompletionNotificationDispatch {
    fn buyer(
        context: &DeliveryCompletionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        delivery_branch: &str,
        manual_acceptance: bool,
    ) -> Self {
        let branch_label = delivery_branch_label(delivery_branch);
        let action = buyer_delivery_action(delivery_branch, &context.order_id, manual_acceptance);
        let scene = if manual_acceptance {
            NotificationScene::PendingAcceptance
        } else {
            NotificationScene::DeliveryCompleted
        };
        let transition_code = if manual_acceptance {
            "delivery_completed_to_pending_acceptance"
        } else {
            "delivery_completed_ready_for_use"
        };
        let headline = if manual_acceptance {
            format!(
                "订单 {} 的{}已完成，请前往验收页确认交付结果。",
                context.order_id, branch_label
            )
        } else {
            format!(
                "订单 {} 的{}已完成，可按约定开始使用。",
                context.order_id, branch_label
            )
        };

        Self {
            scene,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": buyer_delivery_subject(delivery_branch, manual_acceptance),
                "headline": headline,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_branch_label": branch_label,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "show_ops_context": false,
                "action_label": action.label,
                "action_href": action.href,
            }),
            metadata: json!({
                "task_id": "NOTIF-005",
                "transition_code": transition_code,
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_branch": delivery_branch,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: action.link_code,
                href: action.href,
            }],
        }
    }

    fn seller(
        context: &DeliveryCompletionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        delivery_branch: &str,
        manual_acceptance: bool,
    ) -> Self {
        let branch_label = delivery_branch_label(delivery_branch);
        let action = seller_delivery_action(&context.order_id);
        let headline = if manual_acceptance {
            format!(
                "订单 {} 的{}已完成，当前等待买方验收。",
                context.order_id, branch_label
            )
        } else {
            format!(
                "订单 {} 的{}已完成，买方已可开始使用。",
                context.order_id, branch_label
            )
        };

        Self {
            scene: NotificationScene::DeliveryCompleted,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": format!("{}已完成", branch_label),
                "headline": headline,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_branch_label": branch_label,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "show_ops_context": false,
                "action_label": action.label,
                "action_href": action.href,
            }),
            metadata: json!({
                "task_id": "NOTIF-005",
                "transition_code": "delivery_completed_seller_visible",
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_branch": delivery_branch,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: action.link_code,
                href: action.href,
            }],
        }
    }

    fn ops(
        context: &DeliveryCompletionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        delivery_branch: &str,
        result_ref_type: &str,
        receipt_hash: Option<&str>,
        delivery_commit_hash: Option<&str>,
        manual_acceptance: bool,
    ) -> Self {
        let branch_label = delivery_branch_label(delivery_branch);
        let action = ops_delivery_action(&context.order_id);
        let headline = if manual_acceptance {
            format!(
                "订单 {} 的{}已完成并进入待验收，请关注后续验收和争议状态。",
                context.order_id, branch_label
            )
        } else {
            format!(
                "订单 {} 的{}已完成并可直接使用，请关注交付与审计链路状态。",
                context.order_id, branch_label
            )
        };

        Self {
            scene: NotificationScene::DeliveryCompleted,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "交付完成（运营联查）",
                "headline": headline,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_branch_label": branch_label,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "show_ops_context": true,
                "delivery_ref_type": result_ref_type,
                "delivery_ref_id": source_event.aggregate_id,
                "receipt_hash": receipt_hash,
                "delivery_commit_hash": delivery_commit_hash,
                "action_label": action.label,
                "action_href": action.href,
            }),
            metadata: json!({
                "task_id": "NOTIF-005",
                "transition_code": "delivery_completed_ops_visible",
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_branch": delivery_branch,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "delivery_ref_type": result_ref_type,
                "delivery_ref_id": source_event.aggregate_id,
                "receipt_hash": receipt_hash,
                "delivery_commit_hash": delivery_commit_hash,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: result_ref_type.to_string(),
                    ref_id: source_event.aggregate_id.clone(),
                },
            ],
            links: vec![NotificationActionLink {
                link_code: action.link_code,
                href: action.href,
            }],
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

async fn load_delivery_completion_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    delivery_type_override: Option<&str>,
    delivery_route_override: Option<&str>,
) -> Result<DeliveryCompletionNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               COALESCE(delivery.delivery_type, ''),
               COALESCE(delivery.delivery_route, '')
             FROM trade.order_main o
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             LEFT JOIN LATERAL (
               SELECT delivery_type, delivery_route
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) delivery ON TRUE
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing order context for delivery notification dispatch: {order_id}"
        )));
    };

    let fallback_delivery_type: String = row.get(17);
    let fallback_delivery_route: String = row.get(18);
    Ok(DeliveryCompletionNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        acceptance_status: row.get(10),
        settlement_status: row.get(11),
        dispute_status: row.get(12),
        buyer_org_id: row.get(13),
        buyer_org_name: row.get(14),
        seller_org_id: row.get(15),
        seller_org_name: row.get(16),
        delivery_type: normalize_delivery_field(delivery_type_override, &fallback_delivery_type),
        delivery_route: normalize_delivery_field(delivery_route_override, &fallback_delivery_route),
    })
}

pub struct AcceptanceOutcomeNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub acceptance_record_id: &'a str,
    pub scene: &'a str,
    pub occurred_at: Option<&'a str>,
    pub reason_code: &'a str,
    pub reason_detail: Option<&'a str>,
    pub verification_summary: Option<&'a Value>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AcceptanceOutcomeNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct AcceptanceOutcomeNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    delivery_type: String,
    delivery_route: String,
}

#[derive(Debug, Clone)]
struct AcceptanceOutcomeNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

pub async fn queue_acceptance_outcome_notifications(
    client: &(impl GenericClient + Sync),
    input: AcceptanceOutcomeNotificationDispatchInput<'_>,
) -> Result<AcceptanceOutcomeNotificationDispatchResult, Error> {
    let scene = parse_acceptance_scene(input.scene)?;
    let context = load_acceptance_outcome_notification_context(client, input.order_id).await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;
    let source_event = NotificationSourceEvent {
        aggregate_type: "trade.acceptance_record".to_string(),
        aggregate_id: input.acceptance_record_id.to_string(),
        event_type: input.scene.to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.occurred_at.map(str::to_string),
    };

    let dispatches = vec![
        AcceptanceOutcomeNotificationDispatch::buyer(
            &context,
            &buyer_recipient,
            &source_event,
            scene,
            input.reason_code,
            input.reason_detail,
            input.verification_summary,
        ),
        AcceptanceOutcomeNotificationDispatch::seller(
            &context,
            &seller_recipient,
            &source_event,
            scene,
            input.reason_code,
            input.reason_detail,
            input.verification_summary,
        ),
        AcceptanceOutcomeNotificationDispatch::ops(
            &context,
            &ops_recipient,
            &source_event,
            scene,
            input.reason_code,
            input.reason_detail,
            input.verification_summary,
        ),
    ];

    let mut result = AcceptanceOutcomeNotificationDispatchResult::default();
    for dispatch in dispatches {
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: input.acceptance_record_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

impl AcceptanceOutcomeNotificationDispatch {
    fn buyer(
        context: &AcceptanceOutcomeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
        reason_code: &str,
        reason_detail: Option<&str>,
        verification_summary: Option<&Value>,
    ) -> Self {
        let (subject, headline, action_summary, action_label, action_href, links, transition_code) =
            match scene {
                NotificationScene::AcceptancePassed => (
                    "验收通过，订单进入结算处理".to_string(),
                    format!(
                        "订单 {} 已完成验收通过，可回到订单详情查看后续结算与归档状态。",
                        context.order_id
                    ),
                    "回到订单详情查看验收结论，并关注后续账单与结算变化。".to_string(),
                    "查看订单详情".to_string(),
                    order_detail_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "billing_center".to_string(),
                            href: billing_center_href(&context.order_id),
                        },
                    ],
                    "acceptance_passed_buyer_visible",
                ),
                NotificationScene::AcceptanceRejected => (
                    "验收未通过，请处理争议".to_string(),
                    format!(
                        "订单 {} 的交付结果已被拒收，可前往争议提交页补充原因与证据。",
                        context.order_id
                    ),
                    "如需发起正式争议，请在争议提交页补充拒收原因和证据摘要。".to_string(),
                    "进入争议提交页".to_string(),
                    dispute_create_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "dispute_create".to_string(),
                            href: dispute_create_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                    ],
                    "acceptance_rejected_buyer_visible",
                ),
                other => unreachable!("unsupported acceptance scene: {}", other.as_str()),
            };

        Self {
            scene,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "reason_code": reason_code,
                "reason_detail": reason_detail,
                "verification_summary": verification_summary,
                "show_ops_context": false,
                "action_label": action_label,
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "reason_code": reason_code,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "acceptance_record".to_string(),
                    ref_id: source_event.aggregate_id.clone(),
                },
            ],
            links,
        }
    }

    fn seller(
        context: &AcceptanceOutcomeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
        reason_code: &str,
        reason_detail: Option<&str>,
        verification_summary: Option<&Value>,
    ) -> Self {
        let (subject, headline, action_summary, action_label, action_href, links, transition_code) =
            match scene {
                NotificationScene::AcceptancePassed => (
                    "买方已验收通过".to_string(),
                    format!(
                        "订单 {} 已被买方确认验收通过，可回到订单详情查看后续结算状态。",
                        context.order_id
                    ),
                    "回到订单详情确认验收结果，并关注后续放款与结算进度。".to_string(),
                    "查看订单详情".to_string(),
                    order_detail_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "billing_center".to_string(),
                            href: billing_center_href(&context.order_id),
                        },
                    ],
                    "acceptance_passed_seller_visible",
                ),
                NotificationScene::AcceptanceRejected => (
                    "买方已拒收交付结果".to_string(),
                    format!(
                        "订单 {} 的交付结果已被买方拒收，请在订单详情确认原因并准备后续处理。",
                        context.order_id
                    ),
                    "回到订单详情查看拒收原因和验收摘要，必要时准备争议处理材料。".to_string(),
                    "查看订单详情".to_string(),
                    order_detail_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "dispute_create".to_string(),
                            href: dispute_create_href(&context.order_id),
                        },
                    ],
                    "acceptance_rejected_seller_visible",
                ),
                other => unreachable!("unsupported acceptance scene: {}", other.as_str()),
            };

        Self {
            scene,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "reason_code": reason_code,
                "reason_detail": reason_detail,
                "verification_summary": verification_summary,
                "show_ops_context": false,
                "action_label": action_label,
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "reason_code": reason_code,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "acceptance_record".to_string(),
                    ref_id: source_event.aggregate_id.clone(),
                },
            ],
            links,
        }
    }

    fn ops(
        context: &AcceptanceOutcomeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
        reason_code: &str,
        reason_detail: Option<&str>,
        verification_summary: Option<&Value>,
    ) -> Self {
        let (subject, headline, action_summary, action_label, action_href, links, transition_code) =
            match scene {
                NotificationScene::AcceptancePassed => (
                    "验收通过（运营联查）".to_string(),
                    format!(
                        "订单 {} 已完成验收通过，请在账单中心跟踪结算、放款与归档状态。",
                        context.order_id
                    ),
                    "进入账单中心回查结算结果，并确认验收事件已推进后续账单链路。".to_string(),
                    "查看账单中心".to_string(),
                    billing_center_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "billing_center".to_string(),
                            href: billing_center_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                    ],
                    "acceptance_passed_ops_visible",
                ),
                NotificationScene::AcceptanceRejected => (
                    "验收拒收（运营联查）".to_string(),
                    format!(
                        "订单 {} 已进入拒收处理，请前往争议提交页关注后续案件流转。",
                        context.order_id
                    ),
                    "进入争议提交页联查拒收原因与证据摘要，并跟踪后续案件处理。".to_string(),
                    "查看争议提交页".to_string(),
                    dispute_create_href(&context.order_id),
                    vec![
                        NotificationActionLink {
                            link_code: "dispute_create".to_string(),
                            href: dispute_create_href(&context.order_id),
                        },
                        NotificationActionLink {
                            link_code: "order_detail".to_string(),
                            href: order_detail_href(&context.order_id),
                        },
                    ],
                    "acceptance_rejected_ops_visible",
                ),
                other => unreachable!("unsupported acceptance scene: {}", other.as_str()),
            };

        Self {
            scene,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "acceptance_status_label": acceptance_status_label(&context.acceptance_status),
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "reason_code": reason_code,
                "reason_detail": reason_detail,
                "verification_summary": verification_summary,
                "acceptance_record_id": source_event.aggregate_id,
                "show_ops_context": true,
                "action_label": action_label,
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "delivery_type": context.delivery_type,
                "delivery_route": context.delivery_route,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "reason_code": reason_code,
                "acceptance_record_id": source_event.aggregate_id,
            }),
            subject_refs: vec![
                NotificationSubjectRef {
                    ref_type: "order".to_string(),
                    ref_id: context.order_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "product".to_string(),
                    ref_id: context.product_id.clone(),
                },
                NotificationSubjectRef {
                    ref_type: "acceptance_record".to_string(),
                    ref_id: source_event.aggregate_id.clone(),
                },
            ],
            links,
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

async fn load_acceptance_outcome_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<AcceptanceOutcomeNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               COALESCE(delivery.delivery_type, ''),
               COALESCE(delivery.delivery_route, '')
             FROM trade.order_main o
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             LEFT JOIN LATERAL (
               SELECT delivery_type, delivery_route
               FROM delivery.delivery_record
               WHERE order_id = o.order_id
               ORDER BY created_at DESC, delivery_id DESC
               LIMIT 1
             ) delivery ON TRUE
             WHERE o.order_id = $1::text::uuid",
            &[&order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing order context for acceptance notification dispatch: {order_id}"
        )));
    };

    Ok(AcceptanceOutcomeNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        acceptance_status: row.get(10),
        settlement_status: row.get(11),
        dispute_status: row.get(12),
        buyer_org_id: row.get(13),
        buyer_org_name: row.get(14),
        seller_org_id: row.get(15),
        seller_org_name: row.get(16),
        delivery_type: normalize_delivery_field(None, &row.get::<_, String>(17)),
        delivery_route: normalize_delivery_field(None, &row.get::<_, String>(18)),
    })
}

pub struct BillingResolutionNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub billing_event_id: &'a str,
    pub scene: &'a str,
    pub occurred_at: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BillingResolutionNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct BillingResolutionNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    order_currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    resolution_amount: String,
    resolution_currency_code: String,
    case_id: String,
    decision_code: String,
    penalty_code: Option<String>,
    reason_code: Option<String>,
    liability_type: Option<String>,
    provider_key: Option<String>,
    provider_status: Option<String>,
    provider_result_id: Option<String>,
    resolution_record_id: String,
    resolution_ref_type: String,
}

#[derive(Debug, Clone)]
struct BillingResolutionNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

pub async fn queue_billing_resolution_notifications(
    client: &(impl GenericClient + Sync),
    input: BillingResolutionNotificationDispatchInput<'_>,
) -> Result<BillingResolutionNotificationDispatchResult, Error> {
    let scene = parse_billing_resolution_scene(input.scene)?;
    let context = load_billing_resolution_notification_context(
        client,
        input.order_id,
        input.billing_event_id,
        scene,
    )
    .await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;
    let source_event = NotificationSourceEvent {
        aggregate_type: "billing.billing_event".to_string(),
        aggregate_id: input.billing_event_id.to_string(),
        event_type: "billing.event.recorded".to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.occurred_at.map(str::to_string),
    };

    let dispatches = vec![
        BillingResolutionNotificationDispatch::buyer(
            &context,
            &buyer_recipient,
            &source_event,
            scene,
        ),
        BillingResolutionNotificationDispatch::seller(
            &context,
            &seller_recipient,
            &source_event,
            scene,
        ),
        BillingResolutionNotificationDispatch::ops(&context, &ops_recipient, &source_event, scene),
    ];

    let mut result = BillingResolutionNotificationDispatchResult::default();
    for dispatch in dispatches {
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: input.billing_event_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

impl BillingResolutionNotificationDispatch {
    fn buyer(
        context: &BillingResolutionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
    ) -> Self {
        let (subject, headline, action_summary, transition_code) = match scene {
            NotificationScene::RefundCompleted => (
                "退款已完成".to_string(),
                format!(
                    "订单 {} 已完成退款执行，可前往账单页查看退款与责任判定摘要。",
                    context.order_id
                ),
                "进入退款/赔付处理页查看执行记录、责任判定和账单调整结果。".to_string(),
                "refund_completed_buyer_visible",
            ),
            NotificationScene::CompensationCompleted => (
                "赔付已完成".to_string(),
                format!(
                    "订单 {} 已完成赔付执行，可前往账单页查看赔付与责任判定摘要。",
                    context.order_id
                ),
                "进入退款/赔付处理页查看赔付记录、责任判定和账单调整结果。".to_string(),
                "compensation_completed_buyer_visible",
            ),
            other => unreachable!("unsupported billing scene: {}", other.as_str()),
        };
        let action_href = billing_resolution_href(&context.order_id, Some(&context.case_id));

        Self {
            scene,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "order_currency_code": context.order_currency_code,
                "resolution_amount": context.resolution_amount,
                "resolution_currency_code": context.resolution_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "decision_code": context.decision_code,
                "penalty_code": context.penalty_code,
                "reason_code": context.reason_code,
                "case_id": context.case_id,
                "show_ops_context": false,
                "action_label": "查看退款/赔付处理页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "decision_code": context.decision_code,
                "reason_code": context.reason_code,
                "resolution_ref_type": context.resolution_ref_type,
                "resolution_record_id": context.resolution_record_id,
            }),
            subject_refs: context.subject_refs(source_event),
            links: billing_resolution_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn seller(
        context: &BillingResolutionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
    ) -> Self {
        let (subject, headline, action_summary, transition_code) = match scene {
            NotificationScene::RefundCompleted => (
                "退款已完成，请核对账单调整".to_string(),
                format!(
                    "订单 {} 已完成退款执行，请前往账单页查看退款金额与责任判定结果。",
                    context.order_id
                ),
                "进入退款/赔付处理页核对退款金额、责任判定和结算调整。".to_string(),
                "refund_completed_seller_visible",
            ),
            NotificationScene::CompensationCompleted => (
                "赔付已完成，请核对账单调整".to_string(),
                format!(
                    "订单 {} 已完成赔付执行，请前往账单页查看赔付金额与责任判定结果。",
                    context.order_id
                ),
                "进入退款/赔付处理页核对赔付金额、责任判定和结算调整。".to_string(),
                "compensation_completed_seller_visible",
            ),
            other => unreachable!("unsupported billing scene: {}", other.as_str()),
        };
        let action_href = billing_resolution_href(&context.order_id, Some(&context.case_id));

        Self {
            scene,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "order_currency_code": context.order_currency_code,
                "resolution_amount": context.resolution_amount,
                "resolution_currency_code": context.resolution_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "decision_code": context.decision_code,
                "penalty_code": context.penalty_code,
                "reason_code": context.reason_code,
                "case_id": context.case_id,
                "show_ops_context": false,
                "action_label": "查看退款/赔付处理页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "decision_code": context.decision_code,
                "reason_code": context.reason_code,
                "resolution_ref_type": context.resolution_ref_type,
                "resolution_record_id": context.resolution_record_id,
            }),
            subject_refs: context.subject_refs(source_event),
            links: billing_resolution_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn ops(
        context: &BillingResolutionNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
        scene: NotificationScene,
    ) -> Self {
        let (subject, headline, action_summary, transition_code) = match scene {
            NotificationScene::RefundCompleted => (
                "退款已完成（运营联查）".to_string(),
                format!(
                    "订单 {} 已完成退款执行，请在账单页联查退款、责任判定与渠道结果。",
                    context.order_id
                ),
                "进入退款/赔付处理页联查退款记录、责任判定和渠道执行结果。".to_string(),
                "refund_completed_ops_visible",
            ),
            NotificationScene::CompensationCompleted => (
                "赔付已完成（运营联查）".to_string(),
                format!(
                    "订单 {} 已完成赔付执行，请在账单页联查赔付、责任判定与渠道结果。",
                    context.order_id
                ),
                "进入退款/赔付处理页联查赔付记录、责任判定和渠道执行结果。".to_string(),
                "compensation_completed_ops_visible",
            ),
            other => unreachable!("unsupported billing scene: {}", other.as_str()),
        };
        let action_href = billing_resolution_href(&context.order_id, Some(&context.case_id));

        Self {
            scene,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": subject,
                "headline": headline,
                "action_summary": action_summary,
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "order_currency_code": context.order_currency_code,
                "resolution_amount": context.resolution_amount,
                "resolution_currency_code": context.resolution_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "current_state": context.order_status,
                "current_state_label": order_state_label(&context.order_status),
                "case_id": context.case_id,
                "decision_code": context.decision_code,
                "penalty_code": context.penalty_code,
                "reason_code": context.reason_code,
                "liability_type": context.liability_type,
                "provider_key": context.provider_key,
                "provider_status": context.provider_status,
                "provider_result_id": context.provider_result_id,
                "resolution_ref_type": context.resolution_ref_type,
                "resolution_record_id": context.resolution_record_id,
                "show_ops_context": true,
                "action_label": "查看退款/赔付处理页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-006",
                "transition_code": transition_code,
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "decision_code": context.decision_code,
                "reason_code": context.reason_code,
                "liability_type": context.liability_type,
                "provider_key": context.provider_key,
                "provider_status": context.provider_status,
                "provider_result_id": context.provider_result_id,
                "resolution_ref_type": context.resolution_ref_type,
                "resolution_record_id": context.resolution_record_id,
            }),
            subject_refs: context.subject_refs(source_event),
            links: billing_resolution_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

impl BillingResolutionNotificationContext {
    fn subject_refs(&self, source_event: &NotificationSourceEvent) -> Vec<NotificationSubjectRef> {
        let mut refs = vec![
            NotificationSubjectRef {
                ref_type: "order".to_string(),
                ref_id: self.order_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "product".to_string(),
                ref_id: self.product_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "billing_event".to_string(),
                ref_id: source_event.aggregate_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: self.resolution_ref_type.clone(),
                ref_id: self.resolution_record_id.clone(),
            },
        ];
        if !self.case_id.is_empty() {
            refs.push(NotificationSubjectRef {
                ref_type: "dispute_case".to_string(),
                ref_id: self.case_id.clone(),
            });
        }
        refs
    }
}

async fn load_billing_resolution_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    billing_event_id: &str,
    scene: NotificationScene,
) -> Result<BillingResolutionNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               be.amount::text,
               be.currency_code,
               COALESCE(be.metadata, '{}'::jsonb)
             FROM billing.billing_event be
             JOIN trade.order_main o ON o.order_id = be.order_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             WHERE be.billing_event_id = $1::text::uuid
               AND o.order_id = $2::text::uuid",
            &[&billing_event_id, &order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing billing event context for notification dispatch: {billing_event_id}"
        )));
    };

    let metadata: Value = row.get(19);
    let (resolution_ref_type, resolution_record_field, provider_result_field) = match scene {
        NotificationScene::RefundCompleted => (
            "refund_record".to_string(),
            "refund_id",
            "provider_refund_id",
        ),
        NotificationScene::CompensationCompleted => (
            "compensation_record".to_string(),
            "compensation_id",
            "provider_transfer_id",
        ),
        other => {
            return Err(Error::Bind(format!(
                "unsupported billing resolution scene: {}",
                other.as_str()
            )));
        }
    };
    let resolution_record_id = json_text(&metadata, resolution_record_field).ok_or_else(|| {
        Error::Bind(format!(
            "missing {resolution_record_field} in billing event metadata: {billing_event_id}"
        ))
    })?;

    Ok(BillingResolutionNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        order_currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        acceptance_status: row.get(10),
        settlement_status: row.get(11),
        dispute_status: row.get(12),
        buyer_org_id: row.get(13),
        buyer_org_name: row.get(14),
        seller_org_id: row.get(15),
        seller_org_name: row.get(16),
        resolution_amount: row.get(17),
        resolution_currency_code: row.get(18),
        case_id: json_text(&metadata, "case_id").unwrap_or_default(),
        decision_code: json_text(&metadata, "decision_code").unwrap_or_default(),
        penalty_code: json_text(&metadata, "penalty_code"),
        reason_code: json_text(&metadata, "reason_code"),
        liability_type: json_text(&metadata, "liability_type"),
        provider_key: json_text(&metadata, "provider_key"),
        provider_status: json_text(&metadata, "provider_status"),
        provider_result_id: json_text(&metadata, provider_result_field),
        resolution_record_id,
        resolution_ref_type,
    })
}

fn parse_acceptance_scene(scene: &str) -> Result<NotificationScene, Error> {
    match NotificationScene::from_str(scene).map_err(Error::Bind)? {
        NotificationScene::AcceptancePassed => Ok(NotificationScene::AcceptancePassed),
        NotificationScene::AcceptanceRejected => Ok(NotificationScene::AcceptanceRejected),
        other => Err(Error::Bind(format!(
            "unsupported acceptance notification scene: {}",
            other.as_str()
        ))),
    }
}

fn parse_billing_resolution_scene(scene: &str) -> Result<NotificationScene, Error> {
    match NotificationScene::from_str(scene).map_err(Error::Bind)? {
        NotificationScene::RefundCompleted => Ok(NotificationScene::RefundCompleted),
        NotificationScene::CompensationCompleted => Ok(NotificationScene::CompensationCompleted),
        other => Err(Error::Bind(format!(
            "unsupported billing resolution notification scene: {}",
            other.as_str()
        ))),
    }
}

pub struct DisputeLifecycleNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub case_id: &'a str,
    pub dispute_occurred_at: Option<&'a str>,
    pub settlement_hold_event_id: Option<&'a str>,
    pub settlement_hold_occurred_at: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DisputeLifecycleNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct DisputeLifecycleNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    case_id: String,
    reason_code: String,
    case_status: String,
    freeze_ticket_id: Option<String>,
    legal_hold_id: Option<String>,
    governance_action_count: i64,
    settlement_freeze_count: i64,
}

#[derive(Debug, Clone)]
struct DisputeLifecycleNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

pub async fn queue_dispute_lifecycle_notifications(
    client: &(impl GenericClient + Sync),
    input: DisputeLifecycleNotificationDispatchInput<'_>,
) -> Result<DisputeLifecycleNotificationDispatchResult, Error> {
    let context =
        load_dispute_lifecycle_notification_context(client, input.order_id, input.case_id).await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;

    let dispute_source_event = NotificationSourceEvent {
        aggregate_type: "support.dispute_case".to_string(),
        aggregate_id: input.case_id.to_string(),
        event_type: "dispute.created".to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.dispute_occurred_at.map(str::to_string),
    };

    let mut dispatches = vec![
        DisputeLifecycleNotificationDispatch::buyer_dispute(
            &context,
            &buyer_recipient,
            &dispute_source_event,
        ),
        DisputeLifecycleNotificationDispatch::seller_dispute(
            &context,
            &seller_recipient,
            &dispute_source_event,
        ),
        DisputeLifecycleNotificationDispatch::ops_dispute(
            &context,
            &ops_recipient,
            &dispute_source_event,
        ),
    ];

    if let Some(settlement_hold_event_id) = input
        .settlement_hold_event_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let settlement_source_event = NotificationSourceEvent {
            aggregate_type: "billing.billing_event".to_string(),
            aggregate_id: settlement_hold_event_id.to_string(),
            event_type: "billing.event.recorded".to_string(),
            event_id: None,
            target_topic: Some("dtp.outbox.domain-events".to_string()),
            occurred_at: input.settlement_hold_occurred_at.map(str::to_string),
        };
        dispatches.extend([
            DisputeLifecycleNotificationDispatch::buyer_settlement_frozen(
                &context,
                &buyer_recipient,
                &settlement_source_event,
            ),
            DisputeLifecycleNotificationDispatch::seller_settlement_frozen(
                &context,
                &seller_recipient,
                &settlement_source_event,
            ),
            DisputeLifecycleNotificationDispatch::ops_settlement_frozen(
                &context,
                &ops_recipient,
                &settlement_source_event,
            ),
        ]);
    }

    let mut result = DisputeLifecycleNotificationDispatchResult::default();
    for dispatch in dispatches {
        let aggregate_id = dispatch.source_event.aggregate_id.clone();
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: &aggregate_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

impl DisputeLifecycleNotificationDispatch {
    fn buyer_dispute(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = dispute_create_href(&context.order_id);
        Self {
            scene: NotificationScene::DisputeEscalated,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "争议已受理，结算将暂缓处理",
                "headline": format!(
                    "订单 {} 已进入争议处理流程，请前往争议页查看受理状态并补充材料。",
                    context.order_id
                ),
                "action_summary": "进入争议提交页查看当前案件状态，必要时继续补充说明或证据。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "show_ops_context": false,
                "action_label": "查看争议处理页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "dispute_escalated_buyer_visible",
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: context.dispute_subject_refs(),
            links: dispute_case_links(&context.order_id),
        }
    }

    fn seller_dispute(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = order_detail_href(&context.order_id);
        Self {
            scene: NotificationScene::DisputeEscalated,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "订单进入争议处理，请准备后续材料",
                "headline": format!(
                    "订单 {} 已被提交争议，请先在订单详情核对原因并准备处理材料。",
                    context.order_id
                ),
                "action_summary": "先查看订单详情确认争议原因与当前状态，后续如需补充材料再进入争议页。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "show_ops_context": false,
                "action_label": "查看订单详情",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "dispute_escalated_seller_visible",
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: context.dispute_subject_refs(),
            links: seller_dispute_links(&context.order_id),
        }
    }

    fn ops_dispute(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = risk_console_href(&context.order_id, Some(&context.case_id));
        Self {
            scene: NotificationScene::DisputeEscalated,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "争议升级（运营联查）",
                "headline": format!(
                    "订单 {} 已进入争议处理，请核对冻结、保全和治理动作是否齐备。",
                    context.order_id
                ),
                "action_summary": "进入风控工作台和审计联查页确认冻结票据、legal hold 与治理动作都已落盘。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "governance_action_count": context.governance_action_count,
                "settlement_freeze_count": context.settlement_freeze_count,
                "show_ops_context": true,
                "action_label": "查看风控工作台",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "dispute_escalated_ops_visible",
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "governance_action_count": context.governance_action_count,
                "settlement_freeze_count": context.settlement_freeze_count,
            }),
            subject_refs: context.dispute_subject_refs(),
            links: ops_governance_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn buyer_settlement_frozen(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = billing_center_href(&context.order_id);
        Self {
            scene: NotificationScene::SettlementFrozen,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "结算已冻结，等待争议处理结果",
                "headline": format!(
                    "订单 {} 的待结算金额已被冻结，请关注争议处理进度。",
                    context.order_id
                ),
                "action_summary": "进入账单中心查看当前冻结状态，必要时继续在争议页补充材料。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "show_ops_context": false,
                "action_label": "查看账单中心",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_frozen_buyer_visible",
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: context.settlement_subject_refs(source_event),
            links: settlement_party_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn seller_settlement_frozen(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = billing_center_href(&context.order_id);
        Self {
            scene: NotificationScene::SettlementFrozen,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "结算已冻结，请关注争议处理",
                "headline": format!(
                    "订单 {} 的结算已被冻结，请在账单中心关注冻结与后续处理结果。",
                    context.order_id
                ),
                "action_summary": "进入账单中心确认冻结状态，并准备在争议处理过程中配合补充材料。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "show_ops_context": false,
                "action_label": "查看账单中心",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_frozen_seller_visible",
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
            }),
            subject_refs: context.settlement_subject_refs(source_event),
            links: settlement_party_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn ops_settlement_frozen(
        context: &DisputeLifecycleNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = risk_console_href(&context.order_id, Some(&context.case_id));
        Self {
            scene: NotificationScene::SettlementFrozen,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "结算已冻结（运营联查）",
                "headline": format!(
                    "订单 {} 已落冻结账单事实，请联查冻结票据、保全状态与治理动作。",
                    context.order_id
                ),
                "action_summary": "进入风控工作台和审计联查页核对冻结事实、治理动作和保全过程。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "governance_action_count": context.governance_action_count,
                "settlement_freeze_count": context.settlement_freeze_count,
                "hold_billing_event_id": source_event.aggregate_id,
                "show_ops_context": true,
                "action_label": "查看风控工作台",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_frozen_ops_visible",
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "case_status": context.case_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "governance_action_count": context.governance_action_count,
                "settlement_freeze_count": context.settlement_freeze_count,
                "hold_billing_event_id": source_event.aggregate_id,
            }),
            subject_refs: context.settlement_subject_refs(source_event),
            links: ops_governance_links(&context.order_id, Some(&context.case_id)),
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

impl DisputeLifecycleNotificationContext {
    fn dispute_subject_refs(&self) -> Vec<NotificationSubjectRef> {
        vec![
            NotificationSubjectRef {
                ref_type: "order".to_string(),
                ref_id: self.order_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "product".to_string(),
                ref_id: self.product_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "dispute_case".to_string(),
                ref_id: self.case_id.clone(),
            },
        ]
    }

    fn settlement_subject_refs(
        &self,
        source_event: &NotificationSourceEvent,
    ) -> Vec<NotificationSubjectRef> {
        let mut refs = self.dispute_subject_refs();
        refs.push(NotificationSubjectRef {
            ref_type: "billing_event".to_string(),
            ref_id: source_event.aggregate_id.clone(),
        });
        refs
    }
}

async fn load_dispute_lifecycle_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    case_id: &str,
) -> Result<DisputeLifecycleNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               dc.case_id::text,
               dc.reason_code,
               dc.status,
               (
                 SELECT freeze_ticket_id::text
                 FROM risk.freeze_ticket
                 WHERE ref_type = 'order'
                   AND ref_id = o.order_id
                 ORDER BY created_at DESC, freeze_ticket_id DESC
                 LIMIT 1
               ),
               (
                 SELECT legal_hold_id::text
                 FROM audit.legal_hold
                 WHERE hold_scope_type = 'order'
                   AND hold_scope_id = o.order_id
                 ORDER BY created_at DESC, legal_hold_id DESC
                 LIMIT 1
               ),
               (
                 SELECT COUNT(*)::bigint
                 FROM risk.governance_action_log g
                 JOIN risk.freeze_ticket f ON f.freeze_ticket_id = g.freeze_ticket_id
                 WHERE f.ref_type = 'order'
                   AND f.ref_id = o.order_id
               ),
               (
                 SELECT COUNT(*)::bigint
                 FROM billing.settlement_record sr
                 WHERE sr.order_id = o.order_id
                   AND sr.settlement_status = 'frozen'
               )
             FROM support.dispute_case dc
             JOIN trade.order_main o ON o.order_id = dc.order_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             WHERE dc.case_id = $1::text::uuid
               AND o.order_id = $2::text::uuid",
            &[&case_id, &order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing dispute notification context for order={order_id} case={case_id}"
        )));
    };

    Ok(DisputeLifecycleNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        acceptance_status: row.get(10),
        settlement_status: row.get(11),
        dispute_status: row.get(12),
        buyer_org_id: row.get(13),
        buyer_org_name: row.get(14),
        seller_org_id: row.get(15),
        seller_org_name: row.get(16),
        case_id: row.get(17),
        reason_code: row.get(18),
        case_status: row.get(19),
        freeze_ticket_id: row.get(20),
        legal_hold_id: row.get(21),
        governance_action_count: row.get(22),
        settlement_freeze_count: row.get(23),
    })
}

pub struct SettlementResumeNotificationDispatchInput<'a> {
    pub order_id: &'a str,
    pub billing_event_id: &'a str,
    pub occurred_at: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SettlementResumeNotificationDispatchResult {
    pub inserted_count: usize,
    pub replayed_count: usize,
    pub idempotency_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct SettlementResumeNotificationContext {
    order_id: String,
    product_id: String,
    product_title: String,
    sku_code: String,
    sku_type: String,
    order_amount: String,
    currency_code: String,
    order_status: String,
    payment_status: String,
    delivery_status: String,
    acceptance_status: String,
    settlement_status: String,
    dispute_status: String,
    buyer_org_id: String,
    buyer_org_name: String,
    seller_org_id: String,
    seller_org_name: String,
    release_amount: String,
    release_currency_code: String,
    billing_event_source: String,
    case_id: Option<String>,
    reason_code: Option<String>,
    decision_code: Option<String>,
    penalty_code: Option<String>,
    liability_type: Option<String>,
    resolution_action: Option<String>,
    resolution_ref_id: Option<String>,
    freeze_ticket_id: Option<String>,
    legal_hold_id: Option<String>,
}

#[derive(Debug, Clone)]
struct SettlementResumeNotificationDispatch {
    scene: NotificationScene,
    audience: NotificationAudience,
    recipient: NotificationRecipient,
    source_event: NotificationSourceEvent,
    variables: Value,
    metadata: Value,
    subject_refs: Vec<NotificationSubjectRef>,
    links: Vec<NotificationActionLink>,
}

pub async fn queue_settlement_resume_notifications(
    client: &(impl GenericClient + Sync),
    input: SettlementResumeNotificationDispatchInput<'_>,
) -> Result<SettlementResumeNotificationDispatchResult, Error> {
    let context =
        load_settlement_resume_notification_context(client, input.order_id, input.billing_event_id)
            .await?;
    let buyer_recipient = load_org_recipient(
        client,
        &context.buyer_org_id,
        &context.buyer_org_name,
        &["buyer_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let seller_recipient = load_org_recipient(
        client,
        &context.seller_org_id,
        &context.seller_org_name,
        &["seller_operator", "tenant_admin", "tenant_operator"],
    )
    .await?;
    let ops_recipient = load_ops_recipient(client).await?;
    let source_event = NotificationSourceEvent {
        aggregate_type: "billing.billing_event".to_string(),
        aggregate_id: input.billing_event_id.to_string(),
        event_type: "billing.event.recorded".to_string(),
        event_id: None,
        target_topic: Some("dtp.outbox.domain-events".to_string()),
        occurred_at: input.occurred_at.map(str::to_string),
    };
    let dispatches = vec![
        SettlementResumeNotificationDispatch::buyer(&context, &buyer_recipient, &source_event),
        SettlementResumeNotificationDispatch::seller(&context, &seller_recipient, &source_event),
        SettlementResumeNotificationDispatch::ops(&context, &ops_recipient, &source_event),
    ];

    let mut result = SettlementResumeNotificationDispatchResult::default();
    for dispatch in dispatches {
        let aggregate_id = dispatch.source_event.aggregate_id.clone();
        let (payload, idempotency_key) = prepare_notification_request(dispatch.build_input());
        let inserted = queue_notification_request(
            client,
            QueueNotificationRequest {
                aggregate_id: &aggregate_id,
                payload: &payload,
                idempotency_key: &idempotency_key,
                request_id: input.request_id,
                trace_id: input.trace_id,
            },
        )
        .await?;
        if inserted {
            result.inserted_count += 1;
        } else {
            result.replayed_count += 1;
        }
        result.idempotency_keys.push(idempotency_key);
    }

    Ok(result)
}

impl SettlementResumeNotificationDispatch {
    fn buyer(
        context: &SettlementResumeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = settlement_resume_href(&context.order_id, context.case_id.as_deref());
        Self {
            scene: NotificationScene::SettlementResumed,
            audience: NotificationAudience::Buyer,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "结算已恢复处理",
                "headline": format!(
                    "订单 {} 的争议冻结已解除，可回到账单页查看后续结算或退款处理。",
                    context.order_id
                ),
                "action_summary": "进入账单页查看恢复后的结算状态，并关注后续退款、赔付或放款结果。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "release_amount": context.release_amount,
                "release_currency_code": context.release_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "show_ops_context": false,
                "action_label": "查看账单页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_resumed_buyer_visible",
                "recipient_scope": "buyer_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "billing_event_source": context.billing_event_source,
            }),
            subject_refs: context.subject_refs(source_event),
            links: settlement_party_links(&context.order_id, context.case_id.as_deref()),
        }
    }

    fn seller(
        context: &SettlementResumeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = settlement_resume_href(&context.order_id, context.case_id.as_deref());
        Self {
            scene: NotificationScene::SettlementResumed,
            audience: NotificationAudience::Seller,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "结算已恢复，请继续关注处理结果",
                "headline": format!(
                    "订单 {} 的冻结结算已解除，请回到账单页确认后续结算或退款结果。",
                    context.order_id
                ),
                "action_summary": "进入账单页查看恢复后的结算状态和案件处理结果，准备配合后续执行。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "release_amount": context.release_amount,
                "release_currency_code": context.release_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "show_ops_context": false,
                "action_label": "查看账单页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_resumed_seller_visible",
                "recipient_scope": "seller_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "billing_event_source": context.billing_event_source,
            }),
            subject_refs: context.subject_refs(source_event),
            links: settlement_party_links(&context.order_id, context.case_id.as_deref()),
        }
    }

    fn ops(
        context: &SettlementResumeNotificationContext,
        recipient: &NotificationRecipient,
        source_event: &NotificationSourceEvent,
    ) -> Self {
        let action_href = audit_trace_href(&context.order_id, context.case_id.as_deref());
        Self {
            scene: NotificationScene::SettlementResumed,
            audience: NotificationAudience::Ops,
            recipient: recipient.clone(),
            source_event: source_event.clone(),
            variables: json!({
                "subject": "恢复结算（运营联查）",
                "headline": format!(
                    "订单 {} 已写入恢复结算事实，请联查释放事件和后续处理结果。",
                    context.order_id
                ),
                "action_summary": "进入审计联查页核对释放账单事件，并在账单页确认后续执行是否按预期推进。".to_string(),
                "order_id": context.order_id,
                "product_title": context.product_title,
                "buyer_org_name": context.buyer_org_name,
                "seller_org_name": context.seller_org_name,
                "order_amount": context.order_amount,
                "currency_code": context.currency_code,
                "release_amount": context.release_amount,
                "release_currency_code": context.release_currency_code,
                "payment_status": context.payment_status,
                "delivery_status": context.delivery_status,
                "acceptance_status": context.acceptance_status,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "penalty_code": context.penalty_code,
                "liability_type": context.liability_type,
                "resolution_action": context.resolution_action,
                "resolution_ref_id": context.resolution_ref_id,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "billing_event_source": context.billing_event_source,
                "show_ops_context": true,
                "action_label": "查看审计联查页",
                "action_href": action_href,
            }),
            metadata: json!({
                "task_id": "NOTIF-007",
                "transition_code": "settlement_resumed_ops_visible",
                "recipient_scope": "ops_visible",
                "order_status": context.order_status,
                "sku_code": context.sku_code,
                "sku_type": context.sku_type,
                "case_id": context.case_id,
                "reason_code": context.reason_code,
                "decision_code": context.decision_code,
                "penalty_code": context.penalty_code,
                "liability_type": context.liability_type,
                "settlement_status": context.settlement_status,
                "dispute_status": context.dispute_status,
                "resolution_action": context.resolution_action,
                "resolution_ref_id": context.resolution_ref_id,
                "freeze_ticket_id": context.freeze_ticket_id,
                "legal_hold_id": context.legal_hold_id,
                "billing_event_source": context.billing_event_source,
            }),
            subject_refs: context.subject_refs(source_event),
            links: settlement_resume_ops_links(&context.order_id, context.case_id.as_deref()),
        }
    }

    fn build_input(self) -> BuildNotificationRequestInput {
        BuildNotificationRequestInput {
            scene: self.scene,
            audience: self.audience,
            recipient: self.recipient,
            source_event: self.source_event,
            variables: self.variables,
            metadata: self.metadata,
            retry_policy: None,
            subject_refs: self.subject_refs,
            links: self.links,
            template_code: None,
            channel: None,
        }
    }
}

impl SettlementResumeNotificationContext {
    fn subject_refs(&self, source_event: &NotificationSourceEvent) -> Vec<NotificationSubjectRef> {
        let mut refs = vec![
            NotificationSubjectRef {
                ref_type: "order".to_string(),
                ref_id: self.order_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "product".to_string(),
                ref_id: self.product_id.clone(),
            },
            NotificationSubjectRef {
                ref_type: "billing_event".to_string(),
                ref_id: source_event.aggregate_id.clone(),
            },
        ];
        if let Some(case_id) = self.case_id.as_ref() {
            refs.push(NotificationSubjectRef {
                ref_type: "dispute_case".to_string(),
                ref_id: case_id.clone(),
            });
        }
        refs
    }
}

async fn load_settlement_resume_notification_context(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    billing_event_id: &str,
) -> Result<SettlementResumeNotificationContext, Error> {
    let row = client
        .query_opt(
            "SELECT
               o.order_id::text,
               o.product_id::text,
               p.title,
               COALESCE(sku.sku_code, ''),
               COALESCE(sku.sku_type, ''),
               o.amount::text,
               o.currency_code,
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               buyer.org_id::text,
               buyer.org_name,
               seller.org_id::text,
               seller.org_name,
               be.amount::text,
               be.currency_code,
               be.event_source,
               COALESCE(be.metadata, '{}'::jsonb),
               dc.case_id::text,
               dc.reason_code,
               dc.decision_code,
               dc.penalty_code,
               dr.liability_type,
               (
                 SELECT freeze_ticket_id::text
                 FROM risk.freeze_ticket
                 WHERE ref_type = 'order'
                   AND ref_id = o.order_id
                 ORDER BY created_at DESC, freeze_ticket_id DESC
                 LIMIT 1
               ),
               (
                 SELECT legal_hold_id::text
                 FROM audit.legal_hold
                 WHERE hold_scope_type = 'order'
                   AND hold_scope_id = o.order_id
                 ORDER BY created_at DESC, legal_hold_id DESC
                 LIMIT 1
               )
             FROM billing.billing_event be
             JOIN trade.order_main o ON o.order_id = be.order_id
             JOIN catalog.product p ON p.product_id = o.product_id
             JOIN catalog.product_sku sku ON sku.sku_id = o.sku_id
             JOIN core.organization buyer ON buyer.org_id = o.buyer_org_id
             JOIN core.organization seller ON seller.org_id = o.seller_org_id
             LEFT JOIN LATERAL (
               SELECT case_id, reason_code, decision_code, penalty_code
               FROM support.dispute_case
               WHERE order_id = o.order_id
               ORDER BY COALESCE(resolved_at, updated_at) DESC, updated_at DESC, case_id DESC
               LIMIT 1
             ) dc ON TRUE
             LEFT JOIN LATERAL (
               SELECT liability_type
               FROM support.decision_record
               WHERE case_id = dc.case_id
               ORDER BY decided_at DESC, decision_id DESC
               LIMIT 1
             ) dr ON TRUE
             WHERE be.billing_event_id = $1::text::uuid
               AND o.order_id = $2::text::uuid",
            &[&billing_event_id, &order_id],
        )
        .await?;
    let Some(row) = row else {
        return Err(Error::Bind(format!(
            "missing settlement resume notification context for order={order_id} billing_event={billing_event_id}"
        )));
    };
    let metadata: Value = row.get(20);

    Ok(SettlementResumeNotificationContext {
        order_id: row.get(0),
        product_id: row.get(1),
        product_title: row.get(2),
        sku_code: row.get(3),
        sku_type: row.get(4),
        order_amount: row.get(5),
        currency_code: row.get(6),
        order_status: row.get(7),
        payment_status: row.get(8),
        delivery_status: row.get(9),
        acceptance_status: row.get(10),
        settlement_status: row.get(11),
        dispute_status: row.get(12),
        buyer_org_id: row.get(13),
        buyer_org_name: row.get(14),
        seller_org_id: row.get(15),
        seller_org_name: row.get(16),
        release_amount: row.get(17),
        release_currency_code: row.get(18),
        billing_event_source: row.get(19),
        case_id: row.get(21),
        reason_code: row.get(22),
        decision_code: row.get(23),
        penalty_code: row.get(24),
        liability_type: row.get(25),
        resolution_action: json_text(&metadata, "resolution_action"),
        resolution_ref_id: json_text(&metadata, "resolution_ref_id"),
        freeze_ticket_id: row.get(26),
        legal_hold_id: row.get(27),
    })
}

fn json_text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn billing_resolution_links(order_id: &str, case_id: Option<&str>) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "billing_resolution".to_string(),
            href: billing_resolution_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
        NotificationActionLink {
            link_code: "order_detail".to_string(),
            href: order_detail_href(order_id),
        },
        NotificationActionLink {
            link_code: "dispute_create".to_string(),
            href: dispute_create_href(order_id),
        },
    ]
}

fn dispute_case_links(order_id: &str) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "dispute_create".to_string(),
            href: dispute_create_href(order_id),
        },
        NotificationActionLink {
            link_code: "order_detail".to_string(),
            href: order_detail_href(order_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
    ]
}

fn seller_dispute_links(order_id: &str) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "order_detail".to_string(),
            href: order_detail_href(order_id),
        },
        NotificationActionLink {
            link_code: "dispute_create".to_string(),
            href: dispute_create_href(order_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
    ]
}

fn settlement_party_links(order_id: &str, case_id: Option<&str>) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "billing_resolution".to_string(),
            href: settlement_resume_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
        NotificationActionLink {
            link_code: "order_detail".to_string(),
            href: order_detail_href(order_id),
        },
        NotificationActionLink {
            link_code: "dispute_create".to_string(),
            href: dispute_create_href(order_id),
        },
    ]
}

fn ops_governance_links(order_id: &str, case_id: Option<&str>) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "risk_console".to_string(),
            href: risk_console_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "audit_trace".to_string(),
            href: audit_trace_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
        NotificationActionLink {
            link_code: "order_detail".to_string(),
            href: order_detail_href(order_id),
        },
    ]
}

fn settlement_resume_ops_links(
    order_id: &str,
    case_id: Option<&str>,
) -> Vec<NotificationActionLink> {
    vec![
        NotificationActionLink {
            link_code: "audit_trace".to_string(),
            href: audit_trace_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "billing_resolution".to_string(),
            href: settlement_resume_href(order_id, case_id),
        },
        NotificationActionLink {
            link_code: "billing_center".to_string(),
            href: billing_center_href(order_id),
        },
        NotificationActionLink {
            link_code: "risk_console".to_string(),
            href: risk_console_href(order_id, case_id),
        },
    ]
}

#[derive(Debug, Clone)]
struct DeliveryAction {
    label: String,
    href: String,
    link_code: String,
}

fn requires_manual_acceptance_follow_up(acceptance_status: &str) -> bool {
    matches!(acceptance_status, "pending_acceptance" | "in_progress")
}

fn normalize_delivery_field(override_value: Option<&str>, fallback: &str) -> String {
    override_value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .trim()
        .to_string()
}

fn delivery_branch_label(delivery_branch: &str) -> &'static str {
    match delivery_branch {
        "file" => "文件包交付",
        "share" => "共享开通",
        "api" => "API 开通",
        "query_run" => "查询结果可取",
        "sandbox" => "沙箱开通",
        "report" => "报告交付",
        _ => "交付结果",
    }
}

fn buyer_delivery_subject(delivery_branch: &str, manual_acceptance: bool) -> String {
    let branch_label = delivery_branch_label(delivery_branch);
    if manual_acceptance {
        format!("{branch_label}已完成，请验收")
    } else {
        format!("{branch_label}已完成")
    }
}

fn buyer_delivery_action(
    delivery_branch: &str,
    order_id: &str,
    manual_acceptance: bool,
) -> DeliveryAction {
    if manual_acceptance {
        return DeliveryAction {
            label: "进入验收页".to_string(),
            href: format!("/portal/orders/{order_id}/acceptance"),
            link_code: "acceptance_page".to_string(),
        };
    }

    match delivery_branch {
        "api" => DeliveryAction {
            label: "查看 API 开通详情".to_string(),
            href: format!("/portal/orders/{order_id}"),
            link_code: "order_detail".to_string(),
        },
        "query_run" => DeliveryAction {
            label: "查看查询结果".to_string(),
            href: format!("/portal/orders/{order_id}"),
            link_code: "order_detail".to_string(),
        },
        "sandbox" => DeliveryAction {
            label: "进入工作区".to_string(),
            href: format!("/portal/orders/{order_id}"),
            link_code: "order_detail".to_string(),
        },
        _ => DeliveryAction {
            label: "查看订单详情".to_string(),
            href: format!("/portal/orders/{order_id}"),
            link_code: "order_detail".to_string(),
        },
    }
}

fn seller_delivery_action(order_id: &str) -> DeliveryAction {
    DeliveryAction {
        label: "查看交付工作台".to_string(),
        href: format!("/portal/orders/{order_id}/deliveries"),
        link_code: "delivery_console".to_string(),
    }
}

fn ops_delivery_action(order_id: &str) -> DeliveryAction {
    DeliveryAction {
        label: "查看运营联查页".to_string(),
        href: format!("/ops/orders/{order_id}"),
        link_code: "ops_order_detail".to_string(),
    }
}

fn order_state_label(order_status: &str) -> &'static str {
    match order_status {
        "delivered" => "已交付",
        "report_delivered" => "报告已交付",
        "api_key_issued" => "API Key 已开通",
        "quota_ready" => "配额已就绪",
        "share_granted" => "共享已开通",
        "query_executed" => "查询已执行",
        "result_available" => "结果可获取",
        "seat_issued" => "沙箱席位已开通",
        other if other.is_empty() => "未知",
        _ => "交付处理中",
    }
}

fn acceptance_status_label(acceptance_status: &str) -> &'static str {
    match acceptance_status {
        "pending_acceptance" => "待验收",
        "in_progress" => "验收中",
        "accepted" => "已验收",
        "not_started" => "无需手工验收",
        "closed" => "已关闭",
        "rejected" => "已拒收",
        other if other.is_empty() => "未知",
        _ => "状态更新中",
    }
}

fn order_detail_href(order_id: &str) -> String {
    format!("/trade/orders/{order_id}")
}

fn billing_center_href(order_id: &str) -> String {
    format!("/billing?order_id={order_id}")
}

fn billing_resolution_href(order_id: &str, case_id: Option<&str>) -> String {
    match case_id {
        Some(case_id) if !case_id.trim().is_empty() => {
            format!("/billing/refunds?order_id={order_id}&case_id={case_id}")
        }
        _ => format!("/billing/refunds?order_id={order_id}"),
    }
}

fn settlement_resume_href(order_id: &str, case_id: Option<&str>) -> String {
    billing_resolution_href(order_id, case_id)
}

fn dispute_create_href(order_id: &str) -> String {
    format!("/support/cases/new?order_id={order_id}")
}

fn risk_console_href(order_id: &str, case_id: Option<&str>) -> String {
    match case_id {
        Some(case_id) if !case_id.trim().is_empty() => {
            format!("/ops/risk?order_id={order_id}&case_id={case_id}")
        }
        _ => format!("/ops/risk?order_id={order_id}"),
    }
}

fn audit_trace_href(order_id: &str, case_id: Option<&str>) -> String {
    match case_id {
        Some(case_id) if !case_id.trim().is_empty() => {
            format!("/ops/audit/trace?order_id={order_id}&case_id={case_id}")
        }
        _ => format!("/ops/audit/trace?order_id={order_id}"),
    }
}

async fn load_org_recipient(
    client: &(impl GenericClient + Sync),
    org_id: &str,
    org_name: &str,
    preferred_personas: &[&str],
) -> Result<NotificationRecipient, Error> {
    let rows = client
        .query(
            "SELECT
               user_id::text,
               COALESCE(NULLIF(email::text, ''), login_id::text),
               display_name,
               attrs ->> 'persona'
             FROM core.user_account
             WHERE org_id = $1::text::uuid
               AND status = 'active'
             ORDER BY created_at ASC, user_id ASC",
            &[&org_id],
        )
        .await?;
    let candidates = rows
        .into_iter()
        .filter_map(|row| {
            let address: String = row.get(1);
            if address.trim().is_empty() {
                return None;
            }
            Some(NotificationRecipientCandidate {
                user_id: row.get(0),
                address,
                display_name: row.get(2),
                persona: row.get(3),
            })
        })
        .collect::<Vec<_>>();
    if let Some(candidate) = choose_candidate(candidates, preferred_personas) {
        return Ok(NotificationRecipient {
            kind: "user".to_string(),
            address: candidate.address,
            id: Some(candidate.user_id),
            display_name: Some(candidate.display_name),
        });
    }

    Ok(NotificationRecipient {
        kind: "org".to_string(),
        address: format!("org:{org_id}"),
        id: Some(org_id.to_string()),
        display_name: Some(org_name.to_string()),
    })
}

async fn load_ops_recipient(
    client: &(impl GenericClient + Sync),
) -> Result<NotificationRecipient, Error> {
    let rows = client
        .query(
            "SELECT
               u.user_id::text,
               COALESCE(NULLIF(u.email::text, ''), u.login_id::text),
               u.display_name,
               u.attrs ->> 'persona',
               o.org_id::text,
               o.org_name
             FROM core.user_account u
             JOIN core.organization o ON o.org_id = u.org_id
             WHERE u.status = 'active'
               AND o.status = 'active'
               AND o.org_type = 'platform'
             ORDER BY u.created_at ASC, u.user_id ASC",
            &[],
        )
        .await?;
    let mut fallback_org: Option<(String, String)> = None;
    let candidates = rows
        .into_iter()
        .filter_map(|row| {
            let org_id: String = row.get(4);
            let org_name: String = row.get(5);
            if fallback_org.is_none() {
                fallback_org = Some((org_id.clone(), org_name.clone()));
            }
            let address: String = row.get(1);
            if address.trim().is_empty() {
                return None;
            }
            Some(NotificationRecipientCandidate {
                user_id: row.get(0),
                address,
                display_name: row.get(2),
                persona: row.get(3),
            })
        })
        .collect::<Vec<_>>();
    if let Some(candidate) = choose_candidate(
        candidates,
        &[
            "platform_admin",
            "platform_risk_settlement",
            "platform_finance_operator",
            "platform_audit_security",
        ],
    ) {
        return Ok(NotificationRecipient {
            kind: "user".to_string(),
            address: candidate.address,
            id: Some(candidate.user_id),
            display_name: Some(candidate.display_name),
        });
    }

    if let Some((org_id, org_name)) = fallback_org {
        return Ok(NotificationRecipient {
            kind: "org".to_string(),
            address: format!("org:{org_id}"),
            id: Some(org_id),
            display_name: Some(org_name),
        });
    }

    Err(Error::Bind(
        "missing platform organization recipient for notification dispatch".to_string(),
    ))
}

fn choose_candidate(
    candidates: Vec<NotificationRecipientCandidate>,
    preferred_personas: &[&str],
) -> Option<NotificationRecipientCandidate> {
    candidates.into_iter().min_by_key(|candidate| {
        candidate
            .persona
            .as_deref()
            .and_then(|persona| {
                preferred_personas
                    .iter()
                    .position(|preferred| preferred == &persona)
            })
            .unwrap_or(preferred_personas.len())
    })
}
