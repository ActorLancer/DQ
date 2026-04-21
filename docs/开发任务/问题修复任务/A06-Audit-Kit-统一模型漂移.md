# A06 Audit-Kit 统一模型漂移

## 1. 任务定位

- 问题编号：`A06`
- 严重级别：`high`
- 关联阶段：`AUD`
- 关联任务：`AUD-001`、`AUD-002`、`AUD-004`、`AUD-005`、`AUD-006`、`AUD-007`、`AUD-029`
- 处理方式：先收口 `audit-kit` 的统一模型，使其与冻结协议和数据库 schema 对齐，再通过 `AUD-029` 收敛已完成阶段的历史写入点，最后继续补 API、导出、回放、legal hold、anchor 等能力

## 2. 问题描述

当前 `audit-kit` 所提供的统一模型过于简化，已经和正式冻结协议、数据库强化 schema 明显漂移。

当前已确认的典型现象：

1. 冻结协议已要求较完整的 `AuditTrace / AuditEvent` 字段集
2. 审计强化 migration 已落地更完整的审计字段与相关对象
3. `audit-kit` 当前模型仍只有 `action / object / result / context / evidence` 等简化字段

这意味着：

- 审计框架层无法承载正式字段语义
- 后续 API、DTO、数据库、导出、回放对象将继续各写各的模型
- 即使补了接口，也会出现“框架层模型、数据库模型、OpenAPI 模型”三套不一致

## 3. 正确冻结口径

以审计接口协议和审计强化 schema 为冻结基线，`AuditTrace / AuditEvent` 至少应覆盖以下正式字段语义：

- `event_schema_version`
- `domain_name`
- `actor_org_id`
- `error_code`
- `request_id`
- `trace_id`
- `tx_hash`
- `evidence_manifest_id`
- `event_hash`
- `occurred_at`

同时还应支持以下审计域正式对象或相关能力：

- `EvidenceItem`
- `EvidenceManifest`
- `EvidencePackage`
- `ReplayJob`
- `ReplayResult`
- `AnchorBatch`
- `LegalHold`
- `AuditAccessRecord`

以及相关关键语义：

- `step_up_challenge_id`
- hash chain
- retention / retention_class
- legal hold
- access audit

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [审计、证据链与回放接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md)
  - 已冻结审计对象与接口字段
- [055_audit_hardening.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/055_audit_hardening.sql)
  - 已落地审计强化表结构与相关字段
- [audit-kit/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/crates/audit-kit/src/lib.rs)
  - 当前统一模型仍停留在极简形态

## 5. 任务目标

将 `audit-kit` 收口为真正的审计域统一模型基座，确保：

1. `audit-kit` 字段语义与冻结协议一致
2. `audit-kit` 与数据库 schema 不再明显漂移
3. 后续 `audit API / export / replay / legal hold / anchor` 可以建立在统一模型之上
4. 避免后续模块各自定义第二套审计 DTO

## 6. 强约束

1. 不能只补 API DTO，不补 `audit-kit`
2. 不能只补数据库字段映射，不补框架层统一模型
3. 不能继续让 `audit-kit` 保持“极简占位”而由业务模块各自补充
4. 不能把 `step_up / hash chain / evidence manifest / legal hold / access audit` 继续当作后续再说的边角字段
5. 修复后应明确哪些字段是 `V1 必需`，哪些才是 `V2/V3 reserved`

## 7. 建议修复方案

### 7.1 先扩展统一审计事件模型

优先在 `audit-kit` 中补齐：

- 冻结协议要求的核心 trace / event 字段
- 与 schema 对齐的标识、摘要、时间、上下文、错误、组织维度字段

要求：

- 统一字段命名
- 避免 `API DTO / DB row / kit model` 三套命名漂移

### 7.2 把核心审计对象提升为正式模型

至少在 `audit-kit` 层显式表达：

- `EvidenceManifest`
- `ReplayJob`
- `AnchorBatch`
- `LegalHold`
- `AuditAccessRecord`

即使部分能力仍先以最小实现落地，也不应继续缺席统一模型层。

### 7.3 明确 hash chain 与 retention 语义

应在统一模型中保留正式字段/语义位置，用于承接：

- `previous_event_hash`
- `event_hash`
- `retention_class`
- `legal_hold_status`
- `sensitivity_level`

避免后续导出、回放、锚定时再重复造字段。

### 7.4 区分 V1 必需与后续预留

建议显式标注：

- 哪些字段/对象是 `V1 Active`
- 哪些字段是 `V2/V3 reserved`

这样可以避免把真正阻塞项遗漏，也避免把未来增强项误塞进当前阶段 blocker。

## 8. 实施范围

至少覆盖以下内容：

### 8.1 框架层

- `apps/platform-core/crates/audit-kit/**`

### 8.2 协议与 schema 对齐

- `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`
- `docs/数据库设计/V1/upgrade/055_audit_hardening.sql`

### 8.3 下游影响面

- audit API DTO
- export / replay / legal hold / anchor 路径
- 审计写入器
- 审计联查与访问记录
- `catalog / search / billing dispute` 等历史模块接入统一 `audit writer / evidence writer` 的改造路径

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- `audit-kit` 不再只有极简 `action/object/result/context/evidence`
- 核心冻结字段在统一模型层有正式表达
- `EvidenceManifest / ReplayJob / AnchorBatch / LegalHold / AuditAccessRecord` 至少有统一模型定义

### 9.2 一致性验证

应能明确映射：

1. `audit-kit` 字段
2. DB schema 字段
3. OpenAPI / DTO 字段

三者之间不再出现大面积断层。

### 9.3 后续阶段可承接性

修复后应能直接支撑：

- `AUD-004` 证据包导出
- `AUD-005` 回放任务
- `AUD-006` legal hold
- `AUD-007` anchor batch
- `AUD-029` 历史模块审计与证据写入收口

若仍不能支撑，则视为模型收口不完整。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. `audit-kit` 最终统一模型清单
3. `kit -> DB schema -> OpenAPI/DTO` 映射表
4. `V1 Active / reserved` 字段划分
5. 测试与对齐验证结果

## 11. 一句话结论

`A06` 的核心问题不是“审计表够不够多”，而是 `audit-kit` 这个统一基座仍停留在极简占位，无法承接冻结协议和强化 schema；如果不先收口模型，后续 API、导出、回放和锚定只会继续分叉。
