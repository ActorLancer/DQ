# console-web

`WEB-002` 起正式承接控制台前端入口，覆盖：

- 审核与风控
- 审计与监管
- 开发态运维与联调
- 开发者控制面

当前工程基线采用 `Next.js App Router + TypeScript + Tailwind v4 + TanStack Query + React Hook Form + Zod`，并通过 `/api/platform/**` 受控代理接入 `platform-core`。

## 本地开发

```bash
pnpm --filter @datab/console-web dev --hostname 127.0.0.1 --port 3102
```

如需把控制台代理到本地 `platform-core`：

```bash
PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm --filter @datab/console-web dev --hostname 127.0.0.1 --port 3102
```

## 边界

- 浏览器只请求 `console-web -> /api/platform -> platform-core`
- 通知联查 / dead-letter replay 页面同样只走 `/api/platform/** -> platform-core`，不直连 `notification-worker`
- 不直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`
- 认证会话统一走 `Keycloak / IAM`，开发联调身份仅用于受控本地验证

## Scaffold 与 Preview 口径

- `ConsoleRouteScaffold` 仅用于布局、元信息和权限/API 提示，不构成任务完成证据。
- 正式控制台页面默认只以真实后端响应承接 `loading / empty / error / forbidden`，不允许依赖 URL `preview` 作为正式功能态。
- `preview` 只在显式设置 `NEXT_PUBLIC_WEB_ROUTE_PREVIEW=1`（或 `true`）时启用，用于 UI 状态预演测试，不计入正式功能闭环验收。
