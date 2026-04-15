CREATE TABLE IF NOT EXISTS recommend.model_registry (
  recommendation_model_registry_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  model_key text NOT NULL UNIQUE,
  model_type text NOT NULL,
  backend_type text NOT NULL,
  model_version text NOT NULL,
  stage_from text NOT NULL DEFAULT 'V2',
  status text NOT NULL DEFAULT 'draft',
  endpoint_ref text,
  metrics_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recommend.experiment_assignment (
  experiment_assignment_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  experiment_key text NOT NULL,
  variant_key text NOT NULL,
  subject_scope text NOT NULL,
  subject_ref text NOT NULL,
  assignment_rule jsonb NOT NULL DEFAULT '{}'::jsonb,
  assigned_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (experiment_key, subject_scope, subject_ref)
);

CREATE TABLE IF NOT EXISTS recommend.model_inference_log (
  model_inference_log_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  recommendation_request_id uuid REFERENCES recommend.recommendation_request(recommendation_request_id) ON DELETE SET NULL,
  recommendation_result_id uuid REFERENCES recommend.recommendation_result(recommendation_result_id) ON DELETE SET NULL,
  recommendation_model_registry_id uuid REFERENCES recommend.model_registry(recommendation_model_registry_id) ON DELETE SET NULL,
  experiment_assignment_id uuid REFERENCES recommend.experiment_assignment(experiment_assignment_id) ON DELETE SET NULL,
  inference_status text NOT NULL DEFAULT 'success',
  inference_latency_ms integer,
  candidate_count integer NOT NULL DEFAULT 0,
  output_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  error_code text,
  error_message text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_recommend_model_registry_status
  ON recommend.model_registry(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_recommend_experiment_subject
  ON recommend.experiment_assignment(experiment_key, subject_scope, subject_ref);
CREATE INDEX IF NOT EXISTS idx_recommend_model_inference_request
  ON recommend.model_inference_log(recommendation_request_id, created_at DESC);

CREATE TRIGGER trg_recommend_model_registry_updated_at BEFORE UPDATE ON recommend.model_registry
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

-- V2 recommendation baseline: optional external engines such as Gorse or LibRecommender may attach here, but PostgreSQL remains the recommendation audit/result authority.
