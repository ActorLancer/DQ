# TEST-010 Search / Recommendation PG Authority

`TEST-010` 的正式目标不是证明“搜索 / 推荐接口能返回结果”，而是证明读链即使命中 `OpenSearch` 候选或缓存，也必须回到 `PostgreSQL` 做最终业务校验；一旦商品在 `PostgreSQL` 权威状态已经变为 `delisted / frozen / rejected` 等不可见状态，就不能继续出现在搜索结果或推荐结果中。

## 正式入口

- 本地 / CI checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-searchrec-pg-authority.sh`
- 该 checker 会先执行 `./scripts/smoke-local.sh`，然后串行运行以下 live smoke：
  - `SEARCH_DB_SMOKE=1 cargo test -p platform-core search_visibility_and_alias_consistency_db_smoke -- --nocapture`
  - `SEARCH_DB_SMOKE=1 APP_MODE=local OPENSEARCH_ENDPOINT=http://127.0.0.1:1 cargo test -p platform-core search_catalog_pg_fallback_db_smoke -- --nocapture`
  - `RECOMMEND_DB_SMOKE=1 cargo test -p platform-core recommendation_get_api_db_smoke -- --nocapture`
  - `RECOMMEND_DB_SMOKE=1 cargo test -p platform-core recommendation_filters_frozen_product_db_smoke -- --nocapture`

## 覆盖闭环

- 搜索读链：`platform-core -> OpenSearch read alias -> PostgreSQL final business check -> HTTP response`
- 搜索 fallback：`platform-core -> PostgreSQL search projection -> PostgreSQL final business check -> Redis cache`
- 推荐读链：`platform-core -> OpenSearch candidate recall / PG relation recall -> PostgreSQL final business check -> recommendation_result_item`
- 正式参与服务：`PostgreSQL`、`OpenSearch`、`Redis`、`platform-core`

## 通过标准

- `search_visibility_and_alias_consistency_db_smoke`
  - 先证明 alias 切换后搜索请求真实命中新 `read alias`
  - 再把商品在 `PostgreSQL` 中改为 `delisted`
  - 断言 `OpenSearch` 旧文档仍可命中时，接口返回仍会把该商品过滤掉
- `search_catalog_pg_fallback_db_smoke`
  - 证明 `OpenSearch` 不可用时，目录搜索会退化到 `backend=postgresql`
  - 同时验证 Redis 搜索缓存 `cache_hit=false -> true`
- `recommendation_get_api_db_smoke`
  - 证明推荐请求、结果、结果项、`audit.access_audit`、`ops.system_log` 都真实落库
  - 作为 `recommendation_filters_frozen_product_db_smoke` 的正常基线路径
- `recommendation_filters_frozen_product_db_smoke`
  - 先证明冻结前推荐结果能命中目标商品
  - 再把商品在 `PostgreSQL` 中改为 `frozen`
  - 断言冻结后接口要么返回成功但结果中不再包含该商品，要么在候选被全部过滤后返回 `RECOMMENDATION_RESULT_UNAVAILABLE`
  - 无论哪种分支，新的 `recommend.recommendation_result_item` 都不能继续写入该冻结商品

## 关键回查

- 搜索投影权威状态：

```sql
SELECT listing_status,
       visibility_status,
       visible_to_search
FROM search.product_search_document
WHERE product_id = '<product_uuid>'::uuid;
```

- 推荐结果项不得落入冻结商品：

```sql
SELECT count(*)::bigint
FROM recommend.recommendation_result_item
WHERE recommendation_result_id = '<recommendation_result_uuid>'::uuid
  AND entity_id = '<frozen_product_uuid>'::uuid;
```

## 禁止误报

- 只看到 `OpenSearch` 仍有文档，不算失败证据；必须看最终 HTTP 结果是否已经被 `PostgreSQL` 过滤
- 只看到推荐候选召回成功，不算通过；必须再回查 `recommendation_result_item` 没有把冻结商品写进去
- 只看缓存命中或 `200 OK` 不算通过；必须同时验证 `backend`、最终结果集和数据库落点
