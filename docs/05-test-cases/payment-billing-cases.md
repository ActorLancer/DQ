# Payment Billing Cases

## Scope

本矩阵冻结 Billing / Payment / Settlement / Dispute 子域在 V1 的最小可信回归面，重点覆盖：

- 支付回调乱序
- 重复回调幂等
- 重复扣费防护
- 争议升级后的结算冻结
- 裁决退款 / 赔付后的结算重算

主状态、审计、事件与接口契约口径以 `payment.payment_intent`、`trade.order_main`、`billing.billing_event`、`billing.settlement_record`、`support.dispute_case` 为准，不允许由页面或临时脚本改写业务真值。

## Invariants

- `payment.payment_webhook_event` 对同一 `provider_event_id` 必须可判定 `processed / duplicate / out_of_order_ignored / processed_noop`。
- `payment.payment_intent.status` 允许前进，不允许被旧回调回退。
- `trade.order_main` 的支付相关推进必须遵守主状态不可倒退规则。
- `billing.billing_event` 只能新增标准事件或冲销/补差事件，不允许直接手改最终结算金额。
- 争议进入冻结态后，`billing.settlement_record` 必须进入 `frozen` 或等价冻结口径；恢复靠标准事件重算，不靠人工篡改汇总字段。

## Matrix

| 用例ID | 场景 | 触发 | 预期结果 | 自动化证据 |
| --- | --- | --- | --- | --- |
| `PWB-001` | 支付成功回调 | `POST /api/v1/payments/webhooks/mock_payment` with `payment.succeeded` | `payment_intent.status=succeeded`；`trade.order_main.status=buyer_locked`；`payment.webhook.processed=1` | `apps/platform-core/src/modules/billing/tests/bil005_payment_webhook_db.rs`、`apps/platform-core/src/modules/order/tests/trade030_payment_result_orchestrator_db.rs` |
| `PWB-002` | 重复回调去重 | 对同一 `provider_event_id` 重放 success webhook | 第二次返回 `processed_status=duplicate`；不新增第二条 `payment_transaction`；不重复推进订单 | `apps/platform-core/src/modules/billing/tests/bil005_payment_webhook_db.rs` |
| `PWB-003` | 回调乱序保护 | 先处理较新的 success，再发送较旧 failed | 较旧回调返回 `out_of_order_ignored`；`payment_intent.status` 保持 `succeeded`；订单不回退 | `apps/platform-core/src/modules/billing/tests/bil005_payment_webhook_db.rs`、`apps/platform-core/src/modules/order/tests/trade024_illegal_state_regression_db.rs` |
| `PWB-004` | 超时支付口径 | 处理 `payment.timeout` | `payment_intent.status=expired`；订单进入 `payment_timeout_pending_compensation_cancel`；审计保留 `payment.webhook.processed` | `apps/platform-core/src/modules/billing/tests/bil004_mock_payment_adapter_db.rs`、`apps/platform-core/src/modules/order/tests/trade030_payment_result_orchestrator_db.rs` |
| `PWB-005` | FILE 类重复扣费防护 | 同一订单重复创建/重放相同扣费语义 | 同一幂等键只保留一条 `payment_intent` / `billing_event`；不会重复累计结算 | `apps/platform-core/src/modules/billing/tests/bil002_payment_intent_db.rs`、`apps/platform-core/src/modules/billing/tests/bil006_billing_event_db.rs` |
| `PWB-006` | API_SUB 周期计费重放防护 | 同一 `bill_cycle` 重放 | 返回相同 `billing_event_id`，`billing_event_replayed=true`；只存在一条 `recurring_charge` | `apps/platform-core/src/modules/billing/tests/bil017_api_sku_billing_basis_db.rs` |
| `PWB-007` | API_PPU 用量计费重放防护 | 同一 request_id 重放 `settle_success_call` | 返回相同 `billing_event_id`，`billing_event_replayed=true`；只存在一条 `usage_charge` | `apps/platform-core/src/modules/billing/tests/bil017_api_sku_billing_basis_db.rs` |
| `PWB-008` | 争议升级冻结结算 | `POST /api/v1/cases` 打开争议 | `trade.order_main.settlement_status=frozen`；`risk.freeze_ticket` 与 `audit.legal_hold` 落库；下载票据失效 | `apps/platform-core/src/modules/billing/tests/bil014_dispute_linkage_db.rs` |
| `PWB-009` | 退款后结算重算 | 争议裁决 `refund_full` 后执行 `POST /api/v1/refunds` | `billing.refund_record.status=succeeded`；`settlement_summary.refund_adjustment_amount` 正确；保留 `billing.refund_record` outbox | `apps/platform-core/src/modules/billing/tests/bil009_refund_db.rs`、`apps/platform-core/src/modules/billing/tests/bil019_payment_billing_integration_db.rs` |
| `PWB-010` | 赔付后结算重算 | 争议裁决 `compensation_full` 后执行 `POST /api/v1/compensations` | `billing.compensation_record.status=succeeded`；`settlement_summary.compensation_adjustment_amount` 正确；保留 `billing.compensation_record` outbox | `apps/platform-core/src/modules/billing/tests/bil010_compensation_db.rs`、`apps/platform-core/src/modules/billing/tests/bil019_payment_billing_integration_db.rs` |
| `PWB-011` | 统一结算聚合 | 支付、退款、赔付、人工打款任意组合后读取账单 | `billing.settlement_record` 只保留单条聚合结果，且金额由事件重算得到 | `apps/platform-core/src/modules/billing/tests/bil015_settlement_aggregate_db.rs` |
| `PWB-012` | 结算摘要 outbox 边界 | 结算汇总创建/完成 | `ops.outbox_event(target_topic=dtp.outbox.domain-events)` 命中 `settlement.created / settlement.completed` | `apps/platform-core/src/modules/billing/tests/bil016_settlement_summary_outbox_db.rs` |
| `PWB-013` | SHARE_RO 开通费 + 周期共享费 + 撤权退款占位 | `enable_share -> POST /api/v1/billing/{order_id}/share-ro/cycle-charge -> revoke share` | `billing_events` 依次出现 `one_time_charge / recurring_charge / refund_adjustment`；`sku_billing_basis` 暴露 `cycle_event_type / periodic_settlement_cycle / refund_placeholder_entry`；撤权后账单摘要按 placeholder 重算 | `apps/platform-core/src/modules/billing/tests/bil026_share_ro_billing_db.rs` |
| `PWB-014` | SHARE_RO 争议冻结 | 已开通共享订单执行 `POST /api/v1/cases` | `trade.order_main.settlement_status=frozen`；`billing.billing_event(event_source=settlement_dispute_hold)=1`；`sku_billing_basis.dispute_freeze_trigger=freeze_on_share_dispute_opened` | `apps/platform-core/src/modules/billing/tests/bil026_share_ro_billing_db.rs` |

## Manual Smoke Baseline

最小手工回归至少覆盖以下 5 步：

1. 创建支付意图并锁单，然后发送 success webhook，确认 `payment_intent.status=succeeded`。
2. 重放相同 success webhook，确认 `processed_status=duplicate`。
3. 再发送旧时间戳 failed webhook，确认 `processed_status=out_of_order_ignored`。
4. 为已交付订单创建争议，确认 `trade.order_main.settlement_status=frozen` 且 `billing.order.read` 可联查冻结后的账单摘要。
5. 对 `SHARE_RO` 订单依次执行开通、周期计费、撤权，确认 `GET /api/v1/billing/{order_id}` 返回 `one_time_charge / recurring_charge / refund_adjustment` 三类事件。

## Traceability

- 业务分层：`docs/原始PRD/支付、资金流与轻结算设计.md` `4`
- 幂等与一致性：`docs/数据库设计/接口协议/支付域接口协议正式版.md` `6`
- 集成基线：`docs/全集成文档/数据交易平台-全集成基线-V1.md` `27`

## TEST-013 Official Entry

- 正式用例清单：`docs/05-test-cases/dispute-settlement-linkage-cases.md`
- 正式 checker：`ENV_FILE=infra/docker/.env.local ./scripts/check-dispute-settlement-linkage.sh`
- 核心自动化证据：`apps/platform-core/src/modules/billing/tests/bil019_payment_billing_integration_db.rs` 中的 `bil019_dispute_refund_compensation_recompute_db_smoke`
