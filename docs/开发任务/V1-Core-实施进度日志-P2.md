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

### BATCH-140（计划中）
- 状态：计划中
- 当前任务编号：TRADE-031
- 当前批次目标：实现统一“可交付判定器”，在各 SKU 首个交付/开通动作前综合校验支付状态、合同状态、主体状态、商品审核状态、风控状态；只有满足门禁时才创建最小 `delivery.delivery_record` 并允许进入交付/履约；禁止绕过门禁直接进入已交付链路。
- 前置依赖核对结果：`TRADE-021`、`TRADE-030`、`CAT-010` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-031` 为当前单任务批次，目标是“可交付判定器”，并要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，强调“只有全部满足时才创建交付任务并把订单推进到待交付；禁止支付成功后绕过前置校验进入已交付”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程先写计划中，再编码、验证、更新 TODO 与待审批。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务批次，不跨任务扩展。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批写入计划中与后续待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：后续追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `contract_effective -> payment_locked -> delivery_in_progress` 主链路、交付记录状态机、首批 8 个标准 SKU 交付语义。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认订单编排在 `order`，交付实体在 `delivery`，不得越界实现后续专用交付能力。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增接口，保持既有 transition API 契约不漂移。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付锁定后进入待交付的事件语义与审计要求。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持状态冲突类错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批补统一可交付门禁专项回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：在 `order/repo` 下新增独立门禁仓储，不把规则继续堆进单个 transition 文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证继续使用 core 栈数据库 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用现有本地 `KAFKA_*` 和 `DATABASE_URL` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL 实现统一门禁仓储与测试。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，本批只补交易主编排与交付证据落库缺口。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单主状态必须保持唯一主轴，交付聚合为 `Order 1 -> N Delivery`，`Delivery` 自身状态机为 `prepared -> committed -> available -> consumed | expired`。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：主交易链路必须遵守 `contract_effective -> payment_locked -> delivery_in_progress`，交付/开通前必须通过主体、合同、风控等最终放行链。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方锁资后才通知卖方待交付，交付服务先创建 `delivery_id` 再进入各类具体交付/开通动作。

### BATCH-140（待审批）
- 状态：待审批
- 当前任务编号：TRADE-031
- 当前批次目标：实现统一“可交付判定器”，在各 SKU 首个交付/开通动作前综合校验支付状态、合同状态、主体状态、商品审核状态、风控状态；只有满足门禁时才创建最小 `delivery.delivery_record` 并允许进入交付/履约；禁止绕过门禁直接进入已交付链路。
- 前置依赖核对结果：`TRADE-021`、`TRADE-030`、`CAT-010` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-031` 为当前单任务批次，目标是“可交付判定器”，并要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，强调“只有全部满足时才创建交付任务并把订单推进到待交付；禁止支付成功后绕过前置校验进入已交付”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程先写计划中，再编码、验证、更新 TODO 与待审批。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务批次，不跨任务扩展。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：已写入本批计划中与待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：已追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `contract_effective -> payment_locked -> delivery_in_progress` 主链路、交付记录状态机、首批 8 个标准 SKU 交付语义。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认订单编排在 `order`，交付实体在 `delivery`，不得越界实现后续专用交付能力。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增接口，保持既有 transition API 契约不漂移。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付锁定后进入待交付的事件语义与审计要求。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持状态冲突类错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批补统一可交付门禁专项回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：在 `order/repo` 下新增独立门禁仓储，不把规则继续堆进单个 transition 文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证继续使用 core 栈数据库 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用现有本地 `KAFKA_*` 和 `DATABASE_URL` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL 实现统一门禁仓储与测试。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，本批只补交易主编排与交付证据落库缺口。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单主状态必须保持唯一主轴，交付聚合为 `Order 1 -> N Delivery`，`Delivery` 自身状态机为 `prepared -> committed -> available -> consumed | expired`。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：主交易链路必须遵守 `contract_effective -> payment_locked -> delivery_in_progress`，交付/开通前必须通过主体、合同、风控等最终放行链。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方锁资后才通知卖方待交付，交付服务先创建 `delivery_id` 再进入各类具体交付/开通动作。
- 已实现功能：
  1. 新增 `order_deliverability_repository`，统一封装支付状态、合同状态、主体状态、商品状态、商品审核状态、风控状态、资产版本状态、SKU 状态的可交付门禁校验。
  2. 门禁通过后创建最小 `delivery.delivery_record`，状态固定为 `prepared`，并按 8 个标准 SKU 写入对应 `delivery_type / delivery_route`；若已有 `prepared` 记录则复用，不重复创建。
  3. 在 `FILE_STD / FILE_SUB / API_SUB / API_PPU / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 的首个交付/开通动作前接入统一门禁，未通过时返回 `409 ORDER_DELIVERABILITY_CHECK_FAILED:*`。
  4. 收紧 `API_PPU / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 首个动作的起始主状态到 `buyer_locked`，与 `payment_locked -> delivery_in_progress` 主轴保持一致，避免支付成功后直接绕过门禁进入已交付/已开通链路。
  5. 新增 `trade031_deliverability_gate_db_smoke`，覆盖缺少合同、主体风控阻断、商品审核阻断、通过门禁后创建 `prepared` 交付记录并推进状态的完整链路。
  6. 更新 `trade008~trade015` 状态机 smoke 种子，补齐签署合同与 `buyer_locked/paid` 前提，保证 8 个标准 SKU 在统一门禁接入后仍然可回归通过。
  7. 更新 `packages/openapi/trade.yaml` 与 `docs/05-test-cases/order-state-machine.md`，补充 TRADE-031 门禁口径与首个动作的冻结说明。
- 涉及文件：
  - `apps/platform-core/src/modules/order/repo/order_deliverability_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_ppu_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_share_ro_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_qry_lite_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_sbx_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_rpt_std_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade008_file_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade009_file_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade010_api_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade011_api_ppu_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade012_share_ro_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade013_qry_lite_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade014_sbx_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade015_rpt_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade031_deliverability_gate_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/05-test-cases/order-state-machine.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade008_file_std_state_machine_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade009_file_sub_state_machine_db_smoke -- --nocapture`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade010_api_sub_state_machine_db_smoke -- --nocapture`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade011_api_ppu_state_machine_db_smoke -- --nocapture`
  7. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade012_share_ro_state_machine_db_smoke -- --nocapture`
  8. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade013_qry_lite_state_machine_db_smoke -- --nocapture`
  9. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade014_sbx_std_state_machine_db_smoke -- --nocapture`
  10. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade015_rpt_std_state_machine_db_smoke -- --nocapture`
  11. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade031_deliverability_gate_db_smoke -- --nocapture`
  12. 启动服务：`APP_PORT=8091 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  13. `psql` 写入临时 `SHARE_RO` 订单、合同和商品数据，`curl POST /api/v1/orders/{id}/share-ro/transition` 依次验证主体风控阻断、商品审核阻断、门禁放行成功。
  14. `psql` 回查 `trade.order_main`、`delivery.delivery_record`、`audit.audit_event`，再清理临时业务数据；审计记录按 append-only 保留。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`156 passed, 0 failed, 1 ignored`）。
  - `trade008~trade015` 8 个标准 SKU 状态机 DB smoke：全部通过。
  - `trade031_deliverability_gate_db_smoke`：通过；覆盖缺少合同、主体阻断、商品审核阻断、门禁放行四条路径。
  - 真实 API 联调：
    - 主体风控阻断：`POST /api/v1/orders/{id}/share-ro/transition` 返回 `HTTP 409`
    - 商品审核阻断：同接口返回 `HTTP 409`
    - 放行成功：同接口返回 `HTTP 200`
  - DB 回查：
    - 订单状态：`share_enabled / paid / in_progress / not_started / pending_settlement`
    - 交付记录：`share_grant / share_link / prepared`
    - 审计：`trade.order.delivery_gate.prepared=1`、`trade.order.share_ro.transition=1`
  - 清理结果：临时业务数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（交易与订单聚合）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-031`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-141（计划中）
- 状态：计划中
- 当前任务编号：TRADE-032
- 当前批次目标：实现五条标准链路的场景到 SKU 快照规则；同一场景可包含主 SKU 与补充 SKU，但订单、合同、授权及后续验收/结算依据必须按 SKU 单独快照，不允许仅记录场景名。
- 前置依赖核对结果：`CTX-021`、`TRADE-023` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-032` 为当前单任务批次，依赖为 `CTX-021; TRADE-023`，目标是场景到 SKU 快照规则落地。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，强调“一个场景可包含主 SKU 与补充 SKU，但订单、合同、授权、验收、结算仍必须按 SKU 单独快照”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 编码 -> 验证 -> TODO -> 待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务批次，不跨任务扩展，不以场景名替代事实源。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批先登记计划中，完成后补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对五条标准链路与 5.3.2A 场景到主/补充 SKU 映射，确认 `API_SUB`、`RPT_STD` 等存在多场景歧义。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易快照逻辑应在 `order/contract/authorization` 内闭环，不越界到 `delivery` 或 `billing` 的后续实现。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务允许在 Trade OpenAPI 中补充场景快照字段，但必须保持现有路径不变。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对订单创建事件应可携带足够业务快照，不得只留场景名。
  12. `docs/开发准备/统一错误码字典正式版.md`：继续沿用状态冲突类错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批补场景-SKU 快照专项回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立场景快照领域文件，避免把规则堆进单个仓储。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证继续使用 core 栈数据库 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用现有 `DATABASE_URL`、`KAFKA_*` 本地配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL，实现快照解析与持久化。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，本批只补快照事实源与下游快照同步。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单是唯一主聚合，主状态之外的合同、授权、验收、结算证据都必须围绕具体订单/SKU 事实留存。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：五条标准链路均以标准 SKU 为主轴推进，不允许场景名覆盖 SKU 事实。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：下单时需冻结模板/价格/权利/有效期等事实，后续合同、授权、交付、验收、结算均基于订单快照推进。
- 额外读取依赖文档：
  1. `docs/00-context/v1-closed-loop-matrix.md`：确认 `8 SKU × 5 场景` 下主挂点/补充挂点/非挂点矩阵，非挂点必须阻断且审计。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md` `5.3.2 / 5.3.2A`：确认五条标准链路与主/补充 SKU 映射，以及“若一个场景同时使用多个 SKU，订单、合同、授权、验收、结算仍应按 SKU 逐一快照”的补充要求。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/domain/**`
  - `apps/platform-core/src/modules/order/repo/order_create_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_authorization_repository.rs`
  - `apps/platform-core/src/modules/order/repo/price_snapshot_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_pre_payment_lock_repository.rs`
  - `apps/platform-core/src/modules/order/tests/**`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade021_pre_payment_lock_checks_db_smoke -- --nocapture`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade032_scenario_sku_snapshot_db_smoke -- --nocapture`
  6. 启动服务后用 `curl` 真实联调：验证歧义 SKU 不带 `scenario_code` 被阻断、显式 `scenario_code` 创建成功、合同与授权快照含场景-SKU 快照。

### BATCH-141（待审批）
- 状态：待审批
- 当前任务编号：TRADE-032
- 当前批次目标：实现五条标准链路的场景到 SKU 快照规则；同一场景可包含主 SKU 与补充 SKU，但订单、合同、授权及后续验收/结算依据必须按 SKU 单独快照，不允许仅记录场景名。
- 前置依赖核对结果：`CTX-021`、`TRADE-023` 已完成且审批通过。
- 已实现功能：
  1. 新增 `order/domain/scenario_snapshot.rs`，基于冻结五条标准链路与 `8 SKU × 5 场景` 矩阵解析 `ScenarioSkuSnapshot`，支持显式 `scenario_code`、商品元数据 hint 与唯一映射自动判定；对 `API_SUB`、`RPT_STD` 这类多场景歧义 SKU 在未指明 `scenario_code` 时返回冲突。
  2. `CreateOrderRequest` 新增可选 `scenario_code`，下单价格快照 `OrderPriceSnapshot` 新增可选 `scenario_snapshot`，并在 `trade.order.created` outbox 事件中带出场景-SKU 快照事实。
  3. `freeze_order_price_snapshot(...)` 现在会优先复用订单现有 `scenario_snapshot.scenario_code`，确保歧义 SKU 在补冻结快照时仍能稳定回放到原场景，不会丢失 SKU 角色与模板快照。
  4. 合同确认链路改为校验 `contract.template_definition.template_name` 与 `scenario_snapshot.contract_template` 的冻结模板名，而不是错误拿 UUID 对比；确认后把 `scenario_sku_snapshot` 合并写入 `contract.digital_contract.variables_json`。
  5. 授权迁移链路把 `scenario_sku_snapshot` 写入 `policy_snapshot`，订单详情与生命周期授权视图通过既有聚合读取即可稳定返回按 SKU 冻结后的场景快照。
  6. 锁资前校验的模板完整性检查已纳入 `scenario_snapshot` 必填字段，防止后续链路只存场景名不存 SKU 事实。
  7. 新增 `trade032_scenario_sku_snapshot_db_smoke`，并补齐 `trade003`、`trade008`、`trade009`、`trade010`、`trade021` 的快照断言/seed，使历史链路与新规则一致。
- 涉及文件：
  - `apps/platform-core/src/modules/order/domain/mod.rs`
  - `apps/platform-core/src/modules/order/domain/price_snapshot.rs`
  - `apps/platform-core/src/modules/order/domain/scenario_snapshot.rs`
  - `apps/platform-core/src/modules/order/dto/order_create.rs`
  - `apps/platform-core/src/modules/order/repo/order_authorization_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_create_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_pre_payment_lock_repository.rs`
  - `apps/platform-core/src/modules/order/repo/price_snapshot_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade008_file_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade009_file_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade010_api_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade021_pre_payment_lock_checks_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade032_scenario_sku_snapshot_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/02-openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade008_file_std_state_machine_db_smoke -- --nocapture`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade009_file_sub_state_machine_db_smoke -- --nocapture`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade010_api_sub_state_machine_db_smoke -- --nocapture`
  7. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade021_pre_payment_lock_checks_db_smoke -- --nocapture`
  8. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade032_scenario_sku_snapshot_db_smoke -- --nocapture`
  9. 启动服务：`APP_PORT=8092 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  10. `psql` 写入临时 `API_SUB` / `RPT_STD` 业务数据，`curl` 依次验证：歧义 `API_SUB` 缺少 `scenario_code` 返回 `409`、指定 `S4` 创建成功、补冻结快照成功、`RPT_STD + S5` 以补充 SKU 成功建单、合同确认与授权发放均回传 `scenario_sku_snapshot`。
  11. `psql` 回查 `trade.order_main.price_snapshot_json`、`contract.digital_contract.variables_json`、`trade.authorization_grant.policy_snapshot` 与 `audit.audit_event`，再清理临时业务数据；审计记录按 append-only 保留。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`162 passed, 0 failed, 1 ignored`）。
  - DB smoke：`trade003`、`trade008`、`trade009`、`trade010`、`trade021`、`trade032` 全部通过。
  - 真实 API 联调：
    - 歧义 `API_SUB` 不带 `scenario_code` 建单：`HTTP 409`，消息为 `ORDER_CREATE_FORBIDDEN: scenario_code is required for sku_type \`API_SUB\` because it belongs to multiple frozen scenarios: S1,S4`
    - `API_SUB + S4` 建单：`HTTP 200`，`price_snapshot.scenario_snapshot = S4 / primary`
    - `POST /api/v1/trade/orders/{id}/price-snapshot/freeze`：`HTTP 200`，补冻结后仍为 `S4`
    - `RPT_STD + S5` 建单：`HTTP 200`，`selected_sku_role = supplementary`，`primary_sku = QRY_LITE`
    - `POST /api/v1/orders/{id}/contract-confirm`：`HTTP 200`，`variables_json.scenario_sku_snapshot.scenario_code = S4`
    - `POST /api/v1/orders/{id}/authorization/transition`：`HTTP 200`，`policy_snapshot.scenario_sku_snapshot.scenario_code = S4`
    - `GET /api/v1/orders/{id}`：`HTTP 200`，合同与授权聚合均回传 `scenario_sku_snapshot`
  - DB 回查：
    - `trade.order_main.price_snapshot_json#>>'{scenario_snapshot,scenario_code}' = S4`
    - `contract.digital_contract.variables_json#>>'{scenario_sku_snapshot,selected_sku_type}' = API_SUB`
    - `trade.authorization_grant.policy_snapshot#>>'{scenario_sku_snapshot,selected_sku_type}' = API_SUB`
    - 审计命中：`trade.order.create=1`、`trade.order.price_snapshot.freeze=1`、`trade.contract.confirm=1`、`trade.authorization.grant=1`
  - 清理结果：临时业务数据已清理；回查结果 `order_main=0 | digital_contract=0 | organization=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单聚合与关联事实必须围绕具体订单/SKU 留存）
  - `全集成基线-V1` 15 / 5.3.2 / 5.3.2A（五条标准链路与场景到主/补充 SKU 映射）
  - `业务流程图-V1` 4.3（下单冻结模板/价格/权利/有效期等事实）
- 覆盖的任务清单条目：`TRADE-032`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-134（计划中）
- 状态：计划中
- 当前任务编号：TRADE-025
- 当前批次目标：为授权模块补充 `scope / subject / resource / action` 最小结构，形成 V1 可用且可向 OPA 演进的稳定授权快照。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-024` 已审批通过。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-025` 目标、范围、验收口径与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认任务详细解释，要求补齐授权最小结构而非引入 OPA 依赖。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务批次执行，先记录“计划中”，后编码与验证。
  - `docs/开发任务/AI-Agent-执行提示词.md`：沿用冻结开发规则、审计与 TODO 留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接上一批已审批状态，从 `TRADE-025` 单任务继续。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：延续 `TODO-PROC-BIL-001` 审计追溯要求。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对核心交易链路中授权应作为订单/合同后的可审计聚合。
  - `docs/领域模型/全量领域模型与对象关系说明.md:L530`：合同与策略聚合必须具备可序列化的策略/授权表达。
  - `docs/原始PRD/数据商品元信息与数据契约设计.md:L86`：策略/契约需独立建模，不能只靠零散字段拼装。
- 当前实现与验证计划：
  1. 在授权快照中补充 `scope / subject / resource / action` 最小结构，并保持 V1 现有接口行为兼容。
  2. 将最小结构接入授权迁移结果、订单详情关联视图、生命周期快照。
  3. 补充 `TRADE-025` 专项 DB smoke。
  4. 执行 `cargo fmt --all`、`cargo test -p platform-core`、`TRADE_DB_SMOKE=1 ... trade025_authorization_min_structure_db_smoke`。
  5. 启动服务后执行真实 API 联调，验证授权迁移与订单详情返回的最小结构，并回查审计。

### BATCH-134（待审批）
- 状态：待审批
- 当前任务编号：TRADE-025
- 当前批次目标：为授权模块补充 `scope / subject / resource / action` 最小结构，形成 V1 可用且可向 OPA 演进的稳定授权快照。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-024` 已审批通过。
- 已实现功能：
  1. 新增 `AuthorizationModelSnapshot` 及其 `scope / subject / resource / action` 四段最小结构，约束授权快照显式表达主体、资源、动作与上下文范围。
  2. 在授权迁移写路径中基于 `order/product/sku/policy/grantee/grant_type` 构造 `authorization_model`，并将其规范化写入 `policy_snapshot`。
  3. 在授权迁移返回 DTO、订单详情 `relations.authorizations`、生命周期快照 `authorization` 中统一暴露 `authorization_model`。
  4. 为历史/兜底快照补充 `extract_or_build_authorization_model(...)`，确保旧数据缺少最小结构时仍可由订单上下文回填。
  5. 修复授权聚合查询中 `status/order_id` 联表歧义列，消除真实授权迁移 DB 错误。
  6. 新增 `TRADE-025` 专项 DB smoke，并补充 `TRADE-017 / TRADE-019 / TRADE-022` 断言覆盖授权最小结构。
- 涉及文件：
  - `apps/platform-core/src/modules/authorization/domain/mod.rs`
  - `apps/platform-core/src/modules/order/dto/order_authorization_transition.rs`
  - `apps/platform-core/src/modules/order/dto/order_lifecycle_snapshot.rs`
  - `apps/platform-core/src/modules/order/dto/order_read.rs`
  - `apps/platform-core/src/modules/order/dto/order_relations.rs`
  - `apps/platform-core/src/modules/order/repo/order_authorization_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_lifecycle_snapshot_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_relation_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade017_authorization_aggregate_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade019_lifecycle_snapshots_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade022_order_relations_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade025_authorization_min_structure_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade025_authorization_min_structure_db_smoke -- --nocapture`
  4. 启动最新服务：`APP_PORT=8083 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 插入临时 `SHARE_RO` 业务数据与绑定策略。
  6. `curl POST http://127.0.0.1:8083/api/v1/orders/{id}/authorization/transition` 触发授权发放。
  7. `curl GET http://127.0.0.1:8083/api/v1/orders/{id}` 校验订单详情聚合中的 `authorization_model`。
  8. `psql` 回查 `trade.authorization_grant.policy_snapshot` 与 `audit.audit_event`，然后清理临时业务数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`147 passed, 0 failed, 1 ignored`）。
  - `trade025_authorization_min_structure_db_smoke`：通过。
  - 真实 API 联调：`POST /api/v1/orders/80935453-55ee-40f7-b841-9f09561d11db/authorization/transition` 返回 `HTTP 200`，`authorization_model.scope.order_id / resource.sku_id / subject.subject_id / action.grant_type` 全部正确。
  - 真实 API 联调：`GET /api/v1/orders/80935453-55ee-40f7-b841-9f09561d11db` 返回 `HTTP 200`，`relations.authorizations[0].authorization_model` 与授权迁移返回保持一致。
  - DB 回查：`trade.authorization_grant.policy_snapshot` 已持久化 `scope / subject / resource / action`；`audit.audit_event` 命中 `trade.authorization.grant` 与 `trade.order.read`。
  - 清理结果：临时业务测试数据已清理；审计记录按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `领域模型/全量领域模型与对象关系说明.md` 4.3（合同与策略聚合）
  - `原始PRD/数据商品元信息与数据契约设计.md` 3.2（策略/契约独立建模）
  - `全集成文档/数据交易平台-全集成基线-V1.md` 15（交易链路中的授权聚合与审计）
- 覆盖的任务清单条目：`TRADE-025`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-135（计划中）
- 状态：计划中
- 当前任务编号：TRADE-026
- 当前批次目标：为合同模块补充电子签章 Provider 占位与 mock 实现；`local` 模式下合同确认链路不依赖真实签章服务。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-025` 已审批通过。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-026` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认本任务重点是“签章 provider 占位 + local mock”，不是引入真实外部服务。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务批次执行“计划中 -> 编码 -> 完整验证 -> 待审批”。
  - `docs/开发任务/AI-Agent-执行提示词.md`：只做当前任务，不跨任务扩展。
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 P2 主线批次，保持相同审计格式。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：合同步骤必须经过电子签章，签署后进入后续支付/锁定链路。
  - `docs/领域模型/全量领域模型与对象关系说明.md:L530`：`DigitalContract` 表示订单签署时的合同快照，应明确签署事实与数据契约绑定。
  - `docs/原始PRD/数据商品元信息与数据契约设计.md:L86`：契约相关事实必须独立建模，不能只靠元数据拼接。
- technical_reference 约束映射：
  1. `领域模型/全量领域模型与对象关系说明.md` 4.3：数字合同应承载正式签署结果，签署能力应有明确边界。
  2. `原始PRD/数据商品元信息与数据契约设计.md` 3.2：契约事实应独立建模，不依赖松散 metadata。
  3. `全集成文档/数据交易平台-全集成基线-V1.md` 15：统一主链路第 5 步包含电子签章，local 模式允许 mock 实现。
- 预计涉及文件：
  - `apps/platform-core/src/modules/contract/**`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/dto/order_contract_confirm.rs`
  - `apps/platform-core/src/modules/order/tests/*`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture`
  4. 启动服务：`APP_PORT=8083 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `curl POST /api/v1/orders/{id}/contract-confirm` 验证 local/mock 签章结果已进入响应、持久化与审计。

### BATCH-135（待审批）
- 状态：待审批
- 当前任务编号：TRADE-026
- 当前批次目标：为合同模块补充电子签章 Provider 占位与 mock 实现；`local` 模式下合同确认链路不依赖真实签章服务。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-025` 已审批通过。
- 已实现功能：
  1. 在 `modules/contract/application` 新增签章应用层，按 `RuntimeConfig.PROVIDER_MODE` 选择 `provider-kit` 的 `mock/real SigningProvider`。
  2. 合同确认链路改为通过签章 provider 生成签章引用，再写入 `contract.contract_signer.signature_digest`。
  3. 合同确认响应新增 `signature_provider_mode / signature_provider_kind / signature_provider_ref`，明确签章 provider 占位结果。
  4. `local` 模式默认走 `mock` provider，无需真实电子签章服务即可完成合同确认。
  5. 新增 `TRADE-026` 专项 DB smoke，并补充 `TRADE-006 / TRADE-016` 断言，验证 mock provider 已实际接入主链路。
- 涉及文件：
  - `apps/platform-core/src/modules/contract/mod.rs`
  - `apps/platform-core/src/modules/contract/application/mod.rs`
  - `apps/platform-core/src/modules/contract/application/signing.rs`
  - `apps/platform-core/src/modules/order/dto/order_contract_confirm.rs`
  - `apps/platform-core/src/modules/order/repo/order_contract_repository.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade006_contract_confirm_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade016_digital_contract_aggregate_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade026_contract_signing_provider_db.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture`
  4. 启动最新服务：`APP_PORT=8084 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 插入临时 `FILE_STD` 合同待确认订单、模板与签署用户。
  6. `curl POST http://127.0.0.1:8084/api/v1/orders/{id}/contract-confirm` 验证返回 mock provider 信息。
  7. `curl GET http://127.0.0.1:8084/api/v1/orders/{id}` 验证合同已进入 `contract_effective`。
  8. `psql` 回查 `contract.contract_signer.signature_digest` 与 `audit.audit_event`，然后清理临时业务数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`148 passed, 0 failed, 1 ignored`）。
  - `trade026_contract_signing_provider_db_smoke`：通过。
  - 真实 API 联调：`POST /api/v1/orders/3fa6a69f-d0ce-407f-ae29-217460b1ae5d/contract-confirm` 返回 `HTTP 200`，`signature_provider_mode=mock`、`signature_provider_kind=mock`、`signature_provider_ref=mock-signing-ok:6738b03f-8b33-49a9-8e05-dd586cd4be35:SIGN-...`。
  - 真实 API 联调：`GET /api/v1/orders/3fa6a69f-d0ce-407f-ae29-217460b1ae5d` 返回 `HTTP 200`，订单主状态已推进到 `contract_effective`，合同聚合为 `signed`。
  - DB 回查：`contract.contract_signer.signature_digest` 已持久化 mock provider 引用；`audit.audit_event` 命中 `trade.contract.confirm` 与 `trade.order.read`。
  - 清理结果：临时业务测试数据已清理；审计记录按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `领域模型/全量领域模型与对象关系说明.md` 4.3（数字合同聚合）
  - `原始PRD/数据商品元信息与数据契约设计.md` 3.2（契约事实独立建模）
  - `全集成文档/数据交易平台-全集成基线-V1.md` 15（统一主链路第 5 步电子签章）
- 覆盖的任务清单条目：`TRADE-026`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-136（计划中）
- 状态：计划中
- 当前任务编号：TRADE-027
- 当前批次目标：为主交易链路补充集成测试，覆盖下单、合同确认、锁资前校验、非法状态跳转、自动断权。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-026` 已审批通过。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-027` 验收要求是主链路集成测试，不是新增业务能力。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认需覆盖五类关键节点：下单、合同确认、锁资前校验、非法状态跳转、自动断权。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务批次执行“计划中 -> 编码 -> 完整验证 -> 待审批”。
  - `docs/开发任务/AI-Agent-执行提示词.md`：只做当前任务，不跨任务扩展。
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 P2 主线批次审计格式。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  - `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单聚合必须贯穿完整交易主链路。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：主链路至少包含合同、锁定、交付/授权等关键步骤。
  - `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方搜索、选购与下单流程要求形成从发现到交易推进的可验证链路。
- technical_reference 约束映射：
  1. `领域模型/全量领域模型与对象关系说明.md` 4.4：订单聚合是完整交易实例，集成测试应覆盖状态推进与关联对象。
  2. `全集成文档/数据交易平台-全集成基线-V1.md` 15：主链路需覆盖合同、付款锁定、授权/交付关键节点。
  3. `业务流程图-V1-完整版.md` 4.3：买方选购、下单、后续交易动作需可串联验证。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/tests/*`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade027_main_trade_flow_db_smoke -- --nocapture`
  4. 启动服务：`APP_PORT=8085 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `curl` 真实联调覆盖：下单、合同确认、锁资前校验阻断/成功、非法状态跳转冲突、自动断权结果与审计。

### BATCH-126（计划中）
- 状态：计划中
- 当前任务编号：TRADE-017
- 当前批次目标：实现授权聚合：Authorization、UsagePolicy、grant、revoke、expire、suspend、恢复。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-016` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-017` 描述、DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-017` 详细语义与顺序执行要求。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：遵循单任务批次与不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续批次编号与记录格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：确认 `TODO-PROC-BIL-001` 持续追溯要求。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认核心交易链授权节点与审计约束。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认授权聚合归属 `platform-core` 且 PostgreSQL 为权威。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：对齐交易接口对象化字段与 OpenAPI 一致性。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认授权动作需有可追溯审计留痕。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` / `IAM_UNAUTHORIZED` / `AUTHORIZATION_NOT_ACTIVE` 口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：对齐交易主链授权状态与审计验证要求。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：保持 order/authorization 分层与可审查性。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：接口联调沿用环境变量注入方式。
  17. `docs/开发准备/技术选型正式版.md`：遵循“业务真值回 PostgreSQL”。
  18. `docs/开发准备/平台总体架构设计草案.md`：维持模块化单体内聚实现，不引入越界能力。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L530`：`Authorization` 作为独立聚合对象，`UsagePolicy 1 -> N Authorization`，实现层可用 `authorization_grant` 存储名。
  2. `docs/原始PRD/数据商品查询与执行面设计.md:L185`：授权必须表达“主体、模板/范围、时效、导出限制”，并可与审计联动。
  3. `docs/原始PRD/敏感数据处理与受控交付设计.md:L124`：敏感数据交易需策略快照化，支持风控收口与生命周期控制（含暂停/恢复）。

### BATCH-126（待审批）
- 状态：待审批
- 当前任务编号：TRADE-017
- 当前批次目标：实现授权聚合：Authorization、UsagePolicy、grant、revoke、expire、suspend、恢复。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-016` 已审批通过。
- 已实现功能：
  1. 新增订单授权聚合接口：`POST /api/v1/orders/{id}/authorization/transition`，支持 `grant/revoke/expire/suspend/recover`。
  2. 新增授权聚合仓储状态机：
     - `grant`：写入 `trade.authorization_grant(active)`，回填 `trade.order_main.policy_id`；
     - `suspend/recover`：在 `active <-> suspended` 间切换；
     - `revoke/expire`：终态落 `revoked/expired` 且补齐 `valid_to`。
  3. 新增 `UsagePolicy` 解析与绑定规则：优先 `payload.policy_id`，其次 `order_main.policy_id`，否则回退 `contract.policy_binding(product/sku)`。
  4. 新增授权聚合审计动作：`trade.authorization.grant/revoke/expire/suspend/recover`。
  5. 新增 `TRADE-017` DB smoke：覆盖 `grant->suspend->recover->expire` 与 `grant->revoke` 两条链路。
  6. 更新 OpenAPI：新增 path 与请求/响应 schema，保持既有接口不变。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_authorization_transition.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_authorization_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade017_authorization_aggregate_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade017_authorization_aggregate_db_smoke -- --nocapture`
  4. `make up-local`
  5. `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
  6. 启动服务：`APP_PORT=18085 DATABASE_URL=... KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
  7. `psql` 插入联调测试数据（buyer/seller/asset/version/product/sku/usage_policy/policy_binding/order）。
  8. `curl` 调用 `grant -> suspend -> recover -> expire`。
  9. `psql` 回查 `trade.authorization_grant`、`trade.order_main.policy_id`、`audit.audit_event`。
  10. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`134 passed, 0 failed, 1 ignored`）。
  - `trade017_authorization_aggregate_db_smoke`：通过（`1 passed`，连接 `datab-postgres:5432`）。
  - `make up-local`：通过，核心容器就绪。
  - `check-local-stack core`：脚本仍报告 `5432` 不可达；但后续 `psql`、DB smoke、`curl` 全部成功，判定为脚本可达性噪声。
  - API 联调（新鲜测试数据）：
    - `grant` 返回 `HTTP 200`，`current_status=active`；
    - `suspend` 返回 `HTTP 200`，`current_status=suspended`；
    - `recover` 返回 `HTTP 200`，`current_status=active`；
    - `expire` 返回 `HTTP 200`，`current_status=expired`。
  - DB 回查：
    - `trade.authorization_grant.status=expired` 且 `valid_to` 已写入；
    - `trade.order_main.policy_id` 正确回填为策略 ID；
    - 审计计数：`trade.authorization.grant/suspend/recover/expire` 各 `1`。
  - 清理结果：临时业务数据已清理；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.3（`UsagePolicy 1->N Authorization`，授权作为独立聚合）
  - `原始PRD/数据商品查询与执行面设计` 8.1（授权表达主体/范围/时效/导出约束）
  - `原始PRD/敏感数据处理与受控交付设计` 5（策略快照化与敏感链路风控收口）
  - `数据库表字典` `trade.authorization_grant`、`contract.usage_policy` 字段口径
- 覆盖的任务清单条目：`TRADE-017`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-127（计划中）
- 状态：计划中
- 当前任务编号：TRADE-018
- 当前批次目标：实现基础断权机制：订单取消、到期、风控冻结、争议升级后自动触发交付入口断权。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-017` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-018` DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-018` 为单任务批次且不可跳步。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：编码前先写计划中，保持冻结约束。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续批次日志格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认交易链授权与争议闭环要求。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：断权逻辑应内聚在交易主编排（order/authorization）。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：保持既有接口语义并对象化状态返回。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：授权撤销/暂停需可审计追踪。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用交易与授权错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：覆盖生命周期与争议分支的状态一致性验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：维持 order/repo/dto/tests 分层。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不引入额外配置项。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 持续作为业务权威状态。
  18. `docs/开发准备/平台总体架构设计草案.md`：遵循模块化单体内聚，不拆分外部新服务。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L530`：`Authorization` 是独立聚合，需随订单关键状态变化自动收口。
  2. `docs/原始PRD/数据商品查询与执行面设计.md:L185`：授权与审计强关联，阻断动作需落审计证据。
  3. `docs/原始PRD/敏感数据处理与受控交付设计.md:L124`：高风险/争议场景必须策略收口并阻断交付入口。

### BATCH-127（待审批）
- 状态：待审批
- 当前任务编号：TRADE-018
- 当前批次目标：实现基础断权机制：订单取消、到期、风控冻结、争议升级后自动触发交付入口断权。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-017` 已审批通过。
- 已实现功能：
  1. 新增统一自动断权仓储钩子 `apply_authorization_cutoff_if_needed(...)`，在订单状态迁移事务内自动收口授权。
  2. 断权目标状态规则落地：
     - 订单取消/关闭/撤销 -> `revoked`
     - 到期 -> `expired`
     - 风控冻结/争议升级 -> `suspended`
  3. 自动断权接入以下迁移仓储：`cancel`、`file-std`、`file-sub`、`api-sub`、`api-ppu`、`share-ro`、`sbx-std`。
  4. 补齐审计动作：`trade.authorization.auto_cutoff.revoked|expired|suspended`。
  5. 将 `api-ppu disable_access` 风控语义 reason_code 对齐为 `api_ppu_risk_frozen`，确保自动断权判定一致。
  6. 新增 `TRADE-018` DB smoke，覆盖 cancel/expire/dispute/risk 四触发路径与授权状态/审计断言。
- 涉及文件：
  - `apps/platform-core/src/modules/order/repo/order_authorization_cutoff_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_cancel_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_ppu_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_share_ro_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_sbx_std_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade018_auto_cutoff_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade018_auto_cutoff_db_smoke -- --nocapture`
  4. 启动服务：`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
  5. `psql` 插入联调测试数据（buyer/seller/asset/version/product/sku/policy/order）。
  6. `curl` 调用：
     - `POST /api/v1/orders/{id}/authorization/transition`（4个订单 `action=grant`）
     - `POST /api/v1/orders/{id}/cancel`
     - `POST /api/v1/orders/{id}/share-ro/transition`（`action=expire_share`）
     - `POST /api/v1/orders/{id}/share-ro/transition`（`action=interrupt_dispute`）
     - `POST /api/v1/orders/{id}/api-ppu/transition`（`action=disable_access`）
  7. `psql` 回查 `trade.authorization_grant` 与 `audit.audit_event`。
  8. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`136 passed, 0 failed, 1 ignored`）。
  - `trade018_auto_cutoff_db_smoke`：通过（`1 passed`，连接 `datab-postgres:5432`）。
  - API 联调（新鲜数据）全部返回 `HTTP 200`，并出现预期状态：
    - cancel -> `closed`；expire_share -> `expired`；interrupt_dispute -> `dispute_interrupted`；disable_access -> `disabled`。
  - DB 回查授权状态：
    - cancel 订单 `revoked`
    - expire 订单 `expired`
    - dispute 订单 `suspended`
    - risk 订单 `suspended`
  - 审计回查计数：
    - `trade.authorization.auto_cutoff.revoked` = 1
    - `trade.authorization.auto_cutoff.expired` = 1
    - `trade.authorization.auto_cutoff.suspended` = 2
  - 清理结果：临时业务数据已清理；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.3（授权聚合独立、生命周期收口）
  - `原始PRD/数据商品查询与执行面设计` 8.1（授权与审计联动）
  - `原始PRD/敏感数据处理与受控交付设计` 5（争议/风控场景收口与断权）
- 覆盖的任务清单条目：`TRADE-018`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-128（计划中）
- 状态：计划中
- 当前任务编号：TRADE-019
- 当前批次目标：实现生命周期摘要接口 `GET /api/v1/orders/{id}/lifecycle-snapshots`，返回对象化字段名而不是拼装字符串。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-018` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-019` DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-019` 的接口与对象化返回要求。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次、不可跳步。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续批次记录格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认交易闭环中的订单生命周期观测需求。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认生命周期摘要接口归属 `order/platform-core`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 `GET /api/v1/orders/{id}/lifecycle-snapshots` 已冻结。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：读取类接口需留审计动作。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `IAM_UNAUTHORIZED` 与 `TRD_STATE_CONFLICT`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：覆盖生命周期状态读取与审计验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增 DTO/Repository 按模块职责拆分。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调继续使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 作为交易状态权威源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体内聚实现。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单聚合需显式输出主状态与支付/交付/验收/结算/争议等子状态。
  2. `docs/原始PRD/审计、证据链与回放设计.md:L93`：接口动作应写统一审计事件并带 request/trace 关联字段。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：交易闭环需可回放、可联查的生命周期摘要视图。

### BATCH-128（待审批）
- 状态：待审批
- 当前任务编号：TRADE-019
- 当前批次目标：实现生命周期摘要接口 `GET /api/v1/orders/{id}/lifecycle-snapshots`，返回对象化字段名而不是拼装字符串。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-018` 已审批通过。
- 已实现功能：
  1. 新增生命周期摘要接口：`GET /api/v1/orders/{id}/lifecycle-snapshots`。
  2. 新增生命周期 DTO，按对象化结构拆分：`order/payment/acceptance/settlement/dispute/contract/authorization/delivery`。
  3. 新增生命周期聚合查询仓储：从 `trade.order_main` + `contract.digital_contract` + `trade.authorization_grant` + `delivery.delivery_record` 组装摘要。
  4. 新接口接入 `ReadOrder` 权限矩阵与租户范围校验（buyer/seller scope）。
  5. 新增审计动作：`trade.order.lifecycle_snapshots.read`。
  6. 新增 `TRADE-019` DB smoke，验证接口响应对象结构与审计留痕。
  7. 更新 OpenAPI：新增 path 与对应 schema，保持字段语义与实现一致。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_lifecycle_snapshot.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_lifecycle_snapshot_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade019_lifecycle_snapshots_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade019_order_lifecycle_snapshots_db_smoke -- --nocapture`
  4. 启动服务：`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
  5. `psql` 插入临时联调数据（org/asset/version/product/sku/order/contract/authorization/delivery）。
  6. `curl` 调用 `GET /api/v1/orders/{id}/lifecycle-snapshots`。
  7. `psql` 回查审计 `trade.order.lifecycle_snapshots.read` 与授权状态。
  8. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`138 passed, 0 failed, 1 ignored`）。
  - `trade019_order_lifecycle_snapshots_db_smoke`：通过（`1 passed`，连接 `datab-postgres:5432`）。
  - API 联调：`GET /api/v1/orders/{id}/lifecycle-snapshots` 返回 `HTTP 200`，`order/contract/authorization/delivery` 对象均正确返回。
  - 审计回查：`trade.order.lifecycle_snapshots.read = 1`。
  - 状态回查：授权记录 `status=active` 与响应一致。
  - 清理结果：临时业务数据已清理；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单聚合主状态与子状态显式建模）
  - `原始PRD/审计、证据链与回放设计` 4（统一审计事件模型）
  - `全集成基线-V1` 15（交易闭环可观测、可回放）
  - `接口清单与OpenAPI-Schema冻结表` 5.6（`GET /api/v1/orders/{id}/lifecycle-snapshots`）
- 覆盖的任务清单条目：`TRADE-019`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-129（计划中）
- 状态：计划中
- 当前任务编号：TRADE-020
- 当前批次目标：实现订单创建事务：业务对象 + 审计事件 + outbox 事件同事务落库。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-019` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-020` DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-020` 聚焦“创建事务 + 审计 + outbox 同事务”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次、流程步骤完整。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续批次记录格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认交易闭环中的订单创建与后续编排衔接。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认订单主编排归属 `platform-core/order`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：订单创建接口冻结口径不变。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：业务写路径需有事件可追踪。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` 与 `IAM_UNAUTHORIZED`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐事务一致性与审计/事件验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：保持 repo/dto/tests 分层，避免单文件继续膨胀。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 作为业务真值与事务权威。
  18. `docs/开发准备/平台总体架构设计草案.md`：遵循模块化单体内聚。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单聚合创建必须落主状态及子状态快照。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：交易闭环要求创建事件可追踪并可驱动后续环节。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：下单阶段需产出订单对象、审计记录与事件草稿。

### BATCH-129（待审批）
- 状态：待审批
- 当前任务编号：TRADE-020
- 当前批次目标：实现订单创建事务：业务对象 + 审计事件 + outbox 事件同事务落库。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-019` 已审批通过。
- 已实现功能：
  1. 在订单创建事务 `create_order_with_snapshot(...)` 中新增显式 outbox 写入：`ops.outbox_event`，事件 `trade.order.created`。
  2. 订单业务对象落库、审计写入 `trade.order.create`、outbox 写入 `trade.order.created` 全部在同一事务中执行，统一在 `tx.commit()` 后生效。
  3. outbox payload 增加订单核心字段（order/buyer/seller/product/sku/status/payment/amount/currency/created_at）用于后续事件消费。
  4. 扩展 `TRADE-003` DB smoke：新增 outbox 断言与清理逻辑，确保创建链路最小闭环可回归。
- 涉及文件：
  - `apps/platform-core/src/modules/order/repo/order_create_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade003_create_order_db.rs`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade003_create_order_db_smoke -- --nocapture`
  4. 启动服务：`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 cargo run -p platform-core`
  5. `psql` 插入联调测试数据（buyer/seller/asset/version/product/sku）。
  6. `curl` 调用 `POST /api/v1/orders`（带 `x-request-id/x-idempotency-key`）。
  7. `psql` 回查：`trade.order_main`、`audit.audit_event`、`ops.outbox_event`。
  8. 清理临时业务与 outbox 测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`138 passed, 0 failed, 1 ignored`）。
  - `trade003_create_order_db_smoke`：通过（`1 passed`，包含 outbox 断言）。
  - API 联调：`POST /api/v1/orders` 返回 `HTTP 200`，订单状态 `created/unpaid` 正确。
  - DB 回查：
    - `trade.order_main` 命中创建订单；
    - 审计事件 `trade.order.create = 1`；
    - outbox 事件 `aggregate_type=trade.order`、`event_type=trade.order.created`、`status=pending` 命中。
  - 清理结果：临时业务与 outbox 测试数据已清理；审计记录保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单聚合创建状态与子状态落库）
  - `全集成基线-V1` 15（交易闭环创建阶段事件可追踪）
  - `业务流程图-V1` 4.3（下单阶段订单对象+审计+事件草稿）
- 覆盖的任务清单条目：`TRADE-020`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-130（计划中）
- 状态：计划中
- 当前任务编号：TRADE-021
- 当前批次目标：实现支付锁定前的前置校验：主体状态、商品状态、审核状态、模板齐备、价格快照完整。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-020` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-021` DoD、验收与 technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认 `TRADE-021` 前置校验范围与强制项。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中→编码→验证→待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次执行，不跳步骤。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续批次审计记录格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认（按约定不写入）。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认支付锁定前不得绕过主体/商品/审核门禁。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交易门禁属于 `platform-core/order` 聚合边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认不新增路径，仅在既有 transition 路径加门禁。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：失败场景不新增事件，成功场景维持既有审计动作。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT`。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐门禁失败/成功分支验证。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立仓储与专项测试文件，避免继续膨胀单文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 作为交易门禁权威数据源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体内聚。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单主状态推进必须受聚合前置约束保护。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：付款/锁定前需通过主体、商品、审核等门禁。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：下单后锁款前需校验商品与主体有效性、快照完整性。

### BATCH-130（待审批）
- 状态：待审批
- 当前任务编号：TRADE-021
- 当前批次目标：实现支付锁定前的前置校验：主体状态、商品状态、审核状态、模板齐备、价格快照完整。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-020` 已审批通过。
- 已实现功能：
  1. 新增统一前置校验仓储 `ensure_pre_payment_lock_checks(...)`，并接入 `FILE_STD.lock_funds`、`FILE_SUB.establish_subscription/renew_subscription`、`API_SUB.lock_funds`。
  2. 落地门禁校验项：买卖主体状态、商品状态、资产版本审核态（`active/published`）、SKU 可售态、产品审核态（`metadata.review_status`）、风控阻断标记、价格快照完整性、模板快照完整性。
  3. 失败统一返回 `409 TRD_STATE_CONFLICT`，错误前缀统一为 `ORDER_PRE_LOCK_CHECK_FAILED:`。
  4. 修复受影响历史 smoke seed：`trade008`、`trade009`、`trade010` 补齐完整 `price_snapshot_json`。
  5. 新增 `TRADE-021` 专项 DB smoke：覆盖“审核态拦截 -> 快照拦截 -> 通过门禁后成功锁款”完整链路。
- 涉及文件：
  - `apps/platform-core/src/modules/order/repo/order_pre_payment_lock_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_std_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_file_sub_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_api_sub_repository.rs`
  - `apps/platform-core/src/modules/order/tests/trade008_file_std_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade009_file_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade010_api_sub_state_machine_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade021_pre_payment_lock_checks_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade008_file_std_state_machine_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade009_file_sub_state_machine_db_smoke -- --nocapture`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade010_api_sub_state_machine_db_smoke -- --nocapture`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade021_pre_payment_lock_checks_db_smoke -- --nocapture`
  7. 启动服务：`KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. `psql` 插入临时测试数据（org/asset/version/product/sku/order），`curl` 调用 `POST /api/v1/orders/{id}/file-std/transition` 三次验证（审核态拦截、快照拦截、通过成功），`psql` 回查订单状态与审计计数。
  9. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`139 passed, 0 failed, 1 ignored`）。
  - `trade008/trade009/trade010` DB smoke：通过。
  - `trade021_pre_payment_lock_checks_db_smoke`：通过。
  - API 联调结果：
    - 第一次（审核态不通过）返回 `409`：`ORDER_PRE_LOCK_CHECK_FAILED: product review status is not approved`
    - 第二次（快照不完整）返回 `409`：`ORDER_PRE_LOCK_CHECK_FAILED: price snapshot is incomplete`
    - 第三次（补齐后）返回 `200`，订单推进到 `buyer_locked/paid`。
  - DB 回查：`trade.order_main` 命中 `buyer_locked|paid|file_std_lock_funds`；`audit.audit_event` 命中 `trade.order.file_std.transition = 1`。
  - 清理结果：临时业务数据已清理；审计记录按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单状态推进门禁）
  - `全集成基线-V1` 15（付款锁定前校验）
  - `业务流程图-V1` 4.3（下单至锁款前校验口径）
- 覆盖的任务清单条目：`TRADE-021`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-131（计划中）
- 状态：计划中
- 当前任务编号：TRADE-022
- 当前批次目标：实现订单与合同/授权/交付/账单/争议的一对一或一对多关系装配器，补齐订单详情页与审计联查所需只读聚合。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-021` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-022` DoD、验收要求与 `technical_reference`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认任务目标是“关系装配器”，不是新增交易动作。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 编码 -> 验证 -> 待审批”冻结流程。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次执行，不跳步骤，不省略联调验证。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续 P2 批次审计格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：订单详情页必须展示主链路、交付、账单、争议与审计联查入口。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：订单详情聚合属于 `platform-core/order` 读模型装配边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：维持 `/api/v1/orders/{id}` 既有接口，对外逻辑字段优先。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务为只读聚合，不新增事件主题。
  12. `docs/开发准备/统一错误码字典正式版.md`：继续沿用既有 `NOT_FOUND/FORBIDDEN/CONFLICT` 语义，不发明新错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：生命周期对象 `Authorization/Delivery/Settlement/Dispute/Billing Event` 必须可联查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：使用独立 relation assembler 仓储与专项测试文件，避免继续膨胀单仓储。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库使用 `datab-postgres:5432`，本地服务需连通 Kafka/Redis 等基础栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项，沿用现有 `DATABASE_URL/KAFKA_*`。
  17. `docs/开发准备/技术选型正式版.md`：PostgreSQL 为订单关系聚合权威查询源。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体内聚，避免跨模块写入耦合。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：`Order` 聚合需联通 `contract.digital_contract`、`trade.authorization_grant`、`delivery.delivery_record`、`billing.*`、`support.dispute_case`。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：核心交易闭环要求合同、授权、交付、账单、争议可围绕订单主键联查。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方下单后详情页与后续链路需要围绕订单对象查看状态、交付、账务和争议摘要。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_read.rs`
  - `apps/platform-core/src/modules/order/repo/order_read_repository.rs`
  - `apps/platform-core/src/modules/order/repo/*`（新增关系装配辅助仓储）
  - `apps/platform-core/src/modules/order/tests/trade004_order_detail_db.rs`
  - `apps/platform-core/src/modules/order/tests/*`（新增 `TRADE-022` 专项测试）
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade004_order_detail_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade022_order_relations_db_smoke -- --nocapture`
  5. 启动服务：`KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `psql` 插入临时订单关系数据，`curl` 调用 `GET /api/v1/orders/{id}`，回查审计与关系对象字段，再清理临时业务数据。

### BATCH-131（待审批）
- 状态：待审批
- 当前任务编号：TRADE-022
- 当前批次目标：实现订单与合同/授权/交付/账单/争议的一对一或一对多关系装配器，补齐订单详情页与审计联查所需只读聚合。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-021` 已审批通过。
- 已实现功能：
  1. 为 `GET /api/v1/orders/{id}` 新增稳定 `relations` 聚合对象，保留既有订单核心字段与权限/审计行为不变。
  2. 新增独立关系装配仓储 `load_order_relations(...)`，按订单主键装配：
     - `contract`：一对一数字合同关系；
     - `authorizations`：授权记录数组；
     - `deliveries`：交付记录数组；
     - `billing`：账单事件、结算、退款、赔付、发票数组；
     - `disputes`：争议记录数组，并附带 `evidence_count`。
  3. `order_read_repository` 仅负责订单主记录读取与关系仓储拼装，避免把跨域查询继续堆进单文件。
  4. 更新 OpenAPI `GetOrderDetailResponseData`，对外冻结新增关系对象 schema，不改接口路径。
  5. 回归 `TRADE-004` 详情读取 smoke，新增 `TRADE-022` 专项 smoke，覆盖空关系与完整关系两种读场景。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_relations.rs`
  - `apps/platform-core/src/modules/order/dto/order_read.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_relation_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_read_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade004_order_detail_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade022_order_relations_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade004_order_detail_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade022_order_relations_db_smoke -- --nocapture`
  5. 启动服务：`KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `psql` 插入临时订单关系数据。
  7. `curl GET http://127.0.0.1:8080/api/v1/orders/{id}`（带 `x-role/x-tenant-id/x-request-id/x-trace-id`）。
  8. `psql` 回查 `audit.audit_event` 的 `trade.order.read` 记录。
  9. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`140 passed, 0 failed, 1 ignored`）。
  - `trade004_order_detail_db_smoke`：通过。
  - `trade022_order_relations_db_smoke`：通过。
  - API 联调：`GET /api/v1/orders/{id}` 返回 `HTTP 200`，关系装配摘要为：
    - `contract_status=signed`
    - `authorizations=2`
    - `deliveries=2`
    - `billing_events=2`
    - `disputes=2`
  - DB 回查：`audit.audit_event` 命中 `trade.order.read = 1`。
  - 清理结果：临时业务数据已清理；审计记录按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单主对象与合同/授权/交付/账单/争议关系）
  - `全集成基线-V1` 15（核心交易闭环围绕订单主键联查）
  - `业务流程图-V1` 4.3（下单后详情/后续链路需回看订单相关对象）
- 覆盖的任务清单条目：`TRADE-022`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-132（计划中）
- 状态：计划中
- 当前任务编号：TRADE-023
- 当前批次目标：实现五条标准链路的订单模板，固化场景到主 SKU / 可选补充 SKU / 合同模板 / 验收模板 / 退款模板 / 交易流程骨架的交易侧只读模板视图。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-022` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-023` 目标、DoD 与 `technical_reference`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务是“五条标准链路订单模板”，不是新增状态机或支付流程。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 编码 -> 验证 -> 待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次，不跳步骤。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：沿用 P2 批次审计格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：冻结五条标准链路与场景到 SKU/模板映射表。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：订单模板属于 `platform-core/order` 交易侧读模型。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前未冻结现成交易模板路径，可新增最小只读接口并保持逻辑字段命名。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务为静态只读模板，不新增事件主题。
  12. `docs/开发准备/统一错误码字典正式版.md`：复用 `FORBIDDEN/NOT_FOUND/CONFLICT` 体系，不发明新错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：五条标准链路需支持首批端到端验证，八个标准 SKU 必须全部挂到正式场景。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立订单模板 DTO/静态模板源/专项测试，避免堆入已有 handler 大块逻辑。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：静态模板由 Rust 常量视图输出，审计落 PostgreSQL。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体内聚。
- technical_reference 约束映射：
  1. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L216`：五条标准链路必须覆盖工业制造/供应链、零售/本地生活两个首批行业。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：场景必须映射到主标准 SKU、可选补充 SKU、合同模板、验收模板、退款模板，并覆盖全部 8 个 V1 SKU。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L66`：模板需要与主业务流程对齐，为下单页/合同页/支付锁定页提供可复用标准链路骨架。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/dto/*`
  - `apps/platform-core/src/modules/order/api/*`
  - `apps/platform-core/src/modules/order/tests/*`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade023_order_templates_db_smoke -- --nocapture`
  4. 启动服务：`KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `curl` 调用交易模板接口，验证五条模板、八个 SKU 覆盖与审计记录。

### BATCH-132（待审批）
- 状态：待审批
- 当前任务编号：TRADE-023
- 当前批次目标：实现五条标准链路的订单模板，固化场景到主 SKU / 可选补充 SKU / 合同模板 / 验收模板 / 退款模板 / 交易流程骨架的交易侧只读模板视图。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-022` 已审批通过。
- 已实现功能：
  1. 新增 `GET /api/v1/orders/standard-templates`，输出五条冻结标准链路订单模板。
  2. 固化 `S1~S5` 场景到交易侧模板视图，覆盖主 SKU、补充 SKU、合同模板、验收模板、退款模板、流程步骤与订单草稿骨架。
  3. 通过聚合映射保证八个 V1 标准 SKU 全部出现在五条链路模板中。
  4. 接口接入现有 `TradePermission::ReadOrder` 权限校验与 `trade.order.templates.read` 审计记录。
  5. OpenAPI 补齐 `OrderTemplateView` 与响应契约，保持只读查询语义。
- 涉及文件：
  - `apps/platform-core/src/modules/order/dto/order_template.rs`
  - `apps/platform-core/src/modules/order/dto/mod.rs`
  - `apps/platform-core/src/modules/order/order_templates.rs`
  - `apps/platform-core/src/modules/order/mod.rs`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/api/mod.rs`
  - `apps/platform-core/src/modules/order/tests/trade023_order_templates_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade023_order_templates_db_smoke -- --nocapture`
  4. `curl -H 'x-user-id: 00000000-0000-0000-0000-000000000001' -H 'x-tenant-id: 11111111-1111-1111-1111-111111111111' -H 'x-role: tenant_admin' -H 'x-request-id: trade023-api-001' http://127.0.0.1:8080/api/v1/orders/standard-templates`
  5. `psql postgresql://datab:datab_local_pass@127.0.0.1:5432/datab -At -F $'\t' -c "select action_name, ref_type, ref_id, request_id from audit.audit_event where request_id = 'trade023-api-001' order by event_time desc limit 5;"`
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`143 passed, 0 failed, 1 ignored`）。
  - `trade023_order_templates_db_smoke`：通过。
  - API 联调：`GET /api/v1/orders/standard-templates` 返回 `HTTP 200`，共 `5` 条模板，场景覆盖 `S1,S2,S3,S4,S5`。
  - API 联调：返回模板覆盖八个标准 SKU：`API_SUB, API_PPU, FILE_STD, FILE_SUB, SBX_STD, SHARE_RO, QRY_LITE, RPT_STD`。
  - API 联调：每条模板的 `order_draft.per_sku_snapshot_required=true`，符合冻结要求。
  - DB 回查：`audit.audit_event` 命中 `trade.order.templates.read / trade_order_templates / 00000000-0000-0000-0000-000000000123 / trade023-api-001`。
- 覆盖的冻结文档条目：
  - `数据交易平台-全集成基线-V1` 5.3.2（五条标准链路）
  - `数据交易平台-全集成基线-V1` 5.3.2A（场景到 SKU/模板映射）
  - `业务流程图-V1-完整版` 4.1（主业务流程骨架）
- 覆盖的任务清单条目：`TRADE-023`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-133（计划中）
- 状态：计划中
- 当前任务编号：TRADE-024
- 当前批次目标：为订单状态机补充拒绝非法回退保护，避免支付/回调乱序把订单主状态回写到更早阶段。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-023` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-024` 描述、DoD、验收与 `technical_reference`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务聚焦“非法回退保护”，不是新增新状态机分支。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 编码 -> 验证 -> 待审批”。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次，不跳步骤。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：沿用 P2 批次审计格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：首批标准链路仍按 SKU 独立快照与单向履约推进。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：订单状态编排位于 `platform-core/order` 与 `billing webhook` 联动边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：不新增公开接口，仅强化既有状态迁移行为。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务不新增 topic，复用既有审计痕迹。
  12. `docs/开发准备/统一错误码字典正式版.md`：拒绝非法回退仍复用既有冲突/忽略语义，不发明新错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：需要覆盖乱序回调 / 状态保护回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立 `TRADE-024` 专项 smoke，避免堆入旧测试。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：通过 Rust 应用层事务与 PostgreSQL 行锁完成状态保护。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体内聚，不跨模块引入无关重构。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L1445`：订单生命周期是单向总表，不应回退到更早阶段。
  2. `docs/data_trading_blockchain_system_design_split/06-Phase 1：最小可信交易闭环系统设计.md:L65`：迁移必须具备明确触发源、互斥、幂等性，重复或乱序事件不能造成矛盾状态。
  3. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L229`：首批场景按 SKU 独立履约，回调不能覆盖已进入交付/验收/结算阶段的订单事实。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/application/*`
  - `apps/platform-core/src/modules/order/domain/*`
  - `apps/platform-core/src/modules/order/tests/*`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade024_illegal_state_regression_db_smoke -- --nocapture`
  4. 启动服务：`KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `curl POST /api/v1/payments/webhooks/mock_payment` 发送乱序失败回调，验证订单状态不回退并回查审计。

### BATCH-133（待审批）
- 状态：待审批
- 当前任务编号：TRADE-024
- 当前批次目标：为订单状态机补充拒绝非法回退保护，避免支付/回调乱序把订单主状态回写到更早阶段。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-023` 已审批通过。
- 已实现功能：
  1. 将支付结果驱动订单状态的适用范围收敛到 `created / quoted / approval_pending / contract_pending / contract_effective` 五类预锁定状态。
  2. 支付结果应用改为事务内 `SELECT ... FOR UPDATE` + `UPDATE ... WHERE status = current_status`，补齐 compare-and-swap 保护。
  3. 对已进入 `buyer_locked` 及其后续履约状态、以及 SKU 特有履约状态，晚到支付回调统一记为 `order.payment.result.ignored`，不再回写订单主状态。
  4. 新增 `TRADE-024` 专项 DB smoke，经真实 webhook 路由验证“支付意图可更新，但订单不回退”。
  5. 补充 `payment_state` 单元测试，覆盖 `buyer_locked` 和 `api_bound` 等晚到回调忽略逻辑。
- 涉及文件：
  - `apps/platform-core/src/modules/order/domain/payment_state.rs`
  - `apps/platform-core/src/modules/order/domain/mod.rs`
  - `apps/platform-core/src/modules/order/application/mod.rs`
  - `apps/platform-core/src/modules/billing/handlers.rs`
  - `apps/platform-core/src/modules/order/tests/trade007_state_machine_fields_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade024_illegal_state_regression_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade024_illegal_state_regression_db_smoke -- --nocapture`
  4. 启动最新服务：`APP_PORT=8082 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 插入临时 `seller_delivering / paid` 订单与 `processing` 支付意图。
  6. `curl POST http://127.0.0.1:8082/api/v1/payments/webhooks/mock_payment` 发送乱序 `payment.failed` 回调。
  7. `psql` 回查 `trade.order_main`、`payment.payment_intent`、`audit.audit_event`。
  8. 清理临时业务数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`146 passed, 0 failed, 1 ignored`）。
  - `trade024_illegal_state_regression_db_smoke`：通过。
  - 真实 API 联调：`POST /api/v1/payments/webhooks/mock_payment` 返回 `HTTP 200`，`processed_status=processed`，`applied_payment_status=failed`。
  - DB 回查：`payment.payment_intent.status=failed`，但 `trade.order_main` 仍保持 `seller_delivering / paid / trade024_api_seed_seller_delivering`，未发生状态倒退。
  - 审计回查：命中 `order.payment.result.ignored / ignored`。
  - 清理结果：临时业务测试数据已清理；审计记录按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `领域模型` 7.2（订单生命周期单向推进）
  - `Phase 1：最小可信交易闭环系统设计` 6.5（迁移触发源明确、互斥、幂等）
  - `全集成基线-V1` 5.3.2A（首批场景按 SKU 独立履约事实，不得被乱序回调覆盖）
- 覆盖的任务清单条目：`TRADE-024`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-136（待审批）
- 状态：待审批
- 当前任务编号：TRADE-027
- 当前批次目标：为主交易链路补齐集成测试，覆盖下单、合同确认、锁资前校验、非法状态跳转、自动断权。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-026` 已审批通过。
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认下一任务为 `TRADE-027`，目标是主交易链路集成测试。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务关注主链路关键节点联动，不是新增业务能力。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 编码 -> 验证 -> 待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：单任务批次，不跳步骤。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：沿用 P2 审计格式。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯约束不变。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：主交易链路需覆盖下单、合同、支付前门禁、授权、断权。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：交易主链路位于 `platform-core/order`，联动 `billing/contract/authorization`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：不新增公开接口，复用既有交易接口完成联动测试。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本任务不新增 topic，以审计和状态落库作为验证依据。
  12. `docs/开发准备/统一错误码字典正式版.md`：锁资前校验和非法跳转继续复用既有冲突错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：需补齐主链路集成场景覆盖。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：新增独立 `TRADE-027` 测试文件，避免堆积进既有 smoke。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调数据库使用 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用现有 `local/mock` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：通过 Rust 集成测试 + 本地 curl/psql 联调完成验证。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，不做行为扩展。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：订单聚合需串联合同、支付、授权等关系事实。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：首批标准链路必须覆盖下单、履约、授权和异常阻断。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方从下单到锁资、授权、断权的顺序必须可验证。
- 已实现功能：
  1. 新增 `TRADE-027` 独立 DB smoke：真实串联 `POST /api/v1/orders`、`POST /contract-confirm`、`POST /file-std/transition`、`POST /authorization/transition`、`POST /share-ro/transition`。
  2. 覆盖主链路关键断言：下单成功、合同确认成功、商品审核不通过时锁资前校验阻断、审核恢复后锁资成功、非法状态跳转被拒绝、`SHARE_RO` 订单自动断权后授权状态变为 `expired`。
  3. 覆盖审计断言：`trade.order.create`、`trade.contract.confirm`、`trade.order.file_std.transition`、`trade.authorization.auto_cutoff.expired`。
  4. 修正 smoke 清理顺序，先删 `trade.order_main` 再删 `contract.digital_contract`，避免 `order_main_contract_id_fkey` 留存测试业务数据。
- 涉及文件：
  - `apps/platform-core/src/modules/order/tests/trade027_main_trade_flow_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade027_main_trade_flow_db_smoke -- --nocapture`
  4. 启动服务：`APP_PORT=8085 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 写入临时 `FILE_STD` / `SHARE_RO` 业务数据。
  6. `curl` 依次验证下单、合同确认、锁资前阻断/成功、非法状态跳转、授权发放、自动断权。
  7. `psql` 回查 `trade.order_main`、`trade.authorization_grant`、`audit.audit_event`。
  8. 清理临时业务测试数据（审计 append-only 保留）。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`149 passed, 0 failed, 1 ignored`）。
  - `trade027_main_trade_flow_db_smoke`：通过。
  - 真实 API 联调：
    - `POST /api/v1/orders` 返回 `HTTP 200`。
    - `POST /api/v1/orders/{id}/contract-confirm` 返回 `HTTP 200`，`signature_provider_mode=mock`。
    - 商品 `review_status=rejected` 时，`POST /file-std/transition` 返回 `409 ORDER_PRE_LOCK_CHECK_FAILED: product review status is not approved`。
    - 商品恢复 `approved` 后，`POST /file-std/transition` 返回 `HTTP 200`，`current_state=buyer_locked`。
    - 非法 `close_completed` 跳转返回 `409 FILE_STD_TRANSITION_FORBIDDEN`。
    - `POST /authorization/transition` 返回 `HTTP 200`；`POST /share-ro/transition action=expire_share` 返回 `HTTP 200`。
  - DB 回查：`trade.order_main` 文件订单状态为 `buyer_locked / paid`；`trade.authorization_grant` 最新状态为 `expired`；审计命中 `trade.order.create=1`、`trade.contract.confirm=1`、`trade.order.file_std.transition=1`、`trade.authorization.auto_cutoff.expired=1`。
  - 清理结果：临时业务测试数据已清理；验证回查结果为 `order_main=0 | digital_contract=0 | organization=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（订单聚合关系与主链路事实）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-027`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-137（计划中）
- 状态：计划中
- 当前任务编号：TRADE-028
- 当前批次目标：生成 `docs/02-openapi/trade.yaml` 第一版并与当前 Trade 实现完成一致性校验。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-027` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-028` 交付物、DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务解释，与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 冻结边界，不扩展功能。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对主交易链路与订单聚合基线。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade/order/contract/authorization 边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 V1 交易接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认本任务不新增 topic，以审计验证为准。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持现有错误码命名，不在归档文档中发明新口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐文档与接口联调验证证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按既有 `docs/02-openapi/` 结构落盘归档。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调基于本地 core 栈与 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用现有 `local/mock` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补文档归档与校验。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：OpenAPI 需覆盖订单聚合、合同、授权等主对象与状态命名。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：OpenAPI 需体现主交易闭环关键接口，术语与流程节点一致。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：接口归档需覆盖下单、锁资前校验、授权与断权相关主链路节点。
- 预计涉及文件：
  - `docs/02-openapi/trade.yaml`
  - `docs/02-openapi/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cmp -s docs/02-openapi/trade.yaml packages/openapi/trade.yaml`
  2. Trade OpenAPI vs `apps/platform-core/src/modules/order/api/mod.rs` 路径/方法一致性校验。
  3. `cargo fmt --all`
  4. `cargo test -p platform-core`
  5. 启动服务：`APP_PORT=8086 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `curl GET /api/v1/orders/standard-templates` + `psql` 审计计数核验。

### BATCH-137（待审批）
- 状态：待审批
- 当前任务编号：TRADE-028
- 当前批次目标：生成 `docs/02-openapi/trade.yaml` 第一版并与当前 Trade 实现完成一致性校验。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-027` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-028` 交付物、DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务解释，与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 冻结边界，不扩展功能。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对主交易链路与订单聚合基线。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade/order/contract/authorization 边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 V1 交易接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认本任务不新增 topic，以审计验证为准。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持现有错误码命名，不在归档文档中发明新口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：补齐文档与接口联调验证证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按既有 `docs/02-openapi/` 结构落盘归档。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调基于本地 core 栈与 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用现有 `local/mock` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补文档归档与校验。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：OpenAPI 覆盖订单聚合、合同、授权等主对象与状态命名。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：OpenAPI 体现主交易闭环关键接口，术语与流程节点一致。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：接口归档覆盖下单、锁资前校验、授权与断权相关主链路节点。
- 已实现功能：
  1. 新增交付文件 `docs/02-openapi/trade.yaml`（由 `packages/openapi/trade.yaml` 同步生成第一版）。
  2. 更新 `docs/02-openapi/README.md`，增加交易 OpenAPI 归档引用，满足 README/索引引用要求。
  3. 完成 Trade OpenAPI 与实现路由一致性校验：`docs/02-openapi/trade.yaml` 对比 `apps/platform-core/src/modules/order/api/mod.rs`，路径/方法无漂移。
  4. 完成手工 API 联调：`GET /api/v1/orders/standard-templates` 返回 5 条标准模板并验证审计落库。
- 涉及文件：
  - `docs/02-openapi/trade.yaml`
  - `docs/02-openapi/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cmp -s docs/02-openapi/trade.yaml packages/openapi/trade.yaml`
  2. Trade OpenAPI vs `apps/platform-core/src/modules/order/api/mod.rs` 路径/方法一致性校验。
  3. `cargo fmt --all`
  4. `cargo test -p platform-core`
  5. 启动服务：`APP_PORT=8086 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `curl GET /api/v1/orders/standard-templates` + `psql` 审计计数核验。
- 验证结果：
  - `trade_openapi_synced=yes`（`docs/02-openapi/trade.yaml` 与 `packages/openapi/trade.yaml` 一致）。
  - 路径/方法一致性校验结果：`missing_paths=[] extra_paths=[] method_mismatch=[]`。
  - `cargo test -p platform-core`：通过（`149 passed, 0 failed, 1 ignored`）。
  - `curl` 联调：`HTTP 200`，`scenario_count=5`，`scenario_codes=S1,S2,S3,S4,S5`。
  - 审计回查：`audit_count=1`，`action_name=trade.order.templates.read`（request_id=`req-trade028-openapi-1776602625`）。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（交易与订单聚合）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-028`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入；本批仅执行只读接口联调，无业务测试数据需要清理。

### BATCH-138（计划中）
- 状态：计划中
- 当前任务编号：TRADE-029
- 当前批次目标：生成 `docs/05-test-cases/order-state-machine.md`，按 8 个标准 SKU 编写状态转换测试矩阵。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-028` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-029` 交付物、DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务解释，与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 冻结边界，不扩展功能。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对主交易链路、标准交易链路与状态机基线。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade/order/contract/authorization 边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 8 个标准 SKU 对应接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认本任务不新增 topic，以审计验证为准。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持既有状态冲突与非法跳转命名。
  13. `docs/开发准备/测试用例矩阵正式版.md`：当前任务属于测试用例/回归基线归档。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按既有 `docs/05-test-cases/` 结构落盘归档。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调基于本地 core 栈与 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用现有 `local/mock` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补矩阵文档与校验。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：测试矩阵必须围绕订单聚合主状态、支付状态和交付/验收/结算子状态命名。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：矩阵需覆盖首批标准交易链路的 8 个标准 SKU 与关键流程节点。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：矩阵需覆盖下单、锁资、交付/执行、验收、结算/断权等主流程节点。
- 预计涉及文件：
  - `docs/05-test-cases/order-state-machine.md`
  - `docs/05-test-cases/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 8 个标准 SKU 状态机 smoke：`trade008~trade015`
  4. 启动服务：`APP_PORT=8087 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 写入临时 `SHARE_RO` 测试数据并 `curl POST /api/v1/orders/{id}/share-ro/transition` 验证状态迁移与审计落库。

### BATCH-138（待审批）
- 状态：待审批
- 当前任务编号：TRADE-029
- 当前批次目标：生成 `docs/05-test-cases/order-state-machine.md`，按 8 个标准 SKU 编写状态转换测试矩阵。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；`TRADE-028` 已审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：定位 `TRADE-029` 交付物、DoD、acceptance、technical_reference。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版任务解释，与 CSV 一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持 V1 冻结边界，不扩展功能。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记计划中，再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：同步批次记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对主交易链路、标准交易链路与状态机基线。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 trade/order/contract/authorization 边界。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认 8 个标准 SKU 对应接口冻结口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：确认本任务不新增 topic，以审计验证为准。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持既有状态冲突与非法跳转命名。
  13. `docs/开发准备/测试用例矩阵正式版.md`：当前任务属于测试用例/回归基线归档。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：按既有 `docs/05-test-cases/` 结构落盘归档。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：联调基于本地 core 栈与 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：复用现有 `local/mock` 配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补矩阵文档与校验。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：测试矩阵围绕订单聚合主状态、支付状态和交付/验收/结算子状态命名。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：矩阵覆盖首批标准交易链路的 8 个标准 SKU 与关键流程节点。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：矩阵覆盖下单、锁资、交付/执行、验收、结算/断权等主流程节点。
- 已实现功能：
  1. 新增 `docs/05-test-cases/order-state-machine.md`，按 `FILE_STD / FILE_SUB / API_SUB / API_PPU / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 8 个标准 SKU 落盘状态转换测试矩阵。
  2. 文档中同步冻结通用规则：主状态唯一性、通用子状态映射、锁资前门禁、晚到支付回退保护、非法跳转统一冲突语义。
  3. 为每个 SKU 写明：正向链路、分支动作、禁止样例、自动化证据文件与关联交叉验证批次。
  4. 更新 `docs/05-test-cases/README.md`，将 `order-state-machine.md` 纳入测试用例索引。
- 涉及文件：
  - `docs/05-test-cases/order-state-machine.md`
  - `docs/05-test-cases/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 8 个标准 SKU 状态机 smoke：`trade008_file_std_state_machine_db_smoke`、`trade009_file_sub_state_machine_db_smoke`、`trade010_api_sub_state_machine_db_smoke`、`trade011_api_ppu_state_machine_db_smoke`、`trade012_share_ro_state_machine_db_smoke`、`trade013_qry_lite_state_machine_db_smoke`、`trade014_sbx_std_state_machine_db_smoke`、`trade015_rpt_std_state_machine_db_smoke`
  4. 启动服务：`APP_PORT=8087 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  5. `psql` 写入临时 `SHARE_RO` 测试数据并 `curl POST /api/v1/orders/{id}/share-ro/transition` 执行 `enable_share`。
  6. `psql` 回查 `trade.order_main` 与 `audit.audit_event`，再清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`149 passed, 0 failed, 1 ignored`）。
  - 8 个标准 SKU 状态机 smoke：全部通过。
  - 真实 API 联调：`POST /api/v1/orders/{id}/share-ro/transition` 返回 `HTTP 200`，`action=enable_share`，`current_state=share_enabled`，`payment_status=unpaid`。
  - DB 回查：`trade.order_main` 为 `share_enabled / unpaid / in_progress / not_started / not_started / none`。
  - 审计回查：`audit_count=1`，`action_name=trade.order.share_ro.transition`。
  - 清理结果：临时业务测试数据已清理；回查结果 `order_main=0 | organization=0 | product=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（交易与订单聚合）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-029`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-139（计划中）
- 当前任务编号：TRADE-030
- 当前批次目标：实现支付结果到订单推进编排器，收紧可推进状态范围，覆盖支付成功/失败/超时三类结果，并保证状态不可倒退。
- 前置依赖核对结果：`BIL-005`、`TRADE-007`、`CORE-014` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-030` 为当前单任务批次，DoD 要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，与 CSV 描述一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 验证 -> TODO -> 待审批”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：当前只补 `TRADE-030` 范围，不跨任务扩展。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批先登记计划中，完成后再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后同步批次追溯。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对“付款与锁定”“支付成功前不得进入交付”“支付成功/失败/超时”的主链路约束。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认支付回调处理归 `billing`，订单推进编排归 `order`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增接口，维持既有 webhook/订单接口口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付成功回调事件载荷与“支付成功前不得放行交付”约束。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持现有状态冲突和 webhook 处理错误码语义。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批补支付结果编排专项回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：保持现有 `order/application` 与 `order/tests` 结构，不做无关重构。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证继续使用 core 栈数据库 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用现有 `mock_payment` 与 `KAFKA_*` 本地配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补支付结果编排缺口与测试。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：`current_state` 是唯一主状态，支付结果若触发推进必须同步更新主状态和子状态，且不可倒退。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：支付成功进入锁定/待交付链路，支付失败/超时不能错误放行交付。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：支付结果属于下单主流程关键节点，需保留审计证据并支撑后续待交付流程。
- 预计涉及文件：
  - `apps/platform-core/src/modules/order/application/mod.rs`
  - `apps/platform-core/src/modules/order/domain/payment_state.rs`
  - `apps/platform-core/src/modules/order/tests/trade007_state_machine_fields_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade030_payment_result_orchestrator_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade007_state_machine_fields_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade030_payment_result_orchestrator_db_smoke -- --nocapture`
  5. 启动服务：`APP_PORT=8088 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `psql` 写入多组临时订单/支付意图，`curl POST /api/v1/payments/webhooks/mock_payment` 验证 success / failed / timeout / early-state-ignore 四条链路，并回查 `trade.order_main`、`payment.payment_intent`、`audit.audit_event`。

### BATCH-139（待审批）
- 状态：待审批
- 当前任务编号：TRADE-030
- 当前批次目标：实现支付结果到订单推进编排器，收紧可推进状态范围，覆盖支付成功/失败/超时三类结果，并保证状态不可倒退。
- 前置依赖核对结果：`BIL-005`、`TRADE-007`、`CORE-014` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-030` 为当前单任务批次，DoD 要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，与 CSV 描述一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：按固定流程先写计划中，再实现与验证，最后补待审批。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：当前只修 `TRADE-030` 范围，不跨任务扩展。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：已写入本批计划中与待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：已追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对“付款与锁定”“支付成功前不得进入交付”“支付成功/失败/超时”的主链路约束。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认支付回调处理归 `billing`，订单推进编排归 `order`。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增接口，维持既有 webhook/订单接口口径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付成功回调事件载荷与“支付成功前不得放行交付”约束。
  12. `docs/开发准备/统一错误码字典正式版.md`：维持现有状态冲突和 webhook 处理错误码语义。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批补支付结果编排专项回归。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：保持现有 `order/application` 与 `order/tests` 结构，不做无关重构。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：验证使用 core 栈数据库 `datab-postgres:5432`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用现有 `mock_payment` 与 `KAFKA_*` 本地配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：沿用 Rust + Axum + PostgreSQL + Kafka 现状。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，仅补支付结果编排缺口与测试。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：支付结果推进必须同步更新 `current_state` 与子状态，且不可倒退。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：支付成功进入锁定/待交付链路，支付失败/超时不能错误放行交付。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：支付结果属于下单主流程关键节点，需保留审计证据并支撑后续待交付流程。
- 已实现功能：
  1. 收紧支付结果编排适用主状态，仅允许 `created / contract_effective` 接收支付成功/失败/超时推进，`approval_pending / contract_pending` 等前序状态统一忽略。
  2. 支付成功推进到 `buyer_locked` 时补写 `buyer_locked_at`，确保“已锁资”状态有落库时间证据。
  3. 保留并复用 `order.payment.result.applied / ignored` 审计；真实 webhook 链路下 success / failed / timeout / ignored 四条路径均已验证。
  4. 新增 `trade030_payment_result_orchestrator_db_smoke`，覆盖 success / failed / timeout / early-state-ignore 四条链路。
  5. 补强 `trade007_state_machine_fields_db_smoke`，成功推进后校验 `buyer_locked_at IS NOT NULL`。
  6. 更新 `packages/openapi/trade.yaml` 说明，明确 `TRADE-030` 仅对 `created / contract_effective` 两类可支付主状态生效。
- 涉及文件：
  - `apps/platform-core/src/modules/order/application/mod.rs`
  - `apps/platform-core/src/modules/order/domain/payment_state.rs`
  - `apps/platform-core/src/modules/order/tests/trade007_state_machine_fields_db.rs`
  - `apps/platform-core/src/modules/order/tests/trade030_payment_result_orchestrator_db.rs`
  - `apps/platform-core/src/modules/order/tests/mod.rs`
  - `packages/openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade007_state_machine_fields_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core trade030_payment_result_orchestrator_db_smoke -- --nocapture`
  5. 启动服务：`APP_PORT=8088 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  6. `psql` 写入四组临时订单/支付意图，`curl POST /api/v1/payments/webhooks/mock_payment` 依次验证 success / failed / timeout / early-state-ignore。
  7. `psql` 回查 `trade.order_main`、`payment.payment_intent`、`audit.audit_event`，再清理临时业务数据；审计记录按 append-only 保留。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`153 passed, 0 failed, 1 ignored`）。
  - `trade007_state_machine_fields_db_smoke`：通过，成功推进后 `buyer_locked_at IS NOT NULL`。
  - `trade030_payment_result_orchestrator_db_smoke`：通过，覆盖 success / failed / timeout / early-state-ignore 四条链路。
  - 真实 API 联调：4 次 `POST /api/v1/payments/webhooks/mock_payment` 均返回 `HTTP 200`；返回 `applied_payment_status` 分别为 `succeeded / failed / expired / failed`。
  - DB 回查：
    - success 订单：`buyer_locked / paid / pending_delivery / pending_settlement / payment_succeeded_to_buyer_locked / buyer_locked_at=true`
    - failed 订单：`payment_failed_pending_resolution / failed / pending_delivery / not_started`
    - timeout 订单：`payment_timeout_pending_compensation_cancel / expired / pending_delivery / not_started`
    - ignored 订单：保持 `contract_pending / unpaid / not_started / not_started` 不变
  - 审计回查：
    - `trade030-api-success-* -> order.payment.result.applied / success / created -> buyer_locked`
    - `trade030-api-failed-* -> order.payment.result.applied / success / contract_effective -> payment_failed_pending_resolution`
    - `trade030-api-timeout-* -> order.payment.result.applied / success / contract_effective -> payment_timeout_pending_compensation_cancel`
    - `trade030-api-ignored-* -> order.payment.result.ignored / ignored / contract_pending -> null`
  - 清理结果：临时业务数据已清理；回查结果 `order_main=0 | payment_intent=0 | organization=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（交易与订单聚合）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-030`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-142（计划中）
- 状态：计划中
- 当前任务编号：TRADE-033
- 当前批次目标：输出 `docs/01-architecture/order-orchestration.md`，冻结订单主状态、支付/交付/验收/结算/争议子状态、推进规则、互斥关系与回调乱序保护，并补索引引用。
- 前置依赖核对结果：`TRADE-007`、`TRADE-024`、`BIL-022`、`DLV-029` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-033` 是当前单任务批次，交付物为 `docs/01-architecture/order-orchestration.md`，DoD 要求文档落盘、结构完整、术语一致并被 README/索引引用。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，与 CSV 对 `TRADE-033` 的目标、依赖与验证要求一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 验证 -> TODO -> 待审批 -> 本地提交”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：当前只实现 `TRADE-033` 范围，不跨任务扩展为新的运行时能力。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批先登记计划中，完成后再补待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后同步追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对交易主链路、支付推进、待交付门禁与完整闭环术语。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 `billing` 负责支付结果处理，`order` 负责订单推进与履约门禁。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增 API，只引用既有订单/支付接口作为编排证据。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付 webhook / outbox 审计在编排链路中的事件边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` 等既有错误口径，不在文档中发明新错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：确认本批需至少一条集成测试或手工 API 验证，并保留审计/日志证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：文档落在 `docs/01-architecture/`，并通过 README 索引暴露。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：API 验证继续使用 core 栈数据库 `datab-postgres:5432` 与本地 Kafka。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用 `KAFKA_BROKERS`、`KAFKA_BOOTSTRAP_SERVERS`、`DATABASE_URL` 现有配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：维持 Rust + Axum + PostgreSQL + Kafka 的实现上下文，文档只做冻结说明。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，强调 `order/billing/delivery` 的编排职责分工。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：`trade.order_main.status` 是唯一主状态，支付/交付/验收/结算/争议子状态是镜像状态，文档需画清其同步关系和互斥边界。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：交易闭环必须覆盖下单、支付、交付、验收、结算全链路，且支付成功前不得放行交付。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：买方搜索、选购与下单流程中的支付、锁资、交付、验收节点必须在编排文档中有明确推进顺序。
- 预计涉及文件：
  - `docs/01-architecture/order-orchestration.md`
  - `docs/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`APP_PORT=8093 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  4. `psql` 写入乱序支付回调临时数据，`curl POST /api/v1/payments/webhooks/mock_payment` 验证 `out_of_order_ignored` 与 `order.payment.result.ignored` 审计痕迹。
  5. `psql` 回查 `payment.payment_intent`、`trade.order_main`、`audit.audit_event`，再清理临时业务数据；审计记录按 append-only 保留。

### BATCH-142（待审批）
- 状态：待审批
- 当前任务编号：TRADE-033
- 当前批次目标：输出 `docs/01-architecture/order-orchestration.md`，冻结订单主状态、支付/交付/验收/结算/争议子状态、推进规则、互斥关系与回调乱序保护，并补索引引用。
- 前置依赖核对结果：`TRADE-007`、`TRADE-024`、`BIL-022`、`DLV-029` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `TRADE-033` 是当前单任务批次，交付物为 `docs/01-architecture/order-orchestration.md`，DoD 要求文档落盘、结构完整、术语一致并被 README/索引引用。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版解释，与 CSV 对 `TRADE-033` 的目标、依赖与验证要求一致。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 验证 -> TODO -> 待审批 -> 本地提交”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：当前只实现 `TRADE-033` 范围，不跨任务扩展为新的运行时能力。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批已写入计划中与待审批。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：已同步追加本批追溯记录。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读确认，按约定不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对交易主链路、支付推进、待交付门禁与完整闭环术语。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认 `billing` 负责支付结果处理，`order` 负责订单推进与履约门禁。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：当前任务不新增 API，只引用既有订单/支付接口作为编排证据。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：核对支付 webhook / outbox 审计在编排链路中的事件边界。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用 `TRD_STATE_CONFLICT` 等既有错误口径，不在文档中发明新错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：确认本批需至少一条集成测试或手工 API 验证，并保留审计/日志证据。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：文档落在 `docs/01-architecture/`，并通过 README 索引暴露。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：API 验证使用 core 栈数据库 `datab-postgres:5432` 与本地 Kafka。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用 `KAFKA_BROKERS`、`KAFKA_BOOTSTRAP_SERVERS`、`DATABASE_URL` 现有配置，不新增配置项。
  17. `docs/开发准备/技术选型正式版.md`：维持 Rust + Axum + PostgreSQL + Kafka 的实现上下文，文档只做冻结说明。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，强调 `order/billing/delivery` 的编排职责分工。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L620`：`trade.order_main.status` 是唯一主状态，支付/交付/验收/结算/争议子状态是镜像状态，文档已画清其同步关系和互斥边界。
  2. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L1723`：文档已覆盖下单、支付、交付、验收、结算全链路，并强调支付成功前不得放行交付。
  3. `docs/业务流程/业务流程图-V1-完整版.md:L204`：文档已按买方搜索、选购与下单主流程梳理支付、锁资、交付、验收节点的推进顺序。
- 已实现功能：
  1. 新增 `docs/01-architecture/order-orchestration.md`，冻结 `trade.order_main.status` 主状态唯一性、五类子状态职责、主链路推进关系、SKU 首个交付/开通动作、互斥规则与维护约束。
  2. 文档中明确当前 V1 运行时职责边界：`billing` 处理支付 webhook 与乱序保护，`order` 负责主状态推进、交付门禁与最小 `delivery_record(prepared)` 落库，`delivery` 模块目录仍是占位。
  3. 文档中落地两层回调乱序保护：`payment.payment_intent` 时间戳/rank 防回退，以及 `trade.order_main` 的可变状态白名单 + `SELECT ... FOR UPDATE` + compare-and-swap。
  4. 更新 `docs/README.md`，将 `docs/01-architecture/order-orchestration.md` 纳入工程文档索引，满足 README/索引引用要求。
- 涉及文件：
  - `docs/01-architecture/order-orchestration.md`
  - `docs/README.md`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo test -p platform-core`
  3. 启动服务：`APP_PORT=8093 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  4. `psql` 写入两组临时数据：一组用于验证 webhook 层 `out_of_order_ignored`，一组用于验证订单层 `order.payment.result.ignored`。
  5. `curl POST /api/v1/payments/webhooks/mock_payment` 两次，分别发送 `payment.failed` 晚到回调和 `payment.succeeded` 的前序状态回调。
  6. `psql` 回查 `payment.payment_intent`、`trade.order_main`、`payment.payment_webhook_event`、`audit.audit_event`，再清理临时业务数据；审计记录按 append-only 保留。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p platform-core`：通过（`162 passed, 0 failed, 1 ignored`）。
  - 真实 API 联调通过：
    - `POST /api/v1/payments/webhooks/mock_payment`（`trade033-api-ooo`）返回 `HTTP 200`，`processed_status=out_of_order_ignored`，`out_of_order_ignored=true`。
    - `POST /api/v1/payments/webhooks/mock_payment`（`trade033-api-order-ignore`）返回 `HTTP 200`，`processed_status=processed`，`applied_payment_status=succeeded`。
  - DB 回查：
    - `intent_ooo` 保持 `succeeded`，`order_ooo` 保持 `seller_delivering / paid`
    - `intent_ignore` 更新为 `succeeded`，`order_ignore` 保持 `contract_pending / unpaid`
    - `payment.payment_webhook_event`：`evt-trade033-ooo -> out_of_order_ignored`，`evt-trade033-order-ignore -> processed`
    - 审计命中：`payment.webhook.out_of_order_ignored=1`、`payment.webhook.processed=1`、`order.payment.result.ignored=1`
  - 清理结果：临时业务数据已清理；回查结果 `order_main=0 | payment_intent=0 | organization=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.4（交易与订单聚合）
  - `全集成基线-V1` 15（核心交易链路设计）
  - `业务流程图-V1` 4.3（买方搜索、选购与下单流程）
- 覆盖的任务清单条目：`TRADE-033`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。


### BATCH-143（计划中）
- 状态：计划中
- 当前任务编号：DLV-001
- 当前批次目标：实现 `storage-gateway` 领域模型，冻结对象定位、bucket/key、hash、watermark 策略、下载限制、访问审计，并接入现有订单只读聚合供后续文件交付链路复用。
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-001` 为当前起始任务，交付要求落在 `modules/delivery/**`、`modules/storage/**`、`packages/openapi/delivery.yaml`，DoD 要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：核对阅读版条目与 CSV 一致，当前任务只冻结 `storage-gateway` 领域模型，不提前宣称 `deliver/download-ticket` 路由完成。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”执行，但本阶段按你的新规则连续推进下一个 DLV task。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单 task 实施，不合并多 task；代码组织优先模块化，不把交付逻辑塞回 `order` 大文件。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：本批先登记计划中，完成后补待审批记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001` 审计说明。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：按当前约定只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `storage-gateway` 是 V1 核心服务；文件类交付主流程要求对象上传、key_envelope、delivery_ticket、delivery_commit_hash、下载验真与审计联动。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：确认交付/对象存储边界在 `delivery/storage-gateway`，订单服务只保留主状态与聚合编排职责。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结的交付接口从 `POST /api/v1/orders/{id}/deliver`、`GET /api/v1/orders/{id}/download-ticket` 起步；本任务先补领域模型和 OpenAPI 基线文件，不发明额外路径。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：交付域后续事件仍须走 `outbox -> Kafka`，不能把 Kafka 当业务真值；本任务需为后续交付事件留稳定聚合结构。
  12. `docs/开发准备/统一错误码字典正式版.md`：继续沿用现有 `TRD_STATE_CONFLICT` / 统一错误响应结构，不额外发明偏离字典的新格式。
  13. `docs/开发准备/测试用例矩阵正式版.md`：当前任务至少需要一条集成测试或手工 API 验证，并保留审计/日志证据；本阶段额外执行 curl、联调和 DB 回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：交付能力独立组织在 `modules/delivery` 与 `modules/storage`，避免继续堆积到 `order` 模块。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：DLV 阶段联调使用 `datab-postgres`、`datab-minio`、`datab-redis`、`datab-kafka`；文件/对象类交付必须实际接入 MinIO。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用 `DATABASE_URL`、`MINIO_ENDPOINT`、bucket 环境变量、Kafka topic 配置和 Redis 本地口径，不另起一套配置命名。
  17. `docs/开发准备/技术选型正式版.md`：维持 Rust + Axum + PostgreSQL + MinIO/S3-compatible + Redis + Kafka/outbox 技术基线。
  18. `docs/开发准备/平台总体架构设计草案.md`：保持模块化单体边界，交付模型通过独立模块对订单读聚合提供能力，不反向侵入订单主状态机职责。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L709`：`Delivery`、`StorageObject`、`DeliveryTicket`、`KeyEnvelope` 是交付聚合的核心对象，订单对交付是 `1 -> N` 关系，需在领域模型中稳定表达对象、票据和访问痕迹。
  2. `docs/业务流程/业务流程图-V1-完整版.md:L270`：文件类交付流程要求对象上传、key_envelope、delivery_ticket、download_limit/expire_at、delivery_commit_hash、下载验真与回执链路完整衔接。
  3. `docs/页面说明书/页面说明书-V1-完整版.md:L590`：文件交付页核心模块是对象上传区、密钥封装状态、下载令牌状态、交付回执列表和 Hash 校验提示；本任务输出的领域模型需覆盖这些页面对象。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/**`
  - `apps/platform-core/src/modules/storage/**`
  - `apps/platform-core/src/modules/order/dto/order_relations.rs`
  - `apps/platform-core/src/modules/order/dto/order_lifecycle_snapshot.rs`
  - `apps/platform-core/src/modules/order/repo/order_relation_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_lifecycle_snapshot_repository.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv001_storage_gateway_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `mc`/MinIO 写入临时对象、`psql` 写入交付记录与票据、`curl GET /api/v1/orders/{id}` / `curl GET /api/v1/orders/{id}/lifecycle-snapshots` 联调，再回查 `audit.audit_event` 与清理临时业务数据。

### BATCH-143（待审批）
- 状态：待审批
- 当前任务编号：DLV-001
- 当前批次目标：实现 `storage-gateway` 领域模型，冻结对象定位、bucket/key、hash、watermark 策略、下载限制、访问审计，并接入现有订单只读聚合供后续文件交付链路复用。
- 已实现功能：
  1. 新增 `modules/storage`，落地 `object_uri -> bucket/key` 解析，支持 `s3://bucket/key`、HTTP bucket 前缀与相对路径回退。
  2. 新增 `modules/delivery` 领域快照 `StorageGatewaySnapshot`，覆盖对象定位、完整性、watermark 策略、下载限制、访问审计。
  3. 新增 `load_storage_gateway_snapshots(order_id)`，从 `delivery.delivery_record / storage_object / key_envelope / delivery_ticket / delivery_receipt / catalog.storage_namespace` 聚合交付存储视图。
  4. 将 `storage_gateway` 接入 `GET /api/v1/orders/{id}` 与 `GET /api/v1/orders/{id}/lifecycle-snapshots` 的交付聚合输出。
  5. 新增 `delivery.storage_gateway.read` 审计写入，修正 append-only 审计表不能 `UPDATE` 的口径，改为直接 `INSERT`。
  6. 新增 `packages/openapi/delivery.yaml` 基线，并更新 `packages/openapi/trade.yaml`、`docs/02-openapi/trade.yaml` 的订单交付聚合 schema。
  7. 新增 `DLV-001` 专项 DB smoke，覆盖订单详情、生命周期快照、审计回查。
- 涉及文件：
  - `apps/platform-core/src/modules/storage/**`
  - `apps/platform-core/src/modules/delivery/**`
  - `apps/platform-core/src/modules/order/api/handlers.rs`
  - `apps/platform-core/src/modules/order/dto/order_relations.rs`
  - `apps/platform-core/src/modules/order/dto/order_lifecycle_snapshot.rs`
  - `apps/platform-core/src/modules/order/repo/order_relation_repository.rs`
  - `apps/platform-core/src/modules/order/repo/order_lifecycle_snapshot_repository.rs`
  - `apps/platform-core/src/modules/order/repo/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `packages/openapi/trade.yaml`
  - `docs/02-openapi/trade.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv001_storage_gateway_db_smoke -- --nocapture`
  7. `./infra/minio/init-minio.sh`
  8. 启动服务：`APP_PORT=8094 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  9. 使用真实 MinIO 对象 + `psql` 临时业务数据，`curl GET /api/v1/orders/{id}`、`curl GET /api/v1/orders/{id}/lifecycle-snapshots` 联调，回查 `audit.audit_event` 并清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`166 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；根目录 `.sqlx` 已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv001_storage_gateway_db_smoke`：通过。
  - 真实 API 联调通过：
    - `GET /api/v1/orders/{id}`：`HTTP 200`，返回 `storage_gateway.object_locator.bucket_name=delivery-objects`、`object_key=orders/{suffix}/payload.enc`、`remaining_downloads=3`、`access_count=2`。
    - `GET /api/v1/orders/{id}/lifecycle-snapshots`：`HTTP 200`，返回相同 `storage_gateway.object_locator.bucket_name=delivery-objects`。
  - MinIO 实体联动通过：真实对象已上传并 `mc stat` 成功，DB 中 `delivery.storage_object.object_uri` 与对象路径一致。
  - 审计回查：`delivery.storage_gateway.read` 命中 `2` 条（订单详情 + 生命周期快照）。
  - 清理结果：临时业务数据与 MinIO 测试对象已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.5（交付与执行聚合）
  - `业务流程图-V1` 4.4.1（文件类交付）
  - `页面说明书-V1` 7.1（文件交付页）
- 覆盖的任务清单条目：`DLV-001`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-144（计划中）
- 状态：计划中
- 当前任务编号：DLV-002
- 当前批次目标：实现文件交付接口 `POST /api/v1/orders/{id}/deliver` 的文件分支，打通 `prepared -> committed`、对象关联、密钥封装、下载票据摘要、回执摘要、订单主状态推进与审计。
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过；`DLV-001` 已完成并本地提交。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-002` 仅覆盖文件交付接口，不提前实现下载票据接口或订阅接口。
  2. `docs/开发任务/v1-core-开发任务清单.md`：完成定义要求接口、DTO、权限、审计、错误码和最小测试齐备，并与 OpenAPI 不漂移。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”，且按新规则直接推进下一个 task。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单 task 实施、模块化拆分，避免把交付 API 继续塞进 `order/api.rs` 这类大文件。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记本批计划中，完成后补待审批记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：文件交付中心需覆盖对象关联、下载令牌、限次下载、水印、Hash 校验与回执摘要；文件交付动作权限对应 `delivery.file.commit`。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：交付 API 归属 `delivery` 模块，订单只保留主状态与聚合编排。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口路径为 `POST /api/v1/orders/{id}/deliver`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：当前先保持领域事件边界，后续 outbox/Kafka 在 DLV-020/030 再补标准化桥接，不提前发散。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用统一错误响应结构与现有错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批仍执行编译/单测/DB smoke/真实 API 联调/DB 回查/业务数据清理。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：交付接口、DTO、repo、tests 保持在 `modules/delivery/**` 下独立组织。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批继续使用 `datab-postgres`、`datab-minio`、`datab-kafka` 联调。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用 `DATABASE_URL`、`MINIO_*`、bucket 口径与 Kafka 本地配置。
  17. `docs/开发准备/技术选型正式版.md`：维持 Rust + Axum + PostgreSQL + MinIO/S3-compatible 技术基线。
  18. `docs/开发准备/平台总体架构设计草案.md`：交付接口通过独立 delivery 模块挂载，不反向污染订单模块。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L709`：`Delivery / StorageObject / DeliveryTicket / KeyEnvelope / DeliveryReceipt` 是文件交付主对象，`Order 1 -> N Delivery`。
  2. `docs/业务流程/业务流程图-V1-完整版.md:L270`：文件类交付需按顺序完成对象上传、`key_envelope`、`delivery_ticket`、`delivery_commit_hash`，并推动订单进入 `delivered`。
  3. `docs/页面说明书/页面说明书-V1-完整版.md:L590`：文件交付页核心模块包含对象上传区、密钥封装状态、下载令牌状态、交付回执列表、Hash 校验提示。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv002_file_delivery_commit_db_smoke -- --nocapture`
  7. 启动服务并使用真实 MinIO 对象 + `curl POST /api/v1/orders/{id}/deliver` 联调，再回查 `delivery_record / delivery_ticket / key_envelope / audit.audit_event / trade.order_main`。

### BATCH-144（待审批）
- 状态：待审批
- 当前任务编号：DLV-002
- 当前批次目标：实现文件交付接口 `POST /api/v1/orders/{id}/deliver` 的文件分支，打通 `prepared -> committed`、对象关联、密钥封装、下载票据摘要、回执摘要、订单主状态推进与审计。
- 已实现功能：
  1. 在 `modules/delivery` 新增文件交付 API handler、DTO、repo，独立承接 `POST /api/v1/orders/{id}/deliver`，未把交付逻辑回塞到订单 API。
  2. 落地 `commit_file_delivery(...)`：校验角色与卖方租户边界、限定 `branch=file` 与 `FILE_STD`、解析 `s3://` 对象定位、按卖方/桶解析有效 `storage_namespace`。
  3. 交付提交事务内完成：
     - 新建 `delivery.storage_object`
     - 新建 `delivery.key_envelope`
     - 关闭旧 `active` ticket 并签发新 `delivery.delivery_ticket`
     - 将 `delivery.delivery_record` 从 `prepared` 推进到 `committed`
     - 将 `trade.order_main` 推进到 `delivered`
     - 写入 `delivery.file.commit` 审计
  4. 已提交交付的订单再次调用时走幂等返回，避免重复创建对象/票据。
  5. 订单详情聚合已能读到本次提交后的 `storage_gateway.object_locator.bucket_name/object_key`。
  6. `packages/openapi/delivery.yaml` 已同步新增交付提交路径与请求/响应 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/file_delivery_commit.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/events/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/file_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv002_file_delivery_commit_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv002_file_delivery_commit_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8095 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用真实 MinIO 对象 + `psql` 临时业务数据 + `curl POST /api/v1/orders/{id}/deliver` / `curl GET /api/v1/orders/{id}` 联调，再回查数据库并清理业务数据与 MinIO 测试对象。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`167 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv002_file_delivery_commit_db_smoke`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/deliver`：`HTTP 200`，返回 `current_state=delivered`、`bucket_name=delivery-objects`、`object_key=orders/{suffix}/payload.enc`、`ticket_id`、`delivery_id`、`download_limit=5`。
    - `GET /api/v1/orders/{id}`：`HTTP 200`，返回 `relations.deliveries[0].storage_gateway.object_locator.bucket_name=delivery-objects`。
  - DB 回查通过：
    - `trade.order_main`：`delivered / paid / delivered / pending_acceptance / pending_settlement`
    - `delivery.delivery_record`：`committed`，并已写入 `object_id / envelope_id / delivery_commit_hash / receipt_hash`
    - `delivery.delivery_ticket`：`download_limit=5 / download_count=0 / status=active`
    - `audit.audit_event`：`delivery.file.commit` 命中 `1` 条
  - MinIO 实体联动通过：真实对象上传到 `delivery-objects/orders/{suffix}/payload.enc`，接口与 DB 记录的对象路径一致。
  - 清理结果：临时业务数据与 MinIO 测试对象已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.5（交付与执行聚合）
  - `业务流程图-V1` 4.4.1（文件类交付）
  - `页面说明书-V1` 7.1（文件交付页）
- 覆盖的任务清单条目：`DLV-002`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-145（计划中）
- 状态：计划中
- 当前任务编号：DLV-003
- 当前批次目标：实现下载票据接口 `GET /api/v1/orders/{id}/download-ticket`，对已提交文件交付签发短时下载令牌，落 Redis 缓存，校验买方身份、剩余次数与过期时间，并记录 `delivery.file.download` 审计。
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过；`DLV-001`、`DLV-002` 已完成并本地提交。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-003` 只覆盖下载票据签发接口，不提前实现下载中间件校验。
  2. `docs/开发任务/v1-core-开发任务清单.md`：完成定义要求接口、DTO、权限、审计、错误码和最小测试齐备，并与 OpenAPI 不漂移。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”，按新流程在本地提交后直接推进下一任务。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：继续保持 `modules/delivery/**` 独立组织，不把交付接口塞回订单模块。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记本批计划中，完成后补待审批记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认文件副本产品允许“一次性或限次下载令牌”、下载令牌短时有效、获取下载令牌需 `delivery.file.download` 并满足买方身份/次数/时效/审计约束。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：票据签发与交付控制归属 `delivery` 模块，不把 Redis/下载令牌状态塞进订单领域真值。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结接口路径为 `GET /api/v1/orders/{id}/download-ticket`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批不提前发 Kafka 业务事件；Redis 只用于短时票据缓存，数据库仍是主真值。
  12. `docs/开发准备/统一错误码字典正式版.md`：沿用统一错误响应结构与现有错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批仍执行编译/单测/DB+Redis smoke/真实 API 联调/DB 与 Redis 回查/业务数据清理。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：接口、DTO、repo、tests 继续放在 `modules/delivery/**` 下。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批要真实联动 `datab-postgres`、`datab-redis`、`datab-kafka`、`datab-minio`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：采用 `REDIS_PORT`、`REDIS_PASSWORD`、`REDIS_NAMESPACE` 与本地 Redis DB 3 作为下载票据缓存。
  17. `docs/开发准备/技术选型正式版.md`：继续遵循 Rust + Axum + PostgreSQL + Redis + MinIO 技术基线。
  18. `docs/开发准备/平台总体架构设计草案.md`：短时票据缓存归入基础设施接缝，避免把下载令牌当业务真值。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L709`：`DeliveryTicket` 是下载或访问凭证，归属于交付与执行聚合。
  2. `docs/业务流程/业务流程图-V1-完整版.md:L270`：文件类交付链路要求 `delivery_ticket` 带 `expire_at / download_limit`，并在买方获取下载令牌后由下载网关执行次数与时效校验。
  3. `docs/页面说明书/页面说明书-V1-完整版.md:L590`：文件交付页核心模块包含“下载令牌状态”，说明接口需返回可展示的票据状态字段。
- 补充约束文档：
  1. `docs/04-runbooks/redis-keys.md`：下载票据缓存 key 固定为 `{ns}:download-ticket:{ticket_id}`，本地建议落 DB 3，TTL 5 分钟。
  2. `docs/权限设计/接口权限校验清单.md`：`GET /api/v1/orders/{id}/download-ticket` 需要 `delivery.file.download`，额外校验买方身份、次数、到期、审计。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `apps/platform-core/Cargo.toml`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv003_download_ticket_db_smoke -- --nocapture`
  7. 启动服务并使用真实 Redis + `curl GET /api/v1/orders/{id}/download-ticket` 联调，再回查 `delivery_ticket / audit.audit_event / redis download-ticket key`。

### BATCH-145（待审批）
- 状态：待审批
- 当前任务编号：DLV-003
- 当前批次目标：实现下载票据接口 `GET /api/v1/orders/{id}/download-ticket`，对已提交文件交付签发短时下载令牌，落 Redis 缓存，校验买方身份、剩余次数与过期时间，并记录 `delivery.file.download` 审计。
- 已实现功能：
  1. 在 `modules/delivery` 新增下载票据 DTO、仓储与 API handler，独立承接 `GET /api/v1/orders/{id}/download-ticket`，未把票据逻辑塞回订单模块。
  2. 落地 `issue_download_ticket(...)`：校验平台角色或买方租户边界、限制 `FILE_STD` + `delivered/accepted/settled/closed` 订单、要求存在 `committed` 交付记录和 `active` 票据。
  3. 对过期票据和次数耗尽票据会先回写 `delivery.delivery_ticket.status=expired/exhausted`，再返回冲突错误，避免 Redis 缓存与数据库状态漂移。
  4. 票据签发会基于 committed `storage_object.bucket_name/object_key` 生成短时下载 token，计算 `token_hash`，并写入 Redis DB 3 键 `datab:v1:download-ticket:{ticket_id}`，TTL 取 5 分钟与票据剩余有效期的较小值。
  5. Redis 缓存失败时会回滚 `token_hash` 和 Redis 残留键，数据库仍保持主真值；成功后写入 `delivery.file.download` 审计。
  6. `packages/openapi/delivery.yaml` 已同步新增下载票据路径与响应 schema。
- 涉及文件：
  - `apps/platform-core/Cargo.toml`
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/download_ticket.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/download_ticket_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv003_download_ticket_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv003_download_ticket_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8096 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用真实 MinIO 对象 + `psql` 临时业务数据 + `curl GET /api/v1/orders/{id}/download-ticket` 联调，再回查数据库、Redis 并清理业务数据、Redis 测试键和 MinIO 测试对象。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`168 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv003_download_ticket_db_smoke`：通过。
  - 真实 API 联调通过：
    - `GET /api/v1/orders/{id}/download-ticket`：`HTTP 200`，返回 `ticket_id`、`bucket_name=delivery-objects`、`object_key=orders/{suffix}/payload.enc`、`download_count=2`、`remaining_downloads=3`、短时 `token`。
  - DB 回查通过：
    - `delivery.delivery_ticket`：`download_limit=5 / download_count=2 / status=active / token_hash=<md5>`
    - `audit.audit_event`：`delivery.file.download` 命中 `1` 条
  - Redis 联动通过：`datab:v1:download-ticket:{ticket_id}` 已写入 DB 3，缓存 payload 与响应中的票据、对象、次数信息一致。
  - MinIO 实体联动通过：缓存中的 `bucket_name/object_key` 与 committed 交付对象一致。
  - 清理结果：临时业务数据、Redis 测试 key 和 MinIO 测试对象已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.5（交付与执行聚合）
  - `业务流程图-V1` 4.4.1（文件类交付）
  - `页面说明书-V1` 7.1（文件交付页）
  - `docs/04-runbooks/redis-keys.md`（下载票据 Redis key/TTL/DB 约束）
- 覆盖的任务清单条目：`DLV-003`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-146（计划中）
- 状态：计划中
- 当前任务编号：DLV-004
- 当前批次目标：实现下载票据验证中间件与文件下载入口，确保文件下载请求必须携带有效 ticket，经 Redis + 数据库双重校验后写入 `delivery.delivery_receipt` 下载日志，并返回真实 MinIO 对象内容。
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过；`DLV-001`、`DLV-002`、`DLV-003` 已完成并本地提交。
- 已阅读证据（文件+要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-004` 要求实现“下载票据验证中间件”，重点是下载请求校验与下载日志，不是重复实现票据签发。
  2. `docs/开发任务/v1-core-开发任务清单.md`：明确 `DLV-004` 与 `DLV-003` 分层，前者负责下载网关校验与日志落库。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续执行“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”，提交后直接推进下一任务。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：继续保持 `modules/delivery/**` 和 `modules/storage/**` 按功能拆分，不把下载网关逻辑塞回订单模块。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：先登记本批计划中，完成后补待审批记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认文件副本交付链路是“下载令牌签发 -> 下载网关校验 -> 返回密文对象 + key_envelope -> 写 delivery_receipt / download_log / audit_event”，且文件下载属于受控入口。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：下载网关与对象读取仍归属 `delivery/storage-gateway`，Redis 只做短期票据缓存，PostgreSQL 保持主真值。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已存在 `GET /api/v1/orders/{id}/download-ticket`，本批需要补齐与之配套的实际受控下载入口说明与 schema。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批不提前发 Kafka 业务事件；下载日志与回执先落数据库/审计。
  12. `docs/开发准备/统一错误码字典正式版.md`：继续沿用统一错误响应结构与现有错误码口径。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批仍执行编译/单测/DB+Redis smoke/真实 API 联调/DB+Redis+MinIO 回查/业务数据清理。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：下载中间件、下载 handler、下载仓储、对象存储读取能力继续拆分到 `modules/delivery/**`、`modules/storage/**`。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批要真实联动 `datab-postgres`、`datab-redis`、`datab-kafka`、`datab-minio`。
  16. `docs/开发准备/配置项与密钥管理清单.md`：本批继续使用 `REDIS_PORT`、`REDIS_PASSWORD`、`REDIS_NAMESPACE`，并新增读取 `MINIO_ENDPOINT / MINIO_ROOT_USER / MINIO_ROOT_PASSWORD / BUCKET_DELIVERY_OBJECTS`。
  17. `docs/开发准备/技术选型正式版.md`：继续遵循 Rust + Axum + PostgreSQL + Redis + MinIO 技术基线；下载网关需要实质性 S3-compatible 对象读取能力。
  18. `docs/开发准备/平台总体架构设计草案.md`：下载网关属于受控基础设施接缝，必须把校验、回执、审计和对象读取串联起来。
- technical_reference 约束映射：
  1. `docs/领域模型/全量领域模型与对象关系说明.md:L709`：`DeliveryTicket` 表示下载或访问凭证，`DeliveryReceipt` 表示下载回执或访问回执，本批必须把两者串起来。
  2. `docs/业务流程/业务流程图-V1-完整版.md:L270`：文件类交付链路要求“买方获取下载令牌 -> 下载网关校验 token、buyer_did、次数、时效 -> 返回密文对象 + key_envelope -> 写 delivery_receipt / download_log”。
  3. `docs/页面说明书/页面说明书-V1-完整版.md:L590`：文件交付页核心模块包含“下载令牌状态 / 交付回执列表 / Hash 校验提示”，说明下载接口要把票据校验结果、回执和对象内容打通。
- 补充约束文档：
  1. `docs/04-runbooks/redis-keys.md`：下载票据缓存 key 固定为 `{ns}:download-ticket:{ticket_id}`，DB 3，TTL 5 分钟，文档建议一次性使用，本批至少要保证每次下载会回写次数与失效状态。
  2. `docs/权限设计/接口权限校验清单.md`：`delivery.file.download` 需要 tenant + order 作用域，且附加校验“买方身份、次数、到期、审计”。
  3. `docs/数据库设计/数据库表字典正式版.md`：`delivery.delivery_receipt` 字段固定为 `receipt_id / delivery_id / order_id / receipt_hash / client_fingerprint / source_ip / downloaded_at`，不得自由扩表。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `apps/platform-core/src/modules/storage/**`
  - `apps/platform-core/Cargo.toml`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv004_download_validation_db_smoke -- --nocapture`
  7. 启动服务并使用真实 MinIO + Redis + `curl GET /api/v1/orders/{id}/download-ticket` / `curl GET /api/v1/orders/{id}/download?...` 联调，再回查 `delivery_ticket / delivery_receipt / audit.audit_event / redis download-ticket key`。

### BATCH-146（待审批）
- 状态：待审批
- 当前任务编号：DLV-004
- 当前批次目标：实现下载票据验证中间件与文件下载入口，确保文件下载请求必须携带有效 ticket，经 Redis + 数据库双重校验后写入 `delivery.delivery_receipt` 下载日志，并返回真实 MinIO 对象内容。
- 已实现功能：
  1. 在 `modules/delivery/api` 新增下载票据验证中间件 `validate_download_ticket_middleware`，对 `GET /api/v1/orders/{id}/download` 强制校验 `delivery.file.download` 权限、ticket 传递、Redis 缓存存在、买方租户边界、订单路径与 token 绑定关系。
  2. 新增 `download_file_api` 与 `consume_download_ticket(...)`：下载时以 PostgreSQL 为主真值再次校验 `FILE_STD`、订单状态、ticket 状态、次数与 `token_hash`，然后递增 `download_count`、写入 `delivery.delivery_receipt`、写入 `delivery.file.download` 审计。
  3. 下载成功后会同步回写 Redis DB 3 中的下载票据缓存次数；若达到上限则删除缓存，避免已耗尽 ticket 被重复命中。
  4. 在 `modules/storage/application` 新增 S3-compatible 对象读取能力，使用 MinIO endpoint + access key/secret 真实拉取对象，未用脚本伪造下载结果。
  5. 下载接口返回 `delivery_receipt` 元数据、`key_envelope` 与 `object_base64`，把“下载网关校验 -> 返回密文对象 + key_envelope -> 写回执”的冻结链路串通。
  6. `packages/openapi/delivery.yaml` 已同步新增受控下载路径与 schema。
- 涉及文件：
  - `Cargo.lock`
  - `apps/platform-core/Cargo.toml`
  - `apps/platform-core/src/modules/delivery/api/download_middleware.rs`
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/download_file.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/download_file_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/download_ticket_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv004_download_validation_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/storage/application/mod.rs`
  - `apps/platform-core/src/modules/storage/application/object_store.rs`
  - `apps/platform-core/src/modules/storage/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv004_download_validation_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8097 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab MINIO_ENDPOINT=http://127.0.0.1:9000 MINIO_ROOT_USER=datab MINIO_ROOT_PASSWORD=datab_local_pass cargo run -p platform-core`
  8. 使用真实 MinIO 对象 + `psql` 临时业务数据 + `curl GET /api/v1/orders/{id}/download-ticket` / `curl GET /api/v1/orders/{id}/download?ticket=...` 联调，再回查数据库、Redis 并清理业务数据与 MinIO 测试对象。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv004_download_validation_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`169 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - `GET /api/v1/orders/{id}/download-ticket`：`HTTP 200`，`remaining_downloads=2`
    - `GET /api/v1/orders/{id}/download?ticket=...`：`HTTP 200`，返回 `download_count=2`、`remaining_downloads=1`、`ticket_status=active`、`key_cipher=cipher-{suffix}`、真实 MinIO 对象内容 `encrypted-manual-{suffix}`
  - DB 回查通过：
    - `delivery.delivery_ticket`：`download_count=2 / status=active`
    - `delivery.delivery_receipt`：写入 `receipt_hash / client_fingerprint / source_ip`
    - `audit.audit_event`：`delivery.file.download` 命中 `1` 条 `ref_type=delivery_receipt / result_code=downloaded`
  - Redis 联动通过：`datab:v1:download-ticket:{ticket_id}` 在下载后 `remaining_downloads=1`，与 DB 状态一致。
  - MinIO 实体联动通过：接口返回的 `object_base64` 解码后与真实上传对象内容一致。
  - 清理结果：临时业务数据、Redis 测试 key 和 MinIO 测试对象已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `领域模型` 4.5（`DeliveryTicket / DeliveryReceipt`）
  - `业务流程图-V1` 4.4.1（下载网关校验 token、buyer_did、次数、时效）
  - `页面说明书-V1` 7.1（下载令牌状态、交付回执列表、Hash 校验提示）
  - `docs/04-runbooks/redis-keys.md`（下载票据 Redis key/TTL/DB 约束）
  - `docs/数据库设计/数据库表字典正式版.md`（`delivery.delivery_receipt` 字段约束）
- 覆盖的任务清单条目：`DLV-004`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-147（计划中）
- 状态：计划中
- 当前任务编号：DLV-005
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-005` 目标为 `POST/GET /api/v1/orders/{id}/subscriptions`，Definition of Done 要求接口、DTO、权限校验、审计、错误码、最小测试与 OpenAPI 一致。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-005` 为 `FILE_SUB` 周期交付入口，不能只做状态机占位。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：本批继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守单任务顺序、冻结文档先行、禁止跳步简化。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：延续 DLV 阶段批次日志，承接 `BATCH-146`。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯不丢失。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：按约定由人工维护，本批不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `FILE_SUB` 为版本订阅；接口权限为 `delivery.subscription.manage/read`；流程要求记录 `start_version_no / cadence / delivery_channel / last_delivered_version_no / next_delivery_at / subscription_status`。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：订阅属于 Delivery 领域，需与 Trade 订单边界联动。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：核对 DLV 交付接口须保持路径与契约稳定。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先落订单内订阅真值与审计；若需异步版本推送，后续任务再补 topic/outbox。
  12. `docs/开发准备/统一错误码字典正式版.md`：冲突/权限/内部错误继续复用统一错误码。
  13. `docs/开发准备/测试用例矩阵正式版.md`：需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：继续按 `delivery/api|dto|repo|tests` 组织，避免回到巨型文件。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：订阅 API 作为交付域受控入口，订单仍为主真值来源。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/业务流程/业务流程图-V1-完整版.md:L290`：订单生效时创建 `revision_subscription`，记录 `start_version_no / cadence / delivery_channel`，后续版本发布更新 `last_delivered_version_no`。
  - `docs/原始PRD/数据对象产品族与交付模式增强设计.md:L292`：`FILE_SUB` 为“版本订阅 / 周期更新”，V1 必须支持按版本顺序交付与到期断权。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L590`：文件交付页展示交付对象/令牌/回执，本批需补订阅信息查询能力，供文件订阅页承载。
- 当前批次目标：实现 `POST /api/v1/orders/{id}/subscriptions` 与 `GET /api/v1/orders/{id}/subscriptions`，支持 `FILE_SUB` 订单建立/续订 `delivery.revision_subscription`，补齐权限、审计、OpenAPI、测试与真实 API 联调。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv005_revision_subscription_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST/GET /api/v1/orders/{id}/subscriptions` 联调，回查 `delivery.revision_subscription / trade.order_main / audit.audit_event` 后清理业务数据。

### BATCH-147（待审批）
- 状态：待审批
- 当前任务编号：DLV-005
- 当前批次目标：实现 `POST /api/v1/orders/{id}/subscriptions` 与 `GET /api/v1/orders/{id}/subscriptions`，支持 `FILE_SUB` 周期交付的订阅创建、查询、暂停后状态同步与续订恢复。
- 已实现功能：
  1. 在 `modules/delivery` 新增 `revision_subscription` DTO、仓储与 API 处理器，落地 `POST /api/v1/orders/{id}/subscriptions`、`GET /api/v1/orders/{id}/subscriptions`。
  2. `POST` 已实现 `delivery.subscription.manage` 权限、卖方租户作用域校验、主体状态校验、`FILE_SUB` SKU 校验、产品审核/资产版本状态校验、周期与版本范围校验，并在 `delivery.revision_subscription` 中写入 `cadence / delivery_channel / start_version_no / last_delivered_version_no / next_delivery_at / subscription_status / metadata`。
  3. 当订单处于 `paused / expired` 时，续订会在同一事务内把 `trade.order_main` 恢复到 `buyer_locked / paid`，并补写 `trade.order.file_sub.transition` 审计，避免出现“订阅记录恢复但订单主状态未恢复”的不一致。
  4. `GET` 已实现 `delivery.subscription.read` 权限与买/卖双方最小作用域读取；若订单已被 `pause/expire/close`，读取时会把 `delivery.revision_subscription.subscription_status` 与订单主状态同步，避免展示陈旧 `active` 状态。
  5. `packages/openapi/delivery.yaml` 已同步新增订阅 manage/read 路径和 schema，Delivery OpenAPI 与实现路由保持一致。
  6. 已兼容当前运行库口径：`catalog.product.subscription_cadence` 实列当前未落地时，回退读取 `trade.order_main.price_snapshot_json.subscription_cadence` 与 `catalog.product.metadata.subscription_cadence`，确保真实数据库联调可通过。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/revision_subscription.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/revision_subscription_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv005_revision_subscription_db.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv005_revision_subscription_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8098 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 临时写入 `FILE_SUB` 订单，执行 `curl POST/GET /api/v1/orders/{id}/subscriptions`、`curl POST /api/v1/orders/{id}/file-sub/transition`（`pause_subscription`），再回查 `delivery.revision_subscription / trade.order_main / audit.audit_event`，最后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv005_revision_subscription_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`170 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/subscriptions`：`HTTP 200`，首次返回 `operation=created / subscription_status=active`
    - `GET /api/v1/orders/{id}/subscriptions`：`HTTP 200`，返回 `subscription_status=active`
    - `POST /api/v1/orders/{id}/file-sub/transition`（`pause_subscription`）：`HTTP 200`
    - 暂停后再次 `GET /api/v1/orders/{id}/subscriptions`：`HTTP 200`，返回 `subscription_status=paused`
    - 再次 `POST /api/v1/orders/{id}/subscriptions`：`HTTP 200`，返回 `operation=renewed / subscription_status=active / current_state=buyer_locked`
  - DB 回查通过：
    - `delivery.revision_subscription`：`quarterly | file_ticket | active | metadata.renewal=Q2`
    - `trade.order_main`：`buyer_locked | paid | pending_delivery`
    - 审计：`delivery.subscription.manage/read` 共命中 `4` 条，其中 `read=2`；`pause` 过程另有 `trade.order.file_sub.transition` 审计。
  - 清理结果：临时业务数据已删除，`order_main` 回查为 `0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.1A（订阅创建、记录 cadence/channel、停订关闭）
  - `原始PRD/数据对象产品族与交付模式增强设计` 4.2 / 4. 七类标准交易方式（`FILE_SUB` 周期更新与到期断权）
  - `全集成基线-V1` 3.5 / 10.2（`delivery.subscription.manage/read` 权限、订单作用域、周期与版本范围校验）
  - `数据库表字典正式版`（`delivery.revision_subscription` 字段）
- 覆盖的任务清单条目：`DLV-005`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；当前运行库 `catalog.product.subscription_cadence` 实列未落地，已按快照/metadata 回退兼容，不单独新增 TODO；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-148（计划中）
- 状态：计划中
- 当前任务编号：DLV-006
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-006` 目标为 `POST/GET /api/v1/orders/{id}/share-grants`，Definition of Done 要求接口、DTO、权限校验、审计、错误码、最小测试与 OpenAPI 一致。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-006` 属于 `SHARE_RO` 真实交付入口，不能只做状态机占位或只插入一条授权记录。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：本批继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行，并在提交后继续推进下一任务。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守单任务顺序、冻结文档先行、按模块拆分实现。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-147`，先登记本批计划中，完成后再补待审批记录。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `SHARE_RO` 的主交付对象是“共享开通结果 + recipient/subscriber 绑定 + revoke 生命周期”，不得按文件下载或 API 调用解释；共享接口权限为 `delivery.share.enable/read`，校验顺序为身份 -> 主体状态 -> 权限 -> 订单作用域 -> 共享对象归属 -> recipient 合法性 -> 协议/到期策略 -> 风控 -> 审计。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：共享开通属于 Delivery 领域，但订单与支付/合同真值仍留在 Trade。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已存在 `POST/GET /api/v1/orders/{id}/share-grants`，本批需补齐实现和 schema。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批不提前补 Kafka 业务桥接，先落数据库真值与审计，后续 `DLV-020/030` 再补 outbox 事件。
  12. `docs/开发准备/统一错误码字典正式版.md`：冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：共享授权继续落在 `delivery/api|dto|repo|tests`，不把实现塞回订单模块。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres`、`datab-kafka` 与本地 core 栈；共享授权本批不强依赖 Redis/MinIO 实体读写。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：共享开通是受控交付入口，需把交付结果、审计和后续首个只读访问触发点连通。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/业务流程/业务流程图-V1-完整版.md:L314`：共享开通需校验 `recipient / subscriber / share protocol`，绑定共享对象、范围和到期时间，生成 `data_share_grant`，执行 share grant，并在到期/撤权时写入状态回执与审计。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L625`：只读共享开通页必须呈现共享协议类型、recipient/subscriber 绑定、共享对象列表、授权范围与到期时间、grant 状态与撤权记录。
  - `docs/领域模型/全量领域模型与对象关系说明.md:L709`：`DataShareGrant` 负责记录 recipient/subscriber、共享协议、access locator 与 grant/revoke 生命周期，是 `Order` 到共享交付结果的正式关系对象。
- 补充约束文档：
  1. `docs/权限设计/接口权限校验清单.md`：`POST /share-grants` 需 `delivery.share.enable`，`GET /share-grants` 需 `delivery.share.read`，并额外校验订单已支付、共享对象存在、recipient 合法、最小披露和审计。
  2. `docs/权限设计/角色权限矩阵正式版.md`：卖方运营员/租户管理员可开通；买方运营员、卖方运营员、租户审计只读员可查看。
  3. `docs/权限设计/后端鉴权中间件规则说明.md`：不得仅凭订单状态放行，必须再次校验 `asset_object_binding.object_kind = share_object`。
  4. `docs/数据库设计/数据库表字典正式版.md`：`delivery.data_share_grant` 字段固定为 `asset_object_id / recipient_ref / share_protocol / access_locator / grant_status / read_only / receipt_hash / granted_at / revoked_at / expires_at / metadata`。
- 当前批次目标：实现 `POST /api/v1/orders/{id}/share-grants` 与 `GET /api/v1/orders/{id}/share-grants`，打通 `SHARE_RO` 订单共享开通、recipient/subscriber 绑定、`delivery.data_share_grant` 落库、订单状态推进、交付记录提交、审计与真实 API 联调。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv006_share_grant_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST/GET /api/v1/orders/{id}/share-grants` 联调，回查 `delivery.data_share_grant / delivery.delivery_record / trade.order_main / audit.audit_event` 后清理业务数据。

### BATCH-148（待审批）
- 状态：待审批
- 当前任务编号：DLV-006
- 当前批次目标：实现 `POST /api/v1/orders/{id}/share-grants` 与 `GET /api/v1/orders/{id}/share-grants`，打通 `SHARE_RO` 订单共享开通、recipient/subscriber 绑定、`delivery.data_share_grant` 落库、订单状态推进、交付记录提交、审计与真实 API 联调。
- 已实现功能：
  1. 在 `modules/delivery` 新增 `share_grant` DTO、仓储与 API 处理器，落地 `POST /api/v1/orders/{id}/share-grants`、`GET /api/v1/orders/{id}/share-grants`。
  2. `POST` 已实现 `delivery.share.enable` 权限、卖方租户作用域校验、主体状态校验、`SHARE_RO` SKU 校验、支付完成校验、产品审核/资产版本状态校验、风控阻断校验，并强制 `catalog.asset_object_binding.object_kind = share_object`。
  3. `grant` 操作会校验 `recipient_ref / share_protocol / expires_at / receipt_hash`，通过统一可交付门禁后写入或更新 `delivery.data_share_grant`，并把 `delivery.delivery_record` 从 `prepared` 推进到 `committed`，订单主状态推进到 `share_granted` 或保持 `shared_active`。
  4. `revoke` 操作会回收当前有效 grant，写入 `revoked_at / receipt_hash`，保留既有 `subscriber_ref / scope_json` 元数据，并同步关闭 `delivery.delivery_record` 与 `trade.order_main`，同时触发既有授权断权编排。
  5. `GET` 已实现 `delivery.share.read` 权限，支持买方/卖方最小作用域读取，返回共享协议、recipient/subscriber、access locator、scope、到期时间和 grant/revoke 历史。
  6. `packages/openapi/delivery.yaml` 已同步新增 share-grant manage/read 路径与 schema，Delivery OpenAPI 与实现路由保持一致。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/share_grant.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/share_grant_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv006_share_grant_db.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv006_share_grant_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8099 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 临时写入 `SHARE_RO` 订单，执行 `curl POST /api/v1/orders/{id}/share-grants`、`curl GET /api/v1/orders/{id}/share-grants`、`curl POST /api/v1/orders/{id}/share-grants`（`revoke`），再回查 `delivery.data_share_grant / delivery.delivery_record / trade.order_main / audit.audit_event`，最后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv006_share_grant_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`171 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/share-grants`：`HTTP 200`，返回 `granted | share_granted | active`
    - `GET /api/v1/orders/{id}/share-grants`：`HTTP 200`，返回 `share_granted | active | warehouse://buyer/...`
    - `POST /api/v1/orders/{id}/share-grants`（`revoke`）：`HTTP 200`，返回 `revoked | revoked | revoked`
  - DB 回查通过：
    - `delivery.data_share_grant`：`revoked | warehouse://buyer/... | share_grant | share://seller/.../dataset | subscriber_ref=sub-...`
    - `delivery.delivery_record`：`revoked | share_grant | share_grant | share-revoke-...`
    - `trade.order_main`：`revoked | paid | closed | closed | closed`
    - 审计：`delivery.share.enable=2`、`delivery.share.read=1`、`trade.order.share_ro.transition=2`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.1B（共享开通、recipient/subscriber 绑定、撤权/到期）
  - `页面说明书-V1` 7.3（共享协议、对象、范围、到期时间与撤权展示）
  - `领域模型` 4.5（`DataShareGrant` 作为 Order 到共享交付结果的正式关系对象）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 后端鉴权规则（`delivery.share.enable/read` 与 `share_object` 校验）
- 覆盖的任务清单条目：`DLV-006`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-149（计划中）
- 状态：计划中
- 当前任务编号：DLV-007
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-007` 目标为 `POST /api/v1/orders/{id}/deliver` 的 API 分支，生成应用绑定、访问凭证、调用配额、限流配置。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-007` 必须是“真实 API 开通链路”，不能只推进订单状态或只落一条占位记录。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务顺序、冻结文档先行、实现后必须做真实 API 联调与 DB 回查。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-148`，先登记本批计划中。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 API 订阅链路为“应用绑定 -> API 凭证签发 -> 健康检查 -> 首次成功调用”，交付完成条件是“应用绑定成功、API Key 生效、健康检查通过”。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：API 开通属于 Delivery 领域，但订单/支付/合同真值仍在 Trade。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已存在 `POST /api/v1/orders/{id}/deliver`，`delivery_mode=api_access`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先落数据库真值与审计，不提前补 Kafka 业务桥接。
  12. `docs/开发准备/统一错误码字典正式版.md`：冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：API 开通继续落在 `delivery/api|dto|repo|tests`，不把交付实现塞回订单模块。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：API 产品交付需要把凭证、应用绑定、配额与限流配置统一落入受控交付链路。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/业务流程/业务流程图-V1-完整版.md:L334`：API 类交付需生成 API Key / 应用绑定 / 调用配额，并由网关按调用次数、频率、IP、用途策略校验。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L611`：API 开通页核心模块必须包含“应用绑定、API Key 展示、调用配额、限流规则、调用日志摘要”。
  - `docs/原始PRD/数据对象产品族与交付模式增强设计.md:L292`：API / 服务是 V1 标准交付方式，交付对象应显式落到 `api_endpoint` 与相应订单交付对象。
- 补充约束文档：
  1. `docs/权限设计/接口权限校验清单.md`：`POST /api/v1/orders/{id}/deliver` 在 API 分支需满足 `delivery.api.enable`，并做 tenant + order + 交付对象匹配 + 审计。
  2. `docs/权限设计/角色权限矩阵正式版.md`、`docs/权限设计/菜单权限映射表.md`：API 开通页角色至少覆盖 `tenant_developer`、`tenant_admin`，并保留卖方交付角色辅助开通口径。
  3. `docs/权限设计/后端鉴权中间件规则说明.md`：鉴权必须先过身份/主体状态，再过权限、订单作用域、交付对象匹配、审计。
  4. `docs/数据库设计/数据库表字典正式版.md`、`docs/数据库设计/V1/upgrade/030_trade_delivery.sql`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`：本批核心表为 `core.application`、`delivery.api_credential`、`delivery.delivery_record`。
- 当前批次目标：实现 `/api/v1/orders/{id}/deliver` 的 API 分支，支持 `API_SUB/API_PPU` 订单开通应用绑定、签发 API 凭证、落配额与限流配置，并与交付记录、订单主状态、审计联动。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv007_api_delivery_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/orders/{id}/deliver`（`branch=api`）联调，回查 `core.application / delivery.api_credential / delivery.delivery_record / trade.order_main / audit.audit_event` 后清理业务数据。

### BATCH-149（待审批）
- 状态：待审批
- 当前任务编号：DLV-007
- 前置依赖核对：`TRADE-003`、`TRADE-007`、`CAT-010`、`DLV-001` 已完成并审批通过。
- 当前批次目标：实现 `POST /api/v1/orders/{id}/deliver` 的 API 分支，为 `API_SUB/API_PPU` 订单生成应用绑定、访问凭证、调用配额与限流配置，并联动订单交付状态。
- 实现摘要：
  1. `delivery/api/handlers.rs` 按 `payload.branch` 分流 `file/api` 两条交付路径，新增 API 分支鉴权入口。
  2. `delivery/api/support.rs` 新增 `DeliveryPermission::EnableApiDelivery`，角色口径覆盖 `tenant_developer/tenant_admin/seller_operator/platform_admin/platform_risk_settlement`。
  3. `delivery/dto/file_delivery_commit.rs` 扩展 API 开通请求/响应字段，保留文件交付分支兼容性。
  4. `delivery/repo/api_delivery_repository.rs` 新增 API 开通仓储：校验 `api_endpoint` 绑定、创建或复用买方 `core.application`、签发 `delivery.api_credential`、落配额与限流配置、提交 `delivery.delivery_record`、推进 `trade.order_main` 到 `api_key_issued/quota_ready`。
  5. `delivery/repo/file_delivery_repository.rs` 同步适配通用响应结构，不改变文件交付既有外部行为。
  6. 新增 `dlv007_api_delivery_db.rs`，覆盖 `API_SUB` 与 `API_PPU` 的 DB smoke；`packages/openapi/delivery.yaml` 已同步更新。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/file_delivery_commit.rs`
  - `apps/platform-core/src/modules/delivery/repo/file_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/api_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv007_api_delivery_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv007_api_delivery_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8100 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时 `API_SUB` 订单与 `api_endpoint` 对象，执行 `curl POST /api/v1/orders/{id}/deliver`（`branch=api`），再回查 `core.application / delivery.api_credential / delivery.delivery_record / trade.order_main / audit.audit_event`，最后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv007_api_delivery_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`172 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：`POST /api/v1/orders/{id}/deliver` 返回 `HTTP 200`，摘要为 `api | api_key_issued | platform_proxy | ****505796`。
  - DB 回查通过：
    - `core.application`：已创建/绑定买方应用，`metadata.rate_limit_profile={"burst":20,"concurrency":5,"requests_per_minute":80}`。
    - `delivery.api_credential`：`active | platform_proxy | subscription`。
    - `delivery.delivery_record`：`committed | api_access | platform_proxy | api-sub-receipt-*`。
    - `trade.order_main`：`api_key_issued | in_progress | not_started | pending_settlement`。
    - 审计：`delivery.api.enable=1`、`trade.order.api_sub.transition=1`。
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.1A（API 应用绑定、Key 签发、配额与限流）
  - `页面说明书-V1` 7.2（应用绑定、API Key、配额、限流与调用摘要）
  - `数据对象产品族与交付模式增强设计` 3.2（API / 服务类交付对象与 `api_endpoint`）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 后端鉴权规则（`delivery.api.enable`、tenant + order + 交付对象匹配）
- 覆盖的任务清单条目：`DLV-007`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-150（计划中）
- 状态：计划中
- 当前任务编号：DLV-008
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-008` 目标为 `GET /api/v1/orders/{id}/usage-log`，输出按最小披露裁剪后的 API 使用日志。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-008` 完成定义为“接口、DTO、权限校验、审计、错误码和最小测试齐备，且与 OpenAPI 不漂移”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务顺序、冻结文档先行、实现后必须做真实 API 联调与 DB 回查。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-149`，先登记本批计划中。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `GET /api/v1/orders/{id}/usage-log` 对应权限 `delivery.api.log.read`，额外校验为“应用归属、最小披露”。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：API 使用日志属于 Delivery 读取面，但订单/应用/交付真值分散在 Trade + IAM + Delivery，需要聚合读取。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已列出 `GET /api/v1/orders/{id}/usage-log`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批以数据库已落 usage log 为真值，不提前引入额外 Kafka 读取逻辑。
  12. `docs/开发准备/统一错误码字典正式版.md`：读取冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：读取接口继续落在 `delivery/api|dto|repo|tests`，不把 Delivery 读逻辑塞回 Order。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：Delivery 读接口需基于统一鉴权、主体状态和审计基线提供受控摘要，而不是泄露底层调用细节。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/业务流程/业务流程图-V1-完整版.md:L334`：API 类交付必须采集 `access_log / usage_event / response_hash`，Usage Log 接口至少要可回看调用摘要与验收前证据。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L611`：API 开通页核心模块包含“调用日志摘要”，因此接口不能只返回裸表字段，至少要提供摘要视图。
  - `docs/原始PRD/数据对象产品族与交付模式增强设计.md:L292`：API / 服务类是 V1 标准交付方式，调用日志属于正式交付运营视图，而非调试临时数据。
- 补充约束文档：
  1. `docs/权限设计/接口权限校验清单.md`：`GET /api/v1/orders/{id}/usage-log` 权限为 `delivery.api.log.read`，额外校验为“应用归属、最小披露”。
  2. `docs/权限设计/权限点清单.md`：确认存在 `delivery.api.log.read` 权限点。
  3. `docs/权限设计/后端鉴权中间件规则说明.md`：敏感读接口必须执行“对象归属 + 最小披露 + 审计留痕”。
  4. `docs/数据库设计/数据库表字典正式版.md`、`docs/数据库设计/V1/upgrade/030_trade_delivery.sql`、`docs/数据库设计/表关系总图-ER文本图.md`：本批核心表为 `delivery.api_usage_log`、`delivery.api_credential`、`core.application`、`trade.order_main`。
- 当前批次目标：实现 `GET /api/v1/orders/{id}/usage-log`，返回按最小披露裁剪后的 API 使用日志摘要与记录列表，并联动应用归属校验与 `delivery.api.log.read` 审计。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv008_api_usage_log_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl GET /api/v1/orders/{id}/usage-log` 联调，回查 `delivery.api_usage_log / audit.audit_event` 后清理业务数据。

### BATCH-150（待审批）
- 状态：待审批
- 当前任务编号：DLV-008
- 前置依赖核对：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成并审批通过；`DLV-007` 已在当前主线完成并本地提交。
- 当前批次目标：实现 `GET /api/v1/orders/{id}/usage-log`，返回按最小披露裁剪后的 API 使用日志摘要与记录列表。
- 实现摘要：
  1. `delivery/api/mod.rs` 新增 `GET /api/v1/orders/{id}/usage-log` 路由，`delivery/api/handlers.rs` 新增读取处理器。
  2. `delivery/api/support.rs` 新增 `DeliveryPermission::ReadApiUsageLog`，角色口径覆盖 `buyer_operator/procurement_manager/tenant_developer/tenant_audit_readonly/tenant_admin/platform_*`。
  3. `delivery/dto/api_usage_log.rs` 新增响应 DTO，输出 `app + summary + logs` 三段结构，避免暴露明文 request_id。
  4. `delivery/repo/api_usage_log_repository.rs` 新增读取仓储：校验买卖主体状态、SKU 类型、买方应用归属，聚合 `delivery.api_usage_log + delivery.api_credential + core.application + trade.order_main`，并将 `request_id` 裁剪为 `request_ref`。
  5. `delivery/tests/dlv008_api_usage_log_db.rs` 新增 DB smoke，覆盖“买方成功读取 + 卖方越权失败 + 审计落库”。
  6. `packages/openapi/delivery.yaml` 已同步新增 `usage-log` 路径和 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/api_usage_log.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/api_usage_log_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv008_api_usage_log_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv008_api_usage_log_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8101 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时 `API_SUB` 订单、先执行 `curl POST /api/v1/orders/{id}/deliver` 生成应用绑定与 API 凭证，再向 `delivery.api_usage_log` 插入 3 条 usage 记录，执行 `curl GET /api/v1/orders/{id}/usage-log`（买方）与 `curl GET /api/v1/orders/{id}/usage-log`（卖方越权），最后回查 `delivery.api_usage_log / audit.audit_event` 并清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv008_api_usage_log_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`173 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - 买方 `GET /api/v1/orders/{id}/usage-log`：`HTTP 200`
    - 卖方同路径越权读取：`HTTP 403`
    - 返回摘要：`API_SUB | total=3 | success=1 | failed=2 | usage=4.75000000`
    - 首条日志：`5xx | ***ijkl | 500`
  - DB 回查通过：
    - `delivery.api_usage_log`：`3` 条记录，`usage_units` 聚合为 `4.75000000`
    - 审计：`delivery.api.log.read=1`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.2（`access_log / usage_event / response_hash` 采集）
  - `页面说明书-V1` 7.2（API 开通页的“调用日志摘要”核心模块）
  - `数据对象产品族与交付模式增强设计` 4（API / 服务类为 V1 标准交付方式）
  - `权限设计` 接口权限校验清单 / 权限点清单 / 后端鉴权规则（`delivery.api.log.read`、应用归属、最小披露、审计）
- 覆盖的任务清单条目：`DLV-008`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-151（计划中）
- 状态：计划中
- 当前任务编号：DLV-009
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-009` 目标为 `POST /api/v1/products/{id}/query-surfaces`，保存可查询区域、执行环境、输出限制。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-009` 是 Delivery 阶段的查询面配置起点，完成定义仍是“接口、DTO、权限、审计、错误码与最小测试齐备”。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务顺序、冻结文档先行、实现后必须做真实 API 联调与 DB 回查。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-150`，先登记本批计划中。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `POST /api/v1/products/{id}/query-surfaces` 的权限为 `delivery.query_surface.manage`，额外校验为“资产版本存在、读取区域合法、环境合法、审计”。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：查询面配置属于 Delivery，但商品、资产版本、执行环境真值分布在 Catalog + IAM + Delivery。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已列出 `POST /api/v1/products/{id}/query-surfaces`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先落数据库真值与审计，不提前引入额外异步桥接。
  12. `docs/开发准备/统一错误码字典正式版.md`：配置冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：查询面接口继续落在 `delivery/api|dto|repo|tests`，不混入 Catalog 已有读接口实现。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：查询面属于正式受控执行入口，必须绑定执行环境、读取范围、输出边界与配额策略。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/原始PRD/数据商品查询与执行面设计.md:L35`：`QuerySurface` 必须表达“面向哪个资产版本、运行在哪个执行环境、属于哪种查询面类型、可读数据范围、输出边界、频率/配额约束、surface_status”。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L668`：查询面与模板配置页核心模块包含“查询面类型选择、执行环境绑定、输出边界与导出策略、配额与计费单位、模板审核状态”。
  - `docs/数据库设计/V1/upgrade/065_query_execution_plane.sql:L1`：`catalog.query_surface_definition` 已冻结字段为 `asset_version_id / asset_object_id / environment_id / surface_type / binding_mode / execution_scope / input_contract_json / output_boundary_json / query_policy_json / quota_policy_json / status / metadata`。
- 补充约束文档：
  1. `docs/权限设计/接口权限校验清单.md`：`POST /api/v1/products/{id}/query-surfaces` 权限为 `delivery.query_surface.manage`，额外校验为“资产版本存在、读取区域合法、环境合法、审计”。
  2. `docs/权限设计/角色权限矩阵正式版.md`：`delivery.query_surface.manage` 角色口径为“卖方运营员、租户管理员”。
  3. `docs/权限设计/后端鉴权中间件规则说明.md`：查询面配置鉴权顺序应为“身份 -> 主体状态 -> `delivery.query_surface.manage` -> 商品/资产作用域 -> 读取分层区域合法性 -> 执行环境合法性 -> 输出边界校验 -> 审计”。
  4. `docs/数据库设计/数据库表字典正式版.md`、`docs/数据库设计/V1/upgrade/010_identity_and_access.sql`：本批核心表为 `catalog.query_surface_definition`、`catalog.product`、`catalog.asset_version`、`catalog.asset_object_binding`、`core.execution_environment`、`core.connector`。
- 当前批次目标：实现 `POST /api/v1/products/{id}/query-surfaces`，完成 QuerySurface 的创建/维护、商品作用域与执行环境合法性校验、读取区域与输出边界校验，并落审计。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv009_query_surface_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/products/{id}/query-surfaces` 联调，回查 `catalog.query_surface_definition / audit.audit_event` 后清理业务数据。

### BATCH-151（待审批）
- 状态：待审批
- 当前任务编号：DLV-009
- 前置依赖核对：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成并审批通过；`DLV-008` 已在当前主线完成并本地提交。
- 当前批次目标：实现 `POST /api/v1/products/{id}/query-surfaces`，保存可查询区域、执行环境、输出限制，并完成商品/资产/环境作用域校验与审计落库。
- 实现摘要：
  1. `delivery/api/mod.rs` 新增 `POST /api/v1/products/{id}/query-surfaces` 路由，`delivery/api/handlers.rs` 新增管理处理器。
  2. `delivery/api/support.rs` 新增 `DeliveryPermission::ManageQuerySurface`，角色口径覆盖 `seller_operator / tenant_admin / platform_*`。
  3. `delivery/dto/query_surface.rs` 新增 QuerySurface 请求/响应 DTO，冻结 `asset_object_id / environment_id / surface_type / binding_mode / execution_scope / *_json / status / operation` 结构。
  4. `delivery/repo/query_surface_repository.rs` 新增写仓储：校验卖方主体状态、商品资产版本、执行环境归属与状态、读取分层区域合法性、输出边界限制；支持“显式 `query_surface_id` 更新”与“同 asset_version + surface_type 复用更新”；同步回写 `catalog.asset_version.query_surface_type` 并写入 `delivery.query_surface.manage` 审计。
  5. `delivery/tests/dlv009_query_surface_db.rs` 新增 DB smoke，覆盖创建、更新、非法 `raw_zone/raw` 拒绝和审计/DB 断言。
  6. `packages/openapi/delivery.yaml` 已同步新增 `query-surfaces` 路径与 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/query_surface.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/query_surface_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv009_query_surface_db.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv009_query_surface_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8102 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时 `product / asset_version / asset_object_binding / connector / execution_environment` 数据，执行 3 次 `curl POST /api/v1/products/{id}/query-surfaces`（创建、更新、非法 raw_zone），最后回查 `catalog.query_surface_definition / catalog.asset_version / audit.audit_event` 并清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv009_query_surface_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`174 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - 创建 QuerySurface：`HTTP 200`，`operation=created`，`execution_scope=curated_zone`
    - 更新 QuerySurface：`HTTP 200`，`operation=updated`，`execution_scope=product_zone`，`analysis_rule=whitelist_only`
    - 非法 raw 区域：`HTTP 409`，`QUERY_SURFACE_MANAGE_FORBIDDEN: execution_scope cannot target raw zone`
  - DB 回查通过：
    - `catalog.query_surface_definition`：`execution_scope=product_zone`、`status=active`、`max_rows=200`、`analysis_rule=whitelist_only`、`daily_limit=120`
    - `catalog.asset_version.query_surface_type=template_query_lite`
    - 审计：`delivery.query_surface.manage=2`，`result_code=created/updated`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `数据商品查询与执行面设计` 3 / 4.3 / 7 / 8（QuerySurface 核心字段、只读分层、输出边界、配额与审计）
  - `页面说明书-V1` 7.6（查询面类型、执行环境绑定、输出边界、配额与模板审核状态）
  - `065_query_execution_plane.sql`（`catalog.query_surface_definition` 字段冻结）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 鉴权规则（`delivery.query_surface.manage`、商品/资产/环境作用域、读取区域与输出边界校验、审计）
- 覆盖的任务清单条目：`DLV-009`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-152（计划中）
- 状态：计划中
- 当前任务编号：DLV-010
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-010` 目标为 `POST /api/v1/query-surfaces/{id}/templates`，支持模板版本、参数 schema、输出 schema、白名单字段。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-010` 是 QuerySurface 之后的模板配置落点，要求接口、DTO、权限、审计、错误码与最小测试齐备。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务顺序、冻结文档先行、实现后必须做真实 API 联调与 DB 回查。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-151`，先登记本批计划中。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：查询模板属于受控执行面的一部分，必须与 QuerySurface/授权/审计联动，不允许自由 SQL。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：模板定义归属 Delivery，但会引用 Catalog 的 QuerySurface 真值。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已列出 `POST /api/v1/query-surfaces/{id}/templates`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先落数据库真值与审计，不提前引入额外异步桥接。
  12. `docs/开发准备/统一错误码字典正式版.md`：模板配置冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：模板接口继续落在 `delivery/api|dto|repo|tests`。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：模板定义必须冻结参数 schema、输出 schema、分析规则与导出策略。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/原始PRD/数据商品查询与执行面设计.md:L35`：`QueryTemplate` 必须表达模板名称与版本、参数定义、analysis rule/风险限制、输出结果 schema、可导出边界、风险规则与审计要求。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L668`：查询面与模板配置页核心模块包含“模板版本列表、参数 schema 配置、analysis rule 摘要、输出边界与导出策略、模板审核状态”。
  - `docs/数据库设计/V1/upgrade/065_query_execution_plane.sql:L1`：`delivery.query_template_definition` 已冻结字段为 `query_surface_id / template_name / template_type / template_body_ref / parameter_schema_json / analysis_rule_json / result_schema_json / export_policy_json / risk_guard_json / status / version_no`。
- 当前批次目标：实现 `POST /api/v1/query-surfaces/{id}/templates`，完成 QueryTemplate 的创建/维护、参数/输出 schema 冻结、白名单字段和导出策略校验，并落审计。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv010_query_template_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/query-surfaces/{id}/templates` 联调，回查 `delivery.query_template_definition / audit.audit_event` 后清理业务数据。

### BATCH-152（待审批）
- 状态：待审批
- 当前任务编号：DLV-010
- 当前批次目标：实现 `POST /api/v1/query-surfaces/{id}/templates`，完成 QueryTemplate 的创建/维护、参数/输出 schema 冻结、白名单字段和导出策略校验，并落审计。
- 前置依赖核对结果：`TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008` 已完成且审批通过；`DLV-009` 已完成并本地提交。
- 已实现功能：
  1. 新增 QueryTemplate 接口 `POST /api/v1/query-surfaces/{id}/templates`，接入 `delivery.query_template.manage` 权限。
  2. 新增 DTO：`ManageQueryTemplateRequest/Response` 与 `QueryTemplateResponseData`。
  3. 新增仓储 `manage_query_template(...)`，落库 `delivery.query_template_definition`。
  4. 支持模板版本策略：
     - 首次创建默认 `version_no=1`
     - 同名模板未显式给定版本时自动按 `max(version_no)+1` 创建新版本
     - 传入 `query_template_id` 时只允许更新既有版本，不允许改写版本号
  5. 落地参数 schema / 输出 schema / analysis rule / export policy / risk guard 五类冻结 JSON 字段。
  6. 落地白名单字段校验：去重、非空、必须命中 `result_schema_json.properties|fields` 声明，并同步回填到 `analysis_rule_json.whitelist_fields` 与 `export_policy_json.whitelist_fields`。
  7. 拒绝 `allow_raw_export=true`、拒绝 raw 导出格式、拒绝 `free_sql`、拒绝 `risk_mode=bypass`。
  8. 维持 QuerySurface / 卖方主体 / 资产版本 / tenant scope 联动校验，审计动作 `delivery.query_template.manage`。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/query_template.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/query_template_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv010_query_template_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv010_query_template_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8103 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时卖方/资产/商品/执行环境/QuerySurface 数据，执行真实 `curl POST /api/v1/query-surfaces/{id}/templates` 联调（创建 v1、创建 v2、更新 v2、非法白名单），回查 `delivery.query_template_definition / audit.audit_event` 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv010_query_template_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`175 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - 创建 v1：`HTTP 200`，`operation=created`，`version_no=1`
    - 创建 v2：`HTTP 200`，`operation=created`，`version_no=2`
    - 更新 v2：`HTTP 200`，`operation=updated`，`version_no=2`，`whitelist_fields=["city","total_amount","confidence"]`
    - 非法白名单：`HTTP 409`，`QUERY_TEMPLATE_MANAGE_FORBIDDEN: whitelist field \`missing_field\` is not declared in result_schema_json`
  - DB 回查通过：
    - `delivery.query_template_definition`：同名模板存在 2 个版本；最新 `template_body_ref` 为 patch 后版本；`analysis_rule_json/export_policy_json.whitelist_fields` 已同步写入。
    - 审计：`delivery.query_template.manage=3`，对应 `created/created/updated`。
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `数据商品查询与执行面设计` 3 / 5 / 11（QueryTemplate 核心对象、V1 闭环、数据库落点）
  - `页面说明书-V1` 7.6（模板版本列表、参数 schema、analysis rule、输出边界、模板审核状态）
  - `065_query_execution_plane.sql`（`delivery.query_template_definition` 字段冻结）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 鉴权规则（`delivery.query_template.manage`、QuerySurface 作用域、参数 schema / analysis / 导出策略 / 审计链）
- 覆盖的任务清单条目：`DLV-010`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-153（计划中）
- 状态：计划中
- 当前任务编号：DLV-011
- 已阅读证据（文件 + 要点）：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-011` 目标为 `POST /api/v1/orders/{id}/template-grants`，只允许命中白名单模板。
  2. `docs/开发任务/v1-core-开发任务清单.md`：复核 `DLV-011` 仍要求接口、DTO、权限校验、审计、错误码与最小测试齐备。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地 commit”执行。
  4. `docs/开发任务/AI-Agent-执行提示词.md`：保持单任务顺序、冻结文档先行、实现后必须做真实 API 联调与 DB 回查。
  5. `docs/开发任务/V1-Core-实施进度日志-P2.md`：承接 `BATCH-152`，先登记本批计划中。
  6. `docs/开发任务/V1-Core-TODO与预留清单.md`：完成后追加本批追溯记录，持续保留 `TODO-PROC-BIL-001`。
  7. `docs/开发任务/V1-Core-人工审批记录.md`：只读，不写入。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：`QRY_LITE` 的主交付对象是 `template grant / 白名单模板执行`，买方成交后应创建 `template_query_grant` 并绑定模板摘要、输出边界、执行配额。
  9. `docs/开发准备/服务清单与服务边界正式版.md`：模板授权归属 Delivery，但状态推进仍需和 Trade 订单主状态一致。
  10. `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：冻结表已列出 `POST /api/v1/orders/{id}/template-grants`。
  11. `docs/开发准备/事件模型与Topic清单正式版.md`：本批先落数据库真值与审计，不提前引入额外异步桥接。
  12. `docs/开发准备/统一错误码字典正式版.md`：模板白名单冲突/权限/内部错误继续沿用统一错误结构。
  13. `docs/开发准备/测试用例矩阵正式版.md`：本批需要最小 DB smoke + 真实 API 联调 + 数据库回查。
  14. `docs/开发准备/仓库拆分与目录结构建议.md`：模板授权继续落在 `delivery/api|dto|repo|tests`。
  15. `docs/开发准备/本地开发环境与中间件部署清单.md`：本批联调继续使用 `datab-postgres` 与本地 core 栈。
  16. `docs/开发准备/配置项与密钥管理清单.md`：沿用本地环境变量与固定 DSN/端口口径。
  17. `docs/开发准备/技术选型正式版.md`：沿用当前 `SQLx + SeaORM` 数据访问基线，不回退旧实现。
  18. `docs/开发准备/平台总体架构设计草案.md`：模板授权必须和受控执行、输出边界、审计可追溯绑定。
- 当前任务额外引用的 `technical_reference` 与约束映射：
  - `docs/原始PRD/数据商品查询与执行面设计.md:L127`：`QRY_LITE` 成交后需创建 `template_query_grant`，绑定模板摘要、输出边界、执行配额，执行时按白名单模板/参数范围/导出限制校验。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L639`：模板查询开通页需展示模板白名单、参数 schema 摘要、输出边界与导出限制。
  - `docs/页面说明书/页面说明书-V1-完整版.md:L685`：查询运行记录页依赖 `query surface / template version / 参数摘要 / 结果摘要`，因此授权快照必须足以支撑后续执行与审计。
  - `docs/数据库设计/V1/upgrade/061_data_object_trade_modes.sql` 与 `065_query_execution_plane.sql`：`delivery.template_query_grant` 冻结字段包含 `query_surface_id / allowed_template_ids / execution_rule_snapshot / output_boundary_json / run_quota_json`。
  - `docs/权限设计/接口权限校验清单.md` / `角色权限矩阵正式版.md` / `后端鉴权中间件规则说明.md`：鉴权顺序为 身份 -> 主体状态 -> `delivery.template_query.enable` -> 订单作用域 -> 模板白名单 -> 参数校验 -> 输出边界 -> 风控 -> 审计。
- 当前批次目标：实现模板授权接口 `POST /api/v1/orders/{id}/template-grants`，只允许命中当前 QuerySurface 的白名单模板，并同步交付记录、订单状态与审计。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv011_template_grant_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/orders/{id}/template-grants` 联调，回查 `delivery.template_query_grant / delivery.delivery_record / trade.order_main / audit.audit_event` 后清理业务数据。

### BATCH-153（待审批）
- 状态：待审批
- 当前任务编号：DLV-011
- 当前批次目标：实现模板授权接口 `POST /api/v1/orders/{id}/template-grants`，只允许命中当前 QuerySurface 的白名单模板，并同步交付记录、订单状态与审计。
- 前置依赖核对结果：`TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008` 已完成且审批通过；`DLV-010` 已完成并本地提交。
- 已实现功能：
  1. 新增模板授权接口 `POST /api/v1/orders/{id}/template-grants`，接入 `delivery.template_query.enable` 权限。
  2. 新增 DTO：`ManageTemplateGrantRequest/Response` 与 `TemplateGrantResponseData`。
  3. 新增仓储 `manage_template_grant(...)`，写入/更新 `delivery.template_query_grant`，冻结 `query_surface_id / allowed_template_ids / execution_rule_snapshot / output_boundary_json / run_quota_json`。
  4. 落地鉴权与作用域校验：平台角色直通；卖方 `seller_operator` 仅限卖方租户；`tenant_developer` 仅限买方租户；`tenant_admin` 允许买卖双方租户。
  5. 落地模板白名单校验：只允许命中当前 QuerySurface 的 `active` 模板；模板必须共享同一 `template_type`；禁止混入其他 QuerySurface 模板。
  6. 落地输出边界与配额校验：禁止 raw export；请求的 `allowed_formats / max_rows / max_cells` 不得超出 QuerySurface 与模板导出策略；`run_quota_json.max_runs/daily_limit/monthly_limit` 必须为正整数。
  7. 落地执行规则快照：写入模板摘要、参数 schema、结果 schema、analysis rule、输出边界、配额和授权角色，供后续 `DLV-012/013` 查询执行与运行记录复用。
  8. 与订单/交付联动：模板授权首次创建时通过统一交付门禁；更新时复用既有 `committed` 交付记录，不再残留新的 `prepared` 记录；订单状态推进/保持为 `template_authorized`，交付记录保持 `committed/template_grant/template_query`。
  9. 审计落地：`delivery.template_query.enable` 与 `trade.order.qry_lite.transition`。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/template_grant.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/template_grant_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv011_template_grant_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/order/repo/order_deliverability_repository.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv011_template_grant_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8104 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时买卖方/资产/商品/执行环境/QuerySurface/QueryTemplate 数据，执行真实 `curl POST /api/v1/orders/{id}/template-grants` 联调（创建、更新、非法跨 QuerySurface 模板），回查 `delivery.template_query_grant / delivery.delivery_record / trade.order_main / audit.audit_event` 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`176 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv011_template_grant_db_smoke`：通过。
  - 真实 API 联调通过：
    - 创建授权：`HTTP 200`，`operation=granted`，`current_state=template_authorized`，`allowed_template_ids=2`
    - 更新授权：`HTTP 200`，`operation=updated`，`current_state=template_authorized`，`run_quota_json.max_runs=10`
    - 非法模板：`HTTP 409`，`TEMPLATE_GRANT_FORBIDDEN: allowed_template_ids contains template outside current query surface`
  - DB 回查通过：
    - `delivery.template_query_grant`：`grant_status=active`，`template_type=sql_template`，`output_boundary_json.max_rows=25`，`run_quota_json.max_runs=10`，`execution_rule_snapshot.grant_source=buyer_update`，`allowed_template_ids` 已缩减到更新后白名单。
    - `delivery.delivery_record`：保持 `committed / template_grant / template_query`，`delivery_commit_hash = receipt_hash`，更新授权不再残留新的 `prepared` 记录。
    - `trade.order_main`：保持 `template_authorized / paid / in_progress / not_started / pending_settlement`。
    - 审计：`delivery.template_query.enable=2`，`trade.order.qry_lite.transition=2`。
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `数据商品查询与执行面设计` 5 / 8（V1 闭环、查询权独立表达、模板白名单/输出边界/配额/审计）
  - `页面说明书-V1` 7.4 / 7.7（模板查询开通页、查询运行与结果记录页）
  - `061_data_object_trade_modes.sql` / `065_query_execution_plane.sql`（`delivery.template_query_grant`、`delivery.query_execution_run` 字段冻结）
  - `数据库表字典正式版` / `全量领域模型与对象关系说明`（`TemplateQueryGrant` 职责与关联）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 鉴权规则（`delivery.template_query.enable` 与鉴权顺序）
- 覆盖的任务清单条目：`DLV-011`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-154（计划中）
- 状态：计划中
- 当前任务编号：DLV-012
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-012` 为 `POST /api/v1/orders/{id}/template-runs`，要求参数校验、风控校验、输出边界校验、审计，交付路径为 `delivery/**`、`storage/**` 与 `packages/openapi/delivery.yaml`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务完成定义为接口、DTO、权限校验、审计、错误码和最小测试齐备，并要求至少一条集成测试或手工 API 验证通过。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `AI-Agent-执行提示词.md`：确认单任务批次必须先写日志“计划中”，实现后完整验证、更新 TODO、写“待审批”、本地提交。
  4. `docs/开发任务/V1-Core-TODO与预留清单.md` / `docs/开发任务/V1-Core-人工审批记录.md`：确认 `TODO-PROC-BIL-001` 追溯约束保持不变，人工审批记录继续由人工维护。
  5. `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认模板查询必须通过白名单模板、参数 schema、输出边界和导出限制校验后才能执行，并为每次执行形成正式运行记录。
  6. `docs/原始PRD/数据商品查询与执行面设计.md:L127`：确认 `QuerySurface -> QueryTemplate -> QueryGrant -> QueryExecutionRun -> ResultArtifact` 的 V1 闭环，以及执行记录需包含发起人、模板、输入参数摘要、输出摘要、计费单位、审计与风控结果。
  7. `docs/页面说明书/页面说明书-V1-完整版.md:L639`：模板查询开通页需展示模板白名单、参数 schema 摘要、输出边界和导出限制。
  8. `docs/页面说明书/页面说明书-V1-完整版.md:L685`：查询运行记录页需展示 query run 时间线、模板版本、请求参数摘要、结果摘要/结果对象、计费单位、审计引用与策略命中。
  9. `docs/权限设计/接口权限校验清单.md` / `角色权限矩阵正式版.md` / `后端鉴权中间件规则说明.md`：确认权限为 `delivery.template_query.use`，鉴权顺序为身份 -> 主体状态 -> 权限 -> 订单作用域 -> 模板白名单 -> 参数校验 -> 输出边界 -> 风控 -> 审计。
  10. `docs/数据库设计/V1/upgrade/065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql` / `数据库表字典正式版.md`：确认 `delivery.query_execution_run` 与敏感字段 `masked_level / export_scope / approval_ticket_id / sensitive_policy_snapshot` 的落库结构。
  11. 其余必读冻结文档已按本阶段基线复核标题、边界与服务/事件/配置口径，无新增冲突；当前任务未发现需要升级到问题清单的文档矛盾。
- 当前批次目标：实现模板执行接口 `POST /api/v1/orders/{id}/template-runs`，完成模板授权有效性、参数 schema、输出边界、风控、结果对象（MinIO）、`delivery.query_execution_run`、订单状态联动与审计闭环。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `apps/platform-core/src/modules/storage/application/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv012_template_run_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + MinIO + `curl POST /api/v1/orders/{id}/template-runs` 联调，回查 `delivery.query_execution_run / delivery.storage_object / trade.order_main / audit.audit_event` 后清理业务数据。

### BATCH-154（待审批）
- 状态：待审批
- 当前任务编号：DLV-012
- 当前批次目标：实现模板执行接口 `POST /api/v1/orders/{id}/template-runs`，完成模板授权有效性、参数 schema、输出边界、风控、结果对象（MinIO）、`delivery.query_execution_run`、订单状态联动与审计闭环。
- 前置依赖核对结果：`TRADE-003; TRADE-007; DB-006; DB-019; DB-020; CORE-008` 已完成且审批通过；`DLV-011` 已完成并本地提交。
- 已实现功能：
  1. 新增模板执行接口 `POST /api/v1/orders/{id}/template-runs`，接入 `delivery.template_query.use` 权限。
  2. 新增 DTO：`ExecuteTemplateRunRequest/Response`、`QueryRunResponseData`，返回模板版本、结果对象、计费单位、策略快照、状态推进结果。
  3. 新增仓储 `execute_template_run(...)`，落地模板授权读取、订单/主体作用域校验、请求人校验、参数 schema 校验、输出边界校验、风控校验、配额校验。
  4. 新增 `delivery.query_execution_run` 写路径：先写 `running`，生成结果摘要和敏感策略快照后回写 `completed / completed_at / result_object_id / billed_units / result_row_count`。
  5. 新增 MinIO 结果对象落桶：通过 `storage::put_object_bytes(...)` 写入 `report-results/query-runs/{order_id}/{query_run_id}/result.json`，并同步登记 `delivery.storage_object(result_object)`。
  6. 新增 `QRY_LITE` 状态联动：从 `template_authorized` 推进到 `query_executed`，并同步刷新 `delivery_status=delivered / acceptance_status=accepted / settlement_status=pending_settlement`。
  7. 审计落地：`delivery.template_query.use` 与 `trade.order.qry_lite.transition`。
  8. 新增专项 DB smoke `dlv012_template_run_db_smoke`，覆盖成功执行、缺失审批票据、非法输出格式三条链路，并回查 MinIO 结果对象。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/query_run.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/query_run_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv012_template_run_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/storage/application/mod.rs`
  - `apps/platform-core/src/modules/storage/application/object_store.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv012_template_run_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8105 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时买卖方/资产/商品/执行环境/QuerySurface/QueryTemplate/Grant 数据，执行真实 `curl POST /api/v1/orders/{id}/template-runs` 联调，回查 `delivery.query_execution_run / delivery.storage_object / trade.order_main / audit.audit_event` 与 MinIO 对象路径后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`177 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv012_template_run_db_smoke`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/template-runs`：`HTTP 200`
    - 返回 `status=completed`、`current_state=query_executed`、`result_row_count=2`、`masked_level=masked`、`export_scope=none`
  - DB 回查通过：
    - `delivery.query_execution_run`：`completed / template_query / masked / none / 2 / 1.00000000`
    - `trade.order_main`：`query_executed / paid / delivered / accepted / pending_settlement`
    - `delivery.storage_object`：登记 `s3://report-results/query-runs/{order_id}/{query_run_id}/result.json`
    - 审计：`delivery.template_query.use=1`、`trade.order.qry_lite.transition=1`
  - MinIO 联调通过：
    - `report-results/query-runs/{order_id}/{query_run_id}/result.json` 对象路径已在 `datab-minio` 内创建并可见。
  - 清理结果：临时业务数据与 MinIO 临时对象路径已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `数据商品查询与执行面设计` 5 / 8（执行闭环、参数/边界/风控/计费/审计）
  - `页面说明书-V1` 7.4 / 7.7（模板执行入口、运行记录页字段）
  - `065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql`（`delivery.query_execution_run` 与敏感字段冻结）
  - `数据库表字典正式版` / `全量领域模型与对象关系说明`（`QueryExecutionRun` 与结果对象职责）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 鉴权规则（`delivery.template_query.use` 与鉴权顺序）
- 覆盖的任务清单条目：`DLV-012`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-155（计划中）
- 状态：计划中
- 当前任务编号：DLV-013
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-013` 为 `GET /api/v1/orders/{id}/query-runs`，要求接口、DTO、权限校验、审计、错误码与最小测试齐备，交付路径为 `delivery/**`、`storage/**` 与 `packages/openapi/delivery.yaml`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务完成定义为查询运行记录接口不漂移，且至少一条集成测试或手工 API 验证通过并能在审计/日志中看到痕迹。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `AI-Agent-执行提示词.md`：确认单任务批次必须先写日志“计划中”，实现后完整验证、更新 TODO、写“待审批”、本地提交。
  4. `docs/开发任务/V1-Core-TODO与预留清单.md` / `docs/开发任务/V1-Core-人工审批记录.md`：确认 `TODO-PROC-BIL-001` 追溯约束保持不变，人工审批记录继续由人工维护。
  5. `docs/原始PRD/数据商品查询与执行面设计.md:L127`：确认 V1 闭环要求模板/沙箱每次执行都记录 `QueryExecutionRun`，并保留模板版本、参数摘要、输出摘要、运行人/应用、导出动作与风控结果。
  6. `docs/页面说明书/页面说明书-V1-完整版.md:L639`：模板查询开通页需要展示模板白名单、参数边界和结果导出限制，查询运行记录页必须与授权快照保持一致。
  7. `docs/页面说明书/页面说明书-V1-完整版.md:L685`：查询运行与结果记录页需要展示 query run 时间线、模板版本与 query surface 引用、请求参数摘要、结果摘要与结果对象、计费单位与配额消耗、审计引用与策略命中。
  8. `docs/权限设计/接口权限校验清单.md` / `角色权限矩阵正式版.md` / `后端鉴权中间件规则说明.md`：确认权限为 `delivery.template_query.use`，鉴权顺序仍为 身份 -> 主体状态 -> 权限 -> 订单作用域 -> 模板白名单 -> 参数校验 -> 输出边界 -> 风控 -> 审计。
  9. `docs/数据库设计/V1/upgrade/065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql` / `数据库表字典正式版.md`：确认 `delivery.query_execution_run` 的冻结字段包括 `request_payload_json / result_summary_json / result_object_id / billed_units / export_attempt_count / masked_level / export_scope / approval_ticket_id / sensitive_policy_snapshot`。
  10. 其余必读冻结文档已按当前阶段基线复核服务边界、事件、错误码、环境与配置口径，无新增冲突；当前任务未发现需要升级到问题清单的文档矛盾。
- 当前批次目标：实现查询运行记录接口 `GET /api/v1/orders/{id}/query-runs`，返回订单下模板查询运行时间线、模板版本/QuerySurface、参数摘要、结果摘要与结果对象、计费单位、审计引用与策略命中，并完成权限、审计和最小测试闭环。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv013_query_runs_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + MinIO + `curl GET /api/v1/orders/{id}/query-runs` 联调，回查 `delivery.query_execution_run / audit.audit_event / delivery.storage_object` 后清理业务数据。

### BATCH-155（待审批）
- 状态：待审批
- 当前任务编号：DLV-013
- 前置依赖核对：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成并审批通过。
- 当前批次目标：实现查询运行记录接口 `GET /api/v1/orders/{id}/query-runs`，返回订单下模板查询运行时间线、模板版本/QuerySurface、参数摘要、结果摘要与结果对象、计费单位、审计引用与策略命中，并保持权限、审计与错误码口径一致。
- 已实现功能：
  - 新增订单作用域运行记录读取仓储 `get_query_runs(...)`，复用 `delivery.template_query.use` 权限与订单上下文校验。
  - 运行记录响应补齐 `parameter_summary_json`、`policy_hits`、`audit_refs`，并在查询时联查 `audit.audit_event` 与 `delivery.storage_object` 解析对象路径。
  - 新增 `GET /api/v1/orders/{id}/template-runs` handler 与 OpenAPI 契约；保留既有 `POST /api/v1/orders/{id}/template-runs` 行为不变。
  - 新增 `dlv013_query_runs_db_smoke`，覆盖两次查询执行后的运行记录读取、排序、策略命中、审计引用和读审计。
  - 读取接口写入审计 `delivery.template_query.run.read`。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/query_run.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/query_run_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/query_run_read_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv013_query_runs_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv013_query_runs_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8106 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时买卖方/资产/商品/执行环境/QuerySurface/QueryTemplate/Grant 数据，执行两次真实 `curl POST /api/v1/orders/{id}/template-runs` 和一次 `curl GET /api/v1/orders/{id}/template-runs` 联调，回查 `delivery.query_execution_run / delivery.storage_object / trade.order_main / audit.audit_event` 与 MinIO 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`178 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv013_query_runs_db_smoke`：通过。
  - 真实 API 联调通过：
    - 两次 `POST /api/v1/orders/{id}/template-runs`：`HTTP 200`
    - `GET /api/v1/orders/{id}/template-runs`：`HTTP 200`
    - 返回 `query_run_count=2`，首条 `policy_hit=template_whitelist_passed`，首条审计引用 `action_name=delivery.template_query.use`
    - `trade.order_main` 保持 `query_executed / delivered`
  - DB / MinIO 回查通过：
    - `delivery.query_execution_run` 共 2 条，按创建时间倒序读取
    - `audit.audit_event` 命中 `delivery.template_query.run.read=1`
    - MinIO 结果对象路径存在且可校验
  - 清理结果：临时业务数据与 MinIO 临时对象路径已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `数据商品查询与执行面设计` 5 / 8（运行记录、参数摘要、结果摘要、计费与审计）
  - `页面说明书-V1` 7.4 / 7.7（模板执行入口与查询运行记录页字段）
  - `065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql`（`delivery.query_execution_run` 冻结字段与敏感交付口径）
  - `权限设计` 接口权限校验清单 / 角色矩阵 / 鉴权规则（`delivery.template_query.use`）
- 覆盖的任务清单条目：`DLV-013`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-156（计划中）
- 状态：计划中
- 当前任务编号：DLV-014
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-014` 为 `POST /api/v1/orders/{id}/sandbox-workspaces`，要求接口、DTO、权限校验、审计、错误码和最小测试齐备，交付路径为 `delivery/**`、`storage/**` 与 `packages/openapi/delivery.yaml`。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务完成定义为实现与 OpenAPI 不漂移，且至少一条集成测试或手工 API 验证通过并能在审计/日志中看到痕迹。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次必须先写日志“计划中”，实现后完整验证、更新 TODO、写“待审批”、本地提交。
  4. `docs/开发任务/V1-Core-TODO与预留清单.md` / `docs/开发任务/V1-Core-人工审批记录.md`：确认 `TODO-PROC-BIL-001` 追溯约束保持不变，人工审批记录继续由人工维护。
  5. `docs/业务流程/业务流程图-V1-完整版.md:L349`：确认沙箱交付链路必须创建隔离执行环境、注入最小权限账号与到期时间、控制导出/复制/网络访问，并在到期或次数上限后自动关闭会话。
  6. `docs/页面说明书/页面说明书-V1-完整版.md:L654`：查询沙箱开通页必须展示沙箱实例状态、账号信息、会话时效、环境限制说明和导出限制提示。
  7. `docs/原始PRD/数据商品查询与执行面设计.md:L127`：确认 `sandbox_query` 必须绑定正式执行环境、会话时效、导出策略和最小网络边界，不允许直接发放底库账号。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L10072` / `L7128` / `L24052`：确认接口权限为 `delivery.sandbox.enable`，校验项覆盖执行环境、会话策略、导出限制与审计，实体字段以 `delivery.sandbox_workspace / delivery.sandbox_session` 为准。
  9. `docs/数据库设计/V1/upgrade/030_trade_delivery.sql` / `065_query_execution_plane.sql` / `数据库表字典正式版.md`：确认 `sandbox_workspace` 的冻结字段包含 `query_surface_id / clean_room_mode / data_residency_mode / output_boundary_json`，`sandbox_session` 承载 seat / query_count / export_attempt_count。
  10. 其余 18 份必读冻结文档已按当前阶段基线复核服务边界、事件、错误码、环境与配置口径，无新增冲突；当前任务未发现需要升级到问题清单的文档矛盾。
- 当前批次目标：实现沙箱开通接口 `POST /api/v1/orders/{id}/sandbox-workspaces`，完成 `SBX_STD` 订单下工作区与 seat 会话开通，写入执行环境、会话时效、导出限制、状态和审计，并通过真实 API / DB 联调验证。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv014_sandbox_workspace_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sandbox_workspace / delivery.sandbox_session / delivery.delivery_record / audit.audit_event` 后清理业务数据。

### BATCH-156（待审批）
- 状态：待审批
- 当前任务编号：DLV-014
- 实现内容：
  1. 新增 `POST /api/v1/orders/{id}/sandbox-workspaces`，为 `SBX_STD` 订单提供沙箱工作区与 seat 会话开通/更新接口。
  2. 落地 `ManageSandboxWorkspaceRequest/Response`，稳定返回 workspace/session、执行环境、导出限制、输出边界、状态与时效字段。
  3. 新增 `sandbox_workspace_repository`，校验订单租户、SKU、可交付门禁、QuerySurface、执行环境、seat 用户、到期时间与导出策略，并事务内 upsert `delivery.sandbox_workspace / delivery.sandbox_session`。
  4. 联动更新 `delivery.delivery_record` 与 `trade.order_main`，在开通成功后推进到 `seat_issued / in_progress`，并写入 `delivery.sandbox.enable` 与 `trade.order.sbx_std.transition` 审计。
  5. 同步更新 `packages/openapi/delivery.yaml` 与 `dlv014_sandbox_workspace_db_smoke`。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/mod.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/sandbox_workspace.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/sandbox_workspace_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv014_sandbox_workspace_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv014_sandbox_workspace_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8107 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时买卖方/用户/资产/商品/执行环境/QuerySurface/已签约订单数据，执行真实 `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sandbox_workspace / delivery.sandbox_session / delivery.delivery_record / trade.order_main / audit.audit_event` 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`179 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过；`.sqlx` 离线元数据已刷新。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv014_sandbox_workspace_db_smoke`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/sandbox-workspaces`：`HTTP 200`
    - 返回 `workspace_id=0073affb-7904-48c6-abd0-acc4ed3c44f2`、`session_id=44e8c7f1-99ff-4c3f-8a52-e5acc531819f`
    - `current_state=seat_issued`、`workspace_status=active`、`session_status=active`、`environment_type=sandbox`
    - `allow_export=false`
  - DB 回查通过：
    - `trade.order_main` 推进到 `seat_issued / in_progress`
    - `delivery.sandbox_workspace` 与 `delivery.sandbox_session` 各 1 条
    - `delivery.delivery_record` 更新为 `sandbox_workspace / sandbox_query`
    - `audit.audit_event` 命中 `delivery.sandbox.enable=1`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.5（沙箱交付开通链路）
  - `页面说明书-V1` 7.5（沙箱实例状态、账号信息、时效与限制提示）
  - `数据商品查询与执行面设计` 5 / 8（正式执行环境、导出限制、最小网络边界）
  - `全集成基线-V1` 接口权限与交付模型章节（`delivery.sandbox.enable`、`delivery.sandbox_workspace / sandbox_session`）
- 覆盖的任务清单条目：`DLV-014`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-157（计划中）
- 状态：计划中
- 当前任务编号：DLV-015
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-015` 目标为“沙箱工作区模型：workspace、session、seat、export control、attestation 引用”，完成定义要求业务规则、状态机、审计、事件与测试齐备。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务是模型冻结，不是另起一条独立交付类型；需在现有 Delivery/Storage/Query 边界内实现并联调通过。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `docs/开发任务/AI-Agent-执行提示词.md`：确认继续按单任务批次执行，先写“计划中”，实现后完整验证、更新 TODO、写“待审批”、本地提交后直接进入下一个任务。
  4. `docs/开发任务/V1-Core-TODO与预留清单.md` / `docs/开发任务/V1-Core-人工审批记录.md`：确认 `TODO-PROC-BIL-001` 追溯约束持续保留，人工审批记录继续由人工维护。
  5. `docs/业务流程/业务流程图-V1-完整版.md:L349`：确认沙箱链路必须创建隔离执行环境、注入最小权限账号与到期时间、采集 `query_log / session_log / policy_hit / export_attempt`，并在到期或次数上限后自动关闭会话。
  6. `docs/页面说明书/页面说明书-V1-完整版.md:L654`：确认查询沙箱开通页必须展示沙箱实例状态、账号信息、会话时效、环境限制说明与导出限制提示。
  7. `docs/原始PRD/数据商品查询与执行面设计.md:L127`：确认 `sandbox_query` 的 V1 闭环要求把查询权、运行时长、导出次数纳入独立权利表达，并将结果/审计/计费关系固定下来。
  8. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L24052` / `L26921` / `L7128`：确认 `delivery.sandbox_workspace / sandbox_session / sensitive_execution_policy / attestation_record` 的冻结字段和“正式执行环境 + 会话时效 + 导出策略 + 最小网络边界 + attestation 引用”口径。
  9. `docs/数据库设计/V1/upgrade/030_trade_delivery.sql` / `065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql`：确认沙箱工作区、会话、受控执行策略与 attestation 表结构已冻结，可直接落库而无需新增 migration。
  10. 其余 18 份必读冻结文档已按当前阶段基线复核服务边界、事件、错误码、环境与配置口径，无新增冲突；当前任务未发现需要升级为问题清单的矛盾。
- 当前批次目标：在现有 `POST /api/v1/orders/{id}/sandbox-workspaces` 开通链路中补齐沙箱模型冻结，稳定持久化并返回 workspace / session / seat / export control / attestation 引用，并完成真实 API、DB 联调验证。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv015_sandbox_model_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sandbox_workspace / delivery.sandbox_session / delivery.sensitive_execution_policy / delivery.attestation_record / audit.audit_event` 后清理业务数据。

### BATCH-157（待审批）
- 状态：待审批
- 当前任务编号：DLV-015
- 已实现功能：
  - 在既有 `POST /api/v1/orders/{id}/sandbox-workspaces` 链路中补齐沙箱模型冻结，响应新增 `sensitive_execution_policy_id` 与 `workspace / session / seat / export_control / attestation` 五段结构。
  - 新增 `sandbox_workspace_model_repository`，将 `delivery.sensitive_execution_policy` 与 `delivery.attestation_record` 作为稳定模型持久化；`policy_scope=sandbox_workspace`、`execution_mode=sandbox_query`、`status=active`。
  - 根据 QuerySurface、执行环境与请求导出边界冻结 seat 限额、step-up、attestation、network access 等 export-control 参数，并写入 `delivery.delivery_record.trust_boundary_snapshot`。
  - 扩展 `delivery.sandbox.enable` 审计元数据，补充 `sensitive_execution_policy_id` 与 `attestation_record_id`，保证联查可追溯。
  - 新增 `dlv015_sandbox_model_db_smoke`，覆盖工作区模型、策略记录、attestation 引用与审计字段断言。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/sandbox_workspace.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/repo/sandbox_workspace_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/sandbox_workspace_model_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv015_sandbox_model_db.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv015_sandbox_model_db_smoke -- --nocapture`
  7. 启动服务：`APP_PORT=8108 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入临时买卖方/用户/资产/商品/执行环境/QuerySurface/已签约订单数据，执行真实 `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sensitive_execution_policy / delivery.attestation_record / delivery.delivery_record / audit.audit_event` 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `cargo test -p platform-core`：通过（`180 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过。
  - `./scripts/check-query-compile.sh`：通过。
  - `dlv015_sandbox_model_db_smoke`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/sandbox-workspaces`：`HTTP 200`
    - 返回 `policy_id=7bdf12f5-2c74-4005-bf23-252982edca62`、`attestation_id=4e1a0ebc-d4a0-4bab-a3ef-76f4e32c89e9`
    - `seat_limit=3`、`step_up_required=true`、`attestation_required=true`、`verifier_ref=sandbox-verifier`
  - DB 回查通过：
    - `delivery.sensitive_execution_policy`：`status=active / execution_mode=sandbox_query / export_control_json.seat_limit=3 / export_control_json.network_access=deny`
    - `delivery.attestation_record`：`status=pending / attestation_type=execution_receipt / verifier_ref=sandbox-verifier`
    - `audit.audit_event` 命中 `delivery.sandbox.enable`，且 metadata 含 `sensitive_execution_policy_id / attestation_record_id`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.3（沙箱 / 模板查询类交付）
  - `页面说明书-V1` 7.5（查询沙箱开通页）
  - `数据商品查询与执行面设计` 5（V1 业务闭环）
  - `全集成基线-V1` 中 `delivery.sandbox_workspace / sandbox_session / sensitive_execution_policy / attestation_record` 冻结字段口径
- 覆盖的任务清单条目：`DLV-015`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-158（计划中）
- 状态：计划中
- 当前任务编号：DLV-016
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-016` 目标为“为 `SBX_STD` 预留 gVisor 执行隔离参数位，即便 local 模式先只做配置占位，也要把环境模型冻结”。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务属于沙箱工作区链路的模型补全，不是新增独立交付分支；仍需走既有 `sandbox workspace` API、状态机、审计与测试闭环。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `docs/开发任务/AI-Agent-执行提示词.md`：确认本批继续按单任务流程执行，完整验证后本地提交并自动推进下一任务。
  4. `docs/业务流程/业务流程图-V1-完整版.md:L349`：确认沙箱链路必须创建隔离执行环境，并对导出、复制、网络访问等策略执行点进行控制。
  5. `docs/页面说明书/页面说明书-V1-完整版.md:L654`：确认查询沙箱开通页必须稳定展示环境限制说明，因此执行环境与隔离参数不能只停留在松散 metadata。
  6. `docs/原始PRD/数据商品查询与执行面设计.md:L127`：确认 `sandbox_query` 作为 V1 闭环能力必须把运行时长、导出次数、执行边界和后续审计/计费关系冻结。
  7. `docs/全集成文档/数据交易平台-全集成基线-V1.md:L727` / `L1396` / `L19468`：确认执行环境实体最小字段包括 `isolation_level / export_policy / audit_policy / trusted_attestation_flag / supported_product_types / current_capacity / environment_status`，且 V1 查询沙箱隔离优先评估 `gVisor`。
  8. `docs/开发准备/技术选型正式版.md:L192`：确认查询沙箱 / 模板执行隔离的推荐占位技术为 `gVisor`，但 V1 不把更重的 `TEE / Kata / MPC / FL` 作为正式运行依赖。
  9. `docs/数据库设计/V1/upgrade/010_identity_and_access.sql` / `065_query_execution_plane.sql` / `066_sensitive_data_controlled_delivery.sql`：确认 `core.execution_environment` 仅提供 `metadata` 承载扩展字段，本批应通过模型冻结与快照持久化落地，不新增 migration。
  10. 其余 18 份必读冻结文档已按阶段基线复核服务边界、错误码、事件、环境与配置口径；当前未发现需要升级为问题清单的冲突。
- 当前批次目标：在 `SBX_STD` 的沙箱开通链路中冻结执行环境模型与 gVisor 隔离参数占位，稳定写入响应、策略快照、trust-boundary 快照和审计元数据，并完成真实 API/DB 联调验证。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv016_sandbox_isolation_model_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sensitive_execution_policy / delivery.delivery_record / audit.audit_event` 中的执行环境与 gVisor 占位快照后清理业务数据。

### BATCH-158（待审批）
- 状态：待审批
- 当前任务编号：DLV-016
- 已实现功能：
  - 在 `SBX_STD` 沙箱开通链路中冻结执行环境模型，响应新增 `execution_environment` 结构，稳定输出 `environment_status / isolation_level / export_policy_json / audit_policy_json / trusted_attestation_flag / supported_product_types / current_capacity_json / runtime_isolation`。
  - 为 `gVisor` 预留 V1 占位参数位：`runtime_provider / runtime_mode / runtime_class / profile_name / rootfs_mode / network_mode / seccomp_profile / status`，默认兼容 local 模式的 `local_placeholder + runsc` 口径。
  - 将执行环境模型与 `gVisor` 占位同步写入 `delivery.sensitive_execution_policy.policy_snapshot`、`delivery.delivery_record.trust_boundary_snapshot` 以及 `delivery.sandbox.enable` 审计元数据，避免只留在松散 metadata 中。
  - 新增 `dlv016_sandbox_isolation_model_db_smoke`，覆盖响应、策略快照、trust-boundary 快照与审计字段断言。
  - 同步更新 `packages/openapi/delivery.yaml`，冻结新增 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/dto/mod.rs`
  - `apps/platform-core/src/modules/delivery/dto/sandbox_workspace.rs`
  - `apps/platform-core/src/modules/delivery/repo/sandbox_workspace_model_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/sandbox_workspace_repository.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv016_sandbox_isolation_model_db.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv016_sandbox_isolation_model_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8109 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用 `psql` 写入带 `runtime_isolation=gvisor` 占位的临时 `SBX_STD` 数据，执行真实 `curl POST /api/v1/orders/{id}/sandbox-workspaces` 联调，回查 `delivery.sensitive_execution_policy.policy_snapshot`、`delivery.delivery_record.trust_boundary_snapshot` 与 `audit.audit_event` 后清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv016_sandbox_isolation_model_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`181 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/sandbox-workspaces`：`HTTP 200`
    - 返回 `isolation=container_sandbox`、`trusted_attestation=true`、`supported_product_types=SBX_STD`
    - 返回 `runtime_provider=gvisor / runtime_mode=local_placeholder / runtime_class=runsc / profile_name=sbx-std-default`
  - DB 回查通过：
    - `delivery.sensitive_execution_policy.policy_snapshot.execution_environment` 命中 `container_sandbox / gvisor / local_placeholder / runsc`
    - `delivery.delivery_record.trust_boundary_snapshot.sandbox_workspace.execution_environment` 命中 `container_sandbox / gvisor / local_placeholder`
    - `audit.audit_event` 命中 `delivery.sandbox.enable`，metadata 含 `runtime_provider=gvisor / runtime_mode=local_placeholder / runtime_class=runsc`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4.3（沙箱 / 模板查询类交付）
  - `页面说明书-V1` 7.5（查询沙箱开通页）
  - `数据商品查询与执行面设计` 5（V1 业务闭环）
  - `全集成基线-V1` 10.7（执行环境实体）与 `7.1 / 7.4` 中 `gVisor` 优先评估口径
- 覆盖的任务清单条目：`DLV-016`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。

### BATCH-159（计划中）
- 已阅读证据：
  1. `docs/开发任务/v1-core-开发任务清单.csv`：确认 `DLV-017` 目标为“实现报告交付接口 `POST /api/v1/orders/{id}/deliver` 的报告分支，生成 report artifact、报告 hash、交付回执”。
  2. `docs/开发任务/v1-core-开发任务清单.md`：确认本任务属于现有 `/deliver` 主交付入口的报告类扩展，要求补齐接口、DTO、权限、审计、错误码与最小测试，不允许与 OpenAPI 漂移。
  3. `docs/开发任务/Agent-开发与半人工审核流程.md` / `docs/开发任务/AI-Agent-执行提示词.md`：确认本批继续按单任务流程执行，完整验证、本地提交后自动推进下一任务。
  4. `docs/业务流程/业务流程图-V1-完整版.md:L388`：确认报告类交付链路要求卖方上传报告/结果产物与摘要，买方后续对照模板执行验收或驳回/争议。
  5. `docs/页面说明书/页面说明书-V1-完整版.md:L701`：确认报告/结果产品交付页必须呈现报告文件、摘要、版本记录与买方反馈入口，因此交付结果需要稳定回传 artifact 元信息与版本号。
  6. `docs/领域模型/全量领域模型与对象关系说明.md:L709`：确认 `delivery.report_artifact` 属于交付聚合，用于承载报告结果产物，并与 `delivery.storage_object` 形成对象关联。
  7. `docs/数据库设计/V1/upgrade/030_trade_delivery.sql` / `docs/全集成文档/数据交易平台-全集成基线-V1.md:L24147` / `L29221`：确认 `delivery.report_artifact` 的冻结字段为 `order_id/object_id/report_type/version_no/status`，本批应基于既有表落地，不新增 migration。
  8. 其余 18 份必读冻结文档已按阶段基线复核服务边界、事件、错误码、环境与配置口径；当前未发现需要升级为问题清单的冲突。
- 当前批次目标：在 `POST /api/v1/orders/{id}/deliver` 中新增 `report` 分支，联动 `delivery.storage_object + delivery.delivery_record + delivery.report_artifact + trade.order_main` 完成报告交付、报告摘要/版本持久化、交付回执返回、审计留痕，并完成真实 API/DB/MinIO 联调验证。
- 预计涉及文件：
  - `apps/platform-core/src/modules/delivery/api/**`
  - `apps/platform-core/src/modules/delivery/dto/**`
  - `apps/platform-core/src/modules/delivery/repo/**`
  - `apps/platform-core/src/modules/delivery/tests/**`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 预计验证方式：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  5. `./scripts/check-query-compile.sh`
  6. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv017_report_delivery_db_smoke -- --nocapture`
  7. 启动服务并使用真实 PostgreSQL + MinIO + `curl POST /api/v1/orders/{id}/deliver`（`branch=report`）联调，回查 `delivery.report_artifact / delivery.delivery_record / audit.audit_event` 后清理业务数据。

### BATCH-159（待审批）
- 状态：待审批
- 当前任务编号：DLV-017
- 已实现功能：
  1. 在 `POST /api/v1/orders/{id}/deliver` 中新增 `branch=report`，补齐独立权限 `delivery.report.commit`。
  2. 新增 `report_delivery_repository.rs`，联动 `delivery.storage_object + delivery.delivery_record + delivery.report_artifact + trade.order_main` 落地报告交付。
  3. 报告交付会冻结 `report_type / version_no / report_hash / receipt_hash / delivery_commit_hash`，并把对象定位写入 `trust_boundary_snapshot.report_artifact`。
  4. 交付成功后将 `RPT_STD` 订单推进到 `report_delivered`，子状态更新为 `delivery_status=delivered / acceptance_status=in_progress / settlement_status=pending_settlement`。
  5. 补齐幂等返回：订单已进入 `report_delivered/accepted/settled/closed` 且已有已提交报告产物时，重复提交返回 `already_committed`。
  6. 新增 `dlv017_report_delivery_db_smoke`，覆盖首次交付、重复提交、订单详情聚合与审计断言。
  7. 同步更新 `packages/openapi/delivery.yaml`，冻结 `report` 分支请求/响应 schema。
- 涉及文件：
  - `apps/platform-core/src/modules/delivery/api/handlers.rs`
  - `apps/platform-core/src/modules/delivery/api/support.rs`
  - `apps/platform-core/src/modules/delivery/dto/file_delivery_commit.rs`
  - `apps/platform-core/src/modules/delivery/repo/file_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/api_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/report_delivery_repository.rs`
  - `apps/platform-core/src/modules/delivery/repo/mod.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv017_report_delivery_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/mod.rs`
  - `packages/openapi/delivery.yaml`
  - `docs/开发任务/V1-Core-实施进度日志-P2.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv017_report_delivery_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动服务：`APP_PORT=8110 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`
  8. 使用真实 MinIO `report-results` bucket 上传测试报告对象后，执行 `curl POST /api/v1/orders/{id}/deliver`（`branch=report`）联调，并回查 `trade.order_main / delivery.delivery_record / delivery.report_artifact / audit.audit_event`，再清理业务数据。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo check -p platform-core`：通过。
  - `dlv017_report_delivery_db_smoke`：通过。
  - `cargo test -p platform-core`：通过（`182 passed, 0 failed, 1 ignored`）。
  - `cargo sqlx prepare --workspace`：通过。
  - `./scripts/check-query-compile.sh`：通过。
  - 真实 API 联调通过：
    - `POST /api/v1/orders/{id}/deliver`（`branch=report`）返回 `HTTP 200`
    - 返回 `current_state=report_delivered / delivery_status=delivered / acceptance_status=in_progress`
    - 返回 `report_artifact_id`、`report_type=pdf_report`、`report_version_no=1`、`report_hash=sha256:report:*`
    - 返回 `bucket=report-results / key=orders/<suffix>/monthly-report.pdf`
  - DB 回查通过：
    - `trade.order_main = report_delivered / paid / delivered / in_progress / pending_settlement`
    - `delivery.delivery_record = committed / report_delivery / result_package`
    - `delivery.report_artifact = pdf_report / version_no=1 / delivered`
    - `audit.audit_event` 命中 `delivery.report.commit = 1`
  - 清理结果：临时业务数据已删除；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `业务流程图-V1` 4.4（交付、验真与验收主流程）与 4.4.4（报告类交付）
  - `页面说明书-V1` 7.8（报告 / 结果产品交付页）
  - `全量领域模型与对象关系说明` 4.5（交付与执行聚合）
  - `全集成基线-V1` 15（核心交易链路完整闭环）
- 覆盖的任务清单条目：`DLV-017`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
