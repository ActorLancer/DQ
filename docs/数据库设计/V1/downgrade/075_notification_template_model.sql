DROP TRIGGER IF EXISTS trg_notification_template_updated_at ON ops.notification_template;
DROP INDEX IF EXISTS idx_notification_template_lookup;
DROP INDEX IF EXISTS uq_notification_template_active;
DROP TABLE IF EXISTS ops.notification_template;
