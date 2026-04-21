CREATE TABLE IF NOT EXISTS risk.risk_alert (
  alert_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_type text,
  subject_id uuid,
  alert_type text NOT NULL,
  risk_score numeric(12, 6) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'new',
  object_set jsonb NOT NULL DEFAULT '[]'::jsonb,
  explanation text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.risk_case (
  risk_case_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  alert_id uuid REFERENCES risk.risk_alert(alert_id) ON DELETE SET NULL,
  case_status text NOT NULL DEFAULT 'new',
  severity text NOT NULL DEFAULT 'medium',
  assigned_to uuid REFERENCES core.user_account(user_id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.graph_node (
  graph_node_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  node_type text NOT NULL,
  ref_id uuid,
  attributes jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.graph_edge (
  graph_edge_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  from_node_id uuid NOT NULL REFERENCES risk.graph_node(graph_node_id) ON DELETE CASCADE,
  to_node_id uuid NOT NULL REFERENCES risk.graph_node(graph_node_id) ON DELETE CASCADE,
  edge_type text NOT NULL,
  weight numeric(12, 6) NOT NULL DEFAULT 1,
  attributes jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.regulator_query (
  regulator_query_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  regulator_org_id uuid REFERENCES core.organization(org_id),
  query_type text NOT NULL,
  query_target_id uuid,
  query_status text NOT NULL DEFAULT 'completed',
  result_digest text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit.regulator_export_record (
  regulator_export_record_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  regulator_query_id uuid REFERENCES audit.regulator_query(regulator_query_id) ON DELETE CASCADE,
  package_uri text,
  package_digest text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.freeze_ticket (
  freeze_ticket_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  freeze_type text NOT NULL,
  status text NOT NULL DEFAULT 'opened',
  reason_code text,
  requested_by uuid REFERENCES core.user_account(user_id),
  approved_by uuid REFERENCES core.user_account(user_id),
  executed_at timestamptz,
  released_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk.governance_action_log (
  governance_action_log_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  freeze_ticket_id uuid REFERENCES risk.freeze_ticket(freeze_ticket_id) ON DELETE CASCADE,
  action_type text NOT NULL,
  action_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_risk_alert_subject ON risk.risk_alert(subject_type, subject_id);
CREATE INDEX IF NOT EXISTS idx_graph_edge_from_to ON risk.graph_edge(from_node_id, to_node_id);

DROP TRIGGER IF EXISTS trg_risk_alert_updated_at ON risk.risk_alert;
CREATE TRIGGER trg_risk_alert_updated_at BEFORE UPDATE ON risk.risk_alert
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_risk_case_updated_at ON risk.risk_case;
CREATE TRIGGER trg_risk_case_updated_at BEFORE UPDATE ON risk.risk_case
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
DROP TRIGGER IF EXISTS trg_freeze_ticket_updated_at ON risk.freeze_ticket;
CREATE TRIGGER trg_freeze_ticket_updated_at BEFORE UPDATE ON risk.freeze_ticket
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Trust-boundary baseline sync: V3 storage-trace governance reuses existing risk/regulator tables; no extra structural change required in this file.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
