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
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p recommendation-aggregator
```
