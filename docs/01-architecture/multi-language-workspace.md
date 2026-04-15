# 多语言工作区规范（BOOT-005）

## 1. 语言与工作区策略

- Rust：使用根 `Cargo.toml` workspace 管理（主应用与核心库优先）。
- Go：各服务独立 `go.mod`，目录落位在 `services/`。
- Python：离线/处理任务使用独立 package 结构，目录落位在 `workers/`。
- Frontend：使用 `pnpm workspace` 管理 `portal-web/console-web` 与共享前端包。

## 2. 目录约束

- 业务主状态实现优先在 `apps/platform-core`。
- 外围适配能力落位 `services/` 或 `workers/`，不反向定义主状态。
- 共享契约、配置、SDK、UI 组件统一落位 `packages/`。

## 3. 禁止事项

- 同一条交易主链路在多语言重复实现。
- 绕开共享契约目录直接在业务服务内部复制定义。
