CREATE TABLE IF NOT EXISTS ops.notification_template (
  notification_template_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  template_code text NOT NULL,
  language_code text NOT NULL DEFAULT 'zh-CN',
  channel text NOT NULL DEFAULT 'mock-log',
  version_no integer NOT NULL,
  enabled boolean NOT NULL DEFAULT true,
  status text NOT NULL DEFAULT 'active',
  title_template text NOT NULL,
  body_template text NOT NULL,
  fallback_body_template text NOT NULL,
  variables_schema_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT uq_notification_template_version UNIQUE (template_code, language_code, channel, version_no),
  CONSTRAINT chk_notification_template_version_no CHECK (version_no > 0),
  CONSTRAINT chk_notification_template_status CHECK (status IN ('draft', 'active', 'disabled', 'archived')),
  CONSTRAINT chk_notification_template_code_nonempty CHECK (btrim(template_code) <> ''),
  CONSTRAINT chk_notification_template_language_nonempty CHECK (btrim(language_code) <> ''),
  CONSTRAINT chk_notification_template_channel_nonempty CHECK (btrim(channel) <> '')
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_notification_template_active
  ON ops.notification_template (template_code, language_code, channel)
  WHERE enabled = true AND status = 'active';

CREATE INDEX IF NOT EXISTS idx_notification_template_lookup
  ON ops.notification_template (template_code, channel, language_code, enabled, status, version_no DESC);

DROP TRIGGER IF EXISTS trg_notification_template_updated_at ON ops.notification_template;
CREATE TRIGGER trg_notification_template_updated_at
BEFORE UPDATE ON ops.notification_template
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

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
    'DEFAULT_NOTIFICATION_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification={{notification.notification_code}} recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"default"}'::jsonb
  ),
  (
    'NOTIFY_GENERIC_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification={{notification.notification_code}} recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"generic"}'::jsonb
  ),
  (
    'NOTIFY_ORDER_CREATED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=order.created recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"order.created"}'::jsonb
  ),
  (
    'NOTIFY_PAYMENT_SUCCEEDED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=payment.succeeded recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"payment.succeeded"}'::jsonb
  ),
  (
    'NOTIFY_PAYMENT_FAILED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=payment.failed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"payment.failed"}'::jsonb
  ),
  (
    'NOTIFY_PENDING_DELIVERY_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=order.pending_delivery recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"order.pending_delivery"}'::jsonb
  ),
  (
    'NOTIFY_DELIVERY_COMPLETED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=delivery.completed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"delivery.completed"}'::jsonb
  ),
  (
    'NOTIFY_PENDING_ACCEPTANCE_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=order.pending_acceptance recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"order.pending_acceptance"}'::jsonb
  ),
  (
    'NOTIFY_ACCEPTANCE_PASSED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=acceptance.passed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"acceptance.passed"}'::jsonb
  ),
  (
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=acceptance.rejected recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"acceptance.rejected"}'::jsonb
  ),
  (
    'NOTIFY_DISPUTE_ESCALATED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=dispute.escalated recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"dispute.escalated"}'::jsonb
  ),
  (
    'NOTIFY_REFUND_COMPLETED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=refund.completed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"refund.completed"}'::jsonb
  ),
  (
    'NOTIFY_COMPENSATION_COMPLETED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=compensation.completed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"compensation.completed"}'::jsonb
  ),
  (
    'NOTIFY_SETTLEMENT_FROZEN_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=settlement.frozen recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"settlement.frozen"}'::jsonb
  ),
  (
    'NOTIFY_SETTLEMENT_RESUMED_V1',
    'zh-CN',
    'mock-log',
    1,
    true,
    'active',
    '{{variables.subject}}',
    '{{variables.message}}',
    'notification=settlement.resumed recipient={{recipient.address}}',
    '{"type":"object","required":["subject"],"properties":{"subject":{"type":"string"},"message":{"type":"string"}}}'::jsonb,
    '{"seed_task":"NOTIF-003","notification_code":"settlement.resumed"}'::jsonb
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
