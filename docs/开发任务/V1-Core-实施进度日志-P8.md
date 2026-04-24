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
