# Notification Worker 本地运行与排障（NOTIF-011）

## 口径冻结

- 正式进程名：`notification-worker`
- 对应事件：`notification.requested`
- 正式消费 topic：`dtp.notification.dispatch`
- 本地默认 consumer group：`cg-notification-worker`
- 不直接消费：`dtp.outbox.domain-events`
- `V1` 实接渠道：`mock-log`
- `email` / `webhook`：仅保留 provider 边界，不作为 `V1` 必须实接项

## 当前批次边界

- 本批次只冻结命名、topic、consumer group、渠道边界与排障口径，不代表 `notification-worker` 已完成正式实现。
- 当前文档结论只能回答“通知事件应该怎么走”，不能替代后续代码实现所需的 OpenAPI、发送记录模型、集成测试与 smoke 结果。
- 进入 `NOTIF` 代码实现批次后，Agent 必须同步补齐：
  - `packages/openapi/ops.yaml` 中与通知联查相关的控制面示例（`NOTIF-013`）
  - `docs/02-openapi/ops.yaml` 归档（`NOTIF-013`）
  - `docs/05-test-cases/notification-cases.md`（`NOTIF-014`）
  - runbook 中的模板清单、人工补发步骤、失败重试阈值与联查入口

## 事件来源

- 主来源：`platform-core.integration`
- 冻结链路：`notification.requested -> dtp.notification.dispatch -> notification-worker`
- `dtp.outbox.domain-events` 仅保留为通用主领域事件流，不作为 `notification-worker` 的正式消费入口
- topic 定义权威源：`infra/kafka/topics.v1.json`
- topic 初始化脚本：`infra/kafka/init-topics.sh`

## 事件协议

- `platform-core.integration` 使用共享 `notification-contract` 生成 `notification.requested` payload，并统一写入 `notification.dispatch_request / dtp.notification.dispatch`。
- `NOTIF-004` 已冻结支付成功后的 audience 映射：
  - 买方接收 `payment.succeeded / NOTIFY_PAYMENT_SUCCEEDED_V1`
  - 卖方接收 `order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
  - 运营接收 `order.pending_delivery / NOTIFY_PENDING_DELIVERY_V1`
  - `buyer/seller` payload 只保留订单、商品、金额、状态和操作入口；`ops` payload 才允许附带 `billing_event_id / payment_intent_id / provider_reference_id / provider_result_source`
- `NOTIF-005` 已冻结交付完成后的 audience 映射：
  - 文件包、报告交付：买方接收 `order.pending_acceptance / NOTIFY_PENDING_ACCEPTANCE_V1`
  - 共享开通、API 开通、查询结果可取、沙箱开通：买方接收 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
  - 卖方、运营统一接收 `delivery.completed / NOTIFY_DELIVERY_COMPLETED_V1`
  - `ops` payload 允许附带 `delivery_ref_type / delivery_ref_id / receipt_hash / delivery_commit_hash`；`buyer/seller` payload 不得透传这些联查字段
- 当前冻结的 `notification_code` 仅允许：
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
- payload 最小字段：
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
- 幂等键统一由 `notification_code + audience_scope + source_event.aggregate_type + source_event.aggregate_id + source_event.event_type + recipient(id/address)` 生成。
- 后续 `NOTIF-004 ~ NOTIF-007` 只允许在这个 scene catalog 上补模板与业务触发逻辑，不允许再发明新的旁路 notification code。

## 本地启动

1. 启动基础设施：
   - `make up-local`
2. 启动通知进程：
   - 宿主机运行：
     `APP_PORT=8097 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 KAFKA_BROKERS=127.0.0.1:9094 cargo run -p notification-worker`
   - 容器示例：
     参考 `infra/docker/docker-compose.apps.local.example.yml` 中 `notification-worker` 段
3. 校验 topic 已存在：
   - `dtp.notification.dispatch`
   - `dtp.dead-letter`
4. 校验通知 / Fabric 相关关键拓扑未漂移：
   - `./scripts/check-topic-topology.sh`
   - 该脚本覆盖关键静态 topology / route seed，并回查当前数据库 `ops.event_route_policy` 中 `notification.requested -> dtp.notification.dispatch` 等运行态路由；若要验证全量 canonical topics 是否真实存在，仍需额外执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
5. 校验 worker 健康与指标：
   - `GET http://127.0.0.1:8097/health/live`
   - `GET http://127.0.0.1:8097/health/ready`
   - `GET http://127.0.0.1:8097/health/deps`
   - `GET http://127.0.0.1:8097/metrics`
   - `GET 'http://127.0.0.1:9090/api/v1/query?query=up{job="notification-worker"}'`
   - `GET 'http://127.0.0.1:9090/api/v1/query?query=notification_worker_events_total'`
   - `GET 'http://127.0.0.1:9090/api/v1/query?query=notification_worker_send_total'`
   - Grafana 查看 `Platform Overview` 中的 `Notification Events / Notification Sends / Notification Retry Queue Depth`
   - Alertmanager 规则中应存在 `NotificationRetryQueueBacklog`
6. 手工注入一条 `notification.requested` 事件：
   - `POST http://127.0.0.1:8097/internal/notifications/send`
   - 建议显式传入 `notification_code`、`audience_scope`、`source_event`、`subject_refs`、`links`

## V1 渠道与模板边界

- 默认只允许 `mock-log` 渠道输出发送结果
- 模板需支持：
  - 模板编码
  - 语言
  - 变量 schema
  - 版本号
  - 启用状态
  - 变量渲染
  - 渲染预览
  - fallback 文案
- PostgreSQL 权威表：`ops.notification_template`
  - 运行时只读取 `enabled=true AND status='active'` 的最新版本
  - 查询维度：`template_code + channel + language_code`
  - 若指定语言无匹配，则回退到默认语言 `zh-CN`
  - 若指定模板缺失，则回退到 `DEFAULT_NOTIFICATION_V1`
- `076_notification_payment_success_pending_delivery_templates.sql` 起：
  - `NOTIFY_PAYMENT_SUCCEEDED_V1` version `2` 作为买方支付成功正式模板
  - `NOTIFY_PENDING_DELIVERY_V1` version `2` 作为卖方 / 运营待交付正式模板
  - version `1` 已归档，仅保留回退审计用途
- `077_notification_delivery_completed_pending_acceptance_templates.sql` 起：
  - `NOTIFY_DELIVERY_COMPLETED_V1` version `2` 作为共享/API/查询结果/沙箱开通，以及卖方/运营交付完成正式模板
  - `NOTIFY_PENDING_ACCEPTANCE_V1` version `2` 作为文件包/报告交付后的买方待验收正式模板
  - version `1` 已归档，仅保留回退审计用途
- `078_notification_acceptance_resolution_templates.sql` 起：
  - `NOTIFY_ACCEPTANCE_PASSED_V1` version `2` 作为验收通过正式模板
  - `NOTIFY_ACCEPTANCE_REJECTED_V1` version `2` 作为拒收正式模板
  - `NOTIFY_REFUND_COMPLETED_V1` version `2` 作为退款完成正式模板
  - `NOTIFY_COMPENSATION_COMPLETED_V1` version `2` 作为赔付完成正式模板
  - version `1` 已归档，仅保留回退审计用途
- `079_acceptance_event_route_policy.sql` 起：
  - `trade.acceptance_record / acceptance.passed`
  - `trade.acceptance_record / acceptance.rejected`
  已进入 `ops.event_route_policy`，用于把验收链 canonical outbox 正式桥接到通知链路
- file 模板目录 `apps/notification-worker/templates/` 仅保留为 local fallback，不再作为正式模板权威源
- 不允许把内部风控、审计敏感字段直接透传到业务用户通知正文

## NOTIF-005 交付链路联调

- 六类交付结果的 producer 入口：
  - 文件包：`delivery.delivery_record / delivery.committed`
  - 共享开通：`delivery.delivery_record / delivery.committed`
  - API 开通：`delivery.delivery_record / delivery.committed`
  - 查询结果可取：`delivery.query_execution_run / delivery.template_query.use`
  - 沙箱开通：`delivery.delivery_record / delivery.committed`
  - 报告交付：`delivery.delivery_record / delivery.committed`
- 运行态验证建议至少覆盖三条样例：
  - `order.pending_acceptance / buyer / NOTIFY_PENDING_ACCEPTANCE_V1`
  - `delivery.completed / ops / NOTIFY_DELIVERY_COMPLETED_V1`
  - `delivery.completed` 强制失败一次后重试成功
- 手工验证步骤：
  1. `POST /internal/notifications/templates/preview` 先确认两套模板都命中 version `2`
  2. `POST /internal/notifications/send` 注入待验收样例，回查：
     - `ops.consumer_idempotency_record.result_code=processed`
     - `ops.system_log.message_text='notification sent via mock-log'`
     - `ops.trace_index.root_span_name=notification.dispatch`
     - `audit.audit_event.action_name=notification.dispatch.sent`
  3. 使用同一 `event_id` 再注入一次交付完成样例，确认 `/metrics` 中 `notification_worker_events_total{result="duplicate"}` 增长，且数据库不新增第二条处理记录
  4. 用 `simulate_failures=1` + `retry_policy.max_attempts=2` 注入重试样例，确认：
     - Redis `datab:v1:notification:retry-queue` 深度先变为 `1` 再回到 `0`
     - Redis `datab:v1:notification:state:<event_id>` 先为 `retrying`，最终为 `processed`
     - `audit.audit_event` 先写 `notification.dispatch.retry_scheduled`，随后写 `notification.dispatch.sent`
     - `ops.trace_index` 同时存在 `notification.retrying` 与 `notification.dispatch`
  5. `GET 'http://127.0.0.1:9090/api/v1/query?query=notification_worker_events_total'` 与 `notification_worker_retry_queue_depth`，确认 Prometheus 已抓到 worker 指标
- 业务数据清理要求：
  - 清理本次手工样例产生的非 append-only 辅助状态，例如 Redis 短状态、重试载荷、`ops.consumer_idempotency_record`、`ops.trace_index`
  - `audit.audit_event` 按 append-only 保留

## NOTIF-006 验收 / 退款 / 赔付链路联调

- producer 入口与 source-event 冻结为：
  - 验收通过：`trade.acceptance_record / acceptance.passed`
  - 拒收：`trade.acceptance_record / acceptance.rejected`
  - 退款完成：`billing.billing_event / billing.event.recorded`
  - 赔付完成：`billing.billing_event / billing.event.recorded`
- action 链接冻结为：
  - 验收通过 buyer / seller：`/trade/orders/:orderId`
  - 验收通过 ops：`/billing?order_id=:orderId`
  - 拒收 buyer / ops：`/support/cases/new?order_id=:orderId`
  - 拒收 seller：`/trade/orders/:orderId`
  - 退款完成 / 赔付完成 buyer / seller / ops：`/billing/refunds?order_id=:orderId&case_id=:caseId`
- 运行态验证建议至少覆盖四条样例：
  - `acceptance.passed / buyer / NOTIFY_ACCEPTANCE_PASSED_V1`
  - `acceptance.rejected / ops / NOTIFY_ACCEPTANCE_REJECTED_V1`
  - `refund.completed / seller / NOTIFY_REFUND_COMPLETED_V1` 强制失败一次后重试成功
  - `compensation.completed / ops / NOTIFY_COMPENSATION_COMPLETED_V1` 重试耗尽后进入 `dtp.dead-letter`
- 手工验证步骤：
  1. `POST /internal/notifications/templates/preview`，确认四套模板都命中 version `2`，正文分别带出订单详情、争议提交或账单退款链接。
  2. `POST /internal/notifications/send` 注入 `acceptance.passed` 样例，回查：
     - `ops.consumer_idempotency_record.result_code=processed`
     - `ops.system_log.structured_payload.body` 带 `/trade/orders/:orderId`
     - `audit.audit_event.action_name=notification.dispatch.sent`
     - `ops.trace_index.root_span_name=notification.dispatch`
  3. 注入 `acceptance.rejected` 样例并用同一 `event_id` 再发一次，确认：
     - `/metrics` 与 Prometheus 中 `notification_worker_events_total{result="duplicate"}` 增长
     - `ops.system_log` 对应 `object_id=<event_id>` 仍只保留一次发送记录
     - `ops` 正文带 `acceptance_record_id` 与争议入口，未额外产生第二次发送
  4. 注入 `refund.completed` 样例，使用 `simulate_failures=1` 与 `retry_policy.max_attempts=2, backoff_ms=4000`，确认：
     - Redis `datab:v1:notification:retry-queue` 深度先为 `1`，最终回到 `0`
     - Redis `datab:v1:notification:state:<event_id>` 先为 `retrying`，最终为 `processed`
     - `audit.audit_event` 先写 `notification.dispatch.retry_scheduled`，随后写 `notification.dispatch.sent`
     - `ops.trace_index` 同时存在 `notification.retrying` 与 `notification.dispatch`
  5. 注入 `compensation.completed` 样例，使用 `simulate_failures=3` 与 `retry_policy.max_attempts=2`，确认：
     - `ops.consumer_idempotency_record.result_code=dead_lettered`
     - `ops.dead_letter_event.target_topic=dtp.dead-letter`
     - `ops.alert_event.alert_type=notification_dead_letter`
     - `audit.audit_event` 写入 `notification.dispatch.dead_lettered`
  6. `GET 'http://127.0.0.1:9090/api/v1/query?query=up{job="notification-worker"}'`
     与 `notification_worker_events_total`、`notification_worker_send_total`、`notification_worker_retry_queue_depth`，确认 Prometheus 已抓到当前 worker 指标。
  7. `GET http://127.0.0.1:9093/api/v2/status` 与 Grafana `Platform Overview` dashboard，确认 Alertmanager / Grafana 运行态可联查；Prometheus rules 中应存在 `NotificationRetryQueueBacklog`。
- 业务数据清理要求：
  - 清理本次手工样例产生的 Redis 短状态、`ops.consumer_idempotency_record`、`ops.trace_index`、`ops.alert_event`、`ops.dead_letter_event`
  - `audit.audit_event` 与 `ops.system_log` 按 append-only 保留

## 模板预览

- 内部预览入口：`POST /internal/notifications/templates/preview`
- 预览请求建议显式提供：
  - `template_code`
  - `notification_code`
  - `audience_scope`
  - `language_code`
  - `recipient`
  - `variables`
  - `source_event`
- 预览响应至少返回：
  - `template_code`
  - `channel`
  - `language_code`
  - `requested_language_code`
  - `version_no`
  - `template_fallback_used`
  - `body_fallback_used`
  - `variable_schema`
  - `title`
  - `body`

## 幂等、重试、DLQ

- 同一幂等键事件必须只发送一次
- 失败消息进入重试流程；超过阈值转入 `dtp.dead-letter`
- 人工重放必须保留审计轨迹并可按事件 ID 回查
- 相关 `ops.event_route_policy` 缺失或漂移时，先检查 `notification.dispatch_request / notification.requested -> dtp.notification.dispatch`

## 联查建议

- 按 `order_id` / `case_id` / `template_code` 联查：
  - 发送记录
  - 渲染变量快照
  - 渠道结果
  - 重试轨迹
  - 关联事件 ID
- 交付完成链路优先补查：
  - `ops.notification_template` 中 `NOTIFY_DELIVERY_COMPLETED_V1 / NOTIFY_PENDING_ACCEPTANCE_V1` 的 active version 是否为 `2`
  - 查询结果场景是否使用 `delivery.query_execution_run / delivery.template_query.use`
  - `ops` 正文是否包含 `delivery_ref_* / *_hash`，而 `buyer/seller` 正文不包含
- 验收 / 退款 / 赔付链路优先补查：
  - `ops.notification_template` 中 `NOTIFY_ACCEPTANCE_PASSED_V1 / NOTIFY_ACCEPTANCE_REJECTED_V1 / NOTIFY_REFUND_COMPLETED_V1 / NOTIFY_COMPENSATION_COMPLETED_V1` 的 active version 是否为 `2`
  - `ops.event_route_policy` 是否存在 `trade.acceptance_record / acceptance.passed|acceptance.rejected -> notification.requested -> dtp.notification.dispatch`
  - `ops` 正文是否包含 `acceptance_record_id / provider_* / liability_type / resolution_ref_*`，而 `buyer/seller` 正文不包含这些联查字段

## 常见问题

- 现象：收不到通知
  - 检查 `dtp.notification.dispatch` 是否有消息
  - 检查 consumer group 是否为 `cg-notification-worker`
  - 检查幂等键是否命中去重
- 现象：持续重试
  - 检查模板渲染变量是否完整
  - 检查 provider 返回码与重试策略阈值
