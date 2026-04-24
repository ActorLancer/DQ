# TEST-027 Notification Smoke

## Goal

把通知域现有的 `platform-core` DB smoke、`notification-worker` live smoke、runbook 和 `ops` 控制面 facade 收口成 `TEST-027` 的单一官方 gate，证明以下正式闭环真实成立：

- `payment.succeeded`
- `delivery.completed`
- `acceptance.passed`
- `dispute.escalated`

每类事件都必须完成：

`business event -> notification.requested -> dtp.notification.dispatch -> notification-worker -> mock-log`

并且留下：

- `ops.outbox_event(status=published)`
- `ops.system_log(message_text='notification sent via mock-log')`
- `audit.audit_event(action_name='notification.dispatch.sent')`

此外，本 task 还必须证明：

- `platform-core /api/v1/ops/notifications/audit/search` 是正式联查入口
- `notification-worker` 的 duplicate / retry / DLQ 不是假闭环
- `mock-log` 是 `V1` 唯一正式实接渠道

## Official Checker

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-notification-smoke.sh
```

checker 会依次执行：

1. `smoke-local.sh`
2. 启动宿主机 `outbox-publisher`
3. 启动宿主机 `notification-worker`
4. 运行：
   - `notif004_payment_success_notifications_db_smoke`
   - `notif005_delivery_completion_notifications_db_smoke`
   - `notif006_acceptance_outcome_notifications_db_smoke`
   - `notif007_dispute_settlement_notifications_db_smoke`
5. 通过 `platform-core /api/v1/ops/notifications/audit/search` 做正式控制面联查
6. 停掉背景 worker，再独立运行 `notif012_notification_worker_live_smoke`
7. 汇总 raw artifact 和联查结果，输出 `target/test-artifacts/notification-smoke/summary.json`

## Raw Artifacts

`TEST027_ARTIFACT_DIR` 默认写入：

- `notif004-payment-success.json`
- `notif005-delivery-completion.json`
- `notif006-acceptance-outcome.json`
- `notif007-dispute-settlement.json`
- `notif012-worker-live-smoke.json`
- `platform-payment-audit-search.json`
- `platform-dispute-audit-search.json`
- `platform-audit-lookups.json`
- `notification-worker.health.*.json`
- `notification-worker.metrics.prom`
- `outbox-publisher.health.*.json`

## Required Backchecks

最小验收必须同时满足：

- 业务通知四类主路径的 raw artifact 都带 `live_chain`
- `live_chain.outbox.published_count` 与 `mock_log.count`、`audit.count` 一致
- `platform-payment-audit-search.json` 至少命中 `payment.succeeded`
- `platform-dispute-audit-search.json` 至少命中 `dispute.escalated`
- `audit.audit_event(action_name='notification.dispatch.lookup')` 对两次 facade 查询各留一条记录
- `notif012-worker-live-smoke.json` 至少包含：
  - `payment.succeeded`
  - `delivery.completed`
  - `acceptance.passed`
  - `dispute.escalated`
  - duplicate / retry / dead-letter evidence

## Authority

- 事件 / topic / consumer group：`docs/开发准备/事件模型与Topic清单正式版.md`、`infra/kafka/topics.v1.json`
- 对外控制面路径：`packages/openapi/ops.yaml`、`docs/02-openapi/ops.yaml`
- worker 运行边界：`docs/04-runbooks/notification-worker.md`
- 验收矩阵：`docs/05-test-cases/notification-cases.md`
