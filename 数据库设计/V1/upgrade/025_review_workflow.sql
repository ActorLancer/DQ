CREATE TABLE IF NOT EXISTS review.review_task (
  review_task_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  review_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  submitted_by uuid REFERENCES core.user_account(user_id),
  assigned_role_key text REFERENCES authz.role_definition(role_key),
  assigned_user_id uuid REFERENCES core.user_account(user_id),
  status text NOT NULL DEFAULT 'pending',
  risk_level text NOT NULL DEFAULT 'normal',
  current_step_no integer NOT NULL DEFAULT 1,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS review.review_step (
  review_step_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  review_task_id uuid NOT NULL REFERENCES review.review_task(review_task_id) ON DELETE CASCADE,
  step_no integer NOT NULL,
  reviewer_role_key text REFERENCES authz.role_definition(role_key),
  reviewer_user_id uuid REFERENCES core.user_account(user_id),
  action_name text,
  action_reason text,
  action_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  action_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (review_task_id, step_no)
);

CREATE TABLE IF NOT EXISTS ops.approval_ticket (
  approval_ticket_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  ticket_type text NOT NULL,
  ref_type text NOT NULL,
  ref_id uuid NOT NULL,
  requested_by uuid REFERENCES core.user_account(user_id),
  status text NOT NULL DEFAULT 'pending',
  requires_second_review boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ops.approval_step (
  approval_step_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  approval_ticket_id uuid NOT NULL REFERENCES ops.approval_ticket(approval_ticket_id) ON DELETE CASCADE,
  step_no integer NOT NULL,
  approver_role_key text REFERENCES authz.role_definition(role_key),
  approver_user_id uuid REFERENCES core.user_account(user_id),
  action_name text,
  action_reason text,
  action_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (approval_ticket_id, step_no)
);

CREATE INDEX IF NOT EXISTS idx_review_task_ref ON review.review_task(ref_type, ref_id);

CREATE TRIGGER trg_review_task_updated_at BEFORE UPDATE ON review.review_task
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
CREATE TRIGGER trg_approval_ticket_updated_at BEFORE UPDATE ON ops.approval_ticket
FOR EACH ROW EXECUTE FUNCTION common.tg_set_updated_at();
-- Trust-boundary baseline sync: review workflow schema remains valid; trust-boundary review rules are handled in product/order objects and permission seeds.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
