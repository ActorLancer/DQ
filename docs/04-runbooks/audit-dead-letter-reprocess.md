# AUD-010 Dead Letter Reprocess 运行手册

适用范围：

- `AUD-010`
- `POST /api/v1/ops/dead-letters/{id}/reprocess`

当前批次冻结结论：

- 接口只承接 `SEARCHREC` consumer 失败事件：
  - `dtp.search.sync -> search-indexer`
  - `dtp.recommend.behavior -> recommendation-aggregator`
- 高风险动作必须要求：
  - 权限
  - `step-up`
  - 正式审计
- `V1` 当前仅支持 `dry_run=true` 预演，不在本批直接执行 worker 侧正式 replay

## 前置条件

- 已执行：`set -a; source infra/docker/.env.local; set +a`
- 基础设施可用：`PostgreSQL`、`Kafka`、`Redis`、`Keycloak / IAM`
- `platform-core` 已启动，例如：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
KAFKA_BROKERS=127.0.0.1:9094 \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

## 一次性 live smoke

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture
```

该 smoke 会真实验证：

- `search-indexer` dead letter dry-run
- `recommendation-aggregator` dead letter dry-run
- `step-up` challenge 绑定
- `audit.audit_event + audit.access_audit + ops.system_log`

## 手工 API 验证

### 1. 准备一条 SEARCHREC dead letter 和 step-up challenge

最小需要：

- `ops.dead_letter_event`
- `ops.consumer_idempotency_record`
- `iam.step_up_challenge(target_action='ops.dead_letter.reprocess', target_ref_type='dead_letter_event')`

可复用 `AUD-008` / `SEARCHREC` 已存在样本，或按 live smoke 的插数方式单独准备。

### 2. 发起 dry-run

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/dead-letters/<dead_letter_event_id>/reprocess" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <user_id>' \
  -H 'x-request-id: req-aud010-manual' \
  -H 'x-trace-id: trace-aud010-manual' \
  -H 'x-step-up-challenge-id: <step_up_challenge_id>' \
  -d '{"reason":"preview searchrec dead letter recovery","dry_run":true,"metadata":{"source":"manual-aud010"}}'
```

预期：

- 返回 `200`
- `data.status='dry_run_ready'`
- `data.step_up_bound=true`
- `data.consumer_names` 与 `data.consumer_groups` 对应 SEARCHREC worker
- `data.replay_target_topic` 为原始 SEARCHREC topic

### 3. 回查 dead letter 没有被误修改

```sql
SELECT
  dead_letter_event_id::text,
  reprocess_status,
  reprocessed_at,
  target_topic,
  failure_stage
FROM ops.dead_letter_event
WHERE dead_letter_event_id = '<dead_letter_event_id>'::uuid;
```

预期：

- `reprocess_status='not_reprocessed'`
- `reprocessed_at IS NULL`
- `failure_stage='consumer_handler'`

### 4. 回查正式审计与系统日志

```sql
SELECT action_name, result_code, request_id, trace_id
FROM audit.audit_event
WHERE request_id = 'req-aud010-manual'
ORDER BY event_time DESC, audit_id DESC;

SELECT access_mode, target_type, request_id
FROM audit.access_audit
WHERE request_id = 'req-aud010-manual'
ORDER BY created_at DESC, access_audit_id DESC;

SELECT message_text, request_id, trace_id
FROM ops.system_log
WHERE request_id = 'req-aud010-manual'
ORDER BY created_at DESC, system_log_id DESC;
```

预期：

- `audit.audit_event.action_name='ops.dead_letter.reprocess.dry_run'`
- `audit.access_audit.access_mode='reprocess'`
- `ops.system_log.message_text='ops dead letter reprocess prepared: POST /api/v1/ops/dead-letters/{id}/reprocess'`

## 约束与边界

1. 不支持：
   - `dry_run=false`
   - 非 SEARCHREC topic 的 dead letter
   - `reprocess_status != not_reprocessed` 的记录
2. 当前批次不直接 republish Kafka 消息；真正 worker 侧 replay 与 offset 策略收口由后续 SEARCHREC 可靠性任务承接。
3. `dtp.dead-letter` 和 `ops.dead_letter_event` 仍是失败隔离双层权威，当前接口只做控制面 dry-run 预演，不替代后续正式 replayer。

## 故障排查

### 1. 返回 `400 missing step-up`

- 确认是否提供 `x-step-up-token` 或 `x-step-up-challenge-id`
- 若使用 challenge，确认：
  - `challenge_status='verified'`
  - `target_action='ops.dead_letter.reprocess'`
  - `target_ref_type='dead_letter_event'`
  - `target_ref_id` 与路径中的 dead letter 一致

### 2. 返回 `409 AUDIT_DEAD_LETTER_REPROCESS_NOT_SUPPORTED`

- 该 dead letter 不是 SEARCHREC consumer failure
- 或 `target_topic` 不是 `dtp.search.sync / dtp.recommend.behavior`
- 或 `failure_stage` 不是 `consumer_handler`

### 3. 返回 `409 AUDIT_DEAD_LETTER_REPROCESS_STATE_CONFLICT`

- `ops.dead_letter_event.reprocess_status` 已不再是 `not_reprocessed`
- 说明这条记录已经进入后续处理或被别的控制面接管
