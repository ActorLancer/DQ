DROP TRIGGER IF EXISTS trg_approval_ticket_updated_at ON ops.approval_ticket;
DROP TRIGGER IF EXISTS trg_review_task_updated_at ON review.review_task;

DROP TABLE IF EXISTS ops.approval_step CASCADE;
DROP TABLE IF EXISTS ops.approval_ticket CASCADE;
DROP TABLE IF EXISTS review.review_step CASCADE;
DROP TABLE IF EXISTS review.review_task CASCADE;
-- Trust-boundary baseline sync: downgrade order unchanged.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
