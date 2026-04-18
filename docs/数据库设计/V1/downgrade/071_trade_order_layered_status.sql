ALTER TABLE trade.order_main
  DROP COLUMN IF EXISTS dispute_status,
  DROP COLUMN IF EXISTS settlement_status,
  DROP COLUMN IF EXISTS acceptance_status,
  DROP COLUMN IF EXISTS delivery_status;
