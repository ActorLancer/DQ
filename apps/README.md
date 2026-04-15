# apps 目录校准（BOOT-029）

当前 `apps/` 仅承载应用级骨架，命名已对齐 V1-Core 当前主应用与外围应用：

- `platform-core`
- `portal-web`
- `console-web`
- `fabric-adapter`
- `fabric-event-listener`
- `search-indexer`
- `data-processing-worker`
- `notification-worker`
- `mock-payment-provider`

说明：

- 目录已存在的应用骨架继续复用。
- 若后续按 BOOT/ENV 任务收敛到 `services/` 或 `workers/`，以迁移策略文档为准，当前不做代码迁移。
