# recommendation-aggregator

消费 `dtp.recommend.behavior`，聚合推荐行为对读模型的影响：

- 更新 `search.search_signal_aggregate`
- 刷新 `search.product_search_document` / `search.seller_search_document`
- 补写 `search.index_sync_task` 以触发后续索引同步
- 更新 `recommend.entity_similarity`
- 更新 `recommend.bundle_relation`
- 回写 `recommendation_result_item.click_status`
- 失效推荐缓存

本地启动：

```bash
KAFKA_BROKERS=127.0.0.1:9094 \
REDIS_URL=redis://default:datab_redis_pass@127.0.0.1:6379/1 \
cargo run -p recommendation-aggregator
```

如果手工覆盖了 `REDIS_URL`，必须保留 Redis 鉴权信息；否则缓存失效会报 `NOAUTH`，consumer 会按正式可靠性策略把事件送入 `ops.dead_letter_event + dtp.dead-letter`。
