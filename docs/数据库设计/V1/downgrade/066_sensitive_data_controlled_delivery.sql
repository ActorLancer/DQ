DROP TRIGGER IF EXISTS trg_result_disclosure_review_updated_at ON delivery.result_disclosure_review;
DROP TRIGGER IF EXISTS trg_destruction_attestation_updated_at ON delivery.destruction_attestation;
DROP TRIGGER IF EXISTS trg_attestation_record_updated_at ON delivery.attestation_record;
DROP TRIGGER IF EXISTS trg_sensitive_execution_policy_updated_at ON delivery.sensitive_execution_policy;
DROP TRIGGER IF EXISTS trg_safe_preview_artifact_updated_at ON catalog.safe_preview_artifact;
DROP TRIGGER IF EXISTS trg_legal_basis_evidence_updated_at ON contract.legal_basis_evidence;
DROP TRIGGER IF EXISTS trg_sensitive_handling_policy_updated_at ON catalog.sensitive_handling_policy;

ALTER TABLE delivery.sandbox_workspace
  DROP COLUMN IF EXISTS sensitive_boundary_level;

ALTER TABLE delivery.api_credential
  DROP COLUMN IF EXISTS sensitive_scope_snapshot;

ALTER TABLE delivery.delivery_record
  DROP COLUMN IF EXISTS disclosure_review_status,
  DROP COLUMN IF EXISTS sensitive_delivery_mode;

ALTER TABLE delivery.query_execution_run
  DROP COLUMN IF EXISTS sensitive_policy_snapshot,
  DROP COLUMN IF EXISTS approval_ticket_id,
  DROP COLUMN IF EXISTS export_scope,
  DROP COLUMN IF EXISTS masked_level;

DROP TABLE IF EXISTS delivery.destruction_attestation CASCADE;
DROP TABLE IF EXISTS delivery.result_disclosure_review CASCADE;
DROP TABLE IF EXISTS delivery.attestation_record CASCADE;
DROP TABLE IF EXISTS delivery.sensitive_execution_policy CASCADE;
DROP TABLE IF EXISTS catalog.safe_preview_artifact CASCADE;
DROP TABLE IF EXISTS contract.legal_basis_evidence CASCADE;
DROP TABLE IF EXISTS catalog.sensitive_handling_policy CASCADE;

ALTER TABLE catalog.product
  DROP COLUMN IF EXISTS result_delivery_mode,
  DROP COLUMN IF EXISTS safe_preview_mode,
  DROP COLUMN IF EXISTS sensitive_handling_required;

ALTER TABLE catalog.asset_version
  DROP COLUMN IF EXISTS result_review_required,
  DROP COLUMN IF EXISTS safe_preview_required,
  DROP COLUMN IF EXISTS field_sensitivity_summary;

ALTER TABLE catalog.data_asset
  DROP COLUMN IF EXISTS default_sensitive_delivery_mode,
  DROP COLUMN IF EXISTS sensitive_tags,
  DROP COLUMN IF EXISTS data_classification,
  DROP COLUMN IF EXISTS contains_spi,
  DROP COLUMN IF EXISTS contains_pi;
