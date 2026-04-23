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

### BATCH-299（计划中）
- 任务：`TEST-002` 一键导入 demo 数据
- 状态：计划中
- 说明：`scripts/seed-demo.sh` 仍停留在 `BOOT-004` 占位实现，当前本地库虽然 schema 已齐备，但 `core/catalog/trade/payment/delivery` 全为空，无法支撑 `TEST` 阶段的五条标准链路复现。当前批次将把 `fixtures/demo/` 从静态数据包升级为真实 importer 输入：先执行正式 seed manifest，补齐演示租户、演示用户、演示商品，再按 `orders.json / billing.json / delivery.json` 导入 10 笔 demo 订单、支付记录与交付对象，并新增可重复回查的 checker，确保脚本在 fresh DB 与重复执行场景下都可读、可定位、可复用。
- 前置依赖核对结果：`ENV-040` 已提供本地 PostgreSQL / Kafka / Redis / OpenSearch / MinIO / Keycloak 联调基线；`DB-032` 已提供 migration / seed 兼容回归链路；`CORE-024` 已提供 `platform-core` 基础测试夹具。当前任务依赖满足，可在现有本地数据库上继续推进。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：定位 `TEST-002` 的交付物、依赖、DoD 与顺序约束。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核五条标准链路与 `5.3.2A` SKU/模板映射，确认 importer 不得引入第二套场景命名。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认 demo seed 必须服务可执行验收，而不是一次性样例。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`测试用例矩阵正式版.md`：确认 `PostgreSQL` 仍是权威源，后续搜索/推荐/通知/E2E 都要复用这套 demo 数据，不允许在脚本里再发明 SKU 或 provider 口径。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`：确认 demo importer 至少要落到订单、支付、交付三域真实记录，供后续 state machine / webhook / delivery / acceptance 回归复用。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/mock-payment.md`、`scripts/README.md`、`fixtures/demo/README.md`：确认 `seed-demo.sh` 应在正式 `seed-up` 基线之上工作，并在本地/CI 给出可读输出与回查入口。
  - `docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`020_catalog_contract.sql`、`030_trade_delivery.sql`、`040_billing_support_risk.sql`、`061_data_object_trade_modes.sql`、`065_query_execution_plane.sql`、`071_trade_order_layered_status.sql`：核对 `core.organization / user_account / application`、`catalog.product / product_sku / asset_object_binding / query_surface_definition`、`trade.order_main / order_line / authorization_grant`、`payment.payment_intent / payment_transaction / payment_webhook_event / billing.billing_event`、`delivery.api_credential / revision_subscription / data_share_grant / sandbox_workspace / template_query_grant / query_execution_run / report_artifact / delivery_record / delivery_ticket` 的正式字段。
  - `db/scripts/seed-runner.sh`、`db/scripts/seed-up.sh`、`db/seeds/manifest.csv`、`db/seeds/010_test_tenants.sql`、`032_five_scenarios.sql`、`033_searchrec_recommendation_samples.sql`：确认现有正式 seed manifest 可提供演示主体、应用、五条官方场景商品与 `home_featured` 推荐位，`TEST-002` 只负责在该基线之上追加 demo 订单 / 支付 / 交付对象。
  - `fixtures/demo/subjects.json`、`catalog.json`、`orders.json`、`billing.json`、`delivery.json`：确认 importer 的唯一输入真值与 10 笔订单、10 条 billing sample、11 条 delivery blueprint 的引用关系。
- 当前完成标准理解：
  - `scripts/seed-demo.sh` 变为真实 importer，而不是占位输出；支持一键导入演示租户、演示用户、演示商品、演示订单、演示支付与交付记录。
  - importer 优先复用 `db/seeds/manifest.csv` 的正式 seed 基线，不在脚本中复制 `org/user/product/sku/template` 真值；额外 demo 订单、支付、交付对象直接从 `fixtures/demo/` 读取。
  - importer 需支持重复执行，不因 rerun 产生重复业务对象；并提供 checker / 回查命令确认订单、支付、交付三域记录真实落库。
  - 当前批次要顺手修正 `fixtures/demo/billing.json` 中与正式支付 provider key 不一致的口径，使其与 `payment.provider(provider_key='mock_payment')`、支付 runbook 与现有实现一致。
- 实施计划：
  1. 在 `P8` 留痕后，重写 `scripts/seed-demo.sh`，让其调用新的 importer，并支持 `--dry-run`、`--skip-base-seeds`、`--no-verify` 等基础参数。
  2. 新增 importer，读取 `fixtures/demo/*.json`，先执行 `db/scripts/seed-up.sh`，再按官方五场景商品映射写入 demo 订单、支付意图/交易/webhook/billing event 和交付对象。
  3. 新增 `check-demo-seed` checker，回查 demo 组织、商品、订单、支付、交付计数与关键关系，验证 importer 可重复执行。
  4. 更新 `fixtures/demo/README.md`、`scripts/README.md`、`docs/04-runbooks/local-startup.md`，把 `seed-demo.sh` 的正式入口、参数和回查方式写入索引。

### BATCH-299（待审批）
- 任务：`TEST-002` 一键导入 demo 数据
- 状态：待审批
- 当前任务编号：`TEST-002`
- 前置依赖核对结果：`ENV-040`、`DB-032`、`CORE-024` 均已在前序分卷完成并继续复用；本地 PostgreSQL schema 已可用，`db/scripts/seed-up.sh` 与 `db/seeds/manifest.csv` 可作为当前批次正式 seed 基线。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-002` 交付物是 `seed-demo.sh` 正式 importer，而不是 README 或占位脚本。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核五条标准链路、`5.3.2A` SKU/模板映射与 8 个标准 SKU 的正式挂点。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认演示 seed 需要直接服务 TEST 阶段验收回归。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`测试用例矩阵正式版.md`：确认 importer 需要继续以正式 DB/服务边界为准，不自造第二套场景真值。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`：确认 demo importer 至少要为订单、支付、交付三域提供真实可回查的基线对象。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/mock-payment.md`、`scripts/README.md`、`fixtures/demo/README.md`：确认 `seed-demo.sh` 的正式入口、前置命令与回查方式。
  - `docs/数据库设计/V1/upgrade/010_identity_and_access.sql`、`020_catalog_contract.sql`、`030_trade_delivery.sql`、`040_billing_support_risk.sql`、`061_data_object_trade_modes.sql`、`065_query_execution_plane.sql`、`071_trade_order_layered_status.sql`：核对 demo importer 目标表与必填字段。
  - `db/scripts/seed-runner.sh`、`db/scripts/seed-up.sh`、`db/seeds/manifest.csv`、`db/seeds/010_test_tenants.sql`、`032_five_scenarios.sql`、`033_searchrec_recommendation_samples.sql`：确认正式主体 / 商品 / SKU / 场景商品必须先走已有 seed manifest，不在 importer 里重复定义。
  - `fixtures/demo/subjects.json`、`catalog.json`、`orders.json`、`billing.json`、`delivery.json`：确认 importer 的唯一输入真值、订单 / 支付 / 交付蓝图引用与 ID。
- 实现要点：
  - 重写 `scripts/seed-demo.sh`，改为正式 shell wrapper，统一调用新的 `scripts/seed-demo.mjs`。
  - 新增 `scripts/seed-demo.mjs`：
    - 支持 `--manifest`、`--skip-base-seeds`、`--dry-run`、`--no-verify`
    - 默认先执行 `db/scripts/seed-up.sh`
    - 校验 demo 主体 / 应用 / 五条官方场景商品与正式 `payment.provider(provider_key='mock_payment')` 是否齐备
    - 从 `fixtures/demo/orders.json / billing.json / delivery.json` 读取 10 笔 demo 订单、10 笔 payment intent、10 笔 payment transaction、10 笔 payment webhook、10 笔 billing event、11 笔 delivery record，并补齐 `api_credential / api_usage_log / sandbox_workspace / sandbox_session / data_share_grant / revision_subscription / template_query_grant / query_execution_run / report_artifact / delivery_ticket / storage_object / asset_object_binding / query_surface_definition / query_template_definition`
    - 将 demo bundle checksum 写入 `public.seed_history(version='demo-v1-core-standard-scenarios')`
  - 新增 `scripts/check-demo-seed.mjs` 与 `scripts/check-demo-seed.sh`，回查：
    - 4 个组织、6 个用户、1 个应用
    - 5 个官方场景商品
    - 10 笔 demo 订单
    - 10/10/10/10 笔 payment intent / transaction / webhook / billing event
    - 11 笔 delivery record
    - 以及 `api_credential / api_usage_log / sandbox_workspace / sandbox_session / data_share_grant / revision_subscription / template_query_grant / query_execution_run / report_artifact / delivery_ticket / storage_object / asset_object_binding / query_surface_definition / query_template_definition / seed_history`
  - 修正 `fixtures/demo/billing.json` 的 demo provider key 口径，从错误的服务名 `mock-payment-provider` 收回正式 `payment.provider.provider_key='mock_payment'`，避免 demo 包与正式支付实现漂移。
  - 更新 `fixtures/demo/README.md`、`scripts/README.md`、`docs/04-runbooks/local-startup.md`，冻结 `seed-demo.sh` 与 `check-demo-seed.sh` 的正式入口和用法。
- 验证步骤：
  1. `./scripts/seed-demo.sh --dry-run`
  2. `./scripts/seed-demo.sh`
  3. `./scripts/seed-demo.sh --skip-base-seeds`
  4. `./scripts/check-demo-fixtures.sh`
  5. `./scripts/check-demo-seed.sh`
  6. `cargo fmt --all`
  7. `cargo check -p platform-core`
  8. `cargo test -p platform-core`
  9. `cargo sqlx prepare --workspace`
  10. `./scripts/check-query-compile.sh`
- 验证结果：
  - `./scripts/seed-demo.sh --dry-run` 通过，输出正式导入计划：`4` 组织、`6` 用户、`1` 应用、`5` 商品、`10` SKU、`10` 订单、`10` payment sample、`11` delivery blueprint。
  - `./scripts/seed-demo.sh` 通过，真实落库并自动执行 `check-demo-seed`，回查结果：
    - `4` 组织
    - `6` 用户
    - `1` 应用
    - `5` 官方场景商品
    - `10` 笔 demo 订单
    - `10/10/10/10` 笔 payment intent / transaction / webhook / billing event
    - `11` 笔 delivery record
    - `3` 个 API credential
    - `3` 条 API usage log
    - `1/1/1/1` 个 sandbox workspace / share grant / revision subscription / query run
    - `4` 个 storage object
    - `4` 个 delivery ticket
    - `2` 个 report artifact
    - `1` 个 template query grant
    - `1` 条 `public.seed_history(version='demo-v1-core-standard-scenarios')`
  - `./scripts/seed-demo.sh --skip-base-seeds` 再次通过，重复执行后 demo 目标对象计数保持 `10` 订单 / `10` payment intent / `11` delivery record，不发生膨胀，验证 importer 具备可重复执行性。
  - `./scripts/check-demo-fixtures.sh` 通过。
  - `./scripts/check-demo-seed.sh` 通过。
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core` 通过：`358` 个测试通过、`0` 失败、`0` ignored；另有 `iam_party_access_flow_live` 维持仓库既有 ignored。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx` 编译期查询缓存已可重建。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-002`
  - `数据交易平台-全集成基线-V1.md`：`5.3.2`、`5.3.2A`
  - `15-测试策略、验收标准与实施里程碑.md`：`15.2`
  - `docs/数据库设计/V1/upgrade/010/020/030/040/061/065/071`
  - `docs/04-runbooks/local-startup.md`
  - `docs/05-test-cases/README.md`、`delivery-cases.md`、`payment-billing-cases.md`
- 覆盖的任务清单条目：`TEST-002`
- 未覆盖项：
  - `TEST-003` 的 contract test 目录与 OpenAPI / 错误码 / 状态机 checker 尚未开始，本批只完成正式 demo importer 与回查入口。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
