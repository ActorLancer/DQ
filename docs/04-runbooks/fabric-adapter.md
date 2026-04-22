# Fabric Adapter（AUD-013 / AUD-014 / AUD-017）

`AUD-013` 起，`services/fabric-adapter/` 作为正式 Go 进程落地，负责消费：

- `dtp.audit.anchor`
- `dtp.fabric.requests`

并将提交回执写回：

- `ops.external_fact_receipt`
- `audit.audit_event`
- `ops.system_log`
- `chain.chain_anchor`（存在 `chain_anchor_id` 时更新 `tx_hash / status / reconcile_status`）

当前批次边界：

- `AUD-013` 已落地 Go module、Kafka consumer、canonical envelope 解析、mock provider、PostgreSQL 回执写回
- `AUD-014` 已在 Go 侧补齐四类正式消息处理占位：`evidence_batch_root / order_summary / authorization_summary / acceptance_summary`
- `AUD-017` 起 provider 正式支持：
  - `mock`
  - `fabric-test-network`
- `fabric-test-network` 模式下，adapter 通过 Go SDK 直连 Fabric Gateway，并显式等待 `commit status`
- 当前不消费 `dtp.outbox.domain-events`

## 命令入口

```bash
make fabric-adapter-bootstrap
make fabric-adapter-test
make fabric-adapter-run
./scripts/fabric-adapter-live-smoke.sh
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
- `REDIS_URL`
- `REDIS_NAMESPACE`
- `KAFKA_BROKERS` 或 `KAFKA_BOOTSTRAP_SERVERS`
- `TOPIC_AUDIT_ANCHOR`
- `TOPIC_FABRIC_REQUESTS`
- `FABRIC_ADAPTER_CONSUMER_GROUP`
- `FABRIC_ADAPTER_CONSUMER_LOCK_TTL`
- `FABRIC_ADAPTER_PROVIDER_MODE`
- `FABRIC_CHANNEL_NAME`
- `FABRIC_CHAINCODE_NAME`
- `FABRIC_GATEWAY_ENDPOINT`
- `FABRIC_GATEWAY_PEER`
- `FABRIC_MSP_ID`
- `FABRIC_TLS_CERT_PATH`
- `FABRIC_SIGN_CERT_PATH`
- `FABRIC_PRIVATE_KEY_DIR / FABRIC_PRIVATE_KEY_PATH`

当前本地默认值：

- Kafka：`127.0.0.1:9094`
- PostgreSQL：`postgres://datab:datab_local_pass@127.0.0.1:5432/datab`
- Redis：`redis://:datab_redis_pass@127.0.0.1:6379/4`
- Redis namespace：`datab:v1`
- consumer group：`cg-fabric-adapter`
- consumer lock TTL：`15s`
- provider mode：`mock`

如需切到真实链：

```bash
FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network ./scripts/fabric-adapter-run.sh
```

`scripts/fabric-adapter-run.sh` 现在会保留外部传入的 `FABRIC_ADAPTER_PROVIDER_MODE / TOPIC_* / DATABASE_URL / REDIS_URL / REDIS_NAMESPACE / KAFKA_BROKERS / Gateway` 覆盖，不再被 `.env.local` 反向覆盖。

## 重复投递隔离（AUD-026 TODO 收口）

`fabric-adapter` 现在对每条 canonical event 同时启用两层门禁：

- `Redis` 短锁：`datab:v1:fabric-adapter:consumer-lock:{event_id}`
- `PostgreSQL` 幂等记录：`ops.consumer_idempotency_record(consumer_name='fabric-adapter', event_id=...)`

正式结果口径：

- 首次成功处理：`result_code='processed'`
- 已成功 / 已隔离的重复投递：直接跳过，不再重复提交 Fabric，也不再重复写 `ops.external_fact_receipt`
- 正在处理中的并发重复投递：命中 Redis 短锁后直接跳过，等待首个处理完成或重试
- 提交 / 回写失败：`result_code='failed'`，Kafka offset 不提交，待后续重新投递

## 重复投递 live smoke

仓库内最小重复投递隔离 smoke：

```bash
set -a
source infra/docker/.env.local
set +a

cd services/fabric-adapter

FABRIC_ADAPTER_RELIABILITY_SMOKE=1 \
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
REDIS_URL=redis://:datab_redis_pass@127.0.0.1:6379/4 \
go test ./internal/service -run TestProcessorReliabilityLiveSmoke -count=1 -v
```

该 smoke 会：

1. 使用真实 PostgreSQL 插入最小 `chain.chain_anchor + audit.anchor_batch`
2. 使用真实 Redis 创建 `fabric-adapter` 短锁
3. 用同一 `event_id` 并发调用两次 `ProcessMessage`
4. 验证第二次调用不会重复触发 provider / receipt write-back
5. 回查：
   - `ops.consumer_idempotency_record`
   - `ops.external_fact_receipt`
   - Redis 短锁存在与释放

预期：

- provider 只被调用 `1` 次
- `ops.external_fact_receipt` 只新增 `1` 条
- `ops.consumer_idempotency_record.result_code='processed'`
- `metadata.lock_backend='redis'`
- `metadata.lock_key` 命中 `datab:v1:fabric-adapter:consumer-lock:{event_id}`
- 处理完成后 Redis 短锁被释放

## 消息处理占位（AUD-014）

当前 `fabric-adapter` 仍保持正式单入口：

- `audit.anchor_requested -> dtp.audit.anchor`
- `fabric.proof_submit_requested -> dtp.fabric.requests`

但在 Go 进程内部已显式拆成四类 handler，占位契约如下：

| `submission_kind` | 来源事件 | 目标链码占位名 | 目标交易占位名 |
| --- | --- | --- | --- |
| `evidence_batch_root` | `audit.anchor_requested` | `evidence_batch_root` | `SubmitEvidenceBatchRoot` |
| `order_summary` | `fabric.proof_submit_requested` + `summary_type=order_summary` | `order_digest` | `SubmitOrderDigest` |
| `authorization_summary` | `fabric.proof_submit_requested` + `summary_type=authorization_summary` | `authorization_digest` | `SubmitAuthorizationDigest` |
| `acceptance_summary` | `fabric.proof_submit_requested` + `summary_type=acceptance_summary` | `acceptance_digest` | `SubmitAcceptanceDigest` |

如果 `summary_type` 缺失或不是上述三种之一，Go handler 会直接拒绝消费，不会伪造默认摘要类型。

## 真实 Smoke

优先使用仓库内脚本：

```bash
./scripts/fabric-adapter-live-smoke.sh
```

该 smoke 会：

1. 生成一组新的 `anchor_batch_id / chain_anchor_id / request_id / trace_id`
2. 向 PostgreSQL 写入最小 `chain.chain_anchor + audit.anchor_batch`
3. 通过真实 `fabric-test-network` provider 提交 `SubmitEvidenceBatchRoot`
4. 等待 `commit status`
5. 回查：
   - `ops.external_fact_receipt`
   - `audit.audit_event`
   - `ops.system_log`
   - `chain.chain_anchor`
   - `./infra/fabric/query-anchor.sh`

预期口径：

- `receipt_status = committed`
- `gateway_status = committed`
- `commit_status = VALID`
- `chain.chain_anchor.status = submitted`
- `chain.chain_anchor.tx_hash = provider_reference`

## 旧手工 Smoke（历史 backlog 验证）

1. 启动适配器：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-adapter-run.sh
```

2. 准备最小测试对象。

建议使用 `psql -v ON_ERROR_STOP=1`，避免 SQL 失败后 shell 仍继续向 Kafka 注入消息，污染 append-only 审计留痕：

```sql
INSERT INTO chain.chain_anchor (chain_anchor_id, chain_id, anchor_type, ref_type, ref_id, digest, status)
VALUES
  ('77777777-7777-4777-8777-777777777777'::uuid, 'fabric-local', 'audit_anchor_batch', 'anchor_batch', '66666666-6666-4666-8666-666666666666'::uuid, 'aud014b-root-evidence', 'pending'),
  ('88888888-8888-4888-8888-888888888888'::uuid, 'fabric-local', 'order_summary', 'chain_anchor', NULL, 'aud014b-root-order', 'pending'),
  ('99999999-9999-4999-8999-999999999999'::uuid, 'fabric-local', 'authorization_summary', 'chain_anchor', NULL, 'aud014b-root-auth', 'pending'),
  ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid, 'fabric-local', 'acceptance_summary', 'chain_anchor', NULL, 'aud014b-root-accept', 'pending');

INSERT INTO audit.anchor_batch (
  anchor_batch_id, batch_scope, chain_id, record_count, batch_root, status, chain_anchor_id, metadata
) VALUES (
  '66666666-6666-4666-8666-666666666666'::uuid,
  'audit_event',
  'fabric-local',
  1,
  'aud014b-root-evidence',
  'retry_requested',
  '77777777-7777-4777-8777-777777777777'::uuid,
  '{}'::jsonb
);
```

3. 用 `kcat` 容器注入四类 canonical 事件：

```bash
cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.audit.anchor
{"event_id":"aud014b-anchor-evt-kcat","event_type":"audit.anchor_requested","event_version":1,"occurred_at":"2026-04-22T05:02:00Z","producer_service":"platform-core.audit","aggregate_type":"audit.anchor_batch","aggregate_id":"66666666-6666-4666-8666-666666666666","request_id":"req-aud014b-anchor-kcat","trace_id":"trace-aud014b-anchor-kcat","idempotency_key":"idemp-aud014b-anchor-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"anchor_batch_id":"66666666-6666-4666-8666-666666666666","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud014b-root-evidence","anchor_status":"retry_requested"},"anchor_batch_id":"66666666-6666-4666-8666-666666666666","batch_scope":"audit_event","chain_id":"fabric-local","record_count":1,"batch_root":"aud014b-root-evidence","anchor_status":"retry_requested","chain_anchor_id":"77777777-7777-4777-8777-777777777777"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-order-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:01Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"88888888-8888-4888-8888-888888888888","request_id":"req-aud014b-order-kcat","trace_id":"trace-aud014b-order-kcat","idempotency_key":"idemp-aud014b-order-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"88888888-8888-4888-8888-888888888888","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud014b-root-order"},"chain_anchor_id":"88888888-8888-4888-8888-888888888888","chain_id":"fabric-local","summary_type":"order_summary","summary_digest":"aud014b-root-order"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-auth-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:02Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"99999999-9999-4999-8999-999999999999","request_id":"req-aud014b-auth-kcat","trace_id":"trace-aud014b-auth-kcat","idempotency_key":"idemp-aud014b-auth-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"99999999-9999-4999-8999-999999999999","chain_id":"fabric-local","summary_type":"authorization_summary","summary_digest":"aud014b-root-auth"},"chain_anchor_id":"99999999-9999-4999-8999-999999999999","chain_id":"fabric-local","summary_type":"authorization_summary","summary_digest":"aud014b-root-auth"}
JSON

cat <<'JSON' | docker run --rm -i --network container:datab-kafka edenhill/kcat:1.7.1 -P -b localhost:9092 -t dtp.fabric.requests
{"event_id":"aud014b-accept-evt-kcat","event_type":"fabric.proof_submit_requested","event_version":1,"occurred_at":"2026-04-22T05:02:03Z","producer_service":"platform-core.integration","aggregate_type":"chain.chain_anchor","aggregate_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","request_id":"req-aud014b-accept-kcat","trace_id":"trace-aud014b-accept-kcat","idempotency_key":"idemp-aud014b-accept-kcat","event_schema_version":"v1","authority_scope":"governance","source_of_truth":"postgresql","proof_commit_policy":"async_evidence","payload":{"chain_anchor_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","chain_id":"fabric-local","summary_type":"acceptance_summary","summary_digest":"aud014b-root-accept"},"chain_anchor_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","chain_id":"fabric-local","summary_type":"acceptance_summary","summary_digest":"aud014b-root-accept"}
JSON
```

4. 回查结果：

```sql
SELECT request_id,
       metadata ->> 'submission_kind' AS submission_kind,
       metadata ->> 'contract_name' AS contract_name,
       metadata ->> 'summary_digest' AS summary_digest,
       metadata ->> 'topic' AS topic,
       receipt_payload ->> 'transaction_name' AS transaction_name,
       receipt_payload ->> 'summary_type' AS receipt_summary_type,
       receipt_status
FROM ops.external_fact_receipt
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
ORDER BY request_id;

SELECT request_id, count(*)
FROM audit.audit_event
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
  AND action_name = 'fabric.adapter.submit'
GROUP BY request_id
ORDER BY request_id;

SELECT request_id, count(*)
FROM ops.system_log
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
)
  AND message_text = 'fabric adapter accepted submit event'
GROUP BY request_id
ORDER BY request_id;

SELECT chain_anchor_id::text, anchor_type, status, tx_hash, reconcile_status
FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '77777777-7777-4777-8777-777777777777'::uuid,
  '88888888-8888-4888-8888-888888888888'::uuid,
  '99999999-9999-4999-8999-999999999999'::uuid,
  'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid
)
ORDER BY chain_anchor_id;
```

预期：

- 四个 `request_id` 在 `ops.external_fact_receipt / audit.audit_event / ops.system_log` 中都只出现一次
- `submission_kind / contract_name / transaction_name` 分别为：
  - `evidence_batch_root / evidence_batch_root / SubmitEvidenceBatchRoot`
  - `order_summary / order_digest / SubmitOrderDigest`
  - `authorization_summary / authorization_digest / SubmitAuthorizationDigest`
  - `acceptance_summary / acceptance_digest / SubmitAcceptanceDigest`
- 四条 `chain.chain_anchor` 都被更新为 `status='submitted'`、`reconcile_status='pending_check'`

如果同一条 canonical event 被人工重复注入，额外回查：

```sql
SELECT consumer_name,
       event_id::text,
       result_code,
       metadata ->> 'lock_backend' AS lock_backend,
       metadata ->> 'lock_key' AS lock_key,
       metadata ->> 'provider_reference' AS provider_reference
FROM ops.consumer_idempotency_record
WHERE consumer_name = 'fabric-adapter'
  AND event_id = '<event_id>'::uuid;
```

预期：

- 同一 `event_id` 只保留 `1` 条 `ops.consumer_idempotency_record`
- `lock_backend='redis'`
- `lock_key` 命中 `datab:v1:fabric-adapter:consumer-lock:<event_id>`
- 若首条处理成功，`result_code='processed'`

5. 清理测试业务数据：

```sql
DELETE FROM ops.external_fact_receipt
WHERE request_id IN (
  'req-aud014b-anchor-kcat',
  'req-aud014b-order-kcat',
  'req-aud014b-auth-kcat',
  'req-aud014b-accept-kcat'
);

DELETE FROM audit.anchor_batch
WHERE anchor_batch_id = '66666666-6666-4666-8666-666666666666'::uuid;

DELETE FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '77777777-7777-4777-8777-777777777777'::uuid,
  '88888888-8888-4888-8888-888888888888'::uuid,
  '99999999-9999-4999-8999-999999999999'::uuid,
  'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::uuid
);
```

`audit.audit_event` 与 `ops.system_log` 按 append-only 保留，不清理。

## 排障

- 若 `psql` seed 出错，必须带 `-v ON_ERROR_STOP=1` 重跑，不要在 SQL 失败后继续向 Kafka 发消息，否则同一 `request_id` 的 append-only 审计行会被污染。

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
