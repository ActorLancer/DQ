UPDATE ops.notification_template
SET
  enabled = FALSE,
  status = 'archived',
  metadata = metadata
    || jsonb_build_object(
      'archived_by_task', 'NOTIF-005',
      'replaced_by_version', 2
    ),
  updated_at = now()
WHERE template_code IN (
    'NOTIFY_DELIVERY_COMPLETED_V1',
    'NOTIFY_PENDING_ACCEPTANCE_V1'
  )
  AND language_code = 'zh-CN'
  AND channel = 'mock-log'
  AND version_no < 2
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
    'NOTIFY_DELIVERY_COMPLETED_V1',
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
交付：{{variables.delivery_branch_label}} / {{variables.delivery_type}} / {{variables.delivery_route}}
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status_label}}
{{#if variables.show_ops_context}}
联查：delivery_ref={{variables.delivery_ref_type}}/{{variables.delivery_ref_id}} receipt={{variables.receipt_hash}} commit={{variables.delivery_commit_hash}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=delivery.completed order={{variables.order_id}} state={{variables.current_state}} acceptance={{variables.acceptance_status}}',
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
        "acceptance_status",
        "current_state",
        "current_state_label",
        "acceptance_status_label",
        "delivery_branch_label",
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
        "acceptance_status": {"type":"string"},
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "acceptance_status_label": {"type":"string"},
        "delivery_branch_label": {"type":"string"},
        "delivery_type": {"type":"string"},
        "delivery_route": {"type":"string"},
        "show_ops_context": {"type":"boolean"},
        "delivery_ref_type": {"type":"string"},
        "delivery_ref_id": {"type":"string"},
        "receipt_hash": {"type":"string"},
        "delivery_commit_hash": {"type":"string"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-005",
      "notification_code":"delivery.completed",
      "audience_scope":"buyer_seller_ops_delivery_completion",
      "version_note":"delivery-completed-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_PENDING_ACCEPTANCE_V1',
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
交付：{{variables.delivery_branch_label}} / {{variables.delivery_type}} / {{variables.delivery_route}}
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status_label}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=order.pending_acceptance order={{variables.order_id}} state={{variables.current_state}} acceptance={{variables.acceptance_status}}',
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
        "acceptance_status",
        "current_state",
        "current_state_label",
        "acceptance_status_label",
        "delivery_branch_label",
        "delivery_type",
        "delivery_route",
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
        "acceptance_status": {"type":"string"},
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "acceptance_status_label": {"type":"string"},
        "delivery_branch_label": {"type":"string"},
        "delivery_type": {"type":"string"},
        "delivery_route": {"type":"string"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-005",
      "notification_code":"order.pending_acceptance",
      "audience_scope":"buyer_pending_acceptance_only",
      "version_note":"pending-acceptance-buyer-visible"
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
