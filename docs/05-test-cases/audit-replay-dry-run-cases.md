# TEST-014 Audit Replay Dry-Run

`TEST-014` 的正式目标是证明审计回放不是一个只会返回 `200` 的控制面，而是能按订单重建关键状态、输出差异报告，并把 replay job / replay result / MinIO report / 审计留痕全部写成可回查证据。

## 正式入口

- 本地 / CI checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-audit-replay-dry-run.sh`
- checker 会先执行 route guard：
  - `rejects_replay_job_without_permission`
  - `replay_job_requires_step_up`
  - `replay_job_enforces_dry_run_only`
  - `rejects_replay_lookup_without_permission`
- 然后执行：
  - `./scripts/smoke-local.sh`
  - `AUD_DB_SMOKE=1 cargo test -p platform-core audit_trace_api_db_smoke -- --nocapture`

## 覆盖闭环

- `audit_trace_api_db_smoke`
  - 创建订单审计对象与基础 audit trace
  - 通过 `POST /api/v1/audit/replay-jobs` 对 `order` 目标发起 `state_replay`，并强制 `dry_run=true`
  - 通过 `GET /api/v1/audit/replay-jobs/{id}` 回读 replay job
  - 真实回查：
    - `audit.replay_job`
    - `audit.replay_result`
    - `audit.audit_event(action_name in ('audit.replay.requested', 'audit.replay.completed'))`
    - `audit.access_audit(access_mode='replay')`
    - `ops.system_log(message_text like 'audit replay%')`
    - `s3://evidence-packages/replays/order/<order_id>/replay-<job_id>.json`
  - 差异报告必须至少包含四个正式步骤：
    - `target_snapshot`
    - `audit_timeline`
    - `evidence_projection`
    - `execution_policy`

## 关键不变量

- `V1` replay 只能 `dry_run=true`；`dry_run=false` 必须返回 `AUDIT_REPLAY_DRY_RUN_ONLY`
- replay 创建必须绑定 `audit.replay.execute` + step-up；读取必须绑定 `audit.replay.read`
- 差异报告不能只保留 report URI；必须真实包含：
  - 订单关键状态快照：`payment_status / delivery_status / settlement_status / dispute_status`
  - 审计时间线摘要：`trace_total` 与 preview
  - 证据投影摘要：`manifest_count / item_count / legal_hold_status`
  - 执行策略：`dry_run=true`、`side_effects_executed=false`
- replay 自身也必须留下正式审计、访问留痕与系统日志

## 关键回查

- replay job：

```sql
SELECT replay_type,
       ref_type,
       ref_id::text,
       dry_run,
       status,
       request_reason,
       options_json ->> 'report_storage_uri'
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;
```

- replay results：

```sql
SELECT step_name,
       result_code,
       diff_summary
FROM audit.replay_result
WHERE replay_job_id = '<replay_job_id>'::uuid
ORDER BY created_at, replay_result_id;
```

- 审计 / 访问 / 系统日志：

```sql
SELECT action_name,
       request_id,
       result_code
FROM audit.audit_event
WHERE ref_type = 'replay_job'
  AND ref_id = '<replay_job_id>'::uuid
ORDER BY event_time, audit_id;
```

```sql
SELECT access_mode,
       request_id,
       metadata
FROM audit.access_audit
WHERE target_type = 'replay_job'
  AND target_id = '<replay_job_id>'::uuid
ORDER BY created_at, access_audit_id;
```

## AUD 映射

- `AUD-CASE-004`：replay 创建
- `AUD-CASE-005`：replay 只允许 dry-run
- `AUD-CASE-006`：replay 读取
- `AUD-CASE-007`：高风险动作鉴权

## 禁止误报

- 只看 `audit.replay_job.status=completed` 不算通过；必须继续确认 `replay_result.diff_summary` 与 MinIO report 中的差异内容真实存在
- 只看 report 文件存在不算通过；必须继续确认文件内的 `results[*].step_name / diff_summary` 与 DB `audit.replay_result` 一致
- 只看 replay create 成功不算通过；必须继续确认 lookup、审计留痕、访问留痕和 `dry_run` 无副作用策略都成立
