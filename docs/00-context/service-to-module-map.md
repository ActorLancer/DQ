# 服务名到模块映射（CTX-019）

本文件冻结“技术选型/设计文档中的服务名”到当前 `V1-Core` 运行形态的映射，避免不同实现批次各自按不同拆分方式重建服务。

## 映射规则

- `V1-Core` 主路径采用：`platform-core` 模块化单体 + 外围独立进程。
- 同步事务边界默认在 `platform-core` 内部完成（DB 事务为准）。
- 跨进程副作用默认走异步链路（`outbox_event -> publisher -> ops.event_route_policy 对应 Kafka topic -> worker/adapter`）。
- `V2/V3` 能力当前仅允许接口/trait 占位，不在本映射中落正式运行时实现。

## 服务映射表（冻结）

| 设计服务名 | V1-Core 归属 | 同步边界 | 异步边界 | 所有权 |
| --- | --- | --- | --- | --- |
| `iam-service` | `platform-core::iam + party + access` | 身份、主体、权限写入在 `platform-core` 内部事务完成 | 审计/通知事件通过 outbox 异步分发 | `platform-core` |
| `trade-service` | `platform-core::order + contract + authorization + delivery` | 下单、签约、授权、交付主链路在 `platform-core` 内部推进 | 链回执、通知、索引同步走异步链路 | `platform-core` |
| `catalog-service` | `platform-core::catalog + contract_meta + listing + review` | 商品、模板、上架、审核写入在 `platform-core` 内部事务完成 | 搜索索引刷新、推荐信号写入走异步链路 | `platform-core` |
| `billing-service` | `platform-core::billing + dispute` | 账务状态、争议状态在 `platform-core` 内部推进 | 支付结果回调、账务事件发布异步化 | `platform-core` |
| `audit-service` | `platform-core::audit + consistency` | 审计事件入库、一致性登记在 `platform-core` 同步完成 | 审计锚定与外部告警异步化 | `platform-core` |
| `search-service` | `platform-core::search`（查询放行） + `apps/search-indexer`（索引构建） | 查询请求最终放行在 `platform-core` 校验 | 索引构建/刷新由 `search-indexer` 消费事件执行 | `platform-core` + `search-indexer` |
| `recommendation-service` | `platform-core::recommendation`（策略与放行） + 外围 worker（离线计算） | 推荐结果最终放行在 `platform-core` | 行为流与离线计算异步处理 | `platform-core` + worker |
| `notification-worker` | `apps/notification-worker`（独立进程） + `platform-core::audit`（通知 ops facade） | 对外控制面由 `platform-core` 同步承接；worker 无主交易同步写边界 | 消费 `dtp.notification.dispatch` 发送站内信、邮件、Webhook，并通过 `/internal/notifications/*` 承接 `platform-core` 的通知联查 / replay 转发 | `platform-core` + `notification-worker` |
| `fabric-adapter-service` | `apps/fabric-adapter`（独立进程） | 无主交易同步写边界 | 消费 `dtp.audit.anchor` / `dtp.fabric.requests`，回写链回执 | `fabric-adapter` |
| `outbox-publisher-service` | `workers/outbox-publisher`（独立进程） | 无主交易同步写边界 | 轮询 outbox 并发布 Kafka | `outbox-publisher` |

## 约束

- 禁止在未更新本文件的情况下新增“同名新服务”或把既有服务从单体内随意拆出。
- 若后续批次调整映射，必须同时更新：
  - `docs/01-architecture/service-runtime-map.md`（运行时拓扑）
  - 相关 compose/runbook 文档
  - 对应任务实施日志
