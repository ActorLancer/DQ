# Audit / Consistency 验收清单

当前文件承接 `AUD-003`、`AUD-004`、`AUD-005`、`AUD-006`、`AUD-007`、`AUD-008`、`AUD-009`、`AUD-010`、`AUD-011`、`AUD-012` 已落地的首版审计控制面验收矩阵，覆盖：

- 订单审计联查：`GET /api/v1/audit/orders/{id}`
- 全局审计 trace 查询：`GET /api/v1/audit/traces`
- 证据包导出：`POST /api/v1/audit/packages/export`
- 回放任务 dry-run：`POST /api/v1/audit/replay-jobs`
- 回放任务联查：`GET /api/v1/audit/replay-jobs/{id}`
- legal hold 创建 / 释放：`POST /api/v1/audit/legal-holds`、`POST /api/v1/audit/legal-holds/{id}/release`
- anchor batch 查看 / 重试：`GET /api/v1/audit/anchor-batches`、`POST /api/v1/audit/anchor-batches/{id}/retry`
- canonical outbox 查询：`GET /api/v1/ops/outbox`
- dead letter 查询：`GET /api/v1/ops/dead-letters`
- dead letter dry-run 重处理：`POST /api/v1/ops/dead-letters/{id}/reprocess`
- 一致性联查：`GET /api/v1/ops/consistency/{refType}/{refId}`
- 一致性修复 dry-run：`POST /api/v1/ops/consistency/reconcile`
- outbox publisher：`ops.outbox_event -> workers/outbox-publisher -> Kafka / ops.outbox_publish_attempt / ops.dead_letter_event`
- fabric adapter 基础框架：`dtp.audit.anchor / dtp.fabric.requests -> services/fabric-adapter -> ops.external_fact_receipt / audit.audit_event / ops.system_log / chain.chain_anchor`

后续 Fabric callback、reconcile 等高风险控制面进入对应 `AUD` task 后，再继续追加到本文件，不得另起旁路清单。

## 前置条件

- 已执行：`set -a; source infra/docker/.env.local; set +a`
- 本地基础设施可用：`PostgreSQL`、`MinIO`、`Kafka`、`Redis`、`Keycloak / IAM`、观测栈
- 平台服务可用：`platform-core`
- 如需一次性跑首版 live smoke：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core modules::audit::tests::api_db::audit_trace_api_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture

AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture
```

## 验收矩阵

| 用例ID | 场景 | 输入 / 操作 | 预期结果 | 主要回查点 |
| --- | --- | --- | --- | --- |
| `AUD-CASE-001` | 订单审计联查 | `GET /api/v1/audit/orders/{id}` | 返回订单最小审计视图，租户读场景只允许 buyer/seller org | API 响应、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-002` | 全局 trace 查询 | `GET /api/v1/audit/traces?order_id=...&trace_id=...` | 返回同一请求链的审计事件分页结果 | API 响应、`audit.audit_event`、`audit.access_audit` |
| `AUD-CASE-003` | 证据包导出 | `POST /api/v1/audit/packages/export` + `x-step-up-challenge-id` | 写入 `audit.evidence_package`、MinIO 导出对象、`audit.audit_event(action_name='audit.package.export')` | API 响应、`audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 对象、`ops.system_log` |
| `AUD-CASE-004` | replay 创建 | `POST /api/v1/audit/replay-jobs` with `dry_run=true` | 写入 `audit.replay_job + audit.replay_result`，MinIO replay report 落盘，并生成 `audit.replay.requested / completed` | API 响应、`audit.replay_job`、`audit.replay_result`、`audit.audit_event`、MinIO 对象 |
| `AUD-CASE-005` | replay 只允许 dry-run | `POST /api/v1/audit/replay-jobs` with `dry_run=false` | 返回 `409` 且错误码 `AUDIT_REPLAY_DRY_RUN_ONLY` | HTTP 响应、错误码 |
| `AUD-CASE-006` | replay 读取 | `GET /api/v1/audit/replay-jobs/{id}` | 返回 replay job + results；读取动作也必须落 `audit.access_audit` 与 `ops.system_log` | API 响应、`audit.access_audit(access_mode='replay')`、`ops.system_log` |
| `AUD-CASE-007` | 高风险动作鉴权 | 缺少权限或缺少 step-up 分别调用 export / replay | 返回 `403 / 400`，不得写业务副作用 | HTTP 响应、`audit.evidence_package / replay_job` 无新增 |
| `AUD-CASE-008` | legal hold 创建 | `POST /api/v1/audit/legal-holds` + `x-step-up-challenge-id` | 写入 `audit.legal_hold`，并产生 `audit.audit_event(action_name='audit.legal_hold.create')`；当前 hold 状态以 `audit.legal_hold` 为权威源，历史 evidence/package 行保持 append-only | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-009` | legal hold 重复创建冲突 | 对同一 active scope 再次创建 | 返回 `409` 且错误码 `AUDIT_LEGAL_HOLD_ACTIVE` | HTTP 响应、错误码、无新增 hold |
| `AUD-CASE-010` | legal hold 释放 | `POST /api/v1/audit/legal-holds/{id}/release` | `status=released`、`approved_by / released_at` 落库，并产生 `audit.audit_event(action_name='audit.legal_hold.release')`；当前 hold 状态回到 `audit.legal_hold` 权威视图中的 `none` | API 响应、`audit.legal_hold`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-011` | anchor batch 查看 | `GET /api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local` | 返回 `audit.anchor_batch + chain.chain_anchor` 联查视图，可看到 `anchor_batch_id / batch_scope / record_count / batch_root / chain_id / tx_hash / anchor_status / anchored_at` | API 响应、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-012` | failed batch retry | `POST /api/v1/audit/anchor-batches/{id}/retry` + verified `x-step-up-challenge-id` | 仅允许 `status=failed`；成功后 `audit.anchor_batch.status=retry_requested`，并写出 canonical outbox `audit.anchor_requested -> dtp.audit.anchor` | API 响应、`audit.anchor_batch`、`ops.outbox_event(target_topic='dtp.audit.anchor')`、`audit.audit_event(action_name='audit.anchor.retry')`、`audit.access_audit(access_mode='retry')`、`ops.system_log` |
| `AUD-CASE-013` | canonical outbox 查询 | `GET /api/v1/ops/outbox?target_topic=dtp.search.sync&event_type=search.product.changed` | 返回 `ops.outbox_event` 分页结果，并包含最新 `ops.outbox_publish_attempt`；查询动作写入 `audit.access_audit + ops.system_log` | API 响应、`ops.outbox_event`、`ops.outbox_publish_attempt`、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-014` | dead letter + SEARCHREC 幂等联查 | `GET /api/v1/ops/dead-letters?failure_stage=consumer_handler` | 返回 `ops.dead_letter_event` 分页结果，并挂出 `ops.consumer_idempotency_record`，可定位 `search-indexer` 或 `recommendation-aggregator` 的失败隔离记录 | API 响应、`ops.dead_letter_event`、`ops.consumer_idempotency_record`、`audit.access_audit(access_mode='masked')`、`ops.system_log` |
| `AUD-CASE-015` | outbox publisher 发布成功 | 写入一条 `ops.outbox_event(status=pending,target_topic=dtp.outbox.domain-events)`，运行 `cargo test -p outbox-publisher outbox_publisher_db_smoke -- --nocapture` | `workers/outbox-publisher` 把事件发到 Kafka，`ops.outbox_event.status=published`，并写入 `ops.outbox_publish_attempt(result_code='published')`、`audit.audit_event(action_name='outbox.publisher.publish')`、`ops.system_log(service_name='outbox-publisher')` | Kafka 消息、`ops.outbox_event`、`ops.outbox_publish_attempt`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-016` | outbox publisher 失败隔离 | 写入一条 `ops.outbox_event(status=pending,target_topic=dtp.missing.topic,max_retries=1)`，运行同一 smoke | 事件进入 `ops.dead_letter_event(failure_stage='outbox.publish')`，并向 Kafka `dtp.dead-letter` 发布隔离消息；原 outbox 行变为 `dead_lettered` | `ops.dead_letter_event`、Kafka `dtp.dead-letter`、`ops.outbox_publish_attempt(result_code='dead_lettered')`、`audit.audit_event`、`ops.system_log` |
| `AUD-CASE-017` | SEARCHREC dead letter dry-run 重处理 | `POST /api/v1/ops/dead-letters/{id}/reprocess` + verified `x-step-up-challenge-id` + `{"reason":"...","dry_run":true}` | 仅允许 `reprocess_status=not_reprocessed` 的 SEARCHREC consumer dead letter；返回 `dry_run_ready` 预演计划，不改变 `ops.dead_letter_event.reprocess_status`，并写入 `audit.audit_event(action_name='ops.dead_letter.reprocess.dry_run')`、`audit.access_audit(access_mode='reprocess')`、`ops.system_log` | API 响应、`ops.dead_letter_event`、`audit.audit_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-018` | 一致性联查 | `GET /api/v1/ops/consistency/order/{order_id}` | 返回业务状态、proof/anchor 状态、外部事实状态，以及最近 `ops.outbox_event / ops.dead_letter_event / audit.audit_event`；查询动作写入 `audit.access_audit(target_type='consistency_query')` 与 `ops.system_log` | API 响应、`trade.order_main`、`chain.chain_anchor`、`ops.chain_projection_gap`、`ops.external_fact_receipt`、`ops.outbox_event`、`ops.dead_letter_event`、`audit.access_audit`、`ops.system_log` |
| `AUD-CASE-019` | 一致性修复 dry-run | `POST /api/v1/ops/consistency/reconcile` + verified `x-step-up-challenge-id` + `{"ref_type":"order","ref_id":"...","mode":"full","dry_run":true,"reason":"..."}` | 不新增 `reconcile_job` 表、不改写 `ops.chain_projection_gap`，只返回修复建议并写入 `audit.audit_event(action_name='ops.consistency.reconcile.dry_run')`、`audit.access_audit(access_mode='reconcile', target_type='consistency_reconcile')`、`ops.system_log`；同时不得写出 `dtp.consistency.reconcile` 新 outbox 事件 | API 响应、`ops.chain_projection_gap` 仍为原状态、`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`ops.outbox_event(request_id=...)` |
| `AUD-CASE-020` | Fabric adapter request consume + receipt write-back | 启动 `./scripts/fabric-adapter-run.sh`，使用 `kcat` 向 `dtp.audit.anchor` 写入 `audit.anchor_requested`、向 `dtp.fabric.requests` 写入 `fabric.proof_submit_requested` | `services/fabric-adapter` 真实消费两条正式 topic，使用 Go mock provider 生成回执，并把提交结果写入 `ops.external_fact_receipt`、`audit.audit_event(action_name='fabric.adapter.submit')`、`ops.system_log(message_text='fabric adapter accepted submit event')`；若 payload 提供 `chain_anchor_id`，则 `chain.chain_anchor.status=submitted` 且 `reconcile_status=pending_check` | Kafka topic 内容、`ops.external_fact_receipt`、`audit.audit_event`、`ops.system_log`、`chain.chain_anchor`、`cg-fabric-adapter` consumer group |

补充说明：

- `AUD-008` 同步补齐 `ops.external_fact_receipt` 与 `ops.chain_projection_gap` 的仓储查询能力，但其公共 HTTP 控制面接口分别由后续交易链监控 / 一致性任务承接。
- `reconcile` 在 `V1` 中不是独立正式表；不要把 `ops.chain_projection_gap` 宣传成 `reconcile_job` 的同义词。
- `AUD-013` 只完成 `fabric-adapter` 基础框架与 mock provider 回执回写；`fabric-test-network / Gateway / chaincode / event-listener / CA admin` 留待 `AUD-014~AUD-017`。

## `AUD-011` 手工一致性联查验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 触发 live smoke 或自行准备一笔带双层权威字段的对象：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_lookup_db_smoke -- --nocapture
```

3. 查询一致性视图：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/consistency/order/<order_id>" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud011-manual' \
  -H 'x-trace-id: trace-aud011-manual'
```

4. 回查查询留痕：

```sql
SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id = 'req-aud011-manual';

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud011-manual'
  AND message_text = 'ops lookup executed: GET /api/v1/ops/consistency/{refType}/{refId}';
```

## `AUD-012` 手工一致性修复 dry-run 验证

1. 启动服务：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

2. 触发 live smoke 或自行准备一笔带 `ops.chain_projection_gap` 的对象：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core audit_consistency_reconcile_db_smoke -- --nocapture
```

3. 发起 dry-run 修复预演：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/consistency/reconcile" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud012-manual' \
  -H 'x-trace-id: trace-aud012-manual' \
  -H 'x-step-up-challenge-id: <reconcile_step_up_id>' \
  -d '{
    "ref_type": "order",
    "ref_id": "<order_id>",
    "mode": "full",
    "dry_run": true,
    "reason": "manual consistency reconcile preview"
  }'
```

4. 回查 dry-run 留痕与“无执行副作用”：

```sql
SELECT action_name, result_code, metadata ->> 'mode'
FROM audit.audit_event
WHERE request_id = 'req-aud012-manual'
  AND action_name = 'ops.consistency.reconcile.dry_run';

SELECT access_mode, target_type, target_id::text
FROM audit.access_audit
WHERE request_id = 'req-aud012-manual';

SELECT message_text, structured_payload
FROM ops.system_log
WHERE request_id = 'req-aud012-manual'
  AND message_text = 'ops consistency reconcile prepared: POST /api/v1/ops/consistency/reconcile';

SELECT gap_status, resolution_summary
FROM ops.chain_projection_gap
WHERE aggregate_type = 'order'
  AND aggregate_id = '<order_id>'::uuid
ORDER BY created_at DESC, chain_projection_gap_id DESC
LIMIT 5;

SELECT count(*) AS reconcile_preview_outbox_count
FROM ops.outbox_event
WHERE request_id = 'req-aud012-manual'
  AND target_topic = 'dtp.consistency.reconcile';
```

5. 预期：

- 返回 `dry_run_ready`
- `recommendations` 非空，且推荐目标 topic 为 `dtp.consistency.reconcile`
- `ops.chain_projection_gap` 仍保持原 `gap_status / resolution_summary`
- 当前请求不会写出新的 `dtp.consistency.reconcile` outbox 事件
- `audit.audit_event + audit.access_audit + ops.system_log` 三层留痕齐备

## `AUD-013` 手工 fabric-adapter 验证

1. 启动适配器：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-adapter-run.sh
```

2. 准备最小 `chain_anchor + anchor_batch` 测试对象：

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

3. 用 `kcat` 注入单条 canonical 事件：

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

5. 清理：

```sql
DELETE FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud013-anchor-kcat', 'req-aud013-proof-kcat');

DELETE FROM audit.anchor_batch
WHERE anchor_batch_id = '22222222-2222-4222-8222-222222222222'::uuid;

DELETE FROM chain.chain_anchor
WHERE chain_anchor_id IN (
  '11111111-1111-4111-8111-111111111111'::uuid,
  '33333333-3333-4333-8333-333333333333'::uuid
);
```

`audit.audit_event` 与 `ops.system_log` 保留作为 append-only 留痕。

## `AUD-005` 手工回放验证

1. 启动服务：

```bash
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
APP_PORT=18080 \
cargo run -p platform-core-bin
```

2. 准备最小业务对象与 step-up challenge。下面 SQL 使用 `gen_random_uuid()` 生成一组独立数据，返回：
   - `order_id`
   - `audit_user_id`
   - `replay_challenge_id`

```sql
WITH buyer AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-buyer-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), seller AS (
  INSERT INTO core.organization (org_name, org_type, status, metadata)
  VALUES ('aud005-seller-manual', 'enterprise', 'active', '{}'::jsonb)
  RETURNING org_id
), asset AS (
  INSERT INTO catalog.asset (owner_org_id, asset_name, asset_type, lifecycle_status, metadata)
  SELECT seller.org_id, 'aud005-asset-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING asset_id
), asset_version AS (
  INSERT INTO catalog.asset_version (asset_id, version_no, version_label, schema_json, metadata)
  SELECT asset.asset_id, 1, 'v1', '{}'::jsonb, '{}'::jsonb
  FROM asset
  RETURNING asset_version_id
), product AS (
  INSERT INTO catalog.product (owner_org_id, product_name, product_type, status, metadata)
  SELECT seller.org_id, 'aud005-product-manual', 'dataset', 'published', '{}'::jsonb
  FROM seller
  RETURNING product_id
), sku AS (
  INSERT INTO catalog.sku (
    product_id,
    asset_version_id,
    sku_code,
    sku_type,
    billing_mode,
    price_json,
    entitlement_json,
    status,
    metadata
  )
  SELECT product.product_id, asset_version.asset_version_id, 'AUD005-MANUAL', 'DATA', 'ONE_TIME',
         '{"amount":"88.00","currency":"CNY"}'::jsonb, '{}'::jsonb, 'active', '{}'::jsonb
  FROM product, asset_version
  RETURNING sku_id
), order_main AS (
  INSERT INTO trade.order_main (
    buyer_org_id,
    seller_org_id,
    product_id,
    sku_id,
    order_no,
    status,
    payment_status,
    delivery_status,
    acceptance_status,
    settlement_status,
    dispute_status,
    total_amount,
    currency,
    price_snapshot_json,
    metadata
  )
  SELECT buyer.org_id, seller.org_id, product.product_id, sku.sku_id, 'AUD005-MANUAL',
         'created', 'pending', 'pending', 'pending', 'pending', 'none',
         88.00, 'CNY', '{}'::jsonb, '{}'::jsonb
  FROM buyer, seller, product, sku
  RETURNING order_id
), audit_user AS (
  INSERT INTO core.user_account (
    org_id, login_id, display_name, user_type, status, mfa_status, email, attrs
  )
  SELECT buyer.org_id, 'aud005-manual-user', 'AUD005 Manual User',
         'human', 'active', 'verified', 'aud005-manual@example.com', '{}'::jsonb
  FROM buyer
  RETURNING user_id
), audit_seed AS (
  INSERT INTO audit.audit_event (
    event_schema_version, event_class, domain_name, ref_type, ref_id,
    actor_type, actor_id, actor_org_id, tenant_id, action_name, result_code,
    request_id, trace_id, event_time, sensitivity_level, metadata
  )
  SELECT 'v1', 'business', 'trade', 'order', order_main.order_id,
         'user', audit_user.user_id, buyer.org_id, buyer.org_id::text,
         'trade.order.create', 'accepted',
         'req-aud005-manual-seed', 'trace-aud005-manual-seed', now(), 'normal', '{}'::jsonb
  FROM order_main, audit_user, buyer
  RETURNING ref_id
), replay_challenge AS (
  INSERT INTO iam.step_up_challenge (
    user_id,
    challenge_type,
    target_action,
    target_ref_type,
    target_ref_id,
    challenge_status,
    expires_at,
    completed_at,
    metadata
  )
  SELECT audit_user.user_id,
         'mock_otp',
         'audit.replay.execute',
         'order',
         order_main.order_id,
         'verified',
         now() + interval '10 minutes',
         now(),
         jsonb_build_object('seed', 'aud005-manual')
  FROM audit_user, order_main
  RETURNING step_up_challenge_id
)
SELECT
  (SELECT order_id::text FROM order_main) AS order_id,
  (SELECT user_id::text FROM audit_user) AS audit_user_id,
  (SELECT step_up_challenge_id::text FROM replay_challenge) AS replay_challenge_id;
```

3. 创建 replay job：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/replay-jobs \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-create' \
  -H 'x-trace-id: trace-aud005-manual-create' \
  -H 'x-step-up-challenge-id: <replay_challenge_id>' \
  -d '{
    "replay_type": "state_replay",
    "ref_type": "order",
    "ref_id": "<order_id>",
    "reason": "manual replay verification",
    "dry_run": true,
    "options": {
      "trigger": "manual_http_check"
    }
  }'
```

4. 读取 replay job：

```bash
curl -sS http://127.0.0.1:18080/api/v1/audit/replay-jobs/<replay_job_id> \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud005-manual-get' \
  -H 'x-trace-id: trace-aud005-manual-get'
```

## 回查清单

1. 回查 replay 任务与结果：

```sql
SELECT replay_type, ref_type, ref_id::text, dry_run, status, request_reason
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;

SELECT step_name, result_code
FROM audit.replay_result
WHERE replay_job_id = '<replay_job_id>'::uuid
ORDER BY created_at, replay_result_id;
```

预期：

- `dry_run=true`
- `status=completed`
- 存在 4 条结果：`target_snapshot`、`audit_timeline`、`evidence_projection`、`execution_policy`
- `execution_policy.result_code='AUDIT_REPLAY_DRY_RUN_ONLY'`

2. 回查正式审计与访问留痕：

```sql
SELECT action_name, result_code
FROM audit.audit_event
WHERE ref_type = 'replay_job'
  AND ref_id = '<replay_job_id>'::uuid
ORDER BY event_time, audit_id;

SELECT access_mode, request_id
FROM audit.access_audit
WHERE target_type = 'replay_job'
  AND target_id = '<replay_job_id>'::uuid
ORDER BY created_at, access_audit_id;

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud005-manual-create', 'req-aud005-manual-get')
ORDER BY created_at, system_log_id;
```

预期：

- `audit.audit_event` 至少存在 `audit.replay.requested`、`audit.replay.completed`
- `audit.access_audit.access_mode='replay'`
- `ops.system_log` 包含：
  - `audit replay job executed: POST /api/v1/audit/replay-jobs`
  - `audit replay lookup executed: GET /api/v1/audit/replay-jobs/{id}`

3. 回查 MinIO replay report：

```sql
SELECT options_json ->> 'report_storage_uri'
FROM audit.replay_job
WHERE replay_job_id = '<replay_job_id>'::uuid;
```

预期：

- `report_storage_uri` 指向 `s3://evidence-packages/replays/<ref_type>/<ref_id>/replay-<job_id>.json`
- 对象可正常读取
- 内容包含：
  - `recommendation=dry_run_completed`
  - `results[*].step_name`
  - `step_up.challenge_id`
  - `target.order_id=<order_id>`

## `AUD-006` 手工 legal hold 验证

1. 准备 step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'order',
  '<order_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-create')
)
RETURNING step_up_challenge_id::text;
```

2. 创建 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-create' \
  -H 'x-trace-id: trace-aud006-manual-create' \
  -H 'x-step-up-challenge-id: <create_challenge_id>' \
  -d '{
    "hold_scope_type": "order",
    "hold_scope_id": "<order_id>",
    "reason_code": "regulator_investigation",
    "metadata": {
      "ticket": "AUD-OPS-006"
    }
  }'
```

3. 再次创建同一 scope，确认冲突：

预期：

- 返回 `409`
- 错误码为 `AUDIT_LEGAL_HOLD_ACTIVE`

4. 为释放动作准备新的 challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.legal_hold.manage',
  'legal_hold',
  '<legal_hold_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud006-manual-release')
)
RETURNING step_up_challenge_id::text;
```

5. 释放 legal hold：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/legal-holds/<legal_hold_id>/release \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud006-manual-release' \
  -H 'x-trace-id: trace-aud006-manual-release' \
  -H 'x-step-up-challenge-id: <release_challenge_id>' \
  -d '{
    "reason": "manual review cleared hold"
  }'
```

6. 回查主记录与 scope 状态：

```sql
SELECT status, requested_by::text, approved_by::text, released_at, metadata ->> 'release_reason'
FROM audit.legal_hold
WHERE legal_hold_id = '<legal_hold_id>'::uuid;

SELECT COUNT(*)::bigint
FROM audit.legal_hold
WHERE hold_scope_type = 'order'
  AND hold_scope_id = '<order_id>'::uuid
  AND status = 'active';
```

预期：

- 创建后 `status=active`
- 释放后 `status=released`
- `approved_by=<audit_user_id>`
- `metadata.release_reason='manual review cleared hold'`
- 若上述活跃 hold 计数在释放后重新查询，应返回 `0`
- 历史 `audit.evidence_item / audit.evidence_package` 行保持 append-only，不作为当前 hold 状态权威源

## `AUD-007` 手工锚定批次验证

1. 查询 failed anchor batches：

```bash
curl -sS 'http://127.0.0.1:18080/api/v1/audit/anchor-batches?anchor_status=failed&batch_scope=audit_event&chain_id=fabric-local' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-list' \
  -H 'x-trace-id: trace-aud007-manual-list'
```

预期：

- 至少返回 1 条 failed batch
- `items[0]` 中可见 `tx_hash / anchor_status / chain_id`

2. 为 retry 准备 step-up challenge：

```sql
INSERT INTO iam.step_up_challenge (
  user_id,
  challenge_type,
  target_action,
  target_ref_type,
  target_ref_id,
  challenge_status,
  expires_at,
  completed_at,
  metadata
) VALUES (
  '<audit_user_id>'::uuid,
  'mock_otp',
  'audit.anchor.manage',
  'anchor_batch',
  '<anchor_batch_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('seed', 'aud007-manual-retry')
)
RETURNING step_up_challenge_id::text;
```

3. 触发 retry：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/audit/anchor-batches/<anchor_batch_id>/retry \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <audit_user_id>' \
  -H 'x-request-id: req-aud007-manual-retry' \
  -H 'x-trace-id: trace-aud007-manual-retry' \
  -H 'x-step-up-challenge-id: <retry_challenge_id>' \
  -d '{
    "reason": "retry failed batch after gateway timeout",
    "metadata": {
      "ticket_id": "AUD-OPS-007"
    }
  }'
```

4. 回查 batch 与 outbox：

```sql
SELECT
  status,
  metadata ->> 'previous_status',
  metadata -> 'retry_request' ->> 'reason'
FROM audit.anchor_batch
WHERE anchor_batch_id = '<anchor_batch_id>'::uuid;

SELECT
  target_topic,
  event_type,
  aggregate_type,
  payload ->> 'anchor_status',
  payload ->> 'previous_anchor_status'
FROM ops.outbox_event
WHERE request_id = 'req-aud007-manual-retry'
ORDER BY created_at DESC, outbox_event_id DESC
LIMIT 1;
```

预期：

- `audit.anchor_batch.status='retry_requested'`
- `metadata.previous_status='failed'`
- `ops.outbox_event.target_topic='dtp.audit.anchor'`
- `event_type='audit.anchor_requested'`
- `payload.anchor_status='retry_requested'`

5. 回查审计与系统日志：

```sql
SELECT COUNT(*)::bigint
FROM audit.audit_event
WHERE request_id = 'req-aud007-manual-retry'
  AND action_name = 'audit.anchor.retry';

SELECT COUNT(*)::bigint
FROM audit.access_audit
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry');

SELECT message_text
FROM ops.system_log
WHERE request_id IN ('req-aud007-manual-list', 'req-aud007-manual-retry')
ORDER BY created_at;
```

预期：

- `audit.audit_event` 至少 1 条
- `audit.access_audit` 覆盖 list + retry
- `ops.system_log` 含 list 与 retry 两条记录

## 清理约束

- 业务测试数据可清理：`trade.order_main` 及本手工步骤创建的临时 `core.organization / catalog.*` scope 图数据
- 与高风险动作审计链绑定的 `core.user_account / iam.step_up_challenge` 在当前运行态不做强删；删除它们会触发 FK 尝试回写 append-only `audit.audit_event`
- 审计数据不清理：`audit.audit_event`、`audit.access_audit`、`ops.system_log`、`audit.replay_job`、`audit.replay_result`、`audit.legal_hold`、`audit.anchor_batch`、MinIO replay report 及相关 evidence snapshot 按 append-only 或审计保留规则保留
- `ops.outbox_event` 若为本次手工 retry 临时产生的待发布记录，只允许按唯一 `request_id` 精确删除；不得宽泛清库

## 当前未覆盖项

- `AUD-008+` Fabric request / callback / reconcile
- `AUD-011+` consistency repair / OpenSearch ops

进入对应批次后，必须在本文件继续追加，不得把本文件视为 `AUD` 全阶段完成证明。
