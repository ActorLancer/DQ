UPDATE ops.notification_template
SET
  enabled = FALSE,
  status = 'archived',
  metadata = metadata
    || jsonb_build_object(
      'archived_by_task', 'NOTIF-007',
      'replaced_by_version', 2
    ),
  updated_at = now()
WHERE template_code IN (
    'NOTIFY_DISPUTE_ESCALATED_V1',
    'NOTIFY_SETTLEMENT_FROZEN_V1',
    'NOTIFY_SETTLEMENT_RESUMED_V1'
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
    'NOTIFY_DISPUTE_ESCALATED_V1',
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
状态：支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status}}，结算={{variables.settlement_status}}，争议={{variables.dispute_status}}
案件：{{variables.case_id}} / {{variables.reason_code}}
处理建议：{{variables.action_summary}}
{{#if variables.show_ops_context}}
联查：freeze_ticket={{variables.freeze_ticket_id}} legal_hold={{variables.legal_hold_id}} governance_actions={{variables.governance_action_count}} settlement_freeze_count={{variables.settlement_freeze_count}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=dispute.escalated order={{variables.order_id}} case={{variables.case_id}}',
    '{
      "type": "object",
      "required": [
        "subject",
        "headline",
        "action_summary",
        "order_id",
        "product_title",
        "buyer_org_name",
        "seller_org_name",
        "order_amount",
        "currency_code",
        "payment_status",
        "delivery_status",
        "acceptance_status",
        "settlement_status",
        "dispute_status",
        "case_id",
        "reason_code",
        "show_ops_context",
        "action_label",
        "action_href"
      ],
      "properties": {
        "subject": {"type":"string"},
        "headline": {"type":"string"},
        "action_summary": {"type":"string"},
        "order_id": {"type":"string"},
        "product_title": {"type":"string"},
        "buyer_org_name": {"type":"string"},
        "seller_org_name": {"type":"string"},
        "order_amount": {"type":"string"},
        "currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "acceptance_status": {"type":"string"},
        "settlement_status": {"type":"string"},
        "dispute_status": {"type":"string"},
        "case_id": {"type":"string"},
        "reason_code": {"type":"string"},
        "freeze_ticket_id": {"type":["string","null"]},
        "legal_hold_id": {"type":["string","null"]},
        "governance_action_count": {"type":["integer","null"]},
        "settlement_freeze_count": {"type":["integer","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-007",
      "notification_code":"dispute.escalated",
      "audience_scope":"buyer_seller_ops_dispute_lifecycle",
      "version_note":"dispute-escalated-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_SETTLEMENT_FROZEN_V1',
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
状态：支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status}}，结算={{variables.settlement_status}}，争议={{variables.dispute_status}}
案件：{{variables.case_id}} / {{variables.reason_code}}
处理建议：{{variables.action_summary}}
{{#if variables.show_ops_context}}
联查：freeze_ticket={{variables.freeze_ticket_id}} legal_hold={{variables.legal_hold_id}} governance_actions={{variables.governance_action_count}} settlement_freeze_count={{variables.settlement_freeze_count}} hold_event={{variables.hold_billing_event_id}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=settlement.frozen order={{variables.order_id}} case={{variables.case_id}}',
    '{
      "type": "object",
      "required": [
        "subject",
        "headline",
        "action_summary",
        "order_id",
        "product_title",
        "buyer_org_name",
        "seller_org_name",
        "order_amount",
        "currency_code",
        "payment_status",
        "delivery_status",
        "acceptance_status",
        "settlement_status",
        "dispute_status",
        "case_id",
        "reason_code",
        "show_ops_context",
        "action_label",
        "action_href"
      ],
      "properties": {
        "subject": {"type":"string"},
        "headline": {"type":"string"},
        "action_summary": {"type":"string"},
        "order_id": {"type":"string"},
        "product_title": {"type":"string"},
        "buyer_org_name": {"type":"string"},
        "seller_org_name": {"type":"string"},
        "order_amount": {"type":"string"},
        "currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "acceptance_status": {"type":"string"},
        "settlement_status": {"type":"string"},
        "dispute_status": {"type":"string"},
        "case_id": {"type":"string"},
        "reason_code": {"type":"string"},
        "freeze_ticket_id": {"type":["string","null"]},
        "legal_hold_id": {"type":["string","null"]},
        "governance_action_count": {"type":["integer","null"]},
        "settlement_freeze_count": {"type":["integer","null"]},
        "hold_billing_event_id": {"type":["string","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-007",
      "notification_code":"settlement.frozen",
      "audience_scope":"buyer_seller_ops_dispute_lifecycle",
      "version_note":"settlement-frozen-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_SETTLEMENT_RESUMED_V1',
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
订单金额：{{variables.currency_code}} {{variables.order_amount}}
释放金额：{{variables.release_currency_code}} {{variables.release_amount}}
状态：支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status}}，结算={{variables.settlement_status}}，争议={{variables.dispute_status}}
{{#if variables.case_id}}
案件：{{variables.case_id}}{{#if variables.decision_code}} / {{variables.decision_code}}{{/if}}
{{/if}}
{{#if variables.reason_code}}
原因：{{variables.reason_code}}
{{/if}}
处理建议：{{variables.action_summary}}
{{#if variables.show_ops_context}}
联查：event_source={{variables.billing_event_source}} resolution={{variables.resolution_action}}/{{variables.resolution_ref_id}} freeze_ticket={{variables.freeze_ticket_id}} legal_hold={{variables.legal_hold_id}}{{#if variables.liability_type}} liability={{variables.liability_type}}{{/if}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=settlement.resumed order={{variables.order_id}} event={{variables.billing_event_source}}',
    '{
      "type": "object",
      "required": [
        "subject",
        "headline",
        "action_summary",
        "order_id",
        "product_title",
        "buyer_org_name",
        "seller_org_name",
        "order_amount",
        "currency_code",
        "release_amount",
        "release_currency_code",
        "payment_status",
        "delivery_status",
        "acceptance_status",
        "settlement_status",
        "dispute_status",
        "billing_event_source",
        "show_ops_context",
        "action_label",
        "action_href"
      ],
      "properties": {
        "subject": {"type":"string"},
        "headline": {"type":"string"},
        "action_summary": {"type":"string"},
        "order_id": {"type":"string"},
        "product_title": {"type":"string"},
        "buyer_org_name": {"type":"string"},
        "seller_org_name": {"type":"string"},
        "order_amount": {"type":"string"},
        "currency_code": {"type":"string"},
        "release_amount": {"type":"string"},
        "release_currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "acceptance_status": {"type":"string"},
        "settlement_status": {"type":"string"},
        "dispute_status": {"type":"string"},
        "case_id": {"type":["string","null"]},
        "reason_code": {"type":["string","null"]},
        "decision_code": {"type":["string","null"]},
        "penalty_code": {"type":["string","null"]},
        "liability_type": {"type":["string","null"]},
        "resolution_action": {"type":["string","null"]},
        "resolution_ref_id": {"type":["string","null"]},
        "freeze_ticket_id": {"type":["string","null"]},
        "legal_hold_id": {"type":["string","null"]},
        "billing_event_source": {"type":"string"},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-007",
      "notification_code":"settlement.resumed",
      "audience_scope":"buyer_seller_ops_dispute_lifecycle",
      "version_note":"settlement-resumed-visible"
    }'::jsonb
  )
ON CONFLICT (template_code, language_code, channel, version_no) DO UPDATE
SET enabled = EXCLUDED.enabled,
    status = EXCLUDED.status,
    title_template = EXCLUDED.title_template,
    body_template = EXCLUDED.body_template,
    fallback_body_template = EXCLUDED.fallback_body_template,
    variables_schema_json = EXCLUDED.variables_schema_json,
    metadata = EXCLUDED.metadata,
    updated_at = now();
