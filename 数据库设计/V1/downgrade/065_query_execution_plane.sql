DROP TRIGGER IF EXISTS trg_query_execution_run_updated_at ON delivery.query_execution_run;
DROP TRIGGER IF EXISTS trg_query_template_definition_updated_at ON delivery.query_template_definition;
DROP TRIGGER IF EXISTS trg_query_surface_definition_updated_at ON catalog.query_surface_definition;

ALTER TABLE delivery.sandbox_workspace
  DROP COLUMN IF EXISTS clean_room_mode,
  DROP COLUMN IF EXISTS query_surface_id;

ALTER TABLE delivery.template_query_grant
  DROP COLUMN IF EXISTS execution_rule_snapshot,
  DROP COLUMN IF EXISTS allowed_template_ids,
  DROP COLUMN IF EXISTS query_surface_id;

DROP TABLE IF EXISTS delivery.query_execution_run CASCADE;
DROP TABLE IF EXISTS delivery.query_template_definition CASCADE;
DROP TABLE IF EXISTS catalog.query_surface_definition CASCADE;
