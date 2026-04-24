# TEST-013 Dispute Settlement Linkage

`TEST-013` 的正式目标是证明争议与结算链路不是拆散的局部 smoke，而是同一条正式业务闭环：买方打开争议后，订单与结算必须先进入冻结态；平台裁决后，再通过正式 `refund` / `compensation` 接口把调整金额写回结算聚合、审计和 outbox。

## 正式入口

- 本地 / CI checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-dispute-settlement-linkage.sh`
- checker 会先执行：
  - `./scripts/smoke-local.sh`
- 然后串行运行：
  - `TRADE_DB_SMOKE=1 cargo test -p platform-core bil019_dispute_refund_compensation_recompute_db_smoke -- --nocapture`

## 覆盖闭环

- `bil019_dispute_refund_compensation_recompute_db_smoke`
  - 为退款链路创建争议：
    - `POST /api/v1/cases`
    - `trade.order_main.settlement_status = frozen`
    - `trade.order_main.dispute_status = opened`
    - `billing.settlement_record.settlement_status = frozen`
    - `billing.billing_event(event_source = settlement_dispute_hold) = 1`
    - `GET /api/v1/billing/{order_id}` 返回 `settlement_summary.summary_state = order_settlement:frozen:manual`
  - 为赔付链路创建争议：
    - 同样验证冻结态和结算读模型冻结态
  - 平台裁决并执行正式调整：
    - `POST /api/v1/cases/{case_id}/resolve`
    - `POST /api/v1/refunds`
    - `POST /api/v1/compensations`
  - 裁决后回查：
    - `billing.settlement_record.refund_amount = 20.00000000`
    - `billing.settlement_record.compensation_amount = 20.00000000`
    - `GET /api/v1/billing/{order_id}` 返回
      - `refund_adjustment_amount = 20.00000000`
      - `compensation_adjustment_amount = 20.00000000`
      - `summary_state = order_settlement:pending:manual`
    - `billing.billing_event` 同时保留 `settlement_dispute_hold / settlement_dispute_release`
    - `ops.outbox_event` 保留 `support.dispute_case / billing.refund_record / billing.compensation_record`
    - `audit.audit_event` 保留
      - `dispute.case.create`
      - `dispute.case.resolve`
      - `billing.refund.execute`
      - `billing.compensation.execute`

## 关键不变量

- 争议冻结必须先发生在正式 `POST /api/v1/cases` 之后，而不是测试直接改表
- 结算冻结态必须同时出现在：
  - `trade.order_main`
  - `billing.settlement_record`
  - `GET /api/v1/billing/{order_id}` 的读模型
- 裁决后的退款 / 赔付必须走正式接口，不允许测试直接写 `billing.refund_record` / `billing.compensation_record`
- `billing.billing_event` 只能通过 `settlement_dispute_hold / settlement_dispute_release` 与正式退款/赔付记录来重算，不允许手改结算汇总字段
- 最终入账要同时能从 `PostgreSQL / API 读模型 / 审计 / outbox` 四处回查

## 关键回查

- 冻结态：

```sql
SELECT order_id::text,
       settlement_status,
       dispute_status,
       last_reason_code
FROM trade.order_main
WHERE order_id = '<order_uuid>'::uuid;
```

```sql
SELECT settlement_status,
       reason_code,
       refund_amount::text,
       compensation_amount::text
FROM billing.settlement_record
WHERE settlement_id = '<settlement_uuid>'::uuid;
```

- 调整事件：

```sql
SELECT event_type,
       event_source,
       amount::text
FROM billing.billing_event
WHERE order_id = '<order_uuid>'::uuid
  AND event_source IN ('settlement_dispute_hold', 'settlement_dispute_release')
ORDER BY created_at ASC, billing_event_id ASC;
```

- 审计 / outbox：

```sql
SELECT action_name,
       request_id,
       result_code
FROM audit.audit_event
WHERE request_id = ANY(ARRAY[
  '<case_create_req_id>',
  '<case_resolve_req_id>',
  '<refund_req_id>',
  '<compensation_req_id>'
]);
```

```sql
SELECT aggregate_type,
       aggregate_id::text,
       event_type,
       target_topic
FROM ops.outbox_event
WHERE aggregate_type IN (
  'support.dispute_case',
  'billing.refund_record',
  'billing.compensation_record'
)
ORDER BY created_at ASC, outbox_event_id ASC;
```

## PWB 映射

- `PWB-008`：争议升级冻结结算
- `PWB-009`：退款后结算重算
- `PWB-010`：赔付后结算重算

## 禁止误报

- 只看 `support.dispute_case` 存在一条 opened / resolved 记录不算通过；必须继续确认结算和读模型已同步冻结/解冻
- 只看 `billing.refund_record` 或 `billing.compensation_record` 新增不算通过；必须继续确认 `billing.settlement_record` 与 `GET /api/v1/billing/{order_id}` 的金额聚合一致
- 只看最终 `refund_amount / compensation_amount` 正确不算通过；必须继续证明争议打开时确实进入过冻结态，而不是跳过中间态直接入账
