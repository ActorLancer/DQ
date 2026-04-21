# A10 NOTIF 通知链路与命名边界缺口

## 1. 任务定位

- 问题编号：`A10`
- 严重级别：`blocker`
- 关联阶段：`NOTIF`
- 关联任务：`NOTIF-001` 至 `NOTIF-014`
- 处理方式：先把通知阶段的运行时进程、命名、事件驱动链路与渠道边界收口；进入 `NOTIF` 代码实现批次后，再把幂等/DLQ/审计/OpenAPI/runbook/测试补成闭环

## 1.1 当前批次边界

如果当前批次只处理“口径未收缩/命名冲突/事件拓扑冲突”，则：

- 可以先冻结 `notification.requested -> dtp.notification.dispatch -> notification-worker`
- 可以先修正 runbook 中的正式进程名、topic、consumer group 与渠道边界
- 但不能把尚未实现的 OpenAPI、发送记录模型、集成测试误报为已完成

进入 `NOTIF` 代码实现批次后，Agent 必须同步补齐：

- 通知发送记录模型与联查接口
- `packages/openapi/ops.yaml` / `docs/02-openapi/ops.yaml` 中与通知联查相关的示例
- `docs/05-test-cases/` 中的通知事件链路验收清单
- runbook 中的模板清单、人工补发、失败重试与 DLQ 实操步骤

## 2. 问题描述

当前通知阶段几乎仍停留在占位状态，同时“通知进程”的命名与组织方式已经分裂成两套：

1. `notification-worker`
2. `notification-service`（旧命名，已废弃）

当前已确认的典型现象：

1. 任务与事件模型已经冻结完整通知能力
2. 代码和目录仍只有占位 README / placeholder compose
3. `provider-kit` 只有极简 stub
4. runbook 和通知链路测试资产均未形成
5. 当前未发现业务 handler/repo 中同步直调通知发送逻辑，问题主要是“未落地”，不是“已错误内联”

这意味着：

- 一旦正式进入 `NOTIF` 阶段，topic、部署名、渠道边界、幂等、DLQ、审计和联查口径会同时发散
- 后续 Agent 无法判断到底要实现 `notification-worker` 还是旧命名 `notification-service`
- “V1 只实接 mock-log” 的边界很容易被打破或被遗忘

## 3. 正确冻结口径

根据任务清单与事件模型，通知阶段应满足以下正式口径：

### 3.1 驱动方式

- 通知必须是 `Kafka` 事件驱动
- 正式链路应围绕：
  - `notification.requested`
  - `dtp.notification.dispatch`
  - `notification-worker`
- 当前正式进程名统一为 `notification-worker`

### 3.2 渠道边界

`V1` 正式口径：

- `mock-log` 是实接渠道
- `email / webhook / 其他外部渠道` 只保留边界与适配器接口

不能把真实外部邮件/短信/webhook 提供商提前做成 V1 阶段主阻塞项。

### 3.3 必备能力

通知阶段必须同时具备：

- 模板
- 幂等
- 重试
- DLQ
- 审计联查
- runbook
- 集成测试

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [v1-core-开发任务清单.md](/home/luna/Documents/DataB/docs/开发任务/v1-core-开发任务清单.md)
  - `NOTIF-001 ~ NOTIF-012` 已冻结通知阶段完整要求
- [事件模型与Topic清单正式版.md](/home/luna/Documents/DataB/docs/开发准备/事件模型与Topic清单正式版.md)
  - 已冻结 `notification.requested -> dtp.notification.dispatch -> notification-worker`
- [apps/README.md](/home/luna/Documents/DataB/apps/README.md)
  - 使用 `notification-worker`
- [services/README.md](/home/luna/Documents/DataB/services/README.md)
  - 已移除旧命名落位，改为声明通知进程正式落位 `apps/notification-worker`
- [apps/notification-worker/README.md](/home/luna/Documents/DataB/apps/notification-worker/README.md)
  - 当前仍只有占位
- [docker-compose.apps.local.example.yml](/home/luna/Documents/DataB/infra/docker/docker-compose.apps.local.example.yml)
  - 通知应用仍是 placeholder
- [provider-kit/src/lib.rs](/home/luna/Documents/DataB/apps/platform-core/crates/provider-kit/src/lib.rs)
  - 当前只提供极简通知 provider stub

## 5. 任务目标

将通知阶段从占位状态收口为正式可运行阶段，确保：

1. 通知进程命名和目录边界只有一套正式答案
2. 通知链路真实由 Kafka 事件驱动
3. `mock-log` 成为 `V1` 的正式实接渠道
4. 模板、幂等、重试、DLQ、审计联查、runbook、测试成套落地
5. 通知阶段不再只是“有个 provider stub”

## 6. 强约束

1. 不能只统一命名，不补运行时链路
2. 不能只补运行时链路，不统一命名与目录边界
3. 不能把 `email / webhook` 提前误做成 `V1` 必需实接
4. 不能只做发送逻辑，不补幂等、重试、DLQ、审计联查
5. 不能只靠 `provider-kit` stub 冒充通知阶段完成
6. 不能把通知做成同步业务主链路调用

## 7. 建议修复方案

### 7.1 先冻结唯一进程命名与目录边界

本任务直接收口为：

- 当前正式进程名 = `notification-worker`
- `notification-service` 只能作为历史漂移命名出现在审计说明或迁移映射中，不再作为运行时默认命名

要求：

- `apps/README.md`
- `services/README.md`
- compose
- runbook
- 任务执行目录

全部统一到同一套命名。
并以事件模型与服务边界文档口径为准，不再继续两套名字并存。

### 7.2 落地完整通知运行时（V1 范围）

至少应具备：

- 消费 `dtp.notification.dispatch`
- 模板选择/渲染
- `mock-log` 渠道发送
- 幂等去重
- 重试
- DLQ
- 审计与发送记录

### 7.3 明确渠道边界

在运行时和 provider 层同时冻结：

- `mock-log` 为正式 `V1` 实接
- `email`
- `webhook`
- 其他渠道

仅保留 provider trait / adapter 边界，不要求真实外部连通。

### 7.4 补齐审计联查与 runbook

至少应能按：

- 订单号
- 案件号
- 模板编码

联查：

- 发送记录
- 渲染变量
- 渠道结果
- 重试轨迹
- 关联事件

并在 `docs/04-runbooks/notification-worker.md` 中说明：

- 事件来源
- 模板清单
- 失败排查
- 人工补发

### 7.5 补齐集成测试

至少应覆盖：

- 支付成功通知
- 交付完成通知
- 验收通过/拒收通知
- 争议升级通知
- 重复事件去重
- 失败重试与 DLQ

## 8. 实施范围

至少覆盖以下内容：

### 8.1 进程与目录

- `apps/notification-worker/**`
- `apps/README.md`
- `services/README.md`
- compose 与启动脚本

### 8.2 事件与运行时

- `dtp.notification.dispatch`
- 通知消费逻辑
- 模板模型
- 渲染与发送记录

### 8.3 provider 边界

- `apps/platform-core/crates/provider-kit/**`
- 通知 provider trait / mock-log 实现

### 8.4 文档与测试

- `docs/04-runbooks/notification-worker.md`
- `docs/05-test-cases/**`
- 通知链路 smoke / integration tests

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 通知进程命名只剩一套正式答案
- 通知目录不再只是占位 README
- compose 不再只是 placeholder
- `provider-kit` 中明确 `mock-log` 为正式 `V1` 实接渠道
- runbook 已存在

### 9.2 动态验证

至少验证：

1. Kafka 事件进入通知进程
2. 模板正确渲染
3. `mock-log` 渠道产生发送结果
4. 重复事件不重复发送
5. 失败事件进入重试 / DLQ
6. 通知发送与重试记录可联查

### 9.3 阶段可证明性

修复后应能明确回答：

- 通知正式进程叫什么
- 监听哪个 topic
- `V1` 正式实接渠道是什么
- 模板、幂等、重试、DLQ、审计联查入口在哪里
- runbook 和集成测试在哪里

若仍有多套答案，则视为未完成收口。

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 正式通知进程命名结论
3. 通知链路说明
4. `V1` 渠道边界说明
5. runbook 与测试新增清单
6. 联调与测试结果

## 11. 一句话结论

`A10` 的核心问题不是“通知还没写”，而是通知阶段的进程命名、运行时链路和渠道边界都还没收口；如果不先统一这些基础口径，进入 `NOTIF` 阶段后会同时在 topic、部署、幂等、DLQ 和审计上失控。
