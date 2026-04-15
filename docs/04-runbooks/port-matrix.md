# Port Matrix（ENV-038）

## 说明

- 本矩阵用于统一本地环境端口、URL、默认账号口径。
- 所有默认值来源于 `infra/docker/.env.local` 与 `infra/docker/docker-compose.local.yml`。
- 凭据仅用于本地演示；禁止复用于共享/生产环境。

## 服务端口与访问矩阵

| 服务 | 本地端口 | URL / 端点 | 默认用户名 | 默认密码 |
| --- | --- | --- | --- | --- |
| PostgreSQL | `5432` | `postgresql://127.0.0.1:5432/datab` | `datab` | `datab_local_pass` |
| Redis | `6379` | `redis://127.0.0.1:6379` | `N/A` | `datab_redis_pass` |
| Kafka (external) | `9094` | `127.0.0.1:9094` | `N/A` | `N/A` |
| MinIO API | `9000` | `http://127.0.0.1:9000` | `datab` | `datab_local_pass` |
| MinIO Console | `9001` | `http://127.0.0.1:9001` | `datab` | `datab_local_pass` |
| OpenSearch | `9200` | `http://127.0.0.1:9200` | `N/A` | `Admin123!Admin123!`（admin init password） |
| Keycloak | `8081` | `http://127.0.0.1:8081` | `admin` | `admin123456` |
| OTel Collector Health | `13133` | `http://127.0.0.1:13133/` | `N/A` | `N/A` |
| OTel Collector Metrics | `8889` | `http://127.0.0.1:8889/metrics` | `N/A` | `N/A` |
| Prometheus | `9090` | `http://127.0.0.1:9090` | `N/A` | `N/A` |
| Alertmanager | `9093` | `http://127.0.0.1:9093` | `N/A` | `N/A` |
| Grafana | `3000` | `http://127.0.0.1:3000` | `admin` | `admin123456` |
| Loki | `3100` | `http://127.0.0.1:3100` | `N/A` | `N/A` |
| Tempo | `3200` | `http://127.0.0.1:3200` | `N/A` | `N/A` |
| Mock Payment | `8089` | `http://127.0.0.1:8089` | `N/A` | `N/A` |

## 初始 Bucket 矩阵

| Bucket Key | 默认值 |
| --- | --- |
| `BUCKET_RAW_DATA` | `raw-data` |
| `BUCKET_PREVIEW_ARTIFACTS` | `preview-artifacts` |
| `BUCKET_DELIVERY_OBJECTS` | `delivery-objects` |
| `BUCKET_REPORT_RESULTS` | `report-results` |
| `BUCKET_EVIDENCE_PACKAGES` | `evidence-packages` |
| `BUCKET_MODEL_ARTIFACTS` | `model-artifacts` |

## 初始 Topic 矩阵

| Topic Key | 默认值 |
| --- | --- |
| `TOPIC_OUTBOX_EVENTS` | `outbox.events` |
| `TOPIC_SEARCH_SYNC` | `search.sync` |
| `TOPIC_AUDIT_ANCHOR` | `audit.anchor` |
| `TOPIC_BILLING_EVENTS` | `billing.events` |
| `TOPIC_RECOMMENDATION_BEHAVIOR` | `recommendation.behavior` |
| `TOPIC_DEAD_LETTER_EVENTS` | `dead-letter.events` |

## 参考命令

- 导出当前配置快照：`./scripts/export-local-config.sh`
- 核心健康检查：`ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`
- 全量 smoke 套件：`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
