# Redis Key 规范（ENV-013）

本地默认命名空间前缀：`datab:v1`（来源：`REDIS_NAMESPACE`）。

## key 模式

- 幂等键：`{ns}:idempotency:{scope}:{idempotency_key}`
- 会话缓存：`{ns}:session:{user_id}:{session_id}`
- 权限缓存：`{ns}:perm:{tenant_id}:{actor_id}`
- 搜索候选缓存：`{ns}:search:catalog:{entity_scope}:{query_hash}`
- 搜索缓存版本键：`{ns}:search:catalog:version:{entity_scope}`
- 推荐缓存：`{ns}:recommend:{tenant_id}:{actor_id}:{scene}`
- 推荐已看集合：`{ns}:recommend:seen:{subject_ref}:{placement_code}`
- 限流计数：`{ns}:ratelimit:{api}:{actor_id}:{window}`
- 下载票据缓存：`{ns}:download-ticket:{ticket_id}`
- Fabric consumer 短锁：`{ns}:fabric-adapter:consumer-lock:{event_id}`

## DB 划分建议（本地）

- DB 0：会话与权限缓存
- DB 1：推荐与召回缓存
- DB 2：限流与幂等键
- DB 3：下载票据与短期授权缓存
- DB 4：Fabric consumer 短锁与 Go 侧消费辅助状态

## 过期策略

- 幂等键：15 分钟
- 会话缓存：与会话 TTL 对齐（建议 30 分钟）
- 权限缓存：5 分钟
- 搜索候选缓存：5 分钟
- 搜索缓存版本键：不设置 TTL；通过 `INCR` 推进，配合候选缓存 TTL 做软失效
- 推荐缓存：10 分钟
- 推荐已看集合：24 小时
- 限流计数：按窗口期（1~5 分钟）
- 下载票据：5 分钟（一次性使用）
- Fabric consumer 短锁：15 秒（处理完成后主动释放）

## 搜索缓存失效口径

- 搜索读缓存仍统一落在 `datab:v1:search:catalog:*` 前缀下，候选缓存 key 不引入新前缀。
- `GET /api/v1/catalog/search` 命中缓存时会比对 `search:catalog:version:{entity_scope}` 当前版本；版本漂移的旧值会被当作 miss 并清理。
- `POST /api/v1/ops/search/cache/invalidate`、`PATCH /api/v1/ops/search/ranking-profiles/{id}`、`POST /api/v1/ops/search/aliases/switch` 会推进相关 scope 的版本键，并删除匹配的候选缓存 key。
- `workers/search-indexer` 成功写入 OpenSearch 后会推进相关 scope 的版本键，并删除匹配的候选缓存 key。
- `product / service` 相关运维失效会级联覆盖 `product + service + all`；`seller` 相关失效会级联覆盖 `seller + all`。
