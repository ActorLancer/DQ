# OpenAPI 分域说明（BOOT-007）

本目录按领域拆分当前实现阶段使用的 OpenAPI 设计参考与版本对象。

- `packages/openapi/*.yaml` 是当前实现期的设计参考与变更落点。
- `docs/02-openapi/*.yaml` 只承接实现校验后的归档副本或归档占位，不作为当前实现期权威源。
- 当前阶段约束、归档策略和后续补齐义务以 [docs/02-openapi/README.md](../../docs/02-openapi/README.md) 为准。

当前子域成熟度：

- `iam.yaml`：IAM/Party/Access 领域 V1 当前接口参考。
- `catalog.yaml`：Catalog/Review/Support 领域 V1 当前接口参考。
- `trade.yaml`：Order/Contract/Authorization 主交易链路 V1 当前接口参考。
- `billing.yaml`：Billing/Payment/Settlement/Dispute 子域 V1 当前接口参考。
- `delivery.yaml`：Delivery/Storage/Query Execution 子域 V1 当前接口参考。
- `search.yaml`：Search/Ops Search 子域当前实现期设计参考；实现校验通过后再同步归档到 `docs/02-openapi/search.yaml`。
- `recommendation.yaml`：Recommendation/Ops Recommendation 子域当前实现期设计参考。
- `audit.yaml`：已补齐 `AUD-003` 的订单审计联查 / 全局 `audit trace` 查询、`AUD-004` 的证据包导出、`AUD-005` 的 replay dry-run、`AUD-006` 的 legal hold 创建 / 释放，以及 `AUD-007` 的 anchor batch 查看 / 重试契约；后续仅剩 Fabric callback / reconcile / dead letter reprocess / 一致性修复等高风险控制面继续补齐。
- `ops.yaml`：当前已归档健康检查、内部开发端点，以及 `NOTIF-013` 承接的通知模板预览 / 手工注入 / 审计联查 / dead letter replay 契约；更广义的 `AUD / consistency` 正式控制面接口仍待后续批次补齐。

通知、Fabric 及相关联查/重试/DLQ 的契约补齐，不代表会新建独立 `packages/openapi/notification.yaml` 或 `fabric.yaml`；现阶段统一由 `audit.yaml`、`ops.yaml` 及 `docs/02-openapi/README.md` 中的阶段约束承接。

`merge-openapi.sh` 用于生成聚合输出占位与本地校验输入；这不代表所有子域都处于同一成熟度，也不代表 `audit/ops` 已完成实现期契约落盘。
