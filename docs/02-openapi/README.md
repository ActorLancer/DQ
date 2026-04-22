# 02-openapi

用于收敛 OpenAPI 归档、接口分组说明和契约校验输出。

当前实现阶段的 OpenAPI 设计参考以 `packages/openapi/*.yaml` 为准。
`packages/openapi/README.md` 负责说明当前子域成熟度与实现期使用方式。
本目录只保留实现校验后的归档副本或归档占位，不作为当前实现期权威源。

约束：

- 当前批次若只做“口径收缩/命名收敛/事件拓扑冻结”，不要求补齐尚未落地模块的 OpenAPI 细节。
- 但这不代表相关 OpenAPI 已完成；一旦进入对应模块的代码实现批次，Agent 必须同步补齐 `packages/openapi/*.yaml` 与本目录归档副本，不能把“文档口径已冻结”误报为“契约已落盘”。
- 对于 `notification.requested / audit.anchor_requested / fabric.proof_submit_requested` 这类已冻结事件，后续代码实现时必须同步补齐：
  - `packages/openapi/audit.yaml`
  - `packages/openapi/ops.yaml`
  - `docs/02-openapi/audit.yaml`
  - `docs/02-openapi/ops.yaml`
  - 相关请求/响应示例、`event_type / target_topic / aggregate_type` 过滤口径

- `iam.yaml`：IAM/Party/Access 领域 V1 接口归档（与 `packages/openapi/iam.yaml` 同步）。
- `billing.yaml`：Billing/Payment/Settlement/Dispute 子域 V1 接口归档（与 `packages/openapi/billing.yaml` 同步）。
- `trade.yaml`：Order/Contract/Authorization 主交易链路 V1 接口归档（与 `packages/openapi/trade.yaml` 同步）。
- `delivery.yaml`：Delivery/Storage/Query Execution 子域 V1 接口归档（与 `packages/openapi/delivery.yaml` 同步）。
- `search.yaml`：Search/Ops Search 子域归档占位；当前实现期唯一设计参考为 `packages/openapi/search.yaml`，待实现校验通过后再同步归档。
- `recommendation.yaml`：Recommendation/Ops Recommendation 子域 V1 接口归档（与 `packages/openapi/recommendation.yaml` 同步）。
- `audit.yaml`：已补齐 `AUD-003` 的订单审计联查 / 全局 `audit trace` 查询，以及 `AUD-004` 的证据包导出契约；回放、`legal hold`、锚定批次等剩余高风险控制面仍待后续 `AUD` 批次继续补齐。
- `ops.yaml`：当前已同步归档健康检查、内部开发端点，以及 `NOTIF-013` 承接的通知联查 / 模板预览 / 人工补发与重试-DLQ 相关契约与示例；更广义的 `AUD / consistency` 正式控制面接口仍待后续批次补齐。
