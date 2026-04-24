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
  - 新增 non-blocking tech-debt：`TODO-TEST-003-001`，记录历史 `ErrorResponse` callsite 仍有“通用 fallback code + message 前缀业务码”的兼容路径；当前已由统一序列化层维持对外契约，后续再逐步回收调用点本身。

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

### BATCH-301（计划中）
- 任务：`TEST-004` 建立 migration smoke test，验证空库升级、种子导入、应用启动、重置回滚与重新升级
- 状态：计划中
- 说明：当前仓库已有 `db/scripts/verify-db-compatibility.sh`、`verify-migration-roundtrip.sh` 等 DB-032 资产，但它们只覆盖 migration/seed 兼容性，不覆盖 `platform-core` 启动；同时 `scripts/validate_database_migrations.sh` 仍指向历史 `部署脚本/docker-compose.postgres-test.yml` 方案，`db/scripts/*` 也残留旧的 `55432/luna_data_trading/luna` 默认值。当前批次将以冻结文档和现有 `infra/docker/.env.local` 为 authority，把 migration smoke 收口成正式 checker：统一复用当前 local core stack、执行空库升级/seed/回滚重升级、校验 seed history，并在最终升级后真实启动 `platform-core`、回查健康端点和运行态信息，再把该入口纳入 CI。
- 前置依赖核对结果：`ENV-040` 已提供本地 core stack 与 `infra/docker/.env.local` 运行基线；`DB-032` 已提供 migration/seed 兼容回归链路；`CORE-024` 已提供 `platform-core` 启动与健康端点骨架。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-004` 的正式交付是本地/CI 可重复运行的 migration smoke，而不是单独的 DB roundtrip 脚本。
  - `docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/README.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认 V1 migration 执行顺序、upgrade/downgrade 策略与 TEST 阶段“真实可重复 smoke”要求。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/平台总体架构设计草案.md`：确认 `platform-core` 是唯一主应用，运行时数据库入口统一走 `DATABASE_URL`，宿主机本地依赖口径统一为当前 `infra/docker/.env.local`。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/postgres-local.md`、`docs/05-test-cases/README.md`、`scripts/README.md`：确认 migration smoke 需要覆盖 core stack、bucket/topic 准备、`platform-core` 启动与正式脚本入口，不得继续沿用历史 compose/test 编排。
  - `db/scripts/migration-runner.sh`、`migrate-up.sh`、`migrate-down.sh`、`migrate-reset.sh`、`seed-runner.sh`、`seed-up.sh`、`verify-migration-roundtrip.sh`、`verify-db-compatibility.sh`、`check-db-ready.sh`：确认现有可复用的 migration/seed 兼容回归链路，以及旧默认数据库参数尚未收口到当前 local baseline。
  - `scripts/validate_database_migrations.sh`、`Makefile`、`xtask/src/main.rs`、`.github/workflows/*.yml`：确认当前迁移校验入口和 CI 仍缺失 `TEST-004` 的正式 smoke job。
- 当前完成标准理解：
  - 形成 `TEST-004` 专属 migration smoke checker，执行顺序至少覆盖：启动 core stack、初始化 MinIO buckets、空库 upgrade、全量 seed、回滚/重升级、seed history 回查、最终 `platform-core` 启动与健康/运行态断言。
  - `scripts/validate_database_migrations.sh` 收口到当前正式 smoke 入口，不再继续执行历史 `部署脚本/docker-compose.postgres-test.yml` 路径。
  - `db/scripts/*` 的默认本地数据库参数与 `infra/docker/.env.local` 对齐，避免 migration smoke 依赖隐式 export 才能通过。
  - 文档、fixtures 与 `.github/workflows/**` 同步补齐，确保本地与 CI 至少一处可重复通过并留下可读结果。
- 实施计划：
  1. 追加 `TEST-004` 基线文档与 fixture，明确 required seed versions、应用启动口径和 checker 命令。
  2. 新增 migration smoke checker，并让 `scripts/validate_database_migrations.sh` 兼容转发到该入口。
  3. 收口 `db/scripts/*` 默认本地数据库参数到 `infra/docker/.env.local` 当前口径，避免旧 `55432/luna_*` 漂移继续污染 smoke。
  4. 接入 GitHub Actions workflow，并执行本地真实验证：core stack、migration smoke、Rust 通用校验、`sqlx prepare`、query compile check。

### BATCH-301（待审批）
- 任务：`TEST-004` 建立 migration smoke test，验证空库升级、种子导入、应用启动、重置回滚与重新升级
- 状态：待审批
- 当前任务编号：`TEST-004`
- 前置依赖核对结果：`ENV-040`、`DB-032`、`CORE-024` 均继续满足；本批直接复用当前 `infra/docker/.env.local` core stack、`db/scripts/verify-db-compatibility.sh` 与 `platform-core` 健康端点，不再引入历史 test compose。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-004` 要求的是“migration smoke + app startup + CI”，不是单独的 DB compatibility。
  - `docs/数据库设计/数据库设计总说明.md`、`docs/数据库设计/README.md`、`db/migrations/v1/README.md`、`docs/03-db/migration-compatibility.md`：确认 upgrade/downgrade 顺序、manifest 入口、`verify-db-compatibility.sh` 覆盖边界与 current migration 最新版本来源。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/平台总体架构设计草案.md`：确认 `platform-core` 是唯一主应用，本地运行时入口以 `infra/docker/.env.local` 和 `DATABASE_URL` 为准。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/postgres-local.md`、`docs/05-test-cases/README.md`、`scripts/README.md`、`README.md`：确认 `TEST` 阶段需要正式 smoke/checker/runbook/CI 留痕，且历史 `部署脚本/docker-compose.postgres-test.yml` 已不应继续作为正式入口。
  - `db/scripts/migration-runner.sh`、`seed-runner.sh`、`migrate-reset.sh`、`verify-migration-roundtrip.sh`、`verify-db-compatibility.sh`、`check-db-ready.sh`、`scripts/validate_database_migrations.sh`、`xtask/src/main.rs`、`Makefile`、`.github/workflows/*.yml`：确认现有脚本可复用，但入口和默认数据库参数需要统一收口。
- 实现要点：
  - 新增 `scripts/check-migration-smoke.sh`，作为 `TEST-004` 正式 checker，完整执行：
    - 启动 current local core stack
    - 初始化 MinIO buckets
    - 运行 `db/scripts/verify-db-compatibility.sh`
    - 回查 `public.seed_history` 中 `001/010/020/030/031/032/033`
    - 最终启动 `platform-core-bin`
    - 回查 `/health/live`、`/health/ready`、`/health/deps`、`/internal/runtime`
  - 新增 `fixtures/smoke/test-004/required-seed-versions.txt` 与 `runtime-baseline.env`，冻结 `TEST-004` 的 seed history 和运行态基线；并新增 `fixtures/smoke/test-004/README.md` 说明。
  - 新增 `docs/05-test-cases/migration-smoke-cases.md`，并更新 `docs/05-test-cases/README.md`，把 `TEST-004` checker 与验收断言纳入正式测试文档。
  - 新增 `.github/workflows/migration-smoke.yml`，把 `TEST-004` 作为独立 CI job 落盘。
  - 把 `scripts/validate_database_migrations.sh` 收口为兼容 wrapper，统一转发到 `check-migration-smoke.sh`；同时更新 `README.md`、`scripts/README.md`、`docs/04-runbooks/postgres-local.md` 的正式入口说明。
  - 收口本地数据库默认参数漂移：
    - `db/scripts/*` 中原残留的 `55432/luna_data_trading/luna/5686` 默认值，统一改为当前 `5432/datab/datab/datab_local_pass`
    - `scripts/seed-demo.mjs`、`scripts/check-demo-seed.mjs` 同步对齐 current local baseline，避免 TEST 资产内部继续保留第二套本地数据库默认口径
- 验证步骤：
  1. `chmod +x scripts/check-migration-smoke.sh scripts/validate_database_migrations.sh`
  2. `bash -n scripts/check-migration-smoke.sh scripts/validate_database_migrations.sh db/scripts/verify-db-compatibility.sh db/scripts/verify-migration-roundtrip.sh`
  3. `ENV_FILE=infra/docker/.env.local bash ./scripts/validate_database_migrations.sh`
  4. `cargo fmt --all`
  5. `cargo check -p platform-core`
  6. `cargo test -p platform-core`
  7. `bash -lc 'set -a; source infra/docker/.env.local; set +a; cargo sqlx prepare --workspace'`
  8. `./scripts/check-query-compile.sh`
  9. `git diff --check`
- 验证结果：
  - `bash -n ...` 通过。
  - `ENV_FILE=infra/docker/.env.local bash ./scripts/validate_database_migrations.sh` 通过，完整输出：
    - core stack 启动、Kafka topics 初始化、MinIO buckets 初始化成功
    - `db compatibility baseline verified`
    - `required seed_history versions recorded`
    - `platform-core` 最终以 `APP_MODE=local / PROVIDER_MODE=mock` 启动成功
    - `/health/live`、`/health/ready` 返回 `200`
    - `/health/deps` 回查 `db / redis / kafka / minio / keycloak reachable=true`
    - `/internal/runtime` 回查 `migration_version=083`
    - `TEST-004 migration smoke checker passed`
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `cargo test -p platform-core` 通过：`358` 个测试通过、`0` 失败、`1` ignored（仓库既有 `iam_party_access_flow_live`）。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx` 编译期查询缓存已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - `git diff --check` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-004`
  - `数据库设计总说明.md`：7.1 V1 migration 顺序
  - `数据库设计/README.md`：4. 迁移策略
  - `15-测试策略、验收标准与实施里程碑.md`：15.1 测试策略
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/postgres-local.md`
  - `docs/05-test-cases/README.md`
- 覆盖的任务清单条目：`TEST-004`
- 未覆盖项：
  - `TEST-005` 的 local stack smoke、Grafana/realm/canonical topic 边界仍由下一批单独处理；本批只覆盖 migration/seed/app startup smoke，不提前越界到 observability 或 canonical checker。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-302（计划中）
- 任务：`TEST-005` 建立本地环境 smoke test，验证 compose 启动、核心服务 ready、Grafana 数据源可连、Keycloak realm 导入成功，并校验 canonical topics 与关键控制面入口不再回退到旧口径
- 状态：计划中
- 说明：当前仓库已有 `scripts/check-local-stack.sh`、`scripts/check-keycloak-realm.sh`、`scripts/check-observability-stack.sh`、`scripts/check-topic-topology.sh` 与 `scripts/smoke-local.sh`，但它们仍是分散校验：`smoke-local.sh` 不负责 compose 启动、Grafana 只校验登录、不启动宿主机 `platform-core`，也没有把 `127.0.0.1:9094` / `kafka:9092|localhost:9092` 的双地址边界和活跃 test-case 一起冻结。当前批次将把 `smoke-local.sh` 收口成 `TEST-005` 正式 checker：真实拉起 `core + observability + mocks`、启动或复用宿主机 `platform-core:8094`、回查 realm/datasource/topic/控制面入口，并同步补齐 smoke case、fixture 与 CI。
- 前置依赖核对结果：`ENV-040` 已提供正式 `infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`up-local/check-local-stack` 基线；`DB-032` 已通过 `TEST-004` 验证 migration/seed/app startup smoke；`CORE-024` 已提供 `platform-core` 健康端点与运行态入口。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-005` 正式交付是本地环境 smoke，而不是复用旧 `ENV` 阶段 healthcheck；宿主机/容器 Kafka 双地址边界必须写入公共前置条件与活跃 test-case。
  - `docs/开发准备/技术选型正式版.md`、`docs/原始PRD/日志、可观测性与告警设计.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认本地环境正式观测栈为 `Prometheus/Alertmanager/Grafana/Loki/Tempo`，TEST 阶段 smoke 必须真实触达 compose、观测与控制面入口。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`、`docs/开发准备/测试用例矩阵正式版.md`、`docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`、`docs/开发准备/平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `PostgreSQL` 仍是业务真值，Kafka 不是真相源，宿主机进程默认 Kafka 必须走 `127.0.0.1:9094`，compose 网络内部继续走 `kafka:9092` / `localhost:9092`，且 smoke 需要真实覆盖 Keycloak / Grafana / topic / ops 入口。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`notification-cases.md`、`search-rec-cases.md`、`audit-consistency-cases.md`、`canonical-event-authority-cases.md`：确认 `smoke-local.sh` 只是 local/canonical 验收基线的一部分，不能误报为全量业务闭环；通知、搜索、审计等 test-case 也都统一使用宿主机 `127.0.0.1:9094`。
  - `docs/04-runbooks/local-startup.md`、`port-matrix.md`、`kafka-topics.md`、`keycloak-local.md`、`observability-local.md`、`compose-boundary.md`、`mock-payment.md`、`troubleshooting.md`：确认 current local startup、Kafka dual-listener、realm import、Grafana datasource 与 compose 边界的正式 runbook 口径。
  - `infra/kafka/topics.v1.json`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/docker/monitoring/prometheus.yml`：确认 canonical topic catalog、compose profile、Kafka advertised listeners、Prometheus 抓取 `host.docker.internal:8094` 的真实入口。
  - `scripts/README.md`、`scripts/up-local.sh`、`scripts/check-local-env.sh`、`scripts/check-local-stack.sh`、`scripts/verify-local-stack.sh`、`scripts/check-keycloak-realm.sh`、`scripts/check-observability-stack.sh`、`scripts/check-topic-topology.sh`、`scripts/smoke-local.sh`：确认现有脚本可复用，但 smoke 仍缺少 compose 启动、platform-core 启动、Grafana datasource / ops 入口联动与固定的 Kafka 边界校验。
  - `.github/workflows/*.yml`、`.github/workflows/README.md`、`fixtures/local/*.json`：确认 `TEST-005` 仍缺少正式 CI workflow，且现有 local smoke / keycloak fixture 口径存在老的前置条件与角色命名残留。
- 当前完成标准理解：
  - `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` 成为 `TEST-005` 正式 checker，会真实启动 `core + observability + mocks`、topic init、MinIO buckets 与宿主机 `platform-core`，并输出可定位的失败信息。
  - smoke 至少回查：`check-local-stack full`、`check-keycloak-realm.sh`、`check-observability-stack.sh`、`check-topic-topology.sh`、全量 canonical topic 存在、`/health/live|ready|deps`、`/internal/runtime`、`/api/v1/ops/observability/overview`、`/api/v1/ops/outbox` 等关键控制面入口。
  - 宿主机 Kafka 边界固定为 `127.0.0.1:9094`，容器内 / compose 网络边界固定为 `kafka:9092` 与容器内 `localhost:9092`；脚本、fixture、runbook、活跃 smoke case 与 CI 同步对齐，不再保留 `localhost:9094` 或 host-side `localhost:9092` 漂移默认值。
  - GitHub Actions 补齐独立 `TEST-005` smoke workflow，并在文档中明确 `check-topic-topology.sh` 与 `smoke-local.sh` 的职责边界。
- 实施计划：
  1. 升级 `scripts/smoke-local.sh` 为 `TEST-005` 正式 checker：整合 compose 启动、MinIO/topic/bootstrap、宿主机 `platform-core` 启动、realm/datasource/topic/ops 入口回查与 Kafka 双地址校验。
  2. 收口残留默认值与 fixture：修正 `platform-core` host-side Kafka 默认值、`docker-compose` 的 external advertised host fallback、`fixtures/local` 中过时的 smoke / keycloak baseline，并新增 `fixtures/smoke/test-005/**`。
  3. 新增 `docs/05-test-cases/local-environment-smoke-cases.md`，更新 `docs/05-test-cases/README.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/troubleshooting.md`、`scripts/README.md`、`.github/workflows/README.md`。
  4. 新增 `.github/workflows/local-environment-smoke.yml`，再执行本地真实验证：`smoke-local.sh`、Rust 通用校验、`sqlx prepare`、query compile check。

### BATCH-302（待审批）
- 任务：`TEST-005` 建立本地环境 smoke test，验证 compose 启动、核心服务 ready、Grafana 数据源可连、Keycloak realm 导入成功，并校验 canonical topics 与关键控制面入口不再回退到旧口径
- 状态：待审批
- 当前任务编号：`TEST-005`
- 前置依赖核对结果：`ENV-040`、`DB-032`、`CORE-024` 继续满足；当前批次直接复用 `infra/docker/.env.local`、`infra/docker/docker-compose.local.yml`、`TEST-004` 的 migration/seed/app startup 基线，以及 `platform-core` 现有健康端点与 ops 入口。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-005` 的正式输出是可重复运行的本地环境 smoke checker，且需显式校验宿主机/容器 Kafka 双地址边界与关键控制面入口。
  - `docs/开发准备/技术选型正式版.md`、`docs/原始PRD/日志、可观测性与告警设计.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认本地 smoke 必须真实覆盖 `Prometheus / Alertmanager / Grafana / Loki / Tempo`、Keycloak、mock payment 与 `platform-core`，不能只做页面或端口探活。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`测试用例矩阵正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`平台总体架构设计草案.md`、`数据交易平台-全集成基线-V1.md`：确认 `PostgreSQL` 仍是业务真值，宿主机 Kafka 默认必须是 `127.0.0.1:9094`，compose 网络内部继续使用 `kafka:9092` / `localhost:9092`，且 smoke 需要覆盖 canonical topics、realm import、Grafana datasource 和正式 ops 入口。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`notification-cases.md`、`search-rec-cases.md`、`audit-consistency-cases.md`、`canonical-event-authority-cases.md`：确认 `smoke-local.sh` 只是 `TEST` 阶段 local baseline，不得误报为全量业务验收；通知、搜索、审计等后续 test-case 将继续复用统一 Kafka host/container 边界。
  - `docs/04-runbooks/local-startup.md`、`port-matrix.md`、`kafka-topics.md`、`keycloak-local.md`、`observability-local.md`、`compose-boundary.md`、`mock-payment.md`、`troubleshooting.md`：确认 current local startup、Prometheus 抓取 host app、realm import 与 Kafka dual-listener 的正式 runbook 口径。
  - `infra/kafka/topics.v1.json`、`infra/docker/docker-compose.local.yml`、`infra/docker/.env.local`、`infra/docker/monitoring/prometheus.yml`：确认 canonical topic catalog、compose profiles、Prometheus host scrape 目标与 Kafka advertised listeners 的 current authority。
  - `scripts/README.md`、`scripts/up-local.sh`、`scripts/check-local-env.sh`、`scripts/check-local-stack.sh`、`scripts/check-keycloak-realm.sh`、`scripts/check-observability-stack.sh`、`scripts/check-topic-topology.sh`、`scripts/smoke-local.sh`、`.github/workflows/*.yml`、`fixtures/local/*.json`：确认现有脚本与 fixture 可复用，但需要正式收口为单一 `TEST-005` checker 并补 CI。
- 实现要点：
  - 重写 `scripts/smoke-local.sh` 为 `TEST-005` 正式 checker：
    - 统一加载 `infra/docker/.env.local` 与 `fixtures/smoke/test-005/runtime-baseline.env`
    - 启动 `core + observability + mocks` compose profile
    - 执行 `db/scripts/migrate-up.sh`、`db/scripts/seed-up.sh` 与 `infra/minio/init-minio.sh`
    - 启动或复用宿主机 `platform-core`，绑定 `APP_HOST=0.0.0.0`，对外检查入口固定为 `http://127.0.0.1:8094`
    - 真实回查 `check-local-stack full`、数据库 migration probe、MinIO buckets、Keycloak realm、`/health/live|ready|deps`、`/internal/runtime`、`check-topic-topology.sh`、全量 canonical topic 存在、Grafana/Prometheus/Alertmanager/mock payment，以及 `ops/observability/overview` 和 `ops/outbox`
    - 显式校验 Kafka 双地址边界：宿主机 `127.0.0.1:9094`，容器/compose 网络 `kafka:9092` 与容器内 `localhost:9092`
  - 收口 host-side 默认值漂移：
    - `apps/platform-core/src/lib.rs`、`apps/platform-core/crates/http/src/lib.rs`、对应测试、`infra/docker/docker-compose.local.yml`、`infra/kafka/docker-compose.kafka.local.yml`
    - 去掉 host-side `localhost:9092/9094` 残留，统一对齐 `127.0.0.1:9094`
  - 修正 smoke fixture 与控制面基线：
    - 更新 `fixtures/local/local-smoke-suite-manifest.json`
    - 更新 `fixtures/local/keycloak-realm-manifest.json`
    - 新增 `fixtures/smoke/test-005/README.md`
    - 新增 `fixtures/smoke/test-005/runtime-baseline.env`
    - 新增 `fixtures/smoke/test-005/required-control-plane-endpoints.json`
    - 去掉会触发审计 FK 失败的伪 `x-user-id`，统一由 `platform_audit_security` 角色检查 ops 入口
  - 新增与更新文档/CI：
    - 新增 `docs/05-test-cases/local-environment-smoke-cases.md`
    - 更新 `docs/05-test-cases/README.md`
    - 更新 `docs/04-runbooks/local-startup.md`
    - 更新 `docs/04-runbooks/troubleshooting.md`
    - 更新 `scripts/README.md`
    - 更新 `.github/workflows/README.md`
    - 新增 `.github/workflows/local-environment-smoke.yml`
- 验证步骤：
  1. `ENV_FILE=infra/docker/.env.local bash ./scripts/smoke-local.sh`
  2. `cargo fmt --all`
  3. `cargo check -p platform-core`
  4. `cargo test -p platform-core`
  5. `bash -lc 'set -a; source infra/docker/.env.local; set +a; cargo sqlx prepare --workspace'`
  6. `./scripts/check-query-compile.sh`
- 验证结果：
  - `ENV_FILE=infra/docker/.env.local bash ./scripts/smoke-local.sh` 通过，真实完成并回查：
    - compose 启动 `core + observability + mocks`
    - PostgreSQL migration probe
    - `db/scripts/seed-up.sh` 基线导入
    - MinIO buckets 初始化与对象健康检查
    - Keycloak realm 导入与 password grant
    - `platform-core` `/health/live`、`/health/ready`、`/health/deps`、`/internal/runtime`
    - `check-topic-topology.sh`
    - `infra/kafka/topics.v1.json` 中 `required_in_smoke=true` 的 canonical topics 存在
    - Kafka host/container 双地址边界：`127.0.0.1:9094`、`kafka:9092`、`localhost:9092`
    - `check-observability-stack.sh`
    - `check-mock-payment.sh`
    - `http://127.0.0.1:8081/realms/platform-local/.well-known/openid-configuration`
    - `Prometheus /-/ready`
    - `Alertmanager /-/ready`
    - `Grafana /api/health`
    - `GET /api/v1/ops/observability/overview`
    - `GET /api/v1/ops/outbox?page=1&page_size=1&target_topic=dtp.notification.dispatch`
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；仅输出仓库既有 warning，本批改动未新增编译错误。
  - `cargo test -p platform-core` 通过：`358` 个测试通过、`0` 失败、`1` ignored（仓库既有 live smoke 忽略项）。
  - `cargo sqlx prepare --workspace` 通过；workspace `.sqlx` 查询元数据可重建，本批未引入额外 `.sqlx` 漂移。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-005`
  - `技术选型正式版.md`
  - `日志、可观测性与告警设计.md`
  - `15-测试策略、验收标准与实施里程碑.md`
  - `docs/04-runbooks/local-startup.md`
  - `docs/04-runbooks/port-matrix.md`
  - `docs/04-runbooks/kafka-topics.md`
  - `docs/04-runbooks/keycloak-local.md`
  - `docs/04-runbooks/observability-local.md`
  - `docs/05-test-cases/README.md`
- 覆盖的任务清单条目：`TEST-005`
- 未覆盖项：
  - `TEST-005` 只负责本地环境 smoke 基线，不替代后续 `TEST-006+` 的五条标准链路 E2E、通知闭环、搜索推荐闭环、审计回放与 compose 级全量验收。
  - `check-topic-topology.sh` 仍只承担通知 / Fabric / audit-anchor 相关静态 topology 与 route seed 校验；若需验证 `infra/kafka/topics.v1.json` 全量 canonical topics 是否真实存在，仍以 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` 为正式入口。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-300（计划中）
- 任务：`TEST-003` 建立 contract test 目录，覆盖 OpenAPI schema、错误码、状态机枚举、关键响应字段
- 状态：计划中
- 说明：在按 `TEST-003` 阅读冻结文档与现有实现后，确认仓库当前仍存在两类外部契约漂移：一是成功响应仍混用 `success + data` 与 `data.data` 旧 envelope；二是对外逻辑字段仍暴露 `amount / order_status / units` 等持久化命名。用户已明确要求本批次选择 `A/A`，即以冻结文档为唯一 authority 先做一次正式契约回收，再建立 contract baseline / checker，避免把当前漂移固化成“正式基线”。
- 前置依赖核对结果：`ENV-040`、`DB-032`、`CORE-024` 已在前序分卷完成并作为当前批次基线复用；`TEST-001` 演示数据包与 `TEST-002` demo importer 已完成并留痕，可继续作为 contract fixture 与后续验收链路输入，当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-003` 的正式交付是 contract test 目录与 checker，范围覆盖 OpenAPI schema、错误码、状态机枚举、关键响应字段，且必须先满足 `depends_on=ENV-040;DB-032;CORE-024`。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认成功响应统一为 `code/message/request_id/data`、失败响应统一为 `code/message/request_id/details`，并明确对外逻辑字段应使用 `current_state / order_amount / metered_quantity`，不得直接暴露 `status / amount / units`。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：复核 `9.1.4` 字段映射要求与 `current_state -> trade.order_main.status`、`order_amount -> trade.order_main.amount`、`metered_quantity -> billing.billing_event.units` 的冻结口径。
  - `docs/开发准备/统一错误码字典正式版.md`、`docs/05-test-cases/order-state-machine.md`、`docs/05-test-cases/README.md`、`docs/页面说明书/页面说明书-V1-完整版.md`：确认 contract checker 需要同时约束错误码前缀、订单状态机枚举与前端/后端共享的关键响应字段，不得只校验 schema 文件存在。
  - `docs/data_trading_blockchain_system_design_split/12-API 设计、事件模型与消息总线.md`、`15-测试策略、验收标准与实施里程碑.md`：确认 `TEST` 阶段 contract 检查是冻结契约回归，不是现状快照。
  - `apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/crates/kernel/src/lib.rs`：确认当前运行时成功 envelope 仍是 `ApiResponse { success, data }`，失败 envelope 缺少 `details`，但请求上下文中已有 `request_id` / `trace_id` 可复用，不需要再发明第二套请求标识。
  - `apps/platform-core/src/modules/order/api/handlers.rs`、`dto/order_read.rs`、`repo/order_read_repository.rs`、`tests/trade004_order_detail_db.rs`：确认 Trade 订单详情仍使用 `GetOrderDetailResponse { data }` 双层包装，且对外字段仍暴露 `amount`，测试仍断言 `json["data"]["data"]`。
  - `apps/platform-core/src/modules/billing/billing_read_handlers.rs`、`models.rs`、`repo/billing_read_repository.rs`、`packages/openapi/billing.yaml`：确认 Billing 对外详情仍暴露 `order_status`，样例仍暴露 `units`。
  - `packages/openapi/catalog.yaml`、`trade.yaml`、`billing.yaml` 及 `docs/02-openapi/*`：确认多份 OpenAPI 仍用 `required: [success, data]` 或保留 `data.data` 旧包装，需在本批次统一回收。
  - `scripts/check-openapi-schema.sh`、`scripts/check-canonical-contracts.sh`、`.github/workflows/canonical-contracts.yml`、`.github/workflows/test.yml`：确认仓库已有 OpenAPI / canonical checker 入口可复用，但在正式 checker 落地前必须先让公共响应封装、OpenAPI 和测试回到冻结口径。
- 当前完成标准理解：
  - `platform-core` 成功响应统一输出 `code/message/request_id/data`，失败响应统一输出 `code/message/request_id/details`；仓库内不再把 `success + data` 或 `data.data` 视为正式外部契约。
  - Trade / Billing 等对外 DTO、OpenAPI 与测试统一回收到冻结字段名：`current_state / order_amount / metered_quantity`；数据库字段命名可继续保持内部使用，但不得直接外露。
  - 新建 `TEST-003` contract fixtures / checker / 文档与 CI 接入，能机器化拒绝旧 envelope、错误字段名、错误码/状态机漂移。
- 实施计划：
  1. 回收公共响应封装与失败 envelope，补齐 `request_id`、`details` 等冻结字段，并修复受影响 handler / 测试。
  2. 回收 Trade / Billing 等对外 DTO、OpenAPI 与示例，统一移除 `data.data` 与 `amount / order_status / units` 漂移字段。
  3. 新增 contract fixtures、checker、文档与 CI 接入，锁定 OpenAPI schema、错误码、状态机枚举与关键响应字段的正式基线。
  4. 运行 `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 以及本批次新增的 contract / OpenAPI checker，完成留痕与提交。

### BATCH-300（待审批）
- 任务：`TEST-003` 建立 contract test 目录，覆盖 OpenAPI schema、错误码、状态机枚举、关键响应字段
- 状态：待审批
- 当前任务编号：`TEST-003`
- 前置依赖核对结果：`ENV-040`、`DB-032`、`CORE-024` 继续满足；`TEST-001/002` 已完成并可复用为当前批次的 demo / contract fixture 输入。
- authority 决策留痕：
  - 用户已明确确认 `Q-TEST-003-01=A`、`Q-TEST-003-02=A`：`TEST-003` 以冻结文档为唯一 authority，不允许把当前漂移的 `success + data` / `data.data` / `amount` / `order_status` / `units` 固化成正式 baseline。
  - 本批次因此先做一次“契约回收”，再落 checker / fixture / CI。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-003` 正式交付物为 contract test 目录与可复用 checker，覆盖 OpenAPI schema、错误码、状态机枚举、关键响应字段。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认统一成功/失败 envelope 以及 `current_state / order_amount / metered_quantity` 的冻结映射。
  - `docs/开发准备/统一错误码字典正式版.md`、`docs/05-test-cases/order-state-machine.md`、`docs/05-test-cases/README.md`：确认 `TEST-003` 需要同时覆盖错误码基线、订单状态机 action enum 与禁止错误码绑定，且不得与 `TEST-028` 的 canonical checker 混淆。
  - `packages/openapi/*.yaml`、`docs/02-openapi/*.yaml`、`apps/platform-core/crates/http/src/lib.rs`、`apps/platform-core/crates/kernel/src/lib.rs`、`apps/platform-core/src/modules/order/**`、`apps/platform-core/src/modules/billing/**`、现有 DB smoke：确认旧 envelope 与旧字段名在运行时、OpenAPI 和测试中均有残留，需要统一回收。
  - `scripts/check-openapi-schema.sh`、`.github/workflows/canonical-contracts.yml`、`.github/workflows/test.yml`、`scripts/README.md`：确认现有脚本与 CI 仍未提供 `TEST-003` 专属 contract baseline checker。
- 实现要点：
  - 回收公共响应 envelope：
    - `apps/platform-core/crates/http/src/lib.rs`：成功响应统一序列化为 `code/message/request_id/data`，并自动消除旧 `data.data` 包装。
    - `apps/platform-core/crates/kernel/src/lib.rs`：失败响应统一序列化为 `code/message/request_id/details`；请求上下文统一透传 `request_id`；若错误消息前缀已带正式业务错误码，则优先作为对外 `code` 发出，减少运行时与冻结错误码字典的偏差。
  - 回收对外字段名：
    - Trade DTO / repository / OpenAPI：`amount -> order_amount`、`status/order_status -> current_state`、`units -> metered_quantity`。
    - Billing view / repository / OpenAPI：`order_status -> current_state`、`units -> metered_quantity`、`OrderLock.order_status -> current_state`。
    - 同步修正所有受影响 DB smoke 断言，移除 `json["data"]["data"]`、`json["success"]` 与旧字段名断言。
  - 新增 `TEST-003` 正式 contract baseline：
    - `fixtures/contracts/test-003/README.md`
    - `fixtures/contracts/test-003/key-response-fields.tsv`
    - `fixtures/contracts/test-003/error-code-baseline.tsv`
    - `fixtures/contracts/test-003/state-machine-contracts.tsv`
    - `scripts/check-api-contract-baseline.sh`
    - `.github/workflows/contract-tests.yml`
  - 更新索引文档：
    - `scripts/README.md`
    - `docs/05-test-cases/README.md`
    - `docs/02-openapi/*.yaml` 与 `packages/openapi/*.yaml` 重新逐字同步。
  - 补充 runtime 断言：
    - `bil003_order_lock_db_smoke` 增加 `current_state` 与统一 success envelope 断言。
    - `bil007_billing_read_db_smoke` 增加 `current_state / order_amount / billing_events[0].metered_quantity` 与统一 success envelope 断言。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `bash ./scripts/check-api-contract-baseline.sh`
  4. `cargo test -p platform-core`
  5. `./scripts/check-query-compile.sh`
  6. `bash -lc 'set -a; source infra/docker/.env.local; set +a; cargo sqlx prepare --workspace'`
  7. `cargo test -p platform-core bil003_order_lock_db_smoke -- --nocapture`
  8. `cargo test -p platform-core bil007_billing_read_db_smoke -- --nocapture`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `bash ./scripts/check-api-contract-baseline.sh` 通过，输出：
    - `success envelopes aligned`
    - `error envelopes aligned`
    - `key response fields aligned`
    - `error-code baseline aligned`
    - `state-machine contract baseline aligned`
  - `cargo test -p platform-core` 通过：`358` 个测试通过，`0` 失败，另有仓库既有 `iam_party_access_flow_live` 继续保持 ignored。
  - `./scripts/check-query-compile.sh` 通过。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx` 查询编译元数据已重建并更新。
  - `cargo test -p platform-core bil003_order_lock_db_smoke -- --nocapture` 通过。
  - `cargo test -p platform-core bil007_billing_read_db_smoke -- --nocapture` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-003`
  - `接口清单与OpenAPI-Schema冻结表.md`
  - `数据交易平台-全集成基线-V1.md`
  - `统一错误码字典正式版.md`
  - `docs/05-test-cases/order-state-machine.md`
  - `docs/05-test-cases/README.md`
- 覆盖的任务清单条目：`TEST-003`
- 未覆盖项：
  - `TEST-028` 的 canonical topic / topology / smoke checker 仍由既有 `./scripts/check-canonical-contracts.sh` 负责，本批次未改动其职责边界。
  - 统一错误码字典在若干历史 runtime callsite 仍存在“消息前缀携带业务码、原始 fallback code 保留通用码”的兼容路径；本批通过统一序列化层优先发出消息前缀中的正式业务错误码，未在本任务中逐一重写全部历史调用点。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
