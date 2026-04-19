# docs 目录分层校准（BOOT-035）

当前工程文档分层固定为：

- `docs/00-context/`
- `docs/01-architecture/`
- `docs/02-openapi/`
- `docs/03-db/`
- `docs/04-runbooks/`
- `docs/05-test-cases/`

关键索引：

- `docs/01-architecture/order-orchestration.md`：Order / Contract / Authorization / Billing / Delivery 主交易编排、状态分层与乱序回调保护。
- `docs/04-runbooks/local-startup.md`：本地启动、`db/scripts/*` migration 主流程、`cargo sqlx prepare --workspace` 与 `platform-core` 启动参数。

说明：

- 既有专题文档目录（如 `开发准备/`、`数据库设计/`、`全集成文档/`）继续保留。
- 本分层用于后续工程化收口，不覆盖既有冻结正文。
- `platform-core` 当前数据访问栈已冻结为 `SQLx + SeaORM`，正式数据库仍为 `PostgreSQL`。
