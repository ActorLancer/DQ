# Compose Profiles（ENV-030）

本地 `docker-compose.local.yml` 采用五类 profile，其中 `mocks` 只是 `local` 下的联调子 profile，不是新的正式运行模式：

- `core`：最小核心栈（PostgreSQL/Redis/Kafka/MinIO/OpenSearch/Keycloak/OTel Collector）
- `observability`：观测栈（Prometheus/Alertmanager/Grafana/Loki/Tempo + exporters）
- `mocks`：本地支付/回执联调所需的 mock provider（当前主要是 `mock-payment-provider`）
- `fabric`：本地 Fabric 测试链占位（CA/Orderer/Peer）
- `demo`：一键演示全量组合（core + observability + fabric + mock-payment）

## 常用命令

```bash
# 默认核心栈
make up-local

# 显式核心栈
make up-core

# 核心 + 观测
make up-observability

# 核心 + mocks（支付/回执联调）
make up-mocks

# 核心 + Fabric
make up-fabric

# 一键演示
make up-demo
```

以上 `make` 命令统一调用 `scripts/up-local.sh`，仅通过 `COMPOSE_PROFILES` 切换服务组合。
