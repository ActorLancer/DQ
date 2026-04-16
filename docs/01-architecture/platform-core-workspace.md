# platform-core workspace 结构（V1-Core）

`apps/platform-core` 在 V1-Core 阶段采用“主应用 + 内部共享 crate + 独立 bin 入口”的目录组织：

- `apps/platform-core/src`：主应用代码与业务模块目录（后续模块任务继续在此收敛）。
- `apps/platform-core/bin/platform-core`：独立二进制入口，复用主应用运行时启动逻辑。
- `apps/platform-core/crates/kernel`：应用启动器、模块注册器、依赖容器、生命周期钩子与 shutdown 流程。
- `apps/platform-core/crates/config`：运行模式与 provider 选择配置装载（`local/staging/demo`）。
- `apps/platform-core/crates/http`：HTTP server、路由、统一响应与健康检查端点封装。
- `apps/platform-core/crates/db`：PostgreSQL 连接池抽象、只读/写事务边界、迁移执行接口。
- `apps/platform-core/crates/auth`：JWT/会话主体解析、权限检查门面与 step-up 占位网关。
- `apps/platform-core/crates/audit-kit`：审计上下文与事件写入接口、证据清单挂接、导出记录入口。
- `apps/platform-core/crates/outbox-kit`：`outbox_event` 写入接口、事件 envelope、幂等键、发布状态与重试策略。
- `apps/platform-core/crates/provider-kit`：KYC、签章、支付、通知、Fabric 写入等 Provider trait 与 `mock/real` 实现入口工厂。

当前 V1-Core 基线健康检查路由：

- `/health/live`
- `/health/ready`
- `/health/deps`（依赖可达性：DB/Redis/Kafka/MinIO/Keycloak/Fabric Adapter）
- `/healthz`（兼容入口，映射到 live）
- `/internal/dev/trace-links`（本地 Grafana/Loki/Tempo/Keycloak/MinIO/OpenSearch 快速跳转）
- `/internal/dev/overview`（运行模式与最近 outbox/dead-letter/链回执快照）

当前请求级中间件链（V1-Core）：

- `request_id`：优先透传 `x-request-id`，缺失时自动生成。
- `trace`：优先透传 `x-trace-id`，缺失时复用 `request_id`。
- `tenant`：解析 `x-tenant-id`，缺失回落到 `public`。
- `idempotency`：优先透传 `idempotency-key`，兼容 `x-idempotency-key`，缺失回落到 `request_id`。
- `access-log`：统一记录 method/path/status/elapsed/request_id/trace_id/tenant_id。

统一错误体系：

- `AppError / ErrorCode / ErrorResponse` 在 `crates/kernel` 收口。
- 启动时校验 `docs/01-architecture/error-codes.md` 的错误码前缀分组与运行时错误码前缀一致。

统一事务模板（V1-Core）：

- 在 `crates/db` 提供 `TransactionBundle` 事务编排模板。
- 业务对象变更（`business_mutations`）、审计事件写入（`audit_events`）、outbox 事件写入（`outbox_events`）在同一事务模板内按单次 begin/commit 或 begin/rollback 执行。

统一分页与筛选组件（V1-Core）：

- 在 `crates/http` 提供 `Pagination`、`FilterQuery`、`ListQuery`，供目录搜索、订单列表、审计列表、ops 列表复用。
- 分页默认值：`page=1`、`page_size=20`，并对 `page_size` 做 `1..=200` 的边界收敛。

统一运行时模式页（V1-Core）：

- `/internal/runtime` 返回 `mode`、`provider`、`service_version`、`git_sha`、`migration_version`，用于环境自检与联调排障。
- `/internal/runtime` 同时返回 `feature_flags`，用于演示功能、公链锚定、真实 Provider、敏感实验开关可观测。

统一审计注解机制（V1-Core）：

- 在 `crates/audit-kit` 提供 `AuditAnnotation`，支持在 handler 层声明 `action`、`risk_level`、`object_type`、`object_id`、`result`。
- 在 `crates/http` 提供 `set_audit_annotation/get_audit_annotation`，将注解挂载到请求上下文，供应用层审计写入统一消费。

统一权限门面（V1-Core）：

- 在 `crates/auth` 提供 `AuthorizationFacade` 统一入口，收敛 `Bearer -> SessionSubject` 解析与权限评估流程。
- 权限门面只返回 `AuthorizationDecision`，不直接执行业务放行；真正放行仍在应用层/访问控制层执行。
- `platform-core` 启动时把统一门面注册进容器，业务 handler 避免直接调用 Keycloak/外部 IAM SDK。

统一时间与 ID 策略（V1-Core）：

- 在 `crates/kernel` 提供 `UtcTimestampMs`（统一 UTC 时间戳存储）与 `EntityId`（UUID 主键封装）。
- 对外可读编号通过 `new_external_readable_id(prefix)` 统一生成，避免把内部 UUID 直接暴露为业务编号。

启动自检（V1-Core）：

- `platform-core` 启动时执行 `startup_self_check`，校验关键配置（topic/bucket/index alias）可用。
- `CoreModule` 启动后校验 Provider trait 绑定完整（KYC、签章、支付、通知、Fabric 写入）。

测试与校验骨架（V1-Core）：

- `crates/db` 提供 `TestDbFixture` 与 `run_transaction_rollback_fixture`，用于基础单元测试、测试数据库连接配置与事务回滚夹具。
- 提供 `query-compile-check` 特性和 `scripts/check-query-compile.sh`，把查询编译检查前置到 CI/本地。
- 提供 `scripts/check-openapi-schema.sh` 与 `Makefile` 目标 `openapi-check`，校验 `packages/openapi/*.yaml` 结构与 ops 路径骨架不漂移。
- 提供 `xtask` 工作流与 `cargo xtask all`（别名已配置），一键执行 `fmt`、`lint`、OpenAPI 校验、migration 检查、seed 导入。

Feature Flags 机制（V1-Core）：

- 在 `crates/config` 提供 `FeatureFlags`，由环境变量 `FF_DEMO_FEATURES`、`FF_CHAIN_ANCHORING`、`FF_REAL_PROVIDER`、`FF_SENSITIVE_EXPERIMENTS` 统一装载。
- 启动自检阶段执行安全约束：`provider=real` 必须显式开启 `FF_REAL_PROVIDER`。

仓储接口与内存假实现（V1-Core）：

- 在 `crates/db` 提供 `OrderRepository` trait 与 `InMemoryOrderRepository`，用于业务规则测试先于基础设施联调。

进程内领域事件总线（V1-Core）：

- 在 `crates/kernel` 提供 `InProcessEventBus` 与 `DomainEventEnvelope`，用于模块内解耦事件分发。
- 该总线只覆盖进程内通信；跨进程副作用仍以 DB outbox + Kafka 为准。

开发者总览快照（V1-Core）：

- 在 `crates/http` 提供 `DevOverview` 快照模型与进程内 ring buffer，统一承载最近 outbox/dead-letter/链回执记录。
- `/internal/dev/overview` 默认返回最近 10 条记录；当前阶段用于开发联调观测，不替代审计与生产监控链路。

约束：

- 业务 handler 不直接依赖外部 provider SDK，统一经内部 crate/门面隔离。
- 运行模式切换优先通过配置（环境变量）完成，不通过编译时分叉。
