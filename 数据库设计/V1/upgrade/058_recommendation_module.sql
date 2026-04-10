CREATE SCHEMA IF NOT EXISTS recommend;

CREATE TABLE IF NOT EXISTS recommend.placement_definition (
  placement_code text PRIMARY KEY,
  placement_name text NOT NULL,
  placement_scope text NOT NULL,
  page_context text NOT NULL,
  candidate_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  filter_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  default_ranking_profile_key text,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.ranking_profile (
  recommendation_ranking_profile_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_key text NOT NULL UNIQUE,
  placement_scope text NOT NULL,
  backend_type text NOT NULL DEFAULT 'rule',
  weights_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  diversity_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  exploration_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  explain_codes text[] NOT NULL DEFAULT '{}',
  status text NOT NULL DEFAULT 'active',
  stage_from text NOT NULL DEFAULT 'V1',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.behavior_event (
  behavior_event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_scope text NOT NULL,
  subject_org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  subject_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  anonymous_session_key text,
  event_type text NOT NULL,
  placement_code text REFERENCES recommend.placement_definition(placement_code) ON DELETE SET NULL,
  entity_scope text NOT NULL,
  entity_id uuid,
  page_context text,
  recommendation_request_id uuid,
  recommendation_result_id uuid,
  request_id text,
  trace_id text,
  occurred_at timestamptz NOT NULL DEFAULT now(),
  attrs jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.subject_profile_snapshot (
  subject_profile_snapshot_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subject_scope text NOT NULL,
  subject_ref text NOT NULL,
  org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  profile_version bigint NOT NULL DEFAULT 1,
  preferred_categories text[] NOT NULL DEFAULT '{}',
  preferred_tags text[] NOT NULL DEFAULT '{}',
  preferred_delivery_modes text[] NOT NULL DEFAULT '{}',
  feature_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  last_behavior_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (subject_scope, subject_ref)
);

CREATE TABLE IF NOT EXISTS recommend.cohort_definition (
  cohort_key text PRIMARY KEY,
  subject_scope text NOT NULL,
  dimension_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.cohort_popularity (
  cohort_key text NOT NULL REFERENCES recommend.cohort_definition(cohort_key) ON DELETE CASCADE,
  entity_scope text NOT NULL,
  entity_id uuid NOT NULL,
  exposure_count bigint NOT NULL DEFAULT 0,
  click_count bigint NOT NULL DEFAULT 0,
  order_count bigint NOT NULL DEFAULT 0,
  payment_count bigint NOT NULL DEFAULT 0,
  acceptance_count bigint NOT NULL DEFAULT 0,
  hotness_score numeric(12, 6) NOT NULL DEFAULT 0,
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (cohort_key, entity_scope, entity_id)
);

CREATE TABLE IF NOT EXISTS recommend.entity_similarity (
  entity_similarity_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  source_entity_scope text NOT NULL,
  source_entity_id uuid NOT NULL,
  target_entity_scope text NOT NULL,
  target_entity_id uuid NOT NULL,
  similarity_type text NOT NULL,
  score numeric(12, 6) NOT NULL DEFAULT 0,
  evidence_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  version_no bigint NOT NULL DEFAULT 1,
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (source_entity_scope, source_entity_id, target_entity_scope, target_entity_id, similarity_type)
);

CREATE TABLE IF NOT EXISTS recommend.bundle_relation (
  bundle_relation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  source_entity_scope text NOT NULL,
  source_entity_id uuid NOT NULL,
  target_entity_scope text NOT NULL,
  target_entity_id uuid NOT NULL,
  relation_type text NOT NULL,
  relation_score numeric(12, 6) NOT NULL DEFAULT 0,
  status text NOT NULL DEFAULT 'active',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (source_entity_scope, source_entity_id, target_entity_scope, target_entity_id, relation_type)
);

CREATE TABLE IF NOT EXISTS recommend.recommendation_request (
  recommendation_request_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  placement_code text NOT NULL REFERENCES recommend.placement_definition(placement_code) ON DELETE RESTRICT,
  subject_scope text NOT NULL,
  subject_org_id uuid REFERENCES core.organization(org_id) ON DELETE SET NULL,
  subject_user_id uuid REFERENCES core.user_account(user_id) ON DELETE SET NULL,
  anonymous_session_key text,
  page_context text,
  context_entity_scope text,
  context_entity_id uuid,
  ranking_profile_id uuid REFERENCES recommend.ranking_profile(recommendation_ranking_profile_id) ON DELETE SET NULL,
  filter_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  request_attrs jsonb NOT NULL DEFAULT '{}'::jsonb,
  candidate_source_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  trace_id text,
  request_id text,
  status text NOT NULL DEFAULT 'created',
  requested_count integer NOT NULL DEFAULT 10,
  served_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.recommendation_result (
  recommendation_result_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  recommendation_request_id uuid NOT NULL REFERENCES recommend.recommendation_request(recommendation_request_id) ON DELETE CASCADE,
  placement_code text NOT NULL,
  ranking_profile_id uuid REFERENCES recommend.ranking_profile(recommendation_ranking_profile_id) ON DELETE SET NULL,
  ranking_profile_version text,
  subject_scope text NOT NULL,
  subject_ref text,
  entity_scope text NOT NULL DEFAULT 'mixed',
  result_status text NOT NULL DEFAULT 'served',
  total_candidates integer NOT NULL DEFAULT 0,
  returned_count integer NOT NULL DEFAULT 0,
  explain_level text NOT NULL DEFAULT 'basic',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.recommendation_result_item (
  recommendation_result_item_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  recommendation_result_id uuid NOT NULL REFERENCES recommend.recommendation_result(recommendation_result_id) ON DELETE CASCADE,
  position_no integer NOT NULL,
  entity_scope text NOT NULL,
  entity_id uuid NOT NULL,
  recall_sources text[] NOT NULL DEFAULT '{}',
  raw_score numeric(12, 6) NOT NULL DEFAULT 0,
  final_score numeric(12, 6) NOT NULL DEFAULT 0,
  explanation_codes text[] NOT NULL DEFAULT '{}',
  feature_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
  click_status text NOT NULL DEFAULT 'not_clicked',
  conversion_status text NOT NULL DEFAULT 'none',
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (recommendation_result_id, position_no)
);

CREATE INDEX IF NOT EXISTS idx_behavior_event_subject_time
  ON recommend.behavior_event(subject_scope, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_behavior_event_entity
  ON recommend.behavior_event(entity_scope, entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_behavior_event_type_time
  ON recommend.behavior_event(event_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_subject_profile_org
  ON recommend.subject_profile_snapshot(org_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_cohort_popularity_hot
  ON recommend.cohort_popularity(cohort_key, hotness_score DESC);
CREATE INDEX IF NOT EXISTS idx_entity_similarity_source
  ON recommend.entity_similarity(source_entity_scope, source_entity_id, similarity_type, score DESC);
CREATE INDEX IF NOT EXISTS idx_bundle_relation_source
  ON recommend.bundle_relation(source_entity_scope, source_entity_id, relation_type, relation_score DESC);
CREATE INDEX IF NOT EXISTS idx_recommend_request_subject
  ON recommend.recommendation_request(subject_scope, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recommend_result_request
  ON recommend.recommendation_result(recommendation_request_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recommend_result_item_entity
  ON recommend.recommendation_result_item(entity_scope, entity_id, final_score DESC);

CREATE TRIGGER trg_recommend_placement_updated_at BEFORE UPDATE ON recommend.placement_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_ranking_profile_updated_at BEFORE UPDATE ON recommend.ranking_profile
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_subject_profile_updated_at BEFORE UPDATE ON recommend.subject_profile_snapshot
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_cohort_definition_updated_at BEFORE UPDATE ON recommend.cohort_definition
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_entity_similarity_updated_at BEFORE UPDATE ON recommend.entity_similarity
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_recommend_bundle_relation_updated_at BEFORE UPDATE ON recommend.bundle_relation
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

CREATE OR REPLACE FUNCTION recommend.tg_refresh_subject_profile()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_subject_ref text;
  v_last_category text;
  v_last_delivery text;
BEGIN
  v_subject_ref := COALESCE(NEW.subject_org_id::text, NEW.subject_user_id::text, NEW.anonymous_session_key);
  v_last_category := NEW.attrs ->> 'category';
  v_last_delivery := NEW.attrs ->> 'delivery_type';

  IF v_subject_ref IS NULL THEN
    RETURN NEW;
  END IF;

  INSERT INTO recommend.subject_profile_snapshot (
    subject_profile_snapshot_id,
    subject_scope,
    subject_ref,
    org_id,
    user_id,
    preferred_categories,
    preferred_tags,
    preferred_delivery_modes,
    feature_snapshot,
    last_behavior_at,
    created_at,
    updated_at
  )
  VALUES (
    gen_random_uuid(),
    NEW.subject_scope,
    v_subject_ref,
    NEW.subject_org_id,
    NEW.subject_user_id,
    CASE WHEN v_last_category IS NOT NULL THEN ARRAY[v_last_category] ELSE '{}'::text[] END,
    COALESCE(ARRAY(SELECT jsonb_array_elements_text(COALESCE(NEW.attrs -> 'tags', '[]'::jsonb))), '{}'::text[]),
    CASE WHEN v_last_delivery IS NOT NULL THEN ARRAY[v_last_delivery] ELSE '{}'::text[] END,
    jsonb_build_object(
      'last_event_type', NEW.event_type,
      'last_entity_scope', NEW.entity_scope,
      'last_entity_id', NEW.entity_id,
      'last_placement_code', NEW.placement_code
    ),
    NEW.occurred_at,
    now(),
    now()
  )
  ON CONFLICT (subject_scope, subject_ref) DO UPDATE
  SET org_id = COALESCE(EXCLUDED.org_id, recommend.subject_profile_snapshot.org_id),
      user_id = COALESCE(EXCLUDED.user_id, recommend.subject_profile_snapshot.user_id),
      preferred_categories = CASE
        WHEN v_last_category IS NOT NULL AND NOT (v_last_category = ANY (recommend.subject_profile_snapshot.preferred_categories))
          THEN recommend.subject_profile_snapshot.preferred_categories || v_last_category
        ELSE recommend.subject_profile_snapshot.preferred_categories
      END,
      preferred_tags = (
        SELECT COALESCE(array_agg(DISTINCT t), '{}'::text[])
        FROM unnest(
          recommend.subject_profile_snapshot.preferred_tags
          || COALESCE(ARRAY(SELECT jsonb_array_elements_text(COALESCE(NEW.attrs -> 'tags', '[]'::jsonb))), '{}'::text[])
        ) AS t
      ),
      preferred_delivery_modes = CASE
        WHEN v_last_delivery IS NOT NULL AND NOT (v_last_delivery = ANY (recommend.subject_profile_snapshot.preferred_delivery_modes))
          THEN recommend.subject_profile_snapshot.preferred_delivery_modes || v_last_delivery
        ELSE recommend.subject_profile_snapshot.preferred_delivery_modes
      END,
      feature_snapshot = recommend.subject_profile_snapshot.feature_snapshot || jsonb_build_object(
        'last_event_type', NEW.event_type,
        'last_entity_scope', NEW.entity_scope,
        'last_entity_id', NEW.entity_id,
        'last_placement_code', NEW.placement_code
      ),
      profile_version = recommend.subject_profile_snapshot.profile_version + 1,
      last_behavior_at = GREATEST(COALESCE(recommend.subject_profile_snapshot.last_behavior_at, NEW.occurred_at), NEW.occurred_at),
      updated_at = now();

  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION recommend.tg_update_cohort_popularity()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_cohort_key text;
  v_exposure_inc bigint := 0;
  v_click_inc bigint := 0;
  v_order_inc bigint := 0;
  v_payment_inc bigint := 0;
  v_accept_inc bigint := 0;
  v_hot_delta numeric(12, 6) := 0;
BEGIN
  IF NEW.entity_id IS NULL THEN
    RETURN NEW;
  END IF;

  v_cohort_key := COALESCE(
    NEW.attrs ->> 'cohort_key',
    CASE
      WHEN NEW.subject_org_id IS NOT NULL THEN 'org:' || NEW.subject_org_id::text
      WHEN NEW.subject_user_id IS NOT NULL THEN 'user:' || NEW.subject_user_id::text
      ELSE 'global'
    END
  );

  INSERT INTO recommend.cohort_definition (cohort_key, subject_scope, dimension_json, status, created_at, updated_at)
  VALUES (
    v_cohort_key,
    COALESCE(NEW.subject_scope, 'anonymous'),
    jsonb_build_object('derived_from_event', true),
    'active',
    now(),
    now()
  )
  ON CONFLICT (cohort_key) DO NOTHING;

  IF NEW.event_type IN ('recommendation_panel_viewed', 'recommendation_item_exposed') THEN
    v_exposure_inc := 1;
    v_hot_delta := 0.20;
  ELSIF NEW.event_type IN ('recommendation_item_clicked', 'seller_recommendation_clicked', 'product_detail_viewed', 'service_detail_viewed', 'seller_profile_viewed') THEN
    v_click_inc := 1;
    v_hot_delta := 1.00;
  ELSIF NEW.event_type = 'order_submitted' THEN
    v_order_inc := 1;
    v_hot_delta := 2.00;
  ELSIF NEW.event_type = 'payment_succeeded' THEN
    v_payment_inc := 1;
    v_hot_delta := 3.00;
  ELSIF NEW.event_type = 'delivery_accepted' THEN
    v_accept_inc := 1;
    v_hot_delta := 3.50;
  END IF;

  INSERT INTO recommend.cohort_popularity (
    cohort_key,
    entity_scope,
    entity_id,
    exposure_count,
    click_count,
    order_count,
    payment_count,
    acceptance_count,
    hotness_score,
    updated_at
  )
  VALUES (
    v_cohort_key,
    NEW.entity_scope,
    NEW.entity_id,
    v_exposure_inc,
    v_click_inc,
    v_order_inc,
    v_payment_inc,
    v_accept_inc,
    v_hot_delta,
    now()
  )
  ON CONFLICT (cohort_key, entity_scope, entity_id) DO UPDATE
  SET exposure_count = recommend.cohort_popularity.exposure_count + EXCLUDED.exposure_count,
      click_count = recommend.cohort_popularity.click_count + EXCLUDED.click_count,
      order_count = recommend.cohort_popularity.order_count + EXCLUDED.order_count,
      payment_count = recommend.cohort_popularity.payment_count + EXCLUDED.payment_count,
      acceptance_count = recommend.cohort_popularity.acceptance_count + EXCLUDED.acceptance_count,
      hotness_score = recommend.cohort_popularity.hotness_score + EXCLUDED.hotness_score,
      updated_at = now();

  RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION common.tg_write_outbox()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
  v_ref_id uuid;
  v_payload jsonb;
  v_request_id text;
  v_trace_id text;
  v_idempotency_key text;
  v_target_topic text;
  v_partition_key text;
  v_payload_hash text;
BEGIN
  v_payload := to_jsonb(NEW);
  v_ref_id := COALESCE(
    CASE WHEN COALESCE(v_payload ->> 'order_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'order_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'product_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'product_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'case_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'case_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'audit_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'audit_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'billing_event_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'billing_event_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'delivery_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'delivery_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'payment_intent_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'payment_intent_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'refund_intent_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'refund_intent_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'payout_instruction_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'payout_instruction_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'reconciliation_statement_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'reconciliation_statement_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'crypto_transfer_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'crypto_transfer_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'behavior_event_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'behavior_event_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'recommendation_request_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'recommendation_request_id')::uuid END,
    CASE WHEN COALESCE(v_payload ->> 'recommendation_result_id', '') ~* '^[0-9a-f-]{36}$' THEN (v_payload ->> 'recommendation_result_id')::uuid END
  );
  v_request_id := v_payload ->> 'request_id';
  v_trace_id := COALESCE(v_payload ->> 'trace_id', v_payload ->> 'event_trace_id');
  v_idempotency_key := v_payload ->> 'idempotency_key';
  v_target_topic := replace(TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME, '.', '_');
  v_partition_key := COALESCE(v_ref_id::text, v_request_id, gen_random_uuid()::text);
  v_payload_hash := encode(digest(v_payload::text, 'sha256'), 'hex');

  INSERT INTO ops.outbox_event (
    outbox_event_id,
    aggregate_type,
    aggregate_id,
    event_type,
    payload,
    status,
    created_at,
    event_schema_version,
    request_id,
    trace_id,
    idempotency_key,
    authority_scope,
    source_of_truth,
    proof_commit_policy,
    target_bus,
    target_topic,
    partition_key,
    ordering_key,
    payload_hash
  )
  VALUES (
    gen_random_uuid(),
    TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME,
    v_ref_id,
    TG_OP,
    v_payload,
    'pending',
    now(),
    'v1',
    v_request_id,
    v_trace_id,
    v_idempotency_key,
    'business',
    'database',
    COALESCE(v_payload ->> 'proof_commit_policy', 'async_evidence'),
    'kafka',
    v_target_topic,
    v_partition_key,
    v_partition_key,
    v_payload_hash
  );
  RETURN NEW;
END;
$$;

CREATE TRIGGER trg_recommend_behavior_event_profile AFTER INSERT ON recommend.behavior_event
FOR EACH ROW EXECUTE FUNCTION recommend.tg_refresh_subject_profile();
CREATE TRIGGER trg_recommend_behavior_event_cohort AFTER INSERT ON recommend.behavior_event
FOR EACH ROW EXECUTE FUNCTION recommend.tg_update_cohort_popularity();
CREATE TRIGGER trg_recommend_behavior_event_outbox AFTER INSERT ON recommend.behavior_event
FOR EACH ROW EXECUTE FUNCTION common.tg_write_outbox();

INSERT INTO recommend.placement_definition (
  placement_code,
  placement_name,
  placement_scope,
  page_context,
  candidate_policy_json,
  filter_policy_json,
  default_ranking_profile_key,
  status
) VALUES
('home_featured', '首页精选', 'mixed', 'home', '{"recall":["popular","new_arrival","trusted_seller"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_default', 'active'),
('industry_featured', '行业专题推荐', 'product', 'industry_topic', '{"recall":["popular","similar"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_default', 'active'),
('product_detail_similar', '商品详情页相似推荐', 'product', 'product_detail', '{"recall":["similar","cohort"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_detail', 'active'),
('product_detail_bundle', '商品详情页配套服务推荐', 'service', 'product_detail', '{"recall":["bundle","seller_related"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_bundle', 'active'),
('seller_profile_featured', '卖方主页热门推荐', 'mixed', 'seller_profile', '{"recall":["seller_hot","seller_quality"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_seller', 'active'),
('buyer_workbench_discovery', '买方工作台推荐', 'mixed', 'buyer_workbench', '{"recall":["cohort","new_arrival","renewal"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_default', 'active'),
('search_zero_result_fallback', '搜索零结果兜底推荐', 'mixed', 'search', '{"recall":["similar","popular"]}'::jsonb, '{"require_visible":true}'::jsonb, 'recommend_v1_default', 'active')
ON CONFLICT (placement_code) DO NOTHING;

INSERT INTO recommend.ranking_profile (
  recommendation_ranking_profile_id,
  profile_key,
  placement_scope,
  backend_type,
  weights_json,
  diversity_policy_json,
  exploration_policy_json,
  explain_codes,
  status,
  stage_from
) VALUES
(
  gen_random_uuid(),
  'recommend_v1_default',
  'mixed',
  'rule',
  '{"intent":0.25,"similarity":0.20,"hotness":0.15,"quality":0.15,"reputation":0.10,"freshness":0.05,"conversion":0.05,"bundle":0.05,"risk_penalty":0.10,"repeat_penalty":0.05}'::jsonb,
  '{"max_same_seller":2}'::jsonb,
  '{"new_item_boost":0.10}'::jsonb,
  ARRAY['popular_overall','new_and_qualified','trusted_seller_boost'],
  'active',
  'V1'
),
(
  gen_random_uuid(),
  'recommend_v1_detail',
  'product',
  'rule',
  '{"intent":0.20,"similarity":0.35,"hotness":0.10,"quality":0.15,"reputation":0.10,"freshness":0.05,"conversion":0.05}'::jsonb,
  '{"max_same_seller":2}'::jsonb,
  '{"new_item_boost":0.05}'::jsonb,
  ARRAY['similar_to_current_item','often_purchased_together'],
  'active',
  'V1'
),
(
  gen_random_uuid(),
  'recommend_v1_bundle',
  'service',
  'rule',
  '{"bundle":0.35,"similarity":0.20,"quality":0.15,"reputation":0.10,"hotness":0.10,"conversion":0.10}'::jsonb,
  '{"max_same_seller":2}'::jsonb,
  '{"new_item_boost":0.00}'::jsonb,
  ARRAY['service_bundle_match','same_seller_more_items'],
  'active',
  'V1'
),
(
  gen_random_uuid(),
  'recommend_v1_seller',
  'seller',
  'rule',
  '{"reputation":0.30,"quality":0.20,"hotness":0.20,"freshness":0.10,"conversion":0.10,"similarity":0.10}'::jsonb,
  '{"max_same_seller":1}'::jsonb,
  '{"new_item_boost":0.05}'::jsonb,
  ARRAY['trusted_seller_boost','same_seller_more_items'],
  'active',
  'V1'
)
ON CONFLICT (profile_key) DO NOTHING;

-- Recommendation baseline: PostgreSQL stores behavior, profiles and results as authority; OpenSearch is candidate recall only; Redis is cache only.
