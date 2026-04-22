# AUD-020 公平性事件查询与处理

正式接口：

- `GET /api/v1/ops/fairness-incidents`
- `POST /api/v1/ops/fairness-incidents/{id}/handle`

正式目标：

- 以 `risk.fairness_incident` 作为唯一正式持久化对象返回公平性事件
- 查询动作必须真实写入 `audit.access_audit` 与 `ops.system_log`
- 处理动作必须要求 `risk.fairness_incident.handle + step-up + audit + system log`
- 处理动作只允许回写 `risk.fairness_incident.status / auto_action_code / resolution_summary / metadata / closed_at`
- `freeze_settlement / freeze_delivery / create_dispute_suggestion` 仅作为联动建议写入 metadata，不得直接改写 `trade.order_main`

边界：

- 本接口是 `platform-core` 的高风险 control-plane，不直接推动业务主状态机
- 当前允许动作：`acknowledge`、`escalate`、`close`
- 只有 `close` 会把 `fairness_incident_status` 更新为 `closed` 并写入 `closed_at`
- `acknowledge / escalate` 只记录人工处理与联动建议，事件状态保持 `open`
- 真正的业务修复、冻结、争议创建或补救执行，仍由后续正式业务接口 / worker / reconcile 链路承接

## 权限

- `GET /api/v1/ops/fairness-incidents`
  - `risk.fairness_incident.read`
  - 当前已落地角色：`platform_admin`、`platform_audit_security`、`platform_risk_settlement`
- `POST /api/v1/ops/fairness-incidents/{id}/handle`
  - `risk.fairness_incident.handle`
  - 当前已落地角色：`platform_admin`、`platform_risk_settlement`
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
  cargo test -p platform-core audit_fairness_incident_handle_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入最小订单图、`ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt` 与 `risk.fairness_incident(status='open')`
- 调用 `GET /api/v1/ops/fairness-incidents`
- 调用 `POST /api/v1/ops/fairness-incidents/{id}/handle`
- 回查 `risk.fairness_incident / trade.order_main / audit.audit_event / audit.access_audit / ops.system_log`
- 断言 `freeze_settlement / create_dispute_suggestion` 仅作为 `linked_action_plan` 建议落盘，不直接改写订单主状态

## 手工查询

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/fairness-incidents?order_id=<order_id>&incident_type=seller_delivery_delay&severity=high&fairness_incident_status=open&assigned_role_key=platform_risk_settlement&assigned_user_id=<operator_user_id>&page=1&page_size=20" \
  -H 'x-role: platform_risk_settlement' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud020-list' \
  -H 'x-trace-id: trace-aud020'
```

预期重点：

- 返回 `risk.fairness_incident` 分页结果
- 可直接看到 `incident_type / severity / lifecycle_stage / fairness_incident_status / auto_action_code / assigned_role_key / assigned_user_id`
- 查询动作落 `audit.access_audit(access_mode='masked', target_type='fairness_incident_query')`

## 手工处理

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/fairness-incidents/<fairness_incident_id>/handle" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_risk_settlement' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud020-handle' \
  -H 'x-trace-id: trace-aud020' \
  -H 'x-step-up-challenge-id: <handle_step_up_id>' \
  -d '{
    "action": "close",
    "resolution_summary": "manual review confirmed delivery delay risk",
    "auto_action_override": "notify_ops",
    "freeze_settlement": true,
    "freeze_delivery": false,
    "create_dispute_suggestion": true
  }'
```

预期重点：

- `status=manual_handling_recorded`
- `action=close`
- `action_plan_status=suggestion_recorded`
- `fairness_incident.fairness_incident_status=closed`
- `linked_action_plan.execution_mode=suggestion_only`

## SQL 回查

查询事件：

```sql
SELECT fairness_incident_id::text,
       order_id::text,
       incident_type,
       severity,
       status,
       auto_action_code,
       resolution_summary,
       closed_at,
       metadata -> 'handling' ->> 'action' AS handled_action,
       metadata -> 'linked_action_plan' ->> 'status' AS action_plan_status,
       metadata -> 'linked_action_plan' ->> 'execution_mode' AS execution_mode
FROM risk.fairness_incident
WHERE fairness_incident_id = '<fairness_incident_id>'::uuid;
```

确认不直接改主状态：

```sql
SELECT order_id::text,
       settlement_status,
       delivery_status,
       dispute_status
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;
```

查询审计：

```sql
SELECT action_name, result_code, ref_type, ref_id::text, metadata
FROM audit.audit_event
WHERE request_id = 'req-aud020-handle'
  AND action_name = 'risk.fairness_incident.handle';
```

查询 access audit：

```sql
SELECT access_mode, target_type, target_id::text, step_up_challenge_id::text
FROM audit.access_audit
WHERE request_id IN ('req-aud020-list', 'req-aud020-handle')
ORDER BY created_at;
```

查询系统日志：

```sql
SELECT message_text, request_id, trace_id, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud020-list', 'req-aud020-handle')
ORDER BY created_at;
```

## 排障

1. 返回 `400`

- 先检查 `action / resolution_summary`
- 再检查 `x-step-up-token` 或 `x-step-up-challenge-id` 是否存在
- 如果是 verified challenge，确认其 `target_action='risk.fairness_incident.handle'`、`target_ref_type='fairness_incident'`

2. 返回 `403`

- 先确认查询角色属于 `platform_admin / platform_audit_security / platform_risk_settlement`
- 处理动作角色只允许 `platform_admin / platform_risk_settlement`
- 再确认 step-up challenge 属于当前 `user_id` 且状态为 `verified`

3. 返回 `404`

- 回查 `risk.fairness_incident` 是否存在

4. 返回 `409`

- 当前只允许 `status='open'` 的事件执行 handle
- 若已经 `closed`，这是预期保护，不要强行覆盖

5. 发现 `trade.order_main` 被同步改动

- 这属于缺陷
- `AUD-020` 的边界是“只处理公平性事件与联动建议，不直接改业务主状态”
