# Kafka Topics 本地默认策略（ENV-011）

## Topic 列表

- `outbox.events`
- `search.sync`
- `audit.anchor`
- `billing.events`
- `recommendation.behavior`
- `dead-letter.events`

## Consumer Group 约定（本地默认）

- `cg-platform-core-outbox`
- `cg-search-sync-worker`
- `cg-audit-anchor-worker`
- `cg-billing-worker`
- `cg-recommendation-worker`
- `cg-dead-letter-replayer`

## Topic 策略默认值

- 通用 topic：
  - `cleanup.policy=delete`
  - `retention.ms=604800000`（7 天）
- DLQ topic（`dead-letter.events`）：
  - `cleanup.policy=delete`
  - `retention.ms=1209600000`（14 天）

## 初始化脚本

执行：

```bash
infra/kafka/init-topics.sh
```

该脚本会创建 topic 并下发 `retention.ms` 与 `cleanup.policy` 默认配置。
