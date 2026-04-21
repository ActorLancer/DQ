UPDATE ops.notification_template
SET
  enabled = FALSE,
  status = 'archived',
  metadata = metadata
    || jsonb_build_object(
      'archived_by_task', 'NOTIF-004',
      'replaced_by_version', 2
    ),
  updated_at = now()
WHERE template_code IN (
    'NOTIFY_PAYMENT_SUCCEEDED_V1',
    'NOTIFY_PENDING_DELIVERY_V1'
  )
  AND language_code = 'zh-CN'
  AND channel = 'mock-log'
  AND enabled = TRUE
  AND status = 'active';

INSERT INTO ops.notification_template (
  template_code,
  language_code,
  channel,
  version_no,
  enabled,
  status,
  title_template,
  body_template,
  fallback_body_template,
  variables_schema_json,
  metadata
)
VALUES
  (
    'NOTIFY_PAYMENT_SUCCEEDED_V1',
    'zh-CN',
    'mock-log',
    2,
    TRUE,
    'active',
    '{{variables.subject}}',
    $$ {{variables.headline}}
商品：{{variables.product_title}}
订单：{{variables.order_id}}
金额：{{variables.currency_code}} {{variables.order_amount}}
状态：支付={{variables.payment_status}}，交付={{variables.delivery_status}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=payment.succeeded order={{variables.order_id}} status={{variables.payment_status}}/{{variables.delivery_status}}',
    '{
      "type": "object",
      "required": [
        "subject",
        "headline",
        "order_id",
        "product_title",
        "seller_org_name",
        "order_amount",
        "currency_code",
        "payment_status",
        "delivery_status",
        "action_label",
        "action_href"
      ],
      "properties": {
        "subject": {"type":"string"},
        "headline": {"type":"string"},
        "order_id": {"type":"string"},
        "product_title": {"type":"string"},
        "seller_org_name": {"type":"string"},
        "order_amount": {"type":"string"},
        "currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "buyer_locked_at": {"type":"string"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-004",
      "notification_code":"payment.succeeded",
      "audience_scope":"buyer",
      "version_note":"buyer-visible-payment-success"
    }'::jsonb
  ),
  (
    'NOTIFY_PENDING_DELIVERY_V1',
    'zh-CN',
    'mock-log',
    2,
    TRUE,
    'active',
    '{{variables.subject}}',
    $$ {{variables.headline}}
商品：{{variables.product_title}}
订单：{{variables.order_id}}
买方：{{variables.buyer_org_name}}
卖方：{{variables.seller_org_name}}
金额：{{variables.currency_code}} {{variables.order_amount}}
交付：{{variables.delivery_type}} / {{variables.delivery_route}}
状态：支付={{variables.payment_status}}，交付={{variables.delivery_status}}
{{#if variables.show_ops_context}}
联查：billing_event={{variables.billing_event_id}} payment_intent={{variables.payment_intent_id}} provider_ref={{variables.provider_reference_id}} source={{variables.provider_result_source}} provider_status={{variables.provider_status}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=order.pending_delivery order={{variables.order_id}} delivery={{variables.delivery_status}}',
    '{
      "type": "object",
      "required": [
        "subject",
        "headline",
        "order_id",
        "product_title",
        "buyer_org_name",
        "seller_org_name",
        "order_amount",
        "currency_code",
        "payment_status",
        "delivery_status",
        "delivery_type",
        "delivery_route",
        "show_ops_context",
        "action_label",
        "action_href"
      ],
      "properties": {
        "subject": {"type":"string"},
        "headline": {"type":"string"},
        "order_id": {"type":"string"},
        "product_title": {"type":"string"},
        "buyer_org_name": {"type":"string"},
        "seller_org_name": {"type":"string"},
        "order_amount": {"type":"string"},
        "currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "delivery_type": {"type":"string"},
        "delivery_route": {"type":"string"},
        "show_ops_context": {"type":"boolean"},
        "billing_event_id": {"type":"string"},
        "payment_intent_id": {"type":"string"},
        "provider_reference_id": {"type":"string"},
        "provider_result_source": {"type":"string"},
        "provider_status": {"type":"string"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-004",
      "notification_code":"order.pending_delivery",
      "audience_scope":"seller_ops_split",
      "version_note":"seller-ops-visible-pending-delivery"
    }'::jsonb
  )
ON CONFLICT (template_code, language_code, channel, version_no) DO UPDATE
SET
  enabled = EXCLUDED.enabled,
  status = EXCLUDED.status,
  title_template = EXCLUDED.title_template,
  body_template = EXCLUDED.body_template,
  fallback_body_template = EXCLUDED.fallback_body_template,
  variables_schema_json = EXCLUDED.variables_schema_json,
  metadata = EXCLUDED.metadata,
  updated_at = now();
