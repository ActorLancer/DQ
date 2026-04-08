DROP TRIGGER IF EXISTS trg_report_artifact_updated_at ON delivery.report_artifact;
DROP TRIGGER IF EXISTS trg_sandbox_workspace_updated_at ON delivery.sandbox_workspace;
DROP TRIGGER IF EXISTS trg_api_credential_updated_at ON delivery.api_credential;
DROP TRIGGER IF EXISTS trg_delivery_record_updated_at ON delivery.delivery_record;
DROP TRIGGER IF EXISTS trg_authorization_grant_updated_at ON trade.authorization_grant;
DROP TRIGGER IF EXISTS trg_order_status_history ON trade.order_main;
DROP TRIGGER IF EXISTS trg_order_main_updated_at ON trade.order_main;
DROP TRIGGER IF EXISTS trg_inquiry_updated_at ON trade.inquiry;

DROP TABLE IF EXISTS delivery.report_artifact CASCADE;
DROP TABLE IF EXISTS delivery.sandbox_session CASCADE;
DROP TABLE IF EXISTS delivery.sandbox_workspace CASCADE;
DROP TABLE IF EXISTS delivery.api_usage_log CASCADE;
DROP TABLE IF EXISTS delivery.api_credential CASCADE;
DROP TABLE IF EXISTS delivery.delivery_receipt CASCADE;
DROP TABLE IF EXISTS delivery.delivery_ticket CASCADE;
DROP TABLE IF EXISTS delivery.delivery_record CASCADE;
DROP TABLE IF EXISTS delivery.key_envelope CASCADE;
DROP TABLE IF EXISTS delivery.storage_object CASCADE;
DROP TABLE IF EXISTS contract.contract_signer CASCADE;
DROP TABLE IF EXISTS trade.authorization_grant CASCADE;
DROP TABLE IF EXISTS trade.order_status_history CASCADE;
DROP TABLE IF EXISTS trade.order_line CASCADE;
DROP TABLE IF EXISTS trade.order_main CASCADE;
DROP TABLE IF EXISTS contract.digital_contract CASCADE;
DROP TABLE IF EXISTS trade.inquiry CASCADE;
-- Trust-boundary baseline sync: trade/delivery tables drop as a whole, so no additional downgrade steps are required for new trust-boundary columns.

-- Payment settlement sync: no structural change required in this migration; payment domain changes are handled by dedicated payment/billing migrations.
