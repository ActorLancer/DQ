# Observability Local 运行说明（ENV-026/027/028/029）

## 启动方式

```bash
docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml --profile observability up -d
```

## 组件清单

- `prometheus`：采集宿主机运行的 `platform-core:8094`、`notification-worker:8097`，以及 `mock-payment-provider`、`kafka-exporter`、`postgres-exporter`、`redis-exporter`、`minio-exporter`、`opensearch-exporter`
- `alertmanager`：接收最小规则集告警（服务不可用、通知重试队列积压、队列积压、DB 连接失败、链适配失败、outbox 重试异常、DLQ 增长）
- `grafana`：预置 `Prometheus/Loki/Tempo` 数据源与 4 组 dashboard
- `loki`：本地持久化目录 `/tmp/loki`（挂载到 `loki_data` 卷），保留期 `168h`
- `tempo`：本地持久化目录 `/tmp/tempo`（挂载到 `tempo_data` 卷），trace 块保留 `72h`
- `host.docker.internal`：在本地 compose 中显式固定到 `host-gateway`，避免 Prometheus / Alertmanager / mock-payment-provider 访问宿主机端口时落到错误解析地址

## 验证命令

```bash
./scripts/check-observability-stack.sh
```

若已在宿主机启动 `platform-core` / `notification-worker`，建议额外执行：

```bash
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=up{job="notification-worker"}'
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=notification_worker_events_total'
curl -fsS -u admin:admin123456 'http://127.0.0.1:3000/api/search?query=Platform%20Overview'
```

## 清理策略

- Loki 与 Tempo 均启用本地数据卷挂载，避免容器删除后元数据丢失。
- 通过保留周期控制长期调试占用，超出周期后由 compactor 清理历史数据。
