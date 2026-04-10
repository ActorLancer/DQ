CREATE TABLE IF NOT EXISTS recommend.page_optimization_profile (
  page_optimization_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  page_key text NOT NULL UNIQUE,
  layout_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  placement_order_json jsonb NOT NULL DEFAULT '[]'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.ecosystem_affinity (
  ecosystem_affinity_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  source_entity_scope text NOT NULL,
  source_entity_id uuid NOT NULL,
  target_partner_id uuid NOT NULL REFERENCES ecosystem.partner(partner_id) ON DELETE CASCADE,
  affinity_type text NOT NULL,
  score numeric(12, 6) NOT NULL DEFAULT 0,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (source_entity_scope, source_entity_id, target_partner_id, affinity_type)
);

CREATE INDEX IF NOT EXISTS idx_recommend_page_opt_profile_status
  ON recommend.page_optimization_profile(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_recommend_ecosystem_affinity_source
  ON recommend.ecosystem_affinity(source_entity_scope, source_entity_id, score DESC);

CREATE TRIGGER trg_recommend_page_opt_profile_updated_at BEFORE UPDATE ON recommend.page_optimization_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_ecosystem_affinity_updated_at BEFORE UPDATE ON recommend.ecosystem_affinity
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
