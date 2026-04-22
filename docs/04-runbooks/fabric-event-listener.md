# Fabric Event Listener（AUD-015）

`AUD-015` 起，`services/fabric-event-listener/` 作为正式 Go 进程落地，负责：

- 监听 `fabric-adapter` 已提交的链请求回执
- 生成 `fabric.commit_confirmed / fabric.commit_failed`
- 发布到正式 topic：`dtp.fabric.callbacks`
- 回写：
  - `ops.external_fact_receipt`
  - `audit.audit_event`
  - `ops.system_log`
  - `chain.chain_anchor`
  - `audit.anchor_batch`（仅证据批次根）

当前批次边界：

- local 模式先用“已提交回执 -> mock commit callback”闭环跑通
- 当前 provider mode 固定 `mock`
- `fabric-test-network / Gateway / chaincode` 真事件源切换留待 `AUD-017`
- 当前不直接修改订单/合同/结算等业务主状态

## 命令入口

```bash
make fabric-event-listener-bootstrap
make fabric-event-listener-test
make fabric-event-listener-run
```

等价脚本：

```bash
./scripts/fabric-event-listener-bootstrap.sh
./scripts/fabric-event-listener-test.sh
./scripts/fabric-event-listener-run.sh
```

Go 依赖缓存统一落在：

```text
third_party/external-deps/go
```

## 本地配置

默认从 `infra/docker/.env.local` 加载：

- `DATABASE_URL`
- `KAFKA_BROKERS` 或 `KAFKA_BOOTSTRAP_SERVERS`
- `TOPIC_FABRIC_CALLBACKS`
- `FABRIC_EVENT_LISTENER_PROVIDER_MODE`
- `FABRIC_CHANNEL_NAME`
- `FABRIC_CHAINCODE_NAME`
- `FABRIC_EVENT_LISTENER_POLL_INTERVAL`
- `FABRIC_EVENT_LISTENER_BATCH_SIZE`

当前本地默认值：

- Kafka：`127.0.0.1:9094`
- PostgreSQL：`postgres://datab:datab_local_pass@127.0.0.1:5432/datab`
- callback topic：`dtp.fabric.callbacks`
- provider mode：`mock`

## 回写口径

当前 listener 以 `ops.external_fact_receipt(fact_type in ('fabric_submit_receipt','fabric_anchor_submit_receipt'))` 为 source receipt：

- 若未标记 `listener_callback_event_id`
- 且 `receipt_status='submitted'`

则会生成一条 callback receipt，并补齐以下正式字段：

- `provider_code`
- `provider_request_id`
- `callback_event_id`
- `event_version`
- `provider_status`
- `provider_occurred_at`
- `payload_hash`

回写规则：

- `fabric.commit_confirmed`
  - `ops.external_fact_receipt.receipt_status='confirmed'`
  - `chain.chain_anchor.status='anchored'`
  - `chain.chain_anchor.reconcile_status='matched'`
  - `audit.anchor_batch.status='anchored'`（仅 anchor batch）
- `fabric.commit_failed`
  - `ops.external_fact_receipt.receipt_status='failed'`
  - `chain.chain_anchor.status='failed'`
  - `chain.chain_anchor.reconcile_status='pending_check'`
  - `audit.anchor_batch.status='failed'`（仅 anchor batch）

## 手工 Smoke

1. 启动 `fabric-adapter` 与 `fabric-event-listener`：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-adapter-run.sh
./scripts/fabric-event-listener-run.sh
```

2. 准备最小测试对象：

```sql
INSERT INTO chain.chain_anchor (
  chain_anchor_id, chain_id, anchor_type, ref_type, ref_id, digest, status, authority_model, reconcile_status
) VALUES
  ('12121212-1212-4212-8212-121212121212'::uuid, 'fabric-local', 'audit_batch', 'anchor_batch', '34343434-3434-4343-8343-343434343434'::uuid, 'aud015-root-anchor', 'pending', 'proof_layer', 'pending_check'),
  ('56565656-5656-4565-8565-565656565656'::uuid, 'fabric-local', 'order_summary', 'order', '78787878-7878-4787-8787-787878787878'::uuid, 'aud015-root-order', 'pending', 'proof_layer', 'pending_check');

INSERT INTO audit.anchor_batch (
  anchor_batch_id, batch_scope, chain_id, record_count, batch_root, status, chain_anchor_id, metadata
) VALUES (
  '34343434-3434-4343-8343-343434343434'::uuid,
  'audit_event',
  'fabric-local',
  1,
  'aud015-root-anchor',
  'retry_requested',
  '12121212-1212-4212-8212-121212121212'::uuid,
  '{}'::jsonb
);
```

3. 注入一条成功、一条失败的 canonical request：

```bash
cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.audit.anchor
{"event_id":"aud015-anchor-evt-kcat","event_type":"audit.anchor_requested","event_version":1,"occurred_at":"2026-04-22T10:20:00Z","producer_service":"platform-core.audit","aggregate_type":"audit.anchor_batch","aggregate_id":"34343434-3434-4343-8343-343434343434","request_id":"req-aud015-anchor-kcat","trace_id":"trace-aud015-anchor-kcat","idempotency_key":"idemp-aud015-anchor-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"anchor_batch_id":"34343434-3434-4343-8343-343434343434","batch_root":"aud015-root-anchor","chain_id":"fabric-local"},"anchor_batch_id":"34343434-3434-4343-8343-343434343434","batch_root":"aud015-root-anchor","chain_id":"fabric-local","chain_anchor_id":"12121212-1212-4212-8212-121212121212"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud015-order-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T10:20:01Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"56565656-5656-4565-8565-565656565656","request_id":"req-aud015-order-kcat","trace_id":"trace-aud015-order-kcat","idempotency_key":"idemp-aud015-order-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"56565656-5656-4565-8565-565656565656","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud015-root-order"},"chain_anchor_id":"56565656-5656-4565-8565-565656565656","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud015-root-order"}
JSON
```

4. 把订单摘要 source receipt 标记为失败回调，再等待 listener 轮询：

```sql
UPDATE ops.external_fact_receipt
SET metadata = metadata || jsonb_build_object('mock_callback_status', 'failed')
WHERE request_id = 'req-aud015-order-kcat'
  AND fact_type = 'fabric_submit_receipt';
```

5. 回查 callback topic 与数据库：

```bash
docker run --rm --network container:datab-kafka edenhill/kcat:1.7.1 \
  -b localhost:9092 -C -t dtp.fabric.callbacks -o beginning -e \
  -G cg-aud015-callback-smoke listener-aud015
```

```sql
SELECT request_id,
       fact_type,
       receipt_status,
       metadata ->> 'callback_event_id' AS callback_event_id,
       metadata ->> 'provider_status' AS provider_status,
       metadata ->> 'payload_hash' AS payload_hash
FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud015-anchor-kcat', 'req-aud015-order-kcat')
ORDER BY received_at ASC, external_fact_receipt_id ASC;

SELECT request_id, action_name, result_code
FROM audit.audit_event
WHERE request_id IN ('req-aud015-anchor-kcat', 'req-aud015-order-kcat')
  AND action_name = 'fabric.event_listener.callback'
ORDER BY request_id, created_at ASC;

SELECT request_id, message_text, structured_payload ->> 'event_type' AS event_type
FROM ops.system_log
WHERE request_id IN ('req-aud015-anchor-kcat', 'req-aud015-order-kcat')
  AND message_text = 'fabric event listener published callback'
ORDER BY request_id, created_at ASC;

SELECT chain_anchor_id::text, status, reconcile_status, anchored_at
FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '12121212-1212-4212-8212-121212121212'::uuid,
  '56565656-5656-4565-8565-565656565656'::uuid
)
ORDER BY chain_anchor_id;

SELECT anchor_batch_id::text, status, anchored_at
FROM audit.anchor_batch
WHERE anchor_batch_id = '34343434-3434-4343-8343-343434343434'::uuid;
```

预期：

- `dtp.fabric.callbacks` 中同时可见 `fabric.commit_confirmed` 与 `fabric.commit_failed`
- `ops.external_fact_receipt` 中新增：
  - `fabric_anchor_commit_receipt / confirmed`
  - `fabric_commit_receipt / failed`
- `audit.audit_event(action_name='fabric.event_listener.callback')` 各出现一次
- `ops.system_log(message_text='fabric event listener published callback')` 各出现一次
- 成功的 `chain.chain_anchor.status='anchored'`、`reconcile_status='matched'`
- 失败的 `chain.chain_anchor.status='failed'`
- `audit.anchor_batch.status='anchored'`

6. 清理测试业务数据：

```sql
DELETE FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud015-anchor-kcat', 'req-aud015-order-kcat');

DELETE FROM audit.anchor_batch
WHERE anchor_batch_id = '34343434-3434-4343-8343-343434343434'::uuid;

DELETE FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '12121212-1212-4212-8212-121212121212'::uuid,
  '56565656-5656-4565-8565-565656565656'::uuid
);
```

`audit.audit_event` 与 `ops.system_log` 按 append-only 保留，不清理。
