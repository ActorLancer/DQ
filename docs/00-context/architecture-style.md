# 架构风格冻结（CTX-004）

## 1. 正式架构风格

当前阶段固定采用：

- `platform-core` 模块化单体（主业务状态与主编排）
- 外围独立进程（适配、监听、索引、通知、离线任务）
- 独立基础设施（数据库、消息、缓存、对象存储、搜索、认证、观测、联盟链）

当前阶段不采用：

- 全面微服务拆分
- 多业务主服务并行持有交易主状态

## 2. 进程边界（V1）

主应用：

- `apps/platform-core`

外围进程：

- `apps/fabric-adapter`
- `apps/fabric-event-listener`
- `apps/search-indexer`
- `apps/data-processing-worker`
- `apps/notification-worker`
- `apps/mock-payment-provider`

前端进程：

- `apps/portal-web`
- `apps/console-web`

## 3. 设计约束

1. 生命周期核心对象状态推进只能由 `platform-core` 业务规则驱动。
2. 外围进程只做“接收事件/外部回执/技术副作用”，不得成为主状态裁决者。
3. 接口、事件、数据库、审计的变更必须在同一边界口径下收敛。
4. 后续若拆分服务，需先保持模块边界稳定，再按任务清单推进，不可跳步拆分。

## 4. 与当前仓库现状一致性

- 当前仓库 `apps/` 已存在上述单体 + 外围进程目录骨架。
- 本文档冻结的是“实现与演进约束”，不是重建目录。
