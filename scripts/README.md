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
- `check-keycloak-realm.sh`：校验 Keycloak realm 导入、`portal-web` password grant、正式角色 claim 与 `user_id/org_id` 自定义 claims。
- `reset-keycloak-local.sh`：重建本地独立 Keycloak 数据库并重新导入 `platform-local` realm，修复旧 realm 残留或导入污染。
- `prune-local.sh`：安全清理当前仓库本地卷、网络、Fabric 状态（默认 `--dry-run`）。
- `export-local-config.sh`：导出 compose 解析后的只读快照。
- `smoke-local.sh`：执行本地环境 smoke 套件（DB 迁移探测、bucket/topic/realm/Grafana/mock-payment）；建议在 `make up-demo` 后运行，或至少保证 `core + observability + mocks` 已就绪。
- `fabric-adapter-*.sh` / `fabric-event-listener-*.sh` / `fabric-ca-admin-*.sh`：Go 版 Fabric 适配器、callback listener 与 CA 管理执行面的 bootstrap / test / run 入口，统一复用 `scripts/go-env.sh` 和 `third_party/external-deps/go`。
- `fabric-adapter-live-smoke.sh`：真实 `fabric-test-network` smoke，回查账本 + PostgreSQL + 审计 + 系统日志。
- `fabric-env.sh`：统一导出 Fabric 版本、samples、channel、chaincode、MSP 与证书路径约定。

## 禁止事项

- 禁止在脚本中写死环境专属参数。
- 禁止绕过 `Makefile` 私自新增重复入口。
