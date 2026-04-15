# workers 目录校准（BOOT-031）

`workers/` 用于放置异步与离线 worker 落位，当前先完成目录边界冻结：

- `search-indexer/`
- `outbox-publisher/`
- `data-processing-worker/`
- `quality-profiler/`
- `report-job/`

说明：

- 现有 `apps/search-indexer`、`apps/data-processing-worker` 与本目录存在阶段性并行落位。
- 本批次仅冻结目录，不迁移代码实现。
