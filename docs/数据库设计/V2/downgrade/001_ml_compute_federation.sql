DROP TRIGGER IF EXISTS trg_proof_artifact_updated_at ON ml.proof_artifact;
DROP TRIGGER IF EXISTS trg_task_participant_updated_at ON ml.task_participant;
DROP TRIGGER IF EXISTS trg_training_task_updated_at ON ml.training_task;
DROP TRIGGER IF EXISTS trg_compute_result_updated_at ON ml.compute_result;
DROP TRIGGER IF EXISTS trg_compute_task_updated_at ON ml.compute_task;
DROP TRIGGER IF EXISTS trg_algorithm_artifact_updated_at ON ml.algorithm_artifact;
DROP TRIGGER IF EXISTS trg_model_version_updated_at ON ml.model_version;
DROP TRIGGER IF EXISTS trg_model_asset_updated_at ON ml.model_asset;

DROP TABLE IF EXISTS ml.task_status_history CASCADE;
DROP TABLE IF EXISTS search.model_search_document CASCADE;
DROP TABLE IF EXISTS ml.proof_artifact CASCADE;
DROP TABLE IF EXISTS ml.model_update CASCADE;
DROP TABLE IF EXISTS ml.training_round CASCADE;
DROP TABLE IF EXISTS ml.task_participant CASCADE;
DROP TABLE IF EXISTS ml.training_task CASCADE;
DROP TABLE IF EXISTS ml.compute_result CASCADE;
DROP TABLE IF EXISTS ml.compute_task CASCADE;
DROP TABLE IF EXISTS ml.algorithm_artifact CASCADE;
DROP TABLE IF EXISTS ml.model_version CASCADE;
DROP TABLE IF EXISTS ml.model_asset CASCADE;
-- Trust-boundary baseline sync: table drops already cover newly added V2 trust-boundary columns.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
