# Provider 适配边界冻结（CTX-009）

## 1. 统一原则

- 外部能力统一走 Provider 适配层，不允许在业务主流程中硬编码第三方调用。
- 每类外部能力必须同时具备 `mock` / `real` 双实现。
- `mock` 用于 `local/demo` 开发联调；`real` 仅用于受控环境接入。

## 2. 必须双实现的能力

1. `KYC/KYB`
2. 签章/签署
3. 支付与退款回调
4. 通知（短信/邮件/站内）
5. 链写入（Fabric 适配）
6. 风控外部评分/黑名单能力

## 3. 边界约束

- Provider 只返回外部事实，不得直接推进 `Order/Contract/Authorization/Delivery/Settlement/Dispute` 终态。
- 所有 Provider 回执必须带 `request_id` 与可追溯外部参考 ID，并进入审计与一致性联查。
- Provider 变更必须保持接口契约稳定，避免调用方随意感知供应商差异。
