### BATCH-174（计划中）
- 任务：BIL-001 Payment Jurisdiction / Corridor / Payout Preference 基础模型与接口占位
- 状态：计划中
- 说明：对历史偏移阶段中遗留的 Billing 基础接口做一致性复核与补齐，收敛到冻结支付域协议：补齐 `POST /api/v1/payment-jurisdictions`、`POST /api/v1/payment-corridors`、`POST /api/v1/payout-preferences` 以及建议读取接口，改为真实读取/写入 `payment.jurisdiction_profile`、`payment.corridor_policy`、`payment.payout_preference`，并补齐权限、step-up 占位、高风险审计、OpenAPI 与 DB/API 验证。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，进入 BIL 阶段后先对历史已实现口径做冻结文档一致性复核。
### BATCH-174（待审批）
- 任务：`BIL-001` Payment Jurisdiction / Corridor / Payout Preference 基础模型与接口占位
- 状态：待审批
- 当前任务编号：`BIL-001`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：定位 `BIL-001` 描述、DoD、依赖与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：复核 `BIL-001` 详细解释，确认不是静态占位，而是支付域基础控制面最小可用接口。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”执行。
  - `docs/开发任务/AI-Agent-执行提示词.md`：确认单任务批次、冻结流程与不可跳步约束。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：记录本批计划中与待审批留痕。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：同步本批追溯记录。
  - `docs/开发任务/V1-Core-人工审批记录.md`：按约定只读，不写入。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `POST /api/v1/payment-jurisdictions`、`POST /api/v1/payment-corridors`、`POST /api/v1/payout-preferences` 及 `payment.jurisdiction.manage / payment.corridor.manage / payment.payout_preference.manage` 权限口径。
  - `docs/开发准备/服务清单与服务边界正式版.md`：确认 Billing 仍以 PostgreSQL 为业务真值，支付 provider 为外部依赖，Kafka 为 outbox 事件边界。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认支付域接口冻结口径。
  - `docs/开发准备/事件模型与Topic清单正式版.md`：确认 Billing 阶段事件边界与 `billing.events` 主题用途。
  - `docs/开发准备/统一错误码字典正式版.md`：沿用权限/鉴权/冲突错误口径。
  - `docs/开发准备/测试用例矩阵正式版.md`：对齐本批 DB/API/审计回归覆盖。
  - `docs/开发准备/本地开发环境与中间件部署清单.md`：确认本地 `mock-payment-provider`、Kafka、PostgreSQL 联调环境。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实 7.1/7.2/7.3 的对象结构与 9.1/9.2/9.3/9.9 的接口口径。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：确认司法辖区、走廊策略、受益人出金偏好属于 Billing 基础控制面。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`：核对 `payment.jurisdiction_profile / payment.corridor_policy / payment.payout_preference / payment.provider_account` 表结构。
  - `docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`：确认 `audit.audit_event.ref_id` 为 UUID，辖区类动作需走无 `ref_id` 审计写入路径。
  - 其余必读冻结文档已按本批流程复核，未发现与当前实现冲突的新口径。
- 实现要点：
  - Billing 路由从历史偏移的 `/api/v1/billing/policies` 收敛到冻结支付域协议：新增 `GET/POST /api/v1/payment-jurisdictions`、`GET/POST /api/v1/payment-corridors`、`GET/POST /api/v1/payout-preferences`。
  - `apps/platform-core/src/modules/billing/domain/mod.rs` 补齐冻结字段：`policy_snapshot`、`product_scope`、`effective_from/effective_to`、`beneficiary_snapshot`。
  - 新增 `billing/repo/policy_repository.rs`，改为真实读写 `payment.jurisdiction_profile / payment.corridor_policy / payment.payout_preference`，不再返回静态占位列表。
  - 新增 `billing/policy_handlers.rs`，补齐控制面读写接口、租户范围校验、step-up 占位校验与审计写入。
  - `billing/service.rs` 补齐 `JurisdictionRead/Manage`、`CorridorRead/Manage`、`PayoutPreferenceRead/Manage` 权限矩阵。
  - `billing/db.rs` 新增 `write_audit_event_without_ref(...)`，用于 `jurisdiction` 这类非 UUID 主键控制面动作。
  - `packages/openapi/billing.yaml` 已同步到本批接口与 schema 口径。
  - 清理已被真实文件替代的 `.gitkeep`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil001_payment_policy_db_smoke -- --nocapture`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. `curl http://127.0.0.1:8089/__admin/` 确认 `mock-payment-provider` 可达
  8. 启动 `APP_PORT=8095` 的 `platform-core`，执行真实 API 联调：
     - `POST /api/v1/payment-jurisdictions`
     - `GET /api/v1/payment-jurisdictions`
     - `POST /api/v1/payment-corridors`
     - `GET /api/v1/payment-corridors`
     - `POST /api/v1/payout-preferences`
     - `GET /api/v1/payout-preferences`
     并用 `psql` 回查表与审计，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过。
  - `bil001_payment_policy_db_smoke` 通过，验证了真实路由、真实数据库写入和审计落库。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - `mock-payment-provider` `__admin` 返回 `HTTP 200`，本地 BIL 阶段外围依赖可达。
  - 真实 API 联调通过：
    - `payment-jurisdictions` 返回 `jurisdiction=SG / price_currency=USD`
    - `payment-corridors` 返回唯一 `corridor_policy_id`，`product_scope=general`
    - `payout-preferences` 返回唯一 `payout_preference_id`，`preferred_provider_key=offline_bank / is_default=true`
  - DB 回查通过：`payment.jurisdiction_profile.policy_snapshot.price_currency=USD`、`payment.corridor_policy.product_scope=general`、`payment.payout_preference.preferred_provider_key=offline_bank / is_default=true`。
  - 审计回查通过：`payment.jurisdiction.manage=1`、`payment.corridor.manage=1`、`payment.payout_preference.manage=1`、`payment.payout_preference.read=1`。
  - 清理回查通过：`payment.payout_preference=0 / payment.provider_account=0 / payment.corridor_policy=0 / core.organization=0`；审计按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付域接口协议正式版.md`：7.1/7.2/7.3、9.1/9.2/9.3/9.9
  - `数据交易平台-全集成基线-V1.md`：支付基础控制面与对应权限条目
  - `支付、资金流与轻结算设计.md`：司法辖区/走廊/出金偏好基础模型
- 覆盖的任务清单条目：`BIL-001`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
