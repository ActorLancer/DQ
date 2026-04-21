# A03 统一事务模板落地真实审计与 Outbox Writer

## 1. 任务定位

- 问题编号：`A03`
- 严重级别：`high`
- 关联阶段：`cross-stage`
- 关联任务：`AUD-001`、`AUD-008`、`AUD-025`、`AUD-029`
- 处理方式：修复统一事务基座被 `NoopAuditWriter / NoopOutboxWriter` 架空的问题，让框架层真正承担“主对象 + 审计 + outbox”统一事务能力

## 2. 问题描述

当前框架层已经存在统一事务模板和统一执行入口，但运行时默认绑定的是 no-op writer，导致事务基座在关键路径上并未真正承担正式能力。

当前已确认的典型现象：

1. `db` crate 已提供 `TransactionBundle` 和统一事务模板
2. 启动时默认仍绑定 `NoopAuditWriter`
3. 启动时默认仍绑定 `NoopOutboxWriter`
4. `audit-kit` 和 `outbox-kit` 本身仍主要停留在 no-op 形态

这意味着：

- 框架层能力“看起来存在”，但运行时默认不落真实数据
- 后续模块会继续各自手写 SQL，而不是复用统一模板
- 无法通过统一事务模板收口审计、幂等、路由、权限和一致性语义

## 3. 正确冻结口径

以 [服务清单与服务边界正式版.md](/home/luna/Documents/DataB/docs/开发准备/服务清单与服务边界正式版.md) 为冻结基线：

在 `platform-core` 内，关键业务动作必须在同一业务事务内同时写入：

- 主业务对象
- 审计事件
- outbox 事件

并且：

- 不允许只写主对象不写审计
- 不允许只发事件不落主对象
- 应存在统一、可复用、可审计的正式路径，而不是每个模块自己拼装

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [服务清单与服务边界正式版.md](/home/luna/Documents/DataB/docs/开发准备/服务清单与服务边界正式版.md)
  - 明确要求关键业务动作同事务写主对象、审计、outbox
- [db/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/crates/db/src/lib.rs)
  - 已存在 `TransactionBundle` 与统一执行入口
- [lib.rs](/home/luna/Documents/DataB/apps/platform-core/src/lib.rs)
  - 启动时默认绑定 `NoopAuditWriter` / `NoopOutboxWriter`
- [audit-kit/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/crates/audit-kit/src/lib.rs)
  - 仍保留 no-op writer 形态
- [outbox-kit/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/crates/outbox-kit/src/lib.rs)
  - 仍保留 no-op writer 形态

## 5. 任务目标

让统一事务模板从“框架占位”变成“运行时正式能力”，确保：

1. 框架层默认绑定真实持久化 writer，而不是 no-op
2. 审计写入与 outbox 写入可以在统一事务模板中落地
3. 后续模块能够优先复用统一事务模板，而不是继续各写各的 SQL
4. 为 `AUD / NOTIF / SEARCHREC` 后续统一事件、统一审计、一致性与权限收口提供真正基座

## 6. 强约束

1. 不能只改文档，不改运行时绑定
2. 不能只把 no-op 改成另一个空壳 trait 实现
3. 不能只给新模块用统一模板，老的关键路径仍全部放任手写
4. 不能让“统一事务模板”继续停留在编译可用但运行时无效果的状态
5. 修复后必须明确哪些关键路径优先切到统一模板

## 7. 建议修复方案

### 7.1 替换运行时默认 writer

应将：

- `NoopAuditWriter`
- `NoopOutboxWriter`

替换为真实持久化实现，使默认运行时行为与冻结架构一致。

### 7.2 保留 no-op 但降级为测试/占位用途

`noop` 可以保留，但只能用于：

- 特定单元测试
- 显式 mock 场景
- 特殊 dry-run / 隔离环境

不能继续作为正式应用启动默认值。

### 7.3 明确统一事务模板的正式使用边界

至少要冻结：

- 哪类写路径必须优先走统一模板
- 哪些遗留路径暂时允许保留手写，但要列入迁移清单

优先建议覆盖：

- 订单关键状态写入
- 审计强制动作
- outbox 生产主路径
- 搜索/通知/Fabric 相关上游事件生成路径

### 7.4 输出迁移清单

修复时不应只替换默认绑定，还应列出：

- 当前哪些模块已接入统一事务模板
- 哪些模块仍为遗留手写路径
- 哪些路径必须在后续阶段优先迁移

## 8. 实施范围

至少覆盖以下内容：

### 8.1 框架层

- `apps/platform-core/crates/db/**`
- `apps/platform-core/crates/audit-kit/**`
- `apps/platform-core/crates/outbox-kit/**`

### 8.2 运行时装配层

- `apps/platform-core/src/lib.rs`
- `AppState / AppDb / tx template` 相关绑定

### 8.3 业务落点

- 与审计主链、outbox 主链直接相关的关键写路径
- 后续 `AUD / SEARCHREC / NOTIF` 依赖的上游事件生产路径

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 正式运行时默认不再绑定 `NoopAuditWriter`
- 正式运行时默认不再绑定 `NoopOutboxWriter`
- `audit-kit` / `outbox-kit` 存在真实持久化实现并被正式装配

### 9.2 动态验证

至少验证一条关键业务写路径：

1. 主对象成功写入
2. 审计事件在同事务内写入
3. outbox 事件在同事务内写入
4. 事务失败时三者均不落库

### 9.3 迁移可执行性

必须输出一份明确迁移清单，说明：

- 已切换到统一事务模板的路径
- 仍未切换的遗留路径
- 后续阶段优先迁移顺序

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 真实 writer 的运行时装配方式
3. 保留的 noop 边界说明
4. 已接入统一事务模板的路径清单
5. 尚未迁移路径清单
6. 测试与联调结果

## 11. 一句话结论

`A03` 的核心不是“有无事务模板”，而是“统一事务模板是否在正式运行时真正落地为主对象 + 审计 + outbox 的权威路径”；当前默认 no-op 使这套基座事实上被架空，必须收口。
