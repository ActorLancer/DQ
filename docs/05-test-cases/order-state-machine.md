# Order State Machine Matrix (TRADE-029)

## 1. 目标与范围

本矩阵用于冻结 V1 `Order / Contract / Authorization` 主交易链路中 8 个标准 SKU 的订单状态转换口径，并把现有自动化 smoke / 仓储单测与真实接口验证对应起来，作为后续回归、审查和上游依赖任务的统一测试基线。

范围仅包含：

- `FILE_STD`
- `FILE_SUB`
- `API_SUB`
- `API_PPU`
- `SHARE_RO`
- `QRY_LITE`
- `SBX_STD`
- `RPT_STD`

范围内默认接口均为：`POST /api/v1/orders/{id}/*/transition`。

不包含：

- V2/V3 预留能力
- 计费 webhook 内部回调实现细节
- 非标准 SKU 或跨产品组合包扩展

## 2. 关联冻结约束

- 领域模型：`docs/领域模型/全量领域模型与对象关系说明.md`（4.4 交易与订单聚合）
- 核心集成基线：`docs/全集成文档/数据交易平台-全集成基线-V1.md`（15. 核心交易链路设计）
- 业务流程：`docs/业务流程/业务流程图-V1-完整版.md`（4.3 买方搜索、选购与下单流程）
- 实现路由：`apps/platform-core/src/modules/order/api/mod.rs`
- 自动化证据：`apps/platform-core/src/modules/order/tests/trade008_*` 到 `trade015_*`

## 3. 自动化证据总览

| SKU | Transition Endpoint | 主要 smoke 证据 | 仓储单测证据 | 关联交叉验证 |
| --- | --- | --- | --- | --- |
| `FILE_STD` | `/api/v1/orders/{id}/file-std/transition` | `trade008_file_std_state_machine_db.rs` | `order_file_std_repository.rs` | `TRADE-021`, `TRADE-027` |
| `FILE_SUB` | `/api/v1/orders/{id}/file-sub/transition` | `trade009_file_sub_state_machine_db.rs` | `order_file_sub_repository.rs` | `TRADE-021` |
| `API_SUB` | `/api/v1/orders/{id}/api-sub/transition` | `trade010_api_sub_state_machine_db.rs` | `order_api_sub_repository.rs` | `TRADE-021` |
| `API_PPU` | `/api/v1/orders/{id}/api-ppu/transition` | `trade011_api_ppu_state_machine_db.rs` | `order_api_ppu_repository.rs` | `TRADE-018` |
| `SHARE_RO` | `/api/v1/orders/{id}/share-ro/transition` | `trade012_share_ro_state_machine_db.rs` | `order_share_ro_repository.rs` | `TRADE-018`, `TRADE-027` |
| `QRY_LITE` | `/api/v1/orders/{id}/qry-lite/transition` | `trade013_qry_lite_state_machine_db.rs` | `order_qry_lite_repository.rs` | 无 |
| `SBX_STD` | `/api/v1/orders/{id}/sbx-std/transition` | `trade014_sbx_std_state_machine_db.rs` | `order_sbx_std_repository.rs` | 无 |
| `RPT_STD` | `/api/v1/orders/{id}/rpt-std/transition` | `trade015_rpt_std_state_machine_db.rs` | `order_rpt_std_repository.rs` | 无 |

## 4. 跨 SKU 通用断言

### 4.1 主状态唯一性

- `trade.order_main.status` 是订单推进唯一主状态。
- `payment_status / delivery_status / acceptance_status / settlement_status / dispute_status` 是子域镜像状态，不替代主状态。
- 非法动作必须返回 `409 TRD_STATE_CONFLICT`，消息中包含对应 `*_TRANSITION_FORBIDDEN`。

### 4.2 通用子状态映射

对未定义 SKU 专属覆写的状态，统一使用 `derive_layered_status(...)`：

| 主状态 | delivery_status | acceptance_status | settlement_status | dispute_status |
| --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` / `buyer_locked` | `pending_delivery` | `not_started` | `paid -> pending_settlement`，否则 `not_started` | `none` |
| `seller_delivering` | `in_progress` | `not_started` | `paid -> pending_settlement`，否则 `not_started` | `none` |
| `delivered` | `delivered` | `pending_acceptance` | `paid -> pending_settlement`，否则 `not_started` | `none` |
| `accepted` | `delivered` | `accepted` | `paid -> pending_settlement`，否则 `not_started` | `none` |
| `settled` | `delivered` | `accepted` | `settled` | `none` |
| `closed` | `closed` | `closed` | `closed` | `none` |

### 4.3 前置门禁与回退保护

- `TRADE-021` 已补齐锁资前门禁：主体状态、商品状态、审核状态、模板齐备、价格快照完整性；适用于 `FILE_STD / FILE_SUB / API_SUB` 的支付锁定入口。
- `TRADE-024` 已补齐晚到支付结果的非法回退保护；支付 webhook 不能把已进入更后履约阶段的订单拉回早期状态。
- `TRADE-027` 已做主交易链路集成验证：下单、合同确认、锁资前阻断/成功、非法跳转、自动断权。

## 5. 标准 SKU 状态转换测试矩阵

### 5.1 FILE_STD

正向主链路：`created -> buyer_locked -> seller_delivering -> delivered -> accepted -> settled -> closed`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `lock_funds` | `buyer_locked` | `paid` | 进入支付锁定后才允许交付 | `trade008`, `trade021`, `trade027` |
| `buyer_locked` | `start_delivery` | `seller_delivering` | 保持原值 | 卖方开始交付 | `trade008` |
| `seller_delivering` | `mark_delivered` | `delivered` | 保持原值 | 交付完成 | `trade008` |
| `delivered` | `accept_delivery` | `accepted` | 保持原值 | 买方验收通过 | `trade008` |
| `accepted` | `settle_order` | `settled` | `paid` | 进入结算完成态 | `trade008` |
| `settled` | `close_completed` | `closed` | 保持原值 | 正常完结 | `trade008` |
| `buyer_locked` / `seller_delivering` / `delivered` / `accepted` / `settled` | `request_refund` | `closed` | `refunded` | 退款分支 | `trade008` |
| `delivered` / `accepted` / `settled` | `open_dispute` | `dispute_opened` | 保持原值 | 争议开启；`acceptance=disputed`，`settlement=blocked` | `trade008`, repo 单测 |
| `dispute_opened` | `resolve_dispute_refund` | `closed` | `refunded` | 争议退款关闭 | `trade008` |
| `dispute_opened` | `resolve_dispute_complete` | `settled` | `paid` | 争议完成后继续结算 | `trade008`, repo 单测 |

禁止样例：`buyer_locked -> close_completed`，应返回 `FILE_STD_TRANSITION_FORBIDDEN`。证据：`TRADE-027`。

### 5.2 FILE_SUB

正向主链路：`created -> buyer_locked -> seller_delivering -> delivered -> accepted`

订阅分支：`accepted -> paused -> buyer_locked` 或 `accepted/paused -> expired`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `establish_subscription` | `buyer_locked` | `paid` | 首次订阅锁定 | `trade009`, `trade021` |
| `buyer_locked` / `accepted` | `start_cycle_delivery` | `seller_delivering` | 保持原值 | 周期交付开始 | `trade009` |
| `seller_delivering` | `mark_cycle_delivered` | `delivered` | 保持原值 | 当前周期已交付 | `trade009` |
| `delivered` | `accept_cycle_delivery` | `accepted` | 保持原值 | 当前周期验收通过 | `trade009` |
| `buyer_locked` / `accepted` | `pause_subscription` | `paused` | 保持原值 | 暂停订阅；子状态同步 `paused` | `trade009`, repo 单测 |
| `buyer_locked` / `accepted` / `paused` | `expire_subscription` | `expired` | 保持原值 | 到期终止 | `trade009` |
| `paused` / `expired` | `renew_subscription` | `buyer_locked` | `paid` | 续费回到锁定态 | `trade009`, repo 单测 |
| `buyer_locked` / `seller_delivering` / `delivered` / `accepted` / `paused` | `request_refund` | `closed` | `refunded` | 退款关闭 | `trade009` |
| `delivered` / `accepted` / `paused` | `open_dispute` | `dispute_opened` | 保持原值 | 争议开启 | `trade009` |
| `dispute_opened` | `resolve_dispute_refund` | `closed` | `refunded` | 争议退款关闭 | `trade009`, repo 单测 |
| `dispute_opened` | `resolve_dispute_complete` | `accepted` | `paid` | 争议解决后继续订阅周期 | `trade009` |

禁止样例：在终态后继续执行周期交付动作，应返回 `FILE_SUB_TRANSITION_FORBIDDEN`。证据：`trade009` 负向断言。

### 5.3 API_SUB

正向主链路：`created -> buyer_locked -> api_bound -> api_key_issued -> api_trial_active -> active`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `lock_funds` | `buyer_locked` | `paid` | 首期费用锁定 | `trade010`, `trade021` |
| `buyer_locked` | `bind_application` | `api_bound` | 保持原值 | 应用绑定完成 | `trade010`, repo 单测 |
| `api_bound` | `issue_api_key` | `api_key_issued` | 保持原值 | 发放凭证 | `trade010`, repo 单测 |
| `api_key_issued` | `trial_call` | `api_trial_active` | 保持原值 | 首次试调用 | `trade010`, repo 单测 |
| `api_trial_active` | `activate_subscription` | `active` | 保持原值 | 正式订阅生效 | `trade010`, repo 单测 |
| `active` | `bill_cycle` | `active` | `paid` | 周期账单循环，不改变主状态 | `trade010` |
| `buyer_locked` / `api_bound` / `api_key_issued` / `api_trial_active` / `active` | `terminate_subscription` | `closed` | 保持原值 | 主动终止订阅 | `trade010` |

禁止样例：`closed -> bill_cycle`，应返回 `API_SUB_TRANSITION_FORBIDDEN`。证据：repo 单测 `closed_cannot_reenter_billing_cycle`。

### 5.4 API_PPU

正向主链路：`created -> api_authorized -> quota_ready -> usage_active`

`API_PPU` 的计费特性是：成功调用才把 `payment_status` 提升为 `paid`；失败调用不出账。

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `authorize_access` | `api_authorized` | 保持原值 | 开通访问授权 | `trade011`, repo 单测 |
| `api_authorized` | `configure_quota` | `quota_ready` | 保持原值 | 配额配置完成 | `trade011`, repo 单测 |
| `quota_ready` / `usage_active` | `record_failed_call` | `usage_active` | 保持原值 | 失败调用不计费 | `trade011`, repo 单测 |
| `quota_ready` / `usage_active` | `settle_success_call` | `usage_active` | `paid` | 成功调用出账 | `trade011`, repo 单测 |
| `buyer_locked` / `api_authorized` / `quota_ready` / `usage_active` | `expire_access` | `expired` | 保持原值 | 到期断权 | `trade011` |
| `buyer_locked` / `api_authorized` / `quota_ready` / `usage_active` / `expired` | `disable_access` | `disabled` | 保持原值 | 风险冻结/人工禁用 | `trade011`, `trade018` |

禁止样例：`disabled -> settle_success_call`，应返回 `API_PPU_TRANSITION_FORBIDDEN`。证据：repo 单测 `disabled_cannot_reenter_settlement`。

### 5.5 SHARE_RO

正向主链路：`created -> share_enabled -> share_granted -> shared_active`

断权/异常分支：`revoked` / `expired` / `dispute_interrupted`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `enable_share` | `share_enabled` | 保持原值 | 开启共享 | `trade012`, `TRADE-029` API 联调 |
| `share_enabled` | `grant_read_access` | `share_granted` | 保持原值 | 发放只读访问 | `trade012` |
| `share_granted` | `confirm_first_query` | `shared_active` | 保持原值 | 首次查询成功，进入活跃共享态 | `trade012`, repo 单测 |
| `share_enabled` / `share_granted` / `shared_active` / `expired` | `revoke_share` | `revoked` | 保持原值 | 主动撤销共享 | `trade012`, repo 单测 |
| `share_enabled` / `share_granted` / `shared_active` | `expire_share` | `expired` | 保持原值 | 到期断权 | `trade012`, `trade018`, `trade027` |
| `share_enabled` / `share_granted` / `shared_active` | `interrupt_dispute` | `dispute_interrupted` | 保持原值 | 争议中断；`delivery=blocked`，`settlement=frozen` | `trade012`, `trade018` |

禁止样例：`revoked -> grant_read_access`，应返回 `SHARE_RO_TRANSITION_FORBIDDEN`。证据：repo 单测 `revoked_cannot_grant_again`。

### 5.6 QRY_LITE

正向主链路：`created -> template_authorized -> params_validated -> query_executed -> result_available -> closed`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `authorize_template` | `template_authorized` | 保持原值 | 授权模板查询 | `trade013`, repo 单测 |
| `template_authorized` | `validate_params` | `params_validated` | 保持原值 | 校验查询参数 | `trade013`, repo 单测 |
| `params_validated` | `execute_query` | `query_executed` | `paid` | 执行查询并记账 | `trade013`, repo 单测 |
| `query_executed` | `make_result_available` | `result_available` | 保持原值 | 结果可下载/查看 | `trade013`, repo 单测 |
| `result_available` | `close_acceptance` | `closed` | 保持原值 | 验收结束并关闭 | `trade013`, repo 单测 |

禁止样例：`closed -> execute_query`，应返回 `QRY_LITE_TRANSITION_FORBIDDEN`。证据：repo 单测 `closed_cannot_execute_again`。

### 5.7 SBX_STD

正向主链路：`created -> workspace_enabled -> seat_issued -> sandbox_executed -> result_limited_exported`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `enable_workspace` | `workspace_enabled` | 保持原值 | 开通沙箱工作区 | `trade014`, repo 单测 |
| `workspace_enabled` | `issue_account_seat` | `seat_issued` | 保持原值 | 发放席位/账号 | `trade014`, repo 单测 |
| `seat_issued` | `execute_sandbox_query` | `sandbox_executed` | 保持原值 | 执行沙箱查询 | `trade014`, repo 单测 |
| `sandbox_executed` | `export_limited_result` | `result_limited_exported` | 保持原值 | 导出受限结果 | `trade014`, repo 单测 |
| `workspace_enabled` / `seat_issued` / `sandbox_executed` / `result_limited_exported` | `expire_sandbox` | `expired` | 保持原值 | 到期关闭 | `trade014`, repo 单测 |
| `workspace_enabled` / `seat_issued` / `sandbox_executed` / `result_limited_exported` / `expired` | `revoke_sandbox` | `revoked` | 保持原值 | 主动吊销 | `trade014` |

禁止样例：`expired -> execute_sandbox_query`，应返回 `SBX_STD_TRANSITION_FORBIDDEN`。证据：repo 单测 `expired_cannot_execute_again`。

### 5.8 RPT_STD

正向主链路：`created -> report_task_created -> report_generated -> report_delivered -> accepted -> settled`

| 当前状态 | 动作 | 目标状态 | payment_status 结果 | 关键说明 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `created` / `contract_pending` / `contract_effective` | `create_report_task` | `report_task_created` | 保持原值 | 创建报告任务 | `trade015`, repo 单测 |
| `report_task_created` | `generate_report` | `report_generated` | 保持原值 | 供方生成报告 | `trade015`, repo 单测 |
| `report_generated` | `deliver_report` | `report_delivered` | 保持原值 | 交付报告 | `trade015`, repo 单测 |
| `report_delivered` | `accept_report` | `accepted` | 保持原值 | 买方验收通过 | `trade015`, repo 单测 |
| `accepted` | `settle_report` | `settled` | `paid` | 完成结算 | `trade015`, repo 单测 |

禁止样例：`settled -> generate_report`，应返回 `RPT_STD_TRANSITION_FORBIDDEN`。证据：repo 单测 `settled_cannot_generate_again`。

## 6. 建议执行顺序

1. 先跑 8 个 SKU 的正向 smoke：`TRADE-008` 到 `TRADE-015`。
2. 再看跨 SKU 门禁与保护：`TRADE-021`（锁资前校验）、`TRADE-024`（晚到回调不回退）。
3. 最后看链路串联与断权：`TRADE-018`、`TRADE-027`。

## 7. 最小联调模板

```text
执行日期：
执行人：
环境：local/core
request_id：
SKU：
order_id：
当前状态：
动作：
HTTP 结果：
返回 current_state：
返回 payment_status：
审计 action_name：
结论：pass/fail
```
