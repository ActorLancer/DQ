### BATCH-116（待审批）
- 状态：待审批
- 当前任务编号：TRADE-007
- 当前批次目标：实现订单主状态机字段：`current_state`、`payment_status`、`delivery_status`、`acceptance_status`、`settlement_status`、`dispute_status`，并确保读写路径统一持久化维护。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-006` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-007` 描述、DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-007` 的详细解释与依赖关系。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次与冻结约束。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：记录本批计划与结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `5.3.2A` 标准 SKU 映射与交易状态语义。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易编排与持久化边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 Trade 接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认订单状态变更与审计/事件联动边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：延续 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED` 口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐本批状态字段读写与回归覆盖。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：保持 domain/repo/tests 分层。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：按 `datab-postgres:5432` 进行 API 联调。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动 `platform-core`。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为业务状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界与可演进结构。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期主状态需要可追溯推进，子状态需明确。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：状态迁移需幂等、不可并发矛盾、不可倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：首批标准 SKU 映射下订单状态字段需对象化表达。
- 已实现功能：
  1. 新增统一状态映射域模块 `layered_status`，集中管理主状态到子状态的映射。
  2. 订单创建写入 `delivery_status/acceptance_status/settlement_status/dispute_status`，不再仅依赖读取时推导。
  3. 订单取消、合同确认、支付结果编排均同步维护四个子状态字段。
  4. 订单详情读取优先返回持久化子状态，并保留回退映射保证兼容。
  5. 新增迁移 `071_trade_order_layered_status.sql`（upgrade/downgrade），补齐字段、回填历史数据、设置默认值与非空约束。
  6. 新增 `TRADE-007` DB smoke 测试；并增强 `TRADE-003/005/006` 的子状态落库断言。
- 涉及文件：
  - `apps/platform-core/src/modules/order/domain/layered_status.rs`
  - `apps/platform-core/src/modules/order/domain/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_create_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_read_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_cancel_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/application/mod.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade005_order_cancel_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade006_contract_confirm_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade007_state_machine_fields_db.rs`
  - `docs/数据库设计/V1/upgrade/071_trade_order_layered_status.sql`
  - `docs/数据库设计/V1/downgrade/071_trade_order_layered_status.sql`
  - `db/migrations/v1/manifest.csv`
  - `db/migrations/v1/checksums.sha256`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `DB_HOST=127.0.0.1 DB_PORT=55432 DB_NAME=luna_data_trading DB_USER=luna DB_PASSWORD=5686 ./db/scripts/migrate-up.sh`
  5. `PGPASSWORD=datab_local_pass psql ... -f docs/数据库设计/V1/upgrade/071_trade_order_layered_status.sql`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade007_state_machine_fields_db_smoke -- --nocapture`
  7. 启动服务并联调：
     `APP_PORT=18080 APP_HOST=127.0.0.1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core`
  8. `curl` 调用：`POST /api/v1/orders`、`GET /api/v1/orders/{id}`、`POST /api/v1/orders/{id}/cancel`
  9. `psql` 校验 `trade.order_main` 子状态字段与 `audit.audit_event` 审计记录。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`99 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：通过。
  - `071` 迁移在 `55432` 通过；`5432` 已应用并生效。
  - `trade007_state_machine_fields_db_smoke`：通过。
  - API 联调：
    - `POST /api/v1/orders` 返回 `200`，创建成功。
    - `GET /api/v1/orders/{id}` 返回 `delivery_status=pending_delivery`、`acceptance_status=not_started`、`settlement_status=not_started`、`dispute_status=none`。
    - `POST /api/v1/orders/{id}/cancel` 返回 `200`。
  - DB 证据：取消后 `trade.order_main` 为 `status=closed`、`delivery_status=canceled`、`acceptance_status=canceled`、`settlement_status=canceled`、`dispute_status=none`。
  - 审计证据：存在 `trade.order.create`、`trade.order.read`、`trade.order.cancel` 三条记录。
  - 清理：临时业务测试数据已删除；审计表 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 订单生命周期总表
  - `Phase1 设计` 6.5 订单状态机
  - `全集成基线 V1` 5.3.2A 标准场景到 SKU 映射
  - `数据库设计总说明` 关于 `status/current_state` 与子状态字段分离维护约束
- 覆盖的任务清单条目：`TRADE-007`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-120（待审批）
- 状态：待审批
- 当前任务编号：TRADE-011
- 当前批次目标：实现 API 按次付费状态机 `API_PPU`：授权、额度/计费口径、调用结算、到期或停用，并完成权限、审计、OpenAPI、测试与接口联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-010` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-011` DoD、验收标准、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-011` 详细任务语义。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先写计划中，再写待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `API_PPU` 为标准 SKU、成功调用计费口径。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态机在 core 内闭环。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：核对 SKU 冻结与 API 模板命名口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：保持审计/事件边界不漂移。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐 `SKU-005`、`PAY-010`（成功调用计费、失败不计费）。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层落地。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库口径 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 事务与状态真值源。
  18. `docs/开发准备/平台总体架构设计草案.md`：确认标准 SKU 边界与模块职责。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单状态按生命周期推进并保持可追溯。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：状态机迁移需幂等、不可倒退、禁止并发矛盾状态。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`API_PPU` 作为标准 SKU 落地，并体现“按调用量”分支语义。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/api-ppu/transition`。
  2. 实现 `API_PPU` 状态机动作：`authorize_access`、`configure_quota`、`record_failed_call`、`settle_success_call`、`expire_access`、`disable_access`。
  3. 强制 `sku_type=API_PPU` 校验；非 `API_PPU` 订单拒绝迁移。
  4. 状态迁移同事务更新 `trade.order_main` 分层状态并写审计 `trade.order.api_ppu.transition`。
  5. 补齐 DTO/repo/router/openapi，新增权限拒绝测试与 DB smoke 测试。
  6. 通过联调验证“失败调用不计费（保持 unpaid）+ 成功调用计费（切换 paid）”。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_api_ppu_transition.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_ppu_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade011_api_ppu_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade011_api_ppu_state_machine_db_smoke -- --nocapture`
  5. `cargo run -p platform-core` 启动服务。
  6. `psql` 插入联调数据（`API_PPU` SKU 订单）。
  7. `curl` 依次调用 `api-ppu` 动作链路与非法迁移。
  8. `psql` 回查订单最终状态、审计条数，并清理临时业务测试数据。
- 验证结果：

### BATCH-121（待审批）
- 状态：待审批
- 当前任务编号：TRADE-012
- 当前批次目标：实现只读共享状态机 `SHARE_RO`：共享开通、访问授权、撤销、到期、争议中断，并完成权限、审计、OpenAPI、测试与接口联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-011` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-012` DoD、验收标准、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-012` 详细任务语义与依赖。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次与不可跳步要求。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：记录本批计划与结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `SHARE_RO` 状态机语义（15.3.4）与标准 SKU 映射。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade 模块编排边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 SKU 冻结规则与接口边界。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认审计/事件留痕口径。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐 `SKU-003`（共享开通/撤权）与生命周期断言。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为状态真值源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块职责边界稳定。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需主状态可追溯推进。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：状态迁移必须幂等、不可并发矛盾、不可倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`SHARE_RO` 为标准 SKU 独立语义，不能并入 API/文件路径。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/share-ro/transition`。
  2. 实现 `SHARE_RO` 状态机动作：`enable_share`、`grant_read_access`、`confirm_first_query`、`revoke_share`、`expire_share`、`interrupt_dispute`。
  3. 强制 `sku_type=SHARE_RO` 校验；非 `SHARE_RO` 订单拒绝迁移。
  4. 每次迁移同事务更新 `trade.order_main` 主状态/子状态并写审计 `trade.order.share_ro.transition`。
  5. 新增 DTO/repo/路由/OpenAPI，补齐权限拒绝测试与 DB smoke 测试。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_share_ro_transition.rs`
  - `apps/platform-core/src/modules/order/repo/order_share_ro_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade012_share_ro_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade012_share_ro_state_machine_db_smoke -- --nocapture`
  5. `cargo run -p platform-core` 启动服务。
  6. `psql` 写入 `SHARE_RO` 临时联调数据。
  7. `curl` 执行动作链路 + 非法迁移校验。
  8. `psql` 回查状态与审计并清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`119 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本失败（报告 `5432` 不可达）；但后续 DB smoke、psql、curl 均连通成功，记为环境脚本兼容噪声。
  - `trade012_share_ro_state_machine_db_smoke`：通过（`1 passed`）。
  - API 联调通过：`enable_share/grant_read_access/confirm_first_query/interrupt_dispute` 全部 `200`；非法 `grant_read_access`（当前 `dispute_interrupted`）返回 `409`，消息含 `SHARE_RO_TRANSITION_FORBIDDEN`。
  - DB 证据：`dispute_interrupted|paid|blocked|blocked|frozen|opened`。
  - 审计证据：`trade.order.share_ro.transition` 计数 `4`。
  - 清理：临时业务测试数据已清理；审计 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 生命周期主链路
  - `Phase1 设计` 6.5 状态机约束
  - `全集成基线 V1` 5.3.2A、15.3.4（`SHARE_RO` 独立状态机语义）
  - `业务流程图 V1` 4.4.1B（共享开通/撤权/到期）
- 覆盖的任务清单条目：`TRADE-012`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`115 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本失败（报告 `5432` 不可达），但后续 `TRADE_DB_SMOKE`、`psql`、`curl` 验证均可实际连通并成功，记录为环境检测脚本兼容性噪声。
  - `trade011_api_ppu_state_machine_db_smoke`：通过（`1 passed`）。
  - API 联调：`authorize_access/configure_quota/record_failed_call/settle_success_call/expire_access/disable_access` 全部 `200`；非法 `settle_success_call`（当前 `disabled`）返回 `409`，消息包含 `API_PPU_TRANSITION_FORBIDDEN`。
  - 计费口径证据：
    - `record_failed_call` 响应 `payment_status=unpaid`，`settlement_status=not_started`。
    - `settle_success_call` 响应 `payment_status=paid`，`settlement_status=pending_settlement`。
  - DB 证据：最终 `disabled|paid|closed|closed|closed|none`。
  - 审计证据：`trade.order.api_ppu.transition` 计数 `6`。
  - 清理结果：临时业务测试数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 交易生命周期（订单主状态与子状态并行约束）
  - `Phase1 设计` 6.5 订单状态机（幂等与不可逆约束）
  - `全集成基线 V1` 5.3.2A 与 API_PPU 计费语义（成功调用计费）
- 覆盖的任务清单条目：`TRADE-011`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-117（待审批）
- 状态：待审批
- 当前任务编号：TRADE-008
- 当前批次目标：实现文件交易状态机 `FILE_STD`：创建、待锁资、待交付、待验收、已完成、已退款/争议等路径。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-007` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-008` 描述、DoD、验收、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-008` 详细口径与顺序。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：记录本批计划与结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `FILE_STD` 属于首批标准链路主 SKU。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 Trade API 冻结风格与约束。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态变更的审计/事件留痕口径。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐 FILE_STD 正常链路与争议退款链路测试。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 逻辑分层。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：使用 `datab-postgres:5432` 联调。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：维持 PostgreSQL 业务状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界稳定。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需要覆盖正常与争议分支。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：迁移需幂等、不可并发矛盾、不可倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`FILE_STD` 在首批场景中是独立主 SKU，状态快照需对象化。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/file-std/transition`，支持 `FILE_STD` 状态迁移动作集。
  2. 新增 `FILE_STD` 转换规则：`lock_funds -> start_delivery -> mark_delivered -> accept_delivery -> settle_order -> close_completed`。
  3. 新增争议/退款分支：`open_dispute`、`resolve_dispute_refund`、`resolve_dispute_complete`、`request_refund`。
  4. 强校验仅允许 `sku_type=FILE_STD` 执行该状态机转换。
  5. 每次状态迁移同事务落库：主状态 + 分层子状态 + `last_reason_code` + 审计动作 `trade.order.file_std.transition`。
  6. 新增 DTO、repo、权限拒绝测试、DB smoke 测试；更新 OpenAPI。
- 涉及文件：
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/dto/order_file_std_transition.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_std_repository.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade008_file_std_state_machine_db.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade008_file_std_state_machine_db_smoke -- --nocapture`
  5. 启动服务联调：
     `APP_PORT=18080 APP_HOST=127.0.0.1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 OPENSEARCH_ENDPOINT=http://127.0.0.1:9200 cargo run -p platform-core`
  6. `curl` 联调：`POST /api/v1/orders` + `POST /api/v1/orders/{id}/file-std/transition`（链路+争议退款分支）。
  7. `psql` 校验 `trade.order_main` 最终状态与 `audit.audit_event` 审计记录。
- 验证结果：
  - `cargo test -p platform-core`：通过（`103 passed, 0 failed, 1 ignored`）。
  - `trade008_file_std_state_machine_db_smoke`：通过。
  - API 联调通过：
    - `lock_funds` 后状态 `buyer_locked`；
    - `start_delivery` 后状态 `seller_delivering`；
    - `mark_delivered` 后状态 `delivered`；
    - `open_dispute` 后状态 `dispute_opened`；
    - `resolve_dispute_refund` 后状态 `closed + refunded`。
  - DB 证据：`status=closed, payment_status=refunded, delivery_status=refunded, acceptance_status=refunded, settlement_status=refunded, dispute_status=resolved`。
  - 审计证据：`trade.order.create` 与多条 `trade.order.file_std.transition` 均存在。
  - 清理：临时业务测试数据已清理；审计表 append-only 记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 交易对象生命周期
  - `Phase1 设计` 6.5 订单状态机
  - `全集成基线 V1` 5.3.2A `FILE_STD` 主路径语义
- 覆盖的任务清单条目：`TRADE-008`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 由你手工维护，本批未写入。

### BATCH-118（待审批）
- 状态：待审批
- 当前任务编号：TRADE-009
- 当前批次目标：实现文件订阅状态机 `FILE_SUB`：订阅建立、周期交付、周期验收、暂停、到期、续订，并补齐权限、审计、OpenAPI 与测试。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-008` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-009` 描述、DoD、验收、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-009` 详细条目与依赖。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次与不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：记录本批次计划与结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `FILE_SUB` 为标准 SKU，且需覆盖周期交付、到期断权、续订语义。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界在 `platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 Trade 接口契约与统一响应约束。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态变更审计/事件留痕口径。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT`、`IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐周期型 SKU 的生命周期与多次履约验证要求。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：以环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为业务状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，不跨域扩展。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单主状态需覆盖标准生命周期与争议分支。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：状态迁移必须幂等、不可并发矛盾、不可倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`FILE_SUB` 为独立标准 SKU，需体现周期订阅语义（建立、履约、暂停、到期、续订）。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/file-sub/transition`。
  2. 新增 `FILE_SUB` 状态机动作：`establish_subscription`、`start_cycle_delivery`、`mark_cycle_delivered`、`accept_cycle_delivery`、`pause_subscription`、`expire_subscription`、`renew_subscription`，并保留争议/退款分支动作。
  3. 新增 `sku_type=FILE_SUB` 强校验，非 `FILE_SUB` 订单拒绝迁移。
  4. 新增 `trade.order.file_sub.transition` 审计动作，同事务写入订单状态与审计。
  5. 新增 FILE_SUB DTO、仓储、权限拒绝测试、DB smoke 测试。
  6. 更新 `packages/openapi/trade.yaml`，补齐 FILE_SUB path 与 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_file_sub_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade009_file_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `make up-local`
  4. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade009_file_sub_state_machine_db_smoke -- --nocapture`
  6. `cargo run -p platform-core` 启动服务后执行 `curl`：`POST /api/v1/orders` + `POST /api/v1/orders/{id}/file-sub/transition`（全链路动作 + 非法迁移冲突）。
  7. `psql` 校验 `trade.order_main` 最终状态与 `audit.audit_event` 记录数。
  8. 清理临时业务测试数据（审计表 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`107 passed, 0 failed, 1 ignored`）。
  - `make up-local`：通过，核心容器启动成功。
  - `check-local-stack core`：脚本返回失败（报告 `5432` 不可达），但 `docker ps` 显示 `datab-postgres` healthy，后续 `psql`/`cargo test`/`curl` 均能实际连通，判定为环境检测脚本与当前执行环境兼容性问题，不阻断本批功能验证。
  - `trade009_file_sub_state_machine_db_smoke`：通过（`1 passed`）。
  - API 联调通过：动作 `establish_subscription -> start_cycle_delivery -> mark_cycle_delivered -> accept_cycle_delivery -> pause_subscription -> renew_subscription -> expire_subscription` 均返回 `200`。
  - DB 证据：最终 `status=expired|payment_status=paid|delivery_status=expired|acceptance_status=expired|settlement_status=expired|dispute_status=none`。
  - 审计证据：`trade.order.file_sub.transition` 计数 `7`；非法迁移 `start_cycle_delivery`（当前 `expired`）返回 `409` + `FILE_SUB_TRANSITION_FORBIDDEN`。
  - 清理：临时业务测试数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 生命周期总表（主状态推进 + 争议分支）
  - `Phase1 设计` 6.5 订单状态机幂等与不可倒退要求
  - `全集成基线 V1` 5.3.2A（`FILE_SUB` 作为标准 SKU 的独立语义）
  - `业务流程图 V1` 4.4.1A（版本订阅交付、停订/到期语义）
- 覆盖的任务清单条目：`TRADE-009`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-119（待审批）
- 状态：待审批
- 当前任务编号：TRADE-010
- 当前批次目标：实现 API 订阅状态机 `API_SUB`：锁资、应用绑定、密钥开通、试调用、正式可用、周期计费、终止，并完成权限、审计、OpenAPI、测试与接口联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-009` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-010` 描述、DoD、验收和 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-010` 详细语义与依赖口径。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次与冻结边界。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：登记本批计划与结果。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `API_SUB` 为首批标准场景主 SKU。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 Trade API 冻结风格和错误响应口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态变更审计/事件留痕边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐周期型 SKU 的生命周期与多次履约覆盖要求。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为业务状态权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界稳定。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需主状态可追踪推进。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：状态迁移必须幂等、不可并发矛盾、不可倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`API_SUB` 是首批标准 SKU，需体现订阅主路径语义。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/api-sub/transition`。
  2. 新增 `API_SUB` 状态机动作：`lock_funds`、`bind_application`、`issue_api_key`、`trial_call`、`activate_subscription`、`bill_cycle`、`terminate_subscription`。
  3. 新增 `sku_type=API_SUB` 强校验；非 `API_SUB` 订单拒绝迁移。
  4. 每次迁移同事务更新订单主状态/子状态并写入审计 `trade.order.api_sub.transition`。
  5. 新增 DTO、repo、权限拒绝测试、DB smoke 测试；更新 OpenAPI。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_api_sub_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade010_api_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade010_api_sub_state_machine_db_smoke -- --nocapture`
  5. 启动服务联调：`cargo run -p platform-core`
  6. `curl` 联调：`POST /api/v1/orders` + `POST /api/v1/orders/{id}/api-sub/transition`（全动作链路 + 非法迁移）
  7. `psql` 校验 `trade.order_main` 和 `audit.audit_event`，并清理临时业务测试数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`111 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本返回失败（`5432` 不可达）；但后续 `TRADE_DB_SMOKE`、`psql`、`curl` 全部可连通并成功，判定为该脚本在当前执行环境下的兼容性噪声，不阻断本批功能验证。
  - `trade010_api_sub_state_machine_db_smoke`：通过（`1 passed`）。
  - API 联调通过：`lock_funds/bind_application/issue_api_key/trial_call/activate_subscription/bill_cycle/terminate_subscription` 均返回 `200`。
  - DB 证据：最终状态 `closed|paid|closed|closed|closed|none`。
  - 审计证据：`trade.order.api_sub.transition` 计数 `7`。
  - 非法迁移证据：`bill_cycle` 在 `closed` 状态返回 `409`，消息包含 `API_SUB_TRANSITION_FORBIDDEN`。
  - 清理：临时业务测试数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 交易对象生命周期
  - `Phase1 设计` 6.5 订单状态机要求
  - `全集成基线 V1` 5.3.2A 标准 SKU 映射（`API_SUB`）
- 覆盖的任务清单条目：`TRADE-010`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-122（待审批）
- 状态：待审批
- 当前任务编号：TRADE-013
- 当前批次目标：实现模板查询状态机 `QRY_LITE`：模板授权、参数校验、执行、结果可取、验收关闭，并完成权限、审计、OpenAPI、测试与 API 联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-012` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-013` 描述、DoD、验收、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-013` 详细语义与依赖关系。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：遵循“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批计划与结果记录载体（按当前口径替代旧路径）。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `QRY_LITE` 独立 SKU 语义与 15.3.5 状态机要求。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界在 `platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认接口契约与统一响应结构。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态迁移审计留痕边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐 `SKU-006` / `PAY-010`（模板执行成功计费事件口径）。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为状态真值源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界与职责收敛。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需要可追溯的状态推进与闭环。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：迁移幂等、不可并发矛盾、不可非法倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`QRY_LITE` 是首批标准 SKU 独立链路，不得并入其他类型。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/qry-lite/transition`。
  2. 实现 `QRY_LITE` 状态机动作：`authorize_template`、`validate_params`、`execute_query`、`make_result_available`、`close_acceptance`。
  3. 强制 `sku_type=QRY_LITE` 校验；非 `QRY_LITE` 订单拒绝迁移。
  4. 每次迁移同事务更新 `trade.order_main`（主状态/分层状态）并写审计 `trade.order.qry_lite.transition`。
  5. 新增 DTO/repo/router 接线、权限拒绝测试、DB smoke 测试与 OpenAPI schema。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_qry_lite_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_qry_lite_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade013_qry_lite_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade013_qry_lite_state_machine_db_smoke -- --nocapture`
  5. 启动服务：`APP_PORT=18081 ... cargo run -p platform-core`
  6. `psql` 插入 `QRY_LITE` 联调数据。
  7. `curl` 依次调用 `authorize_template -> validate_params -> execute_query -> make_result_available -> close_acceptance`，并校验非法重复执行。
  8. `psql` 回查订单终态与审计计数。
  9. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`123 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本失败（报告 `5432` 不可达）；但后续 DB smoke、psql、curl 均连通成功，记录为环境脚本兼容性噪声。
  - `trade013_qry_lite_state_machine_db_smoke`：通过（`1 passed`，外部权限运行）。
  - API 联调通过（`18081` 当前代码服务实例）：5 个动作均 `HTTP 200`。
  - 非法迁移校验：`execute_query` 在 `closed` 后再次执行返回 `HTTP 409`，消息包含 `QRY_LITE_TRANSITION_FORBIDDEN`。
  - DB 终态：`closed|paid|closed|closed|closed|none`。
  - 审计证据：`audit.audit_event` 中 `action_name='trade.order.qry_lite.transition'` 计数 `5`。
  - 清理结果：临时业务数据已删除；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 生命周期总表（订单主状态闭环）
  - `Phase1 设计` 6.5 订单状态机（幂等与冲突约束）
  - `全集成基线 V1` 5.3.2A（标准场景与 `QRY_LITE` 映射）
  - `全集成基线 V1` 15.3.5（模板查询 lite 交付/验收条件）
  - `业务流程图 V1` 4.4.3（模板查询交付主流程）
- 覆盖的任务清单条目：`TRADE-013`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-123（待审批）
- 状态：待审批
- 当前任务编号：TRADE-014
- 当前批次目标：实现查询沙箱状态机 `SBX_STD`：空间开通、账号/席位下发、执行、受限导出、到期或撤权，并完成权限、审计、OpenAPI、测试与 API 联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-013` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-014` 描述、DoD、验收、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-014` 详细语义与依赖关系。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：遵循“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批计划与结果记录载体。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `SBX_STD` 独立 SKU 语义与 15.3.6 状态机要求。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界在 `platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认接口契约与统一响应结构。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态迁移审计留痕边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐 `SKU-007`（沙箱席位开通、环境可用、导出限制）与 `AUTHZ-005`。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为状态真值源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界与职责收敛。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需要可追溯的状态推进与闭环。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：迁移幂等、不可并发矛盾、不可非法倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`SBX_STD` 是首批标准 SKU 独立链路，不得并入 `QRY_LITE` 或其他类型。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/sbx-std/transition`。
  2. 实现 `SBX_STD` 状态机动作：`enable_workspace`、`issue_account_seat`、`execute_sandbox_query`、`export_limited_result`、`expire_sandbox`、`revoke_sandbox`。
  3. 强制 `sku_type=SBX_STD` 校验；非 `SBX_STD` 订单拒绝迁移。
  4. 每次迁移同事务更新 `trade.order_main`（主状态/分层状态）并写审计 `trade.order.sbx_std.transition`。
  5. 新增 DTO/repo/router 接线、权限拒绝测试、DB smoke 测试与 OpenAPI schema。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_sbx_std_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_sbx_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade014_sbx_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade014_sbx_std_state_machine_db_smoke -- --nocapture`
  5. 启动服务：`APP_PORT=18082 ... cargo run -p platform-core`
  6. `psql` 插入 `SBX_STD` 联调数据。
  7. `curl` 依次调用 `enable_workspace -> issue_account_seat -> execute_sandbox_query -> export_limited_result -> expire_sandbox`，并校验非法重复执行。
  8. `psql` 回查订单终态与审计计数。
  9. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`127 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本失败（报告 `5432` 不可达）；但后续 DB smoke、psql、curl 均连通成功，记录为环境脚本兼容性噪声。
  - `trade014_sbx_std_state_machine_db_smoke`：通过（`1 passed`，外部权限运行）。
  - API 联调通过（`18082` 当前代码服务实例）：5 个合法动作均 `HTTP 200`。
  - 非法迁移校验：`execute_sandbox_query` 在 `expired` 后再次执行返回 `HTTP 409`，消息包含 `SBX_STD_TRANSITION_FORBIDDEN`。
  - DB 终态：`expired|paid|expired|expired|expired|none`。
  - 审计证据：`audit.audit_event` 中 `action_name='trade.order.sbx_std.transition'` 计数 `5`。
  - 清理结果：临时业务数据已删除；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 生命周期总表（订单主状态闭环）
  - `Phase1 设计` 6.5 订单状态机（幂等与冲突约束）
  - `全集成基线 V1` 5.3.2A（标准场景与 `SBX_STD` 映射）
  - `全集成基线 V1` 15.3.6（查询沙箱交付/验收条件）
  - `业务流程图 V1` 4.4.3（沙箱交付主流程）
- 覆盖的任务清单条目：`TRADE-014`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-124（待审批）
- 状态：待审批
- 当前任务编号：TRADE-015
- 当前批次目标：实现报告产品状态机 `RPT_STD`：任务建立、报告生成、报告交付、验收、结算，并完成权限、审计、OpenAPI、测试与 API 联调闭环。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-014` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-015` 描述、DoD、验收、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对 `TRADE-015` 详细语义与依赖关系。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：遵循“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批计划与结果记录载体。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `RPT_STD` 独立 SKU 语义与 15.3.7 状态机要求。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易状态编排边界在 `platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认接口契约与统一响应结构。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认状态迁移审计留痕边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐 `SKU-008`（报告生成成功、结果包交付、签收/回执有效）。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按 dto/repo/api/tests 分层实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量启动服务联调。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为状态真值源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块边界与职责收敛。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期需要可追溯的状态推进与闭环。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：迁移幂等、不可并发矛盾、不可非法倒退。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：`RPT_STD` 是首批标准 SKU 独立链路，不得并入其他类型。
- 已实现功能：
  1. 新增 `POST /api/v1/orders/{id}/rpt-std/transition`。
  2. 实现 `RPT_STD` 状态机动作：`create_report_task`、`generate_report`、`deliver_report`、`accept_report`、`settle_report`。
  3. 强制 `sku_type=RPT_STD` 校验；非 `RPT_STD` 订单拒绝迁移。
  4. 每次迁移同事务更新 `trade.order_main`（主状态/分层状态）并写审计 `trade.order.rpt_std.transition`。
  5. 新增 DTO/repo/router 接线、权限拒绝测试、DB smoke 测试与 OpenAPI schema。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_rpt_std_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_rpt_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade015_rpt_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade015_rpt_std_state_machine_db_smoke -- --nocapture`
  5. 启动服务：`APP_PORT=18083 ... cargo run -p platform-core`
  6. `psql` 插入 `RPT_STD` 联调数据。
  7. `curl` 依次调用 `create_report_task -> generate_report -> deliver_report -> accept_report -> settle_report`，并校验非法重复执行。
  8. `psql` 回查订单终态与审计计数。
  9. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`131 passed, 0 failed, 1 ignored`）。
  - `check-local-stack core`：脚本失败（报告 `5432` 不可达）；但后续 DB smoke、psql、curl 均连通成功，记录为环境脚本兼容性噪声。
  - `trade015_rpt_std_state_machine_db_smoke`：通过（`1 passed`，外部权限运行）。
  - API 联调通过（`18083` 当前代码服务实例）：5 个合法动作均 `HTTP 200`。
  - 非法迁移校验：`generate_report` 在 `settled` 后再次执行返回 `HTTP 409`，消息包含 `RPT_STD_TRANSITION_FORBIDDEN`。
  - DB 终态：`settled|paid|closed|closed|settled|none`。
  - 审计证据：`audit.audit_event` 中 `action_name='trade.order.rpt_std.transition'` 计数 `5`。
  - 清理结果：临时业务数据已删除；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2 生命周期总表（订单主状态闭环）
  - `Phase1 设计` 6.5 订单状态机（幂等与冲突约束）
  - `全集成基线 V1` 5.3.2A（标准场景与 `RPT_STD` 映射）
  - `全集成基线 V1` 15.3.7（报告产品交付/验收条件）
  - `业务流程图 V1` 4.4.3（结果产品交付主流程）
- 覆盖的任务清单条目：`TRADE-015`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按你的约定由你手工维护，本批未写入。

### BATCH-125（计划中）
- 状态：计划中
- 当前任务编号：TRADE-016
- 当前批次目标：实现数字合约聚合：合同模板、合同快照、签署状态、签约主体、签署时间、摘要上链引用。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-015` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-016` 描述、DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-016` 详细语义与顺序执行要求。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：执行“计划中→编码→验证→待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次与不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志.md`：路径不存在，按当前约定在 `V1-Core-实施进度日志-P2.md` 记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：确认 TODO 追溯与 `TODO-PROC-BIL-001` 约束。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认交易链第 15 章、合同聚合职责与摘要上链口径。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易编排在 `platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 `/api/v1/orders/{id}/contract-confirm` 契约边界。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认交易审计留痕边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐主交易链路的合同确认与审计可追溯要求。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：维持 dto/repo/api/tests 分层。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432` 与 core 栈容器。
  16. `docs/开发准备/配置项与密钥管理清单.md`：按环境变量注入 DB/Kafka。
  17. `docs/开发准备/技术选型正式版.md`：保持 PostgreSQL 为业务真值。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持 order/contract/authorization 职责边界。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L530`：`DigitalContract` 必须体现合同快照，并绑定 `DataContract` 摘要与引用。
  2. `docs/原始PRD/数据商品元信息与数据契约设计.md:L86`：数据契约必须独立建模，不得塞入 product metadata。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：主交易闭环在合同阶段生成可追溯合同与策略对象。

### BATCH-125（待审批）
- 状态：待审批
- 当前任务编号：TRADE-016
- 当前批次目标：实现数字合约聚合：合同模板、合同快照、签署状态、签约主体、签署时间、摘要上链引用。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-015` 已审批通过。
- 已实现功能：
  1. 扩展合同确认请求：新增 `data_contract_id`、`data_contract_digest` 可选字段。
  2. 扩展合同确认响应：返回 `contract_template_id`、`contract_digest`、`data_contract_id`、`data_contract_digest`、`signer_id`、`signer_type`、`variables_json`、`onchain_digest_ref`。
  3. 扩展合同仓储事务：`contract.digital_contract` 同事务落库 `data_contract_id` 与 `data_contract_digest`，并保持签署状态/签署时间/签约主体写入。
  4. 增加 `TRADE-016` DB smoke 测试：校验 API 响应、订单状态、数字合同聚合字段、签署人记录、审计记录。
  5. 更新 OpenAPI：同步请求/响应 schema 与任务描述，保持路由不变。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_contract_confirm.rs`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade016_digital_contract_aggregate_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade016_digital_contract_aggregate_db_smoke -- --nocapture`
  4. 启动服务：`APP_PORT=18084 DATABASE_URL=... KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
  5. `psql` 插入联调测试数据（buyer/seller/user/template/product/sku/data_contract/order）。
  6. `curl` 调用 `POST /api/v1/orders/{id}/contract-confirm` 并校验返回聚合字段。
  7. `psql` 回查 `trade.order_main`、`contract.digital_contract`、`contract.contract_signer`、`audit.audit_event`。
  8. 清理临时业务数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`132 passed, 0 failed, 1 ignored`）。
  - `trade016_digital_contract_aggregate_db_smoke`：通过（`1 passed`）。
  - API 联调：`HTTP 200`，返回包含 `contract_template_id`、`data_contract_id`、`signer_id`、`signed_at`、`variables_json`、`onchain_digest_ref`。
  - DB 回查：
    - `trade.order_main.status=contract_effective`；
    - `contract.digital_contract` 已写入 `data_contract_id`/`data_contract_digest`/`contract_digest`/`variables_json.region`；
    - `contract.contract_signer` 命中 1 条签署记录；
    - `audit.audit_event` 中 `trade.contract.confirm` 命中 1 条。
  - 环境说明：服务启动需显式使用 `KAFKA_BROKERS=127.0.0.1:9094`（或同值的 `KAFKA_BOOTSTRAP_SERVERS`）以避免容器内部地址 `kafka:9092` 对主机进程不可达。
- 覆盖的冻结文档条目：
  - `领域模型` 4.3 合同与策略聚合（DigitalContract + DataContract 绑定）
  - `原始PRD` 3.2 数据契约单独建模
  - `全集成基线 V1` 15 核心交易链路（合同阶段）
  - `数据库表字典` `contract.digital_contract`、`contract.contract_signer` 字段口径
- 覆盖的任务清单条目：`TRADE-016`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
