# Audit Anchor Batches Runbook

本手册对应 `AUD-007`，覆盖：

- `GET /api/v1/audit/anchor-batches`
- `POST /api/v1/audit/anchor-batches/{id}/retry`

目标：

- 查看 `audit.anchor_batch + chain.chain_anchor` 的正式权威视图
- 对 `status=failed` 的锚定批次执行高风险 retry
- 回查 `audit.audit_event / audit.access_audit / ops.system_log / ops.outbox_event`
- 验证 canonical route `audit.anchor_requested -> dtp.audit.anchor`

## 权限与 step-up

- 查看：`audit.anchor.read`
- 重试：`audit.anchor.manage`
- `POST /retry` 必须同时满足：
  - `x-user-id`
  - `x-request-id`
  - `x-step-up-token` 或 `x-step-up-challenge-id`
- 若使用 challenge，必须满足：
  - `target_action='audit.anchor.manage'`
  - `target_ref_type='anchor_batch'`
  - `target_ref_id=<anchor_batch_id>`
  - `challenge_status='verified'`

## 手工验证

1. 启动服务：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

2. 查询 failed 批次：

```bash
curl -sS 'http://127.0.0.1:18080/api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-list' \
  -H 'x-trace-id: trace-aud007-manual-list'
```

预期：

- 返回分页结果
- `items[].anchor_status='failed'`
- 可看到 `anchor_batch_id / batch_scope / record_count / batch_root / chain_id / tx_hash / anchored_at`

3. 为 retry 准备 challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.anchor.manage',
  'anchor_batch',
  '<anchor_batch_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud007-manual-retry')
)
RETURNING step_up_challenge_id::text;
```

4. 发起 retry：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/anchor-batches/<anchor_batch_id>/retry \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-retry' \
  -H 'x-trace-id: trace-aud007-manual-retry' \
  -H 'x-step-up-challenge-id: <retry_challenge_id>' \
  -d '{
    "reason": "retry failed batch after gateway timeout",
    "metadata": {
      "ticket_id": "AUD-OPS-007"
    }
  }'
```

预期：

- HTTP `200`
- `data.anchor_batch.anchor_status='retry_requested'`
- `data.step_up_bound=true`

## SQL 回查

1. 回查 anchor batch 主记录：

```sql
SELECT
  status,
  metadata ->> 'previous_status' AS previous_status,
  metadata -> 'retry_request' ->> 'reason' AS retry_reason,
  metadata -> 'retry_request' ->> 'request_id' AS retry_request_id,
  metadata -> 'retry_request' ->> 'trace_id' AS retry_trace_id
FROM audit.anchor_batch
WHERE anchor_batch_id = '<anchor_batch_id>'::uuid;
```

预期：

- `status='retry_requested'`
- `previous_status='failed'`
- `retry_reason='retry failed batch after gateway timeout'`

2. 回查 canonical outbox：

```sql
SELECT
  target_topic,
  event_type,
  aggregate_type,
  aggregate_id::text,
  payload ->> 'anchor_status' AS anchor_status,
  payload ->> 'previous_anchor_status' AS previous_anchor_status,
  payload ->> 'retry_reason' AS retry_reason,
  status
FROM ops.outbox_event
WHERE request_id = 'req-aud007-manual-retry'
ORDER BY created_at DESC, outbox_event_id DESC
LIMIT 1;
```

预期：

- `target_topic='dtp.audit.anchor'`
- `event_type='audit.anchor_requested'`
- `aggregate_type='audit.anchor_batch'`
- `anchor_status='retry_requested'`
- `previous_anchor_status='failed'`
- `status='pending'`

3. 回查审计与系统日志：

```sql
SELECT COUNT(*)::bigint
FROM audit.audit_event
WHERE request_id = 'req-aud007-manual-retry'
  AND action_name = 'audit.anchor.retry'
  AND ref_type = 'anchor_batch';

SELECT COUNT(*)::bigint
FROM audit.access_audit
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry');

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry')
ORDER BY created_at;
```

预期：

- `audit.audit_event` 至少 1 条
- `audit.access_audit` 覆盖 list + retry 两次访问
- `ops.system_log` 至少包含：
  - `audit lookup executed: GET /api/v1/audit/anchor-batches`
  - `audit anchor batch retry requested: POST /api/v1/audit/anchor-batches/{id}/retry`

## 运行态边界

- 本批只负责 control-plane：
  - `platform-core` 读取 anchor batch
  - `platform-core` 受理 retry 并写 canonical outbox
- 下游 `fabric-adapter`、callback、reconcile 留待 `AUD-008+`
- 不允许把 `dtp.outbox.domain-events` 当 `audit anchor` 正式入口

## 清理约束

- 业务测试对象可清理：临时 `trade.order_main / catalog.* / core.organization`
- 审计对象默认不清理：`audit.anchor_batch`、`audit.audit_event`、`audit.access_audit`、`ops.system_log`
- 若需清理 `ops.outbox_event` 临时行，只允许按唯一 `request_id` 或唯一 `aggregate_id` 精确删除，不能做宽泛清库
