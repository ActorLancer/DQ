# Notification Worker 本地运行与排障（NOTIF-011）

## 口径冻结

- 正式进程名：`notification-worker`
- 对应事件：`notification.requested`
- 正式消费 topic：`dtp.notification.dispatch`
- 本地默认 consumer group：`cg-notification-worker`
- 正式生产者：`platform-core.integration`
- 正式链路：`notification.requested -> dtp.notification.dispatch -> notification-worker`
- 不直接消费：`dtp.outbox.domain-events`
- `V1` 实接渠道：`mock-log`
- `email` / `webhook`：仅保留 provider / adapter 边界，不作为 `V1` 完成证据
- topic / consumer / retention 权威源：`infra/kafka/topics.v1.json`
- route authority 权威源：`ops.event_route_policy`
- `portal-web / console-web` 不得直连 `notification-worker`；通知联查 / replay 的对外正式入口固定为 `platform-core` facade：
  - `POST /api/v1/ops/notifications/audit/search`
  - `POST /api/v1/ops/notifications/dead-letters/{dead_letter_event_id}/replay`
- `notification-worker /internal/notifications/*` 是内部执行契约，只供 `platform-core` 或运维 smoke 使用，不是浏览器目标。

## 当前批次边界

- 当前 runbook 是 `NOTIF-011` 的正式交付物，目标是把已实现的通知链路整理成可执行手册，而不是替代运行时实现本身。
- 当前文档必须与真实运行态一致：topic、consumer group、模板版本、联查入口、replay 入口、审计与观测链路都要能实际回查。
- 当前文档不替代后续承接项：
  - `NOTIF-013`：已将通知联查 / 模板预览 / 手工注入 / 人工补发 OpenAPI 归档同步到 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml`
  - `NOTIF-014`：已将运行态验收矩阵补齐到 `docs/05-test-cases/notification-cases.md`

## 运行前核对

1. 启动本地基础设施：
   - `make up-local`
2. 启动 worker：
   ```bash
   APP_PORT=8097 \
   DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
   REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 \
   KAFKA_BROKERS=127.0.0.1:9094 \
   TOPIC_NOTIFICATION_DISPATCH=dtp.notification.dispatch \
   TOPIC_DEAD_LETTER_EVENTS=dtp.dead-letter \
   cargo run -p notification-worker
   ```
3. 若要验证正式通知控制面 facade，再单独启动 `platform-core`：
   ```bash
   APP_PORT=8080 \
   DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
   REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/0 \
   KAFKA_BROKERS=127.0.0.1:9094 \
   NOTIFICATION_WORKER_BASE_URL=http://127.0.0.1:8097 \
   cargo run -p platform-core-bin
   ```
4. 校验 canonical topic 与 route seed：
   - `./scripts/check-topic-topology.sh`
   - `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
   - `psql postgresql://datab:datab_local_pass@127.0.0.1:5432/datab -c "select aggregate_type, event_type, target_topic, consumer_group_hint, status from ops.event_route_policy where target_topic='dtp.notification.dispatch';"`
5. 校验健康检查与依赖：
   - `curl -sS http://127.0.0.1:8097/health/live`
   - `curl -sS http://127.0.0.1:8097/health/ready`
   - `curl -sS http://127.0.0.1:8097/health/deps`
6. 校验观测栈入口：
   - `curl -sS http://127.0.0.1:8097/metrics | rg 'notification_worker_'`
   - `curl -G -sS http://127.0.0.1:9090/api/v1/query --data-urlencode 'query=up{job="notification-worker"}'`
   - `curl -G -sS http://127.0.0.1:9090/api/v1/query --data-urlencode 'query=notification_worker_events_total'`
   - `curl -sS http://127.0.0.1:9093/api/v2/status`
   - `curl -sS http://127.0.0.1:3000/api/health`
   - `curl -sS http://127.0.0.1:3100/ready`
   - `curl -sS http://127.0.0.1:3200/metrics | rg 'tempo_build_info'`
7. 自动化 live smoke（`NOTIF-012`）：
   - 执行前先停止任何已在运行的 `notification-worker` 进程；该 smoke 会以同一 `SERVICE_NAME` 订阅 `dtp.notification.dispatch`，不应与常驻实例并发抢消费。
   - 运行命令：
     ```bash
     NOTIF_WORKER_DB_SMOKE=1 \
     DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
     REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 \
     KAFKA_BROKERS=127.0.0.1:9094 \
     cargo test -p notification-worker notif012_notification_worker_live_smoke -- --nocapture
     ```
   - smoke 会真实验证：
     - `notification.requested -> dtp.notification.dispatch -> notification-worker`
     - `payment.succeeded / delivery.completed / acceptance.rejected / dispute.escalated`
     - duplicate dedupe
     - retry success
     - DLQ + `dtp.dead-letter`
     - PostgreSQL / Redis / audit / metrics 留痕

## 正式发送策略

- `platform-core.integration` 通过共享 `notification-contract` 生成 `notification.requested` envelope，并以 `aggregate_type=notification.dispatch_request`、`target_topic=dtp.notification.dispatch` 写入 canonical outbox。
- `notification-worker` 只消费 `dtp.notification.dispatch`，不直接把 `dtp.outbox.domain-events` 作为正式消费入口。
- 模板权威源是 `ops.notification_template`：
  - 运行时只读取 `enabled=true AND status='active'` 的最新版本
  - 查询维度：`template_code + channel + language_code`
  - 指定语言缺失时回退到默认语言 `zh-CN`
  - 指定模板缺失时回退到 `DEFAULT_NOTIFICATION_V1`
- 文件目录 `apps/notification-worker/templates/` 只保留 local fallback，不是正式模板真相源。
- 发送适配器通过渠道注册表分发：
  - active：`mock-log -> mock-log-adapter`
  - reserved：`email`、`webhook`
- 同一通知的幂等键由以下字段稳定生成：
  - `notification_code`
  - `audience_scope`
  - `source_event.aggregate_type`
  - `source_event.aggregate_id`
  - `source_event.event_type`
  - `recipient(id/address)`
- 状态与留痕职责分工：
  - PostgreSQL：`ops.consumer_idempotency_record`、`ops.system_log`、`ops.trace_index`、`ops.dead_letter_event`、`ops.alert_event`、`audit.audit_event`
  - Redis：`datab:v1:notification:state:<event_id>`、`datab:v1:notification:retry-payload:<event_id>`、`datab:v1:notification:retry-queue`
  - Kafka：正式 topic `dtp.notification.dispatch` 与隔离 topic `dtp.dead-letter`
  - Loki / Tempo / Prometheus / Alertmanager / Grafana：运行态日志、trace、指标、告警和看板

## 事件来源与模板清单

| notification_code | 冻结 source_event | audience / 触发说明 | template_code | 当前模板状态 |
| --- | --- | --- | --- | --- |
| `order.created` | `trade.order / trade.order.created` | scene catalog 已冻结；当前仍使用通用模板语义 | `NOTIFY_ORDER_CREATED_V1` | `version 1 active` |
| `payment.succeeded` | `billing.billing_event / billing.event.recorded` | 买方支付成功通知 | `NOTIFY_PAYMENT_SUCCEEDED_V1` | `version 2 active` |
| `payment.failed` | `payment.payment_intent / payment.intent_failed` | scene catalog 已冻结；当前仍使用通用模板语义 | `NOTIFY_PAYMENT_FAILED_V1` | `version 1 active` |
| `order.pending_delivery` | `trade.order_main / order.state_changed` | 卖方 / 运营在支付成功后待交付 | `NOTIFY_PENDING_DELIVERY_V1` | `version 2 active` |
| `delivery.completed` | `delivery.delivery_record / delivery.committed`；查询结果场景为 `delivery.query_execution_run / delivery.template_query.use` | 共享 / API / 查询结果 / 沙箱开通后的买方，以及卖方 / 运营交付完成 | `NOTIFY_DELIVERY_COMPLETED_V1` | `version 2 active` |
| `order.pending_acceptance` | `delivery.delivery_record / delivery.committed` | 文件包 / 报告交付后的买方待验收 | `NOTIFY_PENDING_ACCEPTANCE_V1` | `version 2 active` |
| `acceptance.passed` | `trade.acceptance_record / acceptance.passed` | buyer / seller / ops 验收通过 | `NOTIFY_ACCEPTANCE_PASSED_V1` | `version 2 active` |
| `acceptance.rejected` | `trade.acceptance_record / acceptance.rejected` | buyer / seller / ops 拒收 | `NOTIFY_ACCEPTANCE_REJECTED_V1` | `version 2 active` |
| `dispute.escalated` | `support.dispute_case / dispute.created` | buyer / seller / ops 争议升级 | `NOTIFY_DISPUTE_ESCALATED_V1` | `version 2 active` |
| `refund.completed` | `billing.billing_event / billing.event.recorded` | buyer / seller / ops 退款完成 | `NOTIFY_REFUND_COMPLETED_V1` | `version 2 active` |
| `compensation.completed` | `billing.billing_event / billing.event.recorded` | buyer / seller / ops 赔付完成 | `NOTIFY_COMPENSATION_COMPLETED_V1` | `version 2 active` |
| `settlement.frozen` | `billing.billing_event / billing.event.recorded`，且 `event_source=settlement_dispute_hold` | buyer / seller / ops 结算冻结 | `NOTIFY_SETTLEMENT_FROZEN_V1` | `version 2 active` |
| `settlement.resumed` | `billing.billing_event / billing.event.recorded`，且 `event_source=settlement_dispute_release` | buyer / seller / ops 恢复结算；不再使用 `support.dispute_case / dispute.resolved` | `NOTIFY_SETTLEMENT_RESUMED_V1` | `version 2 active` |

运行态模板回查：

```sql
select template_code, channel, language_code, version_no, enabled, status
from ops.notification_template
where template_code like 'NOTIFY_%'
order by template_code, version_no;
```

披露边界补充：

- `buyer / seller` 正文只允许展示订单、商品、金额、状态和动作入口。
- `ops` payload 可带联查字段，例如 `billing_event_id`、`payment_intent_id`、`provider_reference_id`、`delivery_ref_*`、`acceptance_record_id`、`liability_type`、`freeze_ticket_id`、`legal_hold_id`。
- 不允许把内部风控、审计敏感字段直接透传到业务用户通知正文。

## 手工操作入口

### 模板预览

- 入口：`POST /internal/notifications/templates/preview`
- 边界：当前保留为 `notification-worker` 运维 / smoke 内部入口，不是浏览器或 console 的正式直连目标。
- 最小请求示例：

```json
{
  "notification_code": "payment.succeeded",
  "audience_scope": "buyer",
  "template_code": "NOTIFY_PAYMENT_SUCCEEDED_V1",
  "recipient": {
    "kind": "user",
    "address": "buyer@example.test",
    "display_name": "Buyer"
  },
  "variables": {
    "subject": "支付成功通知",
    "headline": "买方托管已完成，订单进入待交付",
    "order_id": "11111111-1111-1111-1111-111111111111",
    "order_no": "ORD-NOTIF-011",
    "product_title": "Example SKU",
    "seller_org_name": "Example Seller Org",
    "order_amount": "128.00",
    "currency_code": "CNY",
    "payment_status": "buyer_locked",
    "delivery_status": "pending_delivery",
    "action_label": "查看订单",
    "action_href": "/trade/orders/11111111-1111-1111-1111-111111111111",
    "buyer_locked_at": "2026-04-22T00:00:00Z"
  },
  "source_event": {
    "aggregate_type": "billing.billing_event",
    "aggregate_id": "11111111-1111-1111-1111-111111111111",
    "event_type": "billing.event.recorded",
    "target_topic": "dtp.outbox.domain-events"
  },
  "subject_refs": [
    {
      "ref_type": "order",
      "ref_id": "11111111-1111-1111-1111-111111111111"
    }
  ],
  "links": [
    {
      "link_code": "order_detail",
      "href": "/trade/orders/11111111-1111-1111-1111-111111111111"
    }
  ]
}
```

- 响应至少检查：
  - `template_code`
  - `language_code`
  - `version_no`
  - `template_fallback_used`
  - `body_fallback_used`
  - `variable_schema`
  - `title`
  - `body`

### 手工注入

- 入口：`POST /internal/notifications/send`
- 边界：当前保留为 `notification-worker` 运维 / smoke 内部入口，不是浏览器或 console 的正式直连目标。
- 建议显式传入：
  - `notification_code`
  - `audience_scope`
  - `source_event`
  - `subject_refs`
  - `links`
  - `retry_policy`
- 若要验证重试，可在请求顶层注入 local-only 故障注入字段 `simulate_failures`：

```json
{
  "notification_code": "payment.succeeded",
  "audience_scope": "buyer",
  "template_code": "NOTIFY_PAYMENT_SUCCEEDED_V1",
  "recipient": {
    "kind": "user",
    "address": "buyer@example.test",
    "display_name": "Buyer"
  },
  "variables": {
    "subject": "支付成功通知",
    "headline": "买方托管已完成，订单进入待交付",
    "order_id": "22222222-2222-2222-2222-222222222222",
    "order_no": "ORD-NOTIF-011-RETRY",
    "product_title": "Example SKU",
    "seller_org_name": "Example Seller Org",
    "order_amount": "128.00",
    "currency_code": "CNY",
    "payment_status": "buyer_locked",
    "delivery_status": "pending_delivery",
    "action_label": "查看订单",
    "action_href": "/trade/orders/22222222-2222-2222-2222-222222222222",
    "buyer_locked_at": "2026-04-22T00:00:00Z"
  },
  "simulate_failures": 3,
  "retry_policy": {
    "max_attempts": 2,
    "backoff_ms": 1000
  },
  "source_event": {
    "aggregate_type": "billing.billing_event",
    "aggregate_id": "22222222-2222-2222-2222-222222222222",
    "event_type": "billing.event.recorded",
    "target_topic": "dtp.outbox.domain-events"
  },
  "subject_refs": [
    {
      "ref_type": "order",
      "ref_id": "22222222-2222-2222-2222-222222222222"
    }
  ],
  "links": [
    {
      "link_code": "order_detail",
      "href": "/trade/orders/22222222-2222-2222-2222-222222222222"
    }
  ]
}
```

### 通知联查

- 正式对外入口：`POST /api/v1/ops/notifications/audit/search`
- 内部执行入口：`POST /internal/notifications/audit/search`
- 必须提供：
  - 至少一个主过滤条件：`order_id / case_id / aggregate_type / event_type / target_topic / template_code / notification_code / event_id`
  - `reason`
  - `x-step-up-token` 或 `x-step-up-challenge-id`
- 浏览器 / console 只能访问 `platform-core` facade；`platform-core` 负责权限、step-up、审计和 header 归一化，再转发到 worker。
- `aggregate_type / event_type / target_topic` 针对正式通知 envelope / canonical outbox 路由，即：
  - `aggregate_type=notification.dispatch_request`
  - `event_type=notification.requested`
  - `target_topic=dtp.notification.dispatch`
- 它们不是 `source_event.aggregate_type / source_event.event_type / source_event.target_topic` 的别名。
- 最小请求示例：

```json
{
  "order_id": "11111111-1111-1111-1111-111111111111",
  "reason": "trace notification delivery for incident review"
}
```

- 响应优先检查：
  - `records[].notification_code`
  - `records[].template_code`
  - `records[].audience_scope`
  - `records[].rendered_variables`
  - `records[].channel_result`
  - `records[].retry_timeline[].status`
  - `records[].audit_timeline[].action_name`
  - `records[].dead_letter.reprocess_status`

### 人工补发 / replay

1. 先定位目标 dead letter：
   ```sql
   select dead_letter_event_id, event_type, failed_reason, reprocess_status, created_at
   from ops.dead_letter_event
   where target_topic = 'dtp.dead-letter'
   order by created_at desc
   limit 20;
   ```
2. 先做 dry-run：
   ```bash
   curl -sS -X POST \
     http://127.0.0.1:8080/api/v1/ops/notifications/dead-letters/<dead_letter_event_id>/replay \
     -H 'content-type: application/json' \
     -H 'x-login-id: local-platform-admin' \
     -H 'x-role: platform_admin' \
     -H 'x-step-up-token: step-up-local-1' \
     -H 'x-idempotency-key: notif-replay-dry-run-001' \
     -d '{"dry_run":true,"reason":"manual replay after mock-log recovery"}'
   ```
3. 回查 dry-run 留痕：
   - `ops.system_log.message_text='notification dead letter replay dry-run prepared'`
   - `audit.audit_event.action_name='notification.dispatch.reprocess.dry_run'`
4. 执行正式 replay：
   ```bash
   curl -sS -X POST \
     http://127.0.0.1:8080/api/v1/ops/notifications/dead-letters/<dead_letter_event_id>/replay \
     -H 'content-type: application/json' \
     -H 'x-login-id: local-platform-admin' \
     -H 'x-role: platform_admin' \
     -H 'x-step-up-token: step-up-local-1' \
     -H 'x-idempotency-key: notif-replay-apply-001' \
     -d '{"dry_run":false,"reason":"manual replay after mock-log recovery"}'
   ```
5. 回查 replay 结果：
  - 原 `ops.dead_letter_event.reprocess_status`：`not_reprocessed -> reprocess_requested -> reprocessed`
  - 新 replay event 返回新的 `event_id / request_id / trace_id`
  - replay 成功后，`ops.consumer_idempotency_record.result_code=processed`
  - `audit.audit_event` 追加 `notification.dispatch.reprocess.requested`
6. 若要直接排查 worker 内部执行面，可再使用 `POST /internal/notifications/dead-letters/{dead_letter_event_id}/replay` 做服务到服务 smoke；该入口不对浏览器暴露。

## 失败排查

### 1. 没有消息进入 worker

- 先查 topology 与 route seed：
  - `./scripts/check-topic-topology.sh`
  - `psql ... -c "select aggregate_type, event_type, target_topic, consumer_group_hint, status from ops.event_route_policy where target_topic='dtp.notification.dispatch';"`
- 再查 Kafka topic 与 group：
  - `docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic dtp.notification.dispatch`
  - `docker exec datab-kafka /opt/kafka/bin/kafka-consumer-groups.sh --bootstrap-server localhost:9092 --describe --group cg-notification-worker`
- 若 `platform-core` 已落 outbox 但 topic 无消息，优先回查 `ops.outbox_event` 与 publisher，而不是把问题归咎于 worker。

### 2. 模板命中错误或正文变量缺失

- 先跑模板预览接口，确认命中的 `template_code / version_no / language_code`。
- 回查模板表：
  ```sql
  select template_code, channel, language_code, version_no, enabled, status
  from ops.notification_template
  where template_code in (
    'NOTIFY_PAYMENT_SUCCEEDED_V1',
    'NOTIFY_PENDING_DELIVERY_V1',
    'NOTIFY_DELIVERY_COMPLETED_V1',
    'NOTIFY_PENDING_ACCEPTANCE_V1',
    'NOTIFY_ACCEPTANCE_PASSED_V1',
    'NOTIFY_ACCEPTANCE_REJECTED_V1',
    'NOTIFY_REFUND_COMPLETED_V1',
    'NOTIFY_COMPENSATION_COMPLETED_V1',
    'NOTIFY_DISPUTE_ESCALATED_V1',
    'NOTIFY_SETTLEMENT_FROZEN_V1',
    'NOTIFY_SETTLEMENT_RESUMED_V1'
  )
  order by template_code, version_no;
  ```
- buyer / seller 正文出现内部联查字段时，优先检查 scene builder 是否错误透传了 `ops` 专属变量。

### 3. 重复通知或幂等异常

- 回查数据库幂等记录：
  ```sql
  select consumer_name, event_id, result_code, metadata, processed_at
  from ops.consumer_idempotency_record
  where consumer_name = 'cg-notification-worker'
    and event_id = '<event_id>'::uuid;
  ```
- 回查 Redis 状态：
  - `redis-cli -a datab_redis_pass -n 2 get datab:v1:notification:state:<event_id>`
- 同一 `event_id` 若被重复发送，预期结果是：
  - `notification_worker_events_total{result="duplicate"}` 增长
  - `ops.system_log` 不新增第二条 `notification sent via mock-log`

### 4. 持续重试或 backlog 不下降

- 回查 Redis：
  - `redis-cli -a datab_redis_pass -n 2 llen datab:v1:notification:retry-queue`
  - `redis-cli -a datab_redis_pass -n 2 get datab:v1:notification:state:<event_id>`
  - `redis-cli -a datab_redis_pass -n 2 get datab:v1:notification:retry-payload:<event_id>`
- 回查指标：
  - `curl -sS http://127.0.0.1:8097/metrics | rg 'notification_worker_(events_total|send_total|retry_queue_depth)'`
  - `curl -G -sS http://127.0.0.1:9090/api/v1/query --data-urlencode 'query=notification_worker_retry_queue_depth'`
- 若 backlog 常驻大于 `0`，同步检查：
  - `audit.audit_event.action_name='notification.dispatch.retry_scheduled'`
  - Alertmanager 中是否已有 `NotificationRetryQueueBacklog`

### 5. 已进入 dead letter

- 回查 PostgreSQL：
  ```sql
  select dead_letter_event_id, event_type, failed_reason, failure_stage, reprocess_status, created_at
  from ops.dead_letter_event
  where payload->>'event_id' = '<event_id>'
  order by created_at desc;
  ```
- 回查告警：
  ```sql
  select fingerprint, alert_type, severity, status, metadata, fired_at
  from ops.alert_event
  where trace_id = '<trace_id>'
  order by fired_at desc;
  ```
- 回查 Kafka DLQ：
  - `docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic dtp.dead-letter --from-beginning --timeout-ms 5000`
- 若已进入 DLQ，先定位失败原因，再按 replay 流程操作；不要直接改数据库状态冒充恢复。

### 6. 审计、日志与观测联查

- PostgreSQL 联查：
  ```sql
  select message_text, structured_payload, created_at
  from ops.system_log
  where object_id = '<event_id>'::uuid
  order by created_at desc;
  ```
  ```sql
  select root_span_name, status, metadata, created_at
  from ops.trace_index
  where object_id = '<event_id>'::uuid
  order by created_at desc;
  ```
  ```sql
  select action_name, result_code, metadata, event_time
  from audit.audit_event
  where trace_id = '<trace_id>'
  order by event_time desc;
  ```
- Loki / Tempo / Prometheus / Grafana / Alertmanager：
  - Loki：查 `service_name=notification-worker` 与 `request_id / trace_id / object_id`
  - Tempo：按 `trace_id` 查 `notification.dispatch` 或 `notification.retrying`
  - Prometheus：查 `notification_worker_events_total`、`notification_worker_send_total`、`notification_worker_retry_queue_depth`
  - Grafana：`Platform Overview` 中的 `Notification Events / Notification Sends / Notification Retry Queue Depth`
  - Alertmanager：确认 `NotificationRetryQueueBacklog` 与相关链路告警可见

## 当前承接关系

- `NOTIF-013` 已把当前 runbook 中已经冻结的模板预览 / 手工注入 / 联查 / replay 入口补齐到 `packages/openapi/ops.yaml` 与 `docs/02-openapi/ops.yaml`，并明确 `aggregate_type / event_type / target_topic` 过滤口径对应正式通知 envelope。
- `NOTIF-014` 已把当前 runbook 中的运行态验证路径补成 `docs/05-test-cases/notification-cases.md`，覆盖：
  - 支付成功
  - 交付完成
  - 验收通过 / 拒收
  - 争议升级
  - 结算冻结 / 恢复
  - 重复去重
  - 失败重试
  - DLQ
  - 人工补发
- 当前阶段若需临时人工排障，应以本 runbook + `apps/notification-worker/README.md` + `docs/04-runbooks/kafka-topics.md` 为准，不得再引入新的旁路 topic、旧进程名或 README 占位口径。
- 当前阶段若需执行正式通知验收，应优先组合使用本 runbook 与 `docs/05-test-cases/notification-cases.md`，而不是自造临时 smoke 步骤。
