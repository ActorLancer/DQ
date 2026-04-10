DROP TRIGGER IF EXISTS trg_storage_policy_profile_updated_at ON catalog.storage_policy_profile;
DROP TRIGGER IF EXISTS trg_storage_namespace_updated_at ON catalog.storage_namespace;

ALTER TABLE delivery.storage_object
  DROP COLUMN IF EXISTS retention_until,
  DROP COLUMN IF EXISTS storage_class,
  DROP COLUMN IF EXISTS storage_zone,
  DROP COLUMN IF EXISTS storage_namespace_id;

ALTER TABLE catalog.asset_storage_binding
  DROP COLUMN IF EXISTS retention_until,
  DROP COLUMN IF EXISTS access_path_type,
  DROP COLUMN IF EXISTS storage_class,
  DROP COLUMN IF EXISTS storage_zone,
  DROP COLUMN IF EXISTS storage_namespace_id;

ALTER TABLE catalog.asset_version
  DROP COLUMN IF EXISTS query_surface_type,
  DROP COLUMN IF EXISTS storage_policy_id;

DROP TABLE IF EXISTS catalog.storage_policy_profile CASCADE;
DROP TABLE IF EXISTS catalog.storage_namespace CASCADE;
