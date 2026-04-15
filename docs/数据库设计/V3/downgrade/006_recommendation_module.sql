DROP TRIGGER IF EXISTS trg_recommend_ecosystem_affinity_updated_at ON recommend.ecosystem_affinity;
DROP TRIGGER IF EXISTS trg_recommend_page_opt_profile_updated_at ON recommend.page_optimization_profile;

DROP TABLE IF EXISTS recommend.ecosystem_affinity CASCADE;
DROP TABLE IF EXISTS recommend.page_optimization_profile CASCADE;
