# Kafka 本地模式与 Topic 初始化

## 本地模式

- 使用 KRaft 单节点模式。
- 参考片段：`infra/kafka/docker-compose.kafka.local.yml`。
- 容器内 listener：`kafka:9092`
- 宿主机 external listener：`127.0.0.1:9094`
- 局域网 external listener：`${KAFKA_EXTERNAL_ADVERTISED_HOST}:9094`

## 局域网访问

若需要让同一局域网内的其他计算机访问本机 Kafka，必须同时满足：

1. `infra/docker/.env.local` 中保持 `KAFKA_EXTERNAL_BIND_HOST=0.0.0.0`。
2. 将 `KAFKA_EXTERNAL_ADVERTISED_HOST` 改成本机局域网 IP 或可被其他计算机解析的主机名，例如 `192.168.1.20`。
3. 本机防火墙允许 TCP `9094` 入站。
4. 修改 advertised host 后需要重启 Kafka：`make down-local && make up-local`。

远端客户端应连接：

```bash
<本机局域网 IP 或 DNS>:9094
```

不要让远端客户端连接 `127.0.0.1:9094`，也不要使用 `kafka:9092`；前者只指向远端机器自己，后者只在 compose 网络内部有效。

## Topic 初始化

`make up-local` / `make up-core` / `make up-demo` 现在会通过 compose 中的一次性 `kafka-topics-init` service 执行 topic 初始化；该 service 由 `scripts/up-local.sh` 显式调用，会在 `kafka` 健康后读取 `infra/kafka/topics.v1.json`，并调用 `infra/kafka/init-topics.sh` 创建 canonical topics。

如需手动重跑初始化，执行：

```bash
infra/kafka/init-topics.sh
```

默认按 `infra/kafka/topics.v1.json` 初始化以下 canonical topic：

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

初始化完成后建议执行：

```bash
./scripts/check-topic-topology.sh
```

用于校验 topic 字典、冻结文档与 route-policy seed 没有再次漂移。

补充约束：

- 本地 Kafka 已关闭 `auto-create topics`，不允许再依赖“发消息时自动生成 topic”。
- 若 topic 名称或配置未先写入 `infra/kafka/topics.v1.json`，初始化流程和 smoke 校验都应失败，而不是静默放过。
