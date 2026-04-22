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
- 成功写操作会真实写入 `audit.audit_event + audit.access_audit + ops.system_log`
- 读操作会真实写入 `audit.access_audit + ops.system_log`

## 本地前置

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
APP_PORT=18080 target/debug/platform-core-bin
```

启动 `search-indexer`：

```bash
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
```

## 手工 smoke

1. 准备一个具备 `platform_admin` 角色的 bearer token，对应 `core.user_account.user_id=<operator_user_id>`。

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
  -H 'Authorization: Bearer <access_token>' \
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
  -H 'Authorization: Bearer <access_token>' \
  -H 'X-Request-Id: req-search-sync-local'
```

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
       retry_count
FROM search.index_sync_task
WHERE entity_id = '<product_uuid>'::uuid
ORDER BY scheduled_at DESC
LIMIT 5;
```

缓存失效：

```bash
redis-cli -u "redis://:${REDIS_PASSWORD}@127.0.0.1:${REDIS_PORT}/0" KEYS 'datab:v1:search:catalog:product:*'
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

- 正式 topic：`dtp.search.sync`
- 正式 worker：`workers/search-indexer`
- 正式权限点：`portal.search.read`、`ops.search_sync.read`、`ops.search_reindex.execute`、`ops.search_alias.manage`、`ops.search_cache.invalidate`、`ops.search_ranking.read`、`ops.search_ranking.manage`
- 正式缓存 key 前缀：`datab:v1:search:catalog:*`
- 正式 alias：`product_search_read/write`、`seller_search_read/write`
- 搜索域错误码：`SEARCH_QUERY_INVALID`、`SEARCH_BACKEND_UNAVAILABLE`、`SEARCH_RESULT_STALE`，以及写权限专属 `SEARCH_REINDEX_FORBIDDEN / SEARCH_ALIAS_SWITCH_FORBIDDEN / SEARCH_CACHE_INVALIDATE_FORBIDDEN`
- 宿主机直连 Kafka 时使用 `127.0.0.1:9094`；容器内监听地址 `kafka:9092` 只供 compose 网络内部使用
- `search-indexer` 的 consumer 幂等 / 双层 DLQ / reprocess 仍由后续 `SEARCHREC` / `AUD-026` 收口，不允许把当前 `AUD-022` 控制面完成误报为 consumer 全链路完成
