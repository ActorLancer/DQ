# AUD-019 外部事实查询与确认

正式接口：

- `GET /api/v1/ops/external-facts`
- `POST /api/v1/ops/external-facts/{id}/confirm`

正式目标：

- 以 `ops.external_fact_receipt` 作为唯一正式持久化对象返回外部事实回执
- 查询动作必须真实写入 `audit.access_audit` 与 `ops.system_log`
- 确认动作必须要求 `ops.external_fact.manage + step-up + audit + system log`
- 确认动作只允许回写 `receipt_status / confirmed_at / metadata`
- 确认动作不得直接改写 `trade.order_main.external_fact_status`

边界：

- 本接口是 `platform-core` 的高风险 control-plane，不直接推动业务主状态机
- `confirm` 只在 `receipt_status='pending'` 时允许执行
- 当前通过 `ops.external_fact_receipt.metadata.rule_evaluation.status='pending_follow_up'` 保留后续规则评估正式信号；真正的公平性事件处理与 projection gap 关闭分别留给 `AUD-020 / AUD-021`
- Fabric / Mock Payment / 其他外围系统继续负责产生原始 receipt；本接口只做平台侧确认和留痕

## 权限

- `GET /api/v1/ops/external-facts`
  - `ops.external_fact.read`
  - 当前已落地角色：`platform_admin`、`platform_audit_security`、`platform_risk_settlement`
- `POST /api/v1/ops/external-facts/{id}/confirm`
  - `ops.external_fact.manage`
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
  cargo test -p platform-core audit_external_fact_confirm_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入最小订单图与一条 `ops.external_fact_receipt(receipt_status='pending')`
- 调用 `GET /api/v1/ops/external-facts`
- 调用 `POST /api/v1/ops/external-facts/{id}/confirm`
- 回查 `ops.external_fact_receipt / audit.audit_event / audit.access_audit / ops.system_log`
- 断言 `trade.order_main.external_fact_status` 保持原值，不被 confirm 直接改写

## 手工查询

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/external-facts?order_id=<order_id>&receipt_status=pending&provider_type=mock_payment_provider&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud019-list' \
  -H 'x-trace-id: trace-aud019'
```

预期重点：

- 返回 `ops.external_fact_receipt` 分页结果
- 可直接看到 `fact_type / provider_type / provider_reference / receipt_status / receipt_hash / confirmed_at`
- 查询动作落 `audit.access_audit(access_mode='masked', target_type='external_fact_query')`

## 手工确认

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/external-facts/<external_fact_receipt_id>/confirm" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud019-confirm' \
  -H 'x-trace-id: trace-aud019' \
  -H 'x-step-up-challenge-id: <confirm_step_up_id>' \
  -d '{
    "confirm_result": "confirmed",
    "reason": "operator verified payment callback",
    "operator_note": "provider callback digest matches expected invoice"
  }'
```

预期重点：

- `status=manual_confirmation_recorded`
- `confirm_result=confirmed`
- `external_fact_receipt.receipt_status=confirmed`
- `rule_evaluation_status=pending_follow_up`

## SQL 回查

查询回执：

```sql
SELECT external_fact_receipt_id::text,
       fact_type,
       provider_type,
       receipt_status,
       confirmed_at,
       metadata -> 'manual_confirmation' ->> 'confirm_result' AS confirm_result,
       metadata -> 'manual_confirmation' ->> 'reason' AS confirm_reason,
       metadata -> 'rule_evaluation' ->> 'status' AS rule_evaluation_status
FROM ops.external_fact_receipt
WHERE external_fact_receipt_id = '<external_fact_receipt_id>'::uuid;
```

确认未直接改主状态：

```sql
SELECT order_id::text,
       external_fact_status,
       reconcile_status,
       proof_commit_state
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;
```

查询审计：

```sql
SELECT action_name, result_code, ref_type, ref_id::text, metadata
FROM audit.audit_event
WHERE request_id = 'req-aud019-confirm'
  AND action_name = 'ops.external_fact.confirm';
```

查询 access audit：

```sql
SELECT access_mode, target_type, target_id::text, step_up_challenge_id::text
FROM audit.access_audit
WHERE request_id IN ('req-aud019-list', 'req-aud019-confirm')
ORDER BY created_at;
```

查询系统日志：

```sql
SELECT message_text, request_id, trace_id, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud019-list', 'req-aud019-confirm')
ORDER BY created_at;
```

## 排障

1. 返回 `400`

- 先检查 `confirm_result / reason`
- 再检查 `x-step-up-token` 或 `x-step-up-challenge-id` 是否存在
- 如果是 verified challenge，确认其 `target_action='ops.external_fact.manage'`、`target_ref_type='external_fact_receipt'`

2. 返回 `403`

- 先确认角色属于 `platform_admin / platform_audit_security`
- 再确认 step-up challenge 属于当前 `user_id` 且状态为 `verified`

3. 返回 `404`

- 回查 `ops.external_fact_receipt` 是否存在

4. 返回 `409`

- 当前只允许 `receipt_status='pending'` 的回执执行 confirm
- 若已经是 `confirmed / matched / mismatched / rejected`，这是预期保护，不要强行覆盖

5. 发现 `trade.order_main.external_fact_status` 被同步改动

- 这属于缺陷
- `AUD-019` 的边界是“只确认 receipt，不直接改业务 / 技术镜像主状态”
