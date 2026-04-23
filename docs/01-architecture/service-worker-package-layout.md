# Service / Worker / Package 布局说明（BOOT-029~032）

## 1. 目标

为 `apps/`、`services/`、`workers/`、`packages/` 提供统一目录口径，避免后续任务各自新建平行目录。

## 2. 当前布局

### 2.1 apps（现有运行骨架）

- `apps/platform-core`
- `apps/portal-web`
- `apps/console-web`
- `apps/fabric-adapter`
- `apps/fabric-event-listener`
- `apps/search-indexer`
- `apps/data-processing-worker`
- `apps/notification-worker`
- `apps/mock-payment-provider`

### 2.2 services（外围服务落位）

- `services/fabric-adapter`
- `services/fabric-event-listener`
- `services/fabric-ca-admin`
- `services/mock-payment-provider`

说明：

- `notification-worker` 当前正式落位在 `apps/notification-worker`。
- 新增通知控制面或前端联查页面时，浏览器入口仍固定在 `platform-core`；`notification-worker` 只保留内部执行接口，不直接暴露给 `portal-web / console-web`。

### 2.3 workers（异步与离线落位）

- `workers/search-indexer`
- `workers/outbox-publisher`
- `workers/data-processing-worker`
- `workers/quality-profiler`
- `workers/report-job`

### 2.4 packages（共享资产落位）

- `packages/openapi`
- `packages/sdk-ts`
- `packages/ui`
- `packages/shared-config`
- `packages/observability-contracts`
- `packages/api-contracts`
- `packages/event-contracts`
- `packages/domain-types`
- `packages/test-fixtures`

## 3. 执行约束

1. 本批仅做目录边界冻结，不迁移既有实现代码。
2. 后续若发生代码迁移，必须在对应 BOOT/CORE/ENV 任务中显式记录与验证。
3. 新增模块必须优先落在上述目录边界内，不再创建平行结构。
