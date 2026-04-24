# Canonical Contracts Cases

`TEST-028` 的正式目标，不是再补一条“topic 存在”检查，而是把 canonical topic、正式 consumer group、正式接口/OpenAPI、验收矩阵、宿主机/容器 Kafka 双地址边界和 checker 职责边界收口成一个唯一官方 gate。

## Authority

- 任务执行源：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- topic / consumer group authority：`infra/kafka/topics.v1.json`
- topic / route / 边界 runbook：`docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/local-startup.md`、`docs/04-runbooks/port-matrix.md`
- 验收矩阵 authority：`docs/05-test-cases/README.md`、`docs/05-test-cases/v1-core-acceptance-checklist.md`
- 问题收口 authority：`docs/开发任务/问题修复任务/A01-Kafka-Topic-口径统一.md`、`docs/开发任务/问题修复任务/A11-测试与Smoke口径误报风险.md`

## Official Commands

- 本地全量：`ENV_FILE=infra/docker/.env.local ./scripts/check-canonical-contracts.sh`
- CI 静态：`CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`
- artifact：`target/test-artifacts/canonical-contracts/summary.json`

## Boundary

- 宿主机 Kafka 正式边界固定为 `127.0.0.1:9094`
- 容器内 / compose 网络继续使用 `kafka:9092`
- 容器内自检可使用 `localhost:9092`
- `./scripts/check-topic-topology.sh` 只负责通知 / Fabric / audit-anchor 相关关键静态 topology 与 route seed，不替代全量 canonical topic existence smoke
- 若要验证 `infra/kafka/topics.v1.json` 中全部 `required_in_smoke=true` 的 canonical topics 真实存在，必须执行 `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`

## Case Matrix

| Case ID | 目标 | 正式命令 / 证据 | 通过判定 |
| --- | --- | --- | --- |
| `TEST028-CASE-001` | checker authority | `docs/05-test-cases/README.md`、`v1-core-acceptance-checklist.md`、`scripts/README.md`、`.github/workflows/README.md`、`.github/workflows/canonical-contracts.yml` | `ACC-CANONICAL`、官方脚本说明、官方 workflow 全部指向同一个 `check-canonical-contracts.sh` 入口；CI 明确使用 `CANONICAL_CHECK_MODE=static` 并上传 artifact |
| `TEST028-CASE-002` | canonical topic / consumer group / env binding | `infra/kafka/topics.v1.json`、`docs/开发准备/事件模型与Topic清单正式版.md`、`docs/04-runbooks/kafka-topics.md`、`docs/04-runbooks/port-matrix.md` | topic catalog、runbook 和 env binding 一一对齐；不得存在第二份活跃 topic manifest |
| `TEST028-CASE-003` | host / container Kafka boundary | `docs/04-runbooks/local-startup.md`、`docs/04-runbooks/port-matrix.md`、`workers/outbox-publisher/src/main.rs`、`workers/search-indexer/src/main.rs`、`workers/recommendation-aggregator/src/main.rs` | 宿主机边界固定 `127.0.0.1:9094`；容器边界固定 `kafka:9092` / `localhost:9092`；正式 docs / scripts / host-run worker 默认值中不得出现宿主机 `127.0.0.1:9092` 或 `localhost:9094` |
| `TEST028-CASE-004` | topology checker 边界 | `./scripts/check-topic-topology.sh`、`docs/05-test-cases/README.md`、本文件 | 文档必须明确 `check-topic-topology.sh` 仅覆盖 notification/fabric/audit-anchor 静态 topology；不能把它误报为全量 canonical smoke |
| `TEST028-CASE-005` | full canonical smoke 边界 | `ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh`、`summary.json` | full 模式会执行 `check-openapi-schema.sh + check-topic-topology.sh + smoke-local.sh`；若 canonical topics 不存在、route policy 漂移或 runtime stack 未起来，则 checker 必须失败 |
| `TEST028-CASE-006` | legacy drift guard | `summary.json`、`check-canonical-contracts.sh` 静态扫描输出 | 正式 docs / scripts / workflow / OpenAPI 中不得残留 `outbox.events`、`search.sync`、`billing.events`、`recommendation.behavior`、`dead-letter.events`、`notification-service` 作为运行态默认值 |

## Explicit Failure Conditions

以下任一情况都必须让 `TEST-028` 失败：

- 把宿主机 Kafka 写成 `127.0.0.1:9092`
- 把宿主机 Kafka 写成 `localhost:9094`
- 把 `check-topic-topology.sh` 写成“全量 canonical smoke”
- 仍然保留第二份活跃 topic manifest
- 正式 docs / scripts / workflow 中仍使用旧 topic 或旧服务命名作为运行态默认值
- `ACC-CANONICAL` 不再指向 `check-canonical-contracts.sh`

## Verification Notes

- 本地 `full` 模式必须真实跑到 `smoke-local.sh`，不能只跑静态检查
- CI `static` 模式不替代本地 `full` 模式；它的职责是尽早拦截 OpenAPI / 文档 / topic catalog / boundary 漂移
- `summary.json` 必须保留：
  - 当前模式 `full/static`
  - checker 状态
  - 最后执行到的检查阶段
  - 官方命令与边界说明
