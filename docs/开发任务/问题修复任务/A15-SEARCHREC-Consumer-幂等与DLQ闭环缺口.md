# A15 SEARCHREC Consumer 幂等与 DLQ 闭环缺口

## 0. 当前状态

- 本文当前角色：`SEARCHREC` consumer 幂等、双层 DLQ 与 reprocess 闭环的治理说明。
- 当前正式口径、任务承接和 runbook / test-case 方向已经冻结；真实代码闭环仍由 `AUD-008 / AUD-010 / SEARCHREC-020 / AUD-026 / SEARCHREC-015 / SEARCHREC-017` 承接。
- 第 `2` 节与第 `4` 节保留问题发现时的历史起点，当前实现状态应以 worker 代码、task、runbook 与测试矩阵为准。

## 1. 任务定位

- 问题编号：`A15`
- 严重级别：`high`
- 关联阶段：`SEARCHREC`
- 关联任务：`AUD-008`、`AUD-010`、`AUD-026`、`SEARCHREC-001`、`SEARCHREC-010`、`SEARCHREC-015`、`SEARCHREC-017`、`SEARCHREC-020`
- 处理方式：先把 SEARCHREC consumer 的正式失败处理口径冻结到统一文档与任务清单，再在实现批次中补齐幂等、双层 DLQ、reprocess 与测试闭环

## 2. 历史问题起点（归档）

问题发现时，仓库中的两个 SEARCHREC Kafka consumer 已经存在基础消费逻辑，但失败处理口径并没有收缩到正式冻结要求，导致“处理失败后仍提交 offset”“只有部分幂等、没有双层 DLQ、没有统一重处理”的不完整状态。

问题发现时已确认的典型现象包括：

1. `search-indexer` 在 `handle_kafka_message` 出错后仍会继续 `commit_message`
2. `search-indexer` 还没有正式接入 `ops.consumer_idempotency_record`
3. `recommendation-aggregator` 虽然已有部分 `consumer_idempotency_record` 逻辑，但同样没有 `ops.dead_letter_event + Kafka DLQ` 双层失败隔离
4. 当前测试主要停留在 API / outbox 或手工 seed OpenSearch，没有覆盖 worker 侧副作用、失败路径与 reprocess 路径

这意味着：

- 一旦 consumer 处理失败，事件可能被提交 offset 后静默丢失
- 运维侧无法在 `ops.dead_letter_event` 或 Kafka DLQ 中找到完整失败隔离记录
- 后续 `AUD-010` 的 dead letter reprocess 也缺少 SEARCHREC worker 的真实落点

## 3. 正确冻结口径

应以双层权威设计、事件接口协议、topic 清单与 `056_dual_authority_consistency.sql` 为冻结基线，明确以下口径：

1. `search-indexer` 与 `recommendation-aggregator` 都是正式 Kafka consumer
2. 所有 SEARCHREC consumer 都必须基于统一 envelope 的 `event_id` 做幂等消费
3. consumer 幂等记录必须落入 `ops.consumer_idempotency_record`
4. 失败必须双层隔离：
   - 数据库：`ops.dead_letter_event`
   - Kafka：统一 DLQ topic `dtp.dead-letter`
5. 只有在“成功处理”或“失败已安全落入 DB/Kafka 双层 DLQ”后，才允许提交 offset
6. `AUD-010` 的 `POST /api/v1/ops/dead-letters/{id}/reprocess` 必须能覆盖 SEARCHREC consumer 失败事件，并支持 `dry-run + step-up`
7. 测试不能只断言 API / outbox 行存在，必须覆盖 worker 侧副作用、失败路径、DLQ 路径与 reprocess 路径

## 4. 历史问题证据（归档）

问题处理前已核对的典型证据包括但不限于：

- [双层权威模型与链上链下一致性设计.md](/home/luna/Documents/DataB/docs/原始PRD/双层权威模型与链上链下一致性设计.md)
  - 已冻结 consumer / DLQ / replay 的正式语义
- [056_dual_authority_consistency.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql)
  - 已有 `consumer_idempotency_record` 相关 schema
- [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md)
  - 已冻结统一 DLQ topic `dtp.dead-letter`
- [workers/search-indexer/src/main.rs](/home/luna/Documents/DataB/workers/search-indexer/src/main.rs)
  - 当前失败后仍提交 offset，且无正式 consumer idempotency / dual-DLQ 闭环
- [workers/recommendation-aggregator/src/main.rs](/home/luna/Documents/DataB/workers/recommendation-aggregator/src/main.rs)
  - 当前只有部分 idempotency，没有双层 DLQ 闭环
- [docs/05-test-cases/search-rec-cases.md](/home/luna/Documents/DataB/docs/05-test-cases/search-rec-cases.md)
  - 现有测试文档仍需明确 worker 侧副作用、DLQ 与 reprocess 验收项

## 5. 任务目标

通过 `AUD-008`、`AUD-010`、`SEARCHREC-020`、`AUD-026`、`SEARCHREC-015`，完成以下收口：

1. SEARCHREC consumer 全部具备正式 `event_id` 幂等
2. SEARCHREC consumer 失败路径具备 `ops.dead_letter_event + dtp.dead-letter` 双层隔离
3. offset 提交策略不再允许“失败后直接提交”
4. `AUD-010` 能对 SEARCHREC dead letter 做正式 dry-run / step-up 重处理
5. SEARCHREC 测试和 runbook 不再只验证 API / outbox，而要覆盖 worker 侧真实副作用

## 6. 强约束

1. 不能只给 `recommendation-aggregator` 保留部分幂等，而让 `search-indexer` 继续无幂等
2. 不能只保留 Kafka DLQ，不补数据库 `ops.dead_letter_event`
3. 不能只落数据库 dead letter，不补 Kafka `dtp.dead-letter`
4. 不能在处理失败后仍直接提交 offset，把丢数风险隐去
5. 不能继续用“手工 seed OpenSearch”“只看 outbox 行存在”来冒充 worker 侧可靠性验证

## 7. 建议修复方案

### 7.1 先补基础设施与仓储

通过 `AUD-008` 先补齐：

- `ops.dead_letter_event`
- `ops.consumer_idempotency_record`
- 相关查询与联查能力

### 7.2 再补 SEARCHREC worker 可靠性

通过 `SEARCHREC-020` 收敛：

- `search-indexer`
- `recommendation-aggregator`

要求：

- 统一按 `event_id` 幂等
- 失败时先写 DB/Kafka 双层 DLQ
- 成功或安全隔离后再提交 offset

### 7.3 最后补测试与 reprocess

通过 `AUD-010`、`AUD-026`、`SEARCHREC-015`、`SEARCHREC-017` 补齐：

- dead letter reprocess
- worker 集成测试
- runbook 与测试矩阵

## 8. 实施范围

至少覆盖以下内容：

### 8.1 SEARCHREC workers

- `workers/search-indexer/**`
- `workers/recommendation-aggregator/**`

### 8.2 AUD 基础设施

- `apps/platform-core/src/modules/audit/**`
- `docs/数据库设计/V1/upgrade/056_dual_authority_consistency.sql`

### 8.3 文档与验证

- `docs/04-runbooks/search-reindex.md`
- `docs/04-runbooks/recommendation-runtime.md`
- `docs/05-test-cases/search-rec-cases.md`

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 两个 SEARCHREC consumer 都已明确接入 `ops.consumer_idempotency_record`
- 两个 SEARCHREC consumer 的失败路径都已明确进入 `ops.dead_letter_event + dtp.dead-letter`
- 不再保留“失败后直接提交 offset”作为默认口径

### 9.2 运行闭环

应能明确证明：

1. `search-indexer` 失败事件可以进入双层 DLQ
2. `recommendation-aggregator` 失败事件可以进入双层 DLQ
3. 通过 `AUD-010` 可对相关 dead letter 进行 dry-run / step-up 重处理
4. worker 侧副作用有正式验证，而不是只看 API / outbox

### 9.3 阶段承接性

修复后应能直接支撑：

- `AUD-026` 审计/一致性/Fabric/Ops 测试中的 DLQ 与 replay 验证
- `SEARCHREC-015` 搜索/推荐 worker 可靠性测试
- `SEARCHREC-017` 搜索/推荐测试矩阵与 runbook 收口

若仍存在“失败已提交 offset，但 DB/Kafka 中查不到隔离记录”的情况，则视为收口不完整。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. SEARCHREC consumer 可靠性改造清单
2. `event_id -> consumer_idempotency_record -> dead_letter_event / dtp.dead-letter -> reprocess` 闭环说明
3. worker 侧副作用与失败路径验证结果
4. runbook / 测试矩阵更新清单

## 11. 一句话结论

`A15` 的核心问题不是“有没有 SEARCHREC worker”，而是 SEARCHREC worker 的正式 consumer 可靠性闭环还没有成立；如果不通过 `AUD-008 / AUD-010 / SEARCHREC-020` 把幂等、双层 DLQ、offset 提交策略和重处理收成一套正式口径，后续搜索投影和推荐行为流会一直带着静默丢数风险。 
