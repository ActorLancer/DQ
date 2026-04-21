# A02 统一事件 Envelope 与路由权威源

## 0. 当前状态

- 本文当前角色：统一事件 envelope、route authority 与 canonical outbox writer 的治理说明，用于约束后续实现继续沿同一口径推进。
- 当前主链已冻结为“应用层 canonical outbox writer + `ops.event_route_policy` 运行时路由 authority”；后文第 `2` 节与第 `4` 节保留问题发现时的历史起点。
- 后续若新增事件源、topic 或兼容层，必须先更新 route policy、task 与 runbook，再补本文件的剩余边界说明。

## 1. 任务定位

- 问题编号：`A02`
- 严重级别：`blocker`
- 关联阶段：`cross-stage`
- 关联任务：`AUD-001`、`AUD-008`、`AUD-009`、`AUD-030`、`SEARCHREC-001`
- 处理方式：先收口事件 envelope 与 route authority，再统一 outbox 生产方式；不允许继续维持触发器自动派生、业务代码手工直写、绕过路由策略三套并存

## 2. 历史问题起点（归档）

当前仓库中的统一事件协议并没有真正成为唯一权威源，存在三类并行路径：

1. 数据库触发器自动写 outbox
2. 业务代码手工写 outbox
3. 完全绕过 `event_route_policy` 的路径

问题发现时已确认的典型现象包括：

- 数据库里已经建了 `ops.event_route_policy`，但应用代码并未实际使用
- `common.tg_write_outbox()` 仍按 `schema.table -> target_topic` 自动派生 topic
- `catalog` 模块存在手工插入 `search.product.changed` 的路径
- 同时 `catalog.product` 又仍挂着统一 outbox 触发器
- `order` 与 `delivery` 事件 payload 顶层仍使用 `event_name`，缺少冻结要求的正式字段

这会导致：

- 事件顶层字段不一致
- 路由来源不一致
- 同一业务动作可能双写或重复写事件
- publisher / consumer 无法只认一种协议
- 幂等、重试、DLQ、回放难以统一

## 3. 正确冻结口径

以 [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md) 为冻结基线，正式事件 envelope 顶层至少应包含：

- `event_id`
- `event_type`
- `event_version`
- `occurred_at`
- `producer_service`
- `aggregate_type`
- `aggregate_id`
- `request_id`
- `trace_id`
- `idempotency_key`
- `payload`

同时要求：

- topic / route 必须由统一策略收敛
- producer 不应在不同模块各自拼装私有 envelope
- 不允许继续依赖“按 schema.table 自动派生 topic”作为正式路由规则

## 4. 历史问题证据（归档）

问题处理前已核对的典型漂移点包括但不限于：

- [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md)
  - 冻结要求完整 envelope 顶层字段
- [056_dual_authority_consistency.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql)
  - 已定义 `ops.event_route_policy`
  - `common.tg_write_outbox()` 仍按 `schema.table -> target_topic` 自动派生
- [050_audit_search_dev_ops.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql)
  - `catalog.product` 仍挂统一触发器
- [product_and_review.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/catalog/api/handlers/product_and_review.rs)
  - `catalog` 手工插入 `search.product.changed`
- [order_create_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/order/repo/order_create_repository.rs)
  - 订单事件顶层仍使用 `event_name`
- [outbox_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/delivery/repo/outbox_repository.rs)
  - 交付事件顶层仍使用 `event_name`
- [072_canonical_outbox_route_policy.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/072_canonical_outbox_route_policy.sql)
  - 已统一 seed 当前正式 route policy、停用旧 outbox 触发器并将 `common.tg_write_outbox()` 退役为异常函数
- [apps/platform-core/src/shared/outbox.rs](/home/luna/Documents/DataB/apps/platform-core/src/shared/outbox.rs)
  - 已落地应用层 canonical outbox writer，运行时从 `ops.event_route_policy` 解析正式路由并统一构造 envelope

## 5. 任务目标

将事件生产收敛为单一、稳定、可验证的一套正式机制，确保：

1. 统一 envelope 成为唯一正式事件协议
2. 路由策略由统一权威源决定，不再散落在触发器或业务模块中
3. 清理“触发器自动派生 + 手工直写 + 绕过策略”并存状态
4. publisher、consumer、DLQ、replay、search-indexer、notification、fabric-adapter 可以只认一种正式协议

## 6. 强约束

1. 不能只改一个模块的 payload，必须统一事件生产机制
2. 不能继续保留自动派生 topic 触发器作为正式默认方案
3. 不能允许部分模块继续使用 `event_name`、部分模块使用 `event_type`
4. 不能允许 `event_route_policy` 继续存在但运行时不使用
5. 不能只在 Rust 代码里修，必须连同 DB / schema / route policy / 触发器一起收口
6. 不允许通过“让 consumer 兼容多种旧 payload”来回避收口
7. 不允许继续让 `catalog.product` 同时走触发器与手工写事件两套路径
8. 不允许 `event_route_policy` 继续只存在于数据库 schema，而不是运行时实际 authority

## 7. 建议修复方案

### 7.1 明确唯一权威源

必须先决定并落实：

- 统一 route authority 由 `ops.event_route_policy` 承担
- 统一 envelope 由应用层公共构造器承担

建议：

- 在 `platform-core` 中建立统一 event envelope builder
- 所有业务模块通过统一入口写 outbox
- topic、key、version、target bus 从统一路由策略解析

### 7.2 停用自动派生 topic 的正式路径

对数据库触发器做收口：

- 停止使用按 `schema.table -> target_topic` 自动派生的正式 outbox 规则
- 如果触发器仍需保留，只能作为历史兼容/迁移期工具，不得作为主链路默认来源
- 正式生产路径应由应用层在事务中显式写入 canonical outbox event

### 7.3 清理双写与旁路

重点清理以下模式：

- 同一对象既挂统一 outbox 触发器，又在业务代码中手工写事件
- 搜索同步事件绕开统一 envelope / route policy 直接写 topic
- delivery / order / catalog 等模块各自定义不同顶层字段名
- `catalog.product` 既保留统一触发器，又手工写 `search.product.changed`

### 7.4 统一顶层字段

对所有正式 outbox payload 顶层统一为冻结字段集：

- `event_id`
- `event_type`
- `event_version`
- `occurred_at`
- `producer_service`
- `aggregate_type`
- `aggregate_id`
- `request_id`
- `trace_id`
- `idempotency_key`
- `payload`

明确要求：

- `event_name` 不再作为正式顶层字段使用
- 若历史消费方仍依赖旧字段，应在迁移窗口内通过受控兼容层过渡，但最终对外协议必须统一

### 7.5 先做一轮事件生产源审计

在动手修改前，应先完整列出：

- 哪些表仍挂 `tg_write_outbox`
- 哪些模块仍手工写 outbox
- 哪些路径绕过 `event_route_policy`
- 哪些 payload 顶层仍不是 canonical envelope

这份清单应作为修复的输入，而不是边改边猜。

## 8. 实施范围

至少覆盖以下内容：

### 8.1 数据库与路由策略

- `docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql`
- `docs/数据库设计/V1/upgrade/050_audit_search_dev_ops.sql`
- `ops.event_route_policy`
- 相关 outbox trigger / function

### 8.2 应用与模块

- `apps/platform-core/src/modules/catalog/**`
- `apps/platform-core/src/modules/order/**`
- `apps/platform-core/src/modules/delivery/**`
- `apps/platform-core/src/modules/billing/**`
- 统一 event / outbox builder 所在模块

### 8.3 下游影响面

- publisher
- `search-indexer`
- `notification-worker`
- `fabric-adapter`
- `dead letter` / replay / consistency 联查

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- `event_route_policy` 被正式代码路径实际使用
- 不再存在正式主链路依赖“schema.table 自动派生 topic”
- 不再存在同一对象“触发器 + 手工直写”双写事件
- 正式 payload 顶层统一，不再混用 `event_name`
- `catalog.product` 不再同时存在触发器事件生产与手工搜索事件直写的双轨默认路径

### 9.2 动态验证

至少验证：

1. 订单创建事件
2. 搜索同步事件
3. 交付事件
4. 审计锚定请求事件

要求验证：

- envelope 顶层字段完整
- 路由命中统一策略
- consumer 幂等维度可统一提取
- 不出现重复 outbox 记录
- 不出现同一业务动作被触发器和代码双重写入事件

### 9.3 publisher / consumer 兼容性

- publisher 只认统一 envelope
- consumer 不需要再根据模块来源写多套解析规则
- DLQ / replay / consistency trace 可以基于统一字段工作

### 9.4 当前收口结果

截至 `2026-04-20`，本任务已完成首轮正式收口，当前冻结口径如下：

- `ops.event_route_policy` 已成为运行时唯一正式 route authority
- `apps/platform-core/src/shared/outbox.rs` 已成为应用层唯一正式 canonical outbox writer
- `trade.order`、`delivery.delivery_record`、`billing.*`、`support.dispute_case`、`product` 等当前主链路事件已切换到统一 writer
- `catalog.product`、`trade.order_main`、`support.dispute_case`、`payment.*`、`recommend.behavior_event` 的旧 outbox trigger 已从正式主链路移除
- `common.tg_write_outbox()` 已退役为显式异常函数，禁止继续作为正式默认实现
- `trade003`、`dlv029`、`dlv002`、`bil024`、`cat022` smoke 已验证 canonical envelope 与 route authority 命中

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 最终统一 envelope 字段定义
3. route authority 说明
4. 被停用/移除的旧触发器或旧写法清单
5. 双写清理结果
6. `event_route_policy` 的实际使用方式
7. 测试与联调结果

## 11. 一句话结论

`A02` 的正确修法不是“补几个字段”，而是“把事件 envelope 和 route authority 收敛成唯一正式机制”，并一次性清理触发器自动派生、手工直写和绕过策略的并行路径。
