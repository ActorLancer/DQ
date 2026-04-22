# V1-Core 实施进度日志 P6

本文件是实施进度日志的当前续写分卷正文。

- 正式入口页：`docs/开发任务/V1-Core-实施进度日志.md`
- 当前活动分卷以入口页为准；当前入口页指向本卷
- 若后续切换到新的 `P{N}` 分卷，必须先更新入口页，再开始续写新分卷

### BATCH-247（计划中）
- 任务：SEARCHREC-001 `workers/search-indexer` 正式搜索同步 worker 与 `local/demo` PG fallback 运行边界收口
- 状态：计划中
- 说明：按 `SEARCHREC-001` 的冻结口径复核并补齐搜索主链运行模式分流，确保 `dtp.search.sync -> search-indexer -> OpenSearch` 继续作为 `staging` 正式路径，同时把 `platform-core` 启动自检与 `GET /api/v1/catalog/search` 收口到 `local/demo` 允许 PostgreSQL 搜索投影候选集 fallback、最终仍回 PostgreSQL 做可见性校验的正式语义，并补齐区分两种模式的真实验证与留痕。
- 追溯：关闭 `TODO-SEARCHREC-FALLBACK-001`，并继续沿 `SEARCHREC` 顺序推进。
### BATCH-247（待审批）
- 任务：`SEARCHREC-001` 初始化 `workers/search-indexer/`
- 状态：待审批
- 当前任务编号：`SEARCHREC-001`
- 前置依赖核对结果：`CAT-001`、`DB-011`、`DB-012`、`CORE-008` 已在前序阶段完成并可作为当前实现基线复用；`search-indexer`、`dtp.search.sync`、`search.index_sync_task`、OpenSearch alias / reindex / sync 运维控制面与统一鉴权/审计链已存在可验证实现，本批重点补齐 `SEARCHREC-001` 仍缺失的运行边界收口与真实验证。
- 完成情况：
  - `apps/platform-core/src/modules/search/repo/mod.rs`、`api/handlers.rs`：把搜索候选源按 `RuntimeMode` 分流为 `staging -> OpenSearch`、`local/demo -> PostgreSQL search projection fallback`，并把缓存指纹与响应 `backend` 一并区分，避免本地 fallback 命中 OpenSearch 路径缓存。
  - `apps/platform-core/src/lib.rs`：`startup_self_check` 只在 `staging` 强制校验 OpenSearch alias / index；`local/demo` 保留 Kafka / Redis / PostgreSQL 等共性依赖校验，但不再把 OpenSearch 初始化作为启动硬前置。
  - `apps/platform-core/src/modules/search/tests/search_api_db.rs`：补齐双模式 DB smoke；`search_api_and_ops_db_smoke` 继续验证 `staging` 正式路径的 OpenSearch / Redis / alias / reindex / ranking / audit 闭环，`search_catalog_pg_fallback_db_smoke` 新增验证 `APP_MODE=local` 且 OpenSearch endpoint 不可用时，目录搜索仍可通过 PostgreSQL 投影返回 `backend=postgresql` 并命中 Redis 搜索短缓存。
  - `workers/search-indexer/src/main.rs`、`apps/platform-core/src/modules/recommendation/repo/mod.rs`、`workers/recommendation-aggregator/src/main.rs`：把本地 Redis 默认连接串统一到当前基础设施实际 ACL 口径 `redis://default:<password>@host:port/db`，消除 `SEARCHREC` smoke 与宿主机 Redis 鉴权的偏差。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/search-reindex.md`、`docs/05-test-cases/search-rec-cases.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`：把 `SEARCHREC-001` 的正式模式矩阵、OpenSearch 运维 runbook 的 `staging` 前提、`local/demo` fallback 的验证命令与 `TODO-SEARCHREC-FALLBACK-001` 关闭状态同步落盘。
- 验证：
  - `cargo fmt --all`
  - `cargo check -p platform-core`
  - `cargo test -p platform-core`
  - `SEARCH_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_api_and_ops_db_smoke -- --nocapture`
  - `SEARCH_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core search_catalog_pg_fallback_db_smoke -- --nocapture`
  - `SEARCHREC_WORKER_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo test -p search-indexer search_indexer_db_smoke -- --nocapture`
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  - `./scripts/check-query-compile.sh`
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`SEARCHREC-001`
  - `商品搜索、排序与索引同步设计.md`：`5. V1 正式方案`、`local/demo 允许 PG 投影运行`、`PostgreSQL fallback 搜索`、`6. 搜索投影设计`
  - `商品搜索、排序与索引同步接口协议正式版.md`：`4. V1 接口` 与搜索读链 / OpenSearch / Redis / PostgreSQL 最终校验边界
  - `A07-搜索同步链路与搜索接口闭环缺口.md`
  - `A15-SEARCHREC-Consumer-幂等与DLQ闭环缺口.md`
  - `A01-Kafka-Topic-口径统一.md`
  - `A02-统一事件-Envelope-与路由权威源.md`
- 覆盖的任务清单条目：`SEARCHREC-001`
- 未覆盖项：
  - 无。`SEARCHREC-001` 要求的 `search-indexer` 正式 worker 基线、`dtp.search.sync` 正式主题、`search.index_sync_task` 记录表定位，以及 `staging` / `local-demo` 搜索候选源边界已通过现有 worker 闭环 + 本批运行时收口全部固定。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；历史 gap `TODO-SEARCHREC-FALLBACK-001` 已关闭。
- 备注：
  - `RuntimeMode` 当前只显式区分 `local / staging / demo`。由于仓库尚无独立 `production` 枚举，本批按冻结口径把 `staging` 视为正式 OpenSearch 路径的本地等价运行模式；后续如引入独立 `production` mode，必须继承同一强制 OpenSearch 语义，不得与本批结果漂移。
