# Audit Replay Runbook

当前 runbook 承接 `AUD-005` 的正式 replay dry-run 控制面：

- `POST /api/v1/audit/replay-jobs`
- `GET /api/v1/audit/replay-jobs/{id}`

`V1` 明确只允许 `dry_run=true`。任何 `dry_run=false` 请求都必须返回 `AUDIT_REPLAY_DRY_RUN_ONLY`，不得产生副作用型 replay。

## 适用范围

- 订单审计时间线复盘
- 争议案件审计联查复盘
- 证据包 / 审计对象投影复盘
- 调查审计漂移、证据缺失或时间线异常

## 权限与 Step-Up

- 创建 replay job：`audit.replay.execute`
- 读取 replay job：`audit.replay.read`
- 创建动作必须提供：
  - `x-user-id`
  - `x-request-id`
  - `x-step-up-token` 或 `x-step-up-challenge-id`
- 若使用 `x-step-up-challenge-id`：
  - challenge 必须属于当前 `x-user-id`
  - `challenge_status=verified`
  - 未过期
  - `target_action='audit.replay.execute'`
  - 若 challenge 绑定了 `target_ref_type / target_ref_id`，必须与当前 replay 目标一致

## 请求模板

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/replay-jobs \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-runbook-create' \
  -H 'x-trace-id: trace-aud005-runbook-create' \
  -H 'x-step-up-challenge-id: <replay_challenge_id>' \
  -d '{
    "replay_type": "state_replay",
    "ref_type": "order",
    "ref_id": "<order_id>",
    "reason": "investigate audit drift",
    "dry_run": true,
    "options": {
      "trigger": "runbook",
      "ticket": "AUD-OPS-001"
    }
  }'
```

支持的 `replay_type`：

- `forensic_replay`
- `state_replay`
- `reconciliation_replay`
- `compensation_replay`

`ref_type` 在当前实现中优先使用：

- `order`
- `case` / `dispute_case`
- `evidence_package`

如使用其他 `ref_type`，必须保证对应对象已在 `audit.audit_event` 中存在正式 `ref_type + ref_id` 记录。

## 返回结果解读

成功创建后返回：

- `replay_job.replay_status=completed`
- `replay_job.dry_run=true`
- `results` 固定包含 4 个步骤：
  - `target_snapshot`
  - `audit_timeline`
  - `evidence_projection`
  - `execution_policy`

`execution_policy.result_code` 在 `V1` 必须为 `AUDIT_REPLAY_DRY_RUN_ONLY`。

## 读取 replay job

```bash
curl -sS http://127.0.0.1:18080/api/v1/audit/replay-jobs/<replay_job_id> \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-runbook-get' \
  -H 'x-trace-id: trace-aud005-runbook-get'
```

读取动作也必须落：

- `audit.access_audit(access_mode='replay')`
- `ops.system_log.message_text='audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}'`

## 数据回查

1. replay job：

```sql
SELECT replay_job_id::text,
       replay_type,
       ref_type,
       ref_id::text,
       dry_run,
       status,
       requested_by::text,
       request_reason,
       step_up_challenge_id::text,
       options_json ->> 'report_storage_uri'
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;
```

2. replay results：

```sql
SELECT step_name,
       result_code,
       expected_digest,
       actual_digest
FROM audit.replay_result
WHERE replay_job_id = '<replay_job_id>'::uuid
ORDER BY created_at, replay_result_id;
```

3. 正式审计与访问留痕：

```sql
SELECT action_name, result_code, metadata
FROM audit.audit_event
WHERE ref_type = 'replay_job'
  AND ref_id = '<replay_job_id>'::uuid
ORDER BY event_time, audit_id;

SELECT access_mode, request_id, metadata
FROM audit.access_audit
WHERE target_type = 'replay_job'
  AND target_id = '<replay_job_id>'::uuid
ORDER BY created_at, access_audit_id;

SELECT request_id, message_text
FROM ops.system_log
WHERE request_id IN ('req-aud005-runbook-create', 'req-aud005-runbook-get')
ORDER BY created_at, system_log_id;
```

4. MinIO replay report：

从 `audit.replay_job.options_json ->> 'report_storage_uri'` 取得对象地址后，确认：

- 对象实际存在
- 内容可解析为 JSON
- `recommendation=dry_run_completed`
- `results[*].step_name` 与数据库中的 `audit.replay_result` 一致

## 观测联查

- `Loki`：检索 `request_id=req-aud005-runbook-create` 或 `replay_job_id=<id>`
- `Tempo`：使用 `x-trace-id` 关联 `platform-core` 请求链
- `Prometheus / Grafana`：确认 `platform-core` 请求被正常采集；如回放用于异常调查，应同时关联同一 `request_id / trace_id / ref_id`
- `Alertmanager`：当前 task 不新增专属 replay 告警规则；若 replay 用于事故调查，应记录对应告警或事件编号到 `options.ticket`

## 常见故障

### 1. 返回 `403 forbidden`

检查：

- `x-role` 是否具备 `audit.replay.execute` / `audit.replay.read`
- 是否误用租户侧角色而非平台审计角色

### 2. 返回 `400 x-user-id is required for high-risk audit action`

检查：

- 创建 replay job 是否缺少 `x-user-id`

### 3. 返回 `400 missing step-up`

检查：

- 是否缺少 `x-step-up-token` 和 `x-step-up-challenge-id`
- challenge 是否已过期或未 `verified`

### 4. 返回 `409 AUDIT_REPLAY_DRY_RUN_ONLY`

说明：

- 这是 `V1` 的正式约束，不是临时异常
- 当前阶段不得试图通过改 header、改 DB 或脚本旁路做正式副作用 replay

### 5. replay job 已写库但 MinIO 对象缺失

当前实现不允许这种状态长期存在。若发现该现象：

1. 先检查 MinIO 连接与 bucket：`evidence-packages`
2. 回查同一 `request_id` 的 `ops.system_log`
3. 若确认是运行时故障，按后续 `AUD` 一致性修复 task 的正式流程处理，不得手工补写伪对象

## 清理规则

- 业务测试数据可清理
- `audit.replay_job`、`audit.replay_result`、`audit.audit_event`、`audit.access_audit`、`ops.system_log`、MinIO replay report 与 evidence snapshot 按 append-only 保留
