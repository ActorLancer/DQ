# Local Environment Smoke Cases

`TEST-005` 的目标不是“端口能通”，而是证明当前本地正式运行基线已经收敛到：

- `infra/docker/docker-compose.local.yml`
- `infra/docker/.env.local`
- 宿主机 `platform-core`
- `Prometheus / Alertmanager / Grafana / Loki / Tempo`
- `Keycloak`
- canonical topic catalog `infra/kafka/topics.v1.json`

并且不会再回退到旧的 Kafka 地址、旧 topic 或旧控制面入口。

## 公共前置条件

1. 宿主机运行时入口统一从 `infra/docker/.env.local` 载入：
   ```bash
   set -a
   source infra/docker/.env.local
   set +a
   ```
2. 宿主机 Kafka 一律使用 `127.0.0.1:9094`。
3. 容器内 / compose 网络 Kafka 继续使用 `kafka:9092`，容器内自检可使用 `localhost:9092`。
4. `./scripts/check-topic-topology.sh` 只负责通知 / Fabric / audit-anchor 相关 topology 与 route seed；若要验证全量 canonical topic 真实存在，必须执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`。
5. 本批 smoke 不以旧 topic、局部 outbox 行或只看 HTTP 200 作为通过证据；必须同时回查 compose profile、realm、datasource、canonical topic 与关键控制面入口。

## 正式执行命令

```bash
ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh
```

该命令会自动：

- 拉起 `core + observability + mocks`
- 执行 `db/scripts/migrate-up.sh + db/scripts/seed-up.sh`
- 初始化 MinIO buckets
- 以 `APP_HOST=0.0.0.0` 启动或复用宿主机 `platform-core`，同时继续通过 `127.0.0.1:8094` 访问
- 校验 `check-local-stack.sh full`
- 校验 Keycloak realm、Grafana datasource、canonical topics、Kafka 双地址边界与关键 ops 控制面入口

## 验收矩阵

| Case ID | 场景 | 必须证明的事实 | 最低回查 |
| --- | --- | --- | --- |
| `TEST005-CASE-001` | compose 启动与核心服务 ready | `core + observability + mocks` profile 能真实启动，且 `postgres / redis / kafka / minio / opensearch / keycloak / prometheus / grafana / loki / tempo / mock-payment-provider` 已就绪 | `./scripts/check-local-stack.sh full` |
| `TEST005-CASE-002` | Keycloak realm 导入 | `platform-local` realm 已真实导入，`portal-web` password grant 可用，token 带正式 `platform_admin` 角色与 `user_id / org_id` claims | `./scripts/check-keycloak-realm.sh` |
| `TEST005-CASE-003` | Grafana datasource / dashboard | `Prometheus / Loki / Tempo` datasource 已导入，关键 dashboard 已存在，Prometheus target/规则可查 | `./scripts/check-observability-stack.sh` |
| `TEST005-CASE-004` | canonical topic + topology | `infra/kafka/topics.v1.json` 中 `required_in_smoke=true` 的 canonical topics 均真实存在，通知/Fabric/audit-anchor route policy 与 runtime DB 一致 | `./scripts/check-topic-topology.sh` + `smoke-local.sh` 自带 topic probe |
| `TEST005-CASE-005` | Kafka 双地址边界 | 宿主机 `127.0.0.1:9094` 可做 metadata 查询；容器内 `localhost:9092` 可做 topic 探测；broker advertised listeners 同时保留 `PLAINTEXT://kafka:9092` 与 `EXTERNAL://127.0.0.1:9094` | `docker exec datab-kafka printenv KAFKA_ADVERTISED_LISTENERS`、`docker exec datab-kafka kafka-topics.sh --bootstrap-server localhost:9092 --list`、host-side `kcat -b 127.0.0.1:9094 -L` |
| `TEST005-CASE-006` | platform-core 健康与运行态 | 宿主机 `platform-core` 可启动并通过 `/health/live`、`/health/ready`、`/health/deps`、`/internal/runtime`；依赖回查必须出现 `db / redis / kafka / minio / keycloak reachable=true` | `curl http://127.0.0.1:8094/health/*`、`curl http://127.0.0.1:8094/internal/runtime` |
| `TEST005-CASE-007` | 关键控制面入口 | 关键控制面入口未回退到旧路径；`/api/v1/ops/observability/overview` 与 `/api/v1/ops/outbox` 仍可被正式角色访问并返回统一 envelope | `curl http://127.0.0.1:8094/api/v1/ops/observability/overview`、`curl http://127.0.0.1:8094/api/v1/ops/outbox` |

## 手工回查模板

执行 smoke 后，至少应能复核以下命令：

```bash
docker exec datab-kafka printenv KAFKA_ADVERTISED_LISTENERS
docker exec datab-kafka /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
docker run --rm --network host edenhill/kcat:1.7.1 -b 127.0.0.1:9094 -L
curl -fsS http://127.0.0.1:8094/health/deps
curl -fsS http://127.0.0.1:8094/internal/runtime
curl -fsS -u admin:admin123456 http://127.0.0.1:3000/api/datasources
curl -fsS http://127.0.0.1:8081/realms/platform-local/.well-known/openid-configuration
```

## CI 入口

- `.github/workflows/local-environment-smoke.yml`
- 本地与 CI 统一执行：`ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`
