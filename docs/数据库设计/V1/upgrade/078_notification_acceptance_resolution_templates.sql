UPDATE ops.notification_template
SET
  enabled = FALSE,
  status = 'archived',
  metadata = metadata
    || jsonb_build_object(
      'archived_by_task', 'NOTIF-006',
      'replaced_by_version', 2
    ),
  updated_at = now()
WHERE template_code IN (
    'NOTIFY_ACCEPTANCE_PASSED_V1',
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
    'NOTIFY_REFUND_COMPLETED_V1',
    'NOTIFY_COMPENSATION_COMPLETED_V1'
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
    'NOTIFY_ACCEPTANCE_PASSED_V1',
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
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status_label}}，结算={{variables.settlement_status}}
原因：{{variables.reason_code}}
处理建议：{{variables.action_summary}}
{{#if variables.reason_detail}}
说明：{{variables.reason_detail}}
{{/if}}
{{#if variables.show_ops_context}}
联查：acceptance_record={{variables.acceptance_record_id}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=acceptance.passed order={{variables.order_id}} acceptance={{variables.acceptance_status}} settlement={{variables.settlement_status}}',
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
        "current_state",
        "current_state_label",
        "acceptance_status_label",
        "delivery_type",
        "delivery_route",
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
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "acceptance_status_label": {"type":"string"},
        "delivery_type": {"type":"string"},
        "delivery_route": {"type":"string"},
        "reason_code": {"type":"string"},
        "reason_detail": {"type":["string","null"]},
        "verification_summary": {"type":["object","null"]},
        "acceptance_record_id": {"type":["string","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-006",
      "notification_code":"acceptance.passed",
      "audience_scope":"buyer_seller_ops_acceptance_outcome",
      "version_note":"acceptance-passed-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
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
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status_label}}，结算={{variables.settlement_status}}，争议={{variables.dispute_status}}
原因：{{variables.reason_code}}
处理建议：{{variables.action_summary}}
{{#if variables.reason_detail}}
说明：{{variables.reason_detail}}
{{/if}}
{{#if variables.show_ops_context}}
联查：acceptance_record={{variables.acceptance_record_id}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=acceptance.rejected order={{variables.order_id}} acceptance={{variables.acceptance_status}} dispute={{variables.dispute_status}}',
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
        "current_state",
        "current_state_label",
        "acceptance_status_label",
        "delivery_type",
        "delivery_route",
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
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "acceptance_status_label": {"type":"string"},
        "delivery_type": {"type":"string"},
        "delivery_route": {"type":"string"},
        "reason_code": {"type":"string"},
        "reason_detail": {"type":["string","null"]},
        "verification_summary": {"type":["object","null"]},
        "acceptance_record_id": {"type":["string","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-006",
      "notification_code":"acceptance.rejected",
      "audience_scope":"buyer_seller_ops_acceptance_outcome",
      "version_note":"acceptance-rejected-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_REFUND_COMPLETED_V1',
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
订单金额：{{variables.order_currency_code}} {{variables.order_amount}}
执行金额：{{variables.resolution_currency_code}} {{variables.resolution_amount}}
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status}}，结算={{variables.settlement_status}}
案件：{{variables.case_id}} / {{variables.decision_code}}
处理建议：{{variables.action_summary}}
{{#if variables.penalty_code}}
扣罚：{{variables.penalty_code}}
{{/if}}
{{#if variables.reason_code}}
原因：{{variables.reason_code}}
{{/if}}
{{#if variables.show_ops_context}}
联查：provider={{variables.provider_key}} status={{variables.provider_status}} result={{variables.provider_result_id}} liability={{variables.liability_type}} resolution={{variables.resolution_ref_type}}/{{variables.resolution_record_id}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=refund.completed order={{variables.order_id}} amount={{variables.resolution_amount}} case={{variables.case_id}}',
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
        "order_currency_code",
        "resolution_amount",
        "resolution_currency_code",
        "payment_status",
        "delivery_status",
        "acceptance_status",
        "settlement_status",
        "current_state",
        "current_state_label",
        "case_id",
        "decision_code",
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
        "order_currency_code": {"type":"string"},
        "resolution_amount": {"type":"string"},
        "resolution_currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "acceptance_status": {"type":"string"},
        "settlement_status": {"type":"string"},
        "dispute_status": {"type":"string"},
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "case_id": {"type":"string"},
        "decision_code": {"type":"string"},
        "penalty_code": {"type":["string","null"]},
        "reason_code": {"type":["string","null"]},
        "liability_type": {"type":["string","null"]},
        "provider_key": {"type":["string","null"]},
        "provider_status": {"type":["string","null"]},
        "provider_result_id": {"type":["string","null"]},
        "resolution_ref_type": {"type":["string","null"]},
        "resolution_record_id": {"type":["string","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-006",
      "notification_code":"refund.completed",
      "audience_scope":"buyer_seller_ops_billing_resolution",
      "version_note":"refund-completed-visible"
    }'::jsonb
  ),
  (
    'NOTIFY_COMPENSATION_COMPLETED_V1',
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
订单金额：{{variables.order_currency_code}} {{variables.order_amount}}
执行金额：{{variables.resolution_currency_code}} {{variables.resolution_amount}}
状态：订单={{variables.current_state_label}}，支付={{variables.payment_status}}，交付={{variables.delivery_status}}，验收={{variables.acceptance_status}}，结算={{variables.settlement_status}}
案件：{{variables.case_id}} / {{variables.decision_code}}
处理建议：{{variables.action_summary}}
{{#if variables.penalty_code}}
扣罚：{{variables.penalty_code}}
{{/if}}
{{#if variables.reason_code}}
原因：{{variables.reason_code}}
{{/if}}
{{#if variables.show_ops_context}}
联查：provider={{variables.provider_key}} status={{variables.provider_status}} result={{variables.provider_result_id}} liability={{variables.liability_type}} resolution={{variables.resolution_ref_type}}/{{variables.resolution_record_id}}
{{/if}}
入口：{{variables.action_label}} {{variables.action_href}} $$,
    'notification=compensation.completed order={{variables.order_id}} amount={{variables.resolution_amount}} case={{variables.case_id}}',
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
        "order_currency_code",
        "resolution_amount",
        "resolution_currency_code",
        "payment_status",
        "delivery_status",
        "acceptance_status",
        "settlement_status",
        "current_state",
        "current_state_label",
        "case_id",
        "decision_code",
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
        "order_currency_code": {"type":"string"},
        "resolution_amount": {"type":"string"},
        "resolution_currency_code": {"type":"string"},
        "payment_status": {"type":"string"},
        "delivery_status": {"type":"string"},
        "acceptance_status": {"type":"string"},
        "settlement_status": {"type":"string"},
        "dispute_status": {"type":"string"},
        "current_state": {"type":"string"},
        "current_state_label": {"type":"string"},
        "case_id": {"type":"string"},
        "decision_code": {"type":"string"},
        "penalty_code": {"type":["string","null"]},
        "reason_code": {"type":["string","null"]},
        "liability_type": {"type":["string","null"]},
        "provider_key": {"type":["string","null"]},
        "provider_status": {"type":["string","null"]},
        "provider_result_id": {"type":["string","null"]},
        "resolution_ref_type": {"type":["string","null"]},
        "resolution_record_id": {"type":["string","null"]},
        "show_ops_context": {"type":"boolean"},
        "action_label": {"type":"string"},
        "action_href": {"type":"string"}
      }
    }'::jsonb,
    '{
      "seed_task":"NOTIF-006",
      "notification_code":"compensation.completed",
      "audience_scope":"buyer_seller_ops_billing_resolution",
      "version_note":"compensation-completed-visible"
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
