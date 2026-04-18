### BATCH-116（计划中）
- 状态：计划中
- 当前任务编号：TRADE-007
- 当前批次目标：实现并持久化订单主状态机字段映射：`current_state`、`payment_status`、`delivery_status`、`acceptance_status`、`settlement_status`、`dispute_status`，并完成 API 联调验证与审计留痕。
- 前置依赖核对结果：`CORE-014; DB-006; IAM-001; CAT-001` 已完成且审批通过；上一批 `TRADE-006` 已审批通过。
- 备注：从本批起实施日志写入 `V1-Core-实施进度日志-P2.md`。

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
