# notification-worker

`NOTIF-001` 起始实现的正式通知进程。

- 正式消费 topic：`dtp.notification.dispatch`
- 对应事件：`notification.requested`
- 本地默认 consumer group：`cg-notification-worker`
- `V1` 实接渠道：`mock-log`
- 不直接消费：`dtp.outbox.domain-events`

当前实现能力：

- Kafka consumer 消费 `notification.requested`
- 共享 `notification-contract` 协议 crate，固定 scene catalog：
  - `order.created`
  - `payment.succeeded`
  - `payment.failed`
  - `order.pending_delivery`
  - `delivery.completed`
  - `order.pending_acceptance`
  - `acceptance.passed`
  - `acceptance.rejected`
  - `dispute.escalated`
  - `refund.completed`
  - `compensation.completed`
  - `settlement.frozen`
  - `settlement.resumed`
- payload 最小字段统一为：
  - `notification_code`
  - `template_code`
  - `channel`
  - `audience_scope`
  - `recipient`
  - `source_event`
  - `variables`
  - `metadata`
  - `retry_policy`
  - `subject_refs`
  - `links`
- `POST /internal/notifications/send` 手工注入通知事件到 Kafka
- 文件模板目录：`apps/notification-worker/templates/`
- Redis 短期状态与重试队列
- PostgreSQL 发送/审计/死信/trace 镜像
- 健康检查与指标端点：`/health/live`、`/health/ready`、`/health/deps`、`/metrics`

本地启动：

```bash
APP_PORT=8097 \
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 \
KAFKA_BROKERS=127.0.0.1:9094 \
cargo run -p notification-worker
```
