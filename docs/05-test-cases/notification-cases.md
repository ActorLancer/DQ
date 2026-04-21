# Notification Cases

## Scope

本文件冻结 `NOTIF` 阶段在 `V1` 的最小可信通知验收面，目标是证明通知链路已经形成真实闭环，而不是只输出日志或只证明 outbox 有行存在。

本文件覆盖以下正式口径：

- 正式进程名：`notification-worker`
- 正式链路：`notification.requested -> dtp.notification.dispatch -> notification-worker`
- `V1` 正式实接渠道：`mock-log`
- `email` / `webhook`：仅保留 provider / adapter 边界，不作为 `V1` 完成证据
- 正式联查入口：`POST /internal/notifications/audit/search`
- 正式人工补发入口：`POST /internal/notifications/dead-letters/{dead_letter_event_id}/replay`

通知主权威源与辅助状态边界如下：

- PostgreSQL：`ops.outbox_event`、`ops.notification_template`、`ops.system_log`、`ops.consumer_idempotency_record`、`ops.dead_letter_event`、`audit.audit_event`
- Redis：`datab:v1:notification:state:<event_id>`、`datab:v1:notification:retry-payload:<event_id>`、`datab:v1:notification:retry-queue`
- Kafka：`dtp.notification.dispatch`、`dtp.dead-letter`
- 观测：Prometheus / Alertmanager / Loki / Tempo / Grafana

## Frozen Boundaries

- `notification-worker` 只消费 `dtp.notification.dispatch`，不直接把 `dtp.outbox.domain-events` 当正式通知消费入口。
- `platform-core.integration` 必须以 `aggregate_type=notification.dispatch_request`、`event_type=notification.requested`、`target_topic=dtp.notification.dispatch` 写入 canonical outbox。
- 模板真相源是 `ops.notification_template`；文件目录 `apps/notification-worker/templates/` 只作为 local fallback。
- 宿主机本地验收默认使用：
  - Kafka：`127.0.0.1:9094`
  - PostgreSQL：`127.0.0.1:5432`
  - Redis：`127.0.0.1:6379`
  - Keycloak：`127.0.0.1:8081`
- `./scripts/check-topic-topology.sh` 只校验关键 topology / route seed；若要证明 Kafka canonical topics 真实存在，必须额外执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。

## Common Preconditions

1. 载入本地运行时入口：
   ```bash
   set -a
   source infra/docker/.env.local
   set +a
   ```
2. 启动基础设施：`make up-local`
3. 校验 route seed 与 canonical topic：
   ```bash
   ./scripts/check-topic-topology.sh
   ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh
   psql "$DATABASE_URL" -c \
     "select aggregate_type, event_type, target_topic, consumer_group_hint, status
        from ops.event_route_policy
       where target_topic = 'dtp.notification.dispatch';"
   ```
4. 验证 worker 健康与依赖：
   ```bash
   curl -sS http://127.0.0.1:8097/health/live
   curl -sS http://127.0.0.1:8097/health/ready
   curl -sS http://127.0.0.1:8097/health/deps
   ```
5. 观测栈至少回查一次：
   ```bash
   curl -sS http://127.0.0.1:8097/metrics | rg 'notification_worker_'
   curl -G -sS http://127.0.0.1:9090/api/v1/query \
     --data-urlencode 'query=notification_worker_events_total'
   curl -sS http://127.0.0.1:9093/api/v2/status
   curl -sS http://127.0.0.1:3100/ready
   curl -sS http://127.0.0.1:3200/metrics | rg 'tempo_build_info'
   curl -sS http://127.0.0.1:3000/api/health
   ```

## Cross-Cut Acceptance Checklist

| 验收项 | 必须证明的事实 | 最低回查要求 |
| --- | --- | --- |
| 正式 topic / route | outbox 事件固定写入 `notification.requested + dtp.notification.dispatch`，且 `aggregate_type=notification.dispatch_request` | `ops.outbox_event` 与 `ops.event_route_policy` 双回查 |
| 正式进程名 / consumer | 真正消费进程是 `notification-worker`，consumer group 为 `cg-notification-worker` 或其 smoke 隔离变体 | worker 日志、Kafka consumer、Prometheus 指标 |
| 模板渲染 | 渲染模板来自 `ops.notification_template` 的 active 版本，变量真实展开 | `ops.notification_template` 版本回查；`ops.system_log.structured_payload.rendered` 或等价发送记录 |
| 正式渠道边界 | `mock-log` 是 `V1` 唯一实接渠道；`email / webhook` 仍是 reserved boundary | `ops.system_log.message_text='notification sent via mock-log'`；禁止以 email/webhook 成功代替 |
| 幂等去重 | 重放同一语义事件不会重复发送第二条通知 | `ops.consumer_idempotency_record`、`ops.system_log` 无重复发送记录 |
| 失败重试 | 临时失败会进入 Redis retry 队列并在成功后转为 `processed` | Redis `retry-queue / retry-payload`、`ops.system_log`、`audit.audit_event` |
| DLQ 隔离 | 重试耗尽后会同时进入 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter` | PostgreSQL + Kafka 双回查 |
| 人工补发 / replay | replay 必须要求 `step_up_ticket`，dry-run 不产生副作用，正式 replay 生成新 `event_id` 且保留 lineage | `POST /internal/notifications/dead-letters/{id}/replay`、审计记录、回放后新 outbox / send 记录 |
| 审计联查 | 能通过 `POST /internal/notifications/audit/search` 用 `order_id / case_id / aggregate_type / event_type / target_topic / event_id` 等维度联查正式链路 | API 响应 + `audit.audit_event(action_name='notification.dispatch.lookup')` |

## Matrix

| 用例ID | 场景 | 触发 | 预期结果 | 证据 |
| --- | --- | --- | --- | --- |
| `NOTIF-CASE-001` | 支付成功通知 | 处理 `payment.succeeded`，生成 buyer / seller / ops 通知 | `ops.outbox_event` 为 3 条 `notification.requested`，`target_topic=dtp.notification.dispatch`；模板变量展开正确；买卖双方与平台收到 `payment.succeeded` 或待交付通知 | `apps/platform-core/src/modules/integration/tests/notif004_payment_success_db.rs` |
| `NOTIF-CASE-002` | 交付完成通知 | 提交 `delivery.completed` 或查询结果交付完成 | 交付完成通知进入 `dtp.notification.dispatch`；buyer / seller / ops 收到对应 `delivery.completed` 或待验收通知 | `apps/platform-core/src/modules/integration/tests/notif005_delivery_completion_db.rs` |
| `NOTIF-CASE-003` | 验收通过通知 | 处理 `trade.acceptance_record / acceptance.passed` | buyer / seller / ops 收到 `acceptance.passed`；发送记录与模板版本可回查 | `apps/platform-core/src/modules/integration/tests/notif006_acceptance_resolution_db.rs` |
| `NOTIF-CASE-004` | 拒收通知 | 处理 `trade.acceptance_record / acceptance.rejected` | buyer / seller / ops 收到 `acceptance.rejected`；拒收原因变量进入模板上下文 | `apps/platform-core/src/modules/integration/tests/notif006_acceptance_resolution_db.rs` |
| `NOTIF-CASE-005` | 争议升级通知 | 打开争议 `support.dispute_case / dispute.created` | buyer / seller / ops 收到 `dispute.escalated`；审计与冻结链启动可联查 | `apps/platform-core/src/modules/integration/tests/notif007_dispute_settlement_db.rs` |
| `NOTIF-CASE-006` | 结算冻结通知 | 记录 `billing.billing_event / billing.event.recorded`，且 `event_source=settlement_dispute_hold` | buyer / seller / ops 收到 `settlement.frozen`；主状态仍以结算冻结事实为准 | `apps/platform-core/src/modules/integration/tests/notif007_dispute_settlement_db.rs` |
| `NOTIF-CASE-007` | 结算恢复通知 | 记录 `billing.billing_event / billing.event.recorded`，且 `event_source=settlement_dispute_release` | buyer / seller / ops 收到 `settlement.resumed`；不得再以 `support.dispute_case / dispute.resolved` 直接冒充恢复结算 | `apps/platform-core/src/modules/integration/tests/notif007_dispute_settlement_db.rs` |
| `NOTIF-CASE-008` | 重复事件去重 | 重放相同 `event_id / idempotency_key / recipient` 组合 | 第二次不新增 `notification sent via mock-log`；`ops.consumer_idempotency_record.result_code=duplicate` 或等价去重状态 | `apps/notification-worker/src/main.rs` 中 `notif012_notification_worker_live_smoke` |
| `NOTIF-CASE-009` | 失败重试后成功 | 构造一次 `notification.send` 临时失败，再恢复 `mock-log` | Redis retry 队列出现待重试任务；重试成功后发送完成，审计轨迹包含 `notification.dispatch.retry_scheduled -> notification.dispatch.sent` | `apps/notification-worker/src/main.rs` 中 `notif012_notification_worker_live_smoke` |
| `NOTIF-CASE-010` | 进入 DLQ | 构造重试耗尽或不可恢复失败 | `ops.dead_letter_event.failure_stage='notification.send'`；Kafka `dtp.dead-letter` 可消费到对应 envelope；`ops.alert_event` 产生 `notification_dead_letter` | `apps/notification-worker/src/main.rs` 中 `notif012_notification_worker_live_smoke` |
| `NOTIF-CASE-011` | 人工补发 / replay | 对目标 dead letter 执行 dry-run 与正式 replay | dry-run 只产出预演审计，不写发送副作用；正式 replay 生成新 `event_id / request_id / trace_id`，并保留 `replayed_from_dead_letter_id` lineage | `docs/04-runbooks/notification-worker.md` 中“人工补发 / replay”；`apps/notification-worker/src/main.rs` 中 `replay_envelope_preserves_idempotency_and_marks_lineage`、`replay_request_requires_reason_and_step_up_ticket` |
| `NOTIF-CASE-012` | 审计联查 | 使用 `POST /internal/notifications/audit/search` 按 `order_id / event_id / aggregate_type / event_type / target_topic` 查询 | `filters` 与 `records[*]` 返回正式 envelope 路由字段；`audit.audit_event.action_name='notification.dispatch.lookup'` 记录本次联查 | `docs/04-runbooks/notification-worker.md` 中“通知审计联查”；`apps/notification-worker/src/main.rs` 中 `notification_lookup_metadata_includes_canonical_route_fields` |

## Recommended Smoke Path

最小可信验收至少跑完下面两组动作，不能只做其中一半：

1. 运行 live smoke，证明正式进程、正式 topic、`mock-log` 渠道、幂等、重试、DLQ、数据库 / Redis / audit / metrics 留痕真实存在：
   ```bash
   NOTIF_WORKER_DB_SMOKE=1 \
   DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
   REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 \
   KAFKA_BROKERS=127.0.0.1:9094 \
   cargo test -p notification-worker notif012_notification_worker_live_smoke -- --nocapture
   ```
2. 再执行一次控制面手工联查，证明验收清单中的正式联查路径不是占位：
   ```bash
   APP_PORT=8097 \
   DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
   REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/2 \
   KAFKA_BROKERS=127.0.0.1:9094 \
   TOPIC_NOTIFICATION_DISPATCH=dtp.notification.dispatch \
   TOPIC_DEAD_LETTER_EVENTS=dtp.dead-letter \
   cargo run -p notification-worker
   ```
   ```bash
   curl -sS -X POST http://127.0.0.1:8097/internal/notifications/send \
     -H 'Content-Type: application/json' \
     -d '{
       "notification_code":"payment.succeeded",
       "audience_scope":"buyer",
       "template_code":"NOTIFY_PAYMENT_SUCCEEDED_V1",
       "recipient":{
         "kind":"user",
         "address":"buyer@example.test",
         "display_name":"Buyer"
       },
       "variables":{
         "subject":"Payment success notification",
         "headline":"Buyer escrow completed and order is pending delivery",
         "order_id":"11111111-1111-1111-1111-111111111111",
         "order_no":"ORD-NOTIF-014-SMOKE",
         "product_title":"Example SKU",
         "seller_org_name":"Example Seller Org",
         "order_amount":"128.00",
         "currency_code":"CNY",
         "payment_status":"buyer_locked",
         "delivery_status":"pending_delivery",
         "action_label":"View order",
         "action_href":"/trade/orders/11111111-1111-1111-1111-111111111111"
       },
       "source_event":{
         "aggregate_type":"billing.billing_event",
         "aggregate_id":"11111111-1111-1111-1111-111111111111",
         "event_type":"billing.event.recorded",
         "target_topic":"dtp.outbox.domain-events"
       },
       "subject_refs":[
         {"ref_type":"order","ref_id":"11111111-1111-1111-1111-111111111111"}
       ],
       "request_id":"req-notif014-smoke",
       "trace_id":"trace-notif014-smoke"
     }'
   ```
   ```bash
   curl -sS -X POST http://127.0.0.1:8097/internal/notifications/audit/search \
     -H 'Content-Type: application/json' \
     -d '{
       "aggregate_type":"notification.dispatch_request",
       "event_type":"notification.requested",
       "target_topic":"dtp.notification.dispatch",
       "notification_code":"payment.succeeded",
       "limit":5,
       "reason":"notif014 manual lookup",
       "step_up_ticket":"step-up-local-1"
     }'
   ```
3. 若要覆盖人工补发，再按 runbook 对同一 dead letter 执行：
   - dry-run：`POST /internal/notifications/dead-letters/{dead_letter_event_id}/replay` with `{"dry_run":true,...}`
   - 正式 replay：`POST /internal/notifications/dead-letters/{dead_letter_event_id}/replay` with `{"dry_run":false,...}`

## Manual Backcheck Template

执行任何通知用例后，至少回查以下对象：

```sql
select event_type, aggregate_type, target_topic, payload->'payload'->>'notification_code' as notification_code
  from ops.outbox_event
 where target_topic = 'dtp.notification.dispatch'
 order by created_at desc
 limit 10;

select message_text, structured_payload
  from ops.system_log
 where object_type = 'notification_dispatch'
 order by created_at desc
 limit 20;

select idempotency_key, result_code, updated_at
  from ops.consumer_idempotency_record
 where consumer_group like 'cg-notification-worker%'
 order by updated_at desc
 limit 20;

select event_id, failure_stage, target_topic, retry_count
  from ops.dead_letter_event
 where failure_stage = 'notification.send'
 order by dead_lettered_at desc
 limit 20;

select action_name, metadata
  from audit.audit_event
 where domain_name = 'notification'
 order by created_at desc
 limit 20;
```

Redis 与 Kafka 也必须抽查：

```bash
redis-cli -u redis://:datab_redis_pass@127.0.0.1:6379/2 \
  LRANGE datab:v1:notification:retry-queue 0 -1

docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh \
  --bootstrap-server localhost:9092 \
  --topic dtp.dead-letter \
  --from-beginning \
  --timeout-ms 5000
```

## Traceability

- `docs/开发准备/测试用例矩阵正式版.md`
- `docs/开发准备/事件模型与Topic清单正式版.md`
- `docs/数据库设计/接口协议/一致性与事件接口协议正式版.md`
- `docs/04-runbooks/notification-worker.md`
- `docs/开发任务/问题修复任务/A10-NOTIF-通知链路与命名边界缺口.md`
- `docs/开发任务/问题修复任务/A11-测试与Smoke口径误报风险.md`
