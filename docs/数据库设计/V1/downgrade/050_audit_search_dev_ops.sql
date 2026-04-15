DROP TRIGGER IF EXISTS trg_dispute_outbox ON support.dispute_case;
DROP TRIGGER IF EXISTS trg_product_outbox ON catalog.product;
DROP TRIGGER IF EXISTS trg_order_outbox ON trade.order_main;
DROP TRIGGER IF EXISTS trg_payout_instruction_outbox ON payment.payout_instruction;
DROP TRIGGER IF EXISTS trg_payment_intent_outbox ON payment.payment_intent;
DROP TRIGGER IF EXISTS trg_mock_payment_case_updated_at ON developer.mock_payment_case;
DROP TRIGGER IF EXISTS trg_mock_provider_binding_updated_at ON developer.mock_provider_binding;
DROP TRIGGER IF EXISTS trg_test_application_updated_at ON developer.test_application;
DROP TRIGGER IF EXISTS trg_test_wallet_updated_at ON developer.test_wallet;
DROP TRIGGER IF EXISTS trg_product_search_refresh ON catalog.product;

DROP FUNCTION IF EXISTS search.tg_refresh_product_search_document();

DROP TABLE IF EXISTS chain.chain_anchor CASCADE;
DROP TABLE IF EXISTS chain.contract_event_projection CASCADE;
DROP TABLE IF EXISTS developer.mock_payment_case CASCADE;
DROP TABLE IF EXISTS developer.mock_provider_binding CASCADE;
DROP TABLE IF EXISTS developer.test_application CASCADE;
DROP TABLE IF EXISTS developer.test_wallet CASCADE;
DROP TABLE IF EXISTS search.product_search_document CASCADE;
DROP TABLE IF EXISTS ops.system_log_default CASCADE;
DROP TABLE IF EXISTS ops.system_log CASCADE;
DROP TABLE IF EXISTS ops.job_run CASCADE;
DROP TABLE IF EXISTS ops.dead_letter_event CASCADE;
DROP TABLE IF EXISTS ops.outbox_event CASCADE;
DROP TABLE IF EXISTS audit.evidence_package CASCADE;
DROP TABLE IF EXISTS audit.audit_event_default CASCADE;
DROP TABLE IF EXISTS audit.audit_event CASCADE;
-- Payment settlement sync: developer mock payment and payment outbox objects removed before audit/dev schema teardown.
