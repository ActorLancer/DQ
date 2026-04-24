# TEST-016 Compose CI Smoke 正式清单

## 目标

- 将 `TEST-016` 收口为单一正式入口：
  - 先真实拉起 `core + observability + mocks`
  - 再执行 `platform-core` 健康、控制面与 canonical topic 运行态回查
  - 最后继续拦截 canonical topic / consumer group / OpenAPI 归档的静态漂移
- 本地与 CI 统一复用 `./scripts/check-compose-smoke.sh`，避免 workflow 内散落第二套命令。

## 正式入口

- 本地 / CI 统一入口：
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-compose-smoke.sh`
- CI workflow：
  - `.github/workflows/local-environment-smoke.yml`

## 收口路径

| 阶段 | 正式命令 | 验收重点 |
| --- | --- | --- |
| Compose 运行态 smoke | `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh` | 必须真实拉起 `core + observability + mocks`，并完成 `check-local-stack/full`、Keycloak realm、Grafana datasource、canonical topic、Kafka 双地址边界与关键 ops 控制面入口回查。 |
| Canonical 静态漂移拦截 | `CANONICAL_CHECK_MODE=static ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh` | 必须继续拦截 consumer group catalog 漂移、关键 OpenAPI / 文档归档缺失或回退到旧命名 / 骨架接口。 |
| CI artifact 留存 | workflow `always()` 分支 | 必须保留 compose 日志、`docker compose ps` 与 `platform-core` log，便于定位启动失败、topic 漂移或 OpenAPI 漂移。 |

## 验收矩阵

| Case ID | 场景 | 必须证明的事实 | 最低回查 |
| --- | --- | --- | --- |
| `TEST016-CASE-001` | Compose 启动与健康检查 | CI 内能真实启动 `core + observability + mocks`，并通过 `platform-core` live/ready/deps/runtime 与关键 ops 控制面探针。 | `./scripts/smoke-local.sh` |
| `TEST016-CASE-002` | Canonical topic 运行态存在性 | `topics.v1.json` 中 `required_in_smoke=true` 的 canonical topics 在 compose 内真实存在，且宿主机/容器 Kafka 双地址边界未漂移。 | `./scripts/smoke-local.sh` |
| `TEST016-CASE-003` | Consumer group / route authority 静态收口 | canonical topic catalog 的 `consumer_groups` 与 `consumers` 对齐，通知 / Fabric / audit-anchor 的正式拓扑与 route-policy seed 没有回退。 | `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh` |
| `TEST016-CASE-004` | 关键 OpenAPI 归档不漂移 | `packages/openapi/**` 与 `docs/02-openapi/**` 的关键业务路径仍在，未回退到旧命名或只剩骨架接口。 | `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh` |
| `TEST016-CASE-005` | 失败可定位 | workflow 失败后仍可下载 compose / app artifacts，定位启动失败、topic 缺失或 OpenAPI 漂移。 | `.github/workflows/local-environment-smoke.yml` artifact |

## 边界

- `TEST-016` 不替代 `TEST-005` 的运行态 smoke；它是在 CI 内复用 `smoke-local.sh`，并追加 canonical 静态漂移拦截。
- `TEST-016` 也不替代 `TEST-028` 的 full canonical checker；当前 compose 作业只跑 `CANONICAL_CHECK_MODE=static`，避免在同一作业里重复拉起第二遍 full smoke。
- `check-topic-topology.sh` 仍只负责通知 / Fabric / audit-anchor 相关 topology 与 route seed 静态校验；全量 canonical topic existence 仍以 `smoke-local.sh` 为准。
