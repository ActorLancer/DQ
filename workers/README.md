# workers 目录校准（BOOT-031）

`workers/` 用于放置异步与离线 worker 落位。

- `search-indexer/`
- `recommendation-aggregator/`
- `outbox-publisher/`
- `data-processing-worker/`
- `quality-profiler/`
- `report-job/`

说明：

- 现有 `apps/search-indexer`、`apps/data-processing-worker` 与本目录存在阶段性并行落位。
- `workers/search-indexer` 已成为当前 `A07` 之后的正式搜索同步 worker 落位，运行命令：

```bash
cargo run -p search-indexer
```

- `apps/search-indexer` 仅保留历史说明，不再作为当前权威实现入口。
- `workers/recommendation-aggregator` 为当前 `A09` 之后的正式推荐行为聚合 worker 落位，运行命令：

```bash
cargo run -p recommendation-aggregator
```
