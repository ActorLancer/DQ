# Outbox Publisher（AUD-009）

## 目标

- 正式进程名：`outbox-publisher`
- 正式链路：`ops.outbox_event -> workers/outbox-publisher -> Kafka`
- 双层失败隔离：
  - PostgreSQL：`ops.dead_letter_event`
  - Kafka：`dtp.dead-letter`
- Billing bridge 边界：
  - `POST /api/v1/billing/{order_id}/bridge-events/process` 只处理**已由 publisher 发布过**的 `billing.trigger.bridge`
  - 准入条件必须同时命中最新 `ops.outbox_publish_attempt.result_code='published'`
  - 不再把 `ops.outbox_event.status='pending'` 当 Billing 私有工作队列

## 启动方式

宿主机正式入口：

```bash
set -a; source infra/docker/.env.local; set +a
APP_PORT=8098 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p outbox-publisher
```

应用层 compose 参考入口：

```bash
COMPOSE_PROFILES=core,apps docker compose \
  --env-file infra/docker/.env.local \
  -f infra/docker/docker-compose.local.yml \
  -f infra/docker/docker-compose.apps.local.example.yml \
  up -d outbox-publisher
```

健康与指标：

```bash
curl -fsS http://127.0.0.1:8098/health/live
curl -fsS http://127.0.0.1:8098/health/ready
curl -fsS http://127.0.0.1:8098/metrics
```

## 运行时要点

- 只轮询 `ops.outbox_event(status='pending', available_at<=now())`
- 每次发布都写 `ops.outbox_publish_attempt`
- 成功后把 outbox 行更新为 `published`
- 失败时按指数退避回写 `retry_count / available_at / last_error_*`
- 重试耗尽后：
  - `ops.outbox_event.status=dead_lettered`
  - 插入或更新 `ops.dead_letter_event(failure_stage='outbox.publish')`
  - 发送 Kafka `dtp.dead-letter` 隔离消息

## 真实验证

建议先启动基础设施，再单独跑 worker live smoke：

```bash
AUD_DB_SMOKE=1 \
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture
```

该 smoke 覆盖两条正式路径：

1. 发布成功：
   - `ops.outbox_event.status=published`
   - Kafka `dtp.outbox.domain-events` 收到统一 envelope
   - `ops.outbox_publish_attempt.result_code='published'`
2. 发布失败：
   - 构造 `target_topic=dtp.missing.topic`
   - `ops.dead_letter_event.failure_stage='outbox.publish'`
   - Kafka `dtp.dead-letter` 收到隔离消息
   - `ops.outbox_publish_attempt.result_code='dead_lettered'`

## 手工联调

1. 启动 `platform-core` 与 `outbox-publisher`
2. 用现有业务 API 产生 canonical outbox；例如执行一个会写 `dtp.notification.dispatch` 或 `dtp.search.sync` 的业务动作
3. 回查 PostgreSQL：

```sql
select
  oe.outbox_event_id,
  oe.event_type,
  oe.status,
  oe.target_topic,
  oe.request_id,
  oe.trace_id,
  pa.attempt_no,
  pa.result_code,
  pa.error_code,
  pa.attempted_at
from ops.outbox_event oe
left join lateral (
  select attempt_no, result_code, error_code, attempted_at
    from ops.outbox_publish_attempt
   where outbox_event_id = oe.outbox_event_id
   order by attempt_no desc, attempted_at desc, outbox_publish_attempt_id desc
   limit 1
) pa on true
where oe.request_id = '<request-id>'
order by oe.created_at desc, oe.outbox_event_id desc;
```

4. 如需验证死信：

```sql
select
  dead_letter_event_id,
  outbox_event_id,
  failure_stage,
  failed_reason,
  target_topic,
  reprocess_status,
  created_at
from ops.dead_letter_event
where failure_stage = 'outbox.publish'
order by created_at desc, dead_letter_event_id desc;
```

5. Kafka 回查：

```bash
docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh \
  --bootstrap-server localhost:9092 \
  --topic dtp.outbox.domain-events \
  --from-beginning \
  --max-messages 1

docker exec datab-kafka /opt/kafka/bin/kafka-console-consumer.sh \
  --bootstrap-server localhost:9092 \
  --topic dtp.dead-letter \
  --from-beginning \
  --max-messages 1
```

## Billing Bridge 变更

- `billing.trigger.bridge` 仍由交付侧写 canonical outbox
- 但 Billing 手工桥接现在只处理：
  - `status='published'`
  - `published_at IS NOT NULL`
  - 最新 `ops.outbox_publish_attempt.result_code='published'`
  - `target_topic='dtp.outbox.domain-events'`
- 物化后的 `billing.billing_event.metadata` 必须携带 `bridge_outbox_event_id / bridge_publish_attempt_id`
- 这样保留了显式人工桥接 / 回放入口，同时移除了“Billing 把 pending outbox 当主工作队列”的默认路径，也把 BillingEvent 回溯绑定到正式 publish authority

## 观测

- Prometheus：

```bash
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=up{job="outbox-publisher"}'
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=outbox_publisher_publish_attempts_total'
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=outbox_publisher_pending_events'
```

- Alertmanager / Grafana：
  - `ServiceDown{job="outbox-publisher"}`
  - `OutboxPublisherPendingBacklog`
- Loki / 日志：
  - `service_name=outbox-publisher` 的 `ops.system_log`
  - 终端 stdout 中的 `outbox-publisher processed event`

## 故障处理

1. `ops.outbox_event` 长期停留 `pending`
   - 先看 `http://127.0.0.1:8098/health/ready`
   - 再看 `outbox_publisher_pending_events`
   - 再查最新 `ops.outbox_publish_attempt / last_error_*`

2. `ops.dead_letter_event` 已写，但 Kafka `dtp.dead-letter` 没消息
   - 说明主 broker 可达但 DLQ publish 失败或 timeout
   - 优先查 `ops.outbox_publish_attempt.metadata.dead_letter_publish_error`

3. Billing bridge API 处理不到刚写入的 `billing.trigger.bridge`
   - 先确认该 outbox 是否已被 publisher 标记为 `published`
   - 再查最新 `ops.outbox_publish_attempt` 是否存在且 `result_code='published'`
   - 若仍是 `pending`，问题在 publisher，不在 Billing bridge API
