# Audit Legal Hold Runbook

当前 runbook 承接 `AUD-006` 的正式 legal hold 控制面：

- `POST /api/v1/audit/legal-holds`
- `POST /api/v1/audit/legal-holds/{id}/release`

`V1` 当前只开放 `order`、`case` / `dispute_case` 两类 scope。创建和释放都属于高风险动作，必须经过权限、step-up 和正式审计留痕。

## 权限与 Step-Up

- 创建 / 解除 legal hold：`audit.legal_hold.manage`
- 两个动作都必须提供：
  - `x-user-id`
  - `x-request-id`
  - `x-step-up-token` 或 `x-step-up-challenge-id`
- 若使用 `x-step-up-challenge-id`：
  - challenge 必须属于当前 `x-user-id`
  - `challenge_status=verified`
  - 未过期
  - `target_action='audit.legal_hold.manage'`
  - 创建时如绑定 `target_ref_type / target_ref_id`，必须与当前 `hold_scope_type / hold_scope_id` 一致
  - 释放时当前实现要求绑定 `target_ref_type='legal_hold'` 且 `target_ref_id=<legal_hold_id>`

## 创建 legal hold

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-runbook-create' \
  -H 'x-trace-id: trace-aud006-runbook-create' \
  -H 'x-step-up-challenge-id: <legal_hold_create_challenge_id>' \
  -d '{
    "hold_scope_type": "order",
    "hold_scope_id": "<order_id>",
    "reason_code": "regulator_investigation",
    "metadata": {
      "ticket": "AUD-OPS-006",
      "trigger": "runbook"
    }
  }'
```

预期：

- 返回 `legal_hold.status=active`
- 返回 `step_up_bound=true`
- 同 scope 下再次创建会返回 `409 AUDIT_LEGAL_HOLD_ACTIVE`

## 解除 legal hold

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds/<legal_hold_id>/release \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-runbook-release' \
  -H 'x-trace-id: trace-aud006-runbook-release' \
  -H 'x-step-up-challenge-id: <legal_hold_release_challenge_id>' \
  -d '{
    "reason": "manual review cleared hold",
    "metadata": {
      "resolution": "cleared"
    }
  }'
```

预期：

- 返回 `legal_hold.status=released`
- `approved_by=<audit_user_id>`
- `released_at` 非空

## 数据回查

1. hold 主记录：

```sql
SELECT legal_hold_id::text,
       hold_scope_type,
       hold_scope_id::text,
       reason_code,
       status,
       requested_by::text,
       approved_by::text,
       hold_until,
       released_at,
       metadata
FROM audit.legal_hold
WHERE legal_hold_id = '<legal_hold_id>'::uuid;
```

2. scope 关联 evidence 状态：

```sql
SELECT evidence_item_id::text, legal_hold_status
FROM audit.evidence_item
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY created_at DESC;

SELECT evidence_package_id::text, legal_hold_status
FROM audit.evidence_package
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY created_at DESC;
```

说明：

- `audit.evidence_item`、`audit.evidence_package` 都是 append-only 历史快照
- legal hold create/release 不会回写这些旧记录
- 当前是否处于保全状态，以 `audit.legal_hold.status` 为正式权威源

3. 审计与日志留痕：

```sql
SELECT action_name, result_code, metadata
FROM audit.audit_event
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
  AND action_name IN ('audit.legal_hold.create', 'audit.legal_hold.release')
ORDER BY event_time, audit_id;

SELECT request_id, message_text, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud006-runbook-create', 'req-aud006-runbook-release')
ORDER BY created_at, system_log_id;
```

预期：

- `audit.audit_event` 中有 `audit.legal_hold.create` / `audit.legal_hold.release`
- `ops.system_log.message_text` 包含：
  - `audit legal hold created: POST /api/v1/audit/legal-holds`
  - `audit legal hold released: POST /api/v1/audit/legal-holds/{id}/release`

## 观测联查

- `Loki`：检索 `request_id=req-aud006-runbook-create`、`legal_hold_id=<id>`
- `Tempo`：用 `x-trace-id` 关联 `platform-core` 高风险控制面请求链
- `Prometheus / Grafana`：确认 `platform-core` 请求链路可见；必要时联查相同 `ref_type / ref_id`
- `Alertmanager`：当前 task 不新增专属 legal hold 告警规则；如本次 hold 源于事故或监管事件，应把事件单号写入 request metadata

## 常见故障

### 1. 返回 `403 forbidden`

检查：

- `x-role` 是否具备 `audit.legal_hold.manage`
- 是否误用租户角色而非平台审计角色

### 2. 返回 `400 missing step-up`

检查：

- 是否缺少 `x-step-up-token` 和 `x-step-up-challenge-id`
- challenge 是否已过期、未验证，或目标绑定不匹配

### 3. 返回 `409 AUDIT_LEGAL_HOLD_ACTIVE`

说明：

- 当前 scope 已有活动 legal hold
- 需先复核现有 hold，再决定是否释放或沿用，不得通过直接改表绕过

## 清理规则

- 业务测试数据可清理：临时 `trade.order_main` 与本次为 scope 造数的 `core.organization / catalog.*` 图数据
- `audit.audit_event`、`ops.system_log`、`audit.legal_hold` 与已被高风险动作留痕引用的 `iam.step_up_challenge / core.user_account` 按运行态现状保留
- 注意：当前库里的 `iam.step_up_challenge` 删除会触发 FK 尝试把 `audit.audit_event.step_up_challenge_id` 置空，而 `audit.audit_event` 是 append-only；不要为了回归环境清理去强行改写审计链
