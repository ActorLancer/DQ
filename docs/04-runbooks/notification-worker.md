# Notification Worker 本地运行与排障（NOTIF-011）

## 口径冻结

- 正式进程名：`notification-worker`
- 对应事件：`notification.requested`
- 正式消费 topic：`dtp.notification.dispatch`
- 本地默认 consumer group：`cg-notification-worker`
- 不直接消费：`dtp.outbox.domain-events`
- `V1` 实接渠道：`mock-log`
- `email` / `webhook`：仅保留 provider 边界，不作为 `V1` 必须实接项

## 当前批次边界

- 本批次只冻结命名、topic、consumer group、渠道边界与排障口径，不代表 `notification-worker` 已完成正式实现。
- 当前文档结论只能回答“通知事件应该怎么走”，不能替代后续代码实现所需的 OpenAPI、发送记录模型、集成测试与 smoke 结果。
- 进入 `NOTIF` 代码实现批次后，Agent 必须同步补齐：
  - `packages/openapi/ops.yaml` 中与通知联查相关的控制面示例（`NOTIF-013`）
  - `docs/02-openapi/ops.yaml` 归档（`NOTIF-013`）
  - `docs/05-test-cases/notification-cases.md`（`NOTIF-014`）
  - runbook 中的模板清单、人工补发步骤、失败重试阈值与联查入口

## 事件来源

- 主来源：`platform-core.integration`
- 冻结链路：`notification.requested -> dtp.notification.dispatch -> notification-worker`
- `dtp.outbox.domain-events` 仅保留为通用主领域事件流，不作为 `notification-worker` 的正式消费入口
- topic 定义权威源：`infra/kafka/topics.v1.json`
- topic 初始化脚本：`infra/kafka/init-topics.sh`

## 本地启动

1. 启动基础设施：
   - `make up-local`
2. 启动通知进程（当前为占位运行时，后续由 NOTIF-001~012 落实真实逻辑）：
   - 参考 `infra/docker/docker-compose.apps.local.example.yml` 中 `notification-worker` 段
3. 校验 topic 已存在：
   - `dtp.notification.dispatch`
   - `dtp.dead-letter`
4. 校验通知 / Fabric 相关关键拓扑未漂移：
   - `./scripts/check-topic-topology.sh`
   - 该脚本只覆盖关键静态 topology / route seed；若要验证全量 canonical topics 是否真实存在，需额外执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`

## V1 渠道与模板边界

- 默认只允许 `mock-log` 渠道输出发送结果
- 模板需支持：
  - 模板编码
  - 变量渲染
  - 版本与启停
- 不允许把内部风控、审计敏感字段直接透传到业务用户通知正文

## 幂等、重试、DLQ

- 同一幂等键事件必须只发送一次
- 失败消息进入重试流程；超过阈值转入 `dtp.dead-letter`
- 人工重放必须保留审计轨迹并可按事件 ID 回查
- 相关 `ops.event_route_policy` 缺失或漂移时，先检查 `notification.dispatch_request / notification.requested -> dtp.notification.dispatch`

## 联查建议

- 按 `order_id` / `case_id` / `template_code` 联查：
  - 发送记录
  - 渲染变量快照
  - 渠道结果
  - 重试轨迹
  - 关联事件 ID

## 常见问题

- 现象：收不到通知
  - 检查 `dtp.notification.dispatch` 是否有消息
  - 检查 consumer group 是否为 `cg-notification-worker`
  - 检查幂等键是否命中去重
- 现象：持续重试
  - 检查模板渲染变量是否完整
  - 检查 provider 返回码与重试策略阈值
