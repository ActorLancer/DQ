# platform-core workspace 结构（V1-Core）

`apps/platform-core` 在 V1-Core 阶段采用“主应用 + 内部共享 crate + 独立 bin 入口”的目录组织：

- `apps/platform-core/src`：主应用代码与业务模块目录（后续模块任务继续在此收敛）。
- `apps/platform-core/bin/platform-core`：独立二进制入口，复用主应用运行时启动逻辑。
- `apps/platform-core/crates/kernel`：应用启动器、模块注册器、依赖容器、生命周期钩子与 shutdown 流程。
- `apps/platform-core/crates/config`：运行模式与 provider 选择配置装载（`local/staging/demo`）。
- `apps/platform-core/crates/http`：HTTP server、路由、统一响应与健康检查端点封装。

当前 V1-Core 基线健康检查路由：

- `/health/live`
- `/health/ready`
- `/healthz`（兼容入口，映射到 live）

约束：

- 业务 handler 不直接依赖外部 provider SDK，统一经内部 crate/门面隔离。
- 运行模式切换优先通过配置（环境变量）完成，不通过编译时分叉。
