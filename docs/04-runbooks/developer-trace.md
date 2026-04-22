# AUD-024 开发者状态联查

正式接口：

- `GET /api/v1/developer/trace`

正式目标：

- 允许开发者或平台审计/调试角色按单一 selector 联查正式运行态对象
- `order_id / event_id / tx_hash` 三种 selector 都必须真实命中 PostgreSQL 权威对象
- 返回订单主状态、proof/anchor 状态、外部事实状态、最近 outbox / dead letter / audit trace / trace index / system log
- 查询动作必须写入 `audit.access_audit` 与 `ops.system_log`

正式边界：

- 本接口是只读开发者联查，不执行修复、重放、重处理或链上动作
- `Kafka / Fabric / Tempo / Loki` 不是本接口的真相源；正式读模型来自 `trade.order_main`、`audit.audit_event`、`ops.outbox_event`、`ops.dead_letter_event`、`ops.external_fact_receipt`、`ops.chain_projection_gap`、`ops.trade_lifecycle_checkpoint`、`ops.trace_index`、`ops.system_log`、`chain.chain_anchor`
- Go 服务 `fabric-adapter / fabric-event-listener / fabric-ca-admin` 负责把链提交、回执、callback、证书治理结果回写正式对象；本接口只联查这些正式回写结果

## 权限与 scope

- 正式权限：`developer.trace.read`
- 当前实现兼容开发者角色种子 `tenant_developer / developer_admin`
- 平台联调角色兼容：
  - `platform_audit_security`
  - `platform_admin`
- tenant 侧必须同时满足：
  - 提供 `x-tenant-id`
  - 命中订单 buyer/seller scope

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

如需让 proof / callback 相关字段真实可见，同时启动：

- `./scripts/fabric-adapter-run.sh`
- `./scripts/fabric-event-listener-run.sh`

## 自动化 smoke

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core developer_trace_api_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入 `ops.outbox_event / ops.dead_letter_event / ops.external_fact_receipt / chain.chain_anchor / ops.chain_projection_gap / ops.trade_lifecycle_checkpoint / ops.trace_index / ops.system_log / audit.audit_event`
- 分别按 `order_id / event_id / tx_hash` 调用 `GET /api/v1/developer/trace`
- 回查 `audit.access_audit(target_type='developer_trace_query')` 与 `ops.system_log(message_text='developer trace lookup executed: GET /api/v1/developer/trace')`

## 手工调用

### 1. 按订单联查

```bash
curl -sS "http://127.0.0.1:18080/api/v1/developer/trace?order_id=<order_id>" \
  -H 'x-role: developer_admin' \
  -H 'x-user-id: <developer_user_id>' \
  -H 'x-tenant-id: <tenant_org_id>' \
  -H 'x-request-id: req-aud024-order' \
  -H 'x-trace-id: trace-aud024-order'
```

### 2. 按事件联查

`event_id` 可使用 `ops.outbox_event.outbox_event_id` 或 `audit.audit_event.audit_id`：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/developer/trace?event_id=<outbox_event_id_or_audit_id>" \
  -H 'x-role: developer_admin' \
  -H 'x-user-id: <developer_user_id>' \
  -H 'x-tenant-id: <tenant_org_id>' \
  -H 'x-request-id: req-aud024-event' \
  -H 'x-trace-id: trace-aud024-event'
```

### 3. 按链交易哈希联查

```bash
curl -sS "http://127.0.0.1:18080/api/v1/developer/trace?tx_hash=<tx_hash>" \
  -H 'x-role: developer_admin' \
  -H 'x-user-id: <developer_user_id>' \
  -H 'x-tenant-id: <tenant_org_id>' \
  -H 'x-request-id: req-aud024-tx' \
  -H 'x-trace-id: trace-aud024-tx'
```

预期重点：

- 只能传一个 selector；多传或全空返回 `400`
- `subject.lookup_mode / lookup_value / matched_object_type / matched_object_id` 与 selector 一致
- `subject.snapshot` 能看到订单状态快照与 authority / version 等上下文
- `matched_chain_anchor / matched_projection_gap / matched_checkpoint / trace` 会按正式对象回填
- `recent_logs / recent_outbox_events / recent_dead_letters / recent_audit_traces` 可直接串联排障

## SQL 回查

访问留痕：

```sql
SELECT request_id,
       access_mode,
       target_type,
       target_id::text,
       metadata
FROM audit.access_audit
WHERE request_id IN ('req-aud024-order', 'req-aud024-event', 'req-aud024-tx')
  AND target_type = 'developer_trace_query';
```

系统日志：

```sql
SELECT request_id,
       trace_id,
       message_text,
       structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud024-order', 'req-aud024-event', 'req-aud024-tx')
  AND message_text = 'developer trace lookup executed: GET /api/v1/developer/trace';
```

链锚与 checkpoint：

```sql
SELECT chain_anchor_id::text,
       chain_id,
       anchor_type,
       status,
       tx_hash,
       reconcile_status
FROM chain.chain_anchor
WHERE ref_type = 'order'
  AND ref_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_anchor_id DESC
LIMIT 10;

SELECT trade_lifecycle_checkpoint_id::text,
       checkpoint_code,
       checkpoint_status,
       related_tx_hash,
       request_id,
       trace_id
FROM ops.trade_lifecycle_checkpoint
WHERE order_id = '<order_id>'::uuid
ORDER BY created_at DESC, trade_lifecycle_checkpoint_id DESC
LIMIT 10;
```

## 排障

1. 返回 `400`

- 确认只传了一个 selector
- `order_id / event_id` 必须是 UUID

2. 返回 `403`

- 确认角色具备 `developer.trace.read`
- tenant 角色要补 `x-tenant-id`
- 再查订单 buyer/seller org 是否和 `x-tenant-id` 一致

3. `tx_hash` 查不到结果

- 先回查 `chain.chain_anchor.tx_hash`
- 若只有 callback/receipt 没有 anchor，再查 `audit.audit_event.tx_hash`、`ops.trade_lifecycle_checkpoint.related_tx_hash`

4. 看不到 Fabric 回写状态

- 确认 Go 服务已经把状态回写到正式对象，而不是只停在容器日志
- 先查 `ops.external_fact_receipt / chain.chain_anchor / ops.trade_lifecycle_checkpoint / ops.system_log`
- 不要把 Gateway、Peer、Orderer 的即时返回当作本接口真相源
