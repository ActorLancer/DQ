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
- `check-outbox-consistency.sh`：`TEST-008` 正式 outbox consistency checker；会复用 `smoke-local.sh`、`trade003_create_order_db_smoke`、`outbox_publisher_db_smoke` 与 `notif012_notification_worker_live_smoke`，验证事务成功写主对象 + 审计 + outbox、事务失败无 outbox，以及重复消费不重复副作用。
- `check-audit-completeness.sh`：`TEST-009` 正式 audit completeness checker；会复用 `smoke-local.sh`、证据包导出 route guard 与 `audit_trace_api_db_smoke`，验证关键审计动作留痕、导出必须 step-up、非法导出被拒绝。
- `check-searchrec-pg-authority.sh`：`TEST-010` 正式搜索 / 推荐 PG 权威 checker；会复用 `smoke-local.sh`、`search_visibility_and_alias_consistency_db_smoke`、`search_catalog_pg_fallback_db_smoke`、`recommendation_get_api_db_smoke` 与 `recommendation_filters_frozen_product_db_smoke`，验证 OpenSearch / Redis 只是候选与缓存，冻结 / 下架商品不能越过 PostgreSQL 最终业务校验。
- `check-payment-webhook-idempotency.sh`：`TEST-011` 正式支付 webhook 幂等 checker；会复用 `smoke-local.sh`、`check-mock-payment.sh` 与 `bil005_payment_webhook_db_smoke`，验证 duplicate success、`success -> fail`、`timeout -> success` 不会破坏 `payment_intent / order_main` 最终状态。
- `check-delivery-revocation.sh`：`TEST-012` 正式交付断权 checker；会复用 `smoke-local.sh` 与 `dlv021_auto_cutoff_resources_db_smoke`，验证文件 ticket、share grant、API credential、sandbox workspace/session 在退款、到期、争议和风控冻结后的正式入口断权与 `Redis / PostgreSQL / audit` 联查。
- `check-dispute-settlement-linkage.sh`：`TEST-013` 正式争议与结算联动 checker；会复用 `smoke-local.sh` 与 `bil019_dispute_refund_compensation_recompute_db_smoke`，验证争议打开后的结算冻结、裁决后的退款/赔付正式入账，以及 `billing_event / settlement_record / audit / outbox` 联查。
- `check-audit-replay-dry-run.sh`：`TEST-014` 正式审计回放 dry-run checker；会先校验 replay 路由的权限 / step-up / dry-run-only 守卫，再复用 `smoke-local.sh` 与 `audit_trace_api_db_smoke`，验证 replay job、diff summary、MinIO report、`audit.access_audit / ops.system_log` 留痕。
- `check-ci-minimal-matrix.sh`：`TEST-015` 正式最小 CI 矩阵 checker；支持 `rust / ts / go / migration / openapi / all` 六个入口，统一收口 Rust `fmt/check/test`、TS lint/typecheck/unit test、Go build/test、migration smoke 与 OpenAPI schema 检查。
- `check-compose-smoke.sh`：`TEST-016` 正式 compose CI smoke checker；先复用 `smoke-local.sh` 拉起 `core + observability + mocks`、完成健康与控制面回查，再串联 `CANONICAL_CHECK_MODE=static ./scripts/check-canonical-contracts.sh`，把 canonical topic、consumer group catalog 与关键 OpenAPI 漂移一起拦在 compose 作业中。
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
