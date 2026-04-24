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

### BATCH-303（计划中）
- 任务：`TEST-006` 建立订单端到端测试，严格按五条标准链路命名与验收：工业设备运行指标 API 订阅、工业质量与产线日报文件包交付、供应链协同查询沙箱、零售门店经营分析 API / 报告订阅、商圈/门店选址查询服务
- 状态：计划中
- 说明：当前仓库已有 `apps/portal-web/e2e/web018-live.spec.ts`、`docs/05-test-cases/web-smoke-cases.md`、`fixtures/demo/*.json`、`apps/platform-core/src/modules/delivery/tests/fixtures/dlv026/**` 和 `trade027 / dlv025` 等局部资产，但 live E2E 仍只真实打通单条门户链路与单条控制台链路，且没有把五条标准链路逐条收口成 `TEST-006` 官方 checker、验收文档与 CI。当前批次将以 `fixtures/demo/scenarios.json + orders.json` 为唯一场景真值源，新增五条标准链路 order E2E 基线：真实 Keycloak 登录、门户搜索/商品详情/下单/订单详情/场景交付页、后端 order detail / lifecycle / trace 回查，以及本地/CI 可重复运行的脚本与 workflow。
- 前置依赖核对结果：`ENV-040` 已提供本地 PostgreSQL / Kafka / Redis / OpenSearch / MinIO / Keycloak / Mock Payment / observability 联调基线；`DB-032` 已通过 `TEST-004`/`TEST-005` 验证 migration/seed/local smoke；`CORE-024` 已提供 `platform-core` 主 API 与测试夹具。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-006` 的正式交付是五条标准链路 order E2E，不得停留在单条 live 示例或静态 smoke。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `5.3.2` 五条标准链路官方命名，以及 `5.3.2A` 到 `API_SUB / API_PPU / FILE_STD / FILE_SUB / SBX_STD / SHARE_RO / QRY_LITE / RPT_STD` 的正式映射。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认 `TEST` 阶段需要把链路演示固化为可重复验收，而不是局部 smoke。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`事件模型与Topic清单正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`测试用例矩阵正式版.md`：确认 `PostgreSQL` 是真值源，浏览器不得直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`，订单 E2E 至少应覆盖 `CORE-003 ~ CORE-008` 的搜索/详情/下单/详情/交付入口与请求边界。
  - `docs/05-test-cases/README.md`、`web-smoke-cases.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`：确认现有 `WEB-018` live E2E 只覆盖单条链路；`TEST-006` 需要把五条标准链路逐条命名、逐条验收，并继续遵守宿主机 `127.0.0.1:9094` 与受控浏览器边界。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认门户链路至少涉及首页、搜索、商品详情、下单、订单详情、场景对应交付页和验收页的正式命名与权限提示。
  - `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/mock-payment.md`、`scripts/README.md`：确认 `TEST-006` 应复用 `smoke-local.sh`、`seed-demo.sh`、`check-demo-seed.sh`、`check-keycloak-realm.sh` 的正式入口，不自造临时环境脚本。
  - `fixtures/demo/README.md`、`fixtures/demo/scenarios.json`、`fixtures/demo/orders.json`、`fixtures/demo/delivery.json`、`apps/platform-core/src/modules/delivery/tests/fixtures/dlv026/manifest.json`：确认五条标准链路的商品、SKU、订单蓝图和场景交付入口已有正式真值，可直接作为 E2E 数据基线。
  - `apps/portal-web/playwright.config.ts`、`apps/portal-web/e2e/smoke-live.spec.ts`、`apps/portal-web/e2e/web018-live.spec.ts`、`apps/portal-web/src/lib/standard-demo.ts`、`apps/portal-web/src/lib/order-workflow.ts`、`apps/portal-web/src/lib/delivery-workflow.ts`：确认门户已有 live helper 和五场景引导骨架，但还没有五条标准链路逐条执行的 order E2E。
- 当前完成标准理解：
  - 形成 `TEST-006` 专属 order E2E checker，可在本地和 CI 重复运行，逐条覆盖 `S1 ~ S5` 五条标准链路的真实前后端联动。
  - 每条链路至少要真实经过：Keycloak / IAM 登录、门户首页或场景页、搜索、商品详情、下单、订单详情、场景对应交付页，并回查后端 order detail / lifecycle 或 trace 证据。
  - 测试命名、场景名、SKU 映射、产品/订单 fixture 必须完全复用 `fixtures/demo/*.json` 与冻结文档，不自创第二套场景矩阵。
  - 浏览器请求必须继续满足 `restrictedRequests=[]`，所有真实 API 都通过 `portal-web -> /api/platform -> platform-core`。
- 实施计划：
  1. 追加 `TEST-006` 文档与官方 checker，冻结五条标准链路的 order E2E 执行入口、断言和失败回查方式。
  2. 新增门户 live E2E spec / helper，按 `S1 ~ S5` 逐条执行真实搜索、详情、下单、订单详情和场景交付页访问，并补充后端回查。
  3. 新增 CI workflow，把 `smoke-local + seed-demo + TEST-006 checker` 串成最小可重复链路。
  4. 执行本地真实验证：`smoke-local`、`seed-demo`、新的 order E2E checker，以及 Rust/TS 通用校验，完成留痕与提交。

### BATCH-303（待审批）
- 任务：`TEST-006` 建立订单端到端测试，严格按五条标准链路命名与验收：工业设备运行指标 API 订阅、工业质量与产线日报文件包交付、供应链协同查询沙箱、零售门店经营分析 API / 报告订阅、商圈/门店选址查询服务
- 状态：待审批
- 当前任务编号：`TEST-006`
- 前置依赖核对结果：`ENV-040` 已提供本地 PostgreSQL / Kafka / Redis / OpenSearch / MinIO / Keycloak / Mock Payment / observability 联调基线；`DB-032` 已通过 `TEST-004`/`TEST-005` 验证 migration/seed/local smoke；`CORE-024` 已提供 `platform-core` 主 API 与测试夹具。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-006` 的正式交付是五条标准链路 order E2E，不得停留在单条 live 示例。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：复核五条标准链路官方命名、`5.3.2A` SKU / 模板映射与 `TEST` 阶段“可重复验收”要求。
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`docs/页面说明书/页面说明书-V1-完整版.md`：确认门户链路至少覆盖首页、搜索、商品详情、下单、订单详情、交付页、验收页与权限态。
  - `docs/04-runbooks/local-startup.md`、`scripts/README.md`、`fixtures/demo/*.json`、`apps/portal-web/e2e/web018-live.spec.ts`、`apps/portal-web/src/lib/standard-demo.ts`：确认当前可复用正式入口、demo 真值源与 live E2E 基线。
- 实现要点：
  - 新增 `apps/portal-web/e2e/test006-standard-order-live.spec.ts`，按 `S1 ~ S5` 逐条执行真实 Keycloak password grant、门户首页 / 标准场景页、搜索、商品详情、下单页、订单详情页、交付页、验收页，并回查 `GET /api/v1/orders/{id}`、`GET /api/v1/orders/{id}/lifecycle-snapshots`、`GET /api/v1/developer/trace?order_id={id}`。
  - 新增 `scripts/check-order-e2e.sh` 与 `.github/workflows/order-e2e.yml`，把 `smoke-local.sh`、`seed-local-iam-test-identities.sh`、`seed-demo.sh`、`check-demo-seed.sh` 和门户 live E2E 串成 `TEST-006` 正式 checker / CI 入口。
  - 新增 `docs/05-test-cases/order-e2e-cases.md`，冻结五条标准链路执行路径、回查 API、宿主机边界与正式命令；同步更新 `docs/05-test-cases/README.md`、`scripts/README.md`、`.github/workflows/README.md`。
  - 修正门户和 fixture 对接中的正式口径漂移：
    - `scripts/seed-demo.mjs` 生成完整合法的 `price_snapshot_json`，消除 demo 订单详情因缺失 `product_id` 触发的 `TRD_STATE_CONFLICT`。
    - `apps/portal-web/src/lib/standard-demo.ts` 搜索词收回到五条标准链路官方全名，避免本地 PG fallback 因短词命不中而误报无结果。
    - `apps/portal-web/src/lib/order-workflow.ts`、`delivery-workflow.ts`、`acceptance-workflow.ts` 补齐对正式统一 envelope 的解包兼容，消除 `data.data` 残留导致的订单详情空态。
    - 交付页断言按正式权限矩阵收口：`local-buyer-operator` 在 `S2(FILE_STD)` 显式验证 `主按钮权限不足`，而不是误断言文件交付可执行表单；验收页标题改为精确匹配，避免与加载态标题冲突。
- 验证步骤：
  1. `pnpm --filter @datab/portal-web typecheck`
  2. `pnpm --filter @datab/portal-web lint`
  3. `WEB_E2E_LIVE=1 WEB_E2E_PORTAL_USERNAME=local-buyer-operator WEB_E2E_PORTAL_PASSWORD=LocalBuyerOperator123! WEB_E2E_TRACE_USERNAME=local-tenant-developer WEB_E2E_TRACE_PASSWORD=LocalTenantDeveloper123! PLATFORM_CORE_BASE_URL=http://127.0.0.1:8094 pnpm --filter @datab/portal-web test:e2e:orders-live`
  4. `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh`
  5. `pnpm --filter @datab/portal-web build`
  6. `cargo fmt --all`
  7. `cargo check -p platform-core`
  8. `cargo test -p platform-core`
  9. `bash -lc 'set -a; source infra/docker/.env.local; source fixtures/smoke/test-005/runtime-baseline.env; set +a; cargo sqlx prepare --workspace'`
  10. `./scripts/check-query-compile.sh`
- 验证结果：
  - `pnpm --filter @datab/portal-web typecheck`、`lint` 通过。
  - 门户 live E2E 通过：`5 passed (17.1s)`，五条标准链路均真实经过页面跳转与后端 `order detail / lifecycle / developer trace` 回查。
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh` 通过：复用 `TEST-005` 本地 smoke、完成 Keycloak 本地身份对齐、demo importer / checker 回查，并再次跑通 `5 passed (21.9s)` 的门户 live E2E。
  - `pnpm --filter @datab/portal-web build` 通过，Next.js 正式构建成功并生成订单 / 交付 / 验收相关动态路由。
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；存在仓库既有 unused/dead-code warning，但无新增编译失败。
  - `cargo test -p platform-core` 通过：`358` 个测试通过、`0` 失败、`1` ignored（仓库既有 live smoke 忽略项）。
  - `cargo sqlx prepare --workspace` 通过，workspace `.sqlx` 编译期查询缓存可重建。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-006`
  - `数据交易平台-全集成基线-V1.md`：`5.3.2`、`5.3.2A`
  - `15-测试策略、验收标准与实施里程碑.md`：`15.2`
  - `docs/05-test-cases/README.md`、`order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`
  - `docs/页面说明书/页面说明书-V1-完整版.md`
- 覆盖的任务清单条目：`TEST-006`
- 未覆盖项：
  - `TEST-023` 的 `8` 个标准 SKU 主路径 / 异常路径 / 退款争议矩阵不在本批范围，后续按任务顺序单独补齐。
  - `TEST-022` 的五条标准链路独立验收文档将在后续任务继续展开；本批先落 `TEST-006` 的执行基线与 checker。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-304（计划中）
- 任务：`TEST-007` 建立 Provider 切换测试：mock 支付 / mock 链写 / mock 签章 与 real 占位实现的切换不改业务代码
- 状态：计划中
- 说明：当前仓库已有 `docs/04-runbooks/provider-switch.md`、`provider-kit` mock/real 占位实现、`platform-core` 的 `PROVIDER_MODE=mock|real` 开关，以及 `fabric-adapter` 的 `FABRIC_ADAPTER_PROVIDER_MODE=mock|fabric-test-network` 配置与 live smoke，但这些能力分散在单元测试、runbook 和脚本里，还没有形成 `TEST-007` 官方 checker / 文档 / CI。当前批次将把支付、签章、链写三类 provider 切换收口成正式测试：验证切换通过配置完成，不改业务代码，并留下本地与 CI 可重复运行的结果。
- 前置依赖核对结果：`ENV-040` 已提供本地 `mock-payment-provider`、Fabric 测试网络与运行基线；`DB-032` 已提供可重复 migration/seed/DB smoke 基础；`CORE-024` 已提供 `platform-core` 运行骨架与真实 DB 测试夹具。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-007` 的正式交付是 provider 切换测试，不是单独 runbook 或局部单元测试。
  - `docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/开发准备/技术选型正式版.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认所有外部能力必须走可审计集成层，本地默认 mock，联调模式接真实/测试依赖。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/04-runbooks/provider-switch.md`：确认 V1 必须存在统一 Provider 适配层，每个 Provider 至少提供 `mock` 与 `real` 两套实现，运行环境只通过配置切换。
  - `apps/platform-core/crates/provider-kit/src/lib.rs`、`tests.rs`、`apps/platform-core/crates/config/src/lib.rs`、`apps/platform-core/src/modules/contract/application/signing.rs`：确认 `platform-core` 已具备 `PROVIDER_MODE=mock|real`、`FF_REAL_PROVIDER` 门控和签章 / 支付 / 链写的 mock/real 占位实现。
  - `services/fabric-adapter/internal/config/config.go`、`internal/provider/factory.go`、`internal/provider/live_smoke_test.go`、`scripts/fabric-adapter-live-smoke.sh`：确认链写 provider 的正式切换口径是 `mock|fabric-test-network`，且已有真实 Fabric live smoke 可复用。
  - `apps/platform-core/src/modules/order/tests/trade026_contract_signing_provider_db.rs`、`apps/platform-core/src/modules/billing/tests/bil004_mock_payment_adapter_db.rs`、`scripts/check-mock-payment.sh`：确认签章和 mock 支付已有真实 DB/live 资产，但尚未收口成统一 provider-switch checker。
- 当前完成标准理解：
  - 需要形成 `TEST-007` 专属 checker / 文档 / CI 入口，覆盖支付、签章、链写三类 provider 切换。
  - 切换结果必须由配置体现，而不是改业务代码或改测试代码路径；至少验证 `platform-core`、`mock-payment-provider`、`fabric-adapter` 三个正式联动对象。
  - `mock` 与 `real`/`fabric-test-network` 切换都要留下可读证据：provider kind / mode、返回字段、live smoke 或 DB 回查结果。
- 实施计划：
  1. 新增 `TEST-007` 文档和官方 checker，冻结支付 / 签章 / 链写三类 provider 的切换断言与执行入口。
  2. 调整或补充现有测试，使 `platform-core` 签章 provider 能在 `mock/real` 下复用同一业务路径完成断言，并复用 `provider-kit` 与 Fabric live smoke 资产。
  3. 新增 CI workflow，把 `mock-payment` live adapter、`platform-core` provider switch smoke、`fabric-adapter` mock / fabric-test-network 切换验证串成最小矩阵。
  4. 执行本地真实验证、更新 `P8` 待审批日志并提交。

### BATCH-304（待审批）
- 任务：`TEST-007` 建立 Provider 切换测试：mock 支付 / mock 链写 / mock 签章 与 real 占位实现的切换不改业务代码
- 状态：待审批
- 当前任务编号：`TEST-007`
- 前置依赖核对结果：`ENV-040` 已提供本地 `mock-payment-provider`、Fabric 测试网络与运行基线；`DB-032` 已通过 `TEST-004`/`TEST-005` 验证 migration/seed/local smoke；`CORE-024` 已提供 `platform-core` 运行骨架、`provider-kit` 适配层和真实 DB 测试夹具。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-007` 的正式交付是 provider 切换测试，不是局部 runbook 或单元测试集合。
  - `docs/原始PRD/链上链下技术架构与能力边界稿.md`、`docs/开发准备/技术选型正式版.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`：确认支付、签章、链写都必须通过统一 Provider 适配层，环境切换只能通过配置完成。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`、`docs/04-runbooks/provider-switch.md`：确认 V1 需要 `mock` 与 `real`/测试网络双实现，并要求保留本地 smoke 与联调入口。
  - `apps/platform-core/crates/provider-kit/src/lib.rs`、`tests.rs`、`apps/platform-core/crates/config/src/lib.rs`、`apps/platform-core/src/lib.rs`、`apps/platform-core/src/modules/contract/application/signing.rs`：确认 `platform-core` 已具备 `PROVIDER_MODE=mock|real`、`FF_REAL_PROVIDER` 门控，以及签章 / 支付 / 链写 provider 抽象。
  - `services/fabric-adapter/internal/config/config.go`、`internal/provider/factory.go`、`internal/provider/live_smoke_test.go`、`scripts/fabric-adapter-test.sh`、`scripts/fabric-adapter-live-smoke.sh`：确认 Fabric 链写 provider 的正式切换口径与可复用 live smoke 入口。
  - `apps/platform-core/src/modules/order/tests/trade026_contract_signing_provider_db.rs`、`apps/platform-core/src/modules/billing/tests/bil004_mock_payment_adapter_db.rs`、`scripts/check-mock-payment.sh`：确认签章和支付已有真实 DB / live 资产，可收口为统一 `TEST-007` checker。
- 实现要点：
  - 新增 `scripts/check-provider-switch.sh`，把 `smoke-local.sh`、`check-mock-payment.sh`、`provider-kit` mock/real 入口测试、`platform-core` 启动门控与签章 DB smoke、`fabric-adapter` 单元测试和 Fabric live smoke 串成 `TEST-007` 官方 checker。
  - 新增 `docs/05-test-cases/provider-switch-cases.md`，冻结支付 / 签章 / 链写三类 provider 的切换矩阵、正式断言、宿主机边界与执行命令；同步更新 `docs/05-test-cases/README.md`、`docs/04-runbooks/provider-switch.md`、`scripts/README.md`、`.github/workflows/README.md`。
  - 新增 `.github/workflows/provider-switch.yml`，在 CI 中执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`，并在结束后统一下线本地 compose 栈与 Fabric 测试网络。
  - 新增 `services/fabric-adapter/internal/provider/factory_test.go`，补齐 Fabric provider factory 的 `mock` 选择与非法 provider mode 拒绝校验。
  - 调整 `apps/platform-core/src/modules/order/tests/trade026_contract_signing_provider_db.rs`，使同一合同确认业务路径可在 `PROVIDER_MODE=mock` 与 `PROVIDER_MODE=real FF_REAL_PROVIDER=true` 下复用并校验 `signature_provider_mode / signature_provider_kind / signature_provider_ref`。
  - 在 `apps/platform-core/src/lib.rs` 增加 `startup_self_check` 的 real-provider 门控测试，确认未启用 `FF_REAL_PROVIDER` 时拒绝启动，启用后允许通过。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`
  4. `cargo test -p platform-core`
  5. `bash -lc 'set -a; source infra/docker/.env.local; source fixtures/smoke/test-005/runtime-baseline.env; set +a; cargo sqlx prepare --workspace'`
  6. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过。
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh` 通过，完成以下真实联动验证：
    - `smoke-local.sh` 校验本地 PostgreSQL / Kafka / Redis / OpenSearch / MinIO / Keycloak / Mock Payment / observability 基线。
    - `check-mock-payment.sh` 校验 mock payment 的 `success / fail / timeout` 三条正式路径。
    - `provider-kit` 测试验证支付、签章、链写 provider 的 mock/real 入口；其中 `live_mock_payment_adapter_hits_three_mock_paths` 在 live 模式下命中真实 mock payment 服务。
    - `platform-core` 启动门控测试验证 `PROVIDER_MODE=real` 必须绑定 `FF_REAL_PROVIDER=true`。
    - `trade026_contract_signing_provider_db_smoke` 在 `PROVIDER_MODE=mock` 与 `PROVIDER_MODE=real` 下复用同一业务路径完成合同确认，并分别回写 `mock-*` / `real-*` provider 证据。
    - `fabric-adapter-test.sh`、`check-fabric-local.sh`、`fabric-adapter-live-smoke.sh` 验证链写 provider 能在 `mock` 与 `fabric-test-network` 间切换，并留下真实 Fabric receipt / ledger 证据。
  - `cargo test -p platform-core` 通过：`360` 个测试通过，`0` 失败；仓库既有 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，workspace 根目录 `.sqlx` 查询编译元数据已重建并更新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-007`
  - `链上链下技术架构与能力边界稿.md`
  - `技术选型正式版.md`
  - `数据交易平台-全集成基线-V1.md`
  - `docs/04-runbooks/provider-switch.md`
  - `docs/05-test-cases/README.md`
- 覆盖的任务清单条目：`TEST-007`
- 未覆盖项：
  - `TEST-012` 的 webhook 幂等 / 乱序保护、`TEST-018` 的审计回放 dry-run、`TEST-021`/`TEST-028` 的 canonical checker 不在本批范围，后续按任务顺序继续补齐。
  - real payment / real signing 的外部生产 provider 仍为占位实现；本批验证的是配置切换与业务路径无代码分叉，不包含真实第三方厂商联调。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-305（计划中）
- 任务：`TEST-008` 建立 outbox 一致性测试：DB 事务成功时有 outbox，事务失败时无 outbox，重复消费不重复产生副作用
- 状态：计划中
- 说明：当前仓库已经分别具备 `trade003_create_order_db_smoke` 的成功写 outbox 断言、`workers/outbox-publisher` 的发布 / dead letter live smoke、以及 `notification-worker` 的 duplicate dedupe live smoke，但这些资产仍分散在不同模块和文档里，尚未形成 `TEST-008` 官方 checker / 文档 / CI，也没有把“事务失败无 outbox”收口成正式断言。本批次将以 `PostgreSQL` 为主状态权威、`Kafka` 为事件总线、`ops.consumer_idempotency_record` 为消费幂等证据，把事务写入、发布、消费三段闭环整理成单一可重复入口。
- 前置依赖核对结果：`ENV-040` 已提供 PostgreSQL / Kafka / Redis / Keycloak / OpenSearch / MinIO / observability 的本地联调基线；`DB-032` 已通过 `TEST-004`/`TEST-005` 验证 migration 与本地 smoke；`CORE-024` 已提供 `platform-core` canonical outbox writer、`outbox-publisher` 与 `notification-worker` 运行骨架。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-008` 的正式交付是 outbox 一致性测试，不是局部 DB 断言或 worker 单测。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`：确认 `ops.outbox_event` / `ops.dead_letter_event` / `ops.consumer_idempotency_record` 的正式字段、双层权威边界以及“不允许把 Kafka 当主状态机 / 不允许重试造成重复副作用”。
  - `docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/测试用例矩阵正式版.md`：确认 `platform-core` 事务内必须同写主对象 + 审计 + outbox，`outbox-publisher` 是正式发布者，`notification-worker` 是正式消费者之一，`ASYNC-001 ~ 003` 是冻结测试面。
  - `docs/04-runbooks/outbox-publisher.md`、`docs/04-runbooks/audit-ops-outbox-dead-letters.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/notification-cases.md`：确认当前已有 outbox publish / dead letter / consumer idempotency 的正式回查方式，可复用到 `TEST-008` checker。
  - `apps/platform-core/src/shared/outbox.rs`、`apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`、`trade006_contract_confirm_db.rs`、`workers/outbox-publisher/src/main.rs`、`apps/notification-worker/src/main.rs`：确认现有成功样本、发布侧 smoke 与 duplicate dedupe live smoke 的实现位置。
- 当前完成标准理解：
  - 必须至少证明三条正式断言：
    1. 业务事务成功时，主对象 / 审计 / outbox 同事务存在。
    2. 业务事务失败时，请求不得留下脏 outbox 或成功审计副作用。
    3. 重复消费正式事件时，不得重复产生下游副作用，并能从 `ops.consumer_idempotency_record` / 审计 / 系统日志回查。
  - 需要形成 `TEST-008` 专属文档、checker 和 CI 入口，且要复用正式服务与正式 topic，不新造第二套事件目录或模拟协议。
- 实施计划：
  1. 补强 `platform-core` 失败路径测试，明确断言冲突事务不会留下 outbox / 成功审计 / 主状态推进副作用。
  2. 新增 `TEST-008` 官方文档与 checker，串联 `trade003` 成功样本、失败无 outbox 样本、`outbox-publisher` live smoke 和 `notification-worker` duplicate dedupe live smoke。
  3. 新增最小 CI workflow，保证 `TEST-008` 在 GitHub Actions 上可重复执行。
  4. 执行真实验证、回写 `P8` 待审批记录并提交，然后继续 `TEST-009`。

### BATCH-305（待审批）
- 任务：`TEST-008` 建立 outbox 一致性测试：DB 事务成功时有 outbox，事务失败时无 outbox，重复消费不重复产生副作用
- 状态：待审批
- 当前任务编号：`TEST-008`
- 前置依赖核对结果：`ENV-040` 已提供 PostgreSQL / Kafka / Redis / Keycloak / OpenSearch / MinIO / observability 的本地联调基线；`DB-032` 已通过 `TEST-004`/`TEST-005` 验证 migration 与本地 smoke；`CORE-024` 已提供 `platform-core` canonical outbox writer、`outbox-publisher` 与 `notification-worker` 正式运行骨架。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-008` 的正式交付是 outbox 一致性测试，不是单条 DB count 或局部 worker 单测。
  - `docs/原始PRD/双层权威模型与链上链下一致性设计.md`：确认 `ops.outbox_event / ops.dead_letter_event / ops.consumer_idempotency_record` 的正式字段，以及“不允许把 Kafka 当主状态机 / 不允许重试造成重复副作用”的边界。
  - `docs/开发准备/事件模型与Topic清单正式版.md`、`docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/测试用例矩阵正式版.md`：确认事务内必须同写主对象 + 审计 + outbox，`outbox-publisher` 是正式发布者，`notification-worker` 是正式消费者，`ASYNC-001 ~ 003` 是冻结测试面。
  - `docs/04-runbooks/outbox-publisher.md`、`docs/04-runbooks/audit-ops-outbox-dead-letters.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/05-test-cases/notification-cases.md`：确认发布、死信和 consumer 幂等的正式回查方式。
  - `apps/platform-core/src/shared/outbox.rs`、`apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`、`workers/outbox-publisher/src/main.rs`、`apps/notification-worker/src/main.rs`：确认成功样本、失败无 outbox 断言补位点，以及发布侧 / 消费侧 live smoke 入口。
- 实现要点：
  - 补强 `apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`，使同一 `create order` live smoke 同时覆盖：
    - 成功分支：`trade.order_main + audit.audit_event + ops.outbox_event` 同事务存在。
    - 失败分支：缺失 buyer org 时返回 `403 / ORDER_CREATE_FORBIDDEN`，并明确断言 `trade.order_main(idempotency_key=failed)`、`audit.audit_event(request_id=failed)`、`ops.outbox_event(request_id=failed)` 全部为 `0`。
  - 新增 `docs/05-test-cases/outbox-consistency-cases.md`，冻结 `TEST-008` 的四个正式 case、官方 checker 和主要回查点。
  - 新增 `scripts/check-outbox-consistency.sh`，串联 `smoke-local.sh`、`trade003_create_order_db_smoke`、`outbox_publisher_db_smoke`、`notif012_notification_worker_live_smoke`，并在运行前显式拦截本机已有 `notification-worker` 常驻实例。
  - 新增 `.github/workflows/outbox-consistency.yml`，把 `TEST-008` checker 纳入最小 CI 路径；同步更新 `docs/05-test-cases/README.md`、`docs/04-runbooks/outbox-publisher.md`、`scripts/README.md`、`.github/workflows/README.md`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`
  4. `ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh`
  5. `cargo test -p platform-core`
  6. `bash -lc 'set -a; source infra/docker/.env.local; source fixtures/smoke/test-005/runtime-baseline.env; set +a; cargo sqlx prepare --workspace'`
  7. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；存在仓库既有 unused / dead-code warning，但无新增编译失败。
  - `TRADE_DB_SMOKE=1 cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture` 通过，新增失败分支已确认不会落任何主对象 / 审计 / outbox 脏副作用。
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-outbox-consistency.sh` 通过，完成以下真实联动验证：
    - `smoke-local.sh` 校验 PostgreSQL / Kafka / Redis / Keycloak / MinIO / Mock Payment / observability 基线。
    - `trade003_create_order_db_smoke` 同时验证 `trade.order.created` 成功写主对象 + 审计 + outbox，以及失败请求无脏 order / audit / outbox。
    - `outbox_publisher_db_smoke` 验证 `ops.outbox_event -> dtp.outbox.domain-events` 成功发布、`ops.outbox_publish_attempt(result_code='published')`、以及失败样本进入 `ops.dead_letter_event + dtp.dead-letter`。
    - `notif012_notification_worker_live_smoke` 验证 `notification.requested -> dtp.notification.dispatch -> notification-worker` 的 duplicate dedupe：第二次消费不重复写 `notification sent via mock-log`，也不重复写 `notification.dispatch.sent` 审计。
  - `cargo test -p platform-core` 通过：`360` 个测试通过，`0` 失败；仓库既有 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，workspace 根目录 `.sqlx` 查询编译元数据已重建并更新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-008`
  - `双层权威模型与链上链下一致性设计.md`
  - `事件模型与Topic清单正式版.md`
  - `服务清单与服务边界正式版.md`
  - `测试用例矩阵正式版.md`
  - `docs/04-runbooks/outbox-publisher.md`
  - `docs/05-test-cases/audit-consistency-cases.md`
  - `docs/05-test-cases/notification-cases.md`
- 覆盖的任务清单条目：`TEST-008`
- 未覆盖项：
  - `TEST-017` 的 dead letter / reprocess 专项、`TEST-018` 的 replay dry-run、`TEST-021`/`TEST-028` 的 canonical checker 不在本批范围，后续按任务顺序继续补齐。
  - 本批的“重复消费不重复副作用”正式样本落在 `notification-worker`；搜索、推荐、Fabric 等其他 consumer 的幂等矩阵由后续专项任务继续扩展。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-306（计划中）
- 任务：`TEST-009` 建立审计完备性测试：关键操作必须产生审计事件，证据导出必须 step-up，非法导出被拒绝
- 状态：计划中
- 说明：当前仓库已经具备 `apps/platform-core/src/modules/audit/tests/api_db.rs` 中的 route guard 测试，以及 `audit_trace_api_db_smoke` 对审计联查、证据包导出、replay、legal hold、anchor retry 的 live smoke，但这些能力还停留在 `AUD` 子域用例里，尚未形成 `TEST-009` 官方 checker / 文档 / CI，也没有把“关键操作必留审计、导出必须 step-up、非法导出被拒绝”以单一验收入口固定下来。本批次将复用现有正式 API 和审计对象，把审计完备性收口为独立可重复任务。
- 前置依赖核对结果：`ENV-040` 已提供 PostgreSQL / MinIO / Kafka / Redis / Keycloak / observability 的本地联调基线；`DB-032` 已通过 migration 与本地 smoke；`CORE-024` 已提供 `platform-core.audit`、证据包导出、replay 和 legal hold 执行面。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-009` 的正式交付是审计完备性测试，不是单独 `AUD` 文档引用。
  - `docs/原始PRD/审计、证据链与回放设计.md`：确认审计事件模型、`step_up_challenge_id`、失败事件也必须记录、证据对象与导出包模型。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认“证据包导出页”是正式高风险控制面，导出内容至少覆盖主体摘要、商品快照、合同快照、交付回执、下载日志、裁决结果、链上摘要。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/04-runbooks/audit-replay.md`、`docs/04-runbooks/audit-consistency-lookup.md`：确认 `TEST` 阶段必须用真实 API + DB / MinIO / 审计回查收口审计闭环。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`、`packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`：确认现有 route guard 和 live smoke 资产、`/api/v1/audit/packages/export` 的 step-up 契约以及成功导出后的正式回查点。
- 当前完成标准理解：
  - 至少要证明：
    1. 关键审计控制面操作会写 `audit.audit_event`、`audit.access_audit`、`ops.system_log`。
    2. `POST /api/v1/audit/packages/export` 缺权限被拒绝、缺 step-up 被拒绝。
    3. 合法 step-up 导出成功后会真实写 `audit.evidence_package`、`audit.evidence_manifest_item`、MinIO 导出对象，并留下 `audit.package.export` 审计。
  - 需要形成 `TEST-009` 专属文档、checker 和 CI 入口，复用正式 API / OpenAPI / 审计对象，不新造旁路导出脚本。
- 实施计划：
  1. 评估现有 `audit_trace_api_db_smoke` 与 route guard 是否已覆盖 `TEST-009` 三个核心断言；若缺口存在，再补最小必要测试。
  2. 新增 `TEST-009` 官方文档与 checker，串联非法导出拒绝、缺 step-up 拒绝和合法导出成功 + 审计回查。
  3. 新增最小 CI workflow，保证 `TEST-009` 在 GitHub Actions 上可重复执行。
  4. 执行真实验证、回写 `P8` 待审批日志并提交，然后继续 `TEST-010`。

### BATCH-306（待审批）
- 任务：`TEST-009` 建立审计完备性测试：关键操作必须产生审计事件，证据导出必须 step-up，非法导出被拒绝
- 状态：待审批
- 当前任务编号：`TEST-009`
- 前置依赖核对结果：`ENV-040` 已提供 PostgreSQL / MinIO / Kafka / Redis / Keycloak / observability 的本地联调基线；`DB-032` 已通过 migration 与本地 smoke；`CORE-024` 已提供 `platform-core.audit`、证据包导出、replay 和 legal hold 正式执行面。当前任务依赖满足。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `TEST-009` 的正式交付是审计完备性测试，不是单独 `AUD` 文档引用。
  - `docs/原始PRD/审计、证据链与回放设计.md`：确认审计事件模型、`step_up_challenge_id`、失败事件也必须记录以及证据对象 / 导出包模型。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认证据包导出页属于正式高风险控制面，导出内容必须覆盖主体摘要、商品快照、合同快照、交付回执、下载日志、裁决结果、链上摘要。
  - `docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`docs/05-test-cases/audit-consistency-cases.md`、`docs/04-runbooks/audit-replay.md`、`docs/04-runbooks/audit-consistency-lookup.md`：确认 `TEST` 阶段必须以真实 API + DB / MinIO / 审计回查收口审计闭环。
  - `apps/platform-core/src/modules/audit/tests/api_db.rs`、`packages/openapi/audit.yaml`、`docs/02-openapi/audit.yaml`：确认现有 route guard 与 live smoke 资产，以及 `/api/v1/audit/packages/export` 的 step-up 契约与导出回查点。
- 实现要点：
  - 新增 `docs/05-test-cases/audit-completeness-cases.md`，冻结 `TEST-009` 的四个正式 case：非法导出拒绝、缺 step-up 拒绝、关键审计动作留痕，以及合法导出后的 `audit.evidence_package / evidence_manifest_item / MinIO` 回查。
  - 新增 `scripts/check-audit-completeness.sh`，把 `rejects_package_export_without_permission`、`package_export_requires_step_up`、`smoke-local.sh` 和 `audit_trace_api_db_smoke` 串成 `TEST-009` 官方 checker。
  - 新增 `.github/workflows/audit-completeness.yml`，在 CI 中执行 `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh`；同步更新 `docs/05-test-cases/README.md`、`scripts/README.md`、`.github/workflows/README.md`。
  - 本批未新增业务逻辑实现，直接复用现有正式 API、OpenAPI 和 audit DB/live smoke 资产收口验收入口，避免再造第二套审计真相源。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh`
  4. `cargo test -p platform-core`
  5. `bash -lc 'set -a; source infra/docker/.env.local; source fixtures/smoke/test-005/runtime-baseline.env; set +a; cargo sqlx prepare --workspace'`
  6. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all` 通过。
  - `cargo check -p platform-core` 通过；存在仓库既有 unused / dead-code warning，但无新增编译失败。
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-audit-completeness.sh` 通过，完成以下真实联动验证：
    - `rejects_package_export_without_permission`：非法角色调用 `POST /api/v1/audit/packages/export` 被正式拒绝。
    - `package_export_requires_step_up`：具备权限但缺少 `x-step-up-token / x-step-up-challenge-id` 的导出请求被正式拒绝。
    - `smoke-local.sh` 校验 PostgreSQL / MinIO / Kafka / Redis / Keycloak / observability 基线。
    - `audit_trace_api_db_smoke` 验证订单审计联查、trace 查询、证据包导出、replay dry-run、legal hold、anchor retry 的真实 API / DB / MinIO / 审计链路，并确认 `audit.package.export`、`audit.access_audit(access_mode='export')`、`ops.system_log`、`audit.evidence_package`、`audit.evidence_manifest_item` 和 MinIO 导出对象全部可回查。
  - `cargo test -p platform-core` 通过：`360` 个测试通过，`0` 失败；仓库既有 `iam_party_access_flow_live` 继续保持 ignored。
  - `cargo sqlx prepare --workspace` 通过，workspace 根目录 `.sqlx` 查询编译元数据已重建并更新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`TEST-009`
  - `审计、证据链与回放设计.md`
  - `页面说明书-V1-完整版.md`
  - `15-测试策略、验收标准与实施里程碑.md`
  - `docs/05-test-cases/audit-consistency-cases.md`
  - `docs/04-runbooks/audit-replay.md`
  - `docs/04-runbooks/audit-consistency-lookup.md`
- 覆盖的任务清单条目：`TEST-009`
- 未覆盖项：
  - `TEST-018` 的 replay dry-run 专项、`TEST-020` 之后的审计控制面扩展与 `TEST-022+` 的前端审计页专项不在本批范围，后续按任务顺序继续补齐。
  - 本批的审计完备性主样本复用 `audit_trace_api_db_smoke`；如果后续冻结任务要求扩大到更多高风险动作，将在后续任务追加到官方 checker。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
