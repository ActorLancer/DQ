# AUD-008 Outbox / Dead Letter 查询运行手册

适用范围：

- `AUD-008`
- `GET /api/v1/ops/outbox`
- `GET /api/v1/ops/dead-letters`

当前批次的正式对象边界：

- 对外 HTTP 查询接口：`ops.outbox_event`、`ops.dead_letter_event`
- 当前已落地但暂未暴露公共 HTTP 查询接口的仓储对象：`ops.consumer_idempotency_record`、`ops.external_fact_receipt`、`ops.chain_projection_gap`
- `consistency/reconcile` 在 `V1` 中仍是控制面动作，由 `AUD-012` 承接；`AUD-008` 不引入独立 `reconcile_job` 表

## 前置条件

- 已执行：`set -a; source infra/docker/.env.local; set +a`
- 基础设施可用：`PostgreSQL`、`Kafka`、`Redis`、`Keycloak / IAM`、`MinIO`、观测栈
- `platform-core` 已启动，例如：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
KAFKA_BROKERS=127.0.0.1:9094 \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

说明：

- 宿主机启动 `platform-core` 时，Kafka 必须走宿主机边界 `127.0.0.1:9094`
- `kafka:9092` 仅用于 compose 网络内部调用

## 一次性 live smoke

`AUD-008` 的集成 smoke 会自动插入并清理一组 `search-indexer` 失败隔离样本，同时保留 append-only 审计与系统日志：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture
```

该 smoke 会验证：

- `GET /api/v1/ops/outbox`
- `GET /api/v1/ops/dead-letters`
- `ops.consumer_idempotency_record` 联查
- `ops.external_fact_receipt` 仓储查询
- `ops.chain_projection_gap` 仓储查询
- `audit.access_audit + ops.system_log` 侧写留痕

## 手工 API 验证

### 1. 查询 canonical outbox

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/outbox?target_topic=dtp.search.sync&event_type=search.product.changed&page=1&page_size=20" \
  -H "x-role: platform_audit_security" \
  -H "x-request-id: req-aud008-manual-outbox" \
  -H "x-trace-id: trace-aud008-manual"
```

预期：

- 返回 `200`
- 可看到 `target_topic=dtp.search.sync`
- 可看到最新 `outbox_publish_attempt`
- `authority_scope / source_of_truth / proof_commit_policy` 与 canonical outbox 口径一致

### 2. 查询 dead letter

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/dead-letters?failure_stage=consumer_handler&page=1&page_size=20" \
  -H "x-role: platform_audit_security" \
  -H "x-request-id: req-aud008-manual-dead-letter" \
  -H "x-trace-id: trace-aud008-manual"
```

预期：

- 返回 `200`
- 可看到 `ops.dead_letter_event`
- 可联查 `consumer_idempotency_records`
- `consumer_name` 可用于区分 `search-indexer` / `recommendation-aggregator`

## 数据库回查

### 1. canonical outbox 与 publish attempt

```sql
SELECT
  oe.outbox_event_id::text,
  oe.aggregate_type,
  oe.event_type,
  oe.status,
  oe.target_topic,
  oe.request_id,
  oe.trace_id,
  opa.attempt_no,
  opa.result_code,
  opa.error_code
FROM ops.outbox_event oe
LEFT JOIN LATERAL (
  SELECT attempt_no, result_code, error_code
  FROM ops.outbox_publish_attempt
  WHERE outbox_event_id = oe.outbox_event_id
  ORDER BY attempt_no DESC, attempted_at DESC, outbox_publish_attempt_id DESC
  LIMIT 1
) opa ON true
WHERE oe.target_topic = 'dtp.search.sync'
ORDER BY oe.created_at DESC, oe.outbox_event_id DESC
LIMIT 20;
```

### 2. dead letter 与 consumer 幂等

```sql
SELECT
  dl.dead_letter_event_id::text,
  dl.outbox_event_id::text,
  dl.failure_stage,
  dl.reprocess_status,
  cir.consumer_name,
  cir.result_code,
  cir.trace_id
FROM ops.dead_letter_event dl
LEFT JOIN ops.consumer_idempotency_record cir
  ON cir.event_id = dl.outbox_event_id
WHERE dl.target_topic = 'dtp.search.sync'
ORDER BY dl.created_at DESC, dl.dead_letter_event_id DESC
LIMIT 20;
```

预期：

- `dead_letter_event -> consumer_idempotency_record` 能通过 `outbox_event_id / event_id` 串起来
- `search-indexer` 失败路径可见 `result_code='dead_lettered'` 或等价失败结果

### 3. 当前批次未暴露公共接口的查询对象

`AUD-008` 同步实现了以下仓储查询，用于后续 `AUD-011 / AUD-012 / AUD-021` 承接：

```sql
SELECT external_fact_receipt_id::text, fact_type, provider_type, receipt_status
FROM ops.external_fact_receipt
ORDER BY received_at DESC, external_fact_receipt_id DESC
LIMIT 20;

SELECT chain_projection_gap_id::text, aggregate_type, chain_id, gap_type, gap_status
FROM ops.chain_projection_gap
ORDER BY first_detected_at DESC, chain_projection_gap_id DESC
LIMIT 20;
```

注意：

- `ops.external_fact_receipt` 是正式对象名，不再使用残留文案 `external_receipt`
- `ops.chain_projection_gap` 是当前持久化查询对象；`reconcile` 动作在 `AUD-012`

## 审计与日志回查

```sql
SELECT target_type, access_mode, request_id, trace_id
FROM audit.access_audit
WHERE request_id IN ('req-aud008-manual-outbox', 'req-aud008-manual-dead-letter')
ORDER BY created_at, access_audit_id;

SELECT message_text, request_id, trace_id
FROM ops.system_log
WHERE request_id IN ('req-aud008-manual-outbox', 'req-aud008-manual-dead-letter')
ORDER BY created_at, system_log_id;
```

预期：

- `audit.access_audit.access_mode='masked'`
- `target_type` 分别为 `ops_outbox_query`、`dead_letter_query`
- `ops.system_log.message_text` 包含：
  - `ops lookup executed: GET /api/v1/ops/outbox`
  - `ops lookup executed: GET /api/v1/ops/dead-letters`

## 故障排查

1. `GET /api/v1/ops/outbox` 为空

- 先回查 `ops.event_route_policy` 是否存在目标 `(aggregate_type, event_type)` 的激活路由
- 再回查 `ops.outbox_event.status / target_topic / request_id`
- 不要把 `dtp.outbox.domain-events` 当成 SEARCHREC 或 AUD consumer 的正式消费入口

2. `GET /api/v1/ops/dead-letters` 有行，但没有 `consumer_idempotency_records`

- 先确认 consumer 是否已按 `event_id` 写入 `ops.consumer_idempotency_record`
- SEARCHREC 正式闭环会在 `AUD-010 / SEARCHREC-020` 继续补齐 dry-run reprocess 与 worker 失败路径

3. 需要一致性修复

- 当前不要新造 `reconcile_job` 表
- `AUD-012` 才是 `POST /api/v1/ops/consistency/reconcile` 的正式动作入口
- `AUD-021` 继续承接 `projection-gaps` 的查询 / 关闭接口
