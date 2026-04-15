DROP TRIGGER IF EXISTS trg_mutual_recognition_updated_at ON ecosystem.mutual_recognition;
DROP TRIGGER IF EXISTS trg_partner_updated_at ON ecosystem.partner;
DROP TRIGGER IF EXISTS trg_compensation_task_updated_at ON crosschain.compensation_task;
DROP TRIGGER IF EXISTS trg_cross_chain_request_updated_at ON crosschain.cross_chain_request;
DROP TRIGGER IF EXISTS trg_gateway_identity_updated_at ON crosschain.gateway_identity;
DROP TRIGGER IF EXISTS trg_crypto_settlement_transfer_updated_at ON payment.crypto_settlement_transfer;
DROP TRIGGER IF EXISTS trg_settlement_route_updated_at ON payment.settlement_route;

DROP TABLE IF EXISTS ecosystem.mutual_recognition CASCADE;
DROP TABLE IF EXISTS ecosystem.connector_version CASCADE;
DROP TABLE IF EXISTS ecosystem.partner CASCADE;
DROP TABLE IF EXISTS payment.crypto_settlement_transfer CASCADE;
DROP TABLE IF EXISTS payment.fx_quote CASCADE;
DROP TABLE IF EXISTS payment.settlement_route CASCADE;
DROP TABLE IF EXISTS crosschain.compensation_task CASCADE;
DROP TABLE IF EXISTS crosschain.request_status_history CASCADE;
DROP TABLE IF EXISTS crosschain.witness_record CASCADE;
DROP TABLE IF EXISTS crosschain.cross_chain_ack CASCADE;
DROP TABLE IF EXISTS crosschain.cross_chain_request CASCADE;
DROP TABLE IF EXISTS crosschain.gateway_identity CASCADE;
-- Payment settlement sync: V3 settlement route, FX and crypto transfer objects removed with crosschain rollback.
-- Trust-boundary baseline sync: table drops already cover newly added V3 trust-boundary columns.
