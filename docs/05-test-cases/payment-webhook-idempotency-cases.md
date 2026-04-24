# TEST-011 Payment Webhook Idempotency

`TEST-011` 的正式目标是证明支付 webhook 的重复投递、乱序投递和晚到投递不会破坏 `payment.payment_intent` 与 `trade.order_main` 的最终状态一致性。通过标准不是“接口返回 200”，而是要同时证明：重复 success 不重复产生副作用，`success -> fail` 不回退，`timeout -> success` 也不回退。

## 正式入口

- 本地 / CI checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-payment-webhook-idempotency.sh`
- checker 会先执行：
  - `./scripts/smoke-local.sh`
  - `./scripts/check-mock-payment.sh`
- 然后串行运行：
  - `TRADE_DB_SMOKE=1 cargo test -p platform-core bil005_payment_webhook_db_smoke -- --nocapture`

## 覆盖闭环

- `bil005_payment_webhook_db_smoke`
  - duplicate success webhook -> `processed_status=duplicate`
  - success 后旧 fail webhook -> `processed_status=out_of_order_ignored`
  - timeout webhook 后 late success webhook -> 仍为 `out_of_order_ignored`
  - 回查 `payment.payment_transaction`、`payment.payment_webhook_event`、`payment.payment_intent`、`trade.order_main`、`audit.audit_event`

## 关键不变量

- 同一 `provider_event_id` 的重复 success webhook 只允许一条业务交易副作用
- `payment.payment_intent.status` 不得被旧 webhook 或晚到 webhook 回退
- `trade.order_main.status / payment_status` 不得与 `payment_intent.status` 冲成分叉
- `payment.timeout` 已生效后，late `payment.succeeded` 只能被忽略，不能把订单从 `payment_timeout_pending_compensation_cancel / expired` 拉回支付成功

## 关键回查

- webhook 处理结果：

```sql
SELECT provider_event_id,
       processed_status,
       duplicate_flag,
       signature_verified,
       payment_transaction_id::text
FROM payment.payment_webhook_event
WHERE provider_key = 'mock_payment'
  AND provider_event_id = ANY(ARRAY[
    '<success_event_id>',
    '<duplicate_event_id>',
    '<timeout_event_id>',
    '<late_success_event_id>'
  ]);
```

- payment intent / order 最终状态：

```sql
SELECT pi.payment_intent_id::text,
       pi.status AS payment_intent_status,
       ord.order_id::text,
       ord.status AS order_status,
       ord.payment_status
FROM payment.payment_intent pi
JOIN trade.order_main ord ON ord.order_id = pi.order_id
WHERE pi.payment_intent_id = '<payment_intent_uuid>'::uuid;
```

## PWB 映射

- `PWB-002`：duplicate success webhook
- `PWB-003`：success 后 fail webhook
- `PWB-004`：timeout 语义与 timeout 后 late success 忽略

## 禁止误报

- 只看 `processed_status=processed` 不算通过；必须继续回查 `payment_transaction` 数量和订单 / payment intent 最终状态
- 只看 mock provider 健康检查不算通过；必须再跑真实 webhook smoke
- 只看 `payment.payment_intent` 一张表不算通过；必须确认 `trade.order_main` 没有被错误回退
