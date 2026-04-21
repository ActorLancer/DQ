# search-indexer

正式搜索同步 worker。

职责：

- 消费 `dtp.search.sync`
- 读取 PostgreSQL 搜索投影
- 写入 OpenSearch `write alias`
- 失效 Redis 搜索缓存
- 回写 `search.index_sync_task` 与投影同步状态

本地运行：

```bash
KAFKA_BROKERS=127.0.0.1:9094 cargo run -p search-indexer
```
