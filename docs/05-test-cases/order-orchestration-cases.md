# Order Orchestration Cases

`TEST-024` 的正式目标，是把 `支付成功 -> 待交付 -> 交付完成 -> 待验收 -> 验收通过/拒收 -> 结算/退款` 编排链路收敛成一个可重复执行的 gate。它不替代 `TEST-006` 的门户 E2E，也不替代 `TEST-023` 的 SKU 覆盖矩阵；它负责证明这些阶段能顺序串起来，并在 webhook 乱序、交付重复、验收重复、结算重算下仍然闭环。

## Authority

- 任务定义：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 标准链路与 SKU 映射：`docs/全集成文档/数据交易平台-全集成基线-V1.md` `5.3.2 / 5.3.2A`
- Phase 1 验收标准：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md` `15.2`
- 前端场景基线：`docs/05-test-cases/order-e2e-cases.md`、`docs/05-test-cases/five-standard-scenarios-e2e.md`
- 后端编排证据：
  - `apps/platform-core/src/modules/order/tests/trade030_payment_result_orchestrator_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv029_delivery_task_autocreation_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv017_report_delivery_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv018_acceptance_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv025_delivery_integration_db.rs`
  - `apps/platform-core/src/modules/billing/tests/bil024_billing_trigger_bridge_db.rs`
  - `apps/platform-core/src/modules/billing/tests/bil025_billing_adjustment_freeze_db.rs`

## Official Entry

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-order-orchestration.sh
```

该 checker 会先复用 `TEST-006` 的 `check-order-e2e.sh`，再串联上述正式 backend smoke，并在 `target/test-artifacts/order-orchestration/` 产出 sign-off 摘要。

## Required Chain

| 编排阶段 | 必须证明的事实 | 正式证据 |
| --- | --- | --- |
| 支付成功 / 乱序保护 | `payment.succeeded` 能推进到 `buyer_locked`；旧 `failed` webhook 不能把状态回退 | `trade030_payment_result_orchestrator_db_smoke` |
| 待交付 | `buyer_locked` 或等价支付成功结果会自动创建 `prepared` delivery task | `dlv029_delivery_task_autocreation_db_smoke` |
| 交付完成 / 重复交付 | 正式 `deliver` 成功后可见 committed 资源；重复提交返回 `already_committed` | `dlv017_report_delivery_db_smoke`、`dlv025_delivery_storage_query_integration_db_smoke` |
| 待验收 / 验收通过 / 重复验收 | 手工验收链路能推进到 `accepted`；重复验收返回 `already_accepted` | `dlv018_acceptance_db_smoke` |
| 验收拒收 / 争议冻结 | 拒收后进入 `rejected + settlement blocked + dispute open` | `dlv018_acceptance_db_smoke`、`bil025_billing_adjustment_freeze_db_smoke` |
| 交付/验收到 Billing bridge | 交付完成、验收通过等桥接点会产生正式 `billing.trigger.bridge` 证据 | `bil024_billing_trigger_bridge_db_smoke` |
| 结算 / 退款重算 | provisional hold、manual payout/release 后，结算摘要重算为正式结果 | `bil025_billing_adjustment_freeze_db_smoke` |

## Artifact Contract

`TEST-024` checker 至少产出以下文件：

- `target/test-artifacts/order-orchestration/order-e2e.log`
- `target/test-artifacts/order-orchestration/cargo-tests.log`
- `target/test-artifacts/order-orchestration/executed-cargo-tests.txt`
- `target/test-artifacts/order-orchestration/raw/*.json`
- `target/test-artifacts/order-orchestration/summary.json`

其中：

- `raw/*.json` 由后端 smoke 在 `TEST024_ARTIFACT_DIR` 打开时写出，保存动态 order ids 与关键状态。
- `summary.json` 由 `scripts/check-order-orchestration.mjs` 汇总，必须至少给出：
  - `primary_demo_order_ids`
  - `dynamic_order_ids`
  - `signoff_order_ids`
  - `signoff_order_count`
  - `checkpoints`

## Sign-off Rule

- `TEST-024` 必须把 `TEST-006` 的 5 条主场景订单与 backend orchestration smoke 的动态订单合并成 `20+` 个唯一 `order_id`。
- `20+ order` 不允许来自静态 README 抄录；必须来自：
  - `fixtures/demo/orders.json` 的 primary scenario orders
  - `trade030 / dlv029 / dlv017 / dlv018 / dlv025 / bil024 / bil025` 在运行时实际生成并写出的 artifact order ids

## Boundaries

- `TEST-024` 不把 `portal-web` 页面验证再发明一遍，而是正式复用 `check-order-e2e.sh` 的前端链路。
- `TEST-024` 不替代 `TEST-023` 的 `8 SKU × 5 场景` 覆盖矩阵；它只验证顺序 orchestration。
- `TEST-024` 不把 `payment-billing-cases.md` 或 `delivery-cases.md` 改成第二套真相源；所有异常口径继续以这些冻结文档为准。
