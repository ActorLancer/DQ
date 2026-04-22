ALTER TABLE search.index_sync_task
  ADD COLUMN IF NOT EXISTS reconcile_status text NOT NULL DEFAULT 'pending_check',
  ADD COLUMN IF NOT EXISTS last_reconciled_at timestamptz,
  ADD COLUMN IF NOT EXISTS dead_letter_event_id uuid REFERENCES ops.dead_letter_event(dead_letter_event_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_index_sync_task_reconcile_status
  ON search.index_sync_task(reconcile_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_index_sync_task_dead_letter_event
  ON search.index_sync_task(dead_letter_event_id)
  WHERE dead_letter_event_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS search.index_sync_exception (
  index_sync_exception_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  index_sync_task_id uuid REFERENCES search.index_sync_task(index_sync_task_id) ON DELETE SET NULL,
  entity_scope text NOT NULL,
  entity_id uuid NOT NULL,
  document_version bigint NOT NULL DEFAULT 1,
  target_backend text NOT NULL DEFAULT 'opensearch',
  target_index text,
  source_event_id uuid REFERENCES ops.outbox_event(outbox_event_id) ON DELETE SET NULL,
  dead_letter_event_id uuid REFERENCES ops.dead_letter_event(dead_letter_event_id) ON DELETE SET NULL,
  exception_type text NOT NULL DEFAULT 'sync_failed',
  exception_status text NOT NULL DEFAULT 'open',
  failure_stage text,
  error_code text,
  error_message text,
  retryable boolean NOT NULL DEFAULT true,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  detected_at timestamptz NOT NULL DEFAULT now(),
  resolved_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_index_sync_exception_task_status
  ON search.index_sync_exception(index_sync_task_id, exception_status, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_index_sync_exception_scope_entity
  ON search.index_sync_exception(entity_scope, entity_id, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_index_sync_exception_dead_letter
  ON search.index_sync_exception(dead_letter_event_id)
  WHERE dead_letter_event_id IS NOT NULL;

UPDATE search.index_sync_task
SET reconcile_status = CASE
      WHEN sync_status = 'completed' THEN 'clean'
      WHEN sync_status = 'failed' THEN 'drift_detected'
      ELSE 'pending_check'
    END,
    last_reconciled_at = CASE
      WHEN sync_status IN ('completed', 'failed')
        THEN COALESCE(last_reconciled_at, completed_at, updated_at, created_at)
      ELSE last_reconciled_at
    END
WHERE reconcile_status = 'pending_check';

WITH latest_dead_letter AS (
  SELECT DISTINCT ON (outbox_event_id)
         outbox_event_id,
         dead_letter_event_id
  FROM ops.dead_letter_event
  WHERE outbox_event_id IS NOT NULL
  ORDER BY outbox_event_id, created_at DESC, dead_letter_event_id DESC
)
UPDATE search.index_sync_task t
SET dead_letter_event_id = d.dead_letter_event_id
FROM latest_dead_letter d
WHERE t.source_event_id = d.outbox_event_id
  AND t.dead_letter_event_id IS NULL;

INSERT INTO search.index_sync_exception (
  index_sync_task_id,
  entity_scope,
  entity_id,
  document_version,
  target_backend,
  target_index,
  source_event_id,
  dead_letter_event_id,
  exception_type,
  exception_status,
  failure_stage,
  error_code,
  error_message,
  retryable,
  metadata,
  detected_at,
  resolved_at
)
SELECT
  t.index_sync_task_id,
  t.entity_scope,
  t.entity_id,
  t.document_version,
  t.target_backend,
  t.target_index,
  t.source_event_id,
  t.dead_letter_event_id,
  'sync_failed',
  'open',
  CASE
    WHEN t.dead_letter_event_id IS NOT NULL THEN 'consumer_handler'
    ELSE 'index_write'
  END,
  t.last_error_code,
  t.last_error_message,
  true,
  jsonb_build_object(
    'backfill', true,
    'sync_status', t.sync_status
  ),
  COALESCE(t.completed_at, t.updated_at, t.created_at),
  NULL
FROM search.index_sync_task t
WHERE t.sync_status = 'failed'
  AND (t.last_error_code IS NOT NULL OR t.last_error_message IS NOT NULL)
  AND NOT EXISTS (
    SELECT 1
    FROM search.index_sync_exception e
    WHERE e.index_sync_task_id = t.index_sync_task_id
  );

CREATE TRIGGER trg_index_sync_exception_updated_at BEFORE UPDATE ON search.index_sync_exception
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
