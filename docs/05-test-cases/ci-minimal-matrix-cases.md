# TEST-015 CI 最小矩阵正式清单

## 目标

- 将 `TEST-015` 的 CI 最小矩阵收口为单一正式入口：
  - Rust lint/test
  - TypeScript lint/test
  - Go build/test
  - migration check
  - OpenAPI check
- 本地与 CI 统一复用 `./scripts/check-ci-minimal-matrix.sh`，避免 workflow 直接散落第二套命令。

## 正式入口

- 本地全量：
  - `./scripts/check-ci-minimal-matrix.sh all`
- 分 lane 执行：
  - `./scripts/check-ci-minimal-matrix.sh rust`
  - `./scripts/check-ci-minimal-matrix.sh ts`
  - `./scripts/check-ci-minimal-matrix.sh go`
  - `./scripts/check-ci-minimal-matrix.sh migration`
  - `./scripts/check-ci-minimal-matrix.sh openapi`
- CI：
  - `.github/workflows/ci-minimal-matrix.yml`

## Lane 范围

| Lane | 正式命令 | 验收重点 |
| --- | --- | --- |
| Rust | `cargo fmt --all --check && cargo check -p platform-core && cargo test -p platform-core` | Rust 基础静态校验与主服务测试回归必须在 CI 内稳定通过。 |
| TypeScript | `pnpm lint && pnpm typecheck && pnpm --filter @datab/sdk-ts test && pnpm --filter @datab/portal-web test:unit && pnpm --filter @datab/console-web test:unit` | TS 最小矩阵只跑 lint/typecheck/unit test，不把 `WEB-018/TEST-006` 的 live Playwright E2E 混入本 lane。 |
| Go | `go build ./... && go test ./...`（覆盖 `services/fabric-adapter`、`services/fabric-event-listener`、`services/fabric-ca-admin`、`infra/fabric/chaincode/datab-audit-anchor`） | 一方 Go 模块与链码至少具备可编译、可测试的最小基线。 |
| migration | `ENV_FILE=infra/docker/.env.local ./scripts/check-migration-smoke.sh` | 必须复用 `TEST-004` 正式 migration smoke，不得退回历史 `部署脚本/docker-compose.postgres-test.yml`。 |
| OpenAPI | `./scripts/check-openapi-schema.sh` | 正式 OpenAPI 与 `docs/02-openapi/**` 同步、关键 control-plane token 存在、旧口径不回流。 |

## 失败定位约定

- `rust` lane 失败：优先查看 `cargo fmt/check/test` 的第一条报错，不把 warning 误报为 blocker。
- `ts` lane 失败：优先区分 `eslint`、`typecheck`、`sdk-ts`、`portal-web`、`console-web` 的具体子命令。
- `go` lane 失败：必须能直接定位到具体模块路径，不允许只看到“Go 失败”。
- `migration` lane 失败：以 `check-migration-smoke.sh` 的 `platform-core` 日志、`seed_history`、health/runtime probe 为准。
- `openapi` lane 失败：以 `check-openapi-schema.sh` 的具体文件/路径/token 报错为准。

## 边界

- `TEST-015` 是“最小矩阵”，不是替代 `TEST-004/005/006/007...` 各自的专项 workflow。
- TS lane 不负责 live Playwright、浏览器 smoke、前后端联动闭环；这仍由 `WEB-018/020` 和 `TEST-006` 承接。
- Go lane 只做 build/test 基线，不替代 Fabric local/live smoke。
- migration lane 必须继续使用 repo 正式 env，宿主机 Kafka 口径固定为 `127.0.0.1:9094`。
