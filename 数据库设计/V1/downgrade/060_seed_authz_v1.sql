DELETE FROM authz.permission_definition WHERE stage_from = 'V1';
DELETE FROM authz.role_definition WHERE stage_from = 'V1';
-- Trust-boundary baseline sync: stage-based delete already covers newly added V1 trust-boundary permissions and roles.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
