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
- `/healthz`（兼容入口，映射到 live）

当前请求级中间件链（V1-Core）：

- `request_id`：优先透传 `x-request-id`，缺失时自动生成。
- `trace`：优先透传 `x-trace-id`，缺失时复用 `request_id`。
- `tenant`：解析 `x-tenant-id`，缺失回落到 `public`。
- `access-log`：统一记录 method/path/status/elapsed/request_id/trace_id/tenant_id。

统一错误体系：

- `AppError / ErrorCode / ErrorResponse` 在 `crates/kernel` 收口。
- 启动时校验 `docs/01-architecture/error-codes.md` 的错误码前缀分组与运行时错误码前缀一致。

统一事务模板（V1-Core）：

- 在 `crates/db` 提供 `TransactionBundle` 事务编排模板。
- 业务对象变更（`business_mutations`）、审计事件写入（`audit_events`）、outbox 事件写入（`outbox_events`）在同一事务模板内按单次 begin/commit 或 begin/rollback 执行。

约束：

- 业务 handler 不直接依赖外部 provider SDK，统一经内部 crate/门面隔离。
- 运行模式切换优先通过配置（环境变量）完成，不通过编译时分叉。
