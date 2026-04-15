DROP TRIGGER IF EXISTS trg_reward_record_updated_at ON billing.reward_record;
DROP TRIGGER IF EXISTS trg_reward_pool_updated_at ON billing.reward_pool;
DROP TRIGGER IF EXISTS trg_profit_share_rule_updated_at ON billing.profit_share_rule;
DROP TRIGGER IF EXISTS trg_recurring_charge_plan_updated_at ON payment.recurring_charge_plan;
DROP TRIGGER IF EXISTS trg_split_instruction_updated_at ON payment.split_instruction;
DROP TRIGGER IF EXISTS trg_sub_merchant_binding_updated_at ON payment.sub_merchant_binding;

DROP TABLE IF EXISTS chain.credential_status_history CASCADE;
DROP TABLE IF EXISTS chain.credential_token CASCADE;
DROP TABLE IF EXISTS chain.public_anchor_batch CASCADE;
DROP TABLE IF EXISTS payment.recurring_charge_plan CASCADE;
DROP TABLE IF EXISTS payment.split_instruction CASCADE;
DROP TABLE IF EXISTS payment.sub_merchant_binding CASCADE;
DROP TABLE IF EXISTS billing.reward_record CASCADE;
DROP TABLE IF EXISTS billing.contribution_record CASCADE;
DROP TABLE IF EXISTS billing.reward_pool CASCADE;
DROP TABLE IF EXISTS billing.profit_share_rule CASCADE;
-- Payment settlement sync: V2 payment split and recurring objects removed with profitshare rollback.
-- Trust-boundary baseline sync: downgrade order unchanged.
