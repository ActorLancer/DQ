# Search Reindex（AUD-022）

`AUD-022` 已把搜索运维控制面收口到正式运行态口径：

- `Authorization: Bearer <access_token>` 是唯一正式鉴权入口，不再接受 `x-role`
- `POST /api/v1/ops/search/reindex`
- `POST /api/v1/ops/search/aliases/switch`
- `POST /api/v1/ops/search/cache/invalidate`
- `GET /api/v1/ops/search/sync`
- `GET /api/v1/ops/search/ranking-profiles`
- `PATCH /api/v1/ops/search/ranking-profiles/{id}`

其中：

- 所有写接口都要求 `X-Idempotency-Key`
- `reindex / aliases/switch / ranking-profiles/{id}` 还要求 `X-Step-Up-Token`
- `ops.search_reindex.execute / ops.search_alias.manage / ops.search_ranking.manage` 当前只开放 `platform_admin`
- `ops.search_sync.read / ops.search_cache.invalidate` 当前开放 `platform_admin + platform_audit_security`
- V1 当前 `X-Step-Up-Token` 承载已验证 `iam.step_up_challenge.step_up_challenge_id`
- 服务端会真实回查 `iam.step_up_challenge.user_id / challenge_status / target_action / target_ref_type / target_ref_id`
- 搜索运维控制面统一使用搜索域错误码：参数/step-up 绑定错误返回 `SEARCH_QUERY_INVALID`，冲突返回 `SEARCH_RESULT_STALE`，下游依赖不可用返回 `SEARCH_BACKEND_UNAVAILABLE`，权限拒绝返回 `SEARCH_REINDEX_FORBIDDEN / SEARCH_ALIAS_SWITCH_FORBIDDEN / SEARCH_CACHE_INVALIDATE_FORBIDDEN`
- 成功写操作会真实写入 `audit.audit_event + audit.access_audit + ops.system_log`
- 读操作会真实写入 `audit.access_audit + ops.system_log`

## 本地前置

以下运维链路对应 `staging` 正式搜索路径；`local / demo` 的 PostgreSQL fallback 只用于前台搜索读路径验证，不替代本 runbook 中的 `OpenSearch` 运维能力。

载入本地环境：

```bash
set -a
source infra/docker/.env.local
set +a
```

初始化 OpenSearch：

```bash
./infra/opensearch/init-opensearch.sh
```

启动 `platform-core`：

```bash
cargo build -p platform-core-bin
APP_MODE=staging APP_PORT=18080 target/debug/platform-core-bin
```

启动 `search-indexer`：

```bash
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
```

## 手工 smoke

1. 准备一个具备 `platform_admin` 角色的 bearer token，对应 `core.user_account.user_id=<operator_user_id>`。
   本地默认可直接使用 `local-platform-admin / LocalPlatformAdmin123!`：

```bash
ACCESS_TOKEN="$(
  curl -sS -X POST \
    'http://127.0.0.1:8081/realms/platform-local/protocol/openid-connect/token' \
    -H 'content-type: application/x-www-form-urlencoded' \
    --data-urlencode 'grant_type=password' \
    --data-urlencode 'client_id=portal-web' \
    --data-urlencode 'username=local-platform-admin' \
    --data-urlencode 'password=LocalPlatformAdmin123!' \
  | jq -r '.access_token'
)"
```

2. 准备一个已验证 step-up challenge。示例：

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
  '<operator_user_id>'::uuid,
  'mock_otp',
  'ops.search_reindex.execute',
  'product',
  '<product_id>'::uuid,
  'verified',
  now() + interval '10 minutes',
  now(),
  jsonb_build_object('source', 'search-reindex-runbook')
)
RETURNING step_up_challenge_id::text;
```

3. 单实体重建：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/ops/search/reindex \
  -H 'content-type: application/json' \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H 'X-Request-Id: req-search-reindex-local' \
  -H 'X-Idempotency-Key: idem-search-reindex-local' \
  -H 'X-Step-Up-Token: <verified_step_up_challenge_id>' \
  -d '{
    "entity_scope":"product",
    "entity_id":"<product_uuid>",
    "mode":"single",
    "force":true
  }'
```

4. 查看同步任务：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/search/sync?entity_scope=product&sync_status=queued&limit=20" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H 'X-Request-Id: req-search-sync-local'
```

重点回查字段：

- `active_index_name`
- `reconcile_status`
- `open_exception_count`
- `latest_exception_error_code`
- `projection_document_version`
- `projection_index_sync_status`

5. 先对同一搜索条件执行两次目录搜索，确认第一次 `cache_hit=false`、第二次 `cache_hit=true`：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/catalog/search?q=<unique_keyword>&entity_scope=product&page=1&page_size=10" \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-read-local-1'
```

```bash
curl -sS "http://127.0.0.1:18080/api/v1/catalog/search?q=<unique_keyword>&entity_scope=product&page=1&page_size=10" \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-read-local-2'
```

6. 失效 Redis 搜索缓存：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/ops/search/cache/invalidate \
  -H 'content-type: application/json' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-cache-local' \
  -H 'X-Idempotency-Key: idem-search-cache-local' \
  -d '{
    "entity_scope":"product"
  }'
```

7. 列出排序配置：

```bash
curl -sS http://127.0.0.1:18080/api/v1/ops/search/ranking-profiles \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-ranking-list-local'
```

8. 更新排序配置：

```bash
curl -sS -X PATCH http://127.0.0.1:18080/api/v1/ops/search/ranking-profiles/<ranking_profile_id> \
  -H 'content-type: application/json' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-ranking-patch-local' \
  -H 'X-Idempotency-Key: idem-search-ranking-patch-local' \
  -H 'X-Step-Up-Token: <verified_step_up_challenge_id>' \
  -d '{
    "weights_json":{"quality_score":0.78,"hotness_score":0.12,"freshness_score":0.10},
    "filter_policy_json":{"blocked_statuses":["delisted"]},
    "status":"active"
  }'
```

9. 切换 alias：

```bash
curl -sS -X POST http://127.0.0.1:18080/api/v1/ops/search/aliases/switch \
  -H 'content-type: application/json' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-alias-local' \
  -H 'X-Idempotency-Key: idem-search-alias-local' \
  -H 'X-Step-Up-Token: <verified_step_up_challenge_id>' \
  -d '{
    "entity_scope":"product",
    "next_index_name":"product_search_v1_manual_candidate"
  }'
```

## 回查

重建任务：

```sql
SELECT entity_scope,
       entity_id::text,
       target_backend,
       target_index,
       sync_status,
       reconcile_status,
       retry_count,
       dead_letter_event_id::text
FROM search.index_sync_task
WHERE entity_id = '<product_uuid>'::uuid
ORDER BY scheduled_at DESC
LIMIT 5;
```

同步异常：

```sql
SELECT index_sync_task_id::text,
       entity_scope,
       entity_id::text,
       exception_type,
       exception_status,
       error_code,
       retryable,
       dead_letter_event_id::text,
       detected_at,
       resolved_at
FROM search.index_sync_exception
WHERE entity_id = '<product_uuid>'::uuid
ORDER BY detected_at DESC
LIMIT 5;
```

缓存失效：

```bash
redis-cli -u "redis://:${REDIS_PASSWORD}@127.0.0.1:${REDIS_PORT}/0" SCAN 0 MATCH 'datab:v1:search:catalog:product:*'
redis-cli -u "redis://:${REDIS_PASSWORD}@127.0.0.1:${REDIS_PORT}/0" MGET \
  'datab:v1:search:catalog:version:product' \
  'datab:v1:search:catalog:version:service' \
  'datab:v1:search:catalog:version:all'
```

alias 权威源与 OpenSearch 实际目标：

```sql
SELECT read_alias,
       write_alias,
       active_index_name
FROM search.index_alias_binding
WHERE entity_scope = 'product';
```

```bash
curl -sS "http://127.0.0.1:9200/_alias/product_search_read"
curl -sS "http://127.0.0.1:9200/_alias/product_search_write"
```

排序配置：

```sql
SELECT ranking_profile_id::text,
       weights_json,
       filter_policy_json,
       status
FROM search.ranking_profile
WHERE ranking_profile_id = '<ranking_profile_id>'::uuid;
```

审计与系统日志：

```sql
SELECT request_id,
       action_name,
       result_code,
       sensitivity_level
FROM audit.audit_event
WHERE request_id IN (
  'req-search-reindex-local',
  'req-search-cache-local',
  'req-search-ranking-patch-local',
  'req-search-alias-local'
)
ORDER BY created_at;

SELECT request_id,
       target_type,
       step_up_challenge_id::text
FROM audit.access_audit
WHERE request_id IN (
  'req-search-sync-local',
  'req-search-reindex-local',
  'req-search-cache-local',
  'req-search-ranking-list-local',
  'req-search-ranking-patch-local',
  'req-search-alias-local'
)
ORDER BY created_at;

SELECT request_id,
       message_text
FROM ops.system_log
WHERE request_id IN (
  'req-search-sync-local',
  'req-search-reindex-local',
  'req-search-cache-local',
  'req-search-ranking-list-local',
  'req-search-ranking-patch-local',
  'req-search-alias-local'
)
ORDER BY created_at;
```

## 当前 V1 口径

- `search.index_alias_binding` 是 product / seller 搜索 alias 的结构化 authority；`read_alias / write_alias / active_index_name` 必须与 ops 接口、初始化脚本和运行时默认值共享同一套答案。
- `search_sync_jobs_v1` 是辅助运维索引，用于本地初始化与排障回查；它不是 product / seller 搜索 alias authority 的组成部分。
- `alias switch` 属于当前 `V1` 的最小运维能力；`V3` 只扩展更复杂的灰度切换、自动回滚和策略化切换能力，不再承接“首次提供 alias switch”。
- 正式 topic：`dtp.search.sync`
- 正式 worker：`workers/search-indexer`
- 正式权限点：`portal.search.read`、`ops.search_sync.read`、`ops.search_reindex.execute`、`ops.search_alias.manage`、`ops.search_cache.invalidate`、`ops.search_ranking.read`、`ops.search_ranking.manage`
- 正式缓存 key 前缀：`datab:v1:search:catalog:*`
- 正式搜索缓存版本键：`datab:v1:search:catalog:version:{scope}`
- 正式 alias：`product_search_read/write`、`seller_search_read/write`
- 搜索域错误码：`SEARCH_QUERY_INVALID`、`SEARCH_BACKEND_UNAVAILABLE`、`SEARCH_RESULT_STALE`，以及写权限专属 `SEARCH_REINDEX_FORBIDDEN / SEARCH_ALIAS_SWITCH_FORBIDDEN / SEARCH_CACHE_INVALIDATE_FORBIDDEN`
- 宿主机直连 Kafka 时使用 `127.0.0.1:9094`；容器内监听地址 `kafka:9092` 只供 compose 网络内部使用
- `search-indexer` 已按 `AUD-026 + SEARCHREC-020` 收口为正式 consumer：统一使用 envelope `event_id` 写 `ops.consumer_idempotency_record`，处理失败时先写 `ops.dead_letter_event + dtp.dead-letter` 双层隔离，再决定 offset 提交；`AUD-010` 的 `POST /api/v1/ops/dead-letters/{id}/reprocess` 可对该失败记录执行 `dry_run + step-up` 预演。

## Worker 可靠性回归

运行 `search-indexer` worker 侧 smoke：

```bash
SEARCHREC_WORKER_DB_SMOKE=1 \
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo test -p search-indexer search_indexer_db_smoke -- --nocapture
```

该 smoke 当前必须同时证明：

- 成功路径会写入 OpenSearch `write alias`，推进相关 `datab:v1:search:catalog:version:{scope}` 版本键，并删除对应 scope 的 `datab:v1:search:catalog:*` Redis 候选缓存
- `ops.consumer_idempotency_record(consumer_name='search-indexer', result_code='processed')` 真实存在
- 重复投递同一 `event_id` 返回 `duplicate`，不会重复写副作用
- 失败路径会先写 `ops.dead_letter_event(failure_stage='consumer_handler', target_topic='dtp.search.sync')`
- Kafka `dtp.dead-letter` 会收到与同一 `event_id` 对齐的隔离消息
- `search.index_sync_task(sync_status='failed')` 与 `ops.consumer_idempotency_record(result_code='dead_lettered')` 真实可回查

对 SEARCHREC dead letter 做正式 `dry_run` 重处理预演：

```bash
AUD_DB_SMOKE=1 \
DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture
```

该 smoke 会真实回查 `search-indexer -> dtp.search.sync` 的 reprocess 计划、`step-up` 绑定、`audit.audit_event`、`audit.access_audit` 与 `ops.system_log`。
