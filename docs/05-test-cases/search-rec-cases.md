# Search / Recommendation Cases

## 当前批次说明

- 本文件当前主要冻结搜索/推荐主链路业务验收点。
- `SEARCHREC-01` 涉及的统一鉴权、正式权限点、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、审计留痕与搜索域 `SEARCH_*` 错误码，当前批次只做文档收口，不在这里伪造完整测试矩阵。
- 进入 `SEARCHREC` / `AUD` 代码实现批次后，Agent 必须补充上述验收项，并明确拒绝继续使用 `x-role` 占位请求来证明接口通过。
- 进入 `SEARCHREC` / `AUD` 代码实现批次后，Agent 还必须补充 SEARCHREC consumer 的 `event_id` 幂等、`ops.consumer_idempotency_record`、`ops.dead_letter_event + dtp.dead-letter` 双层隔离、worker 侧副作用与 `reprocess` 验收项，不允许继续用“手工 seed OpenSearch”“只断言 outbox 行存在”来冒充 worker 可靠性验证。

## Search V1

- 商品变更写入 `search.product.changed`，其 canonical topic 固定为 `dtp.search.sync`。
- `search-indexer` 消费 `dtp.search.sync` 后，从 PostgreSQL 搜索投影读取最新文档并写入 OpenSearch `write alias`。
- `search-indexer` 成功同步后会失效 `datab:v1:search:catalog:*` 缓存并回写 `search.index_sync_task`。
- `GET /api/v1/catalog/search` 必须经 `OpenSearch candidate -> PostgreSQL final check` 返回结果。
- `POST /api/v1/ops/search/reindex` 必须写入 `search.index_sync_task(sync_status='queued')`。
- `POST /api/v1/ops/search/cache/invalidate` 必须删除 Redis 搜索缓存。
- `POST /api/v1/ops/search/aliases/switch` 必须同步更新 OpenSearch alias 与 `search.index_alias_binding.active_index_name`。
- `GET /api/v1/ops/search/ranking-profiles` 与 `PATCH /api/v1/ops/search/ranking-profiles/{id}` 必须与 `search.ranking_profile` 一致。
- `search-indexer` 必须以统一事件 envelope 的 `event_id` 做 consumer 幂等，并把幂等记录写入 `ops.consumer_idempotency_record`。
- `search-indexer` 处理失败时，必须先落 `ops.dead_letter_event` 与 Kafka `dtp.dead-letter`，再决定是否提交 offset。
- `search-indexer` 的测试不得只用手工 seed OpenSearch 证明通过，必须验证 worker 侧真实副作用、失败隔离与 reprocess 路径。

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
