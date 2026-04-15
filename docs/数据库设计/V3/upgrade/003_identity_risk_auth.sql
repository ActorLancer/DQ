ALTER TABLE iam.external_identity_binding
  ADD COLUMN IF NOT EXISTS partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS federation_scope jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS iam.risk_auth_policy (
  risk_auth_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  policy_name text NOT NULL,
  partner_id uuid REFERENCES ecosystem.partner(partner_id) ON DELETE SET NULL,
  signal_scope text NOT NULL DEFAULT 'global',
  risk_level text NOT NULL DEFAULT 'medium',
  device_trust_threshold text,
  step_up_required boolean NOT NULL DEFAULT true,
  allowlist_countries text[] NOT NULL DEFAULT '{}',
  blocklist_countries text[] NOT NULL DEFAULT '{}',
  status text NOT NULL DEFAULT 'draft',
  rule_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_by_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_risk_auth_policy_partner_status
ON iam.risk_auth_policy(partner_id, status);

CREATE TRIGGER trg_risk_auth_policy_updated_at BEFORE UPDATE ON iam.risk_auth_policy
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
