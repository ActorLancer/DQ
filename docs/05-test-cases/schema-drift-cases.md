# TEST-017 Schema Drift 正式清单

## 目标

- 将 `TEST-017` 收口为单一正式入口，持续拦截三类漂移：
  - `migration / .sqlx / query compile` 漂移
  - `db::entity` 受管 catalog 与 live DB 真表漂移
  - `packages/openapi/**` 与 `docs/02-openapi/**` 归档漂移
- 本地与 CI 统一复用 `./scripts/check-schema-drift.sh`，不在 workflow 内另写第二套 schema 检查命令。

## 正式入口

- 本地 / CI 统一入口：
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-schema-drift.sh`
- CI workflow：
  - `.github/workflows/schema-drift.yml`

## 收口路径

| 阶段 | 正式命令 | 验收重点 |
| --- | --- | --- |
| Core stack 前置 | `COMPOSE_PROFILES=core ./scripts/up-local.sh` + `./scripts/check-local-stack.sh core` + `./db/scripts/migrate-up.sh` | checker 需要自带最小前置，不能假设开发者已经先手工拉起 PostgreSQL / Keycloak。 |
| SQLx drift | `cargo sqlx prepare --workspace --check` | 当前 migration / 查询 / `.sqlx` metadata 必须一致，若生成结果需要写盘则直接失败。 |
| Query compile | `./scripts/check-query-compile.sh` | 已提交的 `.sqlx` 离线元数据必须仍能支撑 `db` crate 的离线编译。 |
| Entity catalog | `db::entity` live catalog check | `apps/platform-core/crates/db/src/entity/**` 当前受管 catalog 必须继续对齐 `keycloak.public`，并额外确认 `datab.public.schema_migration_history` 存在。 |
| OpenAPI drift | `./scripts/check-openapi-schema.sh` | `packages/openapi/**` 与 `docs/02-openapi/**` 必须同步，关键业务路径和 control-plane token 不得回退到旧命名或骨架接口。 |

## 验收矩阵

| Case ID | 场景 | 必须证明的事实 | 最低回查 |
| --- | --- | --- | --- |
| `TEST017-CASE-001` | SQLx metadata 不漂移 | `cargo sqlx prepare --workspace --check` 在当前迁移基线上通过；若存在未提交 `.sqlx` 变更会直接失败。 | `cargo sqlx prepare --workspace --check` |
| `TEST017-CASE-002` | 离线 query compile 不漂移 | 已提交 `.sqlx` 元数据仍能支撑 `db` crate 的 `SQLX_OFFLINE` 编译。 | `./scripts/check-query-compile.sh` |
| `TEST017-CASE-003` | 受管 entity catalog 不漂移 | `db::entity` catalog 与 `keycloak.public` 真表保持一致；`schema_migration_history` 继续存在于 `datab.public`。 | `target/test-artifacts/schema-drift/entity-table-catalog.txt`、`keycloak-public-tables.txt`、`datab-public-tables.txt` |
| `TEST017-CASE-004` | OpenAPI 归档不漂移 | `packages/openapi/**` 与 `docs/02-openapi/**` 同步，关键业务路径和 token 仍存在。 | `./scripts/check-openapi-schema.sh` |
| `TEST017-CASE-005` | 失败可定位 | checker 与 workflow 会留下实体/表清单 artifact，便于定位是 SQLx、entity 还是 OpenAPI 漂移。 | `.github/workflows/schema-drift.yml` artifact |

## 边界

- `db::entity` drift 只覆盖当前受管的 SeaORM codegen catalog：
  - `keycloak.public` 全表
  - `datab.public.schema_migration_history`
- `TEST-017` 不要求把业务库 `audit/billing/catalog/...` 全部生成成 SeaORM entity；那不是当前仓库的正式 authority。
- `TEST-017` 也不替代 `TEST-004` migration smoke、`TEST-015` 最小 CI 矩阵或 `TEST-028` canonical contract checker；它只负责 schema / metadata / archive 这条 drift gate。
