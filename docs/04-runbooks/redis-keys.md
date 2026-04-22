# Redis Key 规范（ENV-013）

本地默认命名空间前缀：`datab:v1`（来源：`REDIS_NAMESPACE`）。

## key 模式

- 幂等键：`{ns}:idempotency:{scope}:{idempotency_key}`
- 会话缓存：`{ns}:session:{user_id}:{session_id}`
- 权限缓存：`{ns}:perm:{tenant_id}:{actor_id}`
- 搜索候选缓存：`{ns}:search:catalog:{entity_scope}:{query_hash}`
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
- 推荐缓存：10 分钟
- 推荐已看集合：24 小时
- 限流计数：按窗口期（1~5 分钟）
- 下载票据：5 分钟（一次性使用）
- Fabric consumer 短锁：15 秒（处理完成后主动释放）
