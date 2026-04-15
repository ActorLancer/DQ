# Kafka 本地模式与 Topic 初始化

## 本地模式

- 使用 KRaft 单节点模式。
- 参考片段：`infra/kafka/docker-compose.kafka.local.yml`。

## Topic 初始化

执行：

```bash
infra/kafka/init-topics.sh
```

默认初始化以下 topic：

- `outbox.events`
- `search.sync`
- `audit.anchor`
- `billing.events`
- `recommendation.behavior`
- `dead-letter.events`
