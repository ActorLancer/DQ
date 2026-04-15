DROP TRIGGER IF EXISTS trg_recommend_model_registry_updated_at ON recommend.model_registry;

DROP TABLE IF EXISTS recommend.model_inference_log CASCADE;
DROP TABLE IF EXISTS recommend.experiment_assignment CASCADE;
DROP TABLE IF EXISTS recommend.model_registry CASCADE;
