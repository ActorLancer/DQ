DROP TRIGGER IF EXISTS trg_subject_role_binding_updated_at ON authz.subject_role_binding;
DROP TRIGGER IF EXISTS trg_execution_environment_updated_at ON core.execution_environment;
DROP TRIGGER IF EXISTS trg_connector_updated_at ON core.connector;
DROP TRIGGER IF EXISTS trg_application_updated_at ON core.application;
DROP TRIGGER IF EXISTS trg_user_account_updated_at ON core.user_account;
DROP TRIGGER IF EXISTS trg_department_updated_at ON core.department;
DROP TRIGGER IF EXISTS trg_organization_updated_at ON core.organization;

DROP TABLE IF EXISTS authz.subject_role_binding CASCADE;
DROP TABLE IF EXISTS authz.role_permission CASCADE;
DROP TABLE IF EXISTS authz.permission_definition CASCADE;
DROP TABLE IF EXISTS authz.role_definition CASCADE;
DROP TABLE IF EXISTS core.execution_environment CASCADE;
DROP TABLE IF EXISTS core.connector CASCADE;
DROP TABLE IF EXISTS core.service_identity CASCADE;
DROP TABLE IF EXISTS core.application CASCADE;
DROP TABLE IF EXISTS core.did_binding CASCADE;
DROP TABLE IF EXISTS core.user_account CASCADE;
DROP TABLE IF EXISTS core.department CASCADE;
DROP TABLE IF EXISTS core.organization CASCADE;
-- Trust-boundary baseline sync: downgrade order unchanged.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
