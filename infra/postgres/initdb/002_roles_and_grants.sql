DO
$$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'datab_app_rw') THEN
    CREATE ROLE datab_app_rw LOGIN PASSWORD 'datab_app_rw_local';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'datab_app_ro') THEN
    CREATE ROLE datab_app_ro LOGIN PASSWORD 'datab_app_ro_local';
  END IF;
END
$$;

GRANT CONNECT ON DATABASE datab TO datab_app_rw, datab_app_ro;

GRANT USAGE ON SCHEMA iam, catalog, trade, delivery, billing, audit, ops TO datab_app_rw, datab_app_ro;

ALTER DEFAULT PRIVILEGES IN SCHEMA iam, catalog, trade, delivery, billing, audit, ops
  GRANT SELECT ON TABLES TO datab_app_ro;

ALTER DEFAULT PRIVILEGES IN SCHEMA iam, catalog, trade, delivery, billing, audit, ops
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO datab_app_rw;

ALTER DEFAULT PRIVILEGES IN SCHEMA iam, catalog, trade, delivery, billing, audit, ops
  GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO datab_app_rw;
