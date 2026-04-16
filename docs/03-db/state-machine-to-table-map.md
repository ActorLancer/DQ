# V1 状态机到表字段映射

## 1. 订单状态机（trade）

| 生命周期对象 | 状态字段 | 状态历史/事件表 | 关键时间字段 | 关键约束/索引 |
| --- | --- | --- | --- | --- |
| `trade.order_main` | `status` | `trade.order_status_history` | `created_at`, `delivered_at`, `accepted_at`, `settled_at`, `closed_at` | `idx_order_main_status_created_at`, `idx_order_line_order_id` |
| 支付子状态 | `payment_status` | `billing.billing_event`, `payment.payment_intent` | `updated_at` | `idx_payment_intent_reconcile` |
| 一致性子状态 | `external_fact_status`, `reconcile_status` | `ops.consistency_reconcile_task`, `ops.outbox_publish_attempt` | `updated_at` | `idx_order_main_reconcile` |

说明：
- `trade.order_main.status` 承载订单主状态（创建、交付、验收、结算、关闭）。
- `trade.order_status_history` 通过触发器 `common.tg_order_status_history` 记录状态变迁审计轨迹。

## 2. 授权状态机（authorization）

| 生命周期对象 | 状态字段 | 辅助对象 | 关键时间字段 | 关键约束/索引 |
| --- | --- | --- | --- | --- |
| `trade.authorization_grant` | `status` | `delivery.data_share_grant`, `delivery.template_query_grant` | `valid_from`, `valid_to`, `updated_at` | `idx_authorization_grant_order_id`, `idx_data_share_grant_order`, `idx_template_query_grant_order` |
| `delivery.data_share_grant` | `grant_status` | `trade.order_main` | `granted_at`, `revoked_at`, `expires_at` | `idx_data_share_grant_order` |
| `delivery.template_query_grant` | `grant_status` | `delivery.query_execution_run` | `updated_at` | `idx_template_query_grant_order` |

说明：
- 授权以订单为主键上下文，`order_id` 贯穿授权、共享、查询执行对象。
- 授权过期与撤销通过 `valid_to/revoked_at` 管控。

## 3. 交付状态机（delivery）

| 生命周期对象 | 状态字段 | 辅助对象 | 关键时间字段 | 关键约束/索引 |
| --- | --- | --- | --- | --- |
| `delivery.delivery_record` | `status` | `delivery.delivery_ticket`, `delivery.delivery_receipt`, `delivery.key_envelope` | `committed_at`, `expires_at`, `updated_at` | `idx_delivery_record_order_id`, `idx_delivery_ticket_order_id` |
| `delivery.api_credential` | `status` | `delivery.api_usage_log` | `valid_from`, `valid_to`, `updated_at` | `idx_api_usage_log_order_id` |
| `delivery.sandbox_workspace` | `status` | `delivery.sandbox_session` | `created_at`, `updated_at` | `idx_sandbox_workspace_order_id` |
| `delivery.report_artifact` | `status` | `delivery.storage_object` | `created_at`, `updated_at` | `idx_report_artifact_order_id` |
| `delivery.query_execution_run` | `status` | `delivery.template_query_grant` | `created_at`, `started_at`, `finished_at` | `idx_query_execution_run_order`, `idx_query_execution_run_surface` |

说明：
- 交付路径覆盖文件/API/共享/查询/沙箱/报告六类对象。
- `delivery_*` 状态与 `trade.order_main.status` 共同构成交付闭环。

## 4. 结算状态机（billing/payment）

| 生命周期对象 | 状态字段 | 辅助对象 | 关键时间字段 | 关键约束/索引 |
| --- | --- | --- | --- | --- |
| `payment.payment_intent` | `status` | `payment.payment_webhook_event`, `payment.provider_reconciliation_batch` | `created_at`, `confirmed_at`, `updated_at` | `idx_payment_intent_status`, `idx_payment_intent_order_id`, `idx_payment_intent_reconcile` |
| `billing.settlement_record` | `settlement_status` | `billing.billing_event`, `billing.account_ledger_entry` | `created_at`, `updated_at` | `idx_billing_event_order_id`（按订单回查结算事件） |
| `billing.refund_record` | `status` | `billing.compensation_record` | `created_at`, `updated_at` | `idx_billing_event_order_id`（按订单回查退款/赔付事件） |

说明：
- 交易支付与账务结算为分层状态：支付成功不代表结算完成。
- 一致性字段由 `056_dual_authority_consistency.sql` 对齐链下与外部事实。

## 5. 争议状态机（support/risk）

| 生命周期对象 | 状态字段 | 辅助对象 | 关键时间字段 | 关键约束/索引 |
| --- | --- | --- | --- | --- |
| `support.dispute_case` | `status` | `support.dispute_evidence`, `billing.refund_record` | `created_at`, `updated_at`, `resolved_at` | `idx_dispute_case_order_id` |
| `risk.fairness_incident` | `status` | `ops.trade_lifecycle_checkpoint`, `ops.chain_projection_gap` | `created_at`, `updated_at`, `closed_at` | `idx_fairness_incident_order`, `idx_fairness_incident_ref`, `idx_fairness_incident_trace` |

说明：
- 争议处理与风险事件共享订单/引用对象主键，保证审计联查可追溯。
- `ops.chain_projection_gap` 与 `ops.external_fact_receipt` 用于监控投影缺口与外部事实迟到。

## 6. 主链路主键映射

| 业务对象 | 主键 | 关联主键 |
| --- | --- | --- |
| 订单 | `trade.order_main.order_id` | `trade.order_line.order_id`, `trade.authorization_grant.order_id`, `delivery.*.order_id`, `billing.*.order_id` |
| 商品 | `catalog.product.product_id` | `catalog.product_sku.product_id`, `trade.order_main.product_id` |
| 资产版本 | `catalog.asset_version.asset_version_id` | `catalog.product.asset_version_id`, `trade.order_main.asset_version_id` |
| 租户主体 | `core.organization.org_id` | `catalog.data_asset.owner_org_id`, `trade.order_main.buyer_org_id/seller_org_id` |

本文件用于冻结 V1 状态机字段到数据库实体的映射基线，后续任务以此为主索引进行联调与审计。
