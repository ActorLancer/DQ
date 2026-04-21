# A01 Kafka Topic 口径统一

## 1. 任务定位

- 问题编号：`A01`
- 严重级别：`blocker`
- 关联阶段：`cross-stage`
- 关联任务：`ENV-010`、`ENV-011`、`AUD-009`、`AUD-022`、`SEARCHREC-001`、`NOTIF-001`、`NOTIF-002`、`NOTIF-009`、`TEST-028`
- 处理方式：先统一冻结口径，再统一代码/脚本/文档/测试，不允许仅做表面字符串替换

## 2. 问题描述

当前仓库中的 Kafka topic 已经分裂成三套口径：

1. 冻结文档使用 `dtp.*`
2. 基础设施脚本、本地 smoke、runbook 仍保留旧 topic
3. 业务代码中混用了 `dtp.*` 和历史 topic（如 `billing.events`）

这会导致：

- publisher、worker、runbook、smoke 可能分别对着不同 topic 工作
- 自检脚本通过，但真实异步链路不通
- 后续 `NOTIF / AUD / SEARCHREC` 阶段在错误基线上继续扩散

## 3. 正确冻结口径

以 [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md) 为冻结基线，topic 必须统一到正式命名：

### 3.1 核心 topic

- `dtp.outbox.domain-events`
- `dtp.search.sync`
- `dtp.recommend.behavior`
- `dtp.notification.dispatch`
- `dtp.audit.anchor`
- `dtp.dead-letter`

### 3.2 同一冻结体系下的辅助 topic

- `dtp.payment.callbacks`
- `dtp.fabric.requests`
- `dtp.fabric.callbacks`
- `dtp.consistency.reconcile`

### 3.3 明确边界

- `dtp.dead-letter` 是统一死信流
- `Kafka` 是分发总线，不是业务权威源
- 不允许继续长期保留历史 topic 与新 topic 双轨并存
- 不允许新增未冻结的旁路 topic 作为主链路默认值

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md)
- [init-topics.sh](/home/luna/Documents/DataB/infra/kafka/init-topics.sh)
- [lib.rs](/home/luna/Documents/DataB/apps/platform-core/src/lib.rs)
- [kafka-topics.md](/home/luna/Documents/DataB/docs/04-runbooks/kafka-topics.md)
- [port-matrix.md](/home/luna/Documents/DataB/docs/04-runbooks/port-matrix.md)
- [docker-compose.apps.local.example.yml](/home/luna/Documents/DataB/infra/docker/docker-compose.apps.local.example.yml)
- [outbox_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/delivery/repo/outbox_repository.rs)
- [billing_event_repository.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/billing/repo/billing_event_repository.rs)

## 5. 任务目标

将 Kafka topic 口径收敛为单一、可执行、可验证的一套标准，确保：

1. 文档、脚本、默认配置、代码、测试、runbook 全部一致
2. 所有相关 worker / smoke / 启动自检都使用同一套 topic
3. 旧 topic 不再作为运行时默认值存在
4. `billing.events` 等历史旁路 topic 完成归并或移除

## 6. 强约束

1. 不能只改文档，不改代码
2. 不能只改代码，不改脚本/runbook/smoke
3. 不能只做字符串替换，必须先冻结唯一 topic 字典
4. 不能长期保留双写、双订阅、双 topic 兼容作为默认方案
5. 不能把范围误缩成只保留 6 个 topic，辅助 topic 也必须纳入同一冻结体系
6. 不允许把 `billing.events` 之类历史 topic 继续保留为主链路默认 topic

## 7. 建议修复方案

### 7.1 先建立唯一执行源

新增一份机器可读的 topic 字典文件，例如：

- `infra/kafka/topics.v1.yaml`
- 或 `infra/kafka/topics.v1.json`

至少包含：

- topic 名
- producer
- consumer
- 用途
- key 规则
- 是否 DLQ
- 是否阶段必需

该文件作为：

- `init-topics.sh`
- 应用默认配置
- smoke
- runbook
- 测试断言

的统一执行基线。

### 7.2 统一基础设施层

必须同步收口：

- `infra/kafka/init-topics.sh`
- compose 示例与相关 env 默认值
- `docs/04-runbooks/kafka-topics.md`
- `docs/04-runbooks/port-matrix.md`
- 本地 smoke / 联调脚本

### 7.3 统一代码层

禁止在多个模块中继续散落 topic 字符串字面量。

建议：

- 在 `platform-core` 中建立统一 topic registry / config 模块
- Rust / Go / worker / shell 脚本统一引用同一来源

### 7.4 对历史 topic 做专项归并

重点审计并处理：

- `billing.events`
- `outbox.events`
- `search.sync`
- `audit.anchor`
- `recommendation.behavior`
- `dead-letter.events`

要求：

- 如果表达的是主领域事件，归并到 `dtp.outbox.domain-events`
- 如果表达的是冻结体系下的辅助事件，归并到正式辅助 topic
- 历史 topic 不再出现在运行时默认值中

## 8. 实施范围

至少覆盖以下内容：

### 8.1 文档与脚本

- `infra/kafka/**`
- `scripts/**`
- `docs/04-runbooks/**`
- `infra/docker/**`

### 8.2 应用与模块

- `apps/platform-core/src/lib.rs`
- `apps/platform-core/src/modules/**`
- `workers/**`
- `services/**`

### 8.3 测试与联调

- 本地 smoke
- 集成测试
- topic 自检
- 启动健康检查与 broker/topic 校验

## 9. 验收标准

### 9.1 静态收口

以下历史 runtime topic 不应再作为默认值出现在代码/脚本/runbook/smoke 中：

- `outbox.events`
- `search.sync`
- `audit.anchor`
- `billing.events`
- `recommendation.behavior`
- `dead-letter.events`

允许保留的场景仅限：

- 历史偏移说明
- 审计说明
- 变更记录

### 9.2 动态验证

至少跑通以下链路：

1. `outbox -> notification`
2. `dtp.search.sync -> search-indexer -> OpenSearch`
3. `dead-letter / reprocess` 基本链路

### 9.3 启动校验

- `platform-core` 启动时检查的是 canonical topic
- worker 启动时订阅的是 canonical topic
- `init-topics.sh` 创建的是 canonical topic

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. topic 字典最终清单
3. 旧 topic 到新 topic 的映射表
4. 静态检索结果
5. 联调与测试结果
6. 是否还有历史 topic 仅作为审计/说明残留

## 11. 一句话结论

`A01` 的正确修法不是“逐处替换字符串”，而是“先冻结唯一 topic 字典，再统一代码、脚本、runbook、smoke 和测试”，并一次性移除历史 topic 的运行时默认地位。
