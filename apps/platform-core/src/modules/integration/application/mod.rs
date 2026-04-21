use crate::modules::integration::events::{
    BuildNotificationRequestInput, NotificationActionLink, NotificationAudience,
    NotificationRecipient, NotificationRequestedPayload, NotificationScene,
    NotificationSourceEvent, NotificationSubjectRef, build_notification_idempotency_key,
    build_notification_request_payload,
};
use crate::shared::outbox::{CanonicalOutboxWrite, write_canonical_outbox_event};
use db::{Error, GenericClient};
use serde_json::{Value, json};

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
