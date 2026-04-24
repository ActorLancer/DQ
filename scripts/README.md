# scripts 目录说明

## 职责

- 提供本地开发、联调、校验、重置等统一脚本入口。

## 边界

- 脚本用于编排和检查，不替代业务模块实现。

## 依赖

- 默认依赖 `infra/docker/docker-compose.local.yml` 与根 `Makefile` 目标；旧 `部署脚本/` 目录仅视为历史兼容资产。

## 常用脚本

- `up-local.sh` / `down-local.sh`：本地基础栈启停。
- 根 `Makefile` 统一提供：
  - `make up-local` / `make up-core`：核心基础栈
  - `make up-mocks`：`core + mock-payment-provider`
  - `make up-demo`：全量演示组合
- `check-local-stack.sh`：本地依赖健康检查。
- `check-demo-fixtures.sh`：校验 `fixtures/demo/` 五条标准链路正式数据包的场景顺序、SKU 覆盖、商品/订单/交付/账单/审计引用和上游真值源一致性。
- `seed-demo.sh`：执行 `TEST-002` 正式 demo importer；先跑 `db/seeds/manifest.csv`，再把 `fixtures/demo/` 的 10 笔订单、支付记录和交付对象写入 `trade / payment / billing / delivery`。
- `check-demo-seed.sh`：回查 `seed-demo.sh` 的正式落库结果，验证五条标准链路的 demo 主体、商品、订单、支付和交付对象都已真实落到数据库。
- `check-order-e2e.sh`：`TEST-006` 正式 order E2E checker；会复用 `smoke-local.sh`、`seed-local-iam-test-identities.sh` 与 `seed-demo.sh`，然后以 `local-buyer-operator + local-tenant-developer` 运行门户五条标准链路 live E2E，并回查 order detail / lifecycle / developer trace。
- `check-provider-switch.sh`：`TEST-007` 正式 provider switch checker；会复用 `smoke-local.sh`、`check-mock-payment.sh`、`trade026` 签章 smoke、`fabric-adapter-test.sh` 与 `fabric-adapter-live-smoke.sh`，验证支付 / 签章 / 链写三类 provider 只通过配置完成切换。
- `check-api-contract-baseline.sh`：`TEST-003` 正式 contract checker；校验 OpenAPI 成功/失败 envelope、关键响应字段、错误码基线，以及订单状态机 action enum / 禁止错误码绑定。它不替代 `TEST-028` 的 canonical smoke。
- `check-migration-smoke.sh`：`TEST-004` 正式 migration smoke checker；启动 current local core stack、初始化 MinIO buckets、执行 migration/seed roundtrip，并在最终升级后真实启动 `platform-core-bin` 回查 `/health/live`、`/health/ready`、`/health/deps` 和 `/internal/runtime`。
- `validate_database_migrations.sh`：兼容入口，现已转发到 `check-migration-smoke.sh`，不再使用历史 `部署脚本/docker-compose.postgres-test.yml`。
- `check-keycloak-realm.sh`：校验 Keycloak realm 导入、`portal-web` password grant、正式角色 claim 与 `user_id/org_id` 自定义 claims。
- `reset-keycloak-local.sh`：重建本地独立 Keycloak 数据库并重新导入 `platform-local` realm，修复旧 realm 残留或导入污染。
- `prune-local.sh`：安全清理当前仓库本地卷、网络、Fabric 状态（默认 `--dry-run`）。
- `export-local-config.sh`：导出 compose 解析后的只读快照。
- `smoke-local.sh`：`TEST-005` 正式本地环境 smoke checker；会自动确保 `core + observability + mocks` compose profile、执行基础 `migrate-up + seed-up`、初始化 MinIO buckets，并以 `APP_HOST=0.0.0.0` 拉起或复用宿主机 `platform-core` 供容器侧 Prometheus 抓取，同时继续以 `127.0.0.1:8094` 作为宿主机访问口径；随后回查 `check-local-stack/full`、Keycloak realm、Grafana datasource、canonical topics、宿主机/容器 Kafka 双地址边界以及关键 ops 控制面入口。
- `fabric-adapter-*.sh` / `fabric-event-listener-*.sh` / `fabric-ca-admin-*.sh`：Go 版 Fabric 适配器、callback listener 与 CA 管理执行面的 bootstrap / test / run 入口，统一复用 `scripts/go-env.sh` 和 `third_party/external-deps/go`。
- `fabric-adapter-live-smoke.sh`：真实 `fabric-test-network` smoke，回查账本 + PostgreSQL + 审计 + 系统日志。
- `fabric-env.sh`：统一导出 Fabric 版本、samples、channel、chaincode、MSP 与证书路径约定。

## 禁止事项

- 禁止在脚本中写死环境专属参数。
- 禁止绕过 `Makefile` 私自新增重复入口。
