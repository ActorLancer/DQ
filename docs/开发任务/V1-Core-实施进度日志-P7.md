# V1-Core 实施进度日志 P7

本文件是实施进度日志的当前续写分卷正文。

- 正式入口页：`docs/开发任务/V1-Core-实施进度日志.md`
- 当前活动分卷以入口页为准；当前入口页指向本卷
- 若后续切换到新的 `P{N}` 分卷，必须先更新入口页，再开始续写新分卷

### BATCH-275（计划中）
- 任务：`WEB-001` 初始化 `apps/portal-web/` Next.js 项目，接入 pnpm workspace、基础布局、登录态占位、API SDK
- 状态：计划中
- 说明：从 `WEB` 阶段首任务起，正式建立门户前端工程基线。当前仓库内 `apps/portal-web/`、`apps/console-web/` 与 `packages/sdk-ts/` 仅有 README / `.gitkeep` 参考占位，不能视为完成实现；本批将按冻结文档初始化 `pnpm` workspace、`Next.js App Router + TypeScript` 门户工程、基础布局、统一路由元数据、登录态占位、`platform-core` 受控 API 代理和 `packages/sdk-ts` 契约绑定，并补齐最小自动化与手工 smoke。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已在前序分卷完成并可作为 `WEB-001` 基线输入；`docs/02-openapi/*.yaml` 与 `packages/openapi/*.yaml` 当前一致。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`：定位 `WEB-001` 交付物、DoD、验收口径、依赖与 `technical_reference`。
  - `docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-001` 是门户工程基线与最小页面闭环，不是首页业务功能本体。
  - `docs/开发任务/V1-Core-实施进度日志.md`、`docs/开发任务/V1-Core-实施进度日志-P3.md`：复核分卷入口、分卷切换规则与既有日志格式；本批启用 `P7` 续写。
  - `docs/开发任务/V1-Core-TODO与预留清单.md`：确认新增 gap / reserved 必须登记。
  - `docs/开发准备/服务清单与服务边界正式版.md`：确认 `portal-web` 只调用 `platform-core` 正式 API，前端不得直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`。
  - `docs/开发准备/接口清单与OpenAPI-Schema冻结表.md`：确认前端统一经 `/api/v1`、公共请求头、写接口 `X-Idempotency-Key` 和高风险 `X-Step-Up-Token` 边界；注意通用响应示例与 OpenAPI / 后端当前实现存在表现差异，本批以 `packages/openapi/*.yaml + docs/02-openapi/*.yaml + platform-core` 一致的 `success/data + ErrorResponse(request_id)` 作为 SDK 契约基线。
  - `docs/开发准备/统一错误码字典正式版.md`：确认前端错误态要映射稳定 `code`，不能发明新错误语义。
  - `docs/开发准备/测试用例矩阵正式版.md`：确认 `WEB` 阶段至少要覆盖登录、搜索、详情、下单、交付、验收、联查等链路，且必须具备自动化与手工 smoke。
  - `docs/开发准备/本地开发环境与中间件部署清单.md`、`docs/开发准备/配置项与密钥管理清单.md`：确认本地联调依赖 `platform-core + PostgreSQL + Kafka + Redis + MinIO + OpenSearch + Keycloak`，浏览器身份需围绕 `Keycloak / IAM`，前端不能暴露敏感密钥与对象真实路径。
  - `docs/开发准备/技术选型正式版.md`、`docs/开发准备/平台总体架构设计草案.md`：确认 `TypeScript + React/Next.js` 承担门户 / 控制台，整体架构为前后端分离 + `platform-core` 模块化单体主应用。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 `V1` 两个行业、五条标准链路与八个标准 SKU 是门户后续页面命名与展示真值源。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认门户域页面范围、全局页面规范、首页与页面间路由关系、敏感页面展示要求。
  - `docs/权限设计/按钮级权限说明.md`、`docs/权限设计/接口权限校验清单.md`、`docs/权限设计/菜单权限映射表.md`、`docs/权限设计/菜单树与路由表正式版.md`：确认门户 V1 路由、查看权限、主按钮权限与角色范围，冻结 `portal` 路由路径与页面键。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认卖方主页查看与首页 -> 搜索 -> 商品详情 -> 下单主链路线性关系。
  - `docs/05-test-cases/README.md`、`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`、`docs/05-test-cases/search-rec-cases.md`、`docs/05-test-cases/delivery-cases.md`、`docs/05-test-cases/payment-billing-cases.md`、`docs/05-test-cases/notification-cases.md`：确认 `WEB` 阶段回归必须复用现有搜索、交付、账单、通知 smoke 基线，不得把页面能打开误报为闭环完成。
  - `docs/04-runbooks/search-reindex.md`、`docs/04-runbooks/recommendation-runtime.md`、`docs/04-runbooks/opensearch-local.md`、`infra/docker/docker-compose.local.yml`：确认本地搜索/推荐/Keycloak/OpenSearch 运行方式与宿主机端口。
  - `packages/openapi/catalog.yaml`、`trade.yaml`、`delivery.yaml`、`billing.yaml`、`audit.yaml`、`ops.yaml`、`search.yaml`、`recommendation.yaml`、`iam.yaml` 与对应 `docs/02-openapi/*.yaml`：确认门户首批将绑定的 `iam / catalog / search / recommendation / trade / billing / ops / audit` 契约边界与 schema。
  - `apps/platform-core/src/modules/catalog/**`、`order/**`、`delivery/**`、`billing/**`、`audit/**`、`search/**`、`recommendation/**`、`iam/**`：核对现有真实路由、返回结构、权限实现与测试基线。
  - `apps/portal-web/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认现状仅为占位参考，当前不能视为已完成。
- 当前完成标准理解：
  - 需要形成可运行的门户工程基座，而不是只生成 README 或脚手架。
  - 至少应具备：门户路由骨架、基础布局、登录态占位、`platform-core` 受控 API 访问边界、可复用状态组件、`packages/sdk-ts` 契约绑定、最小 E2E / smoke。
  - `WEB-001` 不提前实现 `WEB-003+` 的完整业务页面，但必须为首页、搜索、卖方、详情、下单、订单详情、卖方中心等 V1 路由提供正式页面入口和可验证的基础状态。
- 实施计划：
  1. 建立 `pnpm` workspace 与 `portal-web` 依赖管理基线。
  2. 初始化 `apps/portal-web` 的 `Next.js App Router + TypeScript + Tailwind v4` 工程，并接入 `TanStack Query`、`React Hook Form`、`Zod`、基础 UI 基座。
  3. 初始化 `packages/sdk-ts`，从冻结 OpenAPI 生成类型并封装门户首批实际要用的 SDK。
  4. 落门户布局、路由元数据、主体信息条、登录态占位、统一状态页与受控 `platform-core` API 代理。
  5. 执行 `pnpm install / lint / typecheck / test / build`，补齐最小 Playwright 与手工 smoke，之后回写 TODO、待审批与本地提交。

### BATCH-275（待审批）
- 任务：`WEB-001` 初始化 `apps/portal-web/` Next.js 项目，接入 pnpm workspace、基础布局、登录态占位、API SDK
- 状态：待审批
- 当前任务编号：`WEB-001`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已作为已完成基线输入继续生效；`portal-web` 只经 `platform-core` 正式 API / 受控代理访问后端，没有新增前端直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric` 的实现。
- 完成情况：
  - 根目录新增 `pnpm-workspace.yaml`、`package.json`、`pnpm-lock.yaml`、`tsconfig.base.json`，正式建立 `portal-web + sdk-ts` 的 `pnpm workspace` 基线。
  - `packages/sdk-ts/**` 从冻结 OpenAPI 生成类型并封装门户首批实际需要的 `iam / catalog / search / recommendation / trade / ops` SDK；补 `PlatformClient`、错误对象、请求头与 Vitest 单测，修复浏览器端运行时 `fetch` 绑定缺陷，避免门户客户端查询在 SSR/浏览器切换时把相对路径请求误绑到宿主机 `fetch`。
  - `apps/portal-web/**` 初始化 `Next.js App Router + TypeScript + Tailwind v4` 门户工程，接入 `TanStack Query`、`React Hook Form`、`Zod`、`Playwright`、`Vitest`，并落统一布局、左侧导航、官方 `page_key` 路由注册、首页骨架、搜索/详情/下单/订单/交付/账单/开发者等正式入口页骨架。
  - `apps/portal-web/src/app/api/platform/[...path]/route.ts` 新增受控代理边界，门户浏览器只访问 `/api/platform/**`，由 Next Route Handler 统一转发到 `platform-core`，同时继承会话 Cookie、补 `x-request-id`，不向浏览器暴露受限系统或对象存储真实地址。
  - `apps/portal-web/src/actions/session.ts`、`src/lib/session.ts`、`src/components/portal/auth-placeholder-dialog.tsx`、`src/components/portal/identity-strip.tsx` 实现登录态占位：支持 Bearer Token 与本地 `x-login-id / x-role` 两种联调模式，通过 `/api/v1/auth/me` 真正校验后写入 HttpOnly Cookie，并在敏感主体条上展示当前主体 / 角色 / 租户 / 作用域。
  - `apps/portal-web/src/components/portal/home-shell.tsx` 把首页基线接到真实 `platform-core` 边界：读取 `/health/ready` 与 `/api/v1/catalog/standard-scenarios`，显式呈现 readiness、标准场景模板数量、标准场景卡片、空态/错态/权限态说明，不再是纯静态 README 页面。
  - `apps/portal-web/e2e/smoke.spec.ts`、`packages/sdk-ts/src/core/http.test.ts`、`apps/portal-web/src/lib/portal-routes.test.ts` 提供最小自动化覆盖，确保门户工程基线、路由骨架和 SDK HTTP 行为可回归。
  - 同步修正 `packages/openapi/*.yaml` 与 `docs/02-openapi/*.yaml` 的实际漂移：
    - `iam.yaml`：补齐 `GET /api/v1/auth/me -> ApiResponse<SessionContextView>`，覆盖 `jwt_mirror / local_test_user` 两种模式。
    - `catalog.yaml`：把 `GET /api/v1/catalog/standard-scenarios` 修正为真实 `ApiResponse<Vec<StandardScenarioTemplateView>>`。
    - `audit.yaml`、`billing.yaml`、`search.yaml`：修正阻塞 SDK 生成的 YAML 语法/重复键问题。
    - `scripts/check-openapi-schema.sh`、`packages/openapi/README.md`、`docs/02-openapi/README.md`：新增 `auth/me` 与 `standard-scenarios` 的最小防漂移校验说明。
- 验证：
  - 前端 / 契约：
    - `pnpm install`
    - `./scripts/check-openapi-schema.sh`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调与 smoke：
    - `./scripts/verify-local-stack.sh core`
    - 宿主机方式启动 `platform-core`：`set -a && source infra/docker/.env.local && set +a && export KAFKA_BROKERS=127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094} APP_MODE=local PROVIDER_MODE=mock APP_HOST=127.0.0.1 && cargo run -p platform-core-bin`
    - 直接 `curl`：
      - `GET http://127.0.0.1:8080/healthz`
      - `GET http://127.0.0.1:8080/api/v1/auth/me`（临时本地测试用户 + `x-login-id / x-role`）
      - `GET http://127.0.0.1:8080/api/v1/catalog/standard-scenarios`
    - 门户代理 `curl`：
      - `GET http://127.0.0.1:3101/api/platform/healthz`
      - `GET http://127.0.0.1:3101/api/platform/api/v1/auth/me`
      - `GET http://127.0.0.1:3101/api/platform/api/v1/catalog/standard-scenarios`
    - 浏览器 smoke：
      - 生产构建方式启动 `portal-web`：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm exec next start --hostname 127.0.0.1 --port 3101`
      - 使用 Playwright + HttpOnly 会话 Cookie 在桌面与移动视口打开首页，确认身份条、readiness、标准场景卡片和移动端可加载。
    - 数据库回查与清理：
      - 插入临时 `core.organization / core.user_account` 本地测试主体用于 `auth/me` 与门户身份条验证
      - 验证结束后删除临时主体，并用 `psql` 回查 `count(*) = 0`
- 验证结果：
  - `sdk-ts` 生成、类型检查、Vitest、门户 Vitest/Playwright、门户构建全部通过；新增的 `PlatformClient` 运行时 `fetch` 绑定单测通过，修复后门户首页客户端查询可真实刷新 `platform-core` readiness、标准场景与身份条。
  - `cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 全部通过；后端无需为 `WEB-001` 增补业务实现。
  - `./scripts/verify-local-stack.sh core` 通过，确认宿主机联调应使用 `127.0.0.1:9094` 的 Kafka 外部监听，而非容器内 `kafka:9092`；据此修正了宿主机启动 `platform-core` 的运行口径。
  - 真实 `curl` 验证通过：
    - `GET /healthz` 返回 `{"success":true,"data":"ok"}`
    - `GET /api/v1/auth/me` 返回 `mode=local_test_user`、真实 `user_id/org_id/login_id/display_name/roles/auth_context_level`
    - `GET /api/v1/catalog/standard-scenarios` 返回 `5` 条标准场景，并覆盖 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU
    - 经门户 `/api/platform/**` 代理的同一路径返回与直连 `platform-core` 一致
  - 浏览器 smoke 通过：门户首页在桌面端正确显示 `主体 WEB001 Portal User ... / 角色 tenant_operator / 作用域 aal1`、`platform-core readiness = ok`、标准场景卡片；移动视口下首页和主体条可正常加载。
  - 临时测试数据已清理，`core.user_account` 与 `core.organization` 对应记录回查均为 `0`；本批未引入新的 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-001`
  - `页面说明书-V1-完整版.md`：门户首页、搜索、详情、下单、订单、交付、账单、开发者等路由入口与敏感身份条展示要求
  - `菜单权限映射表.md`、`按钮级权限说明.md`、`接口权限校验清单.md`：门户路由查看权限、登录态占位和受控接口边界
  - `服务清单与服务边界正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`：前端只调用 `platform-core` 正式 API、写接口幂等与 step-up 头边界
  - `packages/openapi/*.yaml`、`docs/02-openapi/*.yaml`：`iam / catalog / ops` 首批契约与归档同步
- 覆盖的任务清单条目：`WEB-001`
- 未覆盖项：
  - 无。`WEB-001` 要求的门户工程初始化、workspace、基础布局、登录态占位、SDK、受控 API 代理、最小自动化与真实联调闭环均已完成；更完整业务页面能力留在后续 `WEB-002+` 逐任务继续推进。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
