ALTER TABLE trade.order_main
  ADD COLUMN IF NOT EXISTS delivery_status text,
  ADD COLUMN IF NOT EXISTS acceptance_status text,
  ADD COLUMN IF NOT EXISTS settlement_status text,
  ADD COLUMN IF NOT EXISTS dispute_status text;

UPDATE trade.order_main
SET
  delivery_status = CASE
    WHEN status IN ('created', 'contract_pending', 'contract_effective', 'buyer_locked',
                    'payment_failed_pending_resolution', 'payment_timeout_pending_compensation_cancel')
      THEN 'pending_delivery'
    WHEN status = 'seller_delivering' THEN 'in_progress'
    WHEN status IN ('delivered', 'accepted', 'settled') THEN 'delivered'
    WHEN status = 'closed' THEN 'closed'
    ELSE COALESCE(delivery_status, 'pending_delivery')
  END,
  acceptance_status = CASE
    WHEN status = 'delivered' THEN 'pending_acceptance'
    WHEN status IN ('accepted', 'settled') THEN 'accepted'
    WHEN status = 'closed' THEN 'closed'
    ELSE COALESCE(acceptance_status, 'not_started')
  END,
  settlement_status = CASE
    WHEN status = 'settled' THEN 'settled'
    WHEN status = 'closed' THEN 'closed'
    WHEN payment_status = 'paid' THEN 'pending_settlement'
    ELSE COALESCE(settlement_status, 'not_started')
  END,
  dispute_status = COALESCE(dispute_status, 'none')
WHERE
  delivery_status IS NULL
  OR acceptance_status IS NULL
  OR settlement_status IS NULL
  OR dispute_status IS NULL;

ALTER TABLE trade.order_main
  ALTER COLUMN delivery_status SET DEFAULT 'pending_delivery',
  ALTER COLUMN acceptance_status SET DEFAULT 'not_started',
  ALTER COLUMN settlement_status SET DEFAULT 'not_started',
  ALTER COLUMN dispute_status SET DEFAULT 'none';

ALTER TABLE trade.order_main
  ALTER COLUMN delivery_status SET NOT NULL,
  ALTER COLUMN acceptance_status SET NOT NULL,
  ALTER COLUMN settlement_status SET NOT NULL,
  ALTER COLUMN dispute_status SET NOT NULL;
