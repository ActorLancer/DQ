# Compose Profiles（ENV-030）

本地 `docker-compose.local.yml` 采用四类 profile：

- `core`：最小核心栈（PostgreSQL/Redis/Kafka/MinIO/OpenSearch/Keycloak/OTel Collector）
- `observability`：观测栈（Prometheus/Alertmanager/Grafana/Loki/Tempo + exporters）
- `fabric`：本地 Fabric 测试链占位（CA/Orderer/Peer）
- `demo`：一键演示全量组合（core + observability + fabric + mock-payment）

## 常用命令

```bash
# 默认核心栈
make up-local

# 核心 + 观测
COMPOSE_PROFILES=core,observability docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml up -d

# 核心 + Fabric
COMPOSE_PROFILES=core,fabric docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml up -d

# 一键演示
COMPOSE_PROFILES=demo docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml up -d
```
