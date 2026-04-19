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
