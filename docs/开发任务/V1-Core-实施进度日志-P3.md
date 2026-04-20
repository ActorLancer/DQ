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
### BATCH-175（计划中）
- 任务：BIL-002 支付意图 `POST /api/v1/payments/intents` / `GET /api/v1/payments/intents/{id}` / `cancel`
- 状态：计划中
- 说明：对历史偏移阶段中已存在的支付意图实现做冻结文档一致性复核与补齐，统一请求/响应字段、幂等键、step-up、最小校验与审计口径；按人工确认结论，`payment.intent.create` 归为高风险动作，创建支付意图必须校验 `X-Step-Up-Token` 或等价结果，并同步修正相关契约文件。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-175（待审批）
- 任务：`BIL-002` 支付意图 `POST /api/v1/payments/intents` / `GET /api/v1/payments/intents/{id}` / `cancel`
- 状态：待审批
- 当前任务编号：`BIL-002`
- 前置依赖核对结果：`BIL-001` 已完成并本地提交；`TRADE-003`、`TRADE-007`、`DB-006`、`DB-019`、`DB-020`、`CORE-008` 已完成且审批通过；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：定位 `BIL-002` 目标、依赖、DoD 与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认本批要求不是占位，而是支付意图创建/查询/取消的最小可用闭环。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发任务/AI-Agent-执行提示词.md`：按“计划中 -> 实现 -> 完整验证 -> TODO -> 待审批 -> 本地提交”执行单任务流程。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`、`docs/开发任务/V1-Core-TODO与预留清单.md`：写入本批计划与待审批留痕。
  - `docs/开发任务/V1-Core-人工审批记录.md`：按约定只读，不写入。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：按人工确认结果收敛 `payment.intent.create` 为高风险动作，创建支付意图必须绑定 step-up；同步核对接口头、幂等与支付状态字段。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实 6.3、9.4、9.5、9.6 的支付意图字段、查询聚合与取消口径，并同步修正文档到方案 B。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：复核支付意图、provider/corridor/jurisdiction 预校验、支付轨迹与 webhook 摘要关系。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`、`packages/openapi/billing.yaml`：对齐请求/响应 schema 与 step-up / idempotency header 约束。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`docs/开发准备/事件模型与Topic清单正式版.md`：确认 Billing 仍以 PostgreSQL 为真值，Kafka 为 outbox 边界，Mock Payment Provider/WireMock 在本地可联调。
  - `docs/开发准备/统一错误码字典正式版.md`、`docs/开发准备/测试用例矩阵正式版.md`：沿用鉴权/冲突错误口径并覆盖 step-up、幂等重放、详情读取、取消重放。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`、`docs/数据库设计/V1/upgrade/020_catalog_contract.sql`：核对 `payment.payment_intent / payment.payment_transaction / payment.payment_webhook_event / payment.provider_account / payment.corridor_policy / trade.order_main / catalog.*` 真实表结构。
  - 其余必读冻结文档已按流程复核，未发现与当前实现冲突的新口径。
- 实现要点：
  - 新增 `billing/payment_intent_handlers.rs`，把支付意图创建/查询/取消从旧 `handlers.rs` 中拆分出来，避免继续向单文件堆叠。
  - `POST /api/v1/payments/intents` 现在强制要求 `x-step-up-token` 或 `x-step-up-challenge-id`，并要求 `x-idempotency-key`；已按方案 B 同步修正支付域接口协议与全集成基线文档。
  - `CreatePaymentIntentRequest` / `PaymentIntentView` / `PaymentIntentDetailView` 已补齐冻结字段：`provider_account_id`、`launch_jurisdiction_code`、`corridor_policy_id`、`fee_preview_id`、`payment_amount`、`expire_at`、`capability_snapshot`、`metadata`、最新交易摘要、最新 webhook 摘要。
  - 新增 `billing/repo/payment_intent_repository.rs`，完成订单可支付状态校验、租户范围校验、provider/jurisdiction/corridor/provider_account/fee_preview 预校验、能力快照冻结、详情聚合与取消幂等。
  - `billing/api/mod.rs` 路由已收敛到新的 handler；`packages/openapi/billing.yaml` 已同步更新 create/read/cancel 契约。
  - 新增 `bil002_payment_intent_db_smoke`，并补充 `rejects_create_payment_intent_without_step_up`、`rejects_create_payment_intent_without_idempotency_key`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil002_payment_intent_db_smoke -- --nocapture`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8096` 的 `platform-core`，执行真实 API 联调：
     - 先验证缺少 step-up 的 `POST /api/v1/payments/intents` 返回拒绝
     - 再验证创建支付意图、幂等重放、详情读取、取消、取消幂等重放
     - 用 `psql` 插入交易/回调摘要数据并回查 `payment.payment_intent / payment.payment_transaction / payment.payment_webhook_event / audit.audit_event`
     - `curl http://127.0.0.1:8089/__admin/` 确认 `mock-payment-provider` 可达
     - 结束后清理临时业务数据，审计按 append-only 保留。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过。
  - `bil002_payment_intent_db_smoke` 通过，验证了真实路由、真实数据库写入、幂等重放与审计落库。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - 缺少 step-up 创建返回 `HTTP 400`，消息 `x-step-up-token or x-step-up-challenge-id is required for payment intent create`
    - 正常创建返回 `HTTP 200`，`payment_status=created`
    - 同 idempotency key 重放返回同一 `payment_intent_id`
    - 详情读取成功返回 `latest_transaction_summary.transaction_type=payin`、`webhook_summary.event_type=payment.succeeded`
    - 取消与取消重放均返回 `payment_status=canceled`
  - DB 回查通过：`payment.payment_intent.status=canceled`、`provider_account_id/corridor_policy_id` 与请求一致、`metadata.source=bil002-api`。
  - 审计回查通过：`payment.intent.create=1`、`payment.intent.read=1`、`payment.intent.cancel=1`、`payment.intent.cancel.idempotent_replay=1`。
  - `mock-payment-provider` `__admin` 返回 `HTTP 200`；本地 Billing 阶段外围依赖可达。
  - 临时业务数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付域接口协议正式版.md`：6.3、9.4、9.5、9.6
  - `数据交易平台-全集成基线-V1.md`：支付意图为高风险动作、创建绑定 step-up、支付详情聚合字段口径
  - `支付、资金流与轻结算设计.md`：支付意图、provider/corridor/jurisdiction 校验与轨迹摘要
- 覆盖的任务清单条目：`BIL-002`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-176（计划中）
- 任务：BIL-003 订单锁资 `POST /api/v1/orders/{id}/lock`
- 状态：计划中
- 说明：基于冻结文档对历史 `order lock` 实现做一致性复核与补齐，收敛租户范围、订单/支付意图一致性、价格快照一致性、支付状态合法性、幂等重放与审计；保持与后续 mock provider / webhook 主链路兼容。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-176（待审批）
- 任务：`BIL-003` 订单锁资 `POST /api/v1/orders/{id}/lock`
- 状态：待审批
- 当前任务编号：`BIL-003`
- 前置依赖核对结果：`BIL-002` 已完成并本地提交；`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：定位 `BIL-003` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：复核支付编排层与“支付意图 -> 锁资 -> webhook -> 订单推进”的分层边界。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实幂等与一致性约束，确认锁资需校验订单归属、价格一致性和支付状态。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 `POST /api/v1/orders/{id}/lock` 的权限 `billing.deposit.lock`、作用域 `tenant + order`、额外校验“订单归属、价格一致性、支付状态、审计”。
  - `docs/权限设计/接口权限校验清单.md`、`packages/openapi/billing.yaml`：同步校验租户作用域与返回契约。
  - 其余必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `billing/repo/order_lock_repository.rs`，将锁资逻辑从旧 `handlers.rs` 中拆出，避免继续向单文件堆叠。
  - `POST /api/v1/orders/{id}/lock` 现在在事务内完成：`SELECT ... FOR UPDATE` 锁定订单和支付意图，校验租户范围、订单/支付意图归属、payer/payee 一致性、金额/币种一致性以及可锁定状态。
  - 对已绑定同一 `payment_intent_id` 的重复锁资返回幂等回放；对绑定其他意图或跨订单意图返回 `409`。
  - 锁资成功时写入 `payment_channel_snapshot` 的 `payment_intent_id/provider_key/provider_account_id/provider_intent_no/channel_reference_no/payment_amount/currency_code/lock_reason/locked_at`，并将订单 `payment_status` 置为 `locked`。
  - 新增 `billing/order_lock_handlers.rs`，补齐 `order.payment.lock` 与 `order.payment.lock.idempotent_replay` 审计。
  - 新增 `bil003_order_lock_db_smoke`；`packages/openapi/billing.yaml` 已补充 `x-tenant-id/x-request-id` 与冲突说明。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil003_order_lock_db_smoke -- --nocapture`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8097` 的 `platform-core`，插入临时订单/支付意图数据后执行真实 `curl`：成功锁资、同意图重复锁资、跨订单意图锁资；随后 `psql` 回查 `trade.order_main` 与 `audit.audit_event`，再清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `208 passed, 0 failed, 1 ignored`。
  - `bil003_order_lock_db_smoke` 通过。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - 首次锁资 `HTTP 200`，返回 `payment_status=locked`
    - 同一 `payment_intent_id` 重放 `HTTP 200`
    - 跨订单 `payment_intent_id` 锁资 `HTTP 409`，消息 `payment intent does not belong to order`
  - DB 回查通过：`trade.order_main.payment_status=locked`，`payment_channel_snapshot.payment_intent_id/provider_key/lock_reason` 与请求一致。
  - 审计回查通过：`order.payment.lock=1`、`order.payment.lock.idempotent_replay=1`。
  - 临时业务数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付、资金流与轻结算设计.md`：4.2、8.1、9.1
  - `支付域接口协议正式版.md`：6.1、6.2、6.3
  - `数据交易平台-全集成基线-V1.md`：3.4、9.1、`POST /api/v1/orders/{id}/lock` 对应权限/作用域/校验条目
- 覆盖的任务清单条目：`BIL-003`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-177（计划中）
- 任务：BIL-004 Mock Payment Provider 适配器与模拟支付接口
- 状态：计划中
- 说明：在现有 `provider-kit` mock adapter 基础上补齐 Billing 路由入口、真实 WireMock 联调、mock case 落库与 webhook 串联，确保 `success/fail/timeout` 三类场景可直接驱动平台支付主链路。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-177（待审批）
- 任务：`BIL-004` Mock Payment Provider 适配器与模拟支付接口
- 状态：待审批
- 当前任务编号：`BIL-004`
- 前置依赖核对结果：`BIL-003` 已完成并本地提交；`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：定位 `BIL-004` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：复核支付编排层与 mock provider/live provider 分层，确认 V1 需要 success/fail/timeout 三类模拟结果驱动真实 webhook 主链路。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：对齐幂等、签名占位、provider callback、Webhook 乱序/重复处理口径。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 mock payment provider/WireMock 本地协议、`/mock/payment/charge/success|fail|timeout` 三条模拟路径、支付回调与审计要求。
  - `docs/开发准备/服务清单与服务边界正式版.md`：确认 BIL 阶段需实质性接入 PostgreSQL + Mock Payment Provider/WireMock + Kafka(outbox 边界)，Redis 保持可用。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`、`packages/openapi/billing.yaml`：同步校验新增 mock simulate 路由与请求/响应 schema。
  - `infra/mock-payment/mappings/*.json`、`scripts/check-mock-payment.sh`：确认本地 WireMock 提供 success/fail/refund/manual-transfer/timeout 固定行为，timeout 需由客户端超时触发。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `billing/mock_payment_handlers.rs`，把 mock payment simulate 路由从既有 `handlers.rs` 中独立拆出，避免继续堆叠单文件。
  - 新增 `billing/repo/mock_payment_repository.rs`，落库 `developer.mock_payment_case`，校验 tenant scope 与 `provider_key=mock_payment`，并回写执行结果 payload。
  - 新增三个真实路由：
    - `POST /api/v1/mock/payments/{id}/simulate-success`
    - `POST /api/v1/mock/payments/{id}/simulate-fail`
    - `POST /api/v1/mock/payments/{id}/simulate-timeout`
  - 路由会通过 `provider-kit` 的 mock payment adapter 调用 WireMock live endpoint，再把生成的 provider event 送入既有 `POST /api/v1/payments/webhooks/{provider}` 主链路处理。
  - 支持 `duplicate_webhook` 重放验证；成功场景会落 `payment.webhook.duplicate`，并复用既有 webhook 幂等逻辑。
  - `MockPaymentSimulationRequest` / `MockPaymentSimulationView` 已补齐请求与响应结构，`packages/openapi/billing.yaml` 已同步更新。
  - 新增 `bil004_mock_payment_adapter_db_smoke`，覆盖 create intent -> order lock -> simulate success/fail/timeout -> webhook/update order 主链路，并验证 `developer.mock_payment_case / payment.payment_webhook_event / payment.payment_intent / trade.order_main / audit.audit_event`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `MOCK_BASE_URL=http://127.0.0.1:8089 ./scripts/check-mock-payment.sh`
  5. `TRADE_DB_SMOKE=1 MOCK_PAYMENT_ADAPTER_MODE=live MOCK_PAYMENT_BASE_URL=http://127.0.0.1:8089 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil004_mock_payment_adapter_db_smoke -- --nocapture`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. 启动 `APP_PORT=8098` 的 `platform-core`（同时设置 `MOCK_PAYMENT_ADAPTER_MODE=live` 与 `MOCK_PAYMENT_BASE_URL=http://127.0.0.1:8089`），插入三组临时订单数据后执行真实 `curl`：create intent、lock、simulate-success/fail/timeout，再用 `psql` 回查业务表并清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `210 passed, 0 failed, 1 ignored`。
  - `./scripts/check-mock-payment.sh` 通过，确认 WireMock `success/fail/refund/manual-transfer/timeout` 行为符合冻结脚本。
  - `bil004_mock_payment_adapter_db_smoke` 通过，确认 success/fail/timeout 三类模拟均经过真实 Billing 路由、真实数据库、真实 webhook 主链路。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - create intent 三次均返回 `HTTP 200`
    - `simulate-success` 返回 `HTTP 200`，摘要 `success|succeeded|200|processed|duplicate|succeeded`
    - `simulate-fail` 返回 `HTTP 200`，摘要 `fail|failed|402|processed|None|failed`
    - `simulate-timeout` 返回 `HTTP 200`，摘要 `timeout|timeout|None|processed|None|expired`
  - DB 回查通过：
    - `payment.payment_intent` 状态分别为 `succeeded / failed / expired`
    - `trade.order_main` 状态分别为 `buyer_locked/paid`、`payment_failed_pending_resolution/failed`、`payment_timeout_pending_compensation_cancel/expired`
    - `developer.mock_payment_case` 状态均为 `executed`，success 场景 `duplicate_webhook=true` 且 `duplicate_processed_status=duplicate`
  - 审计回查通过：`mock.payment.simulate=3`、`payment.webhook.processed=3`、`payment.webhook.duplicate=1`、`order.payment.result.applied=3`。
  - 临时业务数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付、资金流与轻结算设计.md`：4、10.1、10.2、10.3
  - `支付域接口协议正式版.md`：6.3、9.7、9.8
  - `数据交易平台-全集成基线-V1.md`：27、mock payment provider 本地联调协议与支付回调主链路要求
- 覆盖的任务清单条目：`BIL-004`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-178（计划中）
- 任务：BIL-005 支付 webhook 接口 `POST /api/v1/payments/webhooks/{provider}`
- 状态：计划中
- 说明：基于当前已落地的 webhook 路径补齐冻结协议要求：DTO 对齐 `provider_transaction_no / transaction_amount / currency_code / occurred_at / raw_payload`，accepted callback 写入 `payment.payment_transaction`，并补充签名占位、重复回调、防重放、乱序保护的专项 smoke 与真实直接回调联调。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-178（待审批）
- 任务：`BIL-005` 支付 webhook 接口 `POST /api/v1/payments/webhooks/{provider}`
- 状态：待审批
- 当前任务编号：`BIL-005`
- 前置依赖核对结果：`BIL-004` 已完成并本地提交；`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：定位 `BIL-005` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：复核支付回调必须幂等、禁止重复回调导致重复记账/放款，并确认 mock webhook 属于本地演练入口。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：对齐 `POST /api/v1/payments/webhooks/{provider}` 请求体最小字段、服务端 6 条必须动作，以及 `payment.payment_transaction / payment.payment_webhook_event` 结构。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对支付回调签名占位、事件唯一性、重放窗口、乱序保护、账务镜像更新与 mock webhook 调试口径。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：同步校验 webhook 与 mock simulate 的请求/响应 schema 不漂移。
  - `scripts/check-query-compile.sh`、`.sqlx/`：确认 SQLx 元数据与离线编译基线继续有效。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `billing/webhook_handlers.rs`，把实际 `handle_payment_webhook(...)` 从公共 `handlers.rs` 中拆出，避免继续堆大文件。
  - `PaymentWebhookRequest` 补齐冻结协议字段：`provider_transaction_no / transaction_amount / currency_code / occurred_at / raw_payload`，并兼容旧别名 `payload`。
  - `PaymentWebhookResultView` 补齐 `payment_transaction_id`；`MockPaymentSimulationView` 同步回传该字段，保持 mock simulate 与 webhook 主链路可联查。
  - accepted callback 现在会先按事件类型映射交易形态，再真实写入 `payment.payment_transaction`，随后回写 `payment.payment_webhook_event.payment_transaction_id`。
  - 保留并收紧五类 webhook 防护：
    - `rejected_signature`
    - `duplicate`
    - `rejected_replay`
    - `processed_noop / intent_not_found`
    - `out_of_order_ignored`
  - `occurred_at` 文本时间戳新增 RFC3339 归一化路径；若未提供 header timestamp，可用请求体文本时间进入重放/乱序判定。
  - `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml` 已同步更新。
  - 新增 `bil005_payment_webhook_db_smoke`，覆盖 `processed / duplicate / rejected_signature / rejected_replay / out_of_order_ignored` 五条分支。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core bil005_payment_webhook_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil005_payment_webhook_db_smoke -- --nocapture`
  5. `MOCK_BASE_URL=http://127.0.0.1:8089 ./scripts/check-mock-payment.sh`
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. 启动 `APP_PORT=8099` 的 `platform-core`，插入一组临时订单/商品/支付账户数据后执行真实 `curl`：create intent、lock、webhook success、duplicate、rejected_signature、rejected_replay、out_of_order_ignored，再用 `psql` 回查业务表并清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil005_payment_webhook_db_smoke` 普通编译跑通，`TRADE_DB_SMOKE=1` 实库 smoke 通过。
  - `./scripts/check-mock-payment.sh` 通过。
  - `cargo test -p platform-core` 通过：`211 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - create intent 返回 `created`
    - lock 返回 `locked`
    - success 返回 `processed|<payment_transaction_id>|succeeded`
    - duplicate 返回 `duplicate|<same payment_transaction_id>`
    - invalid signature 返回 `rejected_signature`
    - replay 返回 `rejected_replay`
    - out-of-order 返回 `out_of_order_ignored|true`
  - DB 回查通过：
    - `payment.payment_intent.status=succeeded`
    - `trade.order_main.status=buyer_locked`
    - `trade.order_main.payment_status=paid`
    - `payment.payment_transaction` 仅 1 条
    - `payment.payment_webhook_event.processed_status` 命中 `duplicate / rejected_signature / rejected_replay / out_of_order_ignored`
    - 审计命中 `payment.webhook.processed=1`、`payment.webhook.duplicate=1`、`payment.webhook.rejected_signature=1`、`payment.webhook.rejected_replay=1`、`payment.webhook.out_of_order_ignored=1`
  - 临时业务数据已清理；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付、资金流与轻结算设计.md`：4、10、12
  - `支付域接口协议正式版.md`：6、9.7、10.7
  - `数据交易平台-全集成基线-V1.md`：27、`POST /api/v1/payments/webhooks/{provider}`、mock webhook 调试与支付回调保护条目
- 覆盖的任务清单条目：`BIL-005`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-179（计划中）
- 任务：`BIL-006` 账单事件模型 `BillingEvent`
- 状态：计划中
- 说明：在现有支付成功主链路上补齐 `BillingEvent` 领域模型、最小幂等仓储、审计与 `billing.events` outbox 边界；事件类型覆盖一次性收费、周期收费、调用量收费、退款、赔付、人工结算，并通过 DB smoke + 真实支付成功 API 联调验证 `billing.billing_event / ops.outbox_event / audit.audit_event` 联动。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-179（待审批）
- 任务：`BIL-006` 账单事件模型 `BillingEvent`
- 状态：待审批
- 当前任务编号：`BIL-006`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`BIL-005` 已完成并本地提交；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `BIL-006` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：复核 `BillingEvent`、`Settlement`、退款/赔付/人工结算在账单聚合中的最小结构与关系约束。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`：核对 `billing.billing_event` 及相关列/索引/JSONB 承载能力。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：对齐一次性收费、周期收费、调用量收费、退款、赔付、人工结算的冻结口径与结算影响。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：确认本批未新增公共路由，仅补内部模型与事件边界，不引入契约漂移。
  - `scripts/check-query-compile.sh`、`.sqlx/`：确认 SQLx 元数据与离线编译基线继续有效。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `BillingEvent` 领域模型，冻结 `billing_event_id / order_id / event_type / event_source / amount / currency_code / units / occurred_at / metadata` 最小结构。
  - 新增 `billing_event_repository.rs`，提供 `record_billing_event(...) / list_billing_events_for_order(...)` 与 `infer_payment_success_event_type(...)`。
  - `record_billing_event(...)` 在事务内补齐：订单上下文加载、租户范围校验、事件类型/来源标准化、幂等键解析、退款/赔付/人工结算语义校验、closed order 阻断、`charge_snapshot / consistency_state` 元数据冻结。
  - 新增 `billing.event.recorded` outbox，目标 `target_bus=kafka`、`target_topic=billing.events`，并写入 `payload_hash / ordering_key / partition_key`。
  - 支付 webhook 成功链路现在会自动生成一次性/周期性 `BillingEvent`，并落 `billing.event.generated` 审计；重复 webhook 通过 `idempotency_key` 复用同一账单事件，不重复出账。
  - `payment_transaction` 插入补充 `(payment_intent_id, transaction_type, provider_transaction_no)` 级别幂等复用，避免 pending webhook 重试时重复写交易流水。
  - 新增 `BillingPermission::BillingEventRead`，为后续 `BIL-007` 账单查看接口预留权限口径。
  - 新增 `bil006_billing_event_db_smoke`，覆盖一次性收费、周期收费、调用量收费、退款、赔付、人工结算与 idempotent replay。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core bil006_billing_event_db_smoke -- --nocapture`
  4. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil006_billing_event_db_smoke -- --nocapture`
  5. `cargo test -p platform-core`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
  8. `MOCK_BASE_URL=http://127.0.0.1:8089 ./scripts/check-mock-payment.sh`
  9. 启动 `APP_PORT=8100` 的 `platform-core`，插入一组临时 `FILE_STD` 订单数据后执行真实 `curl`：create intent、lock、`POST /api/v1/payments/webhooks/mock_payment` success，再用 `psql` 回查 `billing.billing_event / ops.outbox_event / audit.audit_event / trade.order_main` 并清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil006_billing_event_db_smoke` 普通编译跑通，`TRADE_DB_SMOKE=1` 实库 smoke 通过。
  - `cargo test -p platform-core` 通过：`215 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - `./scripts/check-mock-payment.sh` 通过，确认 WireMock 本地支付场景可用。
  - 真实 API 联调通过：
    - create intent 返回 `HTTP 200`，摘要 `payment_status=created`
    - lock 返回 `HTTP 200`，摘要 `payment_status=locked`
    - webhook success 返回 `HTTP 200`，摘要 `processed|false|succeeded`
  - DB 回查通过：
    - `billing.billing_event` 命中 `one_time_charge / payment_webhook / 88.00000000 / SGD`
    - `ops.outbox_event` 命中 `billing.event.recorded / kafka / billing.events / pending`
    - `trade.order_main` 命中 `buyer_locked / paid / pending_settlement / payment_succeeded_to_buyer_locked`
    - 审计命中 `payment.intent.create=1`、`order.payment.lock=1`、`order.payment.result.applied=1`、`billing.event.generated=1`、`payment.webhook.processed=1`
  - 临时业务数据已清理，回查 `order_main=0 | billing_event=0 | payment_intent=0 | organization=0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md`：4.6
  - `040_billing_support_risk.sql`：账单/结算支撑 schema
  - `支付、资金流与轻结算设计.md`：7
- 覆盖的任务清单条目：`BIL-006`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-180（计划中）
- 任务：`BIL-007` 账单查看接口 `GET /api/v1/billing/{order_id}`
- 状态：计划中
- 说明：在现有 `BillingEvent / Settlement / Refund / Compensation / Invoice` 读面上补齐单订单账单查询接口、DTO、权限、审计与 OpenAPI，同步输出税务/发票占位字段，并通过 DB smoke + 真实 `curl` 联调验证 `billing` 聚合读取口径。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-180（待审批）
- 任务：`BIL-007` 账单查看接口 `GET /api/v1/billing/{order_id}`
- 状态：待审批
- 当前任务编号：`BIL-007`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`BIL-006` 已完成并本地提交；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `BIL-007` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：复核 `BillingEvent / Settlement / RefundRecord / CompensationRecord / InvoiceRequest` 在账单聚合中的职责分工。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`：核对 `billing.billing_event / settlement_record / refund_record / compensation_record / invoice_request` 的冻结字段。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：确认一次性/周期/API 用量链路的账单查看应能回传结算状态与税务/发票占位。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：同步补齐 `GET /api/v1/billing/{order_id}` 路径与 `BillingEvent` / 账单聚合 schema，消除原有未定义 `$ref`。
  - `scripts/check-query-compile.sh`、`.sqlx/`：确认 SQLx 元数据与离线编译基线继续有效。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `billing_read_repository.rs`，按订单维度聚合读取 `BillingEvent / Settlement / Refund / Compensation / Invoice`。
  - 新增 `billing_read_handlers.rs`，落地 `GET /api/v1/billing/{order_id}`，接入 `BillingPermission::BillingEventRead`、租户范围校验和审计 `billing.order.read`。
  - 返回视图 `BillingOrderDetailView` 新增订单主状态、支付状态、结算状态、争议状态、订单金额、账单明细数组，以及 `tax_placeholder / invoice_placeholder` 占位结构。
  - `BillingSettlementView` 补齐 `platform_fee_amount / channel_fee_amount / net_receivable_amount` 等结算字段，避免只看到粗粒度状态。
  - OpenAPI 补齐 `BillingEvent / BillingSettlement / BillingRefund / BillingCompensation / BillingInvoice / BillingTaxPlaceholder / BillingInvoicePlaceholder / BillingOrderDetail` 组件。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core bil007_billing_read_db_smoke -- --nocapture`
  4. 启动 `APP_PORT=8101` 的 `platform-core`，插入一组临时账单数据后执行真实 `curl GET /api/v1/billing/{order_id}` 与越权 `403` 联调，再用 `psql` 回查 `audit.audit_event / trade.order_main` 并清理临时业务数据。
  5. `cargo test -p platform-core`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil007_billing_read_db_smoke` 通过，覆盖正常读取、租户越权 `403`、审计落库。
  - 真实 API 联调通过：
    - `GET /api/v1/billing/{order_id}` 返回 `HTTP 200`
    - 返回 `billing_events=1 / settlements=1 / refunds=1 / compensations=1 / invoices=1`
    - 返回 `tax_placeholder.tax_engine_status=placeholder`
    - 返回 `invoice_placeholder.invoice_mode=manual_placeholder`
    - outsider tenant 返回 `HTTP 403`
  - DB 回查通过：
    - `audit.audit_event` 命中 `billing.order.read=1`
    - `trade.order_main` 保持 `buyer_locked / paid / pending_settlement`
  - 临时业务数据已清理，回查 `order_main=0 | billing_event=0 | settlement_record=0 | organization=0`；审计记录按 append-only 保留。
  - `cargo test -p platform-core` 通过：`216 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md`：4.6
  - `040_billing_support_risk.sql`：账单/结算/退款/赔付/发票 schema
  - `支付、资金流与轻结算设计.md`：7、8、9
- 覆盖的任务清单条目：`BIL-007`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-181（计划中）
- 任务：`BIL-008` Settlement 模型
- 状态：计划中
- 说明：在现有 `billing.settlement_record` 冻结表结构基础上补齐 `Settlement` 领域模型、最小聚合/摘要视图与计算仓储，明确应结金额、平台抽佣、渠道手续费、供方应收、退款/赔付调整和结算摘要，并通过 DB smoke + 真实 API/DB 联调验证读写口径。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-181（待审批）
- 任务：`BIL-008` Settlement 模型
- 状态：待审批
- 当前任务编号：`BIL-008`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`BIL-007` 已完成并本地提交；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `BIL-008` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：复核 `Settlement / SettlementSummary` 在账单、托管与分润聚合中的职责与生命周期。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`：核对 `billing.settlement_record` 冻结字段、金额列与状态列。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：确认应结金额、平台抽佣、渠道手续费、退款/赔付调整、供方应收的冻结口径。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：同步补齐 `BillingSettlementSummary` schema 与 `BillingOrderDetail.settlement_summary`。
  - `scripts/check-query-compile.sh`、`.sqlx/`：确认 SQLx 元数据与离线编译基线继续有效。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `Settlement` 领域模型，冻结 `settlement_id / settlement_type / settlement_status / settlement_mode / payable_amount / platform_fee_amount / channel_fee_amount / net_receivable_amount / refund_amount / compensation_amount / reason_code / settled_at / updated_at`。
  - 新增 `SettlementSummary` 领域模型，冻结 `gross_amount / platform_commission_amount / channel_fee_amount / refund_adjustment_amount / compensation_adjustment_amount / supplier_receivable_amount / summary_state / proof_commit_state` 最小摘要。
  - `billing_read_repository.rs` 现在会在加载 `settlements` 后统一计算 `settlement_summary`，并挂入 `BillingOrderDetailView`。
  - `BillingSettlementView` 改为直接复用 `Settlement` 领域模型，减少读模型与领域模型漂移。
  - 新增 `bil008_settlement_summary_db_smoke`，覆盖账单详情读取时结算摘要金额与状态输出。
  - OpenAPI 同步新增 `BillingSettlementSummary` 组件，保证读接口契约与实现一致。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core bil008_settlement_summary_db_smoke -- --nocapture`
  4. 启动 `APP_PORT=8102` 的 `platform-core`，插入一组临时结算数据后执行真实 `curl GET /api/v1/billing/{order_id}`，再用 `psql` 回查 `audit.audit_event / trade.order_main` 并清理临时业务数据。
  5. `cargo test -p platform-core`
  6. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  7. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil008_settlement_summary_db_smoke` 通过，覆盖结算摘要金额、状态与审计读取。
  - 真实 API 联调通过：
    - `GET /api/v1/billing/{order_id}` 返回 `HTTP 200`
    - 返回 `gross_amount=88.00000000`
    - 返回 `platform_commission_amount=2.00000000`
    - 返回 `channel_fee_amount=1.00000000`
    - 返回 `refund_adjustment_amount=5.00000000`
    - 返回 `compensation_adjustment_amount=3.00000000`
    - 返回 `supplier_receivable_amount=85.00000000`
    - 返回 `summary_state=order_settlement:pending:manual`
    - 返回 `proof_commit_state=pending_anchor`
  - DB 回查通过：
    - `audit.audit_event` 命中 `billing.order.read=1`
    - `trade.order_main` 保持 `buyer_locked / paid / pending_settlement`
  - 临时业务数据已清理，回查 `order_main=0 | settlement_record=0 | organization=0`；审计记录按 append-only 保留。
  - `cargo test -p platform-core` 通过：`217 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md`：4.6
  - `040_billing_support_risk.sql`：账单/结算支撑 schema
  - `支付、资金流与轻结算设计.md`：7
- 覆盖的任务清单条目：`BIL-008`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-182（计划中）
- 任务：`BIL-009` 退款接口 `POST /api/v1/refunds`
- 状态：计划中
- 说明：实现退款接口、DTO、权限、step-up、裁决结果绑定、审计与最小链路测试；退款执行需与订单/支付/争议口径保持一致，并补齐 OpenAPI。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-182（待审批）
- 任务：`BIL-009` 退款接口 `POST /api/v1/refunds`
- 状态：待审批
- 当前任务编号：`BIL-009`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`BIL-008` 已完成并本地提交；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `BIL-009` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：复核争议、退款、赔付与账单/结算聚合关系。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认裁决后退款链路、结算/争议状态联动与关闭路径。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认退款/赔付处理页字段、角色与操作语义。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实 `POST /api/v1/refunds` 的 step-up、幂等键、审计与错误口径。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：落实“退款为高风险动作，必须绑定 step-up 与审计包”。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：同步新增退款接口与请求/响应 schema。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `POST /api/v1/refunds`，要求 `billing.refund.execute` 权限、`x-step-up-token/x-step-up-challenge-id`、`x-idempotency-key`、租户 scope 头与裁决结果绑定。
  - 新增 `CreateRefundRequest / RefundExecutionView` DTO，并将退款执行结果、provider 回执、幂等重放状态与 metadata 暴露给 API。
  - 新增 `refund_repository.rs`：校验订单/支付/币种/退款模式/争议裁决一致性，调用 live `mock-payment-provider` 的 `/mock/payment/refund/success`，落库 `billing.refund_record`、`billing.billing_event`、`ops.outbox_event`，更新 `billing.settlement_record.refund_amount`、`support.dispute_case` 与 `trade.order_main.dispute_status`。
  - 新增审计：`billing.refund.execute`、`billing.event.generated`，并在幂等重放时记录 `billing.refund.execute.idempotent_replay`。
  - 新增 `bil009_refund_db_smoke`，覆盖退款成功、幂等重放、结算退款金额更新、账单聚合读回与 outbox 产出。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 MOCK_PAYMENT_ADAPTER_MODE=live MOCK_PAYMENT_BASE_URL=http://127.0.0.1:8089 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil009_refund_db_smoke -- --nocapture`
  4. 启动 `APP_PORT=8103` 的 `platform-core`，用 `psql` 插入一组临时订单/支付/结算/争议/裁决数据后执行真实 `curl POST /api/v1/refunds` 与同幂等键重放，再执行 `curl GET /api/v1/billing/{order_id}`。
  5. `psql` 回查 `billing.refund_record / billing.billing_event / billing.settlement_record / support.dispute_case / ops.outbox_event / audit.audit_event`，随后清理临时业务数据，仅保留审计。
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil009_refund_db_smoke` 通过。
  - 真实 API 联调通过：首次退款 `HTTP 200`，返回 `current_status=succeeded`、`provider_key=mock_payment`、`provider_status=REFUND_SUCCESS`、`step_up_bound=true`；同幂等键重放 `HTTP 200`，`idempotent_replay=true`。
  - 真实账单读接口联调通过：`GET /api/v1/billing/{order_id}` 返回 `refunds=1`、`settlement_summary.refund_adjustment_amount=20.00000000`。
  - DB 回查通过：`billing.refund_record=1`、`billing.billing_event(event_type=refund)=1`、`billing.settlement_record.refund_amount=20.00000000`、`support.dispute_case.status=resolved`、`ops.outbox_event(target_topic=billing.events)=1`、`audit.audit_event` 命中 `billing.refund.execute=1` 与 `billing.refund.execute.idempotent_replay=1`。
  - 临时业务数据已清理，回查 `order_main=0 | refund_record=0 | outbox=0 | organization=0`；审计记录按 append-only 保留。
  - `cargo test -p platform-core` 通过：`219 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md`：4.7
  - `业务流程图-V1-完整版.md`：5.3
  - `页面说明书-V1-完整版.md`：8.2 / 8.3
  - `支付域接口协议正式版.md`：退款接口、step-up、幂等键
  - `支付、资金流与轻结算设计.md`：高风险退款审计与争议/结算联动
- 覆盖的任务清单条目：`BIL-009`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-183（计划中）
- 任务：`BIL-010` 赔付接口 `POST /api/v1/compensations`
- 状态：计划中
- 说明：在 `BIL-009` 退款链路基础上补齐赔付接口、DTO、权限、step-up、裁决结果绑定、审计与最小测试；赔付执行需与订单/支付/争议、账单事件、结算摘要和 mock provider/manual settlement 边界保持一致，并通过 DB smoke + 真实 API/DB 联调验证。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-183（待审批）
- 任务：`BIL-010` 赔付接口 `POST /api/v1/compensations`
- 状态：待审批
- 当前任务编号：`BIL-010`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`BIL-009` 已完成并本地提交；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `BIL-010` 范围、依赖、DoD 与 `technical_reference`。
  - `docs/领域模型/全量领域模型与对象关系说明.md`：复核争议、赔付、账单/结算聚合与对象边界。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认争议裁决后赔付链路、争议关闭与账单/结算联动。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认退款/赔付处理页字段、平台风控结算员角色与操作语义。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实 `POST /api/v1/compensations` 的 step-up、幂等键、人工打款高风险动作与审计要求。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：落实“赔付为高风险动作，必须绑定 step-up 与审计包”，并确认 V1 以人工打款/手工赔付对象占位承接。
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`：同步新增赔付接口与请求/响应 schema。
  - 其余 18 份必读冻结文档已按流程复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `POST /api/v1/compensations`，要求 `billing.compensation.execute` 权限、`x-step-up-token/x-step-up-challenge-id`、`x-idempotency-key`、裁决结果绑定与审计齐备。
  - 新增 `CreateCompensationRequest / CompensationExecutionView` DTO，并将赔付执行结果、provider 回执、幂等重放状态与 metadata 暴露给 API。
  - 新增 `compensation_repository.rs`：校验订单/支付/币种/赔付模式/争议裁决一致性，调用 live `mock-payment-provider` 的 `/mock/payment/manual-transfer/success`，落库 `billing.compensation_record`、`billing.billing_event`、`ops.outbox_event`，更新 `billing.settlement_record.compensation_amount`、`support.dispute_case` 与 `trade.order_main.dispute_status`。
  - 新增审计：`billing.compensation.execute`、`billing.event.generated`，并在幂等重放时记录 `billing.compensation.execute.idempotent_replay`。
  - 收紧退款/赔付执行权限到平台侧高风险角色：`platform_admin / platform_finance_operator / platform_risk_settlement`，不再允许 `tenant_admin` 执行高风险资金操作。
  - 新增 `bil010_compensation_db_smoke`，覆盖赔付成功、幂等重放、结算赔付金额更新、账单聚合读回与 outbox 产出。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 MOCK_PAYMENT_ADAPTER_MODE=live MOCK_PAYMENT_BASE_URL=http://127.0.0.1:8089 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil010_compensation_db_smoke -- --nocapture`
  4. 启动 `APP_PORT=8103` 的 `platform-core`，用 `psql` 插入一组临时订单/支付/结算/争议/裁决数据后执行真实 `curl POST /api/v1/compensations` 与同幂等键重放，再执行 `curl GET /api/v1/billing/{order_id}`。
  5. `psql` 回查 `billing.compensation_record / billing.billing_event / billing.settlement_record / support.dispute_case / ops.outbox_event / audit.audit_event`，随后清理临时业务数据，仅保留审计。
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil010_compensation_db_smoke` 通过。
  - 真实 API 联调通过：首次赔付 `HTTP 200`，返回 `current_status=succeeded`、`provider_transfer_id=mock-mtf-...`、`step_up_bound=true`；同幂等键重放 `HTTP 200`，`idempotent_replay=true`；`GET /api/v1/billing/{order_id}` 返回 `compensations=1`。
  - DB 回查通过：`billing.compensation_record.status=succeeded`、`billing.settlement_record.compensation_amount=20.00000000`、`support.dispute_case.status=resolved`、`ops.outbox_event(target_topic=billing.events)=1`、`audit.audit_event` 命中 `billing.compensation.execute=1` 与 `billing.compensation.execute.idempotent_replay=1`。
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
  - `cargo test -p platform-core` 通过：`221 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md`：4.7
  - `业务流程图-V1-完整版.md`：5.3
  - `页面说明书-V1-完整版.md`：8.2 / 8.3
  - `支付域接口协议正式版.md`：赔付接口、step-up、幂等键、人工打款高风险动作
  - `支付、资金流与轻结算设计.md`：高风险赔付审计与争议/结算联动
- 覆盖的任务清单条目：`BIL-010`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-184（计划中）
- 任务：`BIL-011` 人工打款/人工分账占位模型
- 状态：计划中
- 说明：在退款/赔付链路基础上补齐人工打款/人工分账占位对象、状态机、审计、事件与最小联调；V1 先支持人工执行但对象、状态、账单摘要与 OpenAPI 必须完整。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-184（待审批）
- 任务：`BIL-011` 人工打款/人工分账占位模型
- 状态：待审批
- 当前任务编号：`BIL-011`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`DB-007` 缺口已补齐并完成迁移校验；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-011` 只要求“人工打款执行 + 人工分账对象完整占位”，不提前实现 V2 分账控制面。
  - `docs/开发任务/v1-core-开发任务清单.md`：复核完成定义为“业务规则、状态机、审计、事件与测试齐备，并与上下游联调通过”。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发任务/AI-Agent-执行提示词.md`：继续按单 task 冻结流程执行，不跳步骤。
  - `docs/领域模型/全量领域模型与对象关系说明.md` `4.6`：落实 `payout_instruction / split_instruction / sub_merchant_binding / settlement_record` 聚合关系。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`：核对 `payment.payout_instruction / payment.split_instruction / payment.sub_merchant_binding` 表结构，确认你补齐的 schema 缺口已可用。
  - `docs/原始PRD/支付、资金流与轻结算设计.md` `7`：确认人工打款属于平台收费/结算域动作，V1 允许人工执行，分账对象先完整占位。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：沿用高风险 step-up、幂等、一致性与支付 provider 交互口径。
  - `docs/权限设计/角色权限矩阵正式版.md`、`docs/权限设计/接口权限校验清单.md`：确认 `payment.payout.execute_manual` 仅平台侧高风险角色可执行。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：对齐人工打款、账单事件、`billing.events` outbox 边界与 mock provider 联动要求。
  - 其余 18 份必读冻结文档已复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `POST /api/v1/payouts/manual`，要求 `x-idempotency-key` 与 step-up，占位但真实执行人工打款。
  - 新增 `billing/payout_handlers.rs` 与 `billing/repo/payout_repository.rs`，避免继续把 Billing 路由和仓储堆到单文件。
  - `CreateManualPayoutRequest / ManualPayoutExecutionView / BillingPayoutView / BillingSplitInstructionView` 已落到 `models.rs`，`GET /api/v1/billing/{order_id}` 同步返回 `payouts` 与 `split_placeholders`。
  - 人工打款执行会真实调用 live `mock-payment-provider` 的 `manual-transfer/success`，并写入：
    - `payment.payout_instruction(status=succeeded, payout_mode=manual)`
    - `payment.sub_merchant_binding` 占位绑定
    - `payment.split_instruction(split_mode=platform_ledger_then_payout, status=succeeded)`
    - `billing.billing_event(event_type=manual_settlement)`
    - `ops.outbox_event(target_topic=billing.events)`
    - `audit.audit_event(billing.payout.execute_manual / billing.payout.execute_manual.idempotent_replay / billing.event.generated)`
  - `billing.settlement_record` 与 `trade.order_main` 会同步推进到 `settled`，并回写 `billing_manual_payout_succeeded`。
  - OpenAPI 已同步更新到 `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core`
  4. `TRADE_DB_SMOKE=1 MOCK_PAYMENT_ADAPTER_MODE=live MOCK_PAYMENT_BASE_URL=http://127.0.0.1:8089 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil011_manual_payout_db_smoke -- --nocapture`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. `curl http://127.0.0.1:8089/__admin/` 确认 live mock-payment-provider 可达
  8. 启动 `APP_PORT=8104` 的 `platform-core`，插入临时订单/结算数据后执行真实 API 联调：
     - `POST /api/v1/payouts/manual`（首次成功）
     - `POST /api/v1/payouts/manual`（同幂等键重放）
     - `GET /api/v1/billing/{order_id}`
     并用 `psql` 回查 `payment.payout_instruction / payment.split_instruction / payment.sub_merchant_binding / billing.billing_event / billing.settlement_record / trade.order_main / ops.outbox_event / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `224 passed, 0 failed, 1 ignored`。
  - `bil011_manual_payout_db_smoke` 通过，确认真实 DB 写入、幂等重放、账单聚合读取、审计与 outbox 全链路成立。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - `mock-payment-provider` `__admin` 返回 `HTTP 200`。
  - 真实 API 联调通过：
    - 首次人工打款 `HTTP 200`，返回 `current_status=succeeded`、`step_up_bound=true`、`idempotent_replay=false`
    - 同幂等键重放 `HTTP 200`，返回同一 `payout_instruction_id` 且 `idempotent_replay=true`
    - `GET /api/v1/billing/{order_id}` 返回 `settlement_status=settled`、`summary_state=order_settlement:settled:manual`，且 `payouts/split_placeholders` 均为 `1`
  - DB 回查通过：
    - `payment.payout_instruction = succeeded / manual / mock-mtf-* / 85.00000000 / SGD`
    - `payment.split_instruction = succeeded / platform_ledger_then_payout / 85.00000000 / SGD`
    - `payment.sub_merchant_binding = active / organization / seller_org_id`
    - `billing.settlement_record = settled`
    - `trade.order_main = settled / billing_manual_payout_succeeded`
    - `billing.billing_event(manual_settlement)=1`
    - `ops.outbox_event(target_topic=billing.events)=1`
    - `audit.billing.payout.execute_manual=1`、`audit.billing.payout.execute_manual.idempotent_replay=1`
  - 临时业务数据已清理，回查 `order_main=0 / provider_account=0 / settlement=0`；审计按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.6`
  - `040_billing_support_risk.sql` `payment.payout_instruction / payment.split_instruction / payment.sub_merchant_binding`
  - `支付、资金流与轻结算设计.md` `7`
  - `支付域接口协议正式版.md` `6`
- 覆盖的任务清单条目：`BIL-011`
- 未覆盖项：无。
- 新增 TODO / 预留项：`TODO-BIL-011-001` 保持有效，明确 `V2` 再补正式分账控制面；本批无新增 `V1-gap`。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-185（计划中）
- 任务：`BIL-012` 支付对账导入接口 `POST /api/v1/payments/reconciliation/import`
- 状态：计划中
- 说明：按冻结支付协议补齐对账导入占位接口，真实保存对账差异结果、导入批次摘要、审计与最小联调；继续沿用 PostgreSQL + Kafka(outbox 边界) + mock provider 的 BIL 期基线。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-185（待审批）
- 任务：`BIL-012` 支付对账导入接口 `POST /api/v1/payments/reconciliation/import`
- 状态：待审批
- 当前任务编号：`BIL-012`
- 前置依赖核对结果：`TRADE-003`、`TRADE-007`、`DB-007`、`ENV-020`、`CORE-008`、`CORE-009` 已完成且审批通过；`DB-007` 缺口已补齐并完成迁移校验；保留 `TODO-PROC-BIL-001` 追溯。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-012` 只要求“支付对账导入占位 + 差异结果落库 + 最小联调”，不提前展开 reconciliation center 全量读模型。
  - `docs/开发任务/v1-core-开发任务清单.md`：复核完成定义为“接口、DTO、权限、审计、错误码和最小测试齐备，且与 OpenAPI 不漂移”。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`、`docs/开发任务/AI-Agent-执行提示词.md`：继续按单 task 冻结流程执行，不跳步骤。
  - `docs/原始PRD/支付、资金流与轻结算设计.md`：落实对账导入属于支付域分层架构内的对账中心入口，V1 先保存账单摘要和 diff 结果。
  - `docs/数据库设计/接口协议/支付域接口协议正式版.md`：落实 `POST /api/v1/payments/reconciliation/import` 的高风险 step-up、幂等导入口径与差异对象边界。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：对齐 `payment.reconciliation_statement / payment.reconciliation_diff`、`payment.reconciliation.import` 权限与支付/结算对象的 reconcile 状态联动。
  - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql`、`docs/数据库设计/数据库表字典正式版.md`：核对 `payment.reconciliation_statement / payment.reconciliation_diff` 表结构与字段口径。
  - `docs/业务流程/业务流程图-V1-完整版.md`：核对渠道账单导入后生成 diff、进入 reconciliation 处理的流程主线。
  - 其余 18 份必读冻结文档已复核，未发现与本批实现冲突的新口径。
- 实现要点：
  - 新增 `POST /api/v1/payments/reconciliation/import`，接收 multipart：`provider_key / provider_account_id / statement_date / statement_type / diffs_json / file`。
  - 新增 `ReconciliationImportDiffInput / ReconciliationStatementView / ReconciliationDiffView / ReconciliationImportView`，把对账导入批次、差异列表、幂等重放状态与 `step_up_bound` 暴露给 API。
  - 新增 `billing/reconciliation_handlers.rs` 与 `billing/repo/reconciliation_repository.rs`，避免继续把 Billing 路由和仓储堆到单文件。
  - 导入仓储会：
    - 校验 provider account 存在且 `active`
    - 以 `provider_key + provider_account_id + statement_date + statement_type` 做幂等维度
    - 对相同维度但不同文件 hash 返回 `409`
    - 计算上传文件 SHA-256 并写入 `payment.reconciliation_statement.file_hash`
    - 写入 `payment.reconciliation_statement / payment.reconciliation_diff`
    - 根据 diff 状态回写 `payment.payment_intent / billing.settlement_record / trade.order_main / payment.payout_instruction / payment.refund_intent` 的 `reconcile_status`
    - 写入审计 `payment.reconciliation.import / payment.reconciliation.import.idempotent_replay`
  - `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml` 已同步新增 reconciliation import path 与 schema。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `cargo test -p platform-core rejects_reconciliation_import_without_permission -- --nocapture`
  4. `cargo test -p platform-core rejects_reconciliation_import_without_step_up -- --nocapture`
  5. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil012_reconciliation_import_db_smoke -- --nocapture`
  6. `cargo test -p platform-core`
  7. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  8. `./scripts/check-query-compile.sh`
  9. 启动 `APP_PORT=8105` 的 `platform-core`，插入临时 provider account / order / payment_intent / settlement 数据后执行真实 API 联调：
     - `POST /api/v1/payments/reconciliation/import`（首次导入）
     - `POST /api/v1/payments/reconciliation/import`（同文件同维度幂等重放）
     并用 `psql` 回查 `payment.reconciliation_statement / payment.reconciliation_diff / payment.payment_intent / billing.settlement_record / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `227 passed, 0 failed, 0 ignored`，live integration 维持 `1 ignored`。
  - 权限/step-up 定向测试通过，确认无权限返回 `403`，缺 step-up 返回 `400`。
  - `bil012_reconciliation_import_db_smoke` 通过，确认导入成功、幂等重放、statement/diff 落库和 `reconcile_status` 回写成立。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - 首次导入 `HTTP 200`，返回 `import_status=mismatched`、`imported_diff_count=2`、`open_diff_count=1`、`idempotent_replay=false`
    - 同文件同维度重放 `HTTP 200`，返回同一 `reconciliation_statement_id` 且 `idempotent_replay=true`
  - DB 回查通过：
    - `payment.reconciliation_statement.import_status = mismatched`
    - `payment.reconciliation_diff = 2`
    - `payment.payment_intent.reconcile_status = mismatched`
    - `billing.settlement_record.reconcile_status = resolved`
    - `audit.payment.reconciliation.import = 1`
    - `audit.payment.reconciliation.import.idempotent_replay = 1`
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付、资金流与轻结算设计.md` `4`
  - `支付域接口协议正式版.md` `6`
  - `数据交易平台-全集成基线-V1.md` `27`
- 覆盖的任务清单条目：`BIL-012`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-186（计划中）
- 任务：`BIL-013` 争议案件接口 `POST /api/v1/cases`、证据上传 `POST /api/v1/cases/{id}/evidence`、裁决 `POST /api/v1/cases/{id}/resolve`
- 状态：计划中
- 说明：按冻结争议流程补齐案件创建、证据上传、裁决三条接口与争议/订单/结算联动；证据对象优先走 MinIO 实存，继续沿用 PostgreSQL + Kafka(outbox 边界) + MinIO 的 BIL 期基线。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
