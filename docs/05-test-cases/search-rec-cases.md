# Search / Recommendation Cases

## 当前批次说明

- `AUD-022` 已真实落地搜索运维控制面的统一 Bearer 鉴权、正式权限点、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、`audit.audit_event + audit.access_audit + ops.system_log` 留痕，以及搜索域 `SEARCH_*` 错误码。
- `AUD-022` 的自动化 smoke 已真实覆盖 PostgreSQL、Redis 与 OpenSearch：
  - 目录搜索两次命中验证 `cache_hit=false -> true`
  - Redis 搜索缓存真实失效
  - `search.index_sync_task` 真实排队
  - OpenSearch alias 与 `search.index_alias_binding.active_index_name` 真实切换
  - `search.ranking_profile` 真实更新
- `AUD-026` 已真实补齐 SEARCHREC consumer 可靠性闭环：
  - `search-indexer` 与 `recommendation-aggregator` 都基于统一 envelope `event_id` 写入 `ops.consumer_idempotency_record`
  - 失败路径都会进入 `ops.dead_letter_event + dtp.dead-letter` 双层隔离
  - worker 侧副作用、重复投递去重和 `POST /api/v1/ops/dead-letters/{id}/reprocess` 的 `dry_run` 预演都已有真实 smoke
  - 验收时不允许继续用“手工 seed OpenSearch”“只断言 outbox 行存在”冒充 worker 可靠性验证

## Search V1

- 商品变更写入 `search.product.changed`，其 canonical topic 固定为 `dtp.search.sync`。
- `search-indexer` 消费 `dtp.search.sync` 后，从 PostgreSQL 搜索投影读取最新文档并写入 OpenSearch `write alias`。
- `search-indexer` 成功同步后会失效 `datab:v1:search:catalog:*` 缓存并回写 `search.index_sync_task`。
- `GET /api/v1/catalog/search` 在 `staging` 必须经 `OpenSearch candidate -> PostgreSQL final check` 返回结果，并显式返回 `backend=opensearch`。
- `GET /api/v1/catalog/search` 在 `local / demo` 允许经 `PostgreSQL search projection candidate -> PostgreSQL final check` 返回结果，并显式返回 `backend=postgresql`。
- `GET /api/v1/catalog/search` 当前正式要求 `Authorization: Bearer <access_token>`，且两次相同查询应能观察到 `cache_hit=false -> true`。
- `POST /api/v1/ops/search/reindex` 必须写入 `search.index_sync_task(sync_status='queued')`。
- `POST /api/v1/ops/search/reindex` 必须要求 `ops.search_reindex.execute + X-Idempotency-Key + X-Step-Up-Token`，并写入 `audit.audit_event(action_name='search.reindex.queue')`、`audit.access_audit(target_type='search_reindex')`、`ops.system_log`。
- `POST /api/v1/ops/search/cache/invalidate` 必须删除 Redis 搜索缓存。
- `POST /api/v1/ops/search/cache/invalidate` 必须要求 `ops.search_cache.invalidate + X-Idempotency-Key`，且不得伪造 step-up。
- `POST /api/v1/ops/search/aliases/switch` 必须同步更新 OpenSearch alias 与 `search.index_alias_binding.active_index_name`。
- `POST /api/v1/ops/search/aliases/switch` 必须要求 `ops.search_alias.manage + X-Idempotency-Key + X-Step-Up-Token`，并且回查 OpenSearch `/_alias/*` 与 PostgreSQL 权威源一致。
- `GET /api/v1/ops/search/ranking-profiles` 与 `PATCH /api/v1/ops/search/ranking-profiles/{id}` 必须与 `search.ranking_profile` 一致。
- `PATCH /api/v1/ops/search/ranking-profiles/{id}` 必须要求 `ops.search_ranking.manage + X-Idempotency-Key + X-Step-Up-Token`，并真实写入 `search.ranking_profile`。
- 搜索运维控制面必须统一使用 `Authorization`，不再接受 `x-role`。
- 搜索运维控制面必须使用搜索域错误码：`SEARCH_QUERY_INVALID`、`SEARCH_BACKEND_UNAVAILABLE`、`SEARCH_RESULT_STALE`，以及权限专属 `SEARCH_REINDEX_FORBIDDEN / SEARCH_ALIAS_SWITCH_FORBIDDEN / SEARCH_CACHE_INVALIDATE_FORBIDDEN`。
- `search-indexer` 必须以统一事件 envelope 的 `event_id` 做 consumer 幂等，并把幂等记录写入 `ops.consumer_idempotency_record`。
- `search-indexer` 处理失败时，必须先落 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter`，再决定是否提交 offset。
- `search-indexer` 的测试不得只用手工 seed OpenSearch 证明通过，必须验证 worker 侧真实副作用、失败隔离与 reprocess 路径。
- 搜索回归命令：
  - `SEARCH_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture`
  - `SEARCH_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_catalog_pg_fallback_db_smoke -- --nocapture`
  - `SEARCHREC_WORKER_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo test -p search-indexer search_indexer_db_smoke -- --nocapture`
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture`

## Recommendation V1

- `GET /api/v1/recommendations` 必须走 `OpenSearch recall + PostgreSQL final check + recommendation_result` 落库闭环。
- 推荐返回前必须过滤掉 PostgreSQL 最终不可见对象，不能直接信任 OpenSearch 命中。
- 推荐缓存必须写入 `datab:v1:recommend:*`，曝光后应能看到 `datab:v1:recommend:seen:*` 已看集合。
- `POST /api/v1/recommendations/track/exposure` 必须写 `recommendation_panel_viewed` 与 `recommendation_item_exposed`，并支持 `X-Idempotency-Key` 幂等。
- `POST /api/v1/recommendations/track/click` 必须写点击事件，并把 canonical outbox topic 固定到 `dtp.recommend.behavior`。
- `recommendation-aggregator` 消费 `dtp.recommend.behavior` 后，必须更新 `search.search_signal_aggregate`、`recommend.entity_similarity`、`recommend.bundle_relation`。
- 推荐行为导致热度变化后，必须刷新搜索投影并补写 `search.index_sync_task(sync_status='queued')`。
- `GET /api/v1/ops/recommendation/placements` 与 `PATCH /api/v1/ops/recommendation/placements/{placement_code}` 必须与 `recommend.placement_definition` 一致。
- `GET /api/v1/ops/recommendation/ranking-profiles` 与 `PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 必须与 `recommend.ranking_profile` 一致。
- `POST /api/v1/ops/recommendation/rebuild` 必须支持推荐缓存失效和推荐派生特征重建。
- `recommendation-aggregator` 必须同样基于 `event_id` 做 consumer 幂等，并写入 `ops.consumer_idempotency_record`。
- `recommendation-aggregator` 处理失败时，必须进入 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter` 双层隔离，且不得在失败后直接提交 offset。
- 推荐行为流测试不得只断言 `ops.outbox_event` 有行存在，还必须验证 consumer 侧派生状态、副作用、DLQ 与 `POST /api/v1/ops/dead-letters/{id}/reprocess` 路径。
- 推荐回归命令：
  - `SEARCHREC_WORKER_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo test -p recommendation-aggregator recommendation_aggregator_db_smoke -- --nocapture`
  - `AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core audit_dead_letter_reprocess_db_smoke -- --nocapture`
