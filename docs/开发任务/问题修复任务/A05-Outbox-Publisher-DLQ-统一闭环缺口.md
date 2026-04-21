# A05 Outbox Publisher 与 DLQ 统一闭环缺口

## 1. 任务定位

- 问题编号：`A05`
- 严重级别：`blocker`
- 关联阶段：`AUD`
- 关联任务：`AUD-008`、`AUD-009`、`AUD-010`、`AUD-026`、`AUD-031`
- 处理方式：补齐 `outbox_event -> publisher -> Kafka -> consumer -> dead_letter/reconcile` 的正式闭环，并清理模块直接把 `ops.outbox_event` 当工作队列消费的旁路实现

## 2. 问题描述

当前系统中，统一 outbox 闭环尚未真正落地：

1. `workers/outbox-publisher` 仍为空骨架
2. compose 中对应进程仍是 placeholder
3. `dead_letter_event` 基本没有形成完整写入/消费路径
4. 反而 Billing 已直接把 `ops.outbox_event` 当工作队列消费

当前已确认的典型现象：

- schema 已为 outbox / publish attempt / consumer idempotency / dead letter 做好建模
- 但 publisher worker 本身没有正式实现
- Billing bridge 直接 `FOR UPDATE` 读取 `ops.outbox_event` 并更新状态

这意味着：

- 正式事件分发闭环没有成立
- 模块可能各自围绕 `ops.outbox_event` 建私有消费逻辑
- 后续统一 publisher、DLQ、reprocess、审计幂等语义会越来越难收敛

## 3. 正确冻结口径

以全集成基线和一致性设计为冻结基线，`V1` 正式模式必须是：

- `DB transaction + outbox + publisher worker + Kafka`

同时必须满足：

- 保留数据库层 `ops.dead_letter_event`
- 保留 Kafka 层统一 DLQ topic
- consumer 幂等记录可审计、可联查
- 不允许把 `Kafka` 当业务真相源
- 不允许让单个业务模块直接把 `ops.outbox_event` 当私有工作队列长期消费

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [数据交易平台-全集成基线-V1.md](/home/luna/Documents/DataB/docs/全集成文档/数据交易平台-全集成基线-V1.md)
  - 已冻结 `DB transaction + outbox + publisher worker + Kafka`
- [056_dual_authority_consistency.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql)
  - 已存在 `outbox_publish_attempt`
  - 已存在 `consumer_idempotency_record`
  - 已存在 `dead_letter_event`
- [workers/README.md](/home/luna/Documents/DataB/workers/README.md)
  - `outbox-publisher` 仍只是目录级占位
- [docker-compose.apps.local.example.yml](/home/luna/Documents/DataB/infra/docker/docker-compose.apps.local.example.yml)
  - `outbox-publisher` 仍是 placeholder
- [billing_bridge_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/billing/repo/billing_bridge_repository.rs)
  - 直接 `FOR UPDATE` 读取 `ops.outbox_event` 并更新状态

## 5. 任务目标

补齐统一事件分发闭环，确保：

1. outbox 事件由正式 publisher worker 发布到 Kafka
2. `ops.outbox_event` 不再被业务模块当私有工作队列直接消费
3. `dead_letter_event` 与 Kafka DLQ 双层机制都真正可用
4. consumer 幂等记录和发布尝试记录可以联查和审计
5. `AUD-026` 能建立真实集成测试，而不是只围绕单库轮询假装闭环

## 6. 强约束

1. 不能只建 publisher 目录，不实现真正运行逻辑
2. 不能只实现 Kafka 发布，不补 `dead_letter_event`
3. 不能只补 DLQ topic，不补数据库 `dead_letter_event`
4. 不能继续允许业务模块直接把 `ops.outbox_event` 当主工作队列消费
5. 不能只靠“以后让 Billing 改掉”，必须在本次收口时明确清理旁路
6. 不能只记录发布成功，不记录发布尝试、失败原因和消费幂等

## 7. 建议修复方案

### 7.1 正式实现 `workers/outbox-publisher`

至少应具备：

- 批量拉取待发布 outbox
- `SKIP LOCKED`
- 发布到 canonical Kafka topic
- 写 `outbox_publish_attempt`
- 更新 publish status
- 超限失败转入 `dead_letter_event`

### 7.2 停止模块直接消费 `ops.outbox_event`

重点处理：

- Billing bridge 直接 `FOR UPDATE` 读取 `ops.outbox_event`

原则：

- `ops.outbox_event` 是统一分发出站表
- 业务模块若需要消费事件，应通过正式 Kafka consumer 或明确的下游队列/作业表
- 不应继续把 outbox 表本身当模块私有工作队列

### 7.3 补齐双层 DLQ

必须同时保留两层：

- 数据库：`ops.dead_letter_event`
- Kafka：统一 DLQ topic

要求：

- 发布失败可转入数据库 dead letter
- 消费失败也应有受控 dead letter 归集
- 支持 reprocess / dry-run / 审计留痕

### 7.4 落实 consumer 幂等记录

至少应统一使用：

- `event_id`
- 或 `provider_event_id`
- 或 `request_id + aggregate_id + event_type`

并写入：

- `consumer_idempotency_record`

同时保留：

- 消费结果
- 失败原因
- 重处理状态

### 7.5 建立真实集成测试

至少验证：

1. outbox 写入
2. publisher 发布 Kafka
3. consumer 消费
4. 幂等去重
5. 发布失败或消费失败进入 DLQ
6. reprocess / dry-run 路径

## 8. 实施范围

至少覆盖以下内容：

### 8.1 worker 与编排

- `workers/outbox-publisher/**`
- `workers/README.md`
- `infra/docker/docker-compose.apps.local.example.yml`

### 8.2 数据库与一致性层

- `ops.outbox_event`
- `ops.outbox_publish_attempt`
- `ops.consumer_idempotency_record`
- `ops.dead_letter_event`

### 8.3 业务旁路清理

- `apps/platform-core/src/modules/billing/repo/billing_bridge_repository.rs`
- 其他直接轮询/锁定 `ops.outbox_event` 的模块

### 8.4 联动面

- Kafka topic
- DLQ
- replay / reprocess
- consistency / ops 联查接口

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- `workers/outbox-publisher` 不再是空壳
- compose 中有可运行的 outbox publisher
- 不再存在 Billing 把 `ops.outbox_event` 当主工作队列的默认路径
- `dead_letter_event` 有正式写入与查询/reprocess 路径

### 9.2 动态验证

至少验证：

1. 主对象事务内写入 `ops.outbox_event`
2. publisher 把事件发到 Kafka
3. consumer 成功消费并写幂等记录
4. 同一事件重复消费不产生重复副作用
5. 发布失败或消费失败进入 `dead_letter_event`
6. Kafka DLQ 也有对应隔离记录

### 9.3 审计与联查

修复后应可联查：

- 一条 outbox 事件的发布尝试
- 一条消费幂等记录
- 一条 dead letter 失败记录
- 一次 reprocess / dry-run 记录

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. `outbox-publisher` 运行方式
3. Billing 旁路清理说明
4. 发布尝试与幂等记录说明
5. DLQ 双层落地说明
6. 集成测试与联调结果

## 11. 一句话结论

`A05` 的核心问题不是“有没有 outbox 表”，而是统一闭环 `outbox -> publisher -> Kafka -> consumer -> dead_letter/reconcile` 并未真正成立；如果继续允许模块直接消费 `ops.outbox_event`，就会把错误旁路固化成事实标准。
