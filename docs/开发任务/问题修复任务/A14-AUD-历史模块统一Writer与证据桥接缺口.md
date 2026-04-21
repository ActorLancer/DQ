# A14 AUD 历史模块统一 Writer 与证据桥接缺口

## 0. 当前状态

- 本文当前角色：`AUD` 历史模块 writer 收口与证据桥接的治理说明。
- 当前统一模型、历史模块收口任务与桥接方向已经回写到执行源；具体代码改造仍由 `AUD-001 / 002 / 029` 后续实现批次承接。
- 第 `2` 节与第 `4` 节保留问题发现时的历史起点，不再直接代表当前模块接入状态。

## 1. 任务定位

- 问题编号：`A14`
- 严重级别：`high`
- 关联阶段：`AUD`
- 关联任务：`AUD-001`、`AUD-002`、`AUD-029`
- 处理方式：先收口统一 `AuditEvent / EvidenceItem / EvidenceManifest` 模型，再让已完成阶段的历史模块接入统一 `audit writer / evidence writer`，并桥接旧的 `support.evidence_object`

## 2. 历史问题起点（归档）

问题发现时，仓库中已完成阶段的若干模块已经在写审计事件或证据对象，但这些写法并没有收敛到统一 `audit` 域权威模型，而是出现了“同名域，不同权威表 / 不同写法”的历史分叉。

问题发现时已确认的典型现象包括：

1. `catalog` 侧仍直接手写 `audit.audit_event`
2. `search` 侧仍以 ad-hoc SQL 方式写入审计事件，并在 `ref_id` 缺失时临时生成兜底 UUID
3. `billing dispute` 侧把证据对象写入 `support.evidence_object`，没有正式进入 `audit.evidence_item / audit.evidence_manifest`

这意味着：

- 审计域虽然已经有正式权威模型和强化 schema，但历史模块仍在绕过它
- 后续 `audit export / replay / legal hold / access audit` 很难建立单一联查口径
- 即使对象已经存在于 `PostgreSQL + 对象存储`，也可能不在 `audit` 权威模型里

## 3. 正确冻结口径

应以审计专题 PRD、审计接口协议和 `055_audit_hardening.sql` 为冻结基线，明确以下口径：

1. `AuditEvent`、`EvidenceItem`、`EvidenceManifest` 是审计域正式权威对象
2. 关键事务应遵循“主对象 + 审计 + outbox”同口径写入，不再允许业务模块继续 ad-hoc 直写第二套审计语义
3. 证据对象原文应进入 `PostgreSQL + 对象存储`，对象存储不是唯一权威，日志更不能替代证据权威对象
4. 历史模块必须通过统一 `audit writer / evidence writer` 落到正式模型
5. 若 `support.evidence_object` 出于兼容原因短期保留，也必须存在到 `audit.evidence_item / audit.evidence_manifest` 的明确桥接或映射，不能继续作为并行权威表

## 4. 历史问题证据（归档）

问题处理前已核对的典型证据包括但不限于：

- [服务清单与服务边界正式版.md](/home/luna/Documents/DataB/docs/开发准备/服务清单与服务边界正式版.md)
  - 已冻结审计域中的证据对象、证据包与对象存储相关边界
- [审计、证据链与回放设计.md](/home/luna/Documents/DataB/docs/原始PRD/审计、证据链与回放设计.md)
  - 已冻结 `EvidenceItem / EvidenceManifest` 以及 `PG + 对象存储` 的正式语义
- [055_audit_hardening.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/055_audit_hardening.sql)
  - 已落地 `audit.evidence_item / audit.evidence_manifest / audit.evidence_manifest_item`
- [catalog/api/support.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/catalog/api/support.rs)
  - 当前仍直接手写 `audit.audit_event`
- [search/repo/mod.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/search/repo/mod.rs)
  - 当前仍存在 ad-hoc 审计写入与 `ref_id` 兜底逻辑
- [billing/repo/dispute_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/billing/repo/dispute_repository.rs)
  - 当前把证据对象写入 `support.evidence_object`

## 5. 任务目标

通过 `AUD-001`、`AUD-002`、`AUD-029` 完成以下收口：

1. 审计与证据对象只有一套正式权威模型
2. 历史模块不再各写各的审计 SQL 或私有证据表
3. 旧的 `support.evidence_object` 至少有明确桥接路径
4. 后续 `audit export / replay / legal hold / access audit` 可基于统一 authority model 联查

## 6. 强约束

1. 不能只补 `audit-kit` 模型，不收敛历史模块写入点
2. 不能继续新增 ad-hoc `INSERT audit.audit_event` 之类的业务侧直写 SQL
3. 不能让新的证据对象只落对象存储，不进入正式 `audit.evidence_item / evidence_manifest`
4. 不能让 `support.evidence_object` 长期作为与 `audit` 并行的第二套权威表
5. 不能把“日志中能看到”误当成“证据对象已经进入正式审计域”

## 7. 建议修复方案

### 7.1 先完成统一模型基座

先通过 `AUD-001`、`AUD-002` 收口：

- `AuditEvent`
- `EvidenceItem`
- `EvidenceManifest`

并为统一 `audit writer / evidence writer` 提供稳定模型。

### 7.2 再统一历史模块写入入口

通过 `AUD-029` 把下列历史模块接入统一写入器：

- `catalog`
- `search`
- `billing dispute`

要求：

- 后续新增写入路径都必须经统一 writer
- 不再允许业务模块继续手写第二套审计语义

### 7.3 桥接旧证据表

对 `support.evidence_object` 至少明确以下一种正式处理方式：

- 迁移到 `audit.evidence_item / evidence_manifest`
- 保留兼容表，但建立明确映射并以 `audit` 权威对象为主查询口径

无论采用哪种方式，都不能再让旧表独立承担证据权威职责。

## 8. 实施范围

至少覆盖以下内容：

### 8.1 统一模型与写入器

- `apps/platform-core/crates/audit-kit/**`
- `apps/platform-core/src/modules/audit/**`

### 8.2 历史模块收口

- `apps/platform-core/src/modules/catalog/**`
- `apps/platform-core/src/modules/search/**`
- `apps/platform-core/src/modules/billing/**`

### 8.3 schema 与桥接

- `docs/数据库设计/V1/upgrade/055_audit_hardening.sql`
- 旧证据表到正式审计表的桥接策略与运行说明

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 历史模块不再直接扩散新的 ad-hoc 审计 SQL
- 新的证据对象写入路径有统一 authority model
- `support.evidence_object` 与 `audit.evidence_item / evidence_manifest` 的关系已被正式定义

### 9.2 联查一致性

应能明确证明：

1. 历史模块产生的审计事件能进入统一 `AuditEvent`
2. 历史模块产生的证据对象能在统一 `EvidenceItem / EvidenceManifest` 中联查
3. `PG + 对象存储` 中的对象不再游离于审计 authority model 之外

### 9.3 后续能力可承接

修复后应能直接支撑：

- `AUD-004` 证据包导出
- `AUD-005` 回放任务
- `AUD-006` legal hold
- `AUD-029` 历史模块收口后的统一联查

若仍存在“对象在库里、文件在对象存储里，但导出/联查看不到”的情况，则视为收口不完整。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 历史模块改造清单
2. 统一 `audit writer / evidence writer` 接入点清单
3. `support.evidence_object -> audit.evidence_item / evidence_manifest` 的桥接方案
4. 跨域联查验证结果

## 11. 一句话结论

`A14` 的核心问题不是“旧模块有没有写审计”，而是旧模块虽然已经在写审计和证据，但并没有收口到统一 `audit` 权威模型；如果不通过 `AUD-029` 把历史写入点和旧证据表桥回正式模型，后续 `AUD` 阶段会一直面对“同名域，不同权威表”的历史分叉。
