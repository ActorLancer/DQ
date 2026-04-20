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
