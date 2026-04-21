# A11 测试与 Smoke 口径误报风险

## 1. 任务定位

- 问题编号：`A11`
- 严重级别：`high`
- 关联阶段：`cross-stage`
- 关联任务：`AUD-026`、`SEARCHREC-015`、`SEARCHREC-017`、`NOTIF-012`、`TEST-005`、`TEST-016`、`TEST-027`、`TEST-028`
- 处理方式：先收口测试、smoke、契约检查的验证目标，使其对准真实冻结接口、canonical topic 和完整业务闭环，避免“测过但口径仍错”

## 2. 问题描述

当前部分测试、smoke 和契约检查仍停留在旧 topic、局部 outbox 行、局部投影字段或骨架 OpenAPI 路径上，不能证明系统真的符合冻结口径。

当前已确认的典型现象：

1. 搜索相关测试只验证商品详情中的搜索投影字段或 `search.product.changed` outbox 行
2. 多条 Billing / Delivery 测试仍断言历史 topic `billing.events`
3. `scripts/smoke-local.sh` 仍只检查旧 topic
4. `scripts/check-openapi-schema.sh` 只覆盖极少数 `health/internal` 路径

这意味着：

- 本地 smoke 或 CI 可能给出错误正反馈
- 搜索、通知、审计、统一事件等阶段性漂移会被掩盖
- 后续阶段在错误验证基线上继续推进

## 3. 正确冻结口径

测试与 smoke 必须对准以下正式口径：

### 3.1 接口层

- 验证真实冻结接口
- 不能只验证内部表字段或骨架路由

### 3.2 事件层

- 验证 canonical topic
- 不能继续只断言旧 topic 或局部旁路事件

### 3.3 业务闭环

至少应覆盖：

- `PostgreSQL` 主状态
- 正式 outbox / publisher / consumer 链路
- `OpenSearch / Redis / Kafka` 的真实参与
- 最终回 `PostgreSQL` 做业务校验

### 3.4 契约检查

- OpenAPI 检查必须覆盖真实业务路径
- 不能只检查 `health/internal` 一类骨架路径

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [cat022_search_visibility_db.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/catalog/tests/cat022_search_visibility_db.rs)
  - 只验证投影字段和 `search.product.changed` outbox，未验证 `GET /api/v1/catalog/search` 或完整搜索闭环
- [bil006_billing_event_db.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/billing/tests/bil006_billing_event_db.rs)
  - 仍断言 `billing.events`
- [bil011_manual_payout_db.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/billing/tests/bil011_manual_payout_db.rs)
  - 仍断言 `billing.events`
- [dlv002_file_delivery_commit_db.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/delivery/tests/dlv002_file_delivery_commit_db.rs)
  - 仍断言旧 topic
- [smoke-local.sh](/home/luna/Documents/DataB/scripts/smoke-local.sh)
  - 仍检查旧 topic
- [check-openapi-schema.sh](/home/luna/Documents/DataB/scripts/check-openapi-schema.sh)
  - 当前只覆盖极少数路径

## 5. 任务目标

把测试与 smoke 收口为真正的冻结口径验证体系，确保：

1. 测试断言不再围绕旧 topic 或局部代理指标
2. smoke 对准 canonical topic 与真实链路
3. 搜索、通知、审计、事件闭环有真实集成验证
4. OpenAPI 检查能发现业务接口缺失，而不是只验证骨架路径

## 6. 强约束

1. 不能只改实现，不改测试和 smoke
2. 不能只改测试名，不改断言目标
3. 不能继续让旧 topic 出现在正式主线测试断言中
4. 不能把局部 outbox 行、局部投影字段当成阶段完成证明
5. 不能让 OpenAPI 检查继续只覆盖 `health/internal`

## 7. 建议修复方案

### 7.1 先建立“冻结口径测试矩阵”

应先明确每类能力必须验证什么：

- 接口
- canonical topic
- publisher / consumer
- Redis / OpenSearch / MinIO / Fabric 等中间件参与
- `PostgreSQL` 最终业务校验

不要再让测试各自凭历史路径选断言目标。

### 7.2 清理旧 topic 断言

重点清理：

- `billing.events`
- `search.product.changed`
- 其他历史 topic / 私有旁路 topic

要求：

- 测试断言统一迁到 canonical topic
- 或统一迁到正式事件 envelope / route authority 验证

### 7.3 搜索测试改为验证真实闭环

搜索相关测试至少应验证：

1. 搜索同步事件进入正式链路
2. `search-indexer` 更新 `OpenSearch`
3. `Redis` 缓存参与
4. `GET /api/v1/catalog/search` 可用
5. 返回前做 `PostgreSQL` 最终校验

### 7.4 OpenAPI 检查扩展到业务接口

`check-openapi-schema.sh` 应扩展为至少覆盖：

- `audit`
- `ops`
- `search`
- `recommendation`
- `notification` 相关正式接口/契约文件

不能继续只检查骨架路径。

### 7.5 smoke 脚本切到 canonical topic 与真实链路

`scripts/smoke-local.sh` 应至少验证：

- canonical topic 存在
- 真正的 publisher / worker / consumer 链路
- 不是只看旧 topic 或 topic 是否被创建

## 8. 实施范围

至少覆盖以下内容：

### 8.1 测试代码

- `apps/platform-core/src/modules/catalog/tests/**`
- `apps/platform-core/src/modules/billing/tests/**`
- `apps/platform-core/src/modules/delivery/tests/**`
- `apps/platform-core/src/modules/order/tests/**`
- 后续 `audit / search / recommendation / notification` 相关集成测试

### 8.2 脚本

- `scripts/smoke-local.sh`
- `scripts/check-openapi-schema.sh`
- 其他本地联调 / CI 校验脚本

### 8.3 文档

- `docs/05-test-cases/**`
- 与搜索、推荐、审计、通知相关的测试用例文档

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 主线测试中不再默认断言旧 topic
- 搜索测试不再只验证局部投影字段或局部 outbox 行
- `check-openapi-schema.sh` 不再只覆盖骨架路径
- smoke 脚本使用 canonical topic

### 9.2 动态验证

至少验证：

1. 搜索真实闭环
2. 通知真实事件驱动链路
3. 统一 outbox/publisher/consumer/DLQ 链路
4. OpenAPI 与实现路径一致性

### 9.3 误报风险消除

修复后应能明确回答：

- 如果 canonical topic 错了，现有测试/CI 是否会失败
- 如果 `GET /api/v1/catalog/search` 缺失，现有测试/CI 是否会失败
- 如果 `audit/ops` OpenAPI 缺失，契约检查是否会失败

若答案仍然是否定的，则说明误报风险尚未消除。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 被清理的旧断言清单
3. 新的冻结口径测试矩阵
4. smoke 与 OpenAPI 检查扩展结果
5. 联调 / 集成测试结果

## 11. 一句话结论

`A11` 的核心问题不是“测试少”，而是当前测试和 smoke 在验证错误目标；如果不先把验证口径收口，后续阶段会持续得到错误正反馈。
