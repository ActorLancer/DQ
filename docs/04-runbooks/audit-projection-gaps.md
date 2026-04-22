# AUD-021 投影缺口查询与关闭

正式接口：

- `GET /api/v1/ops/projection-gaps`
- `POST /api/v1/ops/projection-gaps/{id}/resolve`

正式目标：

- 以 `ops.chain_projection_gap` 作为唯一正式持久化对象返回链投影缺口
- 查询动作必须真实写入 `audit.access_audit` 与 `ops.system_log`
- 关闭动作必须要求 `ops.projection_gap.manage + step-up + audit + system log`
- 关闭动作默认 `dry_run=true`，并支持 `expected_state_digest` 乐观校验
- 关闭动作只更新正式 `ops.chain_projection_gap`，不得引入 `reconcile_job` 表，也不得直接发布 `dtp.consistency.reconcile`

边界：

- 本接口是 `platform-core` 的高风险 control-plane，不反向定义业务主状态机
- `resolve` 只在 `gap_status != 'resolved'` 时允许执行
- `dry_run=true` 只输出预演结果并留痕，不改写 `ops.chain_projection_gap`
- `consistency/reconcile` 仍由 `AUD-012` 的控制面承接；`resolve` 不是 reconcile 执行 worker

## 权限

- `GET /api/v1/ops/projection-gaps`
  - `ops.projection_gap.read`
  - 当前已落地角色：`platform_admin`、`platform_audit_security`
- `POST /api/v1/ops/projection-gaps/{id}/resolve`
  - `ops.projection_gap.manage`
  - 当前已落地角色：`platform_admin`、`platform_audit_security`
  - 必须带 `x-step-up-token` 或 verified `x-step-up-challenge-id`

## 宿主机启动

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

## 快速真实验证

优先执行本批 live smoke：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_projection_gap_resolve_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入最小订单图与一条 `ops.chain_projection_gap(gap_status='open')`
- 调用 `GET /api/v1/ops/projection-gaps`
- 调用 `POST /api/v1/ops/projection-gaps/{id}/resolve` 的 `dry_run` 与真实关闭
- 回查 `ops.chain_projection_gap / audit.audit_event / audit.access_audit / ops.system_log`
- 断言不会写出新的 `dtp.consistency.reconcile` outbox 事件

## 手工查询

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/projection-gaps?aggregate_type=order&aggregate_id=<order_id>&order_id=<order_id>&chain_id=fabric-local&gap_type=missing_callback&gap_status=open&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-list' \
  -H 'x-trace-id: trace-aud021'
```

## 手工 dry-run 预演

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-dry-run' \
  -H 'x-trace-id: trace-aud021' \
  -H 'x-step-up-challenge-id: <resolve_step_up_id>' \
  -d '{
    "dry_run": true,
    "resolution_mode": "callback_confirmed",
    "reason": "preview close projection gap after callback verification"
  }'
```

预期重点：

- `status=dry_run_ready`
- `projection_gap.gap_status=open`
- `state_digest` 返回当前对象状态摘要

## 手工真实关闭

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/projection-gaps/<chain_projection_gap_id>/resolve" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud021-execute' \
  -H 'x-trace-id: trace-aud021' \
  -H 'x-step-up-challenge-id: <resolve_step_up_id>' \
  -d '{
    "dry_run": false,
    "resolution_mode": "callback_confirmed",
    "reason": "confirmed callback backfilled into projection gap",
    "expected_state_digest": "<state_digest_from_dry_run>"
  }'
```

预期重点：

- `status=resolution_recorded`
- `projection_gap.gap_status=resolved`
- `projection_gap.resolved_at` 非空

## SQL 回查

```sql
SELECT chain_projection_gap_id::text,
       aggregate_type,
       aggregate_id::text,
       order_id::text,
       chain_id,
       gap_type,
       gap_status,
       resolved_at,
       resolution_summary -> 'manual_resolution' ->> 'reason' AS resolve_reason,
       resolution_summary -> 'manual_resolution' ->> 'resolution_mode' AS resolution_mode
FROM ops.chain_projection_gap
WHERE chain_projection_gap_id = '<chain_projection_gap_id>'::uuid;
```

```sql
SELECT action_name, result_code, before_state_digest, after_state_digest, metadata
FROM audit.audit_event
WHERE request_id IN ('req-aud021-dry-run', 'req-aud021-execute')
  AND action_name = 'ops.projection_gap.resolve'
ORDER BY occurred_at;
```

```sql
SELECT COUNT(*)::bigint
FROM ops.outbox_event
WHERE request_id IN ('req-aud021-dry-run', 'req-aud021-execute')
  AND target_topic = 'dtp.consistency.reconcile';
```

## 排障

1. 返回 `400`

- 先检查 `reason`
- 再检查 `x-step-up-token` 或 `x-step-up-challenge-id`
- 如果是 verified challenge，确认其 `target_action='ops.projection_gap.manage'`、`target_ref_type='projection_gap'`

2. 返回 `403`

- 先确认角色属于 `platform_admin / platform_audit_security`
- 再确认 step-up challenge 属于当前 `user_id` 且状态为 `verified`

3. 返回 `404`

- 回查 `ops.chain_projection_gap` 是否存在

4. 返回 `409`

- 当前只允许 `gap_status != 'resolved'` 的缺口执行关闭
- 如果带了 `expected_state_digest`，确认它来自最近一次 `dry_run` 或最新查询结果

5. 发现写出了 `dtp.consistency.reconcile`

- 这属于缺陷
- `AUD-021` 的边界是“只关闭正式 projection gap 对象，不直接派发 reconcile 执行事件”
