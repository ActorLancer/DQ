DROP TRIGGER IF EXISTS trg_freeze_ticket_updated_at ON risk.freeze_ticket;
DROP TRIGGER IF EXISTS trg_risk_case_updated_at ON risk.risk_case;
DROP TRIGGER IF EXISTS trg_risk_alert_updated_at ON risk.risk_alert;

DROP TABLE IF EXISTS risk.governance_action_log CASCADE;
DROP TABLE IF EXISTS risk.freeze_ticket CASCADE;
DROP TABLE IF EXISTS audit.regulator_export_record CASCADE;
DROP TABLE IF EXISTS audit.regulator_query CASCADE;
DROP TABLE IF EXISTS risk.graph_edge CASCADE;
DROP TABLE IF EXISTS risk.graph_node CASCADE;
DROP TABLE IF EXISTS risk.risk_case CASCADE;
DROP TABLE IF EXISTS risk.risk_alert CASCADE;
-- Trust-boundary baseline sync: downgrade order unchanged.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
