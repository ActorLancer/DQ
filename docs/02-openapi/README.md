# 02-openapi

用于收敛 OpenAPI 归档、接口分组说明和契约校验输出。

当前实现阶段的 OpenAPI 设计参考以 `packages/openapi/*.yaml` 为准。
`packages/openapi/README.md` 负责说明当前子域成熟度与实现期使用方式。
本目录只保留实现校验后的归档副本或归档占位，不作为当前实现期权威源。

`AUD-027` 之后，`audit.yaml` 与 `ops.yaml` 的最小要求固定为：

- `packages/openapi/{audit,ops}.yaml` 与 `docs/02-openapi/{audit,ops}.yaml` 必须逐字同步
- `./scripts/check-openapi-schema.sh` 必须覆盖当前 `apps/platform-core/src/modules/audit/api/router.rs` 已落地的 audit / ops / developer 路径与关键术语
- README 索引必须明确说明这两份文件已经进入正式归档与实现对齐校验，不再只是“后续待补”

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
- `search.yaml`：`AUD-022` 已同步归档 Search/Ops Search 的正式契约，覆盖 Bearer 鉴权、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、`SEARCH_*` 错误码与 request/response schema。
- `recommendation.yaml`：Recommendation/Ops Recommendation 子域 V1 接口归档（与 `packages/openapi/recommendation.yaml` 同步）。
- `audit.yaml`：`AUD-027` 已正式归档并校验 `AUD-003 ~ AUD-007` 的订单审计联查、全局 trace 查询、证据包导出、replay dry-run、legal hold 创建 / 释放、anchor batch 查看 / 重试契约；当前文件与 `packages/openapi/audit.yaml` 保持逐字同步，并由 `./scripts/check-openapi-schema.sh` 校验已落地路径与关键术语。
- `ops.yaml`：`AUD-027` 已正式归档并校验健康检查、内部开发端点、`NOTIF-013` 的通知联查 / 模板预览 / 人工补发与重试-DLQ 契约，以及 `AUD-008` 的 canonical outbox / dead letter 查询、`AUD-010` 的 dead letter dry-run reprocess、`AUD-011/012` 的一致性联查 / dry-run reconcile、`AUD-018` 的 trade monitor 总览 / checkpoints、`AUD-019` 的 external facts 查询 / confirm、`AUD-020` 的 fairness incidents 查询 / handle、`AUD-021` 的 projection gaps 查询 / resolve、`AUD-023` 的观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO 查询，以及 `AUD-024` 的 `GET /api/v1/developer/trace` 开发者状态联查；`AUD-022` 的搜索运维控制面继续由 `search.yaml` 正式承接。
