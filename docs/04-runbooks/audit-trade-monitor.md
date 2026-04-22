# AUD-018 交易链监控总览与 Checkpoints

正式接口：

- `GET /api/v1/ops/trade-monitor/orders/{orderId}`
- `GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints`

正式目标：

- 返回订单维度的交易链总览
- 聚合 `trade.order_main`、`ops.trade_lifecycle_checkpoint`、`ops.external_fact_receipt`、`risk.fairness_incident`、`ops.chain_projection_gap`、`chain.chain_anchor`
- 查询动作必须真实写入 `audit.access_audit` 与 `ops.system_log`
- `tenant` 角色只能在 `buyer/seller order scope` 命中时读取

边界：

- 本接口是 `platform-core` 的只读 control-plane，不负责写链、回执确认或修复动作
- Fabric 的真实写链、Gateway、commit status、chaincode event listener、CA admin 仍由 `Go` 服务承担：
  - `services/fabric-adapter`
  - `services/fabric-event-listener`
  - `services/fabric-ca-admin`
- 本接口只读取这些 Go 执行面已经回写到 `PostgreSQL` 的外部事实和证明状态
- `external-facts / fairness-incidents / projection-gaps` 公共接口分别留给 `AUD-019 ~ AUD-021`

## 权限与租户范围

- 正式权限点：`ops.trade_monitor.read`
- 已落地平台角色：
  - `platform_admin`
  - `platform_audit_security`
  - `platform_risk_settlement`
  - `consistency_operator`
  - `audit_admin`
  - `node_ops_admin`
- 已落地租户角色：
  - `tenant_admin`
  - `tenant_audit_readonly`
- 租户侧还必须满足：
  - `x-tenant-id`
  - 订单 `buyer_org_id / seller_org_id` scope 命中

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

优先执行本批 live smoke，确认 API、权限、审计与日志都真实工作：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_trade_monitor_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入最小订单图
- 写入 `ops.trade_lifecycle_checkpoint`
- 写入 `ops.external_fact_receipt`
- 写入 `risk.fairness_incident`
- 写入 `ops.chain_projection_gap`
- 写入 `chain.chain_anchor`
- 调用两条正式 API
- 回查 `audit.access_audit + ops.system_log`
- 清理临时业务对象

## 手工查询

准备一个现存 `order_id` 后，先查总览：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/trade-monitor/orders/<order_id>" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud018-manual-overview' \
  -H 'x-trace-id: trace-aud018-manual'
```

再查 checkpoints 过滤页：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/trade-monitor/orders/<order_id>/checkpoints?checkpoint_status=pending&lifecycle_stage=delivery&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud018-manual-checkpoints' \
  -H 'x-trace-id: trace-aud018-manual'
```

租户侧联查：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/trade-monitor/orders/<order_id>" \
  -H 'x-role: tenant_admin' \
  -H 'x-tenant-id: <buyer_org_id_or_seller_org_id>' \
  -H 'x-request-id: req-aud018-manual-tenant' \
  -H 'x-trace-id: trace-aud018-manual'
```

预期重点：

- `business_state` 直接来自 `trade.order_main.status`
- `current_checkpoint_code / status` 来自最近 `ops.trade_lifecycle_checkpoint`
- `proof_commit_state / external_fact_status / reconcile_status` 来自双层权威状态
- `recent_checkpoints / recent_external_facts / recent_fairness_incidents / recent_projection_gaps` 可直接联查对应正式对象

## SQL 回查

查询读审计：

```sql
SELECT access_mode, target_type, target_id::text, request_id, trace_id
FROM audit.access_audit
WHERE request_id IN (
  'req-aud018-manual-overview',
  'req-aud018-manual-checkpoints',
  'req-aud018-manual-tenant'
)
ORDER BY created_at;
```

查询系统日志：

```sql
SELECT message_text, request_id, trace_id, structured_payload
FROM ops.system_log
WHERE request_id IN (
  'req-aud018-manual-overview',
  'req-aud018-manual-checkpoints',
  'req-aud018-manual-tenant'
)
ORDER BY created_at;
```

查询主状态：

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

查询 checkpoints：

```sql
SELECT trade_lifecycle_checkpoint_id::text,
       checkpoint_code,
       lifecycle_stage,
       checkpoint_status,
       occurred_at,
       request_id,
       trace_id
FROM ops.trade_lifecycle_checkpoint
WHERE order_id = '<order_id>'::uuid
ORDER BY COALESCE(occurred_at, expected_by, created_at) DESC,
         created_at DESC,
         trade_lifecycle_checkpoint_id DESC
LIMIT 20;
```

查询外部事实：

```sql
SELECT external_fact_receipt_id::text,
       fact_type,
       provider_type,
       receipt_status,
       confirmed_at,
       request_id,
       trace_id
FROM ops.external_fact_receipt
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY COALESCE(confirmed_at, received_at, occurred_at) DESC,
         external_fact_receipt_id DESC
LIMIT 20;
```

查询公平性事件：

```sql
SELECT fairness_incident_id::text,
       incident_type,
       severity,
       lifecycle_stage,
       status,
       request_id,
       trace_id
FROM risk.fairness_incident
WHERE order_id = '<order_id>'::uuid
ORDER BY created_at DESC, fairness_incident_id DESC
LIMIT 20;
```

查询链投影缺口：

```sql
SELECT chain_projection_gap_id::text,
       gap_type,
       gap_status,
       chain_id,
       expected_tx_id,
       projected_tx_hash,
       request_id,
       trace_id
FROM ops.chain_projection_gap
WHERE order_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 20;
```

查询最近链确认：

```sql
SELECT chain_anchor_id::text,
       chain_id,
       anchor_type,
       status,
       tx_hash,
       anchored_at,
       authority_model,
       reconcile_status
FROM chain.chain_anchor
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_anchor_id DESC
LIMIT 20;
```

## 排障

1. 返回 `404`

- 先确认 `order_id` 是否真实存在于 `trade.order_main`
- 再确认该订单是否已进入 `AUD` 读模型可联查范围

2. 返回 `403`

- 平台角色先确认 `x-role`
- 租户角色再确认 `x-tenant-id` 是否命中 `buyer_org_id / seller_org_id`
- 不允许租户通过 trade monitor 跨租户读订单

3. `current_checkpoint_code=not_started`

- 表示该订单当前没有任何 `ops.trade_lifecycle_checkpoint`
- 先回查 checkpoint 表，不要直接推断主链失败

4. `last_chain_confirmed_at` 为空

- 先回查 `chain.chain_anchor`
- 只有 `anchored / confirmed / committed / matched` 这类确认态才会折算到总览中的最近链确认时间

5. `recent_projection_gaps` 为空但怀疑存在链上链下漂移

- 先回查 `ops.chain_projection_gap`
- 不要把 `POST /api/v1/ops/consistency/reconcile` 或 Kafka topic 当本接口的替代真相源

6. 不要把 `dtp.outbox.domain-events` 当交易链总览的正式读源

- trade monitor 的正式权威读面是 `PostgreSQL`
- `Kafka` 只承载事件分发，不替代 `trade/order + audit + proof + external_fact` 的正式查询对象
