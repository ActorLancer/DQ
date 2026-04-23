# OpenAPI 分域说明（BOOT-007）

本目录按领域拆分当前实现阶段使用的 OpenAPI 设计参考与版本对象。

- `packages/openapi/*.yaml` 是当前实现期的设计参考与变更落点。
- `docs/02-openapi/*.yaml` 只承接实现校验后的归档副本或归档占位，不作为当前实现期权威源。
- 当前阶段约束、归档策略和后续补齐义务以 [docs/02-openapi/README.md](../../docs/02-openapi/README.md) 为准。
- `AUD-027` 之后，`packages/openapi/{audit,ops}.yaml` 需要同时满足：
  - 与 `docs/02-openapi/{audit,ops}.yaml` 逐字同步
  - 被 README / 索引显式引用
  - 通过 `./scripts/check-openapi-schema.sh` 对当前 audit / ops / developer 已落地路径和关键术语的校验

当前子域成熟度：

- `iam.yaml`：IAM/Party/Access 领域 V1 当前接口参考；`WEB-001` 已补齐 `GET /api/v1/auth/me` 的 `SessionContextView` / `ApiResponseSessionContextView` 正式响应体，并与 `docs/02-openapi/iam.yaml` 保持逐字同步，由 `./scripts/check-openapi-schema.sh` 对该接口做最小防漂移校验。
- `catalog.yaml`：Catalog/Review/Support 领域 V1 当前接口参考；`WEB-001` 已补齐 `GET /api/v1/catalog/standard-scenarios` 的正式 `ApiResponse` 包装响应，并与 `docs/02-openapi/catalog.yaml` 保持逐字同步，由 `./scripts/check-openapi-schema.sh` 对该接口做最小防漂移校验。
- `trade.yaml`：Order/Contract/Authorization 主交易链路 V1 当前接口参考。
- `billing.yaml`：Billing/Payment/Settlement/Dispute 子域 V1 当前接口参考。
- `delivery.yaml`：Delivery/Storage/Query Execution 子域 V1 当前接口参考。
- `search.yaml`：`AUD-022 + SEARCHREC-016` 已补齐并归档 Search/Ops Search 的正式 Bearer 鉴权、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、正式权限点、审计/访问日志副作用、`SEARCH_*` 错误码与 request/response schema；当前文件与 `docs/02-openapi/search.yaml` 保持逐字同步，并由 `./scripts/check-openapi-schema.sh` 强校验。
- `recommendation.yaml`：`SEARCHREC-016` 已补齐并归档 Recommendation/Ops Recommendation 的正式 Bearer 鉴权、正式权限点、`X-Idempotency-Key`、必要 `X-Step-Up-Token`、审计/访问日志副作用与 request/response schema；当前文件与 `docs/02-openapi/recommendation.yaml` 保持逐字同步，并由 `./scripts/check-openapi-schema.sh` 强校验。
- `audit.yaml`：`AUD-027` 已把 `AUD-003 ~ AUD-007` 的订单审计联查、全局 trace 查询、证据包导出、replay dry-run、legal hold 创建 / 释放、anchor batch 查看 / 重试契约收口到正式归档，并要求和 `docs/02-openapi/audit.yaml` 保持逐字同步。
- `ops.yaml`：`AUD-027` 已把健康检查、内部开发端点、`NOTIF-013` 的通知控制面，以及 `AUD-008` 的 canonical outbox / dead letter 查询、`AUD-010` 的 dead letter dry-run reprocess、`AUD-011/012` 的一致性联查 / dry-run reconcile、`AUD-018` 的 trade monitor 总览 / checkpoints、`AUD-019` 的 external facts 查询 / confirm、`AUD-020` 的 fairness incidents 查询 / handle、`AUD-021` 的 projection gaps 查询 / resolve、`AUD-023` 的观测总览 / 日志镜像查询导出 / trace 联查 / 告警与 incident / SLO 查询、`AUD-024` 的 `GET /api/v1/developer/trace` 开发者状态联查一起收口到正式归档；`AUD-022` 的搜索运维控制面则继续由 `search.yaml` 正式承接。

通知、Fabric 及相关联查/重试/DLQ 的契约补齐，不代表会新建独立 `packages/openapi/notification.yaml` 或 `fabric.yaml`；现阶段统一由 `audit.yaml`、`ops.yaml` 及 `docs/02-openapi/README.md` 中的阶段约束承接。

`merge-openapi.sh` 用于生成聚合输出占位与本地校验输入；这不代表所有子域都处于同一成熟度，也不代表 `audit/ops` 已完成实现期契约落盘。
