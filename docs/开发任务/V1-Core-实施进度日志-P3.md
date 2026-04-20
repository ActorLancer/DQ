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
### BATCH-186（待审批）
- 任务：`BIL-013` 争议案件接口 `POST /api/v1/cases`、证据上传 `POST /api/v1/cases/{id}/evidence`、裁决 `POST /api/v1/cases/{id}/resolve`
- 当前任务编号：`BIL-013`
- 已阅读证据：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-013` 只要求补齐案件创建、证据上传、裁决三条接口，不提前展开 Dispute 对 Delivery/Billing 的冻结联动。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认完成定义是接口、DTO、权限校验、审计、错误码和最小测试齐备，且至少一条集成测试或手工 API 验证通过。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务完整流程执行，写“计划中”后编码，验证后写“待审批”，再本地提交并继续下一个任务。
  - `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守冻结文档优先级、连续单任务推进和测试留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：承接 `BIL-012` 后续批次，沿用 P3 记录本阶段实现。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯，不新增无依据 TODO。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：落实 V1 争议提交页、权限矩阵、接口权限清单与售后中心路由口径。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：落实 8.3 争议提交页表单、证据上传和页面角色范围。
  - `docs/权限设计/接口权限校验清单.md` / `docs/权限设计/菜单权限映射表.md`：校对 `create/evidence/resolve` 的权限、作用域与页面入口约束。
  - `technical_reference`：
    - `docs/领域模型/全量领域模型与对象关系说明.md:L1002`：争议与售后聚合最小对象为 `support.dispute_case / evidence_object / decision_record`。
    - `docs/业务流程/业务流程图-V1-完整版.md:L473`：争议流程要求“创建案件 -> 上传证据 -> 平台裁决”三段式闭环。
    - `docs/页面说明书/页面说明书-V1-完整版.md:L773`：争议提交页 V1 按买方入口收敛，证据上传和裁决分别对应租户与平台动作。
- 实现摘要：
  - 新增 `POST /api/v1/cases`、`POST /api/v1/cases/{id}/evidence`、`POST /api/v1/cases/{id}/resolve` 三条 Billing 路由与 DTO。
  - 权限口径按方案 A 收敛：`buyer_operator` 允许创建争议和上传证据，`platform_risk_settlement` 允许裁决且必须 step-up。
  - 争议创建会校验订单归属、争议资格、同订单同原因活跃案件冲突，并写入 `support.dispute_case`、`trade.order_main.dispute_status`、`audit.audit_event` 和 `ops.outbox_event(dispute.created -> dtp.outbox.domain-events)`。
  - 证据上传走 MinIO 实存，写入 `support.evidence_object`，按 `case_id + object_type + object_hash` 做幂等重放，并把订单/案件状态推进到 `evidence_collecting`。
  - 裁决写入 `support.decision_record`，更新 `support.dispute_case` 与 `trade.order_main.dispute_status=resolved`，并写入 `ops.outbox_event(dispute.resolved)` 与审计。
  - 同步更新 `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml`，并把页面/菜单/基线文档中的争议提交流量收敛到“买方侧”。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil013_dispute_case_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8106` 的 `platform-core`，加载 `infra/docker/.env.local` 并覆盖 `KAFKA_BROKERS/KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094`
  8. 插入临时 buyer/seller/platform、asset/product/sku/order/payment_intent/settlement 数据后执行真实 API 联调：
     - `POST /api/v1/cases`
     - `POST /api/v1/cases/{id}/evidence`
     - `POST /api/v1/cases/{id}/resolve`
     再用 `psql` 回查 `support.dispute_case / support.evidence_object / support.decision_record / trade.order_main / ops.outbox_event / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `231 passed, 0 failed, 1 ignored`。
  - `bil013_dispute_case_db_smoke` 通过，确认案件创建、证据上传、幂等重放、裁决、MinIO 对象读取、Outbox 与审计均成立。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - 创建案件 `HTTP 200`，返回 `current_status=opened`
    - 证据上传 `HTTP 200`，返回 `object_uri=s3://evidence-packages/...` 且 `idempotent_replay=false`
    - 裁决 `HTTP 200`，返回 `current_status=resolved`、`decision_code=refund_full`、`step_up_bound=true`
  - DB 回查通过：
    - `support.dispute_case.status = resolved`
    - `trade.order_main.dispute_status = resolved`
    - `support.decision_record = 1`
    - `ops.outbox_event` 命中 `dispute.created / dispute.resolved -> dtp.outbox.domain-events`
    - `audit.audit_event` 命中 `dispute.case.create / dispute.evidence.upload / dispute.case.resolve`
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.7`
  - `业务流程图-V1-完整版.md` `5.3`
  - `页面说明书-V1-完整版.md` `8.3`
- 覆盖的任务清单条目：`BIL-013`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-187（计划中）
- 任务：`BIL-014` 实现 Dispute 对 Order/Delivery/Billing 的联动：争议发起时可冻结结算、可中止交付、可触发审计保全
- 状态：计划中
- 说明：在 `BIL-013` 的 dispute case 基础上补齐对 `trade.order_main / delivery.delivery_record / billing.settlement_record` 的联动冻结与审计保全，不提前展开后续 Settlement 重算器。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-187（待审批）
- 任务：`BIL-014` 实现 Dispute 对 Order/Delivery/Billing 的联动：争议发起时可冻结结算、可中止交付、可触发审计保全
- 当前任务编号：`BIL-014`
- 已阅读证据：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-014` 只要求在争议发起时补齐 Order/Delivery/Billing 三侧联动，不提前实现 `BIL-015` 的 Settlement 重算器。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认完成定义为业务规则、状态机、审计、事件与测试齐备，且至少一条集成测试或手工 API 验证通过。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务完整流程执行，先写“计划中”，验证通过后写“待审批”，本地提交后继续下一个任务。
  - `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守冻结文档优先级、顺序推进、真实联调和留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：承接 `BIL-013` 之后的 Billing 阶段实现记录。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯，不新增无依据 TODO。
  - `technical_reference`：
    - `docs/领域模型/全量领域模型与对象关系说明.md:L1002`：落实 `support.dispute_case` 打开后对结算冻结、交付中止、审计保全的聚合联动。
    - `docs/业务流程/业务流程图-V1-完整版.md:L473`：落实“争议发起 -> 冻结结算/中止交付 -> 平台裁决”的业务链路。
    - `docs/页面说明书/页面说明书-V1-完整版.md:L773`：保持争议提交页仍由买方入口触发，但后端补齐冻结/保全副作用。
- 实现摘要：
  - 新增 `dispute_linkage_repository`，将争议开启时的三侧联动收敛为同一事务内编排。
  - `POST /api/v1/cases` 创建案件后，现在会同步：
    - 冻结 `billing.settlement_record`
    - 中止 `delivery.delivery_record` 与 `delivery.delivery_ticket`
    - 更新 `trade.order_main.delivery_status / acceptance_status / settlement_status / dispute_status`
    - 生成 `risk.freeze_ticket`
    - 生成 `risk.governance_action_log`
    - 生成 `audit.legal_hold`
    - 扩展 `ops.outbox_event(dispute.created)` 的 linkage 快照
  - 争议开启后会在提交完成后主动失效 Redis 下载票据缓存，避免被挂起的文件交付继续使用旧票据访问。
  - 修正结算冻结回写口径：`billing.settlement_record.reason_code` 统一覆盖为 `dispute_opened:{reason_code}`，保证冻结原因可追溯。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil014_dispute_linkage_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8107` 的 `platform-core`，加载 `infra/docker/.env.local` 并覆盖 `KAFKA_BROKERS/KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094`
  8. 插入临时 buyer/seller/user/asset/product/sku/order/payment_intent/settlement/delivery/ticket 数据，写入 Redis 下载票据缓存后执行真实 API：
     - `POST /api/v1/cases`
     再用 `psql + redis-cli` 回查 `trade.order_main / billing.settlement_record / delivery.delivery_record / delivery.delivery_ticket / risk.freeze_ticket / audit.legal_hold / ops.outbox_event / audit.audit_event / Redis ticket key`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core` 通过。
  - `bil014_dispute_linkage_db_smoke` 首次命中 `settlement_record.reason_code` 未覆盖争议原因的问题，已修正后重跑通过。
  - `cargo test -p platform-core` 通过；全量结果 `232 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 顺序重跑后通过；并发执行时曾先读到旧缓存，已确认不是业务缺陷。
  - 真实 API 联调通过：
    - `POST /api/v1/cases` 返回 `HTTP 200`
    - 响应 `current_status=opened`
  - DB/Redis 回查通过：
    - `trade.order_main = blocked / blocked / frozen / opened`
    - `billing.settlement_record = frozen / dispute_opened:delivery_failed`
    - `delivery.delivery_record.status = suspended`
    - `delivery.delivery_ticket.status = suspended`
    - `risk.freeze_ticket = dispute_hold / executed / delivery_failed`
    - `audit.legal_hold = active / delivery_failed`
    - `ops.outbox_event = dispute.created -> dtp.outbox.domain-events`，且 `payload.linkage.order_settlement_status = frozen`
    - `audit.audit_event` 命中 `dispute.case.create / billing.settlement.freeze / audit.legal_hold.activate / delivery.file.auto_cutoff.suspended`
    - Redis 下载票据缓存 `EXISTS` 从 `1 -> 0`
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.7`
  - `业务流程图-V1-完整版.md` `5.3`
  - `页面说明书-V1-完整版.md` `8.3`
- 覆盖的任务清单条目：`BIL-014`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-188（计划中）
- 任务：`BIL-015` 实现 Billing Event 到 Settlement 的聚合计算器，并保证幂等重算能力
- 状态：计划中
- 说明：在既有 `billing_event / settlement_record` 基础上补齐聚合重算器、冲销补差回放与摘要更新，不提前展开 `BIL-025` 的拒收/争议升级冲销规则。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-188（待审批）
- 任务：`BIL-015` 实现 Billing Event 到 Settlement 的聚合计算器，并保证幂等重算能力
- 当前任务编号：`BIL-015`
- 已阅读证据：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-015` 的目标是落地 `BillingEvent -> Settlement` 聚合计算器，并保证幂等重算。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认完成定义要求至少一条集成测试或手工 API 验证通过，且业务规则、状态机、审计、事件与测试齐备。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务完整流程执行，先写“计划中”，验证通过后写“待审批”，本地提交后继续推进下一个任务。
  - `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守冻结文档优先级、顺序推进、真实联调、日志与 TODO 留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：承接 `BIL-014` 之后的 Billing 阶段实现记录。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯，不新增无依据 TODO。
  - `technical_reference`：
    - `docs/领域模型/全量领域模型与对象关系说明.md:L895`：落实 `BillingEvent` 不是最终账单、`Settlement` 才是单次订单或任务的结算结果。
    - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql:L1`：复用 `billing.billing_event / billing.settlement_record` 的 V1 冻结表结构，不引入偏移表。
    - `docs/原始PRD/支付、资金流与轻结算设计.md:L165`：落实平台服务费、渠道手续费、退款、赔付和人工结算共同影响结算结果的收费规则。
- 实现摘要：
  - 新增 `settlement_aggregate_repository`，统一负责从 `billing.billing_event + trade.order_main.fee_preview_snapshot + settlement_record` 重算 `Settlement`。
  - 聚合器支持：
    - `one_time_charge / recurring_charge / usage_charge` 计入 `payable_amount`
    - `refund` 计入 `refund_amount`
    - `compensation` 计入 `compensation_amount`
    - `manual_settlement` 驱动 `settlement_status=settled`
  - 聚合器统一回写：
    - `billing.settlement_record`
    - `trade.order_main.settlement_status`
    - `trade.order_main.settled_at`
    - `audit.audit_event(billing.settlement.recomputed)`
  - `billing_event / refund / compensation / manual payout` 写路径已改为复用统一聚合器，不再各自直接手改 `settlement_record` 最终金额。
  - 新增 `bil015_settlement_aggregate_db_smoke`，覆盖多事件累加、重复重算幂等、账单读取摘要一致性。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil015_settlement_aggregate_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8108` 的 `platform-core`，加载 `infra/docker/.env.local` 并覆盖 `KAFKA_BROKERS/KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094`
  8. 插入临时 buyer/seller/asset/product/sku/order/provider_account 数据后执行真实 API：
     - `POST /api/v1/payments/intents`
     - `POST /api/v1/orders/{id}/lock`
     - `POST /api/v1/payments/webhooks/mock_payment`
     - 重复调用同一 webhook 做幂等重放
     - `GET /api/v1/billing/{order_id}`
     再用 `psql` 回查 `billing.billing_event / billing.settlement_record / trade.order_main / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `233 passed, 0 failed, 1 ignored`。
  - `bil015_settlement_aggregate_db_smoke` 通过，确认 `charge + refund + compensation + manual_settlement` 聚合后只保留单条 `settlement_record`，且重复重算不生成新记录。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - `POST /api/v1/payments/intents` 返回 `HTTP 200`
    - `POST /api/v1/orders/{id}/lock` 返回 `HTTP 200`
    - 首次 `POST /api/v1/payments/webhooks/mock_payment` 返回 `processed`
    - 重复 webhook 返回 `duplicate`
    - `GET /api/v1/billing/{order_id}` 返回 `HTTP 200`
  - DB 回查通过：
    - `billing.billing_event = 1`，且 `one_time_charge` 只生成一次
    - `billing.settlement_record = 1`
    - `billing.settlement_record.payable_amount = 88.00000000`
    - `billing.settlement_record.net_receivable_amount = 85.00000000`
    - `trade.order_main = buyer_locked / paid / pending_settlement`
    - `audit.audit_event` 命中 `billing.settlement.recomputed`
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.6`
  - `040_billing_support_risk.sql`
  - `支付、资金流与轻结算设计.md` `7`
- 覆盖的任务清单条目：`BIL-015`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-189（计划中）
- 任务：`BIL-016` 实现账单/结算摘要 outbox 事件，为后续 Fabric 存证和审计归档做准备
- 状态：计划中
- 说明：在 `BIL-015` 聚合器基础上补齐账单/结算摘要事件，不提前展开 `AUD` 阶段的 Fabric 链上闭环，只先保证 Kafka outbox 边界与审计归档载荷稳定。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-189（待审批）
- 任务：`BIL-016` 实现账单/结算摘要 outbox 事件，为后续 Fabric 存证和审计归档做准备
- 当前任务编号：`BIL-016`
- 已阅读证据：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-016` 目标是落地账单/结算摘要 outbox 事件，为后续 Fabric 存证和审计归档提供稳定事件边界。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认完成定义要求业务规则、状态机、审计、事件与测试齐备，并至少完成一条集成测试或手工 API 验证。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务完整流程执行，先写“计划中”，验证通过后写“待审批”，本地提交后继续推进下一个任务。
  - `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守冻结文档优先级、顺序推进、真实联调、日志与 TODO 留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：承接 `BIL-015` 之后的 Billing 阶段实现记录。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯，不新增无依据 TODO。
  - `technical_reference`：
    - `docs/领域模型/全量领域模型与对象关系说明.md:L895`：落实 `Settlement` 既是财务结算对象，也是后续审计/存证的归档锚点。
    - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql:L1`：复用 `billing.settlement_record` 与 `ops.outbox_event` 的 V1 冻结结构，不提前引入 AUD 阶段 worker。
    - `docs/原始PRD/支付、资金流与轻结算设计.md:L165`：落实账单摘要需可追溯平台服务费、渠道手续费、退款、赔付与人工结算后的最终净额。
  - `docs/开发准备/事件模型与Topic清单正式版.md`：核对 `settlement.created / settlement.completed` 事件进入 `dtp.outbox.domain-events` 的 topic 口径。
- 实现摘要：
  - 在统一 `settlement_aggregate_repository` 内补齐账单/结算摘要 outbox 写入，不新开旁路仓储，确保聚合重算与事件写入保持同一编排口径。
  - 为结算摘要新增稳定 payload：
    - `event_schema_version=v1`
    - `authority_scope=business`
    - `source_of_truth=database`
    - `proof_commit_policy=pending_fabric_anchor`
    - `proof_commit_state=pending_anchor`
    - `summary` 快照包含应结金额、平台费、渠道费、退款、赔付、供方应收与 `summary_state`
    - `order_snapshot` 包含买卖方、订单状态、支付状态、结算状态、争议状态与币种信息
  - 事件类型按结算状态分流：
    - `pending` 等未完成态 -> `settlement.created`
    - `settled/refunded/closed/canceled` -> `settlement.completed`
  - 通过稳定 `idempotency_key` 对同一结算快照去重，避免重复重算反复写同一 outbox。
  - 新增 `billing.settlement.summary.outbox` 审计动作，仅在本轮聚合真正写入 outbox 时记录。
  - 新增 `bil016_settlement_summary_outbox_db_smoke`，覆盖 `created -> completed` 两类事件、重复重算不重复写、账单读取摘要同步输出 `proof_commit_state`。
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil016_settlement_summary_outbox_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8109` 的 `platform-core`，加载 `infra/docker/.env.local` 并覆盖 `KAFKA_BROKERS/KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094`
  8. 插入临时 buyer/seller/user/asset/product/order/provider_account/payout_preference 数据后执行真实 API：
     - `POST /api/v1/payments/intents`
     - `POST /api/v1/orders/{id}/lock`
     - `POST /api/v1/payments/webhooks/mock_payment`
     - `GET /api/v1/billing/{order_id}`
     - `POST /api/v1/payouts/manual`
     - 再次 `GET /api/v1/billing/{order_id}`
     再用 `psql` 回查 `ops.outbox_event / billing.settlement_record / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `234 passed, 0 failed, 1 ignored`。
  - `bil016_settlement_summary_outbox_db_smoke` 通过，确认 `settlement.created / settlement.completed` 双事件都能落库，重复重算不重复写同类事件。
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 顺序重跑通过；此前并发运行时先读旧缓存导致失败，已确认不是业务缺陷。
  - 真实 API 联调通过：
    - `POST /api/v1/payments/intents` 返回 `HTTP 200`
    - `POST /api/v1/orders/{id}/lock` 返回 `HTTP 200`
    - `POST /api/v1/payments/webhooks/mock_payment` 返回 `processed`
    - `POST /api/v1/payouts/manual` 返回 `HTTP 200`
    - `GET /api/v1/billing/{order_id}` 返回 `HTTP 200`
  - DB 回查通过：
    - `ops.outbox_event` 命中 `settlement.created -> dtp.outbox.domain-events`
    - `ops.outbox_event` 命中 `settlement.completed -> dtp.outbox.domain-events`
    - 两条事件 `payload.summary.summary_state` 分别为 `order_settlement:pending:manual`、`order_settlement:settled:manual`
    - 两条事件 `payload.proof_commit_state = pending_anchor`
    - `billing.settlement_record = settled`，且 `settled_at IS NOT NULL`
    - `audit.audit_event` 命中 `billing.settlement.summary.outbox = 2`
    - `GET /api/v1/billing/{order_id}` 返回 `summary_state=order_settlement:settled:manual`、`proof_commit_state=pending_anchor`
  - 临时业务数据已清理，业务表回查为 0；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.6`
  - `040_billing_support_risk.sql`
  - `支付、资金流与轻结算设计.md` `7`
  - `事件模型与Topic清单正式版.md` `billing topic` 与 `dtp.outbox.domain-events`
- 覆盖的任务清单条目：`BIL-016`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-190（计划中）
- 任务：`BIL-017` 为 API_SUB/API_PPU 设计最小计费口径：订阅周期账单 + 按调用量追加事件
- 状态：计划中
- 说明：在既有 `BillingEvent / Settlement / Outbox` 链路上补齐 API 类 SKU 的订阅周期账单与调用量追加事件口径，不提前展开更高阶计费策略引擎。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-190（待审批）
- 任务：`BIL-017` 为 API_SUB/API_PPU 设计最小计费口径：订阅周期账单 + 按调用量追加事件
- 当前任务编号：`BIL-017`
- 已阅读证据：
  - `docs/开发任务/v1-core-开发任务清单.csv`：确认 `BIL-017` 目标是为 API 类 SKU 建立最小计费口径，要求业务规则、状态机、审计、事件与测试齐备。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认 `API_SUB` 需要周期账单，`API_PPU` 需要按成功调用追加账单事件，且需与现有 Billing/Settlement 主编排联动。
  - `docs/开发任务/Agent-开发与半人工审核流程.md`：继续按单任务完整流程执行，验证通过后写“待审批”，本地提交后直接推进下个任务。
  - `docs/开发任务/AI-Agent-执行提示词.md`：继续遵守冻结文档优先级、顺序推进、真实联调、日志与 TODO 留痕要求。
  - `docs/开发任务/V1-Core-实施进度日志-P3.md`：承接 `BIL-016` 后的 Billing 阶段连续实现记录。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：保持 `TODO-PROC-BIL-001` 追溯，不新增无依据 TODO。
  - `technical_reference`：
    - `docs/领域模型/全量领域模型与对象关系说明.md:L895`：落实 `BillingEvent / Settlement` 作为账单与轻结算聚合的基础事件层，支持不同 SKU 以稳定计费基元驱动结算。
    - `docs/数据库设计/V1/upgrade/040_billing_support_risk.sql:L1`：复用 V1 冻结的 `billing.billing_event / billing.settlement_record / ops.outbox_event` 结构，不提前引入后续阶段的独立计费引擎表。
    - `docs/原始PRD/支付、资金流与轻结算设计.md:L165`：落实 API 订阅周期收费与按量计费的最小口径，要求成功调用计费、周期账单与轻结算可追溯。
  - 补充参考：
    - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：核对 API_SUB 的订阅周期收费与 API_PPU 的成功调用收费口径。
    - `docs/03-db/sku-billing-trigger-matrix.md`：核对 `API_SUB / API_PPU` 的计费触发点、结算周期与后续账单矩阵口径。
- 实现摘要：
  - 新增 `billing::domain::api_billing_basis`，冻结 `API_SUB / API_PPU` 的最小计费规则：
    - `API_SUB`：`base_event_type=recurring_charge`，默认周期 `monthly`，计量来源来自 `delivery.api_credential.quota_json`
    - `API_PPU`：`usage_event_type=usage_charge`，默认周期 `per_call`，仅成功调用计费，计量来源为 `delivery.api_usage_log`
  - 新增 `billing::repo::api_billing_repository`：
    - `load_api_billing_basis_view(...)`：按订单、SKU、最新 API 凭证与使用日志构造稳定 `api_billing_basis`
    - `record_api_sub_cycle_charge_in_tx(...)`：在订单事务内为 `bill_cycle` 生成 `recurring_charge`
    - `record_api_ppu_usage_charge_in_tx(...)`：在订单事务内为 `settle_success_call` 生成 `usage_charge`
  - 将 `billing_event_repository` 拆出 `record_billing_event_in_tx(...)`，让 API 类 SKU 状态机能在单事务内同时推进订单状态、写账单事件、重算结算、写 outbox 和审计。
  - 扩展 `API_SUB / API_PPU` 状态机请求/响应：
    - `API_SUB` 请求新增 `billing_cycle_code / billing_amount`
    - `API_PPU` 请求新增 `billing_amount / usage_units / meter_window_code`
    - 两者响应新增 `billing_event_id / billing_event_type / billing_event_replayed`
  - `GET /api/v1/billing/{order_id}` 新增可选 `api_billing_basis` 聚合，稳定回传 SKU 类型、周期、包含量、超额策略、成功调用统计、最近使用量等只读视图。
  - OpenAPI 同步更新：
    - `packages/openapi/trade.yaml`
    - `packages/openapi/billing.yaml`
    - 并同步复制到 `docs/02-openapi/*.yaml`
  - 新增 `bil017_api_sku_billing_basis_db_smoke`，覆盖：
    - API_SUB 周期收费生成/重放
    - API_PPU 成功调用收费生成/重放
    - `billing.read` 返回 `api_billing_basis`
    - `billing.events` outbox 命中
- 验证步骤：
  1. `cargo fmt --all`
  2. `cargo check -p platform-core`
  3. `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil017_api_sku_billing_basis_db_smoke -- --nocapture`
  4. `cargo test -p platform-core`
  5. `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  6. `./scripts/check-query-compile.sh`
  7. 启动 `APP_PORT=8110` 的 `platform-core`，覆盖 `KAFKA_BROKERS/KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094`
  8. 插入临时 buyer/seller/api 产品、SKU、订单、API 凭证和 API usage 数据后执行真实 API：
     - `POST /api/v1/orders/{api_sub_order_id}/api-sub/transition`（`bill_cycle`）
     - 同周期重放一次 `bill_cycle`
     - `POST /api/v1/orders/{api_ppu_order_id}/api-ppu/transition`（`settle_success_call`）
     - 同 request_id 重放一次 `settle_success_call`
     - `GET /api/v1/billing/{api_sub_order_id}`
     - `GET /api/v1/billing/{api_ppu_order_id}`
     再用 `psql` 回查 `billing.billing_event / billing.settlement_record / ops.outbox_event / audit.audit_event`，随后清理临时业务数据。
- 验证结果：
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core` 全部通过；全量结果 `235 passed, 0 failed, 1 ignored`。
  - `bil017_api_sku_billing_basis_db_smoke` 通过，确认：
    - `API_SUB` 同一 `billing_cycle_code` 只生成一条 `recurring_charge`
    - `API_PPU` 同一 `request_id` 只生成一条 `usage_charge`
    - `GET /api/v1/billing/{order_id}` 稳定返回 `api_billing_basis`
    - 对应 `billing.events` outbox 命中 1 条
  - `cargo sqlx prepare --workspace` 通过，`.sqlx/` 元数据已刷新。
  - `./scripts/check-query-compile.sh` 通过。
  - 真实 API 联调通过：
    - `API_SUB bill_cycle` 返回 `HTTP 200`，`current_state=active`，`billing_event_type=recurring_charge`，首次 `billing_event_replayed=false`
    - 同周期重放返回同一 `billing_event_id`，`billing_event_replayed=true`
    - `API_PPU settle_success_call` 返回 `HTTP 200`，`current_state=usage_active`，`billing_event_type=usage_charge`，首次 `billing_event_replayed=false`
    - 同 request_id 重放返回同一 `billing_event_id`，`billing_event_replayed=true`
    - `GET /api/v1/billing/{api_sub_order_id}` 返回 `api_billing_basis = {sku_type=API_SUB, base_event_type=recurring_charge, cycle_period=monthly, included_units=1000}`
    - `GET /api/v1/billing/{api_ppu_order_id}` 返回 `api_billing_basis = {sku_type=API_PPU, usage_event_type=usage_charge, cycle_period=per_call, latest_usage_call_count=2, latest_usage_units=128.00000000}`
  - DB 回查通过：
    - `billing.billing_event` 命中 `recurring_charge=1`、`usage_charge=1`
    - 两条事件均写入 `ops.outbox_event(target_topic=billing.events)`
    - 事件 `metadata.api_billing_basis` 分别保留 `cycle_period=monthly`、`latest_usage_units=128.00000000`
    - `audit.audit_event` 命中：
      - `trade.order.api_sub.transition`
      - `trade.order.api_ppu.transition`
      - `billing.event.record.api_sub_cycle`
      - `billing.event.record.api_sub_cycle.idempotent_replay`
      - `billing.event.record.api_ppu_usage`
      - `billing.event.record.api_ppu_usage.idempotent_replay`
      - `billing.order.read`
      - `billing.settlement.recomputed`
      - `billing.settlement.summary.outbox`
  - 临时业务数据已清理，回查 `trade.order_main / billing.billing_event / ops.outbox_event / core.organization = 0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.6`
  - `040_billing_support_risk.sql`
  - `支付、资金流与轻结算设计.md` `7`
  - `全集成基线-V1.md` 中 API_SUB/API_PPU 收费口径
  - `docs/03-db/sku-billing-trigger-matrix.md` 中 API 类 SKU 计费矩阵
- 覆盖的任务清单条目：`BIL-017`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-191（计划中）
- 任务：`BIL-018` 为 FILE_STD/FILE_SUB/SHARE_RO/QRY_LITE/SBX_STD/RPT_STD 设计默认计费口径与退款逻辑占位，并补充共享开通类 SKU 的计费触发点
- 状态：计划中
- 说明：在既有 `BillingEvent / Settlement / Refund / Outbox` 链路上补齐剩余 6 个标准 SKU 的默认计费规则和退款入口占位，不提前实现 V2 阶段的复杂价规引擎。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-191（待审批）
- 任务：`BIL-018` 为 FILE_STD/FILE_SUB/SHARE_RO/QRY_LITE/SBX_STD/RPT_STD 设计默认计费口径与退款逻辑占位，并补充共享开通类 SKU 的计费触发点
- 状态：待审批
- 实现摘要：
  - 新增统一 `SkuBillingBasisRule / SkuBillingBasisView`，为 `FILE_STD / FILE_SUB / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 冻结默认计费、退款、赔付、争议冻结与恢复口径；`GET /api/v1/billing/{order_id}` 新增 `sku_billing_basis` 聚合视图。
  - `SHARE_RO enable_share` 现在会在订单事务内生成占位 `one_time_charge` 计费事件，补写 `billing.events` outbox 并写入 `billing.event.record.share_ro_enable` 审计。
  - 更新 `ShareRoTransitionResponseData` 与 Billing OpenAPI，使共享开通响应与账单详情对新计费视图保持一致。
- 验证：
  - `cargo fmt --all`
  - `cargo check -p platform-core`
  - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil018_default_sku_billing_basis_db_smoke -- --nocapture`
  - `cargo test -p platform-core`
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  - `./scripts/check-query-compile.sh`
  - 真实 API 联调：启动 `APP_PORT=8111 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`，随后执行：
    - `GET /api/v1/billing/{file_order_id}`
    - `GET /api/v1/billing/{share_order_id}`
    - `POST /api/v1/orders/{share_order_id}/share-ro/transition`
    - `GET /api/v1/billing/{share_order_id}`
    - `psql` 回查 `billing.billing_event / ops.outbox_event / audit.audit_event`
- 验证结果：
  - 专项 smoke 通过；全量测试通过：`239 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 均通过，`.sqlx` 已刷新。
  - 真实 API 联调通过：四个 HTTP 调用全部返回 `200`；`FILE_STD` 账单详情返回 `sku_billing_basis=FILE_STD/one_time_charge/REFUND_FILE_STD_V1`；`SHARE_RO` 开通后返回 `billing_event_type=one_time_charge`、`billing_event_replayed=false`，二次账单读取显示 `billing_events=1`。
  - DB 回查通过：`billing.billing_event(one_time_charge)=1`、`ops.outbox_event(target_topic=billing.events)=1`、审计命中 `billing.event.record.share_ro_enable`、`trade.order.share_ro.transition`、`billing.order.read`；临时业务数据已清理，审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `全量领域模型与对象关系说明.md` `4.6`
  - `040_billing_support_risk.sql`
  - `支付、资金流与轻结算设计.md` `7`
- 覆盖的任务清单条目：`BIL-018`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-192（计划中）
- 任务：`BIL-019` 为支付与账单编写集成测试：支付成功、支付失败、超时重试、退款、赔付、争议升级、账单重算
- 状态：计划中
- 说明：在现有 `payment intent / webhook / refund / compensation / dispute / settlement recompute` 主链路上补齐一条覆盖成功、失败、超时、退款、赔付、争议升级、账单重算的统一集成测试与真实联调验证。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
### BATCH-192（待审批）
- 任务：`BIL-019` 为支付与账单编写集成测试：支付成功、支付失败、超时重试、退款、赔付、争议升级、账单重算
- 状态：待审批
- 实现摘要：
  - 新增 `apps/platform-core/src/modules/billing/tests/bil019_payment_billing_integration_db.rs`，将 `payment intent -> order lock -> webhook(success/failed/timeout) -> billing read` 与 `dispute case -> resolve -> refund/compensation -> settlement recompute` 两条主链路收敛为统一集成 smoke。
  - 复用新的独立 SQLx pool key 进行 seed 与应用路由联调，避免测试侧共享默认池导致连接耗尽；同时将 webhook timeout 口径对齐现实现的 `payment.timeout -> expired` 语义。
  - 校正争议解决后的订单断言到当前冻结实现：`trade.order_main.last_reason_code = billing_dispute_resolved`。
- 验证：
  - `cargo fmt --all`
  - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core bil019_ -- --nocapture`
  - `cargo check -p platform-core`
  - `cargo test -p platform-core`
  - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
  - `./scripts/check-query-compile.sh`
  - 真实 API 联调：启动 `APP_PORT=8112 KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core`，随后执行：
    - `POST /api/v1/payments/intents` x3
    - `POST /api/v1/orders/{id}/lock` x3
    - `POST /api/v1/payments/webhooks/mock_payment` x3
    - `POST /api/v1/cases` x2
    - `POST /api/v1/cases/{id}/resolve` x2
    - `POST /api/v1/refunds`
    - `POST /api/v1/compensations`
    - `GET /api/v1/billing/{order_id}` x3
    - `psql` 回查 `trade.order_main / ops.outbox_event / audit.audit_event`
- 验证结果：
  - 专项 smoke 通过：`2 passed, 0 failed`。
  - 全量测试通过：`241 passed, 0 failed, 1 ignored`。
  - `cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 均通过，`.sqlx` 已刷新。
  - 真实 API 联调通过：
    - payment success -> `processed / succeeded`
    - payment failed -> `processed / failed`
    - payment timeout -> `processed / expired`
    - dispute refund / compensation 两条链路均返回 `HTTP 200`，对应记录状态均为 `succeeded`
    - `GET /api/v1/billing/{refund_order}` 返回 `refunds=1`、`refund_adjustment_amount=20.00000000`
    - `GET /api/v1/billing/{comp_order}` 返回 `compensations=1`、`compensation_adjustment_amount=20.00000000`
  - DB 回查通过：
    - 支付后三笔订单状态分别为：`buyer_locked/paid`、`payment_failed_pending_resolution/failed`、`payment_timeout_pending_compensation_cancel/expired`
    - 退款/赔付订单均为：`settlement_status=frozen`、`dispute_status=resolved`、`last_reason_code=billing_dispute_resolved`
    - `ops.outbox_event` 命中：`support.dispute_case=4`、`billing.refund_record=1`、`billing.compensation_record=1`
    - `audit.audit_event` 命中：`billing.refund.execute=1`、`billing.compensation.execute=1`
  - 真实联调临时业务数据已清理，回查 `core.organization / core.user_account / payment.provider_account / trade.order_main = 0`；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `支付、资金流与轻结算设计.md` `4`
  - `支付域接口协议正式版.md` `6`
  - `数据交易平台-全集成基线-V1.md` `15`
- 覆盖的任务清单条目：`BIL-019`
- 未覆盖项：无。
- 新增 TODO / 预留项：无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`TODO-PROC-BIL-001` 追溯约束保持不变。
- 备注：`V1-Core-人工审批记录.md` 按约定由你手工维护，本批未写入。
### BATCH-193（计划中）
- 任务：`BIL-020` 生成 `docs/02-openapi/billing.yaml` 第一版并与实现校验
- 状态：计划中
- 说明：基于当前 `packages/openapi/billing.yaml` 与 `apps/platform-core/src/modules/billing/api/mod.rs` 的实际路由，归档第一版 `docs/02-openapi/billing.yaml`，并补做实现/契约一致性校验与最小真实 API 读取验证。
- 追溯：`TODO-PROC-BIL-001` 保持追溯，继续按 BIL 顺序推进。
