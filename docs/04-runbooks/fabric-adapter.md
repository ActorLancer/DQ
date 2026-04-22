# Fabric Adapter（AUD-013）

`AUD-013` 起，`services/fabric-adapter/` 作为正式 Go 进程落地，负责消费：

- `dtp.audit.anchor`
- `dtp.fabric.requests`

并将提交回执写回：

- `ops.external_fact_receipt`
- `audit.audit_event`
- `ops.system_log`
- `chain.chain_anchor`（存在 `chain_anchor_id` 时更新 `tx_hash / status / reconcile_status`）

当前批次边界：

- 已落地 Go module、Kafka consumer、canonical envelope 解析、mock provider、PostgreSQL 回执写回
- 当前 provider 仍是 `mock`
- `fabric-test-network / Gateway / chaincode / listener / CA admin` 留待 `AUD-014~AUD-017`
- 当前不消费 `dtp.outbox.domain-events`

## 命令入口

```bash
make fabric-adapter-bootstrap
make fabric-adapter-test
make fabric-adapter-run
```

等价脚本：

```bash
./scripts/fabric-adapter-bootstrap.sh
./scripts/fabric-adapter-test.sh
./scripts/fabric-adapter-run.sh
```

Go 依赖缓存统一落在：

```text
third_party/external-deps/go
```

不要再向 `services/**` 或用户主目录写第二套缓存目录。

## 本地配置

默认从 `infra/docker/.env.local` 加载：

- `DATABASE_URL`
- `KAFKA_BROKERS` 或 `KAFKA_BOOTSTRAP_SERVERS`
- `TOPIC_AUDIT_ANCHOR`
- `TOPIC_FABRIC_REQUESTS`
- `FABRIC_ADAPTER_CONSUMER_GROUP`
- `FABRIC_CHANNEL_NAME`
- `FABRIC_CHAINCODE_NAME`

当前本地默认值：

- Kafka：`127.0.0.1:9094`
- PostgreSQL：`postgres://datab:datab_local_pass@127.0.0.1:5432/datab`
- consumer group：`cg-fabric-adapter`

## 手工 Smoke

1. 启动适配器：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-adapter-run.sh
```

2. 准备最小测试对象：

```sql
INSERT INTO chain.chain_anchor (chain_anchor_id, chain_id, anchor_type, ref_type, ref_id, digest, status)
VALUES
  ('11111111-1111-4111-8111-111111111111'::uuid, 'fabric-local', 'audit_anchor_batch', 'anchor_batch', '22222222-2222-4222-8222-222222222222'::uuid, 'aud013-root-1', 'pending'),
  ('33333333-3333-4333-8333-333333333333'::uuid, 'fabric-local', 'order_summary', 'chain_anchor', NULL, 'aud013-proof-root', 'pending')
ON CONFLICT (chain_anchor_id) DO NOTHING;

INSERT INTO audit.anchor_batch (
  anchor_batch_id, batch_scope, chain_id, record_count, batch_root, status, chain_anchor_id, metadata
) VALUES (
  '22222222-2222-4222-8222-222222222222'::uuid,
  'audit_event',
  'fabric-local',
  1,
  'aud013-root-1',
  'retry_requested',
  '11111111-1111-4111-8111-111111111111'::uuid,
  '{}'::jsonb
)
ON CONFLICT (anchor_batch_id) DO NOTHING;
```

3. 用 `kcat` 容器注入单条 canonical 事件：

```bash
cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.audit.anchor
{"event_id":"aud013-anchor-evt-kcat","event_type":"audit.anchor_requested","event_version":1,"occurred_at":"2026-04-22T04:45:00Z","producer_service":"platform-core.audit","aggregate_type":"audit.anchor_batch","aggregate_id":"22222222-2222-4222-8222-222222222222","request_id":"req-aud013-anchor-kcat","trace_id":"trace-aud013-anchor-kcat","idempotency_key":"idemp-aud013-anchor-kcat","event_schema_version":"v1","authority_scope":"audit_authority","source_of_truth":"postgresql","proof_commit_policy":"async_anchor","payload":{"anchor_batch_id":"22222222-2222-4222-8222-222222222222","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud013-root-1","anchor_status":"retry_requested"},"anchor_batch_id":"22222222-2222-4222-8222-222222222222","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud013-root-1","anchor_status":"retry_requested","chain_anchor_id":"11111111-1111-4111-8111-111111111111"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud013-proof-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T04:45:01Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"33333333-3333-4333-8333-333333333333","request_id":"req-aud013-proof-kcat","trace_id":"trace-aud013-proof-kcat","idempotency_key":"idemp-aud013-proof-kcat","event_schema_version":"v1","authority_scope":"dual_authority","source_of_truth":"postgresql","proof_commit_policy":"async_anchor","payload":{"chain_anchor_id":"33333333-3333-4333-8333-333333333333","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud013-proof-root"},"chain_anchor_id":"33333333-3333-4333-8333-333333333333","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud013-proof-root"}
JSON
```

4. 回查结果：

```sql
SELECT request_id, provider_type, provider_key, provider_reference, receipt_status,
       receipt_payload ->> 'mode' AS mode,
       receipt_payload ->> 'chain_id' AS chain_id,
       metadata ->> 'topic' AS topic
FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud013-anchor-kcat', 'req-aud013-proof-kcat')
ORDER BY request_id;

SELECT action_name, result_code, request_id, tx_hash
FROM audit.audit_event
WHERE request_id IN ('req-aud013-anchor-kcat', 'req-aud013-proof-kcat')
ORDER BY event_time;

SELECT message_text, request_id
FROM ops.system_log
WHERE request_id IN ('req-aud013-anchor-kcat', 'req-aud013-proof-kcat')
  AND message_text = 'fabric adapter accepted submit event'
ORDER BY created_at;

SELECT chain_anchor_id::text, status, tx_hash, reconcile_status
FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '11111111-1111-4111-8111-111111111111'::uuid,
  '33333333-3333-4333-8333-333333333333'::uuid
)
ORDER BY chain_anchor_id;

SELECT request_id, count(*)
FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud013-anchor-kcat', 'req-aud013-proof-kcat')
GROUP BY request_id
ORDER BY request_id;
```

预期：

- 每个 `request_id` 在 `ops.external_fact_receipt / audit.audit_event / ops.system_log` 中各出现一次
- `receipt_payload.mode = mock`
- topic 正确分别为 `dtp.audit.anchor / dtp.fabric.requests`
- `chain.chain_anchor.status = submitted`
- `chain.chain_anchor.reconcile_status = pending_check`

5. 清理测试业务数据：

```sql
DELETE FROM ops.external_fact_receipt
WHERE request_id IN (
  'req-aud013-anchor-kcat',
  'req-aud013-proof-kcat'
);

DELETE FROM audit.anchor_batch
WHERE anchor_batch_id = '22222222-2222-4222-8222-222222222222'::uuid;

DELETE FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '11111111-1111-4111-8111-111111111111'::uuid,
  '33333333-3333-4333-8333-333333333333'::uuid
);
```

`audit.audit_event` 与 `ops.system_log` 按 append-only 保留，不清理。

## 排障

- 若 `cg-fabric-adapter` 未创建，先确认进程是否真正启动：

```bash
docker exec datab-kafka /opt/kafka/bin/kafka-consumer-groups.sh \
  --bootstrap-server localhost:9092 \
  --group cg-fabric-adapter \
  --describe
```

- 若消息未被消费，先确认 topology 与 route seed：
  - `./scripts/check-topic-topology.sh`
  - `docs/04-runbooks/kafka-topics.md`
  - `docs/数据库设计/V1/upgrade/074_event_topology_route_extensions.sql`

- 当前手工注入建议优先用 `kcat` 容器，而不是 `kafka-console-producer.sh`，以避免人工排障时引入额外重复消息噪音。
