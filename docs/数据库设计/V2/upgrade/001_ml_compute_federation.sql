CREATE TABLE IF NOT EXISTS ml.model_asset (
  model_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  source_task_id uuid,
  model_name text NOT NULL,
  model_family text,
  artifact_storage_mode text NOT NULL DEFAULT 'model_registry',
  weight_custody_mode text NOT NULL DEFAULT 'seller_managed',
  platform_plaintext_access boolean NOT NULL DEFAULT false,
  model_hash text,
  watermark_hash text,
  status text NOT NULL DEFAULT 'draft',
  metric_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ml.model_version (
  model_version_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  model_id uuid NOT NULL REFERENCES ml.model_asset(model_id) ON DELETE CASCADE,
  version_no integer NOT NULL,
  artifact_uri text,
  artifact_storage_mode text NOT NULL DEFAULT 'model_registry',
  custody_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  model_hash text,
  status text NOT NULL DEFAULT 'registered',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (model_id, version_no)
);

CREATE TABLE IF NOT EXISTS ml.algorithm_artifact (
  algorithm_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  uploader_user_id uuid REFERENCES core.user_account(user_id),
  algorithm_name text NOT NULL,
  image_uri text,
  image_digest text,
  execution_visibility_level text NOT NULL DEFAULT 'container_visible',
  whitelist_status text NOT NULL DEFAULT 'pending',
  review_status text NOT NULL DEFAULT 'pending',
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ml.compute_task (
  task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id uuid REFERENCES trade.order_main(order_id) ON DELETE SET NULL,
  initiator_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  environment_id uuid REFERENCES core.execution_environment(environment_id),
  algorithm_id uuid REFERENCES ml.algorithm_artifact(algorithm_id),
  task_type text NOT NULL,
  status text NOT NULL DEFAULT 'draft',
  input_residency_mode text NOT NULL DEFAULT 'seller_self_hosted',
  execution_boundary_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  input_params jsonb NOT NULL DEFAULT '{}'::jsonb,
  output_template jsonb NOT NULL DEFAULT '{}'::jsonb,
  result_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  started_at timestamptz,
  finished_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ml.compute_result (
  compute_result_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id uuid NOT NULL REFERENCES ml.compute_task(task_id) ON DELETE CASCADE,
  result_uri text,
  result_hash text,
  result_export_policy jsonb NOT NULL DEFAULT '{}'::jsonb,
  trust_evidence_required boolean NOT NULL DEFAULT false,
  result_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  export_status text NOT NULL DEFAULT 'pending',
  review_status text NOT NULL DEFAULT 'pending',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ml.training_task (
  task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  initiator_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  related_order_id uuid REFERENCES trade.order_main(order_id),
  task_type text NOT NULL,
  model_family text,
  rounds integer NOT NULL DEFAULT 1,
  min_participants integer NOT NULL DEFAULT 1,
  policy_hash text,
  privacy_mode text,
  training_data_residency_mode text NOT NULL DEFAULT 'seller_self_hosted',
  proof_binding_required boolean NOT NULL DEFAULT false,
  allowed_regions text[] NOT NULL DEFAULT '{}',
  allowed_output text[] NOT NULL DEFAULT '{}',
  reward_policy_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'draft',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE ml.model_asset
  ADD CONSTRAINT fk_model_asset_training_task
  FOREIGN KEY (source_task_id) REFERENCES ml.training_task(task_id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS ml.task_participant (
  task_participant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id uuid NOT NULL REFERENCES ml.training_task(task_id) ON DELETE CASCADE,
  participant_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  connector_id uuid REFERENCES core.connector(connector_id),
  environment_id uuid REFERENCES core.execution_environment(environment_id),
  role_in_task text NOT NULL DEFAULT 'participant',
  status text NOT NULL DEFAULT 'invited',
  availability_score numeric(10, 4) NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (task_id, participant_org_id)
);

CREATE TABLE IF NOT EXISTS ml.training_round (
  round_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id uuid NOT NULL REFERENCES ml.training_task(task_id) ON DELETE CASCADE,
  round_no integer NOT NULL,
  round_commit_hash text,
  aggregation_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
  status text NOT NULL DEFAULT 'pending',
  started_at timestamptz,
  finished_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (task_id, round_no)
);

CREATE TABLE IF NOT EXISTS ml.model_update (
  model_update_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  round_id uuid NOT NULL REFERENCES ml.training_round(round_id) ON DELETE CASCADE,
  participant_id uuid NOT NULL REFERENCES ml.task_participant(task_participant_id) ON DELETE CASCADE,
  update_hash text NOT NULL,
  metrics_json jsonb NOT NULL DEFAULT '{}'::jsonb,
  signature_digest text,
  status text NOT NULL DEFAULT 'submitted',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ml.proof_artifact (
  proof_ref uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  proof_type text NOT NULL,
  proof_uri text,
  proof_hash text,
  valid_until timestamptz,
  status text NOT NULL DEFAULT 'active',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS search.model_search_document (
  model_id uuid PRIMARY KEY REFERENCES ml.model_asset(model_id) ON DELETE CASCADE,
  owner_org_id uuid NOT NULL REFERENCES core.organization(org_id),
  model_name text NOT NULL,
  model_family text,
  metric_summary text,
  searchable_tsv tsvector,
  embedding vector(1536),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_model_search_document_tsv
  ON search.model_search_document USING GIN (searchable_tsv);
CREATE INDEX IF NOT EXISTS idx_model_search_document_embedding
  ON search.model_search_document USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

CREATE TABLE IF NOT EXISTS ml.task_status_history (
  task_status_history_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_scope text NOT NULL,
  task_id uuid NOT NULL,
  old_status text,
  new_status text NOT NULL,
  changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_model_asset_owner_org_id ON ml.model_asset(owner_org_id);
CREATE INDEX IF NOT EXISTS idx_compute_task_order_id ON ml.compute_task(order_id);
CREATE INDEX IF NOT EXISTS idx_training_task_initiator_org_id ON ml.training_task(initiator_org_id);
CREATE INDEX IF NOT EXISTS idx_task_participant_task_id ON ml.task_participant(task_id);

CREATE TRIGGER trg_model_asset_updated_at BEFORE UPDATE ON ml.model_asset
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_model_version_updated_at BEFORE UPDATE ON ml.model_version
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_algorithm_artifact_updated_at BEFORE UPDATE ON ml.algorithm_artifact
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_compute_task_updated_at BEFORE UPDATE ON ml.compute_task
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_compute_result_updated_at BEFORE UPDATE ON ml.compute_result
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_training_task_updated_at BEFORE UPDATE ON ml.training_task
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_task_participant_updated_at BEFORE UPDATE ON ml.task_participant
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_proof_artifact_updated_at BEFORE UPDATE ON ml.proof_artifact
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
