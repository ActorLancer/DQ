# Kafka Topics 本地默认策略（ENV-011）

## Canonical Source

- 机器可读 topic 字典：`infra/kafka/topics.v1.json`
- `infra/kafka/init-topics.sh`、compose 内的一次性 `kafka-topics-init` service、`scripts/smoke-local.sh`、`platform-core` 启动自检默认值都以该文件为基线。

## Topic 列表

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

## Canonical Topology

下表与 `infra/kafka/topics.v1.json` 保持一一对应；topic 的 producer、consumer 与本地默认 consumer group 以此为准。

| Topic | Producer | Consumer | Local Consumer Group |
| --- | --- | --- | --- |
| `dtp.outbox.domain-events` | `outbox-publisher` | `-` | `-` |
| `dtp.search.sync` | `outbox-publisher` | `search-indexer` | `cg-search-indexer` |
| `dtp.recommend.behavior` | `platform-core.recommendation` | `recommendation-aggregator` | `cg-recommendation-aggregator` |
| `dtp.notification.dispatch` | `platform-core.integration` | `notification-worker` | `cg-notification-worker` |
| `dtp.fabric.requests` | `platform-core.integration` | `fabric-adapter` | `cg-fabric-adapter` |
| `dtp.fabric.callbacks` | `fabric-event-listener` | `platform-core.consistency` | `cg-platform-core-consistency` |
| `dtp.payment.callbacks` | `mock-payment-provider` | `platform-core.billing` | `cg-payment-callback-handler` |
| `dtp.audit.anchor` | `platform-core.audit` | `fabric-adapter` | `cg-fabric-adapter` |
| `dtp.consistency.reconcile` | `platform-core.consistency` | `consistency-reconcile-worker` | `cg-consistency-reconcile` |
| `dtp.dead-letter` | `all-consumers` | `dead-letter-replayer` | `cg-dead-letter-replayer` |

## 历史 Topic 映射

| 历史 topic | Canonical topic |
| --- | --- |
| `outbox.events` | `dtp.outbox.domain-events` |
| `search.sync` | `dtp.search.sync` |
| `audit.anchor` | `dtp.audit.anchor` |
| `billing.events` | `dtp.outbox.domain-events` |
| `recommendation.behavior` | `dtp.recommend.behavior` |
| `dead-letter.events` | `dtp.dead-letter` |

## Consumer Group 约定（本地默认）

- 同一 consumer service 在多个 topic 上复用同一组名，例如 `fabric-adapter -> cg-fabric-adapter`。
- `outbox-publisher` 轮询 `ops.outbox_event`，不是 `dtp.outbox.domain-events` 的 Kafka consumer。
- `notification-worker` 与 `fabric-adapter` 的 `V1` 正式消费入口分别是 `dtp.notification.dispatch`、`dtp.audit.anchor` / `dtp.fabric.requests`；两者不再直接消费 `dtp.outbox.domain-events`。
- 新增 topic 订阅者前，必须先在 `topics.v1.json` 中声明 consumer 与 group，再同步 runbook / compose / 实现。

## Topic 策略默认值

- 通用 topic：
  - `cleanup.policy=delete`
  - `retention.ms=604800000`（7 天）
- DLQ topic（`dtp.dead-letter`）：
  - `cleanup.policy=delete`
  - `retention.ms=1209600000`（14 天）

## 初始化脚本

执行：

```bash
infra/kafka/init-topics.sh
```

该脚本会按 `infra/kafka/topics.v1.json` 创建 canonical topic，并下发 `retention.ms` 与 `cleanup.policy` 默认配置。

## Compose 一次性初始化

- `infra/docker/docker-compose.local.yml` 中已内置一次性 `kafka-topics-init` service。
- `make up-local` / `make up-core` / `make up-demo` 会在基础设施启动后通过 `scripts/up-local.sh` 显式执行该 service。
- `kafka-topics-init` 会等待 `kafka` healthy，再调用 `infra/kafka/init-topics.sh` 初始化 canonical topics。
- 本地 Kafka 已关闭 `KAFKA_AUTO_CREATE_TOPICS_ENABLE`；topic 必须来自 `topics.v1.json`，不再允许通过自动创建掩盖命名漂移。

## 关键拓扑校验

执行：

```bash
./scripts/check-topic-topology.sh
```

该脚本校验通知 / Fabric / audit-anchor 相关的关键静态拓扑与当前数据库 `ops.event_route_policy` 运行态，不替代全量 canonical smoke。当前覆盖：

- `dtp.outbox.domain-events`
- `dtp.notification.dispatch`
- `dtp.fabric.requests`
- `dtp.fabric.callbacks`
- `dtp.audit.anchor`
- `notification-worker` / `fabric-adapter` 未被错误登记为 `dtp.outbox.domain-events` 直接 consumer
- 事件模型文档与 runbook 中对应关键行是否仍与 canonical source 对齐
- `ops.event_route_policy` 的 V1 seed 是否覆盖 `notification.requested`、`audit.anchor_requested`、`fabric.proof_submit_requested`
- 当前数据库中是否已存在：
  - `notification.dispatch_request / notification.requested -> dtp.notification.dispatch`
  - `audit.anchor_batch / audit.anchor_requested -> dtp.audit.anchor`
  - `chain.chain_anchor / fabric.proof_submit_requested -> dtp.fabric.requests`

## 全量 canonical smoke 校验

执行：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh
```

该 smoke 会按 `infra/kafka/topics.v1.json` 中 `required_in_smoke == true` 的定义，逐项检查 canonical topics 是否真实存在，并同时覆盖宿主机 / 容器内 Kafka 地址边界。若需要验证全量 topic 真实落盘，不应只运行 `check-topic-topology.sh`。
