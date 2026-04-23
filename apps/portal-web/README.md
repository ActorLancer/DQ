# portal-web

`portal-web` 是 `V1-Core / WEB-001` 建立的门户前端基线，职责是承载买方与卖方门户页面，并且只通过受控边界调用 `platform-core` 正式 API。

## 当前范围

- `Next.js App Router + TypeScript + Tailwind CSS v4`
- 根布局、门户导航、身份条、登录态占位弹窗
- 受控 API 代理：`/api/platform/* -> platform-core`
- `@datab/sdk-ts` 契约绑定
- 冻结页面路由键与权限元数据挂载
- 首页联调探针：
  - `GET /health/ready`
  - `GET /api/v1/catalog/standard-scenarios`

## 本地开发

先确保根目录完成依赖安装：

```bash
pnpm install
```

启动门户：

```bash
pnpm --filter @datab/portal-web dev
```

默认通过 `PLATFORM_CORE_BASE_URL` 指向后端，例如：

```bash
PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm --filter @datab/portal-web dev
```

## 登录态占位

当前任务只实现受控登录态占位，不替代正式 `Keycloak / IAM`。

- `Bearer` 模式：把已有 access token 写入 HttpOnly Cookie，由服务端代理附加 `Authorization`
- `Local Header` 模式：本地联调用 `x-login-id / x-role` 头模拟身份
- `Guest` 模式：显式展示未注入会话状态

## 约束

- 前端不得直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`
- 通知相关控制面能力若后续进入门户，也必须先走 `platform-core` facade，不能直连 `notification-worker`
- 页面写操作后续必须透传 `X-Idempotency-Key`
- 高风险操作后续必须透传 `X-Step-Up-Token` 或等价链路
