# 事件模型与 Topic 清单正式版

## 1. 文档定位

本文件用于冻结当前阶段平台的事件模型、Kafka Topic 清单、生产者与消费者边界、事件 key 规则、payload 版本规则、dead letter 规则和回放边界。

本文件服务于以下工作：

- outbox 实现
- Kafka Topic 创建
- 消费者实现
- 搜索与推荐同步
- 审计扩展动作
- 链上链下状态回写
- dead letter 与 replay 实现

本文件不替代业务状态机，不替代数据库设计，不替代 OpenAPI 文件。

阅读入口约束：

- 当前阶段的事件冻结以 `../全集成文档/数据交易平台-全集成基线-V1.md` 为主阅读入口。
- 本文件只冻结 `V1` 正式 topic、producer、consumer 和 replay 规则；`V2/V3` 只保留扩展边界。

上位文档：

- [数据交易平台-全集成基线-V1.md](../全集成文档/数据交易平台-全集成基线-V1.md)
- [服务清单与服务边界正式版.md](../开发准备/服务清单与服务边界正式版.md)
- [接口清单与OpenAPI-Schema冻结表.md](../开发准备/接口清单与OpenAPI-Schema冻结表.md)
- [一致性与事件接口协议正式版.md](../数据库设计/接口协议/一致性与事件接口协议正式版.md)
- [审计、证据链与回放接口协议正式版.md](../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md)
- [商品搜索、排序与索引同步接口协议正式版.md](../数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md)
- [商品推荐与个性化发现接口协议正式版.md](../数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md)

## 2. 事件模型总原则

### 2.1 主状态原则

- 业务主状态权威在 `PostgreSQL`
- 事件是状态传播与副作用触发机制，不是主状态机
- 任何消费者都不得仅凭事件跳过主库校验直接定义主业务状态

### 2.2 统一事件分层

当前统一采用四层事件：

1. 领域事件
2. 集成事件
3. 外部事实事件
4. 审计事件

### 2.3 统一事件字段

所有正式事件 payload 顶层必须至少包含：

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

正式写入 `ops.outbox_event` 的 canonical envelope 在 `V1` 还应补齐以下顶层字段：

- `event_schema_version`
- `authority_scope`
- `source_of_truth`
- `proof_commit_policy`

补充约束：

- `event_name` 不再作为正式 outbox payload 顶层字段使用
- 统一命名采用稳定点分层级风格，优先使用 `domain.object.action`，如 `trade.order.created`、`billing.event.recorded`、`search.product.changed`
- 已冻结事件名以各域接口协议、`ops.event_route_policy` 和正式测试断言为准，不允许模块自行再发明别名

### 2.4 路由权威源原则

- `ops.event_route_policy` 是 `V1` 运行时唯一正式 route authority
- `target_bus`、`target_topic`、`partition_key`、`ordering_key` 必须由应用层 canonical outbox writer 结合 `ops.event_route_policy` 解析
- 不允许继续依赖 `schema.table -> target_topic` 自动派生作为正式主链路
- `common.tg_write_outbox()` 已退役，只允许作为历史迁移说明存在，不得继续作为正式事件生产入口

### 2.5 事件版本原则

- 所有事件类型必须带 `event_version`
- 新增字段允许向后兼容
- 变更字段语义必须升版本，不允许静默改义
- 消费者必须按 `event_type + event_version` 解析

## 3. 事件分类冻结

## 3.1 领域事件

由 `platform-core` 主模块在业务事务后通过 outbox 产生。

典型事件：

- `product.created`
- `product.updated`
- `product.submitted`
- `sku.created`
- `sku.updated`
- `trade.order.created`
- `order.state_changed`
- `contract.confirmed`
- `authorization.granted`
- `authorization.revoked`
- `delivery.created`
- `delivery.completed`
- `acceptance.passed`
- `acceptance.rejected`
- `billing.event.recorded`
- `settlement.created`
- `settlement.completed`
- `dispute.created`
- `dispute.resolved`

## 3.2 集成事件

由 outbox publisher 投递到 Kafka，供外围进程消费。

典型事件：

- `search.sync_requested`
- `recommend.behavior_recorded`
- `notification.requested`
- `audit.anchor_requested`
- `fabric.proof_submit_requested`
- `consistency.reconcile_requested`

## 3.3 外部事实事件

来源于外围进程或外部系统回执。

典型事件：

- `payment.provider_callback_received`
- `payment.intent_succeeded`
- `payment.intent_failed`
- `refund.completed`
- `payout.completed`
- `fabric.commit_confirmed`
- `fabric.commit_failed`
- `storage.object_verified`
- `storage.object_unavailable`
- `ca.certificate_issued`
- `ca.certificate_revoked`

## 3.4 审计事件

由 `platform-core.audit` 按动作落审计，不作为异步副作用的唯一替代机制。

典型事件：

- `audit.trace_recorded`
- `audit.package_exported`
- `audit.replay_requested`
- `audit.replay_completed`
- `audit.legal_hold_created`

## 4. Topic 命名规范

统一使用：

`dtp.<domain>.<stream>`

示例：

- `dtp.outbox.domain-events`
- `dtp.search.sync`
- `dtp.recommend.behavior`
- `dtp.notification.dispatch`
- `dtp.fabric.requests`
- `dtp.fabric.callbacks`
- `dtp.payment.callbacks`
- `dtp.audit.anchor`
- `dtp.consistency.reconcile`
- `dtp.dead-letter`

## 5. V1 Topic 清单

## 5.1 主 Topic

| Topic | 生产者 | 消费者 | 本地默认消费组 | 用途 |
|---|---|---|---|---|
| `dtp.outbox.domain-events` | `outbox-publisher` | `notification-worker` / `fabric-adapter` | `cg-notification-worker` / `cg-fabric-adapter` | 主领域事件分发 |
| `dtp.search.sync` | `outbox-publisher` | `search-indexer` | `cg-search-indexer` | 搜索投影同步 |
| `dtp.recommend.behavior` | `platform-core.recommendation` | `recommendation-aggregator` | `cg-recommendation-aggregator` | 推荐行为回流 |
| `dtp.notification.dispatch` | `platform-core.integration` | `notification-worker` | `cg-notification-worker` | 通知分发 |
| `dtp.fabric.requests` | `platform-core.integration` | `fabric-adapter` | `cg-fabric-adapter` | Fabric 提交请求 |
| `dtp.fabric.callbacks` | `fabric-event-listener` | `platform-core.consistency` | `cg-platform-core-consistency` | Fabric 提交与事件回执 |
| `dtp.payment.callbacks` | `mock-payment-provider` | `platform-core.billing` | `cg-payment-callback-handler` | 支付回调事实 |
| `dtp.audit.anchor` | `platform-core.audit` | `fabric-adapter` | `cg-fabric-adapter` | 审计锚定请求 |
| `dtp.consistency.reconcile` | `platform-core.consistency` | `consistency-reconcile-worker` | `cg-consistency-reconcile` | 一致性修复请求 |
| `dtp.dead-letter` | 各 consumer | `dead-letter-replayer` | `cg-dead-letter-replayer` | 死信隔离 |

## 5.2 补充说明

- `dtp.outbox.domain-events` 是主分发流，不是最终消费者业务落库的唯一依据。
- `dtp.outbox.domain-events` 的正式写入来源是应用层 canonical outbox writer，不再允许数据库触发器自动派生 topic 后直接落主链路。
- `ops.event_route_policy` 是 `dtp.outbox.domain-events` 及其下游分发规则的唯一运行时 authority；topic / key / ordering 变更必须先更新该表，再更新实现与 runbook。
- `infra/kafka/topics.v1.json` 是 producer / consumer / consumer group 的机器可读 canonical source；冻结文档、compose、脚本与迁移 seed 必须与其同步。
- 搜索和推荐可以使用自己的下游 topic，但不得绕过主事件版本规范。
- `dtp.dead-letter` 是统一死信流，具体失败原因放在 payload 内字段，不再额外为每域创建独立 DLQ 主题。
- 若后续新增下游订阅者，必须先补齐唯一进程命名、`topics.v1.json`、runbook 与 `ops.event_route_policy`，再开始实现。

## 6. 事件 Key 规则

## 6.1 主规则

Kafka key 默认使用：

- 与业务聚合直接相关的对象：`aggregate_id`
- 批处理类事件：`batch_id`
- 外部事实回调类事件：`provider_event_id` 或 `external_ref_id`

## 6.2 各类事件推荐 key

| 事件类型 | 推荐 key |
|---|---|
| `product.*` | `product_id` |
| `sku.*` | `sku_id` |
| `order.*` | `order_id` |
| `contract.*` | `contract_id` |
| `authorization.*` | `authorization_id` |
| `delivery.*` | `delivery_id` |
| `billing.event.recorded` | `billing_event_id` |
| `settlement.*` | `settlement_id` |
| `dispute.*` | `dispute_id` |
| `payment.*` | `payment_intent_id` 或 `provider_event_id` |
| `fabric.*` | `anchor_object_id` 或 `tx_request_id` |
| `audit.*` | `audit_id` 或 `anchor_batch_id` |

## 7. Outbox 冻结规则

## 7.1 outbox_event 最小字段

正式最小字段：

- `outbox_event_id`
- `aggregate_type`
- `aggregate_id`
- `event_type`
- `payload`
- `status`
- `request_id`
- `trace_id`
- `idempotency_key`
- `event_schema_version`
- `authority_scope`
- `source_of_truth`
- `proof_commit_policy`
- `target_bus`
- `target_topic`
- `partition_key`
- `ordering_key`
- `payload_hash`
- `retry_count`
- `next_retry_at`
- `created_at`

## 7.2 路由与写入 authority

- 所有正式 outbox 事件必须由应用层统一 writer 写入，禁止模块各自拼装私有 envelope 后直接落库
- writer 必须先查询 `ops.event_route_policy`，再决定 `target_bus`、`target_topic`、`partition_key`、`ordering_key`
- 若 `ops.event_route_policy` 缺失对应 `(aggregate_type, event_type)` 激活路由，写入必须失败，不允许回退到猜测 topic
- 不允许同一对象同时依赖触发器自动写 outbox 和应用层手工写 outbox 两条正式主链路

## 7.3 publisher 规则

- publisher 必须批量拉取
- 必须支持 `SKIP LOCKED`
- 必须限制最大重试次数
- 超限后转入 `dead_letter_event`

## 7.4 消费幂等规则

消费者必须至少使用以下一个维度做幂等：

- `event_id`
- `provider_event_id`
- `request_id + aggregate_id + event_type`

禁止：

- 单纯依赖“消费者自己记得处理过”

## 8. Dead Letter 冻结规则

## 8.1 dead_letter_event 最小字段

- `dead_letter_event_id`
- `source_topic`
- `event_id`
- `event_type`
- `failure_stage`
- `failure_reason`
- `request_id`
- `trace_id`
- `reprocess_status`
- `created_at`

## 8.2 reprocess 规则

- 默认 `dry_run`
- 需要 step-up
- 必须审计
- 不允许绕过主库状态校验直接二次执行副作用

## 9. 各域生产/消费边界

## 9.1 `search`

生产：

- `search.sync_requested`

消费：

- 商品变更
- SKU 变更
- 上架状态变化

约束：

- 最终索引写入前必须重新读取主库投影

## 9.2 `recommend`

生产：

- `recommend.behavior_recorded`

消费：

- 曝光事件
- 点击事件
- 成交事件
- 商品变更事件

约束：

- 推荐返回前必须做主库业务校验

## 9.3 `billing`

生产：

- `billing.event_recorded`
- `settlement.created`
- `settlement.completed`

消费：

- 支付回调
- 退款回调
- 打款回调

约束：

- 支付成功前不得放行交付
- 回调验签前不得更新订单支付终态

## 9.4 `audit`

生产：

- `audit.anchor_requested`
- `audit.package_exported`
- `audit.replay_requested`

消费：

- 关键业务动作完成事件

约束：

- replay 默认 `dry_run`
- 导出与回放动作本身必须被审计

## 9.5 `fabric integration`

生产：

- `fabric.commit_confirmed`
- `fabric.commit_failed`

消费：

- 锚定请求
- 授权摘要提交请求
- 结算摘要提交请求

约束：

- 只回写可信确认类字段
- 不得直接定义业务主状态

## 10. 事件 Payload 示例

## 10.1 订单创建事件

```json
{
  "event_id": "evt_01J...",
  "event_type": "trade.order.created",
  "event_version": 1,
  "occurred_at": "2026-04-13T10:00:00Z",
  "producer_service": "platform-core.order",
  "aggregate_type": "trade.order",
  "aggregate_id": "ord_01J...",
  "request_id": "req_01J...",
  "trace_id": "tr_01J...",
  "idempotency_key": "order:create:ord_01J...",
  "payload": {
    "order_id": "ord_01J...",
    "sku_id": "sku_01J...",
    "buyer_party_id": "pty_buyer",
    "seller_party_id": "pty_seller",
    "current_state": "created",
    "order_amount": "1200.00",
    "currency_code": "USD"
  }
}
```

## 10.2 支付成功回调事件

```json
{
  "event_id": "evt_01J...",
  "event_type": "payment.intent_succeeded",
  "event_version": 1,
  "occurred_at": "2026-04-13T10:05:00Z",
  "producer_service": "mock-payment-provider",
  "aggregate_type": "PaymentIntent",
  "aggregate_id": "pay_01J...",
  "request_id": "req_01J...",
  "trace_id": "tr_01J...",
  "idempotency_key": "payment:callback:provider_evt_01",
  "payload": {
    "payment_intent_id": "pay_01J...",
    "provider_event_id": "provider_evt_01",
    "payment_status": "succeeded",
    "payment_amount": "1200.00",
    "currency_code": "USD",
    "order_id": "ord_01J..."
  }
}
```

## 11. V2 / V3 增量冻结

## 11.1 V2 增量

新增可接入：

- 联邦任务事件
- 计算任务事件
- 证明材料事件
- 分润事件

但仍须遵守：

- 主状态在主库
- 统一事件字段
- 统一 DLQ

## 11.2 V3 增量

新增可接入：

- 跨链请求事件
- FX 报价与结算路由事件
- 伙伴平台回执事件

但仍须遵守：

- 外部事实只能回写镜像状态
- 不得通过跨链回执直接定义本地主状态

## 12. 当前不进入本文件的内容

当前不在本文件中展开：

- Kafka 分区数与保留时长最终运维值
- 每个 topic 的 ACL 细则
- 事件 protobuf/avro 细稿
- CDC / Debezium 方案

这些内容可在后续部署与运维文件中细化。

## 13. 一句话结论

当前阶段应把事件系统理解为：

**以 outbox 为主入口、以 Kafka 为分发总线、以统一事件 schema 和统一 dead letter 规则为基础，把搜索、推荐、审计、通知、Fabric 与支付回调等副作用统一到一套可追踪、可重放、可对账的异步模型中。**
