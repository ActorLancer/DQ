DELETE FROM ops.notification_template
WHERE template_code IN (
    'NOTIFY_ACCEPTANCE_PASSED_V1',
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
    'NOTIFY_REFUND_COMPLETED_V1',
    'NOTIFY_COMPENSATION_COMPLETED_V1'
  )
  AND language_code = 'zh-CN'
  AND channel = 'mock-log'
  AND version_no = 2;

UPDATE ops.notification_template
SET
  enabled = TRUE,
  status = 'active',
  metadata = (metadata - 'archived_by_task' - 'replaced_by_version'),
  updated_at = now()
WHERE template_code IN (
    'NOTIFY_ACCEPTANCE_PASSED_V1',
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
    'NOTIFY_REFUND_COMPLETED_V1',
    'NOTIFY_COMPENSATION_COMPLETED_V1'
  )
  AND language_code = 'zh-CN'
  AND channel = 'mock-log'
  AND version_no = 1;
