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
- PostgreSQL 模板权威源：`ops.notification_template`
  - 字段至少包含：`template_code`、`language_code`、`channel`、`version_no`、`enabled`、`status`、`variables_schema_json`、`title_template`、`body_template`、`fallback_body_template`
  - `notification-worker` 运行时优先从 PostgreSQL 读取启用中的最新模板版本；`apps/notification-worker/templates/` 只保留本地 file fallback
- `NOTIF-004` 起，支付成功链路的正式模板分工固定为：
  - 买方：`payment.succeeded / NOTIFY_PAYMENT_SUCCEEDED_V1`
  - 卖方：`order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
  - 运营：`order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
  - 买方/卖方正文只允许展示订单、商品、金额、状态与操作入口；`payment_intent_id / provider_reference_id / 内部联查字段` 仅允许进入运营视图
- `NOTIF-005` 起，交付完成链路的正式模板分工固定为：
  - 文件包、报告交付后的买方：`order.pending_acceptance / NOTIFY_PENDING_ACCEPTANCE_V1`
  - 共享开通、API 开通、查询结果可取、沙箱开通后的买方：`delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
  - 卖方、运营：统一 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
  - `ops` 正文允许附带 `delivery_ref_type / delivery_ref_id / receipt_hash / delivery_commit_hash`；`buyer/seller` 正文不得透传这些内部联查字段
- `NOTIF-006` 起，验收 / 退款 / 赔付链路的正式模板分工固定为：
  - 验收通过：buyer / seller / ops 统一接收 `acceptance.passed / NOTIFY_ACCEPTANCE_PASSED_V1`
  - 拒收：buyer / seller / ops 统一接收 `acceptance.rejected / NOTIFY_ACCEPTANCE_REJECTED_V1`
  - 退款完成：buyer / seller / ops 统一接收 `refund.completed / NOTIFY_REFUND_COMPLETED_V1`
  - 赔付完成：buyer / seller / ops 统一接收 `compensation.completed / NOTIFY_COMPENSATION_COMPLETED_V1`
  - 验收链路 source-event 固定为 `trade.acceptance_record / acceptance.passed|acceptance.rejected`
  - 退款 / 赔付链路 source-event 固定为 `billing.billing_event / billing.event.recorded`
  - 动作入口固定落到订单详情、争议提交或账单退款页；`ops` 正文允许带 `acceptance_record_id / provider_* / liability_type / resolution_ref_*` 联查字段，`buyer/seller` 正文不得透传这些内部字段
- `POST /internal/notifications/send` 手工注入通知事件到 Kafka
- `POST /internal/notifications/templates/preview` 预览模板渲染结果，返回解析后的语言、版本、schema 与 fallback 使用情况
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
