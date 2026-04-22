# AUD-012 一致性修复 dry-run

正式接口：

- `POST /api/v1/ops/consistency/reconcile`

正式目标：

- 以 `ops.chain_projection_gap` 作为正式持久化查询对象生成修复建议
- 高风险动作必须要求权限、`step-up`、正式审计与系统留痕
- `V1` 只支持 `dry_run=true`
- 当前不会引入 `reconcile_job` 表
- 当前不会真正写出 `dtp.consistency.reconcile` 执行事件

边界：

- 本接口是控制面 dry-run 预演，不修改 `ops.chain_projection_gap`
- 真正执行型 reconcile worker、Go/Fabric 交互与 callback 修复留给后续 `AUD-013+`
- `AUD-021` 继续承接 `projection-gaps` 公共查询 / 关闭接口

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

## 手工 dry-run 调用

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/consistency/reconcile" \
  -H "content-type: application/json" \
  -H "x-role: platform_audit_security" \
  -H "x-user-id: <operator_user_id>" \
  -H "x-request-id: req-aud012-manual" \
  -H "x-trace-id: trace-aud012-manual" \
  -H "x-step-up-challenge-id: <reconcile_step_up_id>" \
  -d '{
    "ref_type": "order",
    "ref_id": "<order_id>",
    "mode": "full",
    "dry_run": true,
    "reason": "manual consistency reconcile preview"
  }'
```

当前 `mode`：

- `projection_gap`：只基于 `ops.chain_projection_gap` 输出建议
- `full`：在 `projection_gap` 基础上，追加 `proof_commit_state / external_fact_status / dead letter` 辅助建议

## 预期响应重点

- `status=dry_run_ready`
- `reconcile_target_topic=dtp.consistency.reconcile`
- `recommendations` 非空
- `subject_snapshot` 含 `proof_commit_state / external_fact_status / reconcile_status`
- `related_projection_gaps` 直接回显当前关联的正式缺口对象

## SQL 回查

```sql
SELECT action_name, result_code, metadata ->> 'mode'
FROM audit.audit_event
WHERE request_id = 'req-aud012-manual'
  AND action_name = 'ops.consistency.reconcile.dry_run';
```

```sql
SELECT access_mode, target_type, target_id::text, step_up_challenge_id::text
FROM audit.access_audit
WHERE request_id = 'req-aud012-manual';
```

```sql
SELECT message_text, request_id, trace_id, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud012-manual'
  AND message_text = 'ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile';
```

```sql
SELECT chain_projection_gap_id::text,
       gap_type,
       gap_status,
       resolution_summary,
       updated_at
FROM ops.chain_projection_gap
WHERE aggregate_type = 'order'
  AND aggregate_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 10;
```

```sql
SELECT count(*) AS reconcile_preview_outbox_count
FROM ops.outbox_event
WHERE request_id = 'req-aud012-manual'
  AND target_topic = 'dtp.consistency.reconcile';
```

## 排障

1. 返回 `400`

- 先检查 `ref_type / ref_id / mode / reason`
- 再检查 `x-step-up-challenge-id` 是否绑定到了同一 `user_id / target_action / target_ref_type / target_ref_id`

2. 返回 `403`

- 先确认角色属于 `platform_admin / platform_audit_security`
- 再确认 `iam.step_up_challenge.challenge_status='verified'` 且未过期

3. 返回 `404`

- 先回查对应业务对象是否存在
- 再确认该对象是否已经具备 dual-authority 镜像字段

4. `recommendations` 为空或只有 `monitor_only`

- 先查 `ops.chain_projection_gap`
- 再查对象当前 `proof_commit_state / external_fact_status / reconcile_status`
- 若当前无开放缺口且状态已稳定，这是预期，不要为了“有建议”伪造缺口

5. 出现新的 `dtp.consistency.reconcile` outbox 行

- 当前 `AUD-012` 不应真正写出执行事件
- 若发现本请求写出了 `target_topic='dtp.consistency.reconcile'` 的 outbox，请按缺陷处理，不要把 dry-run 预演误当执行模式
