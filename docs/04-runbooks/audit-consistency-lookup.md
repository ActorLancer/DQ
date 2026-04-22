# AUD-011 一致性联查

正式接口：

- `GET /api/v1/ops/consistency/{refType}/{refId}`

正式目标：

- 返回业务对象主状态
- 返回最新 proof / chain anchor 状态
- 返回外部事实状态
- 返回最近 outbox / dead letter / audit trace
- 查询动作必须写入 `audit.access_audit` 与 `ops.system_log`

当前 `V1` 支持的正式 `refType`：

- `order`
- `contract` / `digital_contract`
- `delivery` / `delivery_record`
- `settlement` / `settlement_record`
- `payment` / `payment_intent`
- `refund` / `refund_intent`
- `payout` / `payout_instruction`

边界：

- 本接口是只读联查，不执行修复动作
- `POST /api/v1/ops/consistency/reconcile` 留给 `AUD-012`
- Go / Fabric request / callback 真正交互链路留给 `AUD-013+`

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

## 手工查询

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/consistency/order/<order_id>" \
  -H "x-role: platform_audit_security" \
  -H "x-user-id: <operator_user_id>" \
  -H "x-request-id: req-aud011-manual" \
  -H "x-trace-id: trace-aud011-manual"
```

预期重点：

- `business_state` 能看到 `authority_model / business_state_version / proof_commit_state / external_fact_status / reconcile_status`
- `proof_state` 能看到 `latest_chain_anchor / projection_gap_status_breakdown / latest_projection_gap`
- `external_fact_state` 能看到 `receipt_status_breakdown / latest_receipt`
- `recent_outbox_events / recent_dead_letters / recent_audit_traces` 非空时可直接联查

## SQL 回查

```sql
SELECT access_mode, target_type, target_id::text, request_id, trace_id
FROM audit.access_audit
WHERE request_id = 'req-aud011-manual';
```

```sql
SELECT message_text, request_id, trace_id, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud011-manual'
  AND message_text = 'ops lookup executed: GET /api/v1/ops/consistency/{refType}/{refId}';
```

```sql
SELECT order_id::text,
       status,
       authority_model,
       business_state_version,
       proof_commit_state,
       proof_commit_policy,
       external_fact_status,
       reconcile_status,
       last_reconciled_at
FROM trade.order_main
WHERE order_id = '<order_id>'::uuid;
```

```sql
SELECT chain_anchor_id::text,
       chain_id,
       anchor_type,
       status,
       tx_hash,
       authority_model,
       reconcile_status
FROM chain.chain_anchor
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_anchor_id DESC
LIMIT 5;
```

```sql
SELECT chain_projection_gap_id::text,
       gap_type,
       gap_status,
       outbox_event_id::text,
       anchor_id::text,
       request_id,
       trace_id
FROM ops.chain_projection_gap
WHERE aggregate_type = 'order'
  AND aggregate_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 10;
```

```sql
SELECT external_fact_receipt_id::text,
       fact_type,
       provider_type,
       receipt_status,
       request_id,
       trace_id
FROM ops.external_fact_receipt
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY received_at DESC, external_fact_receipt_id DESC
LIMIT 10;
```

## 排障

1. 返回 `404`

- 先确认 `refType` 已规范化到正式对象
- 再查对应业务表是否已有该 `refId`

2. `proof_state.latest_chain_anchor` 为空

- 先回查 `chain.chain_anchor`
- 再确认当前对象是否真的已经进入 proof 提交流程

3. `recent_outbox_events` 为空

- 先回查 `ops.outbox_event.aggregate_type / aggregate_id`
- 不要把 `dtp.outbox.domain-events` 当作此接口的替代真相源

4. `recent_dead_letters` 为空但确实存在处理失败

- 回查 `ops.dead_letter_event.aggregate_type / aggregate_id`
- 再确认失败是否已经只落到了 Kafka `dtp.dead-letter` 而没有回写 DB；若是，这属于后续闭环缺陷，不应在本接口层面硬补假数据
