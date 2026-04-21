# A04 AUD/Ops 接口与契约落地缺口

## 1. 任务定位

- 问题编号：`A04`
- 严重级别：`blocker`
- 关联阶段：`AUD`
- 关联任务：`AUD-003` 至 `AUD-028`
- 处理方式：进入 `AUD` 代码实现批次后，必须把 `audit / ops / consistency / developer` 的正式入口、契约、OpenAPI、测试与 runbook 补成闭环，再谈“阶段完成”

## 1.1 当前批次边界

如果当前批次的目标仅是“收缩文档口径、冻结事件拓扑、消除命名冲突”，则：

- 可以先修正文档设计冲突
- 可以先补 `topic / producer / consumer / consumer_group / route-policy seed` 的冻结口径
- 但不能把尚未实现的 `OpenAPI / 测试样例 / 集成测试` 伪装成已完成

明确要求：

- 本批次允许不补 `packages/openapi/audit.yaml`、`packages/openapi/ops.yaml` 的完整接口内容
- 本批次允许不补 `docs/05-test-cases/audit-consistency-cases.md`
- 但必须在文档中显式声明这些内容属于后续代码实现批次的必补交付物

## 2. 问题描述

`AUD/Ops` 领域当前仍停留在空骨架或半骨架状态。虽然任务清单、接口协议和阶段要求已经冻结，但运行时入口、OpenAPI、测试用例和 runbook 大面积缺失。

当前已确认的典型现象：

1. `audit` 模块仍为空壳
2. 主 router 未挂载 `audit / ops / consistency / developer` 正式路由
3. `packages/openapi/audit.yaml` 仍为空
4. `packages/openapi/ops.yaml` 只剩 `health/internal`
5. `docs/02-openapi/` 下缺失 `audit.yaml / ops.yaml`
6. `docs/05-test-cases/` 下缺失 `audit-consistency-cases.md`

这意味着：

- `AUD` 阶段的大量任务没有可执行入口
- “实现完成”缺少接口层和契约层证明
- OpenAPI 与实现无法做一致性校验
- runbook、测试矩阵、联调脚本无从建立

## 3. 正确冻结口径

以任务清单和接口协议为基线，`AUD` 阶段至少应覆盖以下正式接口族：

### 3.1 audit 域

- `GET /api/v1/audit/orders/{id}`
- `GET /api/v1/audit/traces`
- `POST /api/v1/audit/packages/export`
- `POST /api/v1/audit/replay-jobs`
- `GET /api/v1/audit/replay-jobs/{id}`
- `POST /api/v1/audit/legal-holds`
- `POST /api/v1/audit/legal-holds/{id}/release`
- `GET /api/v1/audit/anchor-batches`
- `POST /api/v1/audit/anchor-batches/{id}/retry`

### 3.2 ops / consistency 域

- `GET /api/v1/ops/outbox`
- `GET /api/v1/ops/dead-letters`
- `POST /api/v1/ops/dead-letters/{id}/reprocess`
- `GET /api/v1/ops/consistency/{refType}/{refId}`
- `POST /api/v1/ops/consistency/reconcile`
- `GET /api/v1/ops/external-facts`
- `POST /api/v1/ops/external-facts/{id}/confirm`
- `GET /api/v1/ops/projection-gaps`
- `POST /api/v1/ops/projection-gaps/{id}/resolve`
- `GET /api/v1/ops/search/sync`
- `POST /api/v1/ops/search/reindex`
- `POST /api/v1/ops/search/cache/invalidate`

### 3.3 developer / observability / trade-monitor 域

- `GET /api/v1/developer/trace`
- `GET /api/v1/ops/observability/overview`
- `GET /api/v1/ops/trade-monitor/orders/{orderId}`
- `GET /api/v1/ops/trade-monitor/orders/{orderId}/checkpoints`

同时要求：

- OpenAPI 必须存在并与实现校验
- runbook 必须存在并说明排障/重放/导出/修复流程
- 测试用例文档和集成测试必须同步收口

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [v1-core-开发任务清单.md](/home/luna/Documents/DataB/docs/开发任务/v1-core-开发任务清单.md)
  - `AUD-003` 至 `AUD-028` 已冻结接口、文档、测试要求
- [审计、证据链与回放接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md)
  - 已冻结 `audit` 域正式接口
- [一致性与事件接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/一致性与事件接口协议正式版.md)
  - 已冻结 `ops/outbox`、`dead letter`、`consistency` 接口
- [apps/platform-core/src/modules/audit/mod.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/audit/mod.rs)
  - 仍为空壳
- [apps/platform-core/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/src/lib.rs)
  - 主 router 未挂载 `audit / ops / consistency / developer`
- [packages/openapi/audit.yaml](/home/luna/Documents/DataB/packages/openapi/audit.yaml)
  - 基本为空
- [packages/openapi/ops.yaml](/home/luna/Documents/DataB/packages/openapi/ops.yaml)
  - 只剩 `health/internal`

## 5. 任务目标

补齐 `AUD/Ops` 领域的最小正式闭环，确保：

1. `audit / ops / consistency / developer` 有可执行 API 入口
2. OpenAPI 不再为空壳
3. `docs/02-openapi` 与实现同步
4. `docs/05-test-cases` 有正式 `audit-consistency-cases.md`
5. `runbook` 能指导导出、回放、dead-letter 重处理与一致性排障

## 6. 强约束

1. 不能只补 router，不补 OpenAPI
2. 不能只补 OpenAPI，不补实现
3. 不能只补接口，不补 runbook / 测试用例文档
4. 不能把 `AUD` 阶段继续解释成“后面再补”，因为任务和协议已经冻结
5. 不能只做 `health/internal` 级别的占位路由来冒充阶段完成

## 7. 建议修复方案

### 7.1 先补最小正式模块骨架

至少建立并接入：

- `audit`
- `consistency`
- `ops`
- `developer`

要求：

- 模块目录、DTO、handler、router、service/repo 分层清晰
- 主应用 router 正式挂载

### 7.2 先补契约，再补实现细节

优先顺序建议：

1. `packages/openapi/audit.yaml`
2. `packages/openapi/ops.yaml`
3. `docs/02-openapi/audit.yaml`
4. `docs/02-openapi/ops.yaml`
5. 与实现做路由对齐校验

补充说明：

- 若当前只做文档口径收缩，则这里的 `OpenAPI` 只允许登记为“后续必补项”，不能伪造完成态。
- 一旦开始 `audit / consistency / integration` 的代码实现，这一组文件必须和路由、DTO、权限、step-up、事件示例同步补齐。

### 7.3 先实现最小可用查询与 dry-run 类接口

建议先以以下最小能力打通：

- `audit.traces`
- `audit.orders/{id}`
- `audit.packages/export`
- `audit.replay-jobs`
- `ops.outbox`
- `ops.dead-letters`
- `ops.consistency`
- `developer.trace`

其中高风险操作默认：

- `dry-run`
- `step-up`
- 审计留痕

### 7.4 同步补 runbook 与测试用例文档

至少新增或补齐：

- `docs/04-runbooks/` 中与 `audit / replay / dead letter / consistency / export` 相关 runbook
- `docs/05-test-cases/audit-consistency-cases.md`

补充说明：

- 当前仅做设计冲突修复时，可以先把“缺文件、待补齐阶段、补齐触发条件”明确写入 README / TODO / runbook。
- 但进入代码实现批次后，上述 runbook 与测试文件必须实补，不能继续停留在“后面再说”。

### 7.5 用集成测试证明接口不是空壳

至少需要：

- API 层 curl / integration 测试
- DB 回查
- OpenAPI 与实现一致性校验
- dry-run 与 step-up 路径断言

## 8. 实施范围

至少覆盖以下内容：

### 8.1 模块与路由

- `apps/platform-core/src/modules/audit/**`
- `apps/platform-core/src/modules/consistency/**`
- `apps/platform-core/src/modules/developer/**`
- `apps/platform-core/src/lib.rs`

### 8.2 契约

- `packages/openapi/audit.yaml`
- `packages/openapi/ops.yaml`
- `docs/02-openapi/audit.yaml`
- `docs/02-openapi/ops.yaml`

### 8.3 文档与测试

- `docs/04-runbooks/**`
- `docs/05-test-cases/audit-consistency-cases.md`
- 集成测试 / smoke / curl 联调脚本

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- `audit` 模块不再是空壳
- 主 router 已挂载 `audit / ops / consistency / developer`
- `packages/openapi/audit.yaml` 不再为空
- `packages/openapi/ops.yaml` 不再只有 `health/internal`
- `docs/02-openapi/` 存在 `audit.yaml / ops.yaml`
- `docs/05-test-cases/` 存在 `audit-consistency-cases.md`

### 9.2 动态验证

至少验证：

1. `GET /api/v1/audit/traces`
2. `POST /api/v1/audit/packages/export`
3. `POST /api/v1/audit/replay-jobs`
4. `GET /api/v1/ops/outbox`
5. `POST /api/v1/ops/dead-letters/{id}/reprocess`
6. `GET /api/v1/ops/consistency/{refType}/{refId}`
7. `GET /api/v1/developer/trace`

要求验证：

- 路由真实可访问
- 权限与 step-up 路径存在
- 审计留痕存在
- OpenAPI 与实现一致

### 9.3 阶段可证明性

修复后应能回答：

- `AUD` 阶段的正式入口是什么
- `AUD` 阶段的契约文件是什么
- `AUD` 阶段的 runbook 在哪里
- `AUD` 阶段的测试样例在哪里

若不能回答，则视为仍未完成收口。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 新增/接入的路由清单
3. OpenAPI 完整接口清单
4. 新增 runbook 清单
5. 新增测试用例文档与集成测试清单
6. API 联调与契约校验结果

## 11. 一句话结论

`A04` 的核心问题不是“口径先冻结就算完成”，而是 `AUD/Ops` 领域最终仍必须补齐正式入口、正式契约、正式测试和正式 runbook；当前若只做设计冲突修复，必须把这些交付物明确登记为后续代码实现批次的强制补齐项。
