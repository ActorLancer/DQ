# V1-Core 实施进度日志 P8

本文件是实施进度日志的当前续写分卷正文。

- 正式入口页：`docs/开发任务/V1-Core-实施进度日志.md`
- 当前活动分卷以入口页为准
- 若后续切换到新的 `P{N}` 分卷，必须先更新入口页，再开始续写新分卷

### BATCH-298（计划中）
- 任务：`TEST-001` 五条标准链路完整演示数据包
- 状态：计划中
- 说明：`fixtures/demo/` 当前缺失，而五条标准链路真值散落在 `fixtures/local/standard-scenarios-*`、`db/seeds/032_five_scenarios.sql`、`db/seeds/033_searchrec_recommendation_samples.sql`、`apps/platform-core/src/modules/catalog/standard_scenarios.rs` 与推荐首页固定样例配置中。当前批次先按冻结文档把主体、商品、SKU、模板、订单、交付对象、账单样例、审计样例收拢成正式 demo 数据包，并补齐可在本地/CI 运行的校验脚本，供后续 `TEST-002` seed importer、`TEST-006` E2E、`TEST-021/022/023` 验收文档与矩阵复用。
- 前置依赖核对结果：`ENV-040` 已在 `docs/开发任务/V1-Core-实施进度日志-P1.md` 完成并留痕，`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` 已作为本地环境 smoke 基线；`CORE-024` 已在 `docs/开发任务/V1-Core-实施进度日志-P1.md` 完成并提供 `platform-core` 基础测试夹具；`DB-032` 已在 `docs/开发任务/V1-Core-实施进度日志-P1.md` 完成并提供 migration/seed 兼容性回归链路，当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：定位 `TEST-001` 描述、依赖、DoD、技术参考与顺序约束。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核五条标准链路官方命名、`5.3.2A` 场景到主/补充 SKU 与合同/验收/退款模板映射。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认 `TEST` 阶段需把演示数据转化为可执行验收基线，不是一次性样例。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`、`docs/开发准备/测试用例矩阵正式版.md`：确认 `PostgreSQL` 为真值、前端只经 `platform-core` 正式 API、五条链路与 8 个标准 SKU 需保持统一真值源。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`search-rec-cases.md`、`audit-consistency-cases.md`、`canonical-event-authority-cases.md`：确认后续回归将直接复用五条场景和 8 个 SKU 的冻结口径。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/recommendation-runtime.md`、`scripts/README.md`、`fixtures/README.md`、`fixtures/local/README.md`：确认现有本地启动与推荐位仍依赖 `fixtures/local` 和 SQL 种子，`TEST-001` 需新增 `fixtures/demo` 正式包而不再把 local 样例误报为完整 demo 数据。
  - `packages/openapi/catalog.yaml`、`packages/openapi/trade.yaml` 及 `docs/02-openapi/catalog.yaml`、`docs/02-openapi/trade.yaml`：复核 `GET /api/v1/catalog/standard-scenarios`、订单模板/场景快照字段与 `scenario_code / primary_sku / supplementary_skus` 契约。
  - `db/seeds/010_test_tenants.sql`、`020_test_products.sql`、`030_test_orders.sql`、`032_five_scenarios.sql`、`033_searchrec_recommendation_samples.sql` 及 `db/scripts/verify-seed-032.sh`、`verify-seed-033.sh`：确认现有可复用的真实 UUID、主体、商品、订单、首页 `home_featured` 固定样例和场景注册表来源。
- 当前完成标准理解：
  - `fixtures/demo/` 成为五条标准链路的正式 demo 数据包目录，覆盖主体、商品、SKU、模板、订单、交付对象、账单样例、审计样例，并显式记录来源种子/表/ID，不再依赖零散 local 样例拼接理解。
  - demo 数据包可被机器校验，至少验证五条场景完整性、8 个标准 SKU 覆盖、场景与模板/订单/交付/搜索推荐映射一致，以及关键 ID/引用不漂移。
  - 校验脚本能在本地/CI 直接运行并输出可读结果，为后续 `seed-demo.sh`、E2E 与验收清单提供统一输入。
- 实施计划：
  1. 新建 `fixtures/demo/` 正式目录，定义 manifest、主体、目录商品、模板、订单、交付、账单、审计与按场景分组的 bundle 文件。
  2. 把五条标准链路与 8 个标准 SKU 的正式真值收敛到 demo 包，并记录对应 SQL seed / DB 表 /前端场景入口 / 推荐位固定样例来源。
  3. 新增 demo 包校验脚本，校验结构、引用、SKU 覆盖、固定场景顺序与来源一致性。
  4. 同步更新 `fixtures/README.md`、`fixtures/local/README.md`、`scripts/README.md` 等索引说明，明确 `fixtures/local` 是历史本地样例，`fixtures/demo` 才是 `TEST` 阶段正式数据包。

### BATCH-298（待审批）
- 任务：`TEST-001` 五条标准链路完整演示数据包
- 状态：待审批
- 当前任务编号：`TEST-001`
- 前置依赖核对结果：`ENV-040`、`CORE-024`、`DB-032` 已在 `docs/开发任务/V1-Core-实施进度日志-P1.md` 完成并作为当前批次基线输入继续复用；`smoke-local.sh`、`platform-core` 测试夹具与 migration/seed 兼容性验证入口均已存在，当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-001` 交付是正式 demo 数据包，不是沿用 `fixtures/local` 轻量样例。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核五条标准链路官方命名、`5.3.2A` 场景到主/补充 SKU 与模板映射。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认演示数据需要成为可执行验收基线。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`测试用例矩阵正式版.md`：确认 demo 数据包必须同时服务 `platform-core`、前端、搜索推荐与后续 outbox/通知/审计测试。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`search-rec-cases.md`、`audit-consistency-cases.md`、`canonical-event-authority-cases.md`：确认后续回归会直接消费五条标准链路和 8 个标准 SKU 的冻结口径。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/recommendation-runtime.md`、`scripts/README.md`、`fixtures/README.md`、`fixtures/local/README.md`：确认需要显式区分 `fixtures/local` 的 ENV bootstrap 样例和 `fixtures/demo` 的 TEST 正式数据包。
  - `packages/openapi/catalog.yaml`、`packages/openapi/trade.yaml`、`docs/02-openapi/catalog.yaml`、`docs/02-openapi/trade.yaml`：复核 `scenario_code / primary_sku / supplementary_skus` 契约与 `GET /api/v1/catalog/standard-scenarios` 口径。
  - `db/seeds/010_test_tenants.sql`、`031_sku_trigger_matrix.sql`、`032_five_scenarios.sql`、`033_searchrec_recommendation_samples.sql`、`apps/platform-core/src/modules/catalog/standard_scenarios.rs`、`apps/platform-core/src/modules/delivery/tests/fixtures/dlv026/manifest.json`：抽取当前可复用的主体、官方展示商品、SKU、billing trigger 和交付 fixture 真值。
- 实现要点：
  - 新增 `fixtures/demo/` 正式数据包目录：
    - `manifest.json`：定义文件清单、覆盖面、校验脚本与上游真值源。
    - `subjects.json`：沉淀卖方、买方、平台、零售卖方四类主体与用户 / 应用真值。
    - `catalog.json`：沉淀五条标准链路的官方展示商品、10 个展示 SKU、共享 seed 模板记录和 `home_featured` 固定样例顺序。
    - `orders.json`：为五条标准链路补齐 10 个交易蓝图（5 条主路径 + 5 条补充 SKU 路径），不再复用现有 seed 中一笔订单兼挂多个场景的模糊关系。
    - `delivery.json`：把 `DLV-026` 的正式 fixture 路径与 payload 快照收拢为可复用交付对象蓝图，覆盖 API、文件包、文件订阅、共享、沙箱、模板授权、查询结果、报告。
    - `billing.json`：把 `db/seeds/031_sku_trigger_matrix.sql` 的 8 个标准 SKU 计费触发矩阵与 10 个账单样例显式绑定。
    - `audit.json`：为每条标准链路冻结必须出现的审计动作、`audit.package.export` step-up 约束与 append-only 保留要求。
    - `scenarios.json`：输出五条标准链路 bundle，总览主体、商品、订单、交付、账单、审计与首页推荐顺序。
  - 新增 `scripts/check-demo-fixtures.mjs` 与 `scripts/check-demo-fixtures.sh`，校验：
    - 五条标准链路顺序与 `home_featured` 固定样例一致
    - 8 个标准 SKU 覆盖完整
    - 场景模板、官方展示商品、订单蓝图、交付/账单/审计引用一致
    - `apps/platform-core/src/modules/catalog/standard_scenarios.rs`、`db/seeds/033_searchrec_recommendation_samples.sql`、`fixtures/local/standard-scenarios-manifest.json` 等上游真值源仍存在且被正确引用
  - 更新索引与 runbook：
    - `fixtures/README.md`
    - `fixtures/local/README.md`
    - `scripts/README.md`
    - `docs/04-runbooks/local-startup.md`
    明确 `fixtures/demo` 是 `TEST` 阶段正式数据包入口，`fixtures/local` 继续保留为 ENV bootstrap 样例。
  - 新建 `docs/开发任务/V1-Core-实施进度日志-P8.md`，并把 `docs/开发任务/V1-Core-实施进度日志.md` 当前活动分卷切换到 `P8`，为 `TEST` 阶段留痕。
- 验证步骤：
  1. `node ./scripts/check-demo-fixtures.mjs`
  2. `bash ./scripts/check-demo-fixtures.sh`
  3. `cargo fmt --all`
  4. `cargo check -p platform-core`
  5. `cargo test -p platform-core`
  6. `cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
- 验证结果：
  - `node ./scripts/check-demo-fixtures.mjs` 与 `bash ./scripts/check-demo-fixtures.sh` 均通过，输出：
    - `5` 条标准链路
    - `10` 个官方展示 SKU
    - `10` 个订单蓝图
    - `11` 个交付蓝图
    - `10` 个账单样例
    - `5` 个审计套件
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 串行重跑后通过；首次并行执行时被 `cargo sqlx prepare --workspace` 的 cargo 锁干扰并触发离线查询缓存报错，改为串行后确认不是当前改动引入的回归。
  - `cargo test -p platform-core` 通过，`358` 个测试通过、`0` 失败、`1` ignored（仓库既有 live smoke 忽略项），仅存在既有 warning。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx` 编译期查询缓存已可重建。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-001`
  - `数据交易平台-全集成基线-V1.md`：`5.3.2`、`5.3.2A`
  - `15-测试策略、验收标准与实施里程碑.md`：`15.2`
  - `packages/openapi/catalog.yaml`、`packages/openapi/trade.yaml` 与 `docs/02-openapi/*`
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`search-rec-cases.md`、`audit-consistency-cases.md`
- 覆盖的任务清单条目：`TEST-001`
- 未覆盖项：
  - `TEST-002` 的 `seed-demo.sh` 导入器尚未实现；当前批次只交付正式 demo 数据包与其校验入口，不提前越界实现 importer。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
