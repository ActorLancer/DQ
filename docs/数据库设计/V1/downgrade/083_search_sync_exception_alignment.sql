DROP TRIGGER IF EXISTS trg_index_sync_exception_updated_at ON search.index_sync_exception;

DROP INDEX IF EXISTS idx_index_sync_exception_dead_letter;
DROP INDEX IF EXISTS idx_index_sync_exception_scope_entity;
DROP INDEX IF EXISTS idx_index_sync_exception_task_status;
DROP TABLE IF EXISTS search.index_sync_exception;

DROP INDEX IF EXISTS idx_index_sync_task_dead_letter_event;
DROP INDEX IF EXISTS idx_index_sync_task_reconcile_status;

ALTER TABLE search.index_sync_task
  DROP COLUMN IF EXISTS dead_letter_event_id,
  DROP COLUMN IF EXISTS last_reconciled_at,
  DROP COLUMN IF EXISTS reconcile_status;
