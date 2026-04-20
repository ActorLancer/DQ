# SKU Billing Trigger Matrix (BIL-023)

本文件冻结 V1 首批 8 个标准 SKU 的 Billing 触发口径，作为 `BillingEvent` 生成、`Settlement` 重算、交付桥接和测试回归的唯一业务基线。

## 单一事实源

以下三处必须保持一一对应，不允许口径漂移：

1. 本文档
2. `db/seeds/031_sku_trigger_matrix.sql`
3. 运行时回退快照
   - `apps/platform-core/src/modules/billing/domain/sku_billing_basis.rs`
   - `apps/platform-core/src/modules/delivery/repo/outbox_repository.rs`

如果其中任意一处调整，另外两处必须同步更新并完成回归验证。

## 字段定义

- `default_event_type`：该 SKU 的标准基础账单事件；`null` 表示仅按用量事件计费。
- `usage_event_type`：该 SKU 的用量事件类型；`null` 表示不按量计费。
- `payment_trigger`：支付/锁资进入业务链路的冻结触发条件。
- `delivery_trigger`：交付、开通或结果准备完成的最小触发条件。
- `acceptance_trigger`：进入验收语义的最小触发条件。
- `billing_trigger`：允许生成标准 `BillingEvent` 的唯一业务触发点。
- `settlement_cycle`：`Settlement` 的聚合周期。
- `refund_entry`：退款合法入口。
- `compensation_entry`：赔付合法入口。
- `dispute_freeze_trigger`：争议立案后冻结结算的触发点。
- `resume_settlement_trigger`：争议关闭后恢复结算的触发点。

## 标准事件映射

| SKU | `default_event_type` | `usage_event_type` | 当前口径说明 |
| --- | --- | --- | --- |
| `FILE_STD` | `one_time_charge` | `null` | 单次文件交易，验收后一次性计费 |
| `FILE_SUB` | `recurring_charge` | `null` | 订阅制文件交付，按周期计费 |
| `SHARE_RO` | `one_time_charge` | `null` | 共享开通费先按最小占位口径处理，后续 `BIL-026` 补齐周期共享费与撤权退款占位 |
| `API_SUB` | `recurring_charge` | `usage_charge` | 订阅基础费 + 用量附加费并存 |
| `API_PPU` | `null` | `usage_charge` | 按量计费，不生成基础固定账单 |
| `QRY_LITE` | `one_time_charge` | `null` | 单次查询任务，验收后一次性计费 |
| `SBX_STD` | `recurring_charge` | `null` | 沙箱/席位类按周期计费 |
| `RPT_STD` | `one_time_charge` | `null` | 单次报告交付，验收后一次性计费 |

## 8 SKU 触发矩阵

下表中的代码字符串必须与 `031_sku_trigger_matrix.sql`、运行时代码快照、`billing.sku_billing_trigger_matrix` 查询结果完全一致。

| SKU | `payment_trigger` | `delivery_trigger` | `acceptance_trigger` | `billing_trigger` | `settlement_cycle` | `refund_entry` | `compensation_entry` | `dispute_freeze_trigger` | `resume_settlement_trigger` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `FILE_STD` | `order_contract_effective_lock_once` | `seller_publish_single_package` | `buyer_manual_accept_or_timeout` | `bill_once_after_acceptance` | `t_plus_1_once` | `pre_acceptance_cancel_or_acceptance_failed` | `delivery_defect_or_delay` | `freeze_on_dispute_opened` | `resume_on_dispute_closed_with_ruling` |
| `FILE_SUB` | `lock_before_each_subscription_cycle` | `generate_delivery_batch_each_cycle` | `cycle_window_manual_acceptance` | `bill_per_cycle_after_acceptance` | `monthly_cycle` | `refund_current_cycle_if_not_delivered` | `compensate_on_repeated_missing_delivery` | `freeze_future_cycles_on_dispute_opened` | `resume_after_dispute_closed_for_cycle` |
| `SHARE_RO` | `lock_before_share_grant_activation` | `readonly_share_grant_enabled` | `accessibility_check_passed` | `bill_once_on_grant_effective` | `t_plus_1_once` | `refund_if_grant_not_effective` | `compensate_on_scope_or_access_violation` | `freeze_on_share_dispute_opened` | `resume_on_dispute_closed_after_fix` |
| `API_SUB` | `lock_before_subscription_cycle_start` | `api_key_and_quota_provisioned` | `first_success_call_or_cycle_acceptance` | `bill_cycle_after_enable_and_acceptance` | `monthly_cycle` | `refund_current_cycle_if_unavailable` | `compensate_on_sla_breach` | `freeze_current_cycle_on_sla_dispute` | `resume_on_sla_dispute_closed` |
| `API_PPU` | `lock_prepaid_quota_or_minimum_commit` | `api_key_enabled_for_metering` | `usage_reconciliation_window_confirmed` | `bill_by_metered_usage` | `daily_with_monthly_statement` | `refund_failed_batch_or_unused_quota` | `compensate_on_metering_or_throttling_fault` | `freeze_metered_settlement_on_dispute` | `resume_after_metering_reconcile` |
| `QRY_LITE` | `lock_before_query_job_execution` | `query_job_succeeded_result_available` | `result_integrity_and_download_check` | `bill_once_after_task_acceptance` | `t_plus_1_once` | `refund_if_task_failed_or_unavailable` | `compensate_on_execution_unavailability` | `freeze_on_query_result_dispute` | `resume_after_result_recheck_closed` |
| `SBX_STD` | `lock_before_workspace_provision` | `workspace_account_quota_ready` | `login_and_probe_check_passed` | `bill_after_workspace_activation_acceptance` | `monthly_resource_cycle` | `refund_if_workspace_not_ready` | `compensate_on_resource_or_isolation_fault` | `freeze_on_security_or_isolation_dispute` | `resume_after_risk_cleared_dispute_closed` |
| `RPT_STD` | `lock_after_report_order_created` | `report_generated_and_downloadable` | `buyer_accept_or_timeout_acceptance` | `bill_once_after_report_acceptance` | `t_plus_1_once` | `refund_if_report_not_generated_or_rejected` | `compensate_on_critical_report_defect` | `freeze_on_report_quality_dispute` | `resume_on_review_passed_dispute_closed` |

## 业务解释补充

### `FILE_STD`
- 基础路径是：合同生效 -> 锁资 -> 交付 -> 验收 -> 一次性计费 -> `T+1` 结算。
- 退款入口严格限制在验收通过前或验收失败。

### `FILE_SUB`
- Billing 的最小粒度是“订阅周期”，不是整单。
- 争议打开后，冻结的是后续周期结算，而不是直接覆盖历史已完成周期的最终金额。

### `SHARE_RO`
- 当前 `V1` 先冻结共享开通费口径，满足授权开通、争议冻结、结算恢复的最小闭环。
- 周期共享费、撤权退款占位在 `BIL-026` 扩展，但不得改写本表已冻结字段语义。

### `API_SUB`
- 周期账单必须同时满足“已开通”与“周期验收/首次成功调用”两个条件，避免只开通未可用就出账。

### `API_PPU`
- 只有 `usage_charge`，没有基础 `default_event_type`。
- 支付与结算允许按日聚合，但对外仍可输出月账单摘要。

### `QRY_LITE`
- 查询成功且结果可用不等于可计费；仍必须经过结果完整性/可下载性验收。

### `SBX_STD`
- 账单口径围绕“工作区/席位资源可用”展开，不以对象下载或文件签收为准。

### `RPT_STD`
- 报告生成完成后仍要以买方验收或超时验收作为最终计费条件。

## 与数据库和运行时的关系

- `db/seeds/031_sku_trigger_matrix.sql` 将上述 8 行固化到 `billing.sku_billing_trigger_matrix`。
- Delivery 到 Billing 的桥接器优先读取 `billing.sku_billing_trigger_matrix`；如果表不存在或缺少 SKU 记录，则回退到与本文件一致的冻结代码快照。
- `GET /api/v1/billing/{order_id}` 通过 `sku_billing_basis` / `api_billing_basis` 向外暴露运行时口径；该响应必须可回溯到本矩阵。

## 回归要求

至少覆盖以下校验：

- `db/scripts/verify-seed-031.sh`：验证 8 个 SKU 矩阵 seed 已落库且关键字段正确。
- `bil017_api_sku_billing_basis_db_smoke`：验证 `API_SUB / API_PPU` 的最小计费规则与事件类型。
- `bil018_default_sku_billing_basis_db_smoke`：验证 `FILE_STD / FILE_SUB / SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 的默认计费规则。
- 至少一条真实 API 联调：通过 `GET /api/v1/billing/{order_id}` 回查 `sku_billing_basis` 或 `api_billing_basis`，确认返回值与本文档一致。
