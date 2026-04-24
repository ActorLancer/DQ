# Outbox Consistency Cases

`TEST-008` 的验收目标不是“看见一条 outbox 行”，而是证明下面三条冻结边界同时成立：

- 业务事务成功时，主对象、审计和 canonical outbox 同事务落库。
- 业务事务失败时，请求不得留下脏主对象、脏审计或脏 outbox。
- 正式消费者重复处理同一事件时，不得重复产生副作用，且必须留下 `ops.consumer_idempotency_record` / 审计 / 系统日志证据。

## 验收矩阵

| Case ID | 场景 | 入口 | 预期 |
| --- | --- | --- | --- |
| `OUTBOX-CASE-001` | 创建订单成功写主对象 + 审计 + outbox | `trade003_create_order_db_smoke` | `trade.order_main` 新增 1 条订单；`audit.audit_event(action_name='trade.order.create')` 至少 1 条；`ops.outbox_event(event_type='trade.order.created')` 恰好 1 条，且 payload 为 canonical envelope |
| `OUTBOX-CASE-002` | 创建订单失败不落脏 outbox | `trade003_create_order_db_smoke` 中失败分支 | 缺失 buyer org 时返回 `403`；`trade.order_main(idempotency_key=failed)`、`audit.audit_event(request_id=failed)`、`ops.outbox_event(request_id=failed)` 均为 `0` |
| `OUTBOX-CASE-003` | outbox 发布成功与死信隔离 | `outbox_publisher_db_smoke` | 成功样本被发布到 `dtp.outbox.domain-events` 并写 `ops.outbox_publish_attempt(result_code='published')`；失败样本进入 `ops.dead_letter_event` 和 Kafka `dtp.dead-letter` |
| `OUTBOX-CASE-004` | 重复消费不重复产生通知副作用 | `notif012_notification_worker_live_smoke` 中 duplicate 分支 | 第二次消费同一 `event_id / idempotency_key` 后，`ops.consumer_idempotency_record.result_code='duplicate'` 或等价去重结果；`notification sent via mock-log` 与 `notification.dispatch.sent` 不新增第二条 |

## 正式 Checker

宿主机正式入口：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh
```

该 checker 会依次执行：

1. `smoke-local.sh`，确认 PostgreSQL / Kafka / Redis / Keycloak / Mock Payment / observability 基线。
2. `TRADE_DB_SMOKE=1 cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`。
3. `AUD_DB_SMOKE=1 cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture`。
4. `NOTIF_WORKER_DB_SMOKE=1 cargo test -p notification-worker notif012_notification_worker_live_smoke -- --nocapture`。

## 回查要点

- `OUTBOX-CASE-001` / `OUTBOX-CASE-002`
  - `trade.order_main`
  - `audit.audit_event(action_name='trade.order.create')`
  - `ops.outbox_event(event_type='trade.order.created')`
- `OUTBOX-CASE-003`
  - `ops.outbox_event.status`
  - `ops.outbox_publish_attempt.result_code`
  - `ops.dead_letter_event.failure_stage='outbox.publish'`
  - Kafka `dtp.outbox.domain-events`
  - Kafka `dtp.dead-letter`
- `OUTBOX-CASE-004`
  - `ops.consumer_idempotency_record`
  - `ops.system_log(message_text='notification sent via mock-log')`
  - `audit.audit_event(action_name='notification.dispatch.sent')`

## 边界说明

- `PostgreSQL` 是业务主状态权威；`Kafka` 只负责传播事件与隔离死信。
- `outbox-publisher` 只发布 `ops.outbox_event(status='pending')`；它不是业务主状态机。
- `notification-worker` 只消费 `dtp.notification.dispatch`，不直接把 `dtp.outbox.domain-events` 当正式入口。
- `TEST-008` 不替代 `TEST-012` 的支付 webhook 幂等、`TEST-017` 的 dead letter / reprocess 专项，也不替代 `TEST-028` 的 canonical contracts checker。
