# SQLx + SeaORM 迁移任务清单（阅读版）

## 0. 说明

本文档是 [sqlx-seaorm-migration-implementation-plan.md](/home/luna/Documents/DataB/plans/sqlx-seaorm-migration-implementation-plan.md) 的可执行任务清单版。

目标：

- 完全移除 `apps/platform-core` 中对 `tokio-postgres` 的依赖。
- 全量迁移到 `SQLx + SeaORM`。
- 保留全部现有功能、接口、错误码、审计、事件、状态机语义。
- 保持现有 SQL migration 主流程不变。
- 当前正式支持 `Postgres`；同时建立未来 `MySQL` 扩展边界。

执行规则：

- 每个任务完成后必须执行“验证命令 + API 联调 + DB 回查 + 完整测试范围”。
- 命令侧默认 `SQLx`；查询侧默认 `SeaORM`，复杂聚合查询允许继续 `SQLx`。
- 严禁在迁移过程中引入新的 `tokio_postgres::connect(...)`。
- 最终必须满足：`rg -n "tokio_postgres|tokio-postgres" apps/platform-core` 返回空。

当前分支：

- `sqlx-seaorm-migration`

## 1. DBX 数据访问基础设施与抽象层迁移

- **DBX-001** [AGENT][P0][W0][serial-first] 冻结迁移边界、影响面、排除项与验收基线。  
  依赖：无  
  交付：`plans/sqlx-seaorm-migration-implementation-plan.md`；`plans/sqlx-seaorm-migration-tasklist.md`；`plans/sqlx-seaorm-migration-tasklist.csv`  
  完成定义：迁移边界、模块拆分、保留项、回滚方式、最终验收标准全部冻结；明确只改数据库访问相关代码与技术说明。  
  验收：任务清单与实施方案一致；明确迁移后仍由 `db/scripts/*` 负责 migration。  
  阻塞风险：边界不冻结会导致实现范围失控、回归验证漂移。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L7-L14` | `plans/sqlx-seaorm-migration-implementation-plan.md:L498-L520` | `docs/开发准备/技术选型正式版.md:L22-L28`  
  验证命令：  
  `git branch --show-current`  
  `git status --short`  
  `rg -n "tokio_postgres|tokio-postgres" apps/platform-core`  
  联调要求：无。  
  完整测试：无；本任务是基线冻结任务。  

- **DBX-002** [AGENT][P0][W0][serial-first] 在 workspace 和 `crates/db` 引入 `SQLx + SeaORM` 依赖骨架，并建立编译通过的最小迁移入口。  
  依赖：DBX-001  
  交付：`apps/platform-core/Cargo.toml`；`apps/platform-core/crates/db/Cargo.toml`；`Cargo.lock`  
  完成定义：新增 `sqlx`、`sea-orm`、`sea-query`、`sea-query-binder`；保留 `tokio-postgres` 只作为过渡依赖但不新增新调用。  
  验收：依赖图中可看到 `sqlx` 与 `sea-orm`；`platform-core` 可通过 `cargo check`。  
  阻塞风险：依赖特性选择错误会导致后续 runtime、query macro、entity 无法落地。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L312-L346` | `plans/sqlx-seaorm-migration-implementation-plan.md:L458-L496` | `docs/开发准备/技术选型正式版.md:L30-L49`  
  验证命令：  
  `cargo check -p platform-core`  
  `cargo tree -p platform-core | rg "sqlx|sea-orm|tokio-postgres"`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core` 冒烟通过。  

- **DBX-003** [AGENT][P0][W0][serial-first] 重构 `crates/db`，建立 `AppDb`、方言识别、连接池、统一错误映射与事务模板。  
  依赖：DBX-002  
  交付：`apps/platform-core/crates/db/src/lib.rs`；`config.rs`；`dialect.rs`；`error.rs`；`runtime/*`；`sqlx/*`  
  完成定义：能够根据 DSN 识别 `Postgres/MySQL`；当前正式实现 `PgPool + DatabaseConnection`；暴露统一错误转换和事务入口。  
  验收：`crates/db` 提供 `AppDb` 和 transaction helper；旧 `DbPool` 仅作为兼容过渡或被替换。  
  阻塞风险：基础 runtime 不稳定会导致后续所有模块重复返工。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L79-L120` | `plans/sqlx-seaorm-migration-implementation-plan.md:L152-L196` | `plans/sqlx-seaorm-migration-implementation-plan.md:L605-L637`  
  验证命令：  
  `cargo check -p db`  
  `cargo test -p db`  
  `cargo check -p platform-core`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **DBX-004** [AGENT][P0][W0][serial-first] 在 `platform-core` 引入 `AppState`，把 Router 与 handler 全部切换到状态注入，不再在 handler 内直连数据库。  
  依赖：DBX-003  
  交付：`apps/platform-core/src/lib.rs`；`apps/platform-core/crates/http/src/lib.rs`；各模块 `router()` 与 handler 签名  
  完成定义：`build_router(...)` 支持注入 `AppState`；模块 router 通过 `with_state` 共享数据库 runtime；handler 不再直接 `connect()`。  
  验收：`/health/live`、`/health/ready`、一个真实业务 GET 接口都可以在新状态注入模型下工作。  
  阻塞风险：状态注入不完整会导致部分模块仍残留 driver 直连。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L111-L148` | `plans/sqlx-seaorm-migration-implementation-plan.md:L500-L520` | `docs/开发准备/仓库拆分与目录结构建议.md:L173-L180`  
  验证命令：  
  `cargo fmt --all`  
  `cargo check -p platform-core`  
  `cargo test -p platform-core`  
  `APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`  
  联调要求：  
  `curl http://127.0.0.1:8094/health/live`  
  `curl http://127.0.0.1:8094/health/ready`  
  `curl -H 'x-user-id: 00000000-0000-0000-0000-000000000001' -H 'x-tenant-id: 11111111-1111-1111-1111-111111111111' -H 'x-role: tenant_admin' http://127.0.0.1:8094/api/v1/orders/standard-templates`  
  完整测试：基础服务启动验证 + `cargo test -p platform-core`。  

- **DBX-005** [AGENT][P0][W0][serial-first] 建立 repository trait 与 backend registry，显式区分 `Postgres` 正式实现与 `MySQL` 预留实现。  
  依赖：DBX-003  
  交付：`apps/platform-core/crates/db/src/runtime/*`；各领域仓储 trait 注册层  
  完成定义：命令与查询仓储 trait 明确；`Postgres*Repo` 已可注册；`MySql*Repo` 预留类型与构造路径存在但不承担当前业务实现承诺。  
  验收：应用初始化时能按 dialect 注册仓储实现；不使用 `sqlx::Any`。  
  阻塞风险：没有 backend seam 会导致未来 MySQL 扩展只能再次推翻架构。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L415-L456` | `plans/sqlx-seaorm-migration-implementation-plan.md:L577-L593` | `docs/开发准备/技术选型正式版.md:L24-L27`  
  验证命令：  
  `cargo check -p platform-core`  
  `rg -n "sqlx::Any|AnyPool" apps/platform-core/crates/db apps/platform-core/src`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **DBX-006** [AGENT][P0][W0][limited] 为 `catalog/iam/billing/trade` 生成并整理 SeaORM entity，建立统一导出层。  
  依赖：DBX-003；DBX-005  
  交付：`apps/platform-core/crates/db/src/entity/*`；`entity/mod.rs`；`entity/prelude.rs`  
  完成定义：entity 文件进入版本控制；禁止 entity 直接暴露到 API DTO；按 schema 组织子模块。  
  验收：至少能被 `catalog`、`iam` 的查询仓储直接引用；生成结果可编译。  
  阻塞风险：entity 组织混乱会导致 ORM 层反向污染业务模型。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L171-L196` | `plans/sqlx-seaorm-migration-implementation-plan.md:L597-L603` | `plans/sqlx-seaorm-migration-implementation-plan.md:L485-L496`  
  验证命令：  
  `sea-orm-cli generate entity --database-url postgres://datab:datab_local_pass@127.0.0.1:5432/datab --output-dir apps/platform-core/crates/db/src/entity --entity-format dense`  
  `cargo check -p platform-core`  
  `rg -n "pub mod" apps/platform-core/crates/db/src/entity`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **DBX-007** [AGENT][P0][W0][limited] 建立 SQLx 编译期查询校验流程，并将 `.sqlx/` 元数据纳入仓库。  
  依赖：DBX-002  
  交付：`.sqlx/`；相关脚本或 README 说明  
  完成定义：稳定 SQL 查询使用 `query!` 或 `query_as!`；`cargo sqlx prepare --workspace` 可生成元数据。  
  验收：schema 变动会通过编译期校验暴露；不影响现有 SQL migration 主流程。  
  阻塞风险：没有编译期查询校验，迁移完成后仍会留下运行期 SQL 漂移风险。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L472-L496` | `plans/sqlx-seaorm-migration-implementation-plan.md:L648-L656` | `docs/开发任务/v1-core-开发任务清单.md:L1037-L1043`  
  验证命令：  
  `cargo sqlx prepare --workspace`  
  `test -d .sqlx`  
  `cargo check -p platform-core`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **DBX-008** [AGENT][P0][W0][serial-first] 把共享 DB 测试夹具、seed helper 与连接帮助函数从 `tokio-postgres` 切换到 `SQLx`。  
  依赖：DBX-003；DBX-004  
  交付：`apps/platform-core/crates/db/src/testing/*`；各模块测试公共 helper  
  完成定义：新测试帮助函数统一使用 `PgPool` 或 transaction fixture；不再要求测试内部手工 `connect(NoTls)`。  
  验收：后续模块 smoke test 可复用统一 SQLx fixture。  
  阻塞风险：测试层不先迁移，后续业务模块会一直被旧驱动牵制。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L182-L196` | `plans/sqlx-seaorm-migration-implementation-plan.md:L583-L593` | `plans/sqlx-seaorm-migration-implementation-plan.md:L669-L685`  
  验证命令：  
  `cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade023_order_templates_db_smoke -- --nocapture`  
  `cargo test -p platform-core cat020_read_db -- --nocapture`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

## 2. ORDX 订单与交易主链路迁移

- **ORDX-001** [AGENT][P0][W1][serial-first] 按 `command/query/shared` 重新组织 `order/repo` 目录，不改变外部行为。  
  依赖：DBX-004；DBX-005  
  交付：`apps/platform-core/src/modules/order/repo/mod.rs`；`repo/command/*`；`repo/query/*`；`repo/shared/*`  
  完成定义：原单层 repo 文件按职责拆分；公共审计、错误和 mapping 移入 `shared`；不产生 API 语义变化。  
  验收：`order` 模块可编译；路由与函数导出保持稳定。  
  阻塞风险：不先完成目录重组，后续 SQLx/SeaORM 改造会继续堆到超大文件。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L198-L238` | `plans/sqlx-seaorm-migration-implementation-plan.md:L347-L357` | `docs/开发准备/仓库拆分与目录结构建议.md:L173-L180`  
  验证命令：  
  `cargo fmt --all`  
  `cargo check -p platform-core`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **ORDX-002** [AGENT][P0][W1][serial-first] 将 `order/api/handlers.rs` 全量切换到 `State<AppState>` 和仓储调用，不再直接依赖数据库驱动。  
  依赖：ORDX-001  
  交付：`apps/platform-core/src/modules/order/api/handlers.rs`；`apps/platform-core/src/modules/order/api/mod.rs`  
  完成定义：建单、查单、模板读取、生命周期快照、各 SKU transition、合同确认、授权迁移全部从状态注入获取数据库访问能力。  
  验收：`order` API 层不再出现 `tokio_postgres::*`。  
  阻塞风险：handler 残留旧驱动会让 repo 迁移收益被抵消。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L123-L128` | `plans/sqlx-seaorm-migration-implementation-plan.md:L529-L538` | `plans/sqlx-seaorm-migration-implementation-plan.md:L607-L616`  
  验证命令：  
  `cargo check -p platform-core`  
  `rg -n "tokio_postgres|NoTls|connect\\(" apps/platform-core/src/modules/order/api`  
  联调要求：  
  启动服务后执行 `GET /api/v1/orders/standard-templates`；`GET /api/v1/orders/{id}` 至少返回 200 或标准 not found。  
  完整测试：`cargo test -p platform-core`。  

- **ORDX-003** [AGENT][P0][W1][serial-first] 将订单命令仓储第一批迁移到 SQLx：`create_order`、`freeze_price_snapshot`、`cancel_order`、`confirm_contract`。  
  依赖：ORDX-002；DBX-008  
  交付：`create_order.rs`；`freeze_price_snapshot.rs`；`cancel_order.rs`；`confirm_contract.rs`  
  完成定义：相关事务全部改为 SQLx transaction；行为保持与当前基线一致；审计和 outbox 继续同事务。  
  验收：建单、补冻结、取消、合同确认全量测试通过。  
  阻塞风险：这是最核心的事务入口，任何语义偏移都会污染后续所有链路。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L605-L637` | `plans/sqlx-seaorm-migration-implementation-plan.md:L522-L539` | `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`  
  验证命令：  
  `cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade005_order_cancel_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade006_contract_confirm_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade016_digital_contract_aggregate_db_smoke -- --nocapture`  
  联调要求：  
  `POST /api/v1/orders`  
  `POST /api/v1/trade/orders/{id}/price-snapshot/freeze`  
  `POST /api/v1/orders/{id}/cancel`  
  `POST /api/v1/orders/{id}/contract-confirm`  
  并使用 `psql` 回查 `trade.order_main`、`contract.digital_contract`、`audit.audit_event`、`ops.outbox_event`。  
  完整测试：`cargo test -p platform-core`。  

- **ORDX-004** [AGENT][P0][W1][serial-first] 将订单命令仓储第二批迁移到 SQLx：全部 SKU transition、授权迁移、自动断权、锁资前校验、可交付判定器、支付结果编排。  
  依赖：ORDX-003  
  交付：`transition_*.rs`；`authorization_transition.rs`；`authorization_cutoff.rs`；`deliverability_gate.rs`；`pre_payment_lock.rs`；订单应用层支付编排  
  完成定义：所有状态机推进和支付编排命令路径改用 SQLx；`FOR UPDATE`、CAS 更新、乱序保护语义保持一致。  
  验收：八个 SKU 状态机、授权 cutoff、锁资门禁、支付编排、交付门禁测试全部通过。  
  阻塞风险：这里覆盖主交易闭环的核心命令；任何并发语义退化都会造成严重回归。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L383-L413` | `plans/sqlx-seaorm-migration-implementation-plan.md:L607-L616` | `docs/01-architecture/order-orchestration.md`  
  验证命令：  
  `cargo test -p platform-core trade007_state_machine_fields_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade008_file_std_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade009_file_sub_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade010_api_sub_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade011_api_ppu_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade012_share_ro_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade013_qry_lite_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade014_sbx_std_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade015_rpt_std_state_machine_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade017_authorization_aggregate_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade018_auto_cutoff_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade021_pre_payment_lock_checks_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade024_illegal_state_regression_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade025_authorization_min_structure_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade030_payment_result_orchestrator_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade031_deliverability_gate_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade032_scenario_sku_snapshot_db_smoke -- --nocapture`  
  联调要求：  
  至少完成 `POST /api/v1/orders/{id}/file-std/transition`、`/share-ro/transition`、`/authorization/transition`、`/api-sub/transition`、`/api-ppu/transition`、`POST /api/v1/payments/webhooks/mock_payment` 联调，并用 `psql` 回查订单、授权、交付、审计。  
  完整测试：`cargo test -p platform-core`。  

- **ORDX-005** [AGENT][P0][W1][serial-first] 迁移订单查询仓储：订单详情、生命周期快照、关系装配器。  
  依赖：ORDX-001；DBX-006  
  交付：`order_detail.rs`；`lifecycle_snapshots.rs`；`relations.rs`  
  完成定义：稳定读模型优先使用 SeaORM；复杂聚合允许继续 SQLx；返回 DTO 与当前接口完全一致。  
  验收：订单详情接口与生命周期接口无字段漂移；`relations` 聚合保持一致。  
  阻塞风险：读模型如果误让 entity 直接泄漏到 DTO，会导致接口层不稳定。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L397-L413` | `plans/sqlx-seaorm-migration-implementation-plan.md:L597-L603` | `plans/sqlx-seaorm-migration-implementation-plan.md:L522-L539`  
  验证命令：  
  `cargo test -p platform-core trade004_order_detail_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade019_lifecycle_snapshots_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade022_order_relations_db_smoke -- --nocapture`  
  联调要求：  
  `GET /api/v1/orders/{id}`  
  `GET /api/v1/orders/{id}/lifecycle-snapshots`  
  用 `psql` 回查 `trade.order_main`、`contract.digital_contract`、`trade.authorization_grant`、`delivery.delivery_record`、`billing` 聚合相关表。  
  完整测试：`cargo test -p platform-core`。  

- **ORDX-006** [AGENT][P0][W1][serial-first] 执行订单模块全量回归：DB smoke、真实 API 主链路、状态机、支付、授权、合同、交付闭环。  
  依赖：ORDX-003；ORDX-004；ORDX-005；DBX-007  
  交付：订单模块回归结果记录；必要修复提交  
  完成定义：订单链路从建单到合同、锁资、交付、授权、自动断权、支付编排均在新栈下通过。  
  验收：`order` 模块不含 `tokio-postgres`；主链路 API 联调全部通过。  
  阻塞风险：如果不做一轮集中回归，很容易留下局部命令和查询语义不一致的问题。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L639-L685` | `docs/05-test-cases/order-state-machine.md` | `docs/01-architecture/order-orchestration.md`  
  验证命令：  
  `cargo test -p platform-core trade027_main_trade_flow_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade023_order_templates_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade030_payment_result_orchestrator_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade031_deliverability_gate_db_smoke -- --nocapture`  
  联调要求：  
  启动服务后完整跑一条 `FILE_STD` 主链路和一条 `SHARE_RO` 主链路：  
  `POST /api/v1/orders` → `POST /api/v1/orders/{id}/contract-confirm` → `POST /api/v1/orders/{id}/file-std/transition` 或 `share-ro/transition` → `POST /api/v1/orders/{id}/authorization/transition` → `GET /api/v1/orders/{id}`；使用 `psql` 验证订单、授权、交付、审计。  
  完整测试：`cargo test -p platform-core`。  

## 3. BILX 支付与计费路径迁移

- **BILX-001** [AGENT][P0][W2][serial-first] 将 `billing` 的 db helper、handlers、webhook 路径迁移到 SQLx。  
  依赖：DBX-004；DBX-008  
  交付：`apps/platform-core/src/modules/billing/db.rs`；`handlers.rs`；必要的 `repo/*`  
  完成定义：支付意图创建、查询、取消、锁资、webhook 处理全部使用 SQLx；旧 `tokio_postgres::Row`/`Client` 全部移除。  
  验收：billing API 层与 helper 层不再引用 `tokio-postgres`。  
  阻塞风险：支付路径如果与订单路径迁移不一致，会出现跨模块事务语义断裂。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L292-L308` | `plans/sqlx-seaorm-migration-implementation-plan.md:L540-L555` | `plans/sqlx-seaorm-migration-implementation-plan.md:L607-L637`  
  验证命令：  
  `cargo fmt --all`  
  `cargo test -p platform-core billing`  
  `cargo check -p platform-core`  
  联调要求：  
  `POST /api/v1/payments/intents`  
  `GET /api/v1/payments/intents/{id}`  
  `POST /api/v1/payments/intents/{id}/cancel`  
  `POST /api/v1/orders/{id}/lock`  
  `POST /api/v1/payments/webhooks/mock_payment`  
  完整测试：`cargo test -p platform-core`。  

- **BILX-002** [AGENT][P0][W2][serial-first] 执行 billing 全量回归，覆盖 webhook 乱序保护、支付结果推进、支付意图一致性。  
  依赖：BILX-001；DBX-007  
  交付：billing 回归结果记录；必要修复提交  
  完成定义：`payment_intent`、`payment_webhook_event`、订单支付结果编排在新栈下表现与基线一致。  
  验收：webhook 联调与 DB 回查通过；乱序回调不会回退订单主状态。  
  阻塞风险：支付路径是交易主链路的耦合点，回归不足会直接破坏订单状态权威。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L544-L555` | `docs/01-architecture/order-orchestration.md` | `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`  
  验证命令：  
  `cargo test -p platform-core trade024_illegal_state_regression_db_smoke -- --nocapture`  
  `cargo test -p platform-core trade030_payment_result_orchestrator_db_smoke -- --nocapture`  
  联调要求：  
  使用真实服务和临时种子数据，分别发送 `payment.succeeded`、`payment.failed`、`payment.timeout`、乱序 `payment.failed` 到 `POST /api/v1/payments/webhooks/mock_payment`；用 `psql` 回查 `payment.payment_intent`、`payment.payment_webhook_event`、`trade.order_main`、`audit.audit_event`。  
  完整测试：`cargo test -p platform-core`。  

## 4. CATX Catalog 读写仓储迁移

- **CATX-001** [AGENT][P0][W2][serial-first] 拆分 `catalog/repository.rs`，按 `command/query/shared` 重组目录与导出。  
  依赖：DBX-004；DBX-006  
  交付：`apps/platform-core/src/modules/catalog/repo/*`；`repository.rs` 的替代模块  
  完成定义：原超大仓储文件按读写职责拆分；API 支撑逻辑仍保留原接口行为。  
  验收：catalog 编译通过；无新增超大单文件；路由行为不变。  
  阻塞风险：不先拆分结构，后续 ORM 与 SQLx 组合会继续堆积到单文件。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L240-L272` | `plans/sqlx-seaorm-migration-implementation-plan.md:L359-L366` | `docs/开发准备/仓库拆分与目录结构建议.md:L173-L180`  
  验证命令：  
  `cargo fmt --all`  
  `cargo check -p platform-core`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **CATX-002** [AGENT][P0][W2][serial-first] 将 `catalog` 的稳定查询面迁移到 SeaORM，复杂查询保留 SQLx。  
  依赖：CATX-001  
  交付：`query/product_read.rs`；`query/seller_profile.rs`；`query/product_listing.rs`；`query/scenario_read.rs`  
  完成定义：`GET /api/v1/products/{id}`、`GET /api/v1/sellers/{orgId}/profile`、标准场景模板与列表类读查询完成迁移；DTO 保持不变。  
  验收：读接口字段与现有 OpenAPI 一致；必要场景可通过 SeaORM relation 或 SQLx 聚合实现。  
  阻塞风险：读模型如果处理不当，会造成接口字段顺序、空值语义或过滤逻辑回归。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L397-L413` | `plans/sqlx-seaorm-migration-implementation-plan.md:L557-L574` | `docs/02-openapi/catalog.yaml`  
  验证命令：  
  `cargo test -p platform-core cat020_read_db -- --nocapture`  
  `cargo test -p platform-core cat022_search_visibility_db -- --nocapture`  
  `cargo test -p platform-core cat023_standard_scenarios_db -- --nocapture`  
  联调要求：  
  `GET /api/v1/products/{id}`  
  `GET /api/v1/sellers/{orgId}/profile`  
  `GET /api/v1/catalog/standard-scenarios`  
  并使用 `psql` 回查 `catalog.product`、`catalog.product_sku`、`contract.template_definition`。  
  完整测试：`cargo test -p platform-core`。  

- **CATX-003** [AGENT][P0][W2][serial-first] 将 `catalog` 写路径、校验和 outbox 路径迁移到 SQLx，保留事务与规则语义。  
  依赖：CATX-001；DBX-005  
  交付：`command/product_write.rs`；`command/review_write.rs`；`command/template_binding_write.rs`；`api/support.rs`；`api/validators/*`  
  完成定义：产品草稿、提交流程、模板绑定、SKU 维护、数据契约、审核写路径全部使用 SQLx；校验逻辑不降级。  
  验收：写路径可继续生成审计和 outbox；所有现有 catalog 规则继续生效。  
  阻塞风险：catalog 写路径涉及模板/审核/契约约束，迁移不完整会影响后续 trade 门禁。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L270-L272` | `plans/sqlx-seaorm-migration-implementation-plan.md:L630-L637` | `docs/业务流程/业务流程图-V1-完整版.md`  
  验证命令：  
  `cargo test -p platform-core cat021_template_policy_db -- --nocapture`  
  `cargo test -p platform-core cat024_catalog_listing_review_db -- --nocapture`  
  `cargo test -p platform-core template_policy_db -- --nocapture`  
  联调要求：  
  `POST /api/v1/products`  
  `POST /api/v1/products/{id}/bind-template`  
  `POST /api/v1/products/{id}/submit`  
  `POST /api/v1/review/products/{id}`  
  `POST /api/v1/skus/{id}/data-contracts`  
  用 `psql` 回查 `catalog.product`、`catalog.product_sku`、`contract.template_binding`、`audit.audit_event`、`ops.outbox_event`。  
  完整测试：`cargo test -p platform-core`。  

- **CATX-004** [AGENT][P0][W2][serial-first] 执行 catalog 全量回归：读接口、审核流程、模板策略、标准场景、加工流程与 DB smoke。  
  依赖：CATX-002；CATX-003；DBX-007  
  交付：catalog 回归结果记录；必要修复提交  
  完成定义：catalog 已迁移路径在新栈下与当前行为一致；trade 依赖的商品/模板/审核语义不受影响。  
  验收：catalog 现有测试和真实 API 联调通过；核心读写接口可被 trade 继续消费。  
  阻塞风险：catalog 回归不足会把风险延迟到交易阶段才暴露。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L669-L677` | `docs/02-openapi/catalog.yaml` | `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`  
  验证命令：  
  `cargo test -p platform-core cat020_read_db -- --nocapture`  
  `cargo test -p platform-core cat021_template_policy_db -- --nocapture`  
  `cargo test -p platform-core cat022_search_visibility_db -- --nocapture`  
  `cargo test -p platform-core cat023_standard_scenarios_db -- --nocapture`  
  `cargo test -p platform-core cat024_catalog_listing_review_db -- --nocapture`  
  联调要求：  
  至少跑一套“建商品草稿 → 绑定模板 → 提交审核 → 审核通过 → 读取商品详情/卖家详情”的真实 API 流程，并用 `psql` 回查状态、模板、审计和 outbox。  
  完整测试：`cargo test -p platform-core`。  

## 5. IAMX IAM 访问层迁移

- **IAMX-001** [AGENT][P0][W3][serial-first] 将 `iam/api.rs` 与 `iam/repository.rs` 的数据库访问迁移到 `SeaORM + SQLx`，并拆分为 `repo/query/shared` 结构。  
  依赖：DBX-004；DBX-006  
  交付：`apps/platform-core/src/modules/iam/repo/*`；`apps/platform-core/src/modules/iam/api.rs`  
  完成定义：组织、部门、用户、应用、连接器、设备、会话、step-up 等读查询迁移；API 层不直接依赖 driver。  
  验收：IAM 模块编译通过；API 层不再出现 `tokio_postgres::{GenericClient NoTls Row}`。  
  阻塞风险：IAM 路径量大且接口多，迁移不做结构拆分会导致后续维护不可控。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L274-L290` | `plans/sqlx-seaorm-migration-implementation-plan.md:L368-L375` | `docs/权限设计/后端鉴权中间件规则说明.md`  
  验证命令：  
  `cargo fmt --all`  
  `cargo check -p platform-core`  
  `rg -n "tokio_postgres|NoTls|GenericClient|Row" apps/platform-core/src/modules/iam`  
  联调要求：无。  
  完整测试：`cargo test -p platform-core`。  

- **IAMX-002** [AGENT][P0][W3][serial-first] 执行 IAM 全量回归：组织、用户、会话、设备、访问检查、party linkage 与 ignored integration test。  
  依赖：IAMX-001；DBX-007  
  交付：IAM 回归结果记录；必要修复提交  
  完成定义：现有 IAM 接口在新栈下全部可用；party access 集成测试切换到 SQLx。  
  验收：`iam_party_access_integration` 与关键 API 联调通过。  
  阻塞风险：IAM 是上游基础能力，回归不足会影响 catalog/trade 权限与 scope 校验。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L576-L593` | `plans/sqlx-seaorm-migration-implementation-plan.md:L669-L677` | `docs/权限设计/接口权限校验清单.md`  
  验证命令：  
  `cargo test -p platform-core --test iam_party_access_integration -- --ignored --nocapture`  
  `cargo test -p platform-core iam`  
  联调要求：  
  `GET /api/v1/iam/orgs`  
  `POST /api/v1/iam/users`  
  `GET /api/v1/auth/me`  
  `POST /api/v1/iam/access/check`  
  用 `psql` 回查 IAM 相关表和审计。  
  完整测试：`cargo test -p platform-core`。  

## 6. DOCX 文档与运行手册同步

- **DOCX-001** [ARCH][P0][W3][serial-first] 同步技术选型、目录结构、启动手册和根 README，明确新数据库访问栈与验证方式。  
  依赖：ORDX-006；BILX-002；CATX-004；IAMX-002  
  交付：`docs/开发准备/技术选型正式版.md`；`docs/开发准备/仓库拆分与目录结构建议.md`；`docs/04-runbooks/local-startup.md`；`docs/README.md`  
  完成定义：文档口径同步到 `SQLx + SeaORM`；明确 migration 仍走 `db/scripts/*`；明确 `cargo sqlx prepare --workspace`。  
  验收：文档与代码实际结构一致；无旧驱动口径残留。  
  阻塞风险：代码迁了但文档不更新，会导致后续开发继续按旧驱动写新代码。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L687-L703` | `docs/开发准备/技术选型正式版.md:L22-L49` | `docs/开发准备/仓库拆分与目录结构建议.md:L173-L180`  
  验证命令：  
  `rg -n "tokio-postgres|tokio_postgres" docs apps/platform-core/crates/db`  
  `rg -n "SQLx|SeaORM|cargo sqlx prepare" docs`  
  联调要求：无。  
  完整测试：文档同步后执行 `cargo test -p platform-core`。  

## 7. CLNX 清理、收口与回合并

- **CLNX-001** [AGENT][P0][W4][serial-first] 删除所有残余 `tokio-postgres`，执行最终全量验证，并准备合并回 `v1core_dev`。  
  依赖：DOCX-001  
  交付：`Cargo.lock`；所有相关模块与测试；最终验证记录  
  完成定义：`apps/platform-core` 范围内无 `tokio-postgres` 依赖与引用；全量测试、DB smoke、API 联调、migration 验证通过；可回合并。  
  验收：`rg -n "tokio_postgres|tokio-postgres" apps/platform-core` 返回空；`git merge --no-ff sqlx-seaorm-migration` 前的主线验证完成。  
  阻塞风险：收口不彻底会留下双栈并存，后续维护成本比现在更高。  
  技术参考：`plans/sqlx-seaorm-migration-implementation-plan.md:L583-L593` | `plans/sqlx-seaorm-migration-implementation-plan.md:L639-L685` | `plans/sqlx-seaorm-migration-implementation-plan.md:L722-L747`  
  验证命令：  
  `cargo fmt --all`  
  `cargo test -p platform-core`  
  `cargo sqlx prepare --workspace`  
  `./db/scripts/migrate-reset.sh`  
  `./db/scripts/migrate-up.sh`  
  `./db/scripts/migrate-status.sh`  
  `./db/scripts/verify-migration-001.sh`  
  `./db/scripts/verify-migration-010-030.sh`  
  `./db/scripts/verify-migration-040-056.sh`  
  `./db/scripts/verify-migration-057-060.sh`  
  `./db/scripts/verify-migration-061-064.sh`  
  `./db/scripts/verify-migration-065-068.sh`  
  `./db/scripts/verify-migration-070.sh`  
  `rg -n "tokio_postgres|tokio-postgres" apps/platform-core`  
  联调要求：  
  启动完整服务后至少复跑一轮 `order + billing + catalog + iam` 的代表性 API 联调；包括 `POST /api/v1/orders`、`POST /api/v1/payments/webhooks/mock_payment`、`GET /api/v1/products/{id}`、`GET /api/v1/iam/orgs`，并用 `psql` 回查主表与审计。  
  完整测试：  
  `cargo test -p platform-core` 全量；模块级 DB smoke；`iam_party_access_integration`；最终 `git checkout v1core_dev && git merge --no-ff sqlx-seaorm-migration` 前后各跑一轮主验证。  
