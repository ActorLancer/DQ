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
  - 浏览器 smoke 通过：门户首页在桌面端正确显示 `主体 WEB001 Portal User ... / 角色 旧本地租户占位角色 / 作用域 aal1`、`platform-core readiness = ok`、标准场景卡片；移动视口下首页和主体条可正常加载。该旧占位角色已在后续批次按正式 V1 角色集合收敛。
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

### BATCH-290（计划中）
- 任务：`WEB-016` 实现开发者页面：应用管理、API Key、调用日志、trace 联查、Mock 支付操作入口
- 状态：计划中
- 当前任务编号：`WEB-016`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-015` 已完成并作为本批基线。本批继续保持 `portal-web / console-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-016` 只实现开发者页面，不合并通知联查或后续全局收口任务；DoD 要求页面可访问、空态/错态/权限态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认 11.1 开发者首页、11.2 测试应用页、11.3 状态联查页必须覆盖网络信息、测试应用、API Key、调用配置、trace 状态联查与调试导航。
  - `docs/权限设计/按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：确认 `developer.home.read / developer.app.read / developer.app.create / developer.app.update / developer.trace.read / developer.mock_payment.simulate` 的页面、按钮、接口权限和 Mock 支付非生产审计口径。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认开发者泳道包括创建测试应用、API Key 管理、本地 / staging / demo 模式、Mock Provider、按 `order_id / event_id / tx_hash` 联查状态与 Mock 支付成功 / 失败 / 超时入口。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`统一错误码字典正式版.md`、`测试用例矩阵正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 WEB 前端只能调用 `platform-core` 正式 API，写操作附带 `Idempotency-Key`，敏感页显示主体 / 角色 / 租户 / 作用域，错误码与权限口径不得漂移。
  - `packages/openapi/ops.yaml`、`billing.yaml`、`iam.yaml` 与 `docs/02-openapi/*.yaml`：确认开发者 trace 已有 `GET /api/v1/developer/trace` 正式契约，Mock 支付已有 simulate-success / fail / timeout 契约；同时发现 `GET/POST /api/v1/apps` 与 API Key 轮换 / 撤销接口在 `iam.yaml` 中仍偏骨架化，需要按后端现有实现和冻结页面 / 权限语义补齐 schema 后再生成 SDK。
  - `apps/platform-core/src/modules/iam/api.rs`、`domain.rs`、`repository.rs` 与 `apps/platform-core/src/modules/audit/api/router.rs`：确认应用管理返回 `ApplicationView`，开发者 trace 返回 `DeveloperTraceLookupResponse`，均通过 `platform-core` 访问真实 PostgreSQL / 审计 / outbox 投影，不允许前端绕过后端直连底层系统。
  - `apps/portal-web/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认现有开发者路由仍为说明页 / skeleton，不能视为已完成；本批将替换为正式页面与 SDK 绑定。
- 当前完成标准理解：
  - portal / console 的开发者首页、测试应用页、状态联查页和 Mock 支付入口必须可访问，并具备加载态、空态、错态、权限态和审计留痕提示。
  - 应用管理与 API Key 页面必须通过 `packages/sdk-ts` 调用 `platform-core` 的 `/api/v1/apps` 与 credentials 接口；所有创建、更新、轮换、撤销动作都附带 `Idempotency-Key`，不暴露真实密钥明文或对象路径。
  - trace 联查必须使用 `/api/v1/developer/trace` 显示 `request_id / trace_id / tx_hash / 链状态 / 投影状态 / 审计与日志摘要`；调用日志不得伪造为前端 mock。
  - Mock 支付操作入口必须绑定 `/api/v1/mock/payments/{id}/simulate-success|fail|timeout`，展示非生产限制、权限态、幂等提交和审计提示。
  - 页面必须继续显示当前主体、角色、租户、作用域，并验证浏览器端仅访问受控 `/api/platform/**` 边界。
- 实施计划：
  1. 补齐 IAM 应用管理 OpenAPI schema，同步 `packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml`，增加最小防漂移校验并重新生成 SDK。
  2. 扩展 `packages/sdk-ts` 的 IAM 应用 / API Key 与 Billing Mock 支付方法，补齐请求头、幂等键和契约测试。
  3. 实现 `portal-web` 与 `console-web` 开发者页面：开发者首页、应用 / API Key 管理、trace / 调用日志联查、Mock 支付入口，并补齐权限、表单校验、空态 / 错态 / 加载态。
  4. 执行前端专项验证、后端通用验证、真实 API 联调、数据库回查、浏览器 smoke 与 E2E，并更新 TODO / 待审批日志后本地提交。

### BATCH-290（待审批）
- 任务：`WEB-016` 实现开发者页面：应用管理、API Key、调用日志、trace 联查、Mock 支付操作入口
- 状态：待审批
- 当前任务编号：`WEB-016`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001 ~ WEB-015` 基线继续生效；本批继续保持 `portal-web / console-web -> /api/platform -> platform-core` 边界，没有新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 完成情况：
  - `packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml` 同步补齐 `/api/v1/apps`、`/api/v1/apps/{id}`、`credentials/rotate`、`credentials/revoke` 的正式响应体、请求体、幂等头和应用视图 schema；`scripts/check-openapi-schema.sh` 已纳入最小防漂移校验，防止 IAM 应用契约再次退化为 skeleton。
  - `packages/sdk-ts` 已重新生成 IAM 类型并新增应用管理 / API Key domain 方法；Billing domain 新增 Mock 支付 success / fail / timeout 方法；单测覆盖请求头、路径、幂等键和响应解包，确保 SDK 与 OpenAPI 契约一致。
  - `console-web` 新增正式开发者工作台、测试应用与 API Key、状态与调用日志联查、Mock 支付与测试资产页面；应用页使用 TanStack Table + Virtual，表单使用 React Hook Form + Zod，创建 / 更新 / 轮换 / 撤销均透传 `Idempotency-Key` 并展示审计提示、权限态、加载态、空态和错误态。
  - `portal-web` 新增对应门户开发者页面，提供移动友好的应用卡片、trace 联查、Mock 支付表单和调试导航；页面统一显示当前主体、角色、租户、作用域，API Key 明文不展示、不持久化，下载或对象路径不在前端暴露。
  - Developer trace 页面真实绑定 `GET /api/v1/developer/trace`，按 `order_id / event_id / tx_hash` 展示 `request_id / trace_id / tx_hash / 链状态 / 投影状态 / 审计 / outbox / dead letter / system log` 摘要，不以前端 mock 伪造调用日志。
  - Mock 支付入口真实绑定 `/api/v1/mock/payments/{id}/simulate-success|fail|timeout`，展示非生产限制、step-up / 幂等提示、provider 结果、webhook 处理状态和支付投影。
  - `portal-routes` 与 `console-routes` 的开发者页面 API 绑定、权限点和 Playwright smoke 已同步更新。
- 验证：
  - 前端 / 契约：
    - `pnpm openapi:generate`
    - `./scripts/check-openapi-schema.sh`
    - `pnpm install --frozen-lockfile`
    - `pnpm install`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/console-web typecheck`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/console-web test:unit`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/console-web lint`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm --filter @datab/console-web test:e2e`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm test`
    - `pnpm build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./infra/kafka/init-topics.sh`
  - 真实 API / DB / 浏览器联调：
    - `./scripts/seed-local-iam-test-identities.sh`
    - `GET /api/v1/auth/me`：本地 `developer.admin@luna.local` 返回 `mode=local_test_user`、`tenant_admin`、正式 `user_id / tenant_id / auth_context_level`
    - `GET /api/v1/apps`、`POST /api/v1/apps`、`PATCH /api/v1/apps/{id}`、`POST /credentials/rotate`、`POST /credentials/revoke`
    - `GET /api/v1/developer/trace?order_id=0b0c5dce-3fca-420e-b416-2433a1552e3e`
    - `POST /api/v1/mock/payments/{payment_intent_id}/simulate-success`
    - DB 回查 `core.application`、`developer.mock_payment_case`、`payment.payment_transaction`、`audit.audit_event`、`audit.access_audit`、`ops.system_log`
    - 桌面 console `1440x920` 与移动 portal `390x844` 浏览器 smoke：注入本地会话 Cookie 后访问开发者首页 / 应用页 / trace 页 / Mock 支付页，并捕获浏览器端 API 请求
    - 前端受限系统扫描：`rg` 检查 `apps/portal-web`、`apps/console-web`、`packages/sdk-ts` 未引入直连 PostgreSQL / Kafka / OpenSearch / Redis / Fabric 客户端或端口访问
- 验证结果：
  - 全量前端 lint / typecheck / unit / E2E / build 均通过；`pnpm test` 中 portal 与 console E2E 均通过。未启动后端时 E2E 会输出预期的 `ECONNREFUSED 127.0.0.1:8094` 代理错误态日志，但最终断言通过；真实后端联调阶段已单独启动 `platform-core` 验证成功路径。
  - 后端通用验证全部通过；`cargo check/test/sqlx prepare` 仅输出仓库既有 warning，未引入新的 Rust / SQLx 回归。
  - OpenAPI 防漂移通过，且 `packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml` 在本批新增应用管理契约上保持同步。
  - 真实 API 联调通过：
    - `auth/me` 返回 `Developer Admin / tenant_admin / 10000000-0000-0000-0000-000000000102 / aal1`
    - 应用创建、更新、API Key 轮换和撤销均成功，响应中的 `client_secret_status` 从 `active` 变为 `revoked`
    - developer trace 返回订单业务状态、支付状态、交付状态、链证明状态、投影状态和 system log 摘要，并写入 `audit.access_audit` 与 `ops.system_log`
    - Mock 支付 success 返回 `provider_kind=mock`、`provider_status=succeeded`、`webhook_processed_status=processed`、重复 webhook 为 `duplicate`
  - DB 回查通过：
    - `core.application` 命中测试应用并展示 `client_secret_status=revoked`
    - `audit.audit_event` 命中 `iam.app.create / iam.app.patch / iam.app.secret.rotate / iam.app.secret.revoke / mock.payment.simulate / payment.webhook.processed / payment.webhook.duplicate`
    - `developer.mock_payment_case` 命中 `scenario_type=success`、`status=executed`、`webhook_processed_status=processed`
    - `payment.payment_transaction` 命中 1 条支付交易
    - developer trace 命中 1 条 `audit.access_audit(target_type=developer_trace_query)` 与 1 条 `ops.system_log`
  - 浏览器 smoke 通过：console 桌面和 portal 移动均能渲染开发者页面与主体条；捕获到的正式请求全部是 `/api/platform/**`，`directRequests=[]`，没有浏览器直连 `127.0.0.1:8094` 或底层中间件。
  - 业务测试数据已清理：本批创建的 `core.application`、`payment.payment_intent`、`developer.mock_payment_case`、`payment.payment_transaction`、`payment.payment_webhook_event` 均已删除或级联清理，残留回查为 `0`；审计与访问日志按 append-only 保留。
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 已登记 `BATCH-290` 无新增项。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-016`
  - `页面说明书-V1-完整版.md`：开发者首页、测试应用页、状态联查页、Mock 支付 / 测试资产入口
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：开发者页面查看、应用管理、trace 读取、Mock 支付模拟权限
  - `业务流程图-V1-完整版.md`：开发者创建测试应用、API Key 管理、trace 联查、Mock Provider 调试路径
  - `packages/openapi/iam.yaml`、`billing.yaml`、`ops.yaml` 与 `docs/02-openapi/*.yaml`
- 覆盖的任务清单条目：`WEB-016`
- 未覆盖项：
  - 无。`WEB-016` 要求的开发者页面、应用管理、API Key、调用日志 / trace 联查、Mock 支付入口、SDK 契约、真实 API 联调、DB 回查、浏览器 smoke 与受控边界验证均已完成；通知联查由 `WEB-022` 继续承接。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-291（计划中）
- 任务：`WEB-017` 为五条标准链路各做一个从首页直达的演示路径和说明卡片
- 状态：计划中
- 当前任务编号：`WEB-017`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-016` 已完成并作为本批基线。本批继续保持 `portal-web / console-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-017` 只实现五条标准链路从首页直达的演示路径与说明卡片，不合并 `WEB-018` 的全链路 E2E 收口；DoD 要求页面可访问、空态 / 错态 / 权限态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认 4.1 首页必须提供平台导航、核心价值、行业入口、推荐商品入口和登录入口；12. 页面间路由关系要求 `首页 -> 搜索页 -> 卖方主页 -> 产品详情页 -> 询单/下单页 -> 合同确认页 -> 支付锁定页 -> 订单详情页`；14. 覆盖校验继续要求页面权限、空态、错态和敏感主体条。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认首批五条标准链路官方命名与顺序为 `S1 工业设备运行指标 API 订阅`、`S2 工业质量与产线日报文件包交付`、`S3 供应链协同查询沙箱`、`S4 零售门店经营分析 API / 报告订阅`、`S5 商圈/门店选址查询服务`；八个标准 SKU 必须完整显式覆盖，`SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 不得并回大类。
  - `docs/权限设计/按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：确认首页查看权限为 `portal.home.read`，SKU 真值字段只允许 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`，后续下单动作仍要求 `trade.order.create` 与 `X-Idempotency-Key`。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认首页主链路必须承接 `首页 -> 搜索 -> 商品详情 -> 下单`，并在上架 / SKU / 模板绑定口径中保持五链路、八 SKU、模板族绑定一致。
  - `docs/05-test-cases/search-rec-cases.md`、`docs/04-runbooks/recommendation-runtime.md`：确认 `home_featured` 推荐位在演示种子完成后固定返回五条官方样例，顺序 `S1 -> S5`，`items[].explanation_codes` 包含 `placement:fixed_sample` 与 `scenario:S1..S5`。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`统一错误码字典正式版.md`、`测试用例矩阵正式版.md`、`本地开发环境与中间件部署清单.md`、`配置项与密钥管理清单.md`、`技术选型正式版.md`、`平台总体架构设计草案.md`、`docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 WEB 前端只能调用 `platform-core` 正式 API，不能直连受限系统；敏感页面显示主体 / 角色 / 租户 / 作用域；错误码、权限点、状态名与 SKU 名不得漂移。
  - `packages/openapi/catalog.yaml`、`trade.yaml`、`recommendation.yaml`、`search.yaml`、`iam.yaml` 与 `docs/02-openapi/*.yaml`：确认本批绑定的正式契约为 `GET /api/v1/catalog/standard-scenarios`、`GET /api/v1/orders/standard-templates`、`GET /api/v1/recommendations?placement_code=home_featured`、`GET /api/v1/catalog/search` 与 `GET /api/v1/auth/me`；两份 OpenAPI 当前保持同步。
  - `apps/platform-core/src/modules/catalog/**`、`order/**`、`search/**`、`recommendation/**`：确认后端已有标准场景模板、标准订单模板、搜索、推荐位与首页五样例测试实现；前端只通过 SDK / 受控代理读取，不自行直连 OpenSearch / Redis / Kafka。
  - `apps/portal-web/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认首页当前已有标准链路卡片、搜索入口和下单入口，但每条链路尚无独立 `/demos/S1..S5` 演示路径；现有代码只能作为基线继续收敛。
- 当前完成标准理解：
  - 首页必须给 `S1 -> S5` 各自提供一张可理解的说明卡片，并有唯一可点击的直达演示路径。
  - 每条演示路径必须展示官方链路名、主 SKU、补充 SKU、合同 / 验收 / 退款模板、路线节点、权限与审计提示，并能回到搜索、下单、交付 / 验收 / 账单等后续页面。
  - 演示页可在未登录时展示冻结说明，但 Bearer 会话下应通过 `packages/sdk-ts` 读取 `platform-core` 的标准场景与订单模板；加载态、空态、错态、权限态需要可验证。
  - 不新增写操作，不新增状态名、SKU 大类、场景名或错误码语义；后续真实全链路 E2E 由 `WEB-018` 继续承接。
- 实施计划：
  1. 抽取五条标准链路的前端演示路径元数据，保持 `S1 -> S5`、八 SKU、合同 / 验收 / 退款模板与冻结文档一致。
  2. 在 `portal-web` 新增 `/demos/S1` 到 `/demos/S5` 动态演示页，绑定 `catalog.standard-scenarios` 与 `trade.standard-templates` SDK，补齐说明卡片、权限 / 审计 / 幂等提示和状态预演。
  3. 改造首页标准链路卡片为直达演示路径，保留搜索与下单入口。
  4. 补齐 `Vitest` 与 `Playwright` 覆盖，验证首页到五条演示路径、SKU 覆盖和受控 API 边界。
  5. 执行前端、后端、OpenAPI、真实 API / 浏览器 smoke、数据库回查与受控边界验证，再更新 TODO、写“待审批”日志并本地提交。

### BATCH-291（待审批）
- 任务：`WEB-017` 为五条标准链路各做一个从首页直达的演示路径和说明卡片
- 状态：待审批
- 当前任务编号：`WEB-017`
- 实现结果：
  - 新增 `apps/portal-web/src/lib/standard-demo.ts` 作为五条官方演示链路的前端单一元数据源，冻结 `S1 -> S5` 顺序、官方命名、八个标准 SKU、合同 / 验收 / 退款模板、交付链路、审计 / 权限 / 幂等说明，避免首页与演示页重复维护口径。
  - 新增 `/demos/[scenarioCode]` 动态路由与 `StandardDemoShell`，为 `/demos/S1` 至 `/demos/S5` 各自展示说明卡片、路线步骤、模板与 SKU 映射、主体 / 角色 / 租户 / 作用域、权限态、加载态、空态、错态和受控 API 边界提示；Bearer 会话下通过 `packages/sdk-ts` 读取 `GET /api/v1/catalog/standard-scenarios` 与 `GET /api/v1/orders/standard-templates`，未登录 / local 模式只展示冻结只读说明，不伪造后端成功态。
  - 改造门户首页标准链路卡片，新增 `查看 S1/S2/S3/S4/S5 演示路径` 直达入口，并保留搜索同类商品、发起下单、推荐位和行业聚合入口。
  - 扩展 `portal-routes` 元数据，把五条演示路径纳入正式路由清单与 API 绑定清单，继续声明 `portal.home.read` 查看权限，不新增写操作或新状态名 / SKU 大类 / 错误码语义。
  - 补齐 `Vitest` 覆盖，校验 `/demos/S1..S5` 顺序、八 SKU 覆盖、`SHARE_RO / QRY_LITE / RPT_STD` 不被并回大类、标准订单模板映射与首页冻结场景同源；补齐 `Playwright` 覆盖，校验从首页逐条点击进入五条演示路径。
- 验证结果：
  - 前端专项：`pnpm install`、`pnpm --filter @datab/portal-web typecheck`、`pnpm --filter @datab/portal-web lint`、`pnpm --filter @datab/portal-web test:unit`、`pnpm --filter @datab/portal-web test:e2e`、`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 均通过；root `pnpm test` 覆盖 SDK、portal、console 单测与 Playwright，E2E 中出现的 `platform-core` 未启动代理日志不影响测试结果，真实联调已另行覆盖。
  - 后端 / 契约：`./scripts/check-openapi-schema.sh`、`cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 均通过；`cargo check/test` 仅保留既有 warning。
  - 本地栈：`./scripts/verify-local-stack.sh core` 确认 PostgreSQL、Redis、Kafka、MinIO、OpenSearch、Keycloak、OTel 均可达。
  - 真实 API：本地 `platform-core` 运行于 `127.0.0.1:8094`，`GET /healthz`、`GET /api/v1/auth/me`、`GET /api/v1/catalog/standard-scenarios`、`GET /api/v1/orders/standard-templates` 均通过；门户生产构建运行于 `127.0.0.1:3101`，通过 `/api/platform/**` 代理访问 `auth/me`、标准场景与标准订单模板均返回正式结构。
  - 浏览器 smoke：使用 Bearer cookie 打开首页和 `/demos/S1..S5`，每页均显示主体 / 角色 / 租户 / 作用域与 `platform-core live` 状态；移动视口打开 `/demos/S5` 正常；请求抓取显示浏览器端只有 `/api/platform/**`，没有直连 `platform-core` 端口或 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
  - DB 回查：`audit.audit_event` 中 `req-web017-auth-me-ok`、`req-web017-standard-scenarios`、`req-web017-order-templates`、`req-web017-portal-auth-me`、`req-web017-portal-scenarios`、`req-web017-portal-templates` 均存在对应 `iam.session.context.read`、`catalog.standard.scenarios.read`、`trade.order.templates.read` 成功审计；`recommend.recommendation_request` 曾记录 `home_featured` 返回 5 条并带 `scenario:S1..S5` explanation codes。
  - 测试数据处理：已删除本批产生的临时 `recommend.recommendation_request` 与 `search.seller_search_document` 投影行；`web017-local` 用户与 `WEB017 Demo Org` 组织因 `audit.access_audit` append-only 外键会触发审计记录更新而无法物理删除，已标记为 `inactive` 并在 `attrs/metadata.web017_cleanup` 中注明 `retained_for_append_only_access_audit_fk`；审计记录按 append-only 规则保留。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 已登记 `BATCH-291` 无新增项。

### BATCH-285（计划中）
- 任务：`WEB-011` 实现验收页面：通过、拒收、拒收原因、生命周期摘要
- 状态：计划中
- 说明：本批将 `/delivery/orders/:orderId/acceptance` 从脚手架升级为正式验收页，围绕人工验收链路展示交付结果摘要、验真结果、合同与模板匹配检查、生命周期摘要、验收通过、拒收和争议入口。页面必须真实读取当前主体、订单详情、生命周期快照，并通过 `packages/sdk-ts` 调用 `POST /api/v1/orders/{id}/accept` / `reject`；所有写动作必须携带 `X-Idempotency-Key`、显示审计留痕提示、回显后端统一错误码与 `request_id`。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-010` 已完成并提交，门户工程、受控 `/api/platform/**` 代理、会话主体条、订单详情、交付中心、`sdk-ts` 与 Delivery 后端基线可复用。本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-011` 只实现验收页面，不合并账单页、争议页、审计联查或通知联查；DoD 要求页面可访问、空态/错态/权限态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认验收页目标为确认收货、拒收和发起争议动作；核心模块为交付结果摘要、验真结果、合同与模板匹配检查、确认验收按钮、拒收按钮、发起争议按钮；关键状态为 `delivered / accepted / rejected / dispute_opened`。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认路由 `/delivery/orders/:orderId/acceptance`，查看权限 `trade.order.read`，主按钮权限 `delivery.accept.execute / delivery.reject.execute`，确认验收和拒收仅在订单 `delivered` 时可执行，争议入口权限为 `dispute.case.create`。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认文件类交付后买方需下载、解密、重算 Hash，Hash 一致后 accept，不一致则 reject 或 open_case；版本订阅、共享、API、沙箱、模板查询的自动验收分支不得在本批前端伪造成手工验收。
  - `docs/05-test-cases/delivery-cases.md`、`payment-billing-cases.md`、`notification-cases.md`：确认重复验收返回 `already_accepted`，拒收需阻断结算并打开争议，验收通过 / 拒收会触发通知 outbox；审计按 append-only 保留。
  - 通用边界文档、OpenAPI 冻结表、统一错误码字典、测试矩阵、本地环境与配置项文档、技术选型、总体架构与全集成基线：确认前端只调用 `platform-core` 正式 API；写接口必须携带 `X-Idempotency-Key`；敏感页面必须展示主体、角色、租户/组织、作用域；不得发明 SKU、状态或错误码语义。
  - `packages/openapi/delivery.yaml` 与 `docs/02-openapi/delivery.yaml`：确认 `accept/reject` schema 已存在，但当前两个写接口尚未声明 `X-Idempotency-Key`；本批需要补齐两份 OpenAPI、重新生成 SDK，并增加防漂移校验。
  - `apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/order/**`、`packages/sdk-ts/**`、`apps/portal-web/**`：确认后端已有验收/拒收状态机、审计、通知、billing bridge 与拒收冻结逻辑；当前门户验收路由仍为 `PortalRoutePage` 脚手架，SDK delivery domain 尚未封装 accept/reject。
- 当前完成标准理解：
  - 验收页必须读取 `auth/me`、订单详情和生命周期快照，显示主体/角色/租户或组织/作用域、订单/SKU、交付摘要、生命周期状态、request_id / tx_hash / 链状态 / 投影状态承接，未返回字段必须显式标注“未返回”。
  - 验收通过与拒收表单必须使用 React Hook Form + Zod 校验，提交时通过 SDK 透传 `X-Idempotency-Key`，重复点击期间禁用主按钮，后端错误码与 `request_id` 要回显。
  - 仅 `FILE_STD / FILE_SUB / RPT_STD` 等人工验收分支在 `delivered` 状态展示可执行主按钮；其他 SKU 或非 `delivered` 状态展示状态说明，不伪造手工验收能力。
  - 拒收必须要求拒收原因，结果区展示 `settlement_status=blocked`、`dispute_status=open` 等正式返回字段；争议页完整创建由 `WEB-013` 承接，本批只提供正式入口。
- 实施计划：
  1. 补齐 delivery OpenAPI 中 accept/reject 的 `X-Idempotency-Key` 声明、防漂移校验和后端审计 metadata 透传，并重新生成 `packages/sdk-ts`。
  2. 扩展 delivery SDK domain 与单测，新增 `acceptOrder` / `rejectOrder` 写方法并强制传入幂等键。
  3. 新增验收页视图模型、Zod/RHF schema、权限判断、响应解包、错误格式化和单元测试。
  4. 替换 `/delivery/orders/:orderId/acceptance` 为正式 `AcceptanceWorkflowShell`，接入真实 API、权限态、空态、错态、加载态、幂等键、审计提示和生命周期摘要。
  5. 更新路由 API 绑定、Playwright 覆盖，执行前端、SDK、后端、OpenAPI、真实 API、浏览器和数据库回查验证。

### BATCH-285（待审批）
- 任务：`WEB-011` 实现验收页面：通过、拒收、拒收原因、生命周期摘要
- 状态：待审批
- 当前任务编号：`WEB-011`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001 ~ WEB-010` 基线继续生效；本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，没有新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 完成情况：
  - `packages/openapi/delivery.yaml` 与 `docs/02-openapi/delivery.yaml` 同步补齐 `POST /api/v1/orders/{id}/accept`、`/reject` 的必填 `X-Idempotency-Key` header、权限说明和最小防漂移校验；`packages/sdk-ts` 已重新生成并新增 `delivery.acceptOrder / rejectOrder` domain 封装，写操作必须显式传入幂等键。
  - 后端验收 / 拒收处理器读取并校验 `x-idempotency-key`，将幂等键写入验收 snapshot、审计 metadata 与 billing bridge payload；缺失或过短时返回统一错误响应，不再允许无幂等键写入。
  - IAM 本地会话链路补齐正式 V1 角色口径：`buyer_operator / seller_operator / platform_risk_settlement` 可读取 `auth/me`，门户本地会话支持显式或推断 `tenant_id`，页面主体条稳定显示主体、角色、租户/组织、作用域。
  - `/delivery/orders/:orderId/acceptance` 已替换为正式验收页，真实读取 `auth/me`、订单详情与 lifecycle 快照，展示交付结果摘要、验真结果、合同与模板匹配检查、生命周期摘要、八个标准 SKU、人工验收 SKU 边界、争议入口、权限态、空态、错态、加载态。
  - 确认验收与拒收表单使用 React Hook Form + Zod，要求 Hash / 合同 / 模板核验确认、审计确认、作用域确认和拒收原因；按钮可执行角色已按冻结矩阵收紧到 `buyer_operator`，拒收成功结果展示 `settlement_status=blocked`、`dispute_status=open` 等正式返回字段。
- 验证：
  - 前端 / SDK：
    - `pnpm install`
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - 权限收紧后补跑：`pnpm --filter @datab/portal-web test:unit`
    - 权限收紧后补跑：`pnpm --filter @datab/portal-web typecheck`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `TRADE_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo test -p platform-core dlv018_acceptance_db_smoke -- --nocapture`
    - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./scripts/check-openapi-schema.sh`
    - `cmp packages/openapi/delivery.yaml docs/02-openapi/delivery.yaml`
  - 真实联调与 smoke：
    - `./scripts/seed-local-iam-test-identities.sh`
    - 宿主机方式启动 `platform-core`，使用 `buyer_operator` 本地身份验证 `GET /api/v1/auth/me`、`GET /api/v1/orders/{id}`、`GET /api/v1/orders/{id}/lifecycle-snapshots`
    - `POST /api/v1/orders/{id}/accept` 携带 `X-Idempotency-Key` 验证验收通过与重复验收 `already_accepted`
    - `POST /api/v1/orders/{id}/reject` 携带 `X-Idempotency-Key` 验证拒收、结算阻断与争议打开
    - 缺失 `X-Idempotency-Key` 的 accept 请求返回 `400`，错误消息为 `x-idempotency-key is required for acceptance writes`
    - 门户浏览器 smoke：桌面与移动视口打开 `/delivery/orders/{orderId}/acceptance`，校验主体/角色/租户/作用域、SKU、request_id / tx_hash / 链状态 / 投影状态承接、验收表单、幂等键 header 与浏览器端仅访问 `/api/platform/**`
    - 数据库回查：验收订单状态、拒收订单状态、`delivery.delivery_record` 验收 snapshot、`audit.audit_event` 中 `delivery.accept / delivery.reject` metadata、`ops.outbox_event` 中 `billing.trigger.bridge / acceptance.passed / acceptance.rejected`、通知 outbox 均与页面动作一致；临时业务测试数据已清理，审计 / outbox 证据按 append-only 保留。
- 验证结果：
  - `pnpm install`、OpenAPI 生成、`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过；权限角色收紧后 `portal-web` 单元测试 `36` 个通过、`tsc --noEmit` 通过。
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`dlv018_acceptance_db_smoke`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh`、`./scripts/check-openapi-schema.sh` 全部通过；Rust 仍有既有 warning，本批未引入失败。
  - 两份 delivery OpenAPI 逐字同步；`accept/reject` SDK 类型不再漂移，幂等键 header 单测通过。
  - 真实 `curl` 验证通过：`auth/me` 返回 `local_test_user / buyer_operator / tenant_id=10000000-0000-0000-0000-000000000102`；accept 成功后订单进入 `accepted`，重复验收返回 `already_accepted`；reject 成功后订单进入 `rejected`，`settlement_status=blocked`、`dispute_status=open`。
  - 浏览器 smoke 通过：桌面端与移动端页面均可加载和交互，提交验收时捕获到 `/api/platform/api/v1/orders/{id}/accept` 且 `X-Idempotency-Key` 前缀为 `web-011-acceptance-accept-`；浏览器请求审计未发现直连 `platform-core` 或受限系统。
  - 数据库回查通过：验收 / 拒收订单、交付 snapshot、审计 metadata、billing bridge outbox 与通知 outbox 均与动作结果一致；本批插入的临时订单、SKU、商品、资产、交付记录已清理，业务临时数据残留为 `0`。
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-285` 无新增项。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-011`
  - `页面说明书-V1-完整版.md`：7.9 验收页
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`、`菜单树与路由表正式版.md`：验收页查看权限、主按钮权限与买方运营员动作边界
  - `业务流程图-V1-完整版.md`：文件/报告人工验收与拒收分支
  - `delivery-cases.md`、`payment-billing-cases.md`、`notification-cases.md`：重复验收、拒收阻断结算、通知 outbox 与审计保留
  - `packages/openapi/delivery.yaml`、`docs/02-openapi/delivery.yaml`、`packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`
- 覆盖的任务清单条目：`WEB-011`
- 未覆盖项：
  - 无。`WEB-011` 要求的验收页面、通过/拒收动作、拒收原因、生命周期摘要、权限态/空态/错态/加载态、SDK/OpenAPI 绑定、真实 API、E2E / 浏览器 smoke、数据库回查与日志留痕均已完成；完整争议创建页由后续 `WEB-013` 继续展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-286（计划中）
- 任务：`WEB-012` 实现账单页面：账单明细、支付状态、退款/赔付状态、争议入口
- 状态：计划中
- 说明：本批将 `/billing` 与 `/billing/refunds` 从路由脚手架升级为正式账单与售后页面，真实读取订单账单聚合、支付状态、结算结果、退款/赔付状态、发票/税务占位、SKU 计费规则和争议入口；高风险退款/赔付执行页必须使用正式 SDK、`X-Idempotency-Key` 与 `X-Step-Up-Token` / `X-Step-Up-Challenge-Id` 边界，不以前端 mock 替代后端执行。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-011` 已完成并提交，门户工程、受控 `/api/platform/**` 代理、会话主体条、订单详情、验收页和 `sdk-ts` 可复用。本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-012` 只实现账单页面与退款/赔付处理页，不合并 `WEB-013` 争议创建完整实现；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认账单中心需联查订单、账单事件、结算结果和保证金状态，核心字段为 `billing_event_id / order_id / settlement_status / deposit_status`；退款/赔付处理页需展示责任判定摘要、退款金额、赔付金额和扣罚明细。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认 V1 支付与轻结算要结构化表达货款、保证金、退款、赔付、人工打款与对账，且账务镜像、渠道结果和订单状态必须可联查。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认结算、退款、赔付流程从验收或争议裁决触发，写入 `billing_event / settlement_record / refund_record` 并保留审计摘要；争议裁决后的退款/赔付属于高风险执行。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认路由 `/billing`、`/billing/refunds`，查看权限 `billing.statement.read`，主操作权限 `billing.invoice.request`、`billing.refund.execute`、`billing.compensation.execute`，退款/赔付高风险且需要 step-up。
  - `docs/05-test-cases/payment-billing-cases.md`：确认必须覆盖支付状态不可回退、重复扣费防护、争议冻结、退款/赔付后结算重算、`GET /api/v1/billing/{order_id}` 联查账单摘要。
  - `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml`：确认 `GET /api/v1/billing/{order_id}`、`POST /api/v1/refunds`、`POST /api/v1/compensations` 已存在，退款/赔付写接口已声明 `x-idempotency-key` 和 step-up header；两份 OpenAPI 当前逐字同步。
  - `apps/platform-core/src/modules/billing/**`、`packages/sdk-ts/**`、`apps/portal-web/**`：确认后端已有 Billing detail、退款与赔付执行、审计、outbox 与通知逻辑；当前 `packages/sdk-ts` 尚未暴露 billing domain，门户 `/billing` 和 `/billing/refunds` 仍为 `PortalRoutePage` 脚手架。
- 当前完成标准理解：
  - 账单中心必须通过 `auth/me` 与 `GET /api/v1/billing/{order_id}` 真实展示当前主体、角色、租户/组织、作用域、账单事件、支付状态、结算状态、退款/赔付状态、发票/税务占位和争议入口。
  - 退款/赔付处理页必须展示责任判定摘要，使用 React Hook Form + Zod 校验 `order_id / case_id / decision_code / amount / reason_code / step-up / idempotency`，通过 SDK 调用正式写接口并回显结果、统一错误码与 `request_id`。
  - 高风险按钮只按正式 V1 角色口径展示给 `platform_risk_settlement`，页面必须显式提示 step-up 和审计强留痕；账单读取至少按 `tenant_admin / buyer_operator / platform_risk_settlement` 正式种子权限承接，不把 `tenant_operator / platform_finance_operator` 传播到前端。
  - 页面必须支持加载态、空态、错态、权限态、桌面/移动加载、E2E 预演、真实 API smoke、数据库回查和受限系统边界验证。
- 实施计划：
  1. 新增 `packages/sdk-ts` billing domain、导出和单测，封装 `getBillingOrder / executeRefund / executeCompensation`，写操作强制幂等键并支持 step-up header。
  2. 必要时收敛 billing read 后端角色口径，使 `GET /api/v1/billing/{order_id}` 与 `billing.statement.read` 正式种子权限一致。
  3. 新增 `billing-workflow` 视图模型、Zod/RHF schema、角色判断、金额/状态格式化、响应解包和错误格式化测试。
  4. 替换 `/billing` 和 `/billing/refunds` 为正式页面，接入真实 API、账单明细、支付/结算/退款/赔付状态、责任摘要、争议入口、step-up/幂等表单和结果区。
  5. 更新路由 API 绑定、Playwright 覆盖、OpenAPI checker，执行前端、SDK、后端、OpenAPI、真实 API、浏览器 smoke、数据库回查与受限系统边界验证。

### BATCH-286（待审批）
- 任务：`WEB-012` 实现账单页面：账单明细、支付状态、退款/赔付状态、争议入口
- 状态：待审批
- 当前任务编号：`WEB-012`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-011` 的门户、订单、交付与验收基线继续生效。页面实现只通过 `portal-web -> /api/platform -> platform-core` 访问正式 API，没有新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 完成情况：
  - 新增 `packages/sdk-ts` billing domain，正式封装 `GET /api/v1/billing/{order_id}`、`POST /api/v1/refunds`、`POST /api/v1/compensations`，写接口强制传入 `X-Idempotency-Key` 并支持 `X-Step-Up-Token / X-Step-Up-Challenge-Id`。
  - 将 `/billing` 替换为正式账单中心，展示当前主体、角色、租户、作用域、订单账单聚合、账单事件、支付状态、结算摘要、退款/赔付状态、发票/税务占位、SKU 计费规则、`request_id / tx_hash / 链状态 / 投影状态` 承接与争议入口。
  - 将 `/billing/refunds` 替换为正式退款/赔付处理页，展示责任判定摘要、退款金额、赔付金额、扣罚/调整信息；表单使用 `React Hook Form + Zod` 校验 `order_id / case_id / decision_code / amount / reason_code / step-up / idempotency`，并通过 SDK 调用正式写接口。
  - 前端角色口径按正式 V1 种子收敛：账单读取承接 `tenant_admin / buyer_operator / platform_risk_settlement`，退款/赔付高风险执行仅展示给 `platform_risk_settlement`；没有把 `tenant_operator / platform_finance_operator` 传播到前端权限判断。
  - 后端 `GET /api/v1/billing/{order_id}` 读取权限与 `billing.statement.read` 种子权限对齐，补充 `buyer_operator` 读账单能力，并增加 DB smoke 覆盖同租户买方运营员读账单成功。
  - 两份 billing OpenAPI 继续逐字同步，并修正税务占位示例字段为当前 schema / 后端实现中的 `tax_engine_status / tax_rule_code / currency_code / latest_invoice_title / latest_tax_no / tax_breakdown_ready`；`scripts/check-openapi-schema.sh` 增加 billing detail、refund、compensation、幂等与 step-up 最小防漂移检查。
  - `apps/portal-web/e2e/smoke.spec.ts` 覆盖账单页与退款/赔付页的权限态、空态、错态；路由元数据同步声明 `/api/v1/auth/me`、billing detail、refund、compensation API 绑定。
- 验证：
  - 前端 / 工作区：
    - `pnpm install`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
  - 后端 / 契约：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./scripts/check-openapi-schema.sh`
  - 真实联调与 smoke：
    - `./scripts/seed-local-iam-test-identities.sh`
    - 宿主机方式启动 `platform-core`：`APP_MODE=local APP_PORT=8094 ... cargo run -p platform-core`
    - `curl` 验证 `GET /api/v1/auth/me` 返回 `local_test_user / buyer_operator / tenant_id=10000000-0000-0000-0000-000000000102`
    - `curl` 验证 `GET /api/v1/billing/30000000-0000-0000-0000-000000000110` 返回账单事件、结算摘要、退款记录、赔付记录、发票/税务占位和 `SHARE_RO` SKU 计费规则
    - `curl` 验证卖方角色读同一买方订单返回 `403 IAM_UNAUTHORIZED`
    - `curl` 验证退款/赔付写接口缺少 step-up 时分别返回 `400 IAM_UNAUTHORIZED`，且请求携带 `X-Idempotency-Key`
    - 浏览器 smoke：桌面 `1440x920` 与移动 `390x900` 打开 `/billing?order_id=...`、`/billing/refunds?order_id=...&case_id=...`，校验主体/角色/租户、账单事件、退款/赔付表单、step-up 提示与浏览器请求仅访问 `/api/platform/**`
    - 数据库回查：`audit.audit_event` 中出现 `iam.session.context.read` 与 `billing.order.read` 审计事件；临时账单事件、结算、退款、赔付、发票记录测试后已清理为 `0`，审计记录按 append-only 保留。
- 验证结果：
  - 所有前端、SDK、后端、SQLx、OpenAPI 与查询编译校验均通过；`pnpm test` 期间仍可看到既有 `127.0.0.1:8094` 代理噪声，但最终 console / portal Playwright smoke 均通过。
  - 真实 API 联调通过：billing detail 返回 `billing_events[0].event_type=one_time_charge`、`settlement_summary.summary_state=order_settlement:pending:manual`、退款 `processing`、赔付 `pending`、发票占位 `pending`；权限态和 step-up 拦截均按统一错误码回显。
  - 浏览器 smoke 捕获到 `6` 次 `/api/platform/...` 请求，`directRestrictedCalls=0`，未发现浏览器直连 `platform-core` 或受限系统。
  - 数据库清理回查通过：`billing.billing_event / settlement_record / refund_record / compensation_record / invoice_request` 中本批临时业务数据残留均为 `0`；审计事件按要求保留。
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-286` 无新增项。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-012`
  - `页面说明书-V1-完整版.md`：8.1 账单中心、8.2 退款/赔付处理页
  - `数据交易平台-全集成基线-V1.md`：27. 支付、资金流与轻结算
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`、`菜单树与路由表正式版.md`：账单查看、退款、赔付、争议入口权限
  - `业务流程图-V1-完整版.md`、`payment-billing-cases.md`：争议冻结、退款/赔付后结算重算、账单联查与幂等/step-up
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`、`packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`
- 覆盖的任务清单条目：`WEB-012`
- 未覆盖项：
  - 无。`WEB-012` 要求的账单明细、支付状态、退款/赔付状态、争议入口、权限态/空态/错态/加载态、SDK/OpenAPI 绑定、真实 API、E2E / 浏览器 smoke、数据库回查与日志留痕均已完成；完整争议创建和证据上传由后续 `WEB-013` 继续展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-287（计划中）
- 任务：`WEB-013` 实现争议页面：创建案件、上传证据、查看裁决
- 状态：计划中
- 说明：本批将 `/support/cases/new` 从路由脚手架升级为正式争议提交与跟踪页，围绕买方发起争议、补充证据、平台风控裁决查看/执行入口形成最小闭环。页面必须真实读取当前主体与订单争议关系，通过 `packages/sdk-ts` 调用 `platform-core` 的 `POST /api/v1/cases`、`POST /api/v1/cases/{id}/evidence`、`POST /api/v1/cases/{id}/resolve` 和 `GET /api/v1/orders/{id}`；所有写动作由前端生成并透传 `X-Idempotency-Key`，裁决动作展示并透传 step-up token，证据展示不得暴露对象真实路径。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-012` 已完成并提交，门户工程、受控 `/api/platform/**` 代理、主体条、订单详情、账单与退款/赔付 SDK 基线可复用。本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-013` 只实现争议页面，不合并审计联查、通知联查或后续 ops / developer 页面；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认争议提交页目标为提交争议、补充证据并跟踪裁决进度；核心模块为争议原因选择、证据上传、案件时间线、裁决结果区；关键字段为 `case_id`、`reason_code`、`evidence_hash`。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认争议流程从买方拒收或任一方 `open_case` 进入 `DisputeService`，证据包括交付回执、下载日志、对象 Hash、密钥信封、合同快照、沟通说明，裁决输出 `decision_code / penalty_code` 并影响退款、赔付、保证金、信誉和风险标记。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认审计与证据链接口边界，争议页只展示证据哈希、审计线索与链路状态，不暴露对象存储真实路径；完整审计包导出与回放由后续审计页面承接。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认路由 `/support/cases/new`，查看权限 `dispute.case.read`，主按钮权限 `dispute.case.create / dispute.evidence.upload`，裁决权限 `dispute.case.resolve` 属于平台侧高风险动作并要求 step-up / 审计提示。
  - 通用边界文档、OpenAPI 冻结表、统一错误码字典、测试矩阵、本地环境与配置项文档、技术选型、总体架构与全集成基线：确认前端只调用 `platform-core` 正式 API；写接口必须携带 `X-Idempotency-Key`；敏感页面必须展示主体、角色、租户/组织、作用域；不得发明 SKU、状态或错误码语义。
  - `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml`：确认争议创建、证据上传、裁决三个接口和 `DisputeCase / DisputeEvidence / DisputeResolution` schema 已冻结且两份同步；证据上传为 `multipart/form-data`。
  - `packages/openapi/trade.yaml` 与 `docs/02-openapi/trade.yaml`：确认 `GET /api/v1/orders/{id}` 的 `relations.disputes` 可作为查看已有案件与裁决摘要的正式读取来源；当前没有单独 `GET /api/v1/cases/{id}` 读取接口。
  - `packages/openapi/audit.yaml`、`iam.yaml` 及对应归档副本：确认主体上下文、审计线索与证据包边界；本批不提前实现审计导出。
  - `apps/platform-core/src/modules/billing/**`、`apps/platform-core/src/modules/order/**`、`packages/sdk-ts/**`、`apps/portal-web/**`：确认后端已有争议状态机、证据对象存储、审计、通知 outbox、结算冻结与 step-up 占位校验；当前门户争议路由仍为 `PortalRoutePage` 脚手架，SDK billing domain 尚未封装争议接口和 multipart 上传。
- 当前完成标准理解：
  - 争议页必须读取 `auth/me` 与订单详情，显示主体、角色、租户/组织、作用域、订单/SKU、已有案件时间线、裁决摘要、request_id / tx_hash / 链状态 / 投影状态承接；未返回字段必须显式标注“未返回”。
  - 发起争议与上传证据仅对具备买方争议权限的角色展示可执行态；无权限、缺订单、缺案件、后端错误和空时间线必须分别有明确状态。
  - 表单必须使用 React Hook Form + Zod 校验，提交时经 SDK 透传 `X-Idempotency-Key`，重复点击期间禁用主按钮，后端错误码与 `request_id` 要回显。
  - 证据上传必须使用正式 multipart 接口，页面只展示 `object_hash / evidence_hash` 与审计元数据，不展示 `object_uri` 或对象路径。
  - 裁决区域必须展示 `decision_code`、`penalty_code`、`step_up_bound`、`idempotent_replay` 与审计留痕提示；平台裁决提交需要 step-up token，不把高风险动作做成假入口。
- 实施计划：
  1. 扩展 `packages/sdk-ts` 的 billing domain，新增争议创建、证据上传 multipart、裁决提交方法与单测，并在 HTTP client 中补支持 FormData 的受控请求路径。
  2. 新增争议页面视图模型、Zod/RHF schema、权限判断、响应解包、错误格式化、幂等键生成和单元测试。
  3. 替换 `/support/cases/new` 为正式 `DisputeWorkflowShell`，接入真实 API、权限态、空态、错态、加载态、幂等键、step-up、审计提示、案件时间线与裁决结果区。
  4. 更新路由元数据、E2E smoke 与 OpenAPI 防漂移检查，执行前端/后端/契约验证、真实 curl + 浏览器 smoke、数据库回查和测试数据清理。

### BATCH-287（待审批）
- 任务：`WEB-013` 实现争议页面：创建案件、上传证据、查看裁决
- 状态：待审批
- 当前任务编号：`WEB-013`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-012` 已完成并作为本批基线。实现保持 `portal-web -> /api/platform -> platform-core` 边界，未新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 完成情况：
  - `packages/sdk-ts` 扩展 Billing domain：新增 `createDisputeCase`、`uploadDisputeEvidence`、`resolveDisputeCase`，写操作统一透传 `X-Idempotency-Key`，裁决透传 `X-Step-Up-Token / X-Step-Up-Challenge-Id`，并在 `PlatformClient` 新增 `postFormData`，避免 multipart 被 JSON 化。
  - `apps/portal-web/src/app/api/platform/[...path]/route.ts` 将非 GET/HEAD 代理体改为 `arrayBuffer()`，保留 multipart 二进制证据上传能力，不向浏览器暴露对象存储真实路径。
  - 新增 `apps/portal-web/src/lib/dispute-workflow.ts` 与单测，覆盖争议创建、证据 FormData、裁决 step-up 校验、正式 V1 角色口径、案件选择、错误码格式化和幂等键生成。
  - 新增 `DisputeWorkflowShell` 并替换 `/support/cases/new`：页面读取 `auth/me` 与 `GET /api/v1/orders/{id}` 的 `relations.disputes`，展示主体、角色、租户/组织、作用域、订单/SKU、案件时间线、裁决摘要、审计与链路承接；创建/证据/裁决表单均用 React Hook Form + Zod 校验。
  - 证据上传区调用正式 multipart 接口，只展示 `evidence_id / object_type / evidence_hash / idempotent_replay`，显式隐藏 `object_uri`；裁决区显示 `decision_code / penalty_code / step_up_bound / idempotent_replay`，无 step-up 时由后端统一错误码拦截。
  - 更新 `portal-routes` 与 Playwright smoke，`dispute_create` 现在显式列出 `auth/me`、订单详情、创建案件、上传证据、裁决提交五个 API 绑定。
  - `packages/openapi/billing.yaml` 与 `docs/02-openapi/billing.yaml` 为三个争议写接口补齐 `x-idempotency-key` 头声明，`scripts/check-openapi-schema.sh` 增加争议接口和 multipart schema 防漂移检查。
- 验证：
  - 前端 / SDK / 契约：
    - `pnpm install`
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
    - `./scripts/check-openapi-schema.sh`
    - 根级 `pnpm lint`
    - 根级 `pnpm typecheck`
    - 根级 `pnpm test`
    - 根级 `pnpm build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调与 smoke：
    - `./scripts/verify-local-stack.sh core`
    - `./scripts/seed-local-iam-test-identities.sh`
    - 宿主机方式启动 `platform-core`：`APP_MODE=local APP_PORT=8094 PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 cargo run -p platform-core-bin`
    - 临时种入 `FILE_STD` 订单 `30000000-0000-0000-0000-000000013001`，直接 `curl` 验证 `auth/me`、订单详情、`POST /api/v1/cases`、`POST /api/v1/cases/{id}/evidence`、缺 step-up 的 `POST /resolve` 失败、带 step-up 的正式裁决成功。
    - 真实返回：`auth=local_test_user:buyer_operator:10000000-0000-0000-0000-000000000102`；创建案件返回 `current_status=opened / reason_code=delivery_failed`；证据上传返回 `object_hash` 且 `idempotent_replay=false`；缺 step-up 裁决返回 `400 IAM_UNAUTHORIZED`；正式裁决返回 `current_status=resolved / decision_code=refund_full / step_up_bound=true`。
    - 数据库回查：`support.dispute_case(status=resolved, decision_code=refund_full, penalty_code=seller_full_refund)`、`support.evidence_object(object_type=delivery_receipt, object_hash IS NOT NULL)`、审计桥接 `audit_evidence_item_id / audit_evidence_manifest_id`、`audit.audit_event` 中 `dispute.case.create / dispute.evidence.upload / dispute.case.resolve` 及冻结/重算审计、`ops.outbox_event` 中 `dispute.created / dispute.resolved / notification.requested`。
    - 生产构建方式启动 `portal-web`：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:8094 pnpm --filter @datab/portal-web exec next start --hostname 127.0.0.1 --port 3113`，使用 Playwright + HttpOnly 本地会话 Cookie 在桌面 `1440x920` 与移动 `390x900` 打开 `/support/cases/new?order_id=...&case_id=...`，确认主体、订单状态、案件时间线、`resolved/refund_full`、上传证据区和平台裁决区可见。
    - 浏览器请求回查：捕获 `6` 次 `/api/platform/**` 请求，`directForbiddenCalls=0`，未发现浏览器直连 `platform-core` 或受限系统。
    - 清理：删除临时 MinIO 证据对象、`support.dispute_case / support.evidence_object / support.decision_record / ops.outbox_event / billing.settlement_record / payment.payment_intent / trade.order_main / catalog.product_sku / catalog.product / catalog.asset_version / catalog.data_asset`，业务残留回查均为 `0`；审计记录按 append-only 保留。
- 验证结果：
  - 所有前端、SDK、OpenAPI、后端、SQLx 与查询编译校验均通过；`pnpm test` 中 portal / console 仍出现既有 `127.0.0.1:8094` 未启动时的代理噪声，但对应 Playwright 用例最终通过，本批另行完成了启动真实 `platform-core` 的浏览器 smoke。
  - `POST /api/v1/cases/{id}/evidence` 经 `/api/platform` 代理保留 multipart 二进制体，证据对象真实写入 MinIO 后已删除；页面没有展示 `object_uri`。
  - 权限与 step-up：买方角色可创建案件与上传证据；平台风控结算角色裁决需要 step-up；缺 step-up 的 curl 返回统一错误码 `IAM_UNAUTHORIZED`。
  - 临时业务测试数据已清理，审计事件、证据审计桥和访问审计按 append-only 规则保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-013`
  - `页面说明书-V1-完整版.md`：8.3 争议提交页
  - `业务流程图-V1-完整版.md`：5.3 争议处理流程
  - `审计、证据链与回放接口协议正式版.md`：V1 审计 / 证据链接口边界
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`、`菜单树与路由表正式版.md`：争议读取、创建、证据上传、裁决权限与 step-up 边界
  - `packages/openapi/billing.yaml`、`docs/02-openapi/billing.yaml`、`packages/openapi/trade.yaml`、`docs/02-openapi/trade.yaml`、`packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`
- 覆盖的任务清单条目：`WEB-013`
- 未覆盖项：
  - 无。`WEB-013` 要求的创建案件、上传证据、查看裁决、权限态/空态/错态/加载态、SDK/OpenAPI 绑定、真实 API、E2E / 浏览器 smoke、数据库回查、证据路径隐藏与日志留痕均已完成；审计导出/回放与通知联查继续由后续 WEB 任务承接。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-287` 无新增项。

### BATCH-288（计划中）
- 任务：`WEB-014` 实现审计联查页：按订单号查看审计事件、证据对象、链回执、外部事实
- 状态：计划中
- 说明：本批将控制台 `/ops/audit/trace` 与证据包导出入口从路由脚手架升级为正式审计联查页面。页面必须真实读取当前主体与权限上下文，通过 `packages/sdk-ts` 调用 `platform-core` 的 `GET /api/v1/audit/traces`、`GET /api/v1/audit/orders/{id}`、`GET /api/v1/developer/trace`、`GET /api/v1/ops/trade-monitor/orders/{orderId}`、`GET /api/v1/ops/external-facts`、`GET /api/v1/ops/projection-gaps` 与 `POST /api/v1/audit/packages/export`；导出动作必须携带 `X-Idempotency-Key` 与 step-up 头，页面不得展示对象存储真实路径。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-013` 已完成并提交，`console-web` 工程、受控 `/api/platform/**` 代理、主体条、争议/账单/交付/订单页面基线可复用。本批继续保持 `console-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-014` 只实现审计联查页，不合并通知联查、ops 总览或开发者完整页面；DoD 要求页面可访问、空态/错态/权限态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认审计联查页按 `order_id / request_id / tx_hash / case_id / delivery_id` 查询，并展示订单状态、链事件、账单事件、交付记录、审计事件、错误码；证据包导出页必须导出主体摘要、商品/合同快照、交付回执、下载日志、裁决结果和链摘要，且下载类页面不得暴露真实对象路径。
  - `docs/数据库设计/接口协议/审计、证据链与回放接口协议正式版.md`：确认 V1 正式接口为 `audit/traces`、`audit/evidence-manifests/{id}`、`audit/packages/export`、replay、legal hold 与 anchor batches；导出、原文查看、回放等敏感动作必须有权限、step-up 与审计留痕。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认 `/ops/audit/trace` 查看权限 `audit.trace.read`，证据包导出入口 `/ops/audit/packages` 查看/主动作权限为 `audit.event.read / audit.package.export`，导出属于高风险动作并要求人工确认 / step-up。
  - 通用边界文档、OpenAPI 冻结表、统一错误码字典、测试矩阵、本地环境、配置项、技术选型、总体架构与全集成基线：确认前端只调用 `platform-core` 正式 API；写接口必须携带 `X-Idempotency-Key`；敏感页面展示主体、角色、租户、作用域；链页面展示 `request_id / tx_hash / 链状态 / 投影状态`；不得发明 SKU、状态或错误码语义。
  - `docs/05-test-cases/audit-consistency-cases.md`、`delivery-cases.md`、`payment-billing-cases.md`、`notification-cases.md`：确认审计联查、证据包导出、贸易监控、外部事实、投影差异、开发者 trace 与数据库回查的验收点。
  - `packages/openapi/audit.yaml`、`ops.yaml`、`iam.yaml` 与对应 `docs/02-openapi/*.yaml`：确认审计联查、证据导出、开发者 trace、贸易监控、外部事实、投影差异和主体上下文 schema；两份 OpenAPI 副本当前同步。
  - `apps/platform-core/src/modules/audit/**`、`apps/platform-core/src/modules/iam/**`、`packages/sdk-ts/**`、`apps/console-web/**`：确认后端已有正式审计/ops 路由、权限矩阵、step-up 头读取、导出返回结构和控制台路由脚手架；SDK audit / ops domain 仍需补齐当前页面用到的封装方法。
- 当前完成标准理解：
  - 审计联查页必须真实支持 `order_id / request_id / tx_hash / case_id / delivery_id` 五类查询键，并按正式 API 返回展示审计事件、证据对象、链回执、外部事实、投影差异、订单/交付/账单事件分组与错误码。
  - 页面必须展示当前主体、角色、租户/组织、作用域；无权限、空结果、后端错误、加载中和高风险导出缺 step-up 都必须有明确状态。
  - 证据包导出必须使用 React Hook Form + Zod 校验，前端生成并透传 `X-Idempotency-Key`，传递 `X-Step-Up-Token` 或等价 challenge，重复点击期间禁用主按钮，成功后仅展示 package / manifest / digest / count 等元信息，不展示 `storage_uri`。
  - 长列表必须使用分页或虚拟滚动策略；审计事件表需要排序/筛选/空态/加载态，不能只渲染静态样例。
  - 页面与 E2E / smoke 必须证明浏览器只访问 `/api/platform/**`，SDK 与 OpenAPI 契约一致，前端没有直连受限系统或对象存储。
- 实施计划：
  1. 扩展 `packages/sdk-ts` 的 audit / ops domain，补齐审计订单、证据包导出、贸易监控、外部事实、投影差异与链路联查 SDK 方法及单测。
  2. 新增控制台审计联查视图模型、Zod/RHF schema、权限判断、查询键映射、错误码格式化、证据对象隐藏、幂等键生成与单元测试。
  3. 替换 `/ops/audit/trace` 与 `/ops/audit/packages` 为正式业务组件，接入真实 API、TanStack Query/Table/Virtual、权限态、空态、错态、加载态、step-up、导出结果和审计留痕提示。
  4. 更新 OpenAPI 防漂移检查、控制台路由元数据和 Playwright smoke，执行前端/后端/契约验证、真实 curl + 浏览器 smoke、数据库回查和测试数据清理。

### BATCH-288（待审批）
- 任务：`WEB-014` 实现审计联查页：按订单号查看审计事件、证据对象、链回执、外部事实
- 状态：待审批
- 当前任务编号：`WEB-014`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-013` 已完成并作为本批基线。实现保持 `console-web -> /api/platform -> platform-core` 边界，未新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 完成情况：
  - `packages/sdk-ts` 扩展 Audit / Ops domain：新增 `getOrderAudit`、`exportPackage`、`listAnchorBatches`、`getTradeMonitorOverview`、`listTradeMonitorCheckpoints`、`listExternalFacts`、`listProjectionGaps`、`getConsistency`，证据包导出正式透传 `X-Idempotency-Key`、`X-Step-Up-Token / X-Step-Up-Challenge-Id`。
  - `packages/openapi/audit.yaml` 与 `docs/02-openapi/audit.yaml` 同步为 `POST /api/v1/audit/packages/export` 补齐幂等与 step-up 头声明，`scripts/check-openapi-schema.sh` 增加防漂移 token 检查；生成后的 `packages/sdk-ts/src/generated/audit.ts` 已更新。
  - 新增 `apps/console-web/src/lib/audit-trace.ts` 与单测，覆盖五类联查键映射、正式 V1 审计角色、developer trace 权限、导出 step-up 校验、证据对象 `storage_uri` 隐藏、审计事件分组、幂等键生成和错误码格式化。
  - 新增 `AuditTraceShell` / `AuditPackageExportShell` 并替换 `/ops/audit/trace`、`/ops/audit/packages`：页面读取 `auth/me`，展示主体、角色、租户/组织、作用域，支持 `order_id / request_id / tx_hash / case_id / delivery_id` 联查，展示订单状态、审计事件、链回执、外部事实、投影差异、生命周期 checkpoint、证据对象和证据包导出结果。
  - 审计事件长列表使用 TanStack Query + TanStack Table + TanStack Virtual，支持事件组筛选、时间排序、加载态、空态、错态与权限态；证据包导出表单使用 React Hook Form + Zod，导出结果只显示 package / manifest / digest / count，不展示真实对象路径。
  - 控制台会话修正：本地 / Bearer 会话在 `auth/me` 验证后保存并透传 `user_id / tenant_id / role`，使高风险审计导出经 `/api/platform/**` 代理时满足后端 `x-user-id` 要求，不依赖人工补头。
  - `console-routes` 与 Playwright smoke 已更新，`audit_trace` 的 API 绑定显式列出 `auth/me`、audit、developer trace、trade monitor、external facts、projection gaps 和 package export；布局补 `min-w-0 / overflow-x-hidden`，确保桌面和移动页面级响应式加载不被长表格污染。
- 验证：
  - 前端 / SDK / 契约：
    - `pnpm install`
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/console-web lint`
    - `pnpm --filter @datab/console-web typecheck`
    - `pnpm --filter @datab/console-web test:unit`
    - `pnpm --filter @datab/console-web test:e2e`
    - `pnpm --filter @datab/console-web build`
    - `./scripts/check-openapi-schema.sh`
    - 根级 `pnpm lint`
    - 根级 `pnpm typecheck`
    - 根级 `pnpm test`
    - 根级 `pnpm build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调与 smoke：
    - `./scripts/verify-local-stack.sh core`
    - `./scripts/seed-local-iam-test-identities.sh`
    - 宿主机方式启动 `platform-core`：`APP_MODE=local APP_PORT=8094 APP_HOST=127.0.0.1 PROVIDER_MODE=mock KAFKA_BROKERS=127.0.0.1:9094 cargo run -p platform-core-bin`
    - 使用 `auditor.admin@luna.local / platform_audit_security` 本地控制台主体直连 `platform-core` 验证：
      - `GET /api/v1/auth/me`
      - `GET /api/v1/audit/traces?order_id=0b0c5dce-3fca-420e-b416-2433a1552e3e`
      - `GET /api/v1/audit/orders/0b0c5dce-3fca-420e-b416-2433a1552e3e`
      - `GET /api/v1/developer/trace?tx_hash=0xaud0241776853622420`
      - `GET /api/v1/ops/trade-monitor/orders/0b0c5dce-3fca-420e-b416-2433a1552e3e`
      - `GET /api/v1/ops/external-facts?order_id=0b0c5dce-3fca-420e-b416-2433a1552e3e`
      - `GET /api/v1/ops/projection-gaps?order_id=0b0c5dce-3fca-420e-b416-2433a1552e3e`
      - `POST /api/v1/audit/packages/export` 缺 step-up 失败、带 `X-Step-Up-Token` 与 `X-Idempotency-Key` 成功。
    - 生产构建方式启动 `console-web`：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:8094 pnpm --filter @datab/console-web exec next start --hostname 127.0.0.1 --port 3114`。
    - 经控制台 `/api/platform/**` 代理验证 `auth/me`、`audit/traces` 与证据包导出缺 step-up 错误，确认代理已透传本地会话 `user_id / tenant_id / role`，错误收敛为 step-up 缺失而不是主体缺失。
    - 使用 Playwright + `datab_console_session` HttpOnly Cookie 在桌面 `1440x980` 与移动 `390x900` 打开 `/ops/audit/trace`，输入订单 `0b0c5dce-3fca-420e-b416-2433a1552e3e` 后确认 `buyer_locked`、`anchored`、`fabric_submit_receipt`、`projection_lag`、链回执、外部事实、投影差异可见；浏览器捕获 `8` 个 `/api/platform/**` 请求，`directForbiddenCount=0`，`documentElement` 桌面 / 移动横向检查均通过。
    - 数据库回查：
      - `audit.audit_event` 中该订单审计事件与 `audit.package.export` 导出事件可查。
      - `audit.evidence_package / audit.evidence_manifest` 中导出 package / manifest 各 `1` 条，`audit.access_audit` 与 `ops.system_log` 均有 `request_id=web014-export-success` 记录。
      - `trade.order_main` 返回 `status=buyer_locked`、`payment_status=paid`、`proof_commit_state=anchored`、`external_fact_status=confirmed`、`reconcile_status=matched`。
      - `ops.external_fact_receipt` 返回 `fabric_submit_receipt`，`ops.chain_projection_gap` 返回 `projection_lag/open`，`ops.trade_lifecycle_checkpoint` 返回对应 lifecycle checkpoint。
- 验证结果：
  - 所有前端、SDK、OpenAPI、后端、SQLx 与查询编译校验均通过；`pnpm test` 中 portal / console 的 Playwright WebServer 仍会打印既有 `127.0.0.1:8094` 未启动时的代理噪声，但对应 E2E 用例最终通过，本批另行完成了启动真实 `platform-core` 的控制台浏览器 smoke。
  - 权限与 step-up：`platform_audit_security` 可读审计联查与 developer trace；证据包导出缺 step-up 返回正式错误，带 step-up 与幂等键成功；经控制台代理时已透传本地主体 `x-user-id / x-tenant-id / x-role`。
  - 前端边界：浏览器真实请求只落 `/api/platform/**`，未发现直连 `platform-core`、`PostgreSQL`、`Kafka`、`OpenSearch`、`Redis`、`Fabric` 或 `MinIO`；页面导出结果隐藏 `storage_uri`。
  - 证据包导出产生的 package、manifest、MinIO 对象与对应审计记录作为 append-only 审计 artifact 保留；本批未创建需要清理的业务订单 / 争议 / 账单临时数据。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-014`
  - `页面说明书-V1-完整版.md`：10.1 审计联查页、10.2 证据包导出页、全局页面规范
  - `审计、证据链与回放接口协议正式版.md`：V1 审计联查、证据包导出、step-up 与审计留痕边界
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`、`菜单树与路由表正式版.md`：审计联查、证据包导出、developer trace、ops trade monitor / external facts / projection gaps 权限边界
  - `audit-consistency-cases.md`、`delivery-cases.md`、`payment-billing-cases.md`：审计联查、证据导出、链回执、外部事实、投影差异、数据库回查验收点
  - `packages/openapi/audit.yaml`、`ops.yaml`、`iam.yaml` 与 `docs/02-openapi/*.yaml`
- 覆盖的任务清单条目：`WEB-014`
- 未覆盖项：
  - 无。`WEB-014` 要求的按订单号查看审计事件、证据对象、链回执、外部事实、权限态/空态/错态/加载态、SDK/OpenAPI 绑定、真实 API、E2E / 浏览器 smoke、数据库回查、证据路径隐藏与日志留痕均已完成；通知联查、ops 页面和开发者页面完整业务细节继续由后续 WEB task 承接。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-288` 无新增项。

### BATCH-289（计划中）
- 任务：`WEB-015` 实现 ops 页面：outbox、dead letter、一致性联查、搜索同步、推荐重建、观测总览入口
- 状态：计划中
- 说明：本批将控制台 `/ops/consistency`、`/ops/consistency/outbox` 与 `/ops/search` 从路由脚手架升级为正式 ops 工作台。页面必须真实读取当前主体与权限上下文，通过 `packages/sdk-ts` 调用 `platform-core` 的 `GET /api/v1/ops/consistency/{refType}/{refId}`、`POST /api/v1/ops/consistency/reconcile`、`GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`、`POST /api/v1/ops/dead-letters/{id}/reprocess`、`GET /api/v1/ops/observability/overview`、搜索运维接口与推荐运维接口；写动作由前端生成并透传 `X-Idempotency-Key`，高风险动作展示并透传 step-up token / challenge。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-014` 已完成并作为本批基线。实现继续保持 `console-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-015` 只实现 ops 页面，不合并开发者页面、通知联查或全量可观测性子页；DoD 要求页面可访问、空态/错态/权限态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认状态联查页按关键主键联查订单、链事件、账单、交付和审计；搜索运维页覆盖索引同步任务、文档版本、Reindex、Alias、Redis 缓存失效、排序配置与权重快照；后续章节补充 Outbox / Dead Letter 与可观测性总览字段。
  - `docs/原始PRD/日志、可观测性与告警设计.md`：确认审计权威为 `audit.*`，运行观测权威为 `ops.* + Loki/Tempo/Prometheus`，日志/观测访问也必须审计；V1 观测栈采用 OpenTelemetry、Prometheus、Alertmanager、Grafana、Loki、Tempo。
  - 通用边界文档、OpenAPI 冻结表、统一错误码字典、测试矩阵、本地环境、配置项、技术选型、总体架构与全集成基线：确认前端只调用 `platform-core` 正式 API；`Kafka / PostgreSQL / OpenSearch / Redis / Fabric` 均不得前端直连；搜索与推荐必须回 PostgreSQL 最终校验，Kafka/outbox 只是同步与隔离边界。
  - `docs/权限设计/按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`、`菜单树与路由表正式版.md`：确认 `ops.consistency.read/reconcile`、`ops.outbox.read`、`ops.dead_letter.read/reprocess`、`ops.search_sync.read`、`ops.search_reindex.execute`、`ops.search_alias.manage`、`ops.search_cache.invalidate`、`ops.search_ranking.read/manage`、`ops.recommendation.read/manage`、`ops.recommend_rebuild.execute` 与 `ops.observability.read` 的页面和按钮边界。
  - `docs/05-test-cases/search-rec-cases.md`、`audit-consistency-cases.md`、`notification-cases.md`、`docs/04-runbooks/search-reindex.md`、`recommendation-runtime.md`、`opensearch-local.md` 与 `infra/docker/docker-compose.local.yml`：确认搜索运维、推荐重建、consumer 幂等、双层 DLQ、dead letter dry-run 重处理、观测后端与本地 OpenSearch / Redis / Kafka 运行口径。
  - `packages/openapi/ops.yaml`、`search.yaml`、`recommendation.yaml`、`iam.yaml` 与对应 `docs/02-openapi/*.yaml`：确认本批使用的 ops / search / recommendation / iam 契约；两份 OpenAPI 副本当前同步。`ops` 的 dead-letter / consistency dry-run 后端要求 step-up，本批前端仍会额外透传幂等键以满足写页面幂等提交要求。
  - `apps/platform-core/src/modules/audit/**`、`search/**`、`recommendation/**`、`iam/**`、`packages/sdk-ts/**`、`apps/console-web/**`：确认后端已有正式路由、权限、审计和 step-up 逻辑；`sdk-ts` 当前缺少多个 ops/search/recommendation domain 封装，控制台三个路由仍为 `ConsoleRoutePage` 脚手架。
- 当前完成标准理解：
  - `/ops/consistency` 必须支持正式 `refType/refId` 联查，展示业务主状态、证明状态、外部事实、recent outbox、recent dead letter、recent audit trace、链状态与投影状态；dry-run 修复表单必须显示幂等键、step-up 和审计强留痕提示。
  - `/ops/consistency/outbox` 必须支持 outbox 与 dead letter 筛选、分页或虚拟滚动、发布尝试与 consumer 幂等摘要、dead letter dry-run 重处理表单、权限态/空态/错态/加载态。
  - `/ops/search` 必须真实读取搜索同步任务、排序配置、推荐 placements / ranking profiles，并提供 reindex、alias switch、cache invalidate、search ranking patch 与 recommendation rebuild 的正式入口；写操作必须使用 React Hook Form + Zod 校验、透传 `X-Idempotency-Key`，高风险动作透传 step-up。
  - 观测总览入口必须真实读取 `GET /api/v1/ops/observability/overview`，展示后端探针、告警、incident、SLO 与关键服务摘要，不把 Grafana/Loki/Tempo 链接当作前端直连数据源。
  - 页面必须展示当前主体、角色、租户、作用域，错误码按后端统一错误响应回显，浏览器请求只能落 `/api/platform/**`。
- 实施计划：
  1. 扩展 `packages/sdk-ts` 的 `ops / search / recommendation` domain，补齐本批读取和写入方法、幂等/step-up header options 与 Vitest 单测。
  2. 新增控制台 ops 视图模型、Zod/RHF schema、权限判断、错误格式化、幂等键生成、列表归一化和单元测试。
  3. 替换 `/ops/consistency`、`/ops/consistency/outbox`、`/ops/search` 为正式业务组件，接入 TanStack Query/Table/Virtual、权限态、空态、错态、加载态、高风险 step-up 与审计提示。
  4. 更新控制台路由元数据、Playwright smoke 和 TODO/预留清单，执行前端、SDK、后端、OpenAPI、真实 curl、浏览器 smoke 和数据库回查验证。

### BATCH-289（待审批）
- 任务：`WEB-015` 实现 ops 页面：outbox、dead letter、一致性联查、搜索同步、推荐重建、观测总览入口
- 状态：待审批
- 当前任务编号：`WEB-015`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-014` 已完成并作为本批基线。本批继续保持 `console-web -> /api/platform -> platform-core` 边界，没有新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO`。
- 完成情况：
  - 扩展 `packages/sdk-ts` 的 `ops / search / recommendation` domain：新增 outbox / dead-letter / consistency dry-run、search sync / reindex / alias / cache / ranking、recommendation placement / ranking / rebuild 等正式方法，写方法统一支持 `X-Idempotency-Key` 与必要 step-up header，并补齐 Vitest 覆盖。
  - 新增 `apps/console-web/src/lib/ops-workbench.ts` 与单测，集中落地 `WEB-015` 的表单 Zod schema、权限判断、正式 payload / query 生成、幂等键生成、错误格式化和状态色映射。
  - 将 `/ops/consistency` 升级为正式一致性联查页，真实展示当前主体、角色、租户/作用域、业务主状态、链证明、外部事实、投影差异、recent outbox、recent dead letter 与审计 trace；dry-run 修复表单展示并透传幂等键、step-up 和审计留痕提示。
  - 将 `/ops/consistency/outbox` 升级为 Outbox / Dead Letter / 观测总览工作台，接入 `GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`、`POST /api/v1/ops/dead-letters/{id}/reprocess`、`GET /api/v1/ops/observability/overview`，列表使用 TanStack Table + Virtual，覆盖筛选、排序、空态、错态、加载态、权限态和 dry-run 重处理。
  - 将 `/ops/search` 升级为搜索同步与推荐重建运维页，接入搜索同步任务、搜索排序配置、推荐位、推荐排序配置、Reindex、Alias、缓存失效、推荐重建入口；写操作统一使用 React Hook Form + Zod、重复提交防护、幂等键和高风险 step-up 提示。
  - 更新控制台路由 API 绑定与 Playwright smoke，使 `WEB-015` 页面纳入控制台 E2E；补充 eslint ignore，避免 Playwright 产物目录污染 lint。
- 验证与证据：
  1. `pnpm install` 通过，lockfile 无漂移。
  2. `pnpm --filter @datab/sdk-ts typecheck`、`pnpm --filter @datab/sdk-ts test` 通过（10 个测试文件、32 个测试）。
  3. `pnpm --filter @datab/console-web lint`、`typecheck`、`test:unit`、`test:e2e`、`build` 通过；根级 `pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过。
  4. `./scripts/verify-local-stack.sh core` 通过；`./scripts/seed-local-iam-test-identities.sh` 完成本地 IAM 主体准备。
  5. 真实运行 `platform-core` 后，`curl` 验证 `GET /api/v1/auth/me`、`GET /api/v1/ops/consistency/order/0b0c5dce-3fca-420e-b416-2433a1552e3e`、`GET /api/v1/ops/outbox`、`GET /api/v1/ops/dead-letters`、`GET /api/v1/ops/observability/overview` 均返回正式数据；缺 step-up 的 consistency / dead-letter 写入口返回 `AUD_EVIDENCE_INVALID`。
  6. 真实 Keycloak password grant 获取 `local-platform-admin / platform_admin` Bearer，验证 `GET /api/v1/ops/search/sync`、`GET /api/v1/ops/search/ranking-profiles`、`GET /api/v1/ops/recommendation/placements`、`GET /api/v1/ops/recommendation/ranking-profiles` 均返回正式数据；缺 step-up 的 reindex / rebuild 分别返回 `SEARCH_QUERY_INVALID` 与 `RECOMMENDATION_REBUILD_INVALID`。
  7. 插入最小 verified step-up 后，真实执行 `POST /api/v1/ops/consistency/reconcile` dry-run、`POST /api/v1/ops/dead-letters/{id}/reprocess` dry-run、`POST /api/v1/ops/search/cache/invalidate`、`POST /api/v1/ops/search/reindex` 与 `POST /api/v1/ops/recommendation/rebuild`；数据库回查命中 `audit.audit_event`、`audit.access_audit`、`ops.system_log`，搜索 reindex 临时 `search.index_sync_task` 已按 target_index 清理，审计记录按 append-only 保留。
  8. 浏览器 smoke 通过：桌面与窄屏访问 `/ops/consistency`、`/ops/consistency/outbox`、`/ops/search`，捕获到 12 次 `/api/platform/**` 代理调用，`directForbidden=[]`，确认前端没有直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`。
  9. 后端通用验证通过：`cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`（358 passed）、`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh`、`./scripts/check-openapi-schema.sh`。
- 遗留与后续：
  - 无。`WEB-015` 要求的 ops 页面、outbox / dead-letter / consistency / search / recommendation / observability 正式接入、权限态/空态/错态/加载态、SDK 契约绑定、幂等键、step-up、审计留痕、真实 API、E2E / 浏览器 smoke、数据库回查和受限系统边界验证均已完成；开发者页面完整业务细节、通知联查继续由后续 WEB task 承接。
- TODO / 预留同步：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-289` 无新增项。

### BATCH-281（计划中）
- 任务：`WEB-007` 实现卖方上架中心：商品草稿、SKU 编辑、元信息、质量报告、模板绑定、提交审核
- 状态：计划中
- 说明：本批将 `/seller/products` 与相关卖方商品配置入口从脚手架升级为正式上架中心。页面必须展示商品草稿/状态列表、商品编辑表单、SKU 真值配置、元信息十大域、质量报告摘要、模板绑定和提交审核动作；所有写操作必须携带 `X-Idempotency-Key`，展示主体/角色/租户/作用域与审计留痕提示，并只通过 `portal-web -> /api/platform -> platform-core` 调用正式 API。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-006` 已本地提交，门户会话、受控 `/api/platform/**` 代理、SDK、商品详情与卖方主页可复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-007` 只实现卖方上架中心，不提前合并审核工作台或下单页；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认上架中心需覆盖商品列表、状态筛选、统计、新建商品、审核状态跟踪；产品编辑页需覆盖基本信息、原始接入摘要、样本、Hash、标签分类、十大元信息域、字段结构、质量报告、数据契约、交付方式、合规字段、保存草稿与提交审核；SKU 配置页必须显式支持 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认商品创建流程为资产/版本 -> 标准 SKU 真值 -> 模板绑定 -> 元信息档案 -> 字段结构与质量报告 -> 数据契约草稿 -> 保存草稿 -> 提交审核 -> 合规/风控 -> 上架与链摘要。
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`：确认 `product` 是目录/审核/search/detail 聚合事实源，`sku` 是下单/合同/授权/交付/验收/账单/结算事实源；`sku_type` 仅允许八个标准 SKU，`sku_code` 只表示商品内商业套餐编码，`trade_mode` 不得替代 `sku_type`。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`、`后端鉴权中间件规则说明.md`：确认路由 `/seller/products`、`/seller/products/:productId/edit`、`/skus`、`/templates` 的权限点分别为 `catalog.product.list/create/read/update/submit`、`catalog.metadata.edit`、`catalog.quality_report.manage`、`catalog.sku.create/update`、`template.contract.bind`；中风险动作需显式审计提示。
  - 通用边界文档与全集成基线：确认前端不得直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`，商品主状态由 `platform-core + PostgreSQL` 承担，搜索/推荐只作为读模型和候选召回，所有写操作必须经正式 API 并保留幂等与审计线索。
  - `packages/openapi/catalog.yaml` 与 `docs/02-openapi/catalog.yaml`：确认创建、编辑、SKU、元信息、质量报告、模板绑定、提交审核接口已存在；同时发现当前缺少 `GET /api/v1/products` 上架中心列表契约，且 catalog 写接口 OpenAPI 响应仍有部分直接声明业务对象而后端实际返回 `ApiResponse<T>`，需要本批同步收敛。
  - `apps/platform-core/src/modules/catalog/**`：确认后端已有创建/编辑/SKU/元信息/质量报告/模板绑定/提交审核处理器与审计/outbox 写入；缺少卖方上架中心列表路由，需要补 `catalog.product.list` 读取面。
  - `apps/portal-web/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认卖方商品中心、编辑、SKU、模板、元信息路由仍为 `PortalRoutePage` 脚手架；`sdk-ts` 目前 catalog domain 只封装了标准场景、商品详情和卖方主页读取，需要补正式写操作封装。
- 当前完成标准理解：
  - 上架中心必须真实读取卖方商品列表，能创建草稿、编辑草稿、创建/更新 SKU、保存元信息、保存质量报告、绑定模板、提交审核。
  - 页面必须显式展示 8 个标准 SKU，并校验 `sku_type` 与 `trade_mode / billing_mode / acceptance_mode / refund_mode` 的组合，不得把 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 并回大类。
  - 所有写操作必须自动生成并展示 `X-Idempotency-Key`，重复点击期间禁用提交，错误回显以后端统一错误码为准。
  - 敏感供方页面必须展示当前主体、角色、租户/组织、作用域和审计留痕提示。
  - 页面可使用当前 `preview=loading/empty/error/forbidden` 预演状态，但正式 ready 态必须调用真实 `platform-core` API，不得用 mock 当完成证据。
- 实施计划：
  1. 补齐 `GET /api/v1/products` 后端列表路由、OpenAPI/归档、SDK 生成物和 `sdk-ts` catalog domain 写操作封装，并增加最小防漂移校验。
  2. 新增上架中心视图模型、Zod/RHF 表单 schema、SKU 真值映射与单元测试，统一生成幂等键和错误文案。
  3. 替换 `/seller/products`、`/seller/products/:productId/edit`、`/skus`、`/templates`、`/metadata-contracts` 脚手架为正式上架中心/工作区组件，接入列表、详情、创建、编辑、SKU、元信息、质量报告、模板绑定和提交审核。
  4. 更新 E2E、路由 API 绑定、OpenAPI checker，执行前端、后端、SDK、真实 API、浏览器 smoke、数据库回查与受限系统边界验证。

### BATCH-281（待审批）
- 任务：`WEB-007` 实现卖方上架中心：商品草稿、SKU 编辑、元信息、质量报告、模板绑定、提交审核
- 状态：待审批
- 当前任务编号：`WEB-007`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-006` 已本地提交并提供门户会话、受控代理、SDK、商品详情和卖方主页基线。本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，没有新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 完成情况：
  - 后端补齐 `GET /api/v1/products` 上架中心列表路由、`ProductListView / ProductStatusCountView`、卖方/平台范围过滤、状态/关键词/分页、状态统计与 `catalog.product.list` 审计；修复 live smoke 暴露的全量平台列表审计 `ref_id="all"` 非 UUID 问题，改用合法 nil UUID。
  - 收敛提交审核前模板完整性检查：正式模板种子使用 `20000000-...` 演示 UUID，原正则误要求 UUID version `1-5` 导致已绑定模板被判无效；本批放宽为 PostgreSQL 可接受 UUID 字面量，保证 `V1 Standard Contract Template` 可用于正式提交。
  - `packages/openapi/catalog.yaml` 与 `docs/02-openapi/catalog.yaml` 同步新增 `listProducts`、`ApiResponseProductList`、`ProductSubmit` 等包装 schema，并把上架中心写接口统一声明 `X-Idempotency-Key`；`scripts/check-openapi-schema.sh` 增加 catalog 上架中心防漂移校验。
  - `packages/sdk-ts` 新增 catalog domain 写操作：创建/编辑商品、SKU 创建/编辑、元信息保存、质量报告创建、产品/ SKU 模板绑定、提交审核；所有 mutation 通过 SDK 透传 `X-Idempotency-Key`，并新增回归测试验证请求头。
  - 将 `/seller/products`、`/seller/products/:productId/edit`、`/skus`、`/templates`、`/metadata-contracts` 从脚手架替换为正式 `SellerProductWorkspaceShell`：列表、状态筛选、关键词、真实商品详情联动、创建草稿、保存草稿、SKU 真值配置、十大元信息域、质量报告、模板绑定和提交审核都走正式 API。
  - 页面显式展示当前主体、角色、租户/组织、作用域、审计留痕提示和写操作幂等键；显式列出 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU，未把 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 并回大类。
  - 补齐权限态、空态、错态、加载态与移动端响应式；live 浏览器 smoke 首次发现移动横向溢出后，将商品列表表格在移动端降级为卡片式行布局，重跑桌面/移动 smoke 通过。
- 验证：
  - 前端 / SDK：
    - `pnpm install`
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
  - 后端 / 契约：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./scripts/check-openapi-schema.sh`
  - 真实 API / DB / 浏览器 smoke：
    - 启动 `platform-core`：`APP_MODE=local PROVIDER_MODE=mock APP_HOST=127.0.0.1 APP_PORT=8094 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo run -p platform-core-bin`
    - `curl GET /healthz`
    - `curl GET /api/v1/products?page=1&page_size=3` 验证列表与状态统计
    - `curl GET /api/v1/products?status=not_a_status` 验证统一错误码 `CAT_VALIDATION_FAILED`
    - 真实写链路：`POST /api/v1/products`、`PATCH /api/v1/products/{id}`、`POST /api/v1/products/{id}/skus`、`PUT /api/v1/products/{id}/metadata-profile`、`POST /api/v1/assets/{versionId}/quality-reports`、`POST /api/v1/skus/{id}/bind-template`、`POST /api/v1/products/{id}/submit`，全部携带 `X-Idempotency-Key`
    - 数据库回查：`catalog.product / product_sku / product_metadata_profile / asset_quality_report / contract.template_binding / review.review_task / ops.outbox_event / audit.audit_event`
    - 启动 `portal-web` dev server，注入 Bearer claims Cookie，桌面 `1440x1100` 与移动 `390x844` 打开 `/seller/products`，校验主体/角色/租户/作用域、8 SKU、`X-Idempotency-Key`、真实商品、无直连 `127.0.0.1:8094`
    - 清理两组临时业务测试商品、SKU、模板绑定、质量报告、review task 与 outbox；审计事件按 append-only 保留
- 验证结果：
  - 前端、SDK、workspace lint/typecheck/test/build 全部通过；`pnpm test` 中 portal / console 的 Playwright WebServer 在未启动 `platform-core` 时仍会打印 `ECONNREFUSED 127.0.0.1:8094` 的受控代理噪音，但测试断言通过，live smoke 已单独覆盖真实联调。
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 与 `./scripts/check-openapi-schema.sh` 全部通过；`cargo check/test/sqlx` 仍有既存 unused warning，不影响本批结果。
  - live API 创建并提交商品 `WEB-007-SMOKE-XIDEM-*` 后回查得到：`product.status=pending_review`、SKU=1、元信息=1、模板绑定=1、review_task=1、质量报告=1；outbox 中 `search.product.changed / catalog.product.submitted` 均记录了对应 `X-Idempotency-Key`；审计事件覆盖 `catalog.product.create / patch / sku.create / product_metadata_profile.upsert / asset_quality_report.create / template.sku.bind / catalog.product.submit / catalog.product.list`。
  - 浏览器 smoke 通过：桌面和移动均可加载 `/seller/products`，显示 `WEB-007 Seller Operator / seller_operator / {seller_org_id} / aal1`、八个标准 SKU、`X-Idempotency-Key` 与真实商品；捕获浏览器请求 `platformProxyRequestCount=1`、`directPlatformCoreRequestCount=0`，桌面/移动均无横向溢出。
  - 静态边界检查通过：`apps/portal-web`、`apps/console-web`、`packages/sdk-ts` 未发现 `pg / postgres / kafkajs / opensearch / redis / fabric-network` 等前端直连受限系统依赖或调用。
  - 业务测试数据清理回查：`web007_products=0`、`web007_quality=0`、`web007_outbox=0`；`web007_audit=16` 按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-007`
  - `页面说明书-V1-完整版.md`：卖方上架中心、产品编辑页、SKU 配置页、元信息与质量报告展示、提交审核动作
  - `目录与商品接口协议正式版.md`：商品、SKU、元信息、质量报告、模板绑定、提交审核与 `X-Idempotency-Key`
  - `业务流程图-V1-完整版.md`：资产/版本 -> 商品草稿 -> SKU -> 模板 -> 元信息/质量 -> 提交审核
  - `菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`
  - `packages/openapi/catalog.yaml`、`docs/02-openapi/catalog.yaml`、`packages/sdk-ts/**`
- 覆盖的任务清单条目：`WEB-007`
- 未覆盖项：
  - 无。`WEB-007` 要求的卖方上架中心读取、写入、幂等、权限态、状态态、八个标准 SKU、SDK/OpenAPI 契约、真实 API 联调、浏览器 smoke、数据库回查与清理均已完成；审核工作台完整处理流由后续 `WEB-008` 展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-282（计划中）
- 任务：`WEB-008` 实现审核工作台：主体审核、商品审核、合规审核列表与详情
- 状态：计划中
- 说明：本批将 `console-web` 的 `/ops/review/subjects`、`/ops/review/products`、`/ops/review/compliance` 从路由脚手架升级为正式审核工作台。页面必须真实读取待审主体和待审商品，展示主体/商品/合规详情，支持审核通过、驳回和高风险合规阻断确认；所有写操作必须携带 `X-Idempotency-Key`，敏感页面必须展示当前主体、角色、租户、作用域和审计留痕提示，并只通过 `console-web -> /api/platform -> platform-core` 调用正式 API。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-007` 已本地提交，门户/控制台工程、SDK、会话上下文、受控代理、商品列表/详情和提交审核链路可复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-008` 只实现审核工作台，不提前合并下单页、订单页、交付页、账单页或审计联查页；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认主体审核台需覆盖组织准入、主体资料和身份绑定状态；产品审核台需覆盖商品上架内容、Hash、模板和元数据完整性；合规审核台需覆盖分类分级、使用目的、地域、导出限制、自动阻断结果、人工复核结论和高风险标签。
  - `docs/权限设计/菜单权限映射表.md`、`docs/权限设计/按钮级权限说明.md`、`docs/权限设计/接口权限校验清单.md`：确认路由权限为 `review.subject.read / review.product.read / review.compliance.read`，审核动作权限为 `review.subject.review / review.product.review / review.compliance.review`，合规阻断为高风险动作 `review.compliance.block`，写接口必须校验幂等、生成 request_id 并写审计。
  - `docs/业务流程/业务流程图-V1-完整版.md` 与全集成基线：确认主体状态、商品状态和合规风控状态由 `platform-core + PostgreSQL` 承担，搜索/推荐/缓存/链上投影仅作为受控后端能力，前端不得直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`。
  - `docs/数据库设计/接口协议/身份与会话接口协议正式版.md`、`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`、`packages/openapi/iam.yaml`、`packages/openapi/catalog.yaml`、`docs/02-openapi/iam.yaml`、`docs/02-openapi/catalog.yaml`：确认审核动作已存在于 catalog API；同时发现 IAM 组织列表/详情后端和领域结构已存在但 OpenAPI 仍为骨架，本批按后端实现、接口协议和页面说明共同语义补齐两份 OpenAPI 并重新生成 SDK。
  - `apps/platform-core/src/modules/iam/**`、`apps/platform-core/src/modules/catalog/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认 IAM 已有 `GET /api/v1/iam/orgs` 和 `GET /api/v1/iam/orgs/{id}`，catalog 已有 `POST /api/v1/review/subjects/{id}`、`POST /api/v1/review/products/{id}`、`POST /api/v1/review/compliance/{id}`；控制台审核路由仍是 `ConsoleRoutePage` 脚手架，SDK domain 尚未封装审核写操作和 IAM 组织读取。
- 当前完成标准理解：
  - 三个审核页必须真实读取 `platform-core` API，列表、详情、分页/筛选、加载态、空态、错态、权限态和移动/桌面加载均可用。
  - 审核动作必须经 SDK 发送 `X-Idempotency-Key`，重复点击防护、后端错误码回显、审计事件提示和高风险合规阻断确认必须可见。
  - 敏感页面必须显示当前主体、角色、租户和作用域；无 `platform_reviewer / platform_admin` 等正式角色时必须拦截主按钮。
  - 商品审核和合规审核不得发明 SKU 大类、状态名或错误码；若展示 SKU，必须保留八个标准 SKU 真值。
  - 浏览器端只能请求 `/api/platform/**` 受控代理，不能直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric` 或 `platform-core` 端口。
- 实施计划：
  1. 补齐 IAM 组织列表/详情 OpenAPI 与 SDK，补齐 catalog 审核写操作 SDK、合规审核幂等头声明和最小防漂移校验。
  2. 修正后端审核读取/写入所需的正式角色与幂等校验，确保 `platform_reviewer` 可读取待审商品并执行审核动作。
  3. 新增审核工作台视图模型、Zod/RHF 决策表单、TanStack Query/Table/Virtual 列表、权限判断、错误码映射和单元测试。
  4. 替换三个 console 审核路由为正式工作台，接入主体/商品/合规详情、通过/驳回/阻断确认、审计提示、身份条和状态预演。
  5. 执行前端、SDK、后端、OpenAPI、真实 API、数据库回查、Playwright 与浏览器 smoke 验证，再更新 TODO、写“待审批”并本地提交。

### BATCH-282（待审批）
- 任务：`WEB-008` 实现审核工作台：主体审核、商品审核、合规审核列表与详情
- 状态：待审批
- 当前任务编号：`WEB-008`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-007` 已本地提交并提供门户/控制台工程、受控 `/api/platform` 代理、会话上下文、SDK 和商品读取/提交审核基础。
- 完成情况：
  - 将 `/ops/review/subjects`、`/ops/review/products`、`/ops/review/compliance` 从 `ConsoleRoutePage` 脚手架升级为正式审核工作台，接入主体审核、商品审核与合规审核的列表、详情、筛选、加载态、空态、错态、权限态和桌面/移动加载。
  - 新增 `ReviewWorkbenchShell`，使用 `TanStack Query` 读取正式 API、`TanStack Table + Virtual` 渲染长列表、`React Hook Form + Zod` 校验审核表单，并在所有写动作中生成和透传 `X-Idempotency-Key`。
  - 审核页显示当前主体、角色、租户/组织、作用域和审计留痕提示；无 `platform_reviewer / platform_admin` 时拦截主按钮；合规阻断要求输入 `BLOCK`，并支持透传 `X-Step-Up-Token` / `x-step-up-challenge-id`。
  - 商品/合规详情展示八个标准 SKU 真值覆盖，不把 `SHARE_RO / QRY_LITE / RPT_STD` 误并入大类；合规页展示分类分级、使用目的、地域、导出限制、风险标签和自动阻断结果。
  - 补齐 `packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml` 的 `GET /api/v1/iam/orgs`、`GET /api/v1/iam/orgs/{id}`、`OrganizationAggregateView` 与响应 wrapper；补齐 `packages/openapi/catalog.yaml` 与 `docs/02-openapi/catalog.yaml` 的审核响应 wrapper 和合规审核 `X-Idempotency-Key`。
  - 重新生成 `packages/sdk-ts`，新增 IAM 组织读取 domain、catalog 审核写 domain、审核写 header 透传和 SDK 单测；`scripts/check-openapi-schema.sh` 增加 IAM 组织与 catalog 审核防漂移检查。
  - 后端新增正式 `platform_reviewer` 角色种子和权限断言，允许审核员读取待审商品并执行审核；catalog 审核写入校验缺失/空 `X-Idempotency-Key` 并返回统一错误码 `CAT_VALIDATION_FAILED`。
  - 更新 console 路由 API 绑定、审核入口 E2E、登录态占位角色，移除旧 `platform_auditor` 口径并使用正式角色 `platform_reviewer / platform_audit_security`。
- 验证结果：
  - 依赖与生成：`pnpm install`、`pnpm --filter @datab/sdk-ts openapi:generate` 通过。
  - SDK：`pnpm --filter @datab/sdk-ts typecheck`、`pnpm --filter @datab/sdk-ts test` 通过。
  - console：`pnpm --filter @datab/console-web lint`、`typecheck`、`test:unit`、`test:e2e`、`build` 通过；E2E 覆盖审核路由预览与 product review API 绑定。
  - workspace：`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 通过；并保留 `pnpm test` 中 portal/console Playwright 在未启动 core 时预期出现的 `ECONNREFUSED` 噪声，测试结果仍为通过。
  - 后端：`cargo fmt --all`、`cargo fmt --all -- --check`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace`、`./scripts/check-query-compile.sh` 通过；既存 unused warning 仍存在，不属于本批新增。
  - OpenAPI：`./scripts/check-openapi-schema.sh` 通过；`packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml`、`packages/openapi/catalog.yaml` 与 `docs/02-openapi/catalog.yaml` 已同步。
  - 真实 API 联调：按 runbook 显式设置 `KAFKA_BROKERS=127.0.0.1:9094`、`KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094` 后启动 `platform-core` 于 `127.0.0.1:8096`，`GET /api/v1/auth/me` 返回 `local_test_user / web008.reviewer / platform_reviewer / aal1`。
  - curl 覆盖：`GET /api/v1/iam/orgs?status=pending_review`、`GET /api/v1/iam/orgs/{id}`、`GET /api/v1/products?status=pending_review`、`GET /api/v1/products/{id}` 均返回 WEB-008 临时数据；`buyer_operator` 读取主体审核队列返回 `IAM_UNAUTHORIZED`；缺少 `X-Idempotency-Key` 的审核写返回 `CAT_VALIDATION_FAILED` 且未产生 `review_task`。
  - 写操作联调：`POST /api/v1/review/subjects/{id}` approve、`POST /api/v1/review/compliance/{id}` reject、`POST /api/v1/review/products/{id}` approve 均通过；产品审核后 `product.status=listed`。
  - 数据库回查：`review.review_task` 覆盖 `subject_review=approved`、`compliance_review=rejected`、`product_review=approved`；`review.review_step` 写入三个动作；`audit.audit_event` 保留 `catalog.review.subject / compliance / product` 三条审计；`ops.outbox_event` 写入 `catalog.product.status.changed` 与 `search.product.changed`，均带 `idem-web008-product-approve`；搜索投影 `listing_status=listed`、`visibility_status=visible`、`sku_types={API_PPU,API_SUB,FILE_STD,FILE_SUB,QRY_LITE,RPT_STD,SBX_STD,SHARE_RO}`。
  - 浏览器 smoke：`console-web` 以 `PLATFORM_CORE_BASE_URL=http://127.0.0.1:8096` 启动于 `127.0.0.1:3112`，Playwright 访问三条审核路由和移动端主体页，捕获 `proxyRequestCount=12`、`directPlatformRequestCount=0`，确认浏览器只访问 `/api/platform/**`。
  - 受限系统边界：运行时依赖与 import 检查未发现 `pg / postgres / kafkajs / node-rdkafka / @opensearch-project/opensearch / ioredis / redis / fabric-network / fabric-ca-client` 等前端直连依赖；宽泛文本检查仅命中 SDK 生成文件中的 ops/search 文档注释。
  - 业务测试数据清理：`core.organization / core.user_account / catalog.product / catalog.product_sku / catalog.asset_version / catalog.data_asset / review.review_task / review.review_step / ops.outbox_event` 的 WEB-008 临时业务数据均已清理为 `0`；`audit.audit_event` 三条审计按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-008`
  - `页面说明书-V1-完整版.md`：审核工作台、主体审核台、产品审核台、合规审核台、敏感页面身份上下文展示
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：审核页查看、主按钮权限、高风险合规阻断提示
  - `业务流程图-V1-完整版.md`：主体准入、商品上架审核、合规复核链路
  - `身份与会话接口协议正式版.md`、`目录与商品接口协议正式版.md`、`packages/openapi/iam.yaml`、`packages/openapi/catalog.yaml`、`docs/02-openapi/iam.yaml`、`docs/02-openapi/catalog.yaml`
- 覆盖的任务清单条目：`WEB-008`
- 未覆盖项：
  - 无。`WEB-008` 要求的主体/商品/合规审核列表与详情、权限态、错态、空态、加载态、幂等头、SDK/OpenAPI 契约、真实 API 联调、浏览器 smoke、数据库回查与清理均已完成；下单页、订单页、交付页、账单页和审计联查页按后续 WEB task 顺序继续推进。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 无需新增条目。

### BATCH-284（计划中）
- 任务：`WEB-010` 实现交付中心页面：文件交付、API 开通、共享开通、模板授权、沙箱开通、报告交付入口
- 状态：计划中
- 说明：本批将 `/delivery/orders/:orderId/file|api|share|subscription|template-query|sandbox|report` 从脚手架升级为正式交付中心工作区。页面必须真实读取订单、生命周期与当前主体，按 SKU 显示正确交付入口，并对文件/报告交付、API 开通、共享开通、版本订阅、模板授权、沙箱开通提供表单、状态、权限、幂等键、审计提示、空态/错态/加载态和最小 E2E。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-009` 已本地提交并提供门户工程、会话、受控代理、SDK、商品详情、下单与订单详情基线。本批继续保持 `portal-web -> /api/platform -> platform-core` 边界，不新增浏览器直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric`。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-010` 只实现交付中心入口，不提前合并验收页、账单页、争议页或审计联查页；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认文件交付页需展示对象、密钥信封、下载票据、回执与 Hash 校验；API 开通页需展示应用绑定、API Key、额度、限流与调用日志；只读共享页需展示授权协议、接收方、共享对象、授权范围、有效期和撤权记录；模板查询、沙箱和报告交付入口需展示授权、工作区、结果包与边界说明。
  - `docs/业务流程/业务流程图-V1-完整版.md`、`docs/05-test-cases/delivery-cases.md`：确认 FILE/REPORT/API/SHARE/QRY_LITE/SBX_STD 各交付分支、票据过期、重复开通幂等、授权撤销、下载不得暴露真实对象路径、审计与链路摘要要求。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认交付页查看权限为 `trade.order.read`，主按钮权限包括 `delivery.file.commit/download`、`delivery.api.enable`、`delivery.share.enable/read`、`delivery.subscription.manage/read`、`delivery.template_query.enable`、`delivery.sandbox.enable`、`delivery.report.commit`。
  - 通用边界文档、OpenAPI 冻结表、统一错误码字典、测试矩阵、本地环境与配置项文档、技术选型、总体架构与全集成基线：确认前端只调用 `platform-core` 正式 API；写接口必须携带 `X-Idempotency-Key`；敏感页面必须展示主体、角色、租户/组织、作用域；不得发明 SKU、状态或错误码语义。
  - `packages/openapi/delivery.yaml` 与 `docs/02-openapi/delivery.yaml`：确认交付分支接口、下载票据、订阅、共享、模板授权、沙箱、查询运行与 API usage log schema；同时发现 `deliver/share-grants/template-grants/sandbox-workspaces` 后端已读取 `x-idempotency-key`，OpenAPI 尚未声明，需要本批同步补齐并加防漂移校验。
  - `apps/platform-core/src/modules/delivery/**`、`apps/platform-core/src/modules/order/**`、`packages/sdk-ts/**`、`apps/portal-web/**`：确认后端已有交付分支处理器、订单详情与生命周期 API；`sdk-ts` 尚无 delivery domain；门户交付路由仍为 `PortalRoutePage` 脚手架。
- 当前完成标准理解：
  - 交付中心必须按真实订单 SKU 展示官方交付入口，并显式支持 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU，不得把 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 并回大类。
  - 页面必须读取 `auth/me`、订单详情、生命周期快照以及分支 API，并展示主体/角色/租户/作用域、状态、request_id / tx_hash / 链状态 / 投影状态承接、审计留痕与受控对象路径说明。
  - 写操作必须通过 SDK 透传 `X-Idempotency-Key`，重复点击期间禁用提交，前端 Zod/RHF 校验与后端错误码回显必须可用。
  - 下载类页面不得展示真实对象路径；下载票据只显示 ticket、状态、有效期、下载次数和 Hash 校验摘要。
- 实施计划：
  1. 补齐 delivery OpenAPI 幂等头声明、防漂移校验和 `packages/sdk-ts` delivery domain，并重新生成类型。
  2. 新增交付中心视图模型、Zod/RHF schema、SKU 到交付入口映射、错误格式化与单元测试。
  3. 替换文件、API、共享、订阅、模板查询、沙箱和报告交付路由为正式 `DeliveryWorkflowShell`，接入真实 API、权限态、状态态、幂等键和审计提示。
  4. 更新路由 API 绑定、E2E 与 smoke 覆盖，执行前端、SDK、后端、OpenAPI、真实 API、浏览器和数据库回查验证。

### BATCH-284（待审批）
- 任务：`WEB-010` 实现交付中心页面：文件交付、API 开通、共享开通、模板授权、沙箱开通、报告交付入口
- 状态：待审批
- 当前任务编号：`WEB-010`
- 完成情况：
  - 已补齐 `packages/openapi/delivery.yaml` 与 `docs/02-openapi/delivery.yaml` 的交付写接口幂等头声明，覆盖 `deliver / subscriptions / share-grants / template-grants / sandbox-workspaces`，并在 `scripts/check-openapi-schema.sh` 增加 Delivery 契约与归档同步防漂移校验。
  - 已重新生成 `packages/sdk-ts/src/generated/delivery.ts`，新增 `packages/sdk-ts/src/domains/delivery.ts` 与单测，正式封装文件/报告/API 提交、下载票据、版本订阅、共享授权、模板授权、沙箱开通、查询运行和 API 用量读取；所有写方法必传并透传 `X-Idempotency-Key`。
  - 已新增 `apps/portal-web/src/lib/delivery-workflow.ts` 与单测，冻结 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 到交付入口的正式映射，补齐 Zod/RHF 表单 schema、幂等键、权限判断、响应解包、敏感字段遮蔽和统一错误文案。
  - 已将 `/delivery/orders/:orderId/file|api|share|subscription|template-query|sandbox|report` 从脚手架升级为正式交付中心页，真实读取 `auth/me`、订单详情、生命周期快照和分支 API，展示主体/角色/租户/作用域、订单状态、SKU 真值、标准 SKU 入口、request_id / tx_hash / 链状态 / 投影状态承接、审计提示、加载态、空态、错态和权限态。
  - 文件与报告交付页不暴露真实对象路径；下载票据只显示受控 ticket、有效期、下载次数和 Hash 摘要；API Key、共享 locator、对象 URI 等敏感字段在结果区遮蔽展示。
  - 已更新门户路由 API 绑定、Playwright smoke 覆盖和 ESLint 输出目录忽略；同时修复 `home-shell` 在 Next build 中对 recommendation 生成类型的推导问题，避免 `RecommendationsResponse` 漂移导致构建失败。
- 验证记录：
  1. `pnpm install`
  2. `pnpm --filter @datab/sdk-ts openapi:generate`
  3. `pnpm --filter @datab/sdk-ts typecheck`
  4. `pnpm --filter @datab/sdk-ts test`
  5. `pnpm --filter @datab/sdk-ts build`
  6. `pnpm --filter @datab/portal-web typecheck`
  7. `pnpm --filter @datab/portal-web lint`
  8. `pnpm --filter @datab/portal-web test:unit`
  9. `pnpm --filter @datab/portal-web test:e2e`
  10. `pnpm --filter @datab/portal-web build`
  11. `pnpm lint`
  12. `pnpm typecheck`
  13. `pnpm test`
  14. `pnpm build`
  15. `cargo fmt --all`
  16. `cargo check -p platform-core`
  17. `cargo test -p platform-core`
  18. `cargo sqlx prepare --workspace`
  19. `./scripts/check-query-compile.sh`
  20. `./scripts/check-openapi-schema.sh`
  21. `docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml ps`：PostgreSQL、Kafka、Redis、OpenSearch、Keycloak、MinIO 等本地依赖可用；前端未直连这些系统。
  22. 真实 API 联调：`platform-core APP_MODE=local APP_PORT=8094`，`curl /api/v1/auth/me`、`GET /api/v1/orders/{id}`、`GET /api/v1/orders/{id}/lifecycle-snapshots`、`POST /api/v1/orders/{id}/subscriptions`、`GET /api/v1/orders/{id}/subscriptions` 均通过。
  23. 数据库回查：`delivery.revision_subscription` 记录 `last_idempotency_key`，`audit.audit_event` 记录 `delivery.subscription.manage` 与幂等键；清理临时业务数据后 `trade.order_main / delivery.revision_subscription / catalog.product / core.organization` 回查为 `0`，审计记录按 append-only 保留 `3` 条。
  24. 浏览器 smoke：桌面 `1440x960` 与移动 `390x844` 均加载交付订阅页；浏览器仅访问 `/api/platform/**`，`auth/me`、订单详情与订阅 POST 均被捕获，POST 带 `X-Idempotency-Key`，直连 `5432/6379/9092/9094/9200/7051/8094` 次数为 `0`。
  25. 边界检查：`rg` 未发现前端或 SDK 引入受限系统客户端；命中项仅为文档/生成契约描述中的边界说明。`git diff --check` 通过。
  26. 最终补丁后复验：`pnpm --filter @datab/portal-web typecheck`、`pnpm --filter @datab/portal-web lint`、`./scripts/check-openapi-schema.sh` 均通过。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-010`
  - `页面说明书-V1-完整版.md`：文件交付、API 开通、共享开通、版本订阅、模板查询、沙箱开通和报告交付入口
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：`trade.order.read` 与交付分支主按钮权限
  - `业务流程图-V1-完整版.md`、`delivery-cases.md`：交付分支、票据、授权、幂等、审计与撤权/空态/错态要求
  - `packages/openapi/delivery.yaml`、`docs/02-openapi/delivery.yaml`、`packages/sdk-ts/**`、`apps/platform-core/src/modules/delivery/**`
- 覆盖的任务清单条目：`WEB-010`
- 未覆盖项：
  - 无。验收页、账单页、争议页、审计联查页和通知联查页按后续 WEB task 顺序推进；本批未合并或提前完成后续任务。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；已在 `docs/开发任务/V1-Core-TODO与预留清单.md` 补记 `BATCH-284` 无新增项。

### BATCH-283（计划中）
- 任务：`WEB-009` 实现订单创建与订单详情页，显式支持 8 个标准 SKU 与五条标准链路官方下单入口
- 状态：计划中
- 说明：本批将门户 `/trade/orders/new` 与 `/trade/orders/:orderId` 从路由脚手架升级为正式订单创建与订单详情页。下单页必须真实读取商品详情、标准订单模板和当前会话主体，按五条官方标准链路展示 `场景名 -> 主 SKU / 补充 SKU / 合同 / 验收 / 退款模板` 映射，并在创建订单时通过 SDK 调用 `POST /api/v1/orders`、携带 `X-Idempotency-Key`、回显 `order_id / buyer_deposit / price_snapshot`。订单详情页必须真实读取 `GET /api/v1/orders/{id}` 与 `/lifecycle-snapshots`，展示订单主状态、分层状态、交付/验收/账单/争议摘要、审计联查入口、SKU 场景快照、权限态、空态、错态和加载态。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-008` 已本地提交并提供门户工程、受控 `/api/platform` 代理、会话主体条、商品详情、卖方主页、上架中心、审核工作台与 `sdk-ts` OpenAPI 生成基线。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-009` 只实现订单创建与订单详情页，不提前合并交付中心、账单中心、争议页或审计联查页；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认询单/下单页需覆盖商品快照、权利与用途确认、数量/期限/订阅配置、价格与保证金估算、风险提示和下单按钮；订单详情页需覆盖基本信息、状态时间线、交付摘要、验收摘要、账单摘要、争议摘要和审计联查入口。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认五条标准链路官方命名和模板映射，必须覆盖 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`，且 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 必须按独立 SKU 理解。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认买方搜索、选购与下单流程需要冻结 `price_snapshot`、计算 `buyer_deposit`、记录订单创建审计，并将订单详情作为后续支付锁定、交付准备和验收入口。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`按钮级权限说明.md`、`接口权限校验清单.md`：确认 `/trade/orders/new` 查看权限 `catalog.product.read`、主按钮 `trade.order.create`；订单详情查看权限 `trade.order.read`、主按钮 `trade.order.cancel`；创建订单和取消订单均需审计提示，普通写动作仍必须携带幂等键。
  - `docs/开发准备/服务清单与服务边界正式版.md`、`接口清单与OpenAPI-Schema冻结表.md`、`统一错误码字典正式版.md`、`测试用例矩阵正式版.md`：确认前端只能经 `platform-core` 正式 API 接入订单、商品、支付、交付、审计等能力，错误码必须沿用 `ORDER_CREATE_FORBIDDEN / ORDER_STATE_INVALID / TRD_STATE_CONFLICT` 等冻结语义。
  - `docs/05-test-cases/order-state-machine.md`、`delivery-cases.md`、`payment-billing-cases.md`、`audit-consistency-cases.md`：确认订单详情必须尊重 8 个标准 SKU 的状态机分支，并把交付、验收、账单、争议和审计联查作为后续页面入口，不在本批伪造下游状态。
  - `packages/openapi/trade.yaml` 与 `docs/02-openapi/trade.yaml`：确认 `listStandardOrderTemplates`、`createOrder`、`getOrderDetail`、`getOrderLifecycleSnapshots`、`cancelOrder` 契约已存在；同时发现订单写接口 OpenAPI 未声明 `X-Idempotency-Key`，本批需要补齐两份 OpenAPI 并重新生成 SDK。
  - `apps/platform-core/src/modules/order/**`、`catalog/**`、`iam/**`：确认订单创建会冻结 `OrderPriceSnapshot` 与 `ScenarioSkuSnapshot`，`API_SUB / RPT_STD` 等多场景 SKU 需要显式 `scenario_code` 消歧，订单详情和生命周期快照已经提供正式读取接口。
  - `apps/portal-web/**`、`apps/console-web/**`、`packages/sdk-ts/**`：确认订单创建和详情路由当前仍为 `PortalRoutePage` 脚手架，`sdk-ts` 的 trade domain 只封装了标准模板读取，需要补 create/detail/lifecycle/cancel 方法、幂等头和单测。
- 当前完成标准理解：
  - 下单页必须真实绑定 `platform-core` 商品详情、标准订单模板与订单创建 API；无 `productId` 时展示五条官方链路入口，有 `productId` 时按商品 SKU 与场景模板生成可提交订单草案。
  - 页面必须显式展示 8 个标准 SKU 和 5 条标准链路，不得重新分类、改名或把 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 并回大类。
  - 所有写动作必须自动生成、发送并展示 `X-Idempotency-Key`，重复点击期间禁用提交，后端统一错误码和 `request_id` 必须可见。
  - 敏感交易页面必须展示当前主体、角色、租户/组织、作用域和审计留痕提示；无 `buyer_operator / tenant_admin` 等正式角色时拦截创建订单主按钮。
  - 订单详情必须使用正式订单详情和生命周期快照字段展示主状态、支付、交付、验收、结算、争议、授权/合同/交付摘要和审计联查入口；链/投影字段未由当前接口返回时必须显式标注未返回，不能伪造 `tx_hash` 或对象路径。
- 实施计划：
  1. 补齐 trade OpenAPI 写接口 `X-Idempotency-Key` 声明、归档同步、SDK 生成物、trade domain 方法和单元测试，并扩展 OpenAPI drift checker。
  2. 新增订单工作流视图模型、五链路/八 SKU 映射、Zod/RHF 表单 schema、响应解包、权限判断、幂等键生成、错误码映射和单元测试。
  3. 替换 `/trade/orders/new` 与 `/trade/orders/:orderId` 为正式订单创建和详情组件，接入商品详情、标准模板、订单创建、订单详情、生命周期快照、取消动作提示、审计入口和状态预演。
  4. 更新门户路由 API 绑定和 Playwright 覆盖，执行前端、SDK、后端、OpenAPI、真实 API、数据库回查、桌面/移动浏览器 smoke 与受限系统边界验证。

### BATCH-283（待审批）
- 任务：`WEB-009` 实现订单创建与订单详情页，显式支持 8 个标准 SKU 与五条标准链路官方下单入口
- 状态：待审批
- 当前任务编号：`WEB-009`
- 完成情况：
  - 已补齐 `packages/openapi/trade.yaml` 与 `docs/02-openapi/trade.yaml` 的订单写接口幂等头契约：`POST /api/v1/orders` 与 `POST /api/v1/orders/{id}/cancel` 均声明必需 `X-Idempotency-Key`，并扩展 `scripts/check-openapi-schema.sh` 防止 trade OpenAPI 再次漂移。
  - 已重新生成 `packages/sdk-ts/src/generated/trade.ts`，并扩展 `packages/sdk-ts/src/domains/trade.ts`：新增 `createOrder`、`getOrderDetail`、`getOrderLifecycleSnapshots`、`cancelOrder`、幂等写选项和 SDK 单测。
  - 已新增 `apps/portal-web/src/lib/order-workflow.ts` 与单测：冻结五条官方标准链路映射，显式覆盖 8 个标准 SKU，保留 `SHARE_RO / QRY_LITE / SBX_STD / RPT_STD` 独立 SKU 语义，支持多场景 SKU 的 `scenario_code` 消歧、Zod/RHF 表单校验、幂等键生成、响应解包、错误码文案、交付路由和权限判断。
  - 已将 `/trade/orders/new` 升级为正式下单页：读取 `auth/me`、商品详情、标准订单模板，展示商品快照、五链路入口、8 SKU 覆盖、买方主体上下文、权利/用途确认、数量/期限/订阅配置、价格/保证金估算、幂等键、审计提示、权限态、空态、错态和加载态；三项确认默认未勾选并由 Zod 强制校验；提交时通过 SDK 调用 `POST /api/v1/orders` 并展示 `order_id / buyer_deposit / price_snapshot`。
  - 已将 `/trade/orders/:orderId` 升级为正式订单详情页：读取订单详情与生命周期快照，展示主状态、支付、交付、验收、结算、争议、合同/授权/交付/账单摘要、场景快照、审计联查入口、链/投影未返回说明、取消动作幂等键提示、权限态、空态、错态和加载态。
  - 已更新门户路由 API 绑定与 Playwright scaffold 覆盖，确保订单创建、详情、权限态、空态、错误态和受控 `/api/platform` 边界可验证。
- 验证记录：
  1. `pnpm install`
  2. `pnpm --filter @datab/sdk-ts openapi:generate`
  3. `pnpm --filter @datab/sdk-ts typecheck`
  4. `pnpm --filter @datab/sdk-ts test`
  5. `pnpm --filter @datab/sdk-ts build`
  6. `pnpm --filter @datab/portal-web typecheck`
  7. `pnpm --filter @datab/portal-web lint`
  8. `pnpm --filter @datab/portal-web test:unit`
  9. `pnpm --filter @datab/portal-web test:e2e`
  10. `pnpm --filter @datab/portal-web build`
  11. `pnpm lint`
  12. `pnpm typecheck`
  13. `pnpm test`
  14. `pnpm build`
  15. `cargo fmt --all`
  16. `cargo check -p platform-core`
  17. `cargo test -p platform-core`
  18. `cargo sqlx prepare --workspace`
  19. `./scripts/check-query-compile.sh`
  20. `./scripts/check-openapi-schema.sh`
  21. 启动 `APP_PORT=8099` 的 `platform-core`，插入临时买方/卖方/资产/版本/商品/`FILE_STD` SKU 数据后执行真实 API 联调：
      - `POST /api/v1/orders` 返回 `order_id=d55f9bfc-3fbd-4fee-8df2-e727aa388613`、`status=created`、`scenario_code=S2`、`selected_sku_role=primary`。
      - 使用同一 `X-Idempotency-Key=idem-web009-1776944713630633936` 重放订单创建，返回同一 `order_id`，数据库审计记录 `trade.order.create.idempotent_replay`。
      - `GET /api/v1/orders/{id}` 返回 `created / unpaid / pending_delivery / S2`。
      - `GET /api/v1/orders/{id}/lifecycle-snapshots` 返回 `created / unpaid / not_started`，当前无交付对象时保持空摘要。
  22. 数据库回查：
      - `trade.order_main`：`status=created`、`payment_status=unpaid`、`delivery_status=pending_delivery`、`acceptance_status=not_started`、`scenario_code=S2`、`selected_sku_role=primary`。
      - `audit.audit_event`：存在 `trade.order.create`、`trade.order.create.idempotent_replay`、`trade.order.read`、`trade.order.lifecycle_snapshots.read`。
      - `ops.outbox_event`：存在 1 条 `trade.order.created`，包含本次 `request_id` 与 `idempotency_key`。
  23. 浏览器真实 smoke：
      - `portal-web` 生产模式连接 `PLATFORM_CORE_BASE_URL=http://127.0.0.1:8099`。
      - 桌面端访问 `/trade/orders/new?productId=...&scenario=S2`，通过 Bearer claims 展示当前主体、角色、租户/组织、作用域，确认三项复选框默认未勾选，完成三项确认后真实提交订单并跳转订单详情。
      - 移动端访问同一订单详情，验证详情页、当前主体访问上下文、审计与链路信任边界可见。
      - 最终 smoke 输出：`WEB009_BROWSER_SMOKE_OK_FINAL order_id=96f99837-81ed-4f07-8401-5c6ec7bb8400 browser_requests=61 direct_core=0`。
      - 浏览器侧请求数 `61`，直接访问 `127.0.0.1:8099` 的请求数 `0`，确认前端未绕过 `/api/platform` 直连 `platform-core`。
  24. 受限系统边界检查：`rg` 检查 `apps/portal-web`、`apps/console-web`、`packages/sdk-ts` 中未发现前端直连 PostgreSQL / Kafka / OpenSearch / Redis / Fabric 的运行入口；命中的仅为 SDK 生成类型和 OPS/IAM API 字段说明。
  25. 业务测试数据清理：首轮 `trade.order_main` 5 条 WEB-009 临时订单与 `ops.outbox_event` 5 条 WEB-009 临时事件已清理；最终 smoke 追加的 `order_id=96f99837-81ed-4f07-8401-5c6ec7bb8400`、对应 `ops.outbox_event`、`catalog.product_sku / catalog.product / catalog.asset_version / catalog.data_asset / core.organization` 临时业务数据均已清理为 `0`；`audit.audit_event` 按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-009`
  - `页面说明书-V1-完整版.md`：6.1 询单/下单页、6.4 订单详情页、敏感页面身份上下文展示
  - `数据交易平台-全集成基线-V1.md`：5.3.2A 五条标准场景到 V1 SKU 与模板映射
  - `业务流程图-V1-完整版.md`：买方选购下单、价格快照冻结、审计记录、后续支付/交付/验收入口
  - `按钮级权限说明.md`、`接口权限校验清单.md`、`菜单权限映射表.md`：订单创建、订单详情、取消动作、审计提示和权限态
  - `packages/openapi/trade.yaml`、`docs/02-openapi/trade.yaml`、`packages/sdk-ts`
- 覆盖的任务清单条目：`WEB-009`
- 未覆盖项：
  - 无。交付中心、验收页、账单页、争议页和审计/ops/开发者/通知联查页按后续 WEB task 顺序继续推进；本批只提供正式入口和当前订单详情摘要，不伪造下游状态、链上 `tx_hash` 或对象路径。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；`docs/开发任务/V1-Core-TODO与预留清单.md` 无需新增条目。

### BATCH-280（计划中）
- 任务：`WEB-006` 实现卖方主页：主体信息、认证标识、商品列表、联系方式/咨询入口
- 状态：计划中
- 说明：在 `WEB-005` 商品详情页已经真实读取卖方 profile 的基础上，本批将 `/sellers/{orgId}` 从路由脚手架升级为正式卖方主页。页面必须真实读取 `GET /api/v1/sellers/{orgId}/profile`，并接入 `seller_profile_featured` 推荐位展示该卖方在售商品/服务与高质量推荐，同时展示主体信息、认证标识、行业标签、信誉/风险摘要、最近成交与争议摘要、联系方式/咨询入口、空态/错态/权限态和敏感访问上下文。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-005` 已本地提交，门户会话、受控 `/api/platform/**` 代理、SDK、搜索页与商品详情页可复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-006` 只实现卖方主页，不提前合并下单页、审核台或订单页任务；DoD 要求页面可访问、空态/错态/权限态可用、契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认卖方主页需展示主体信息卡、行业标签、信誉与风险摘要、在售商品/服务列表、最近成交与争议摘要；推荐区目标为该卖方热门商品、高质量商品和同类优质卖方。
  - `docs/业务流程/业务流程图-V1-完整版.md`：确认流程为搜索结果点击卖方主体，经 Search API 查询 seller_search_document，再由 PostgreSQL 校验组织状态、公开展示策略和风险状态，返回主体、标签、信誉/风险、在售数、近期成交与争议摘要。
  - `docs/数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md`：确认 V1 相关接口为 `GET /api/v1/catalog/search` 与 `GET /api/v1/sellers/{orgId}/profile`，搜索/推荐后端仍由 `platform-core` 访问 OpenSearch / PostgreSQL / Redis，前端不得直连。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：确认卖方主页查看权限 `portal.seller.read`，卖方主页推荐区权限 `portal.recommendation.read`，推荐曝光/点击写接口需要幂等键但本批页面先不伪造曝光写入。
  - `docs/05-test-cases/search-rec-cases.md`、`docs/04-runbooks/recommendation-runtime.md`、`docs/04-runbooks/opensearch-local.md`、`infra/docker/docker-compose.local.yml`：确认 `seller_profile_featured`、`recommend_v1_seller`、本地 PostgreSQL fallback、Keycloak、Kafka/OpenSearch/Redis 宿主机边界。
  - `packages/openapi/catalog.yaml`、`search.yaml`、`recommendation.yaml`、`iam.yaml` 与 `docs/02-openapi/*.yaml`：确认卖方 profile / 搜索 / 推荐 / 会话契约；发现 `SellerProfileView` 后端已返回 `certification_tags / featured_products / rating_summary`，但 catalog OpenAPI 与 SDK 尚未声明，需要本批同步补齐。
  - `apps/platform-core/src/modules/catalog/**`、`search/**`、`recommendation/**`、`apps/portal-web/**`、`packages/sdk-ts/**`：确认 `/sellers/[orgId]` 仍是脚手架；后端卖方 profile 已写 `catalog.seller.profile.read` 审计，推荐读取会写 `recommendation_result` 访问审计。
- 当前完成标准理解：
  - 卖方主页必须只通过 `portal-web -> /api/platform -> platform-core` 调用正式 API，不直连 Kafka / PostgreSQL / OpenSearch / Redis / Fabric。
  - Bearer 会话下真实读取卖方 profile 与 `seller_profile_featured` 推荐；guest/local 或缺少 claims 时显示权限态。
  - 页面需显式展示当前主体、角色、租户/组织、作用域，并展示 `org_id / org_name / org_type / industry_tags / reputation_score / risk_level / listed_product_count / search_document_version / index_sync_status`。
  - 在售商品/服务列表优先来自正式推荐位和卖方投影 `featured_products`，不能以前端 mock 伪造；若后端返回空结果必须展示空态。
  - 联系方式/咨询入口不得暴露私密联系方式或对象路径；本批以受控下单/支持工单入口承接咨询动作。
- 实施计划：
  1. 补齐 catalog OpenAPI 两份 `SellerProfile` schema，并重新生成 `packages/sdk-ts`，避免 profile 扩展字段继续漂移。
  2. 新增卖方主页视图工具与单测，安全解析 `featured_products / rating_summary`，并约束风险/认证展示文案。
  3. 新增 `SellerProfileShell` 并替换 `/sellers/[orgId]` 脚手架，接入 seller profile、`seller_profile_featured` 推荐、状态预演、权限态、空态和错误态。
  4. 更新 E2E 与路由 API 绑定，执行前端、后端、OpenAPI、SDK、真实 API、浏览器 smoke 与数据库审计回查验证。

### BATCH-280（待审批）
- 任务：`WEB-006` 实现卖方主页：主体信息、认证标识、商品列表、联系方式/咨询入口
- 状态：待审批
- 当前任务编号：`WEB-006`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001 ~ WEB-005` 均已本地提交并提供门户会话、受控代理、SDK、搜索与商品详情基础。
- 完成情况：
  - 将 `/sellers/[orgId]` 从路由脚手架升级为正式卖方主页，`SellerProfileShell` 通过 `portal-web -> /api/platform -> platform-core` 读取 `GET /api/v1/sellers/{orgId}/profile`，并显示主体、角色、租户/组织、作用域等敏感访问上下文。
  - 页面展示 `org_id / org_name / org_type / industry_tags / listed_product_count / search_document_version / index_sync_status`、认证标识、信誉/风险摘要、最近成交与争议摘要、链路投影说明、咨询工单入口与下单入口；下载/咨询区域不暴露真实对象路径。
  - 正式接入 `GET /api/v1/recommendations?placement_code=seller_profile_featured`，把推荐结果与卖方 profile 投影中的 `featured_products` 合并为在售商品/服务列表，并覆盖重复去重、空态、错态和无权限态。
  - 补齐 `packages/openapi/catalog.yaml` 与 `docs/02-openapi/catalog.yaml` 的 `SellerProfile.certification_tags / featured_products / rating_summary` 契约，新增 `SellerFeaturedProduct` 与 `SellerRatingSummary` schema，同步重新生成 `packages/sdk-ts`。
  - `scripts/check-openapi-schema.sh` 增加 `SellerProfile` 扩展字段防漂移检查，避免后续 OpenAPI / SDK 再次退化。
  - 新增 `seller-profile-view` 工具与 Vitest 单测，覆盖认证标签、风险等级、信誉分、价格格式化和推荐/投影合并逻辑。
  - `portal-routes` 与 E2E smoke 已同步声明卖方主页的正式 API 绑定和预演态断言。
- 验证：
  - 前端 / SDK：
    - `pnpm --filter @datab/sdk-ts openapi:generate`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
    - `pnpm install --frozen-lockfile`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./scripts/check-openapi-schema.sh`
    - `./scripts/check-keycloak-realm.sh`
  - 真实联调与 smoke：
    - 真实 password grant 获取 `local-platform-admin / platform_admin` Bearer Token。
    - 直连 `platform-core` 验证 `GET /api/v1/sellers/10000000-0000-0000-0000-000000000101/profile`，请求头包含 `Authorization / X-Role / X-Tenant-Id / X-Request-Id`。
    - 直连 `platform-core` 验证 `GET /api/v1/recommendations?placement_code=seller_profile_featured&context_entity_scope=seller&context_entity_id=10000000-0000-0000-0000-000000000101&limit=8`。
    - 生产构建启动 `PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm --filter @datab/portal-web exec next start --hostname 127.0.0.1 --port 3102`。
    - 桌面视口 `1440x1000` 与移动视口 `Pixel 5` 浏览器 smoke 注入真实 Bearer，校验 `Luna Seller Org`、认证标识、在售商品、推荐位、咨询入口与主体上下文。
    - 数据库回查 `audit.audit_event` 的 `catalog.seller.profile.read`、`audit.access_audit` 的 `recommendation_result`、`recommend.recommendation_request` 的 `seller_profile_featured` 请求记录。
    - 静态扫描与浏览器请求捕获确认 `portal-web` 未直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`，浏览器端正式 API 请求均落在 `/api/platform/**`。
- 验证结果：
  - 前端 / SDK 单体与全工作区 `pnpm install / lint / typecheck / test / build` 全部通过；`portal-web` 单测、E2E 与生产构建通过。
  - `pnpm test` 与 `portal-web` E2E 期间仍会打印既有 `ECONNREFUSED 127.0.0.1:8094` 代理噪音，但最终断言通过，未影响本批卖方主页交付。
  - Rust 通用验证全部通过；`cargo check -p platform-core` 与 `cargo test -p platform-core` 仅保留仓库既有 warning，本批未引入新的 Rust 失败。
  - 真实 seller profile API 返回 `success=true`、`org_name=Luna Seller Org`、`certification_tags=["compliance:l2","real_name_verified"]`、`featured_products=3`、`listed_product_count=11`、`index_sync_status=pending` 与 `search_document_version=31`。
  - 真实推荐 API 返回 `seller_profile_featured`、`count=8`，首条商品为 `工业设备运行指标 API 订阅`，`status=listed`，解释码包含 `local:same_seller`。
  - 桌面 / 移动浏览器 smoke 均通过，`horizontalFit=true`；浏览器捕获到 `platformProxyRequests=2`、`directPlatformCoreRequests=0`、`restrictedSystemRequests=0`。
  - 数据库回查确认 `audit.audit_event` 存在 `request_id=web006-curl-seller-profile` 的 `catalog.seller.profile.read / success` 记录，`audit.access_audit` 存在 `request_id=web006-curl-seller-recommendations` 的 `recommendation_result` 记录，`recommend.recommendation_request` 最近记录为 `placement_code=seller_profile_featured / status=served / backend=postgresql_local_minimal`。
  - 本批没有新增业务测试数据；审计记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-006`
  - `页面说明书-V1-完整版.md`：卖方主页主体信息、认证、在售商品、信誉/风险与咨询入口
  - `业务流程图-V1-完整版.md`：搜索结果到卖方主页、卖方 profile 校验与卖方商品过滤流程
  - `商品搜索、排序与索引同步接口协议正式版.md`：`GET /api/v1/sellers/{orgId}/profile` 与搜索/推荐后端边界
  - `菜单权限映射表.md`、`接口权限校验清单.md`、`按钮级权限说明.md`：`portal.seller.read` 与 `portal.recommendation.read`
  - `packages/openapi/catalog.yaml`、`docs/02-openapi/catalog.yaml`、`packages/openapi/recommendation.yaml`、`docs/02-openapi/recommendation.yaml`
- 覆盖的任务清单条目：`WEB-006`
- 未覆盖项：
  - 无。`WEB-006` 要求的卖方主页主体、认证标识、商品列表、联系方式/咨询入口、真实 API 接入、SDK 契约、状态覆盖、E2E / 浏览器 smoke 与数据库回查均已完成；下单创建流程和咨询工单完整写入由后续对应 WEB task 展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-279（计划中）
- 任务：`WEB-005` 实现商品详情页：元信息、卖方信息、SKU、价格、样例预览、下单入口、审核状态徽标
- 状态：计划中
- 说明：在 `WEB-004` 搜索结果已经能跳转 `/products/{productId}` 的基础上，本批将产品详情页从路由脚手架升级为正式商品详情闭环。页面必须真实读取 `GET /api/v1/products/{id}`，并基于返回的 `seller_org_id` 继续读取 `GET /api/v1/sellers/{orgId}/profile`，展示商品基本信息、元数据、SKU、价格、卖方信息、样例/质量/契约摘要、下单入口、审核/上架状态和空态/错态/权限态。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-004` 已提交 `da4e3c8`，搜索页结果卡片会跳转到 `/products/{productId}`，可复用门户会话、受控代理、SDK 与搜索 smoke 环境。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-005` 聚焦商品详情页，不跳到卖方主页或下单页完整实现；DoD 要求页面可访问、状态可用、接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认产品详情页需展示基本信息、样本摘要、元数据详情、字段结构摘要、质量报告摘要、数据契约摘要、权利边界、合同/验收/退款/计费摘要、供方信息、合规限制、审计/存证摘要与下单入口。
  - `docs/数据库设计/接口协议/目录与商品接口协议正式版.md`：确认 V1 商品详情接口为 `GET /api/v1/products/{id}`；`product_type` 不得替代 `sku_type`，`delivery_mode` 只作为默认交付路径或聚合展示值，SKU 真值必须来自 SKU 记录。
  - `docs/权限设计/按钮级权限说明.md`：确认产品详情页按钮权限：`立即下单 -> trade.order.create && 商品 listed`，`收藏 -> portal.search.use`，`查看公开验证 -> catalog.public_verify.read`。
  - `packages/openapi/catalog.yaml`、`docs/02-openapi/catalog.yaml`、`packages/sdk-ts/src/domains/catalog.ts`、`packages/sdk-ts/src/generated/catalog.ts`：确认当前实现期权威契约提供 `ProductDetail` 与 `SellerProfile`，商品详情 schema 包含 `product_id / asset_id / asset_version_id / seller_org_id / title / status / price_mode / price / currency_code / delivery_type / allowed_usage / metadata / skus / search_document_version / index_sync_status`。
  - `apps/platform-core/src/modules/catalog/**`：确认 `get_product_detail` 会对非 `listed` 商品执行卖方 scope 校验，`listed` 商品可按 `catalog.product.read` 读取，并写入 `catalog.product.read` 审计事件。
  - `apps/portal-web/**`、`packages/sdk-ts/**`：确认现有 `/products/[productId]` 仍为脚手架，需要在本批替换为正式业务组件。
- 当前完成标准理解：
  - 商品详情页必须只通过受控 `/api/platform/**` 调用 `platform-core` 正式 API，不直连底层系统。
  - Bearer 会话下真实读取商品详情和卖方信息；guest/local 必须展示明确权限态，不把本地 Header 占位当成正式可售详情完成。
  - 详情页必须展示敏感页面主体、角色、租户、作用域；布局内已有全局身份条，本页也需要在详情上下文中展示当前访问主体。
  - 下单入口只在商品 `status=listed` 时展示；非 listed / suspended / retired 必须禁用并说明原因，不发明新状态语义。
- 实施计划：
  1. 新增 `ProductDetailShell` 客户端组件，替换 `/products/[productId]` 脚手架。
  2. 通过 SDK 读取商品详情与卖方资料，补齐空态、错态、权限态、加载态和状态预演。
  3. 展示 SKU、价格、元数据、权利边界、质量/样本/契约摘要、审计与下单入口约束。
  4. 更新 E2E / 单测，执行完整前后端联调、浏览器 smoke、数据库审计回查后提交。

### BATCH-279（待审批）
- 任务：`WEB-005` 实现商品详情页：元信息、卖方信息、SKU、价格、样例预览、下单入口、审核状态徽标
- 状态：待审批
- 当前任务编号：`WEB-005`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001 ~ WEB-004` 基线继续生效；搜索页结果卡片已能跳转 `/products/{productId}`，本页仍只经 `portal-web -> /api/platform -> platform-core` 访问正式 API。
- 完成情况：
  - `/products/[productId]` 已从脚手架替换为正式 `ProductDetailShell`，Bearer 会话下真实读取 `GET /api/v1/products/{id}`，并基于 `seller_org_id` 继续读取 `GET /api/v1/sellers/{orgId}/profile`。
  - 页面展示商品状态、索引投影状态、价格、SKU 矩阵、元数据、样本哈希、全量哈希、字段/质量/加工/契约摘要、权利边界、审计提示、卖方 profile、相似推荐和下单入口；下单入口严格要求商品 `status=listed` 且存在样本摘要或样本哈希。
  - 敏感页面上下文在页内显式展示主体、角色、租户/组织与作用域；`guest / local / claims 缺失` 显示权限态，加载态、空态、错误态和权限态均可通过 `preview` 参数与 E2E 覆盖。
  - `ProductDetail.metadata` 展示时会脱敏对象路径类字段；浏览器端只发起 `/api/platform/api/v1/products/**`、`/api/platform/api/v1/sellers/**`、`/api/platform/api/v1/recommendations`，没有直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`。
  - 修正 catalog OpenAPI 漂移：`GET /api/v1/products/{id}` 与 `GET /api/v1/sellers/{orgId}/profile` 的 200 响应从裸对象同步为 `ApiResponseProductDetail / ApiResponseSellerProfile`，并重新生成 `packages/sdk-ts`。
  - 收敛 catalog 角色口径：目录读、卖方 profile、商品写、审核与冻结权限矩阵改用正式 V1 角色集合；测试与 `070_seed_role_permissions_v1.sql` 同步补齐正式角色授权，避免旧租户运营占位角色 / `developer` 口径继续传播。
  - 修正种子 SKU 真值：`db028` 与 `searchrec014` 的 `sku_type` 与 `sku_code` 统一为八个标准 SKU，`FILE_SUB / API_PPU / SBX_STD` 的 `trade_mode` 同步为正式值，校验脚本增加 `sku_code = sku_type` 断言。
  - `get_product_detail` 后端查询将 `asset_version` 的 `schema_hash / sample_hash / full_hash / origin_region / allowed_region / release_mode / processing_stage / standardization_status / query_surface_type / status / version_no` 合并进返回 metadata，支持页面样例预览和下单门禁。
- 验证：
  - 前端 / 契约：
    - `pnpm install --frozen-lockfile`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
    - `pnpm --filter @datab/sdk-ts typecheck`
    - `./scripts/check-openapi-schema.sh`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo test -p platform-core catalog::tests::cat020_read_db -- --nocapture`
    - `cargo test -p platform-core catalog::tests::tests --lib`
    - `cargo test -p platform-core modules::catalog::tests::listing_submit_review --lib`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调 / smoke：
    - `./scripts/check-keycloak-realm.sh`
    - `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/verify-seed-010-030.sh`
    - `DB_HOST=127.0.0.1 DB_PORT=5432 DB_NAME=datab DB_USER=datab DB_PASSWORD=datab_local_pass ./db/scripts/verify-seed-033.sh`
    - 启动当前代码临时 `platform-core`：`DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab KAFKA_BROKERS=127.0.0.1:9094 MINIO_ENDPOINT=http://127.0.0.1:9000 APP_HOST=127.0.0.1 APP_PORT=18080 cargo run -p platform-core`
    - 真实 Keycloak password grant 获取 `local-platform-admin / platform_admin` Bearer 后，`curl` 验证商品详情、卖方 profile 与 `product_detail_bundle` 推荐接口。
    - 启动 `portal-web` production server：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:18080 pnpm --filter @datab/portal-web exec next start --hostname 127.0.0.1 --port 3102`
    - Playwright 桌面 `1440x1000` 与移动 `Pixel 5` smoke：校验商品标题、`API_SUB / API_PPU`、`Luna Seller Org`、`searchrec014-s1-sample`、`进入下单页`、主体/角色/租户/作用域可见，并确认浏览器没有直连 `http://127.0.0.1:18080`。
    - 数据库回查 `audit.audit_event`、`catalog.product_sku` 与 `catalog.asset_version`。
- 验证结果：
  - 前端全量 `pnpm install / lint / typecheck / test / build` 全部通过；`portal-web` 单体 lint / typecheck / unit / e2e / build 全部通过；`pnpm test` 中 `console-web / portal-web` Playwright WebServer 仍打印既有 `ECONNREFUSED 127.0.0.1:8094` 代理失败日志，但测试断言通过。
  - 后端 `cargo check -p platform-core`、全量 `cargo test -p platform-core`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 全部通过。全量测试曾暴露两个 listing review 校验用例仍使用 `tenant_admin` 触发 403，已改为正式 `platform_reviewer` 后重跑通过。
  - 真实商品详情 `curl` 返回 `success=true`、标题 `工业设备运行指标 API 订阅`、`status=listed`、`sku_types=[API_SUB, API_PPU]`、`sample_hash=searchrec014-s1-sample`、`full_hash=searchrec014-s1-full`、`origin_region=cn-sh`、`asset_version_status=published`；卖方 profile 返回 `Luna Seller Org`、`status=active`；推荐接口返回 `4` 条真实推荐项。
  - 浏览器 smoke 通过：桌面和移动视口均真实渲染商品详情、卖方、SKU、样本哈希与下单入口；捕获请求仅包含 `/api/platform/api/v1/products/{id}`、`/api/platform/api/v1/sellers/{orgId}/profile`、`/api/platform/api/v1/recommendations`，`directPlatformCoreRequestCount=0`。
  - 数据库回查通过：`audit.audit_event` 出现 `catalog.product.read` 与 `catalog.seller.profile.read` 成功记录；`catalog.product_sku` 中 `db028 / searchrec014` 样例的 `sku_code` 与 `sku_type` 已全部对齐标准 SKU；商品 `20000000-0000-0000-0000-000000000309` 的 `asset_version.sample_hash/full_hash` 与页面展示一致。
  - 受限系统扫描通过：`apps/portal-web/src` 与 `apps/portal-web/e2e` 未发现直连 Kafka / PostgreSQL / OpenSearch / Redis / Fabric 的实现；唯一命中是搜索错误态文案，明确由 `platform-core` 处理 OpenSearch 到 PostgreSQL fallback。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-005`
  - `页面说明书-V1-完整版.md`：产品详情页章节、敏感页面主体/角色/租户/作用域展示要求
  - `目录与商品接口协议正式版.md`：商品详情、卖方 profile、`sku_type` 真值与 `delivery_mode` 边界
  - `按钮级权限说明.md`：产品详情下单、收藏、公开验证按钮权限
  - `packages/openapi/catalog.yaml`、`docs/02-openapi/catalog.yaml`、`packages/sdk-ts/**`：商品详情与卖方 profile 契约
- 覆盖的任务清单条目：`WEB-005`
- 未覆盖项：
  - 无。卖方主页完整页面由下一任务 `WEB-006` 继续展开；下单页完整创建流程由后续 `WEB-009` 展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-278（计划中）
- 任务：`WEB-004` 实现商品搜索页：关键词、筛选、排序、结果卡片、空状态与错误状态
- 状态：计划中
- 说明：在 `WEB-003` 首页已接入搜索预览的基础上，本批将 `/search` 从路由脚手架升级为正式商品搜索页，真实绑定 `GET /api/v1/catalog/search` 与 `packages/sdk-ts` 当前生成契约，支持关键词、对象范围、行业、标签、交付方式、价格区间、排序、分页、结果卡片、空态、错态、加载态与权限态。按复审确认的 A 方案执行：当前 task 不伪造尚未在 `packages/openapi/search.yaml` / 后端 `SearchQuery` 中落地的 Facet 聚合、卖方主体类型、敏感等级、价格模式、供方筛选，而是在页面上明确展示当前契约边界，避免前端发明字段或 mock 聚合。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001`、`WEB-002`、`WEB-003` 已本地提交，门户工程、身份条、受控 `/api/platform/**` 代理、Keycloak Bearer 会话、SDK 绝对 URL 修复与首页搜索预览可复用。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-004` 仅实现商品搜索页，不跳到商品详情或卖方主页完整实现；DoD 要求页面可访问、空态/错态/权限态可用、与接口契约对齐并通过最小 E2E / smoke。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认搜索页目标为统一搜索、筛选、排序、结果列表和统计；异常需覆盖无结果、OpenSearch fallback、风险商品隐藏购买入口、结果状态校验失败提示刷新。
  - `docs/数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md`：确认 V1 搜索接口为 `GET /api/v1/catalog/search`，参数包含 `q / entity_scope / industry / tags / delivery_mode / price_min / price_max / sort / page / page_size`。
  - `docs/原始PRD/商品搜索、排序与索引同步设计.md`：确认 V1 采用 OpenSearch 读模型、Redis 缓存与 PostgreSQL fallback，但前端不得直连这些系统；本地 / demo 可展示 PostgreSQL fallback 后端标识。
  - `packages/openapi/search.yaml`、`docs/02-openapi/search.yaml`、`packages/sdk-ts/src/domains/search.ts`、`packages/sdk-ts/src/generated/search.ts`：确认当前实现期权威契约未返回 facet 聚合，也未提供卖方主体类型、敏感等级、价格模式、供方筛选参数；页面必须以已生成 SDK 类型为准。
  - `apps/platform-core/src/modules/search/**`、`docs/05-test-cases/search-rec-cases.md`、`docs/04-runbooks/search-reindex.md`、`docs/04-runbooks/opensearch-local.md`：确认搜索读取权限、审计写入、backend 标识、fallback 与测试验证方式。
  - `docs/权限设计/菜单权限映射表.md`、`docs/权限设计/按钮级权限说明.md`、`docs/权限设计/接口权限校验清单.md`：确认搜索读取权限 `portal.search.read`，搜索执行权限 `portal.search.use`，购买入口需受后续下单权限和商品状态约束。
  - `packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`、`apps/platform-core/src/modules/iam/**`：按复审结论同步收敛 `GET /api/v1/auth/me` 示例角色到正式 V1 角色集合，避免旧租户运营占位角色继续传播到 WEB 登录态占位。
- 当前完成标准理解：
  - 搜索页必须只通过 `portal-web -> /api/platform -> platform-core` 调用正式 API，不直连 PostgreSQL / Kafka / OpenSearch / Redis / Fabric。
  - Bearer 会话下真实调用 `sdk.search.searchCatalog()`；guest / local header 占位不得把需要 Bearer 的正式搜索误判为完成，必须显示明确权限态。
  - 表单校验必须用 Zod / React Hook Form，查询状态必须有加载 / 空 / 错 / 权限 / 后端 fallback 可视化，结果卡片必须显示状态、索引同步状态、评分、标签、价格和购买入口约束。
  - 结果项状态异常或非 product 结果不得展示购买入口；商品状态 / 索引同步状态不可购买时给出刷新提示，不发明新的业务状态名。
- 实施计划：
  1. 先完成 IAM 角色示例漂移收敛与最小校验，保证 `auth/me` 契约和门户登录占位不继续输出非正式角色。
  2. 新增正式 `SearchShell` 客户端组件，并替换 `/search` 路由脚手架。
  3. 更新门户 E2E / 单测覆盖搜索页权限态、空态、错态、表单校验与 URL 状态同步。
  4. 执行前端、后端、OpenAPI、SDK、真实 API、数据库回查与浏览器 smoke 验证，再写入“待审批”并提交。

### BATCH-278（待审批）
- 任务：`WEB-004` 实现商品搜索页：关键词、筛选、排序、结果卡片、空状态与错误状态
- 状态：待审批
- 当前任务编号：`WEB-004`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001 / WEB-002 / WEB-003` 基线继续生效；本批没有新增浏览器直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric` 的实现，搜索仍只经 `portal-web -> /api/platform -> platform-core`。
- 完成情况：
  - `/search` 已从 `PortalRoutePage` 脚手架替换为正式 `SearchShell`，页面承载 Hero、主体上下文、搜索表单、契约边界提示、结果统计、结果卡片、分页、权限态、加载态、空态与错误态。
  - 新增 `apps/portal-web/src/lib/search-query.ts` 与单测，统一处理 URL 参数、React Hook Form 值、Zod 校验和 `SearchCatalogQuery` 转换；只序列化当前 OpenAPI / SDK 已支持的 `q / entity_scope / industry / tags / delivery_mode / price_min / price_max / sort / page / page_size`。
  - 搜索表单使用 React Hook Form + Zod 校验，覆盖价格格式、价格区间、分页大小、枚举兜底和标签拆分；表单提交同步 URL，分页链接保持当前筛选条件。
  - Bearer 会话下真实调用 `sdk.search.searchCatalog()`；`guest / local / claims 缺失` 显示明确搜索权限态，不把本地 Header 占位当成正式搜索登录态。
  - 结果卡片展示 `entity_scope / status / index_sync_status / document_version / score / quality / reputation / hotness / price / seller / tags / industry_tags / delivery_modes`；仅 `product && status=listed` 展示下单入口，卖方结果与不可购买状态显示刷新/进入卖方主页提示。
  - 页面明确展示契约边界：Facet 聚合、卖方主体类型、敏感等级、价格模式、供方筛选当前未在 `packages/openapi/search.yaml` / 后端 `SearchQuery` 中提供，本批不以前端 mock 或新状态名伪造。
  - 按复审结论修正 IAM 角色漂移：`packages/openapi/iam.yaml` 与 `docs/02-openapi/iam.yaml` 的 `auth/me` 示例改为 `jwt_mirror=["tenant_developer"]`、`local_test_user=["tenant_admin"]`；门户登录态占位移除旧租户运营占位角色；IAM 静态角色矩阵和测试同步为正式 V1 角色名，并在 `070_seed_role_permissions_v1.sql` 补齐 `tenant_developer -> iam.session.read`，确保示例 200 响应可验证。
  - `apps/portal-web/e2e/smoke.spec.ts` 已更新为搜索页正式权限态、空态和错误态断言，不再依赖旧脚手架“空态预演”。
- 验证：
  - 前端 / 工作区：
    - `pnpm install --frozen-lockfile`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo test -p platform-core iam::tests --lib`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
    - `./scripts/check-openapi-schema.sh`
  - 真实联调与 smoke：
    - `./scripts/check-keycloak-realm.sh`
    - 真实 password grant 获取 `local-platform-admin / platform_admin` Bearer Token，并验证 token claims 含 `user_id / org_id / realm_access.roles`
    - 直连 `platform-core`：`GET /api/v1/catalog/search?q=工业设备运行指标&entity_scope=all&page=1&page_size=4`
    - 门户生产构建启动：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm --filter @datab/portal-web exec next start --hostname 127.0.0.1 --port 3101`
    - 浏览器端 Bearer smoke：
      - 桌面视口 `1440x1200`：校验主体条、搜索统计、结果卡片、官方样例结果、请求边界与横向适配
      - 移动视口 `390x844`：校验搜索统计与结果卡片可见、无横向溢出
    - 数据库回查：`audit.access_audit` 中 `accessor_user_id='10000000-0000-0000-0000-000000000353'` 的 `target_type='search_catalog'` 访问记录。
- 验证结果：
  - `pnpm install --frozen-lockfile`、`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过；`portal-web` 单体 lint / typecheck / unit / e2e / build 全部通过；`sdk-ts` 构建时重新生成 OpenAPI 类型，未产生额外契约漂移。
  - `pnpm test` 期间 `portal-web / console-web` 的 Playwright WebServer 仍会打印既有 `ECONNREFUSED 127.0.0.1:8094` 代理日志，但最终 E2E 断言全部通过；真实联调用 `PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080` 已验证正式 API。
  - `cargo fmt --all`、`cargo check -p platform-core`、`cargo test -p platform-core`、`cargo test -p platform-core iam::tests --lib`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 全部通过；Rust 仍有既有 warning，本批未引入失败。
  - `./scripts/check-openapi-schema.sh` 通过，确认 `getAuthMe / SessionContextView / application/json` 防漂移检查仍有效。
  - `./scripts/check-keycloak-realm.sh` 通过，`local-platform-admin` Bearer Token 的 `user_id=10000000-0000-0000-0000-000000000353`、`org_id=10000000-0000-0000-0000-000000000103`、`role=platform_admin` 可用。
  - 真实 `curl` 搜索通过：`success=true`、`total=2`、`backend=postgresql`，首条结果为 `工业设备运行指标 API 订阅`，状态 `listed`，`index_sync_status=pending`。
  - 浏览器端 Bearer smoke 通过：页面 API 响应 `success=true / total=2 / backend=postgresql / first_status=listed / first_index_sync_status=pending`；桌面与移动 `horizontalFit=true`；捕获到的浏览器端搜索请求仅为 `/api/platform/api/v1/catalog/search?...`，`directPlatformCoreRequestCount=0`。
  - 数据库回查通过：`audit.access_audit` 最近记录包含多条 `target_type=search_catalog`、`accessor_user_id=10000000-0000-0000-0000-000000000353`、带 `request_id` 的访问留痕；审计按 append-only 保留，未清理。
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；没有向业务表插入需清理的临时测试数据，搜索访问审计按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-004`
  - `页面说明书-V1-完整版.md`：4.3 搜索页
  - `商品搜索、排序与索引同步接口协议正式版.md`：4.1 / 7.1 / 8 / 9
  - `商品搜索、排序与索引同步设计.md`：5.1 / 5.2 / 6.1 / 6.2
  - `search-rec-cases.md`：Bearer 搜索、PostgreSQL fallback、审计留痕与本地验证边界
  - `packages/openapi/search.yaml`、`docs/02-openapi/search.yaml`、`packages/openapi/iam.yaml`、`docs/02-openapi/iam.yaml`
- 覆盖的任务清单条目：`WEB-004`
- 未覆盖项：
  - 无。`WEB-004` 要求的关键词、筛选、排序、结果卡片、空态与错误态已按当前正式搜索契约完成；OpenAPI / 后端尚未提供的 Facet 聚合与额外筛选字段未在本批伪造，后续如冻结契约新增字段再进入对应任务实现。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-276（计划中）
- 任务：`WEB-002` 初始化 `apps/console-web/` Next.js 项目，承接运营、审计、开发者、ops 页面
- 状态：计划中
- 说明：在 `WEB-001` 门户工程基线已提交（`e2c85e6`）的前提下，继续按 `WEB-002` 冻结口径建立 `console-web`。当前 `apps/console-web/` 仅有 README 占位，不能视为完成；本批将复用已建立的 workspace / `sdk-ts` / 受控代理模式，初始化控制台工程、控制台布局、登录态占位、运营/审计/开发者/ops 路由骨架和最小真实联调首页，不提前把 `WEB-008 / WEB-014 / WEB-015 / WEB-016` 的完整业务页面一次做满。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001` 已完成并提交，可复用 `pnpm workspace`、`packages/sdk-ts`、受控 `/api/platform` 代理模式和会话占位方案。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-002` 交付物是 `apps/console-web/`，DoD 仍是“页面可访问、空态/错态/权限态可用、契约对齐、最小 E2E / 手工 smoke 通过”。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：复核页面域总览、`10.1 审计联查页`、`11.1-11.5 开发者首页 / 测试应用 / 状态联查 / 测试资产 / 搜索运维页`、页面间路由关系和控制台域页面目标。
  - `docs/开发准备/服务清单与服务边界正式版.md`：确认 `console-web` 负责运营后台、审计后台、风控后台、开发态运维与联调页面、可观测性联查入口，不负责供需方主交易门户。
  - `docs/权限设计/菜单树与路由表正式版.md`、`菜单权限映射表.md`、`按钮级权限说明.md`、`接口权限校验清单.md`：确认控制台 V1 路由、`page_key`、查看权限与关键操作权限，包括 `/ops/review/*`、`/ops/risk`、`/ops/audit/trace`、`/ops/consistency`、`/ops/search`、`/developer/*`。
  - `packages/openapi/ops.yaml`、`packages/openapi/audit.yaml`、`packages/openapi/iam.yaml` 与 `apps/platform-core/src/modules/audit/**`、`iam/**`：确认控制台基线可优先接入 `ops.observability.overview`、`ops.outbox`、`audit.traces`、`iam apps` 等稳定读取接口，继续沿 `platform-core` 正式 API 边界实现。
- 当前完成标准理解：
  - 需要形成可运行的控制台工程基座，而不是只创建目录或 README。
  - 至少应具备：控制台布局、路由元数据、会话占位、受控 `platform-core` API 代理、运营/审计/开发者/ops 正式入口页骨架、最小自动化与手工 / 浏览器 smoke。
  - `WEB-002` 不提前实现审核、审计联查、ops 和开发者页面的完整业务细节，但必须把这些页面的正式路由、权限、基础状态和控制台视觉语言先固定下来，并完成至少一组真实 API 摘要读取验证。
- 实施计划：
  1. 初始化 `apps/console-web` 工程文件、依赖、Next.js 配置和控制台主题基座。
  2. 复用 `sdk-ts` 与受控代理方案，扩展控制台首页真正要用到的 `ops / audit / iam` 读取 SDK。
  3. 落控制台导航、主体条、登录态占位、路由注册和运营/审计/开发者/ops 页面骨架。
  4. 用真实 `platform-core` API 完成控制台首页的最小摘要联调，并补最小 Vitest / Playwright / 浏览器 smoke。
  5. 执行完整验证、写“待审批”日志、本地提交后继续进入 `WEB-003`。

### BATCH-276（待审批）
- 任务：`WEB-002` 初始化 `apps/console-web/` Next.js 项目，承接运营、审计、开发者、ops 页面
- 状态：待审批
- 当前任务编号：`WEB-002`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001` 基线继续生效；`console-web` 仍只通过 `platform-core` 正式 API / 受控 `/api/platform/**` 代理访问后端，没有新增浏览器直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric` 的实现。
- 完成情况：
  - 初始化 `apps/console-web/**` 的 `Next.js App Router + TypeScript + Tailwind v4` 工程，接入 `TanStack Query`、`React Hook Form`、`Zod`、`Vitest`、`Playwright`，并形成区别于门户的控制台视觉主题、侧边导航、主体条、登录态占位与统一状态预演组件。
  - 复用 `WEB-001` 的 workspace / SDK / 受控代理模式，为控制台补齐 `packages/sdk-ts/src/domains/audit.ts`，并扩展 `packages/sdk-ts/src/domains/ops.ts` 与 `index.ts`，正式暴露首页要用到的 `ops.observability.overview`、`ops.outbox`、`audit.traces`、`developer.trace` 等读取能力，不再依赖草稿 fetch 或手写漂移字段。
  - 落地 `apps/console-web/src/lib/console-routes.ts` 与配套测试，把 `控制台首页`、`主体审核台`、`产品审核台`、`合规审核台`、`风控工作台`、`审计联查页`、`证据包导出页`、`一致性联查页`、`出站事件 / Dead Letter 页`、`搜索运维页`、`开发者首页`、`测试应用页`、`状态联查页`、`测试资产页` 这些 `WEB-002` 要求的控制台正式入口全部按冻结 `page_key / 路径 / 权限` 注册。
  - `apps/console-web/src/app/api/platform/[...path]/route.ts`、`src/lib/session.ts`、`src/actions/session.ts` 实现控制台 HttpOnly 会话 Cookie 与受控代理边界；浏览器只访问 `/api/platform/**`，由 Next Route Handler 注入 `x-login-id / x-role` 或 Bearer，并统一补 `x-request-id=console-*`。
  - `apps/console-web/src/components/console/home-shell.tsx` 把控制台首页真实绑定到 `platform-core` 的 `GET /healthz`、`GET /api/v1/ops/observability/overview`、`GET /api/v1/ops/outbox`、`GET /api/v1/audit/traces`，显式展示 readiness、可观测后端快照、outbox 摘要、审计 trace 摘要、空态/错态/权限态，而不是只放静态说明。
  - `apps/console-web/src/components/console/auth-placeholder-dialog.tsx` 修正了本地联调预置角色：最初的 `platform_admin` / `audit_admin` 预置不能同时满足首页所需的 `iam.session.read` 与 `ops.observability.read`；本批按后端正式权限矩阵把推荐本地 Header 角色切到 `platform_auditor`，保证首页主体条与 ops 摘要能在同一正式身份链路下同时联通。
  - `apps/console-web/e2e/smoke.spec.ts`、`src/lib/console-routes.test.ts` 提供最小自动化覆盖；README、工程脚本与生产端口口径同步补齐。
- 验证：
  - 前端 / 契约：
    - `pnpm install`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - `pnpm --filter @datab/console-web lint`
    - `pnpm --filter @datab/console-web test`
    - `pnpm --filter @datab/console-web build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调与 smoke：
    - 宿主机方式启动 `platform-core`：`set -a && source infra/docker/.env.local && set +a && export KAFKA_BROKERS=127.0.0.1:${KAFKA_EXTERNAL_PORT:-9094} APP_MODE=local PROVIDER_MODE=mock APP_HOST=127.0.0.1 && cargo run -p platform-core-bin`
    - 直接 `curl`：
      - `GET http://127.0.0.1:8080/healthz`
      - `GET http://127.0.0.1:8080/api/v1/auth/me`
      - `GET http://127.0.0.1:8080/api/v1/ops/observability/overview`
      - `GET http://127.0.0.1:8080/api/v1/ops/outbox?page=1&page_size=3`
      - `GET http://127.0.0.1:8080/api/v1/audit/traces?page=1&page_size=3`
    - 数据库回查：
      - `select count(*) from ops.observability_backend where enabled = true`
      - `select count(*) from ops.outbox_event`
      - `select count(*) from audit.audit_event`
    - 控制台代理 `curl`：
      - `GET http://127.0.0.1:3102/api/platform/healthz`
      - `GET http://127.0.0.1:3102/api/platform/api/v1/auth/me`
      - `GET http://127.0.0.1:3102/api/platform/api/v1/ops/observability/overview`
      - `GET http://127.0.0.1:3102/api/platform/api/v1/ops/outbox?page=1&page_size=3`
      - `GET http://127.0.0.1:3102/api/platform/api/v1/audit/traces?page=1&page_size=3`
    - 浏览器 smoke：
      - 生产构建方式启动 `console-web`：`PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm exec next start --hostname 127.0.0.1 --port 3102`
      - 使用 Playwright + `datab_console_session` HttpOnly Cookie 在桌面与移动视口打开首页，确认主体条、ops 摘要、outbox 摘要、审计摘要和响应式加载。
- 验证结果：
  - `pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过；`console-web` 单体的 lint / test / build 也通过。
  - `pnpm test` 期间 `portal-web` 与 `console-web` 的开发服务器仍会打印 `ECONNREFUSED 127.0.0.1:8094`，这是默认 `PLATFORM_CORE_BASE_URL` 缺省时的预期受控降级噪音，不影响 Playwright 断言结果；门户与控制台 E2E 均实际通过。
  - `cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 全部通过；本批未引入新的 Rust / SQLx 回归。
  - 真实 `curl` 验证通过：
    - `GET /healthz` 返回 `{"success":true,"data":"ok"}`
    - `GET /api/v1/auth/me` 在本地测试主体 `search-ops-user-1776909499493` 下返回真实 `user_id / org_id / login_id / display_name / roles / auth_context_level`
    - `GET /api/v1/ops/outbox?page=1&page_size=3` 返回 `total=67`
    - `GET /api/v1/audit/traces?page=1&page_size=3` 返回真实审计条目，含 `audit_id / request_id / action_name`
    - `GET /api/v1/ops/observability/overview` 在 `platform_admin` 下被正式拒绝 `403 ops.observability.read`，切换到 `audit_admin` / `platform_auditor` 后按正式权限矩阵返回 `backend_statuses`、`alert_summary`、`slo_summary` 与 `recent_incidents`
    - 经控制台 `/api/platform/**` 代理访问的同一路径返回与直连 `platform-core` 一致
  - 数据库回查与 API 结果对齐：
    - `ops.observability_backend enabled = 6`
    - `ops.outbox_event = 67`
    - `audit.audit_event` 在本批 live 验证期间从 `827` 增长到 `832`，增长量来自 `auth/me` 与控制台联调读取产生的 append-only 审计事件，符合预期
  - 浏览器 smoke 通过：使用 `platform_auditor` 本地会话 Cookie 后，桌面与移动视口首页都正确显示 `Search Ops User 1776909499493 / platform_auditor / 8f7a8003-1ba2-44bd-b120-43206bcebf3c / aal1`，同时渲染 `alertmanager_main`、`recommend.behavior_recorded`、`iam.session.context.read` 等真实摘要；抓到的浏览器请求只有 `5` 个 `/api/platform/**` 调用，`directPlatformCoreRequestCount = 0`。
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；也未创建需要清理的业务测试数据，审计事件按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-002`
  - `页面说明书-V1-完整版.md`：控制台域页面总览、审计联查页、开发者首页 / 测试应用 / 状态联查 / 测试资产 / 搜索运维页
  - `菜单树与路由表正式版.md`、`菜单权限映射表.md`、`按钮级权限说明.md`、`接口权限校验清单.md`：控制台 `page_key`、路径、查看权限与操作权限
  - `服务清单与服务边界正式版.md`：控制台只经 `platform-core` 正式 API 接入 ops / audit / developer 控制面
  - `packages/openapi/ops.yaml`、`audit.yaml`、`iam.yaml`：控制台首页读取的真实契约边界
- 覆盖的任务清单条目：`WEB-002`
- 未覆盖项：
  - 无。`WEB-002` 要求的控制台工程初始化、布局、登录态占位、正式路由挂载、SDK / 代理绑定、最小自动化与真实联调闭环均已完成；更完整的审核、审计、ops、开发者业务细节由 `WEB-008 / WEB-014 / WEB-015 / WEB-016` 等后续任务继续展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。

### BATCH-277（计划中）
- 任务：`WEB-003` 实现门户首页：场景导航、推荐位、搜索入口、标准链路快捷入口
- 状态：计划中
- 说明：`WEB-001` 已把门户工程、身份条、受控 `/api/platform/**` 代理和标准场景读取基线建好，`WEB-003` 开始把首页从“基线说明页”升级为正式业务首页。当前批将围绕 `portal_home` 的冻结职责落地四个首页主模块：场景导航、`home_featured` 推荐位、搜索入口与五条标准链路快捷入口；同时显式区分 `guest / local / bearer` 三种会话下的可见能力，不把需要 Bearer 的推荐/搜索 API 误走本地 header 占位链路。
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 已满足；`WEB-001` 与 `WEB-002` 已分别提交 `e2c85e6`、`7b7c5cd`，可复用门户/控制台工程基线、`packages/sdk-ts`、`auth/me` 会话校验、受控代理和 `platform-core` 本地联调环境。
- 已阅读证据（文件+要点）：
  - `docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`：确认 `WEB-003` 仅针对门户首页，不得跳到搜索详情或卖方主页的完整实现，但必须把首页四大模块和最小真实联调闭环做完整。
  - `docs/页面说明书/页面说明书-V1-完整版.md`：确认首页需承载平台导航、热门行业入口、推荐商品/服务、高可信卖方推荐、平台能力说明，并在未登录或推荐异常时做受控降级。
  - `docs/原始PRD/商品推荐与个性化发现设计.md`：确认首页推荐位至少覆盖热门数据、热门服务、新品推荐、高可信卖方推荐、行业特色推荐等首页推荐语义。
  - `docs/全集成文档/数据交易平台-全集成基线-V1.md`：确认首页官方展示必须显式覆盖五条标准链路，且八个标准 SKU 不能被错误并类。
  - `docs/权限设计/菜单权限映射表.md`、`按钮级权限说明.md`、`接口权限校验清单.md`、`菜单树与路由表正式版.md`：确认首页查看权限 `portal.home.read`、推荐读取权限 `portal.recommendation.read`、搜索查看/执行权限 `portal.search.read / portal.search.use`，以及首页仍属于 `portal_home` 正式路由。
  - `docs/05-test-cases/search-rec-cases.md`：确认 `home_featured` 在演示种子后必须返回五条标准链路官方商品样例，顺序固定为 `S1 -> S2 -> S3 -> S4 -> S5`，并且首页推荐异常时要能受控退化。
  - `packages/openapi/catalog.yaml`、`recommendation.yaml`、`search.yaml` 与对应 `docs/02-openapi/*.yaml`：确认首页需要真实绑定 `GET /api/v1/catalog/standard-scenarios`、`GET /api/v1/recommendations`、`GET /api/v1/catalog/search` 的当前实现期契约。
  - `apps/platform-core/src/modules/catalog/standard_scenarios.rs`、`recommendation/**`、`search/**`：确认五条标准链路与推荐 `home_featured`、搜索 API 的后端真实实现、权限矩阵和本地 / staging 运行边界。
  - `apps/portal-web/**`、`packages/sdk-ts/**`：确认现有首页仍是 `WEB-001` 基线说明页，需要在当前 task 内替换为正式首页实现。
- 当前完成标准理解：
  - 首页必须真实显示场景导航、推荐位、搜索入口与标准链路快捷入口，不再停留在路由挂载说明页。
  - 首页必须只通过 `portal-web -> /api/platform -> platform-core` 的正式边界读取数据；推荐与搜索预览只允许在 Bearer 会话下调用正式 API，`guest / local` 要给出明确可理解的空态 / 权限态 / 降级态。
  - 首页必须显式展示五条标准链路和八个标准 SKU 的官方命名，不得改名、合并或弱化。
  - 首页至少要补齐自动化与真实联调 smoke，证明浏览器端请求走受控代理而非直连 `platform-core`。
- 实施计划：
  1. 重构 `portal_home` 首页组件，落地首页 Hero、场景导航、标准链路快捷入口与推荐/搜索模块。
  2. 扩展首页路由元数据与最小测试，确保 `portal_home` API 绑定包含 `recommendation / search`。
  3. 执行 `pnpm lint / typecheck / test / build` 与后端通用校验。
  4. 通过真实 Keycloak Bearer Token 启动门户生产构建，验证首页推荐、搜索预览、场景快捷入口、桌面/移动 smoke 以及浏览器仅访问 `/api/platform/**`。

### BATCH-277（待审批）
- 任务：`WEB-003` 实现门户首页：场景导航、推荐位、搜索入口、标准链路快捷入口
- 状态：待审批
- 当前任务编号：`WEB-003`
- 前置依赖核对结果：`BOOT-007`、`CORE-026`、`TRADE-028`、`BIL-020` 与已提交的 `WEB-001` / `WEB-002` 基线继续生效；门户仍只经 `portal-web -> /api/platform -> platform-core` 访问正式 API，没有新增浏览器直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric` 的实现。
- 完成情况：
  - 将 `apps/portal-web/src/components/portal/home-shell.tsx` 从 `WEB-001` 基线说明页重构为正式首页，落下 Hero、场景导航、标准链路快捷入口、推荐位、搜索入口与首页联调概览五个主模块，并显式区分 `guest / local / bearer` 三种会话能力。
  - 首页标准链路严格按冻结口径直出五条标准链路与八个标准 SKU：`FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`；`S1 -> S5` 官方场景名与搜索入口映射保持不变，不再把 `SHARE_RO / QRY_LITE / RPT_STD` 错并回大类。
  - 为 `catalog/standard-scenarios` 增加“Bearer 实时回填 + guest/local 冻结样例回退”双层承接：未登录或本地 header 占位时首页仍保留官方五链路入口，不把标准场景导航整体锁空；Bearer 建立后自动回填 `GET /api/v1/catalog/standard-scenarios` 真响应，并在首页指标区显示 `5 / live`。
  - 推荐位正式绑定 `placement_code=home_featured`，Bearer 会话下走 `GET /api/v1/recommendations`；当推荐异常时退回官方五链路样例并保留错误提示。搜索入口正式绑定 `GET /api/v1/catalog/search`，采用 `useDeferredValue` 做首页预览，并把默认关键词和标准链路搜索词统一到官方场景名，避免 `S1` 快捷入口落空。
  - `apps/portal-web/src/lib/session.ts`、`src/actions/session.ts`、`src/components/portal/identity-strip.tsx`、`src/app/layout.tsx`、`src/app/page.tsx` 已补齐 Bearer claims 会话预览：从 Keycloak token 解析 `user_id / org_id / preferred_username / name / roles / exp`，在门户主体条稳定显示“当前主体 / 角色 / 租户 / 作用域”，并在服务器到 `platform-core` 的受控调用上附带兼容头 `x-user-id / x-tenant-id / x-role`，用于当前仍依赖头部角色矩阵的 `catalog` / `iam` 读取接口。
  - `packages/sdk-ts/src/core/http.ts` 修复了绝对 URL 场景下 `appendQuery()` 错误裁剪 origin 的缺陷；新增 `packages/sdk-ts/src/core/http.test.ts` 回归用例，确保服务端 `baseUrl=http://127.0.0.1:8080` 时不会再把请求降成非法相对路径。该修复直接解掉了门户 Bearer 登录校验阶段的 `Failed to parse URL from /api/v1/catalog/search?...` 阻塞。
  - `apps/portal-web/src/lib/portal-routes.ts` 与测试已同步更新 `portal_home` 描述和 API 绑定，把 `health/ready`、`auth/me`、`catalog/standard-scenarios`、`recommendations`、`catalog/search`、`orders/standard-templates` 全部纳入首页正式边界说明。
  - `apps/portal-web/e2e/smoke.spec.ts` 已同步刷新为首页当前业务语义，确保首页、搜索页和标准下单页的脚手架 / 状态预演断言与现状一致。
- 验证：
  - 前端 / 工作区：
    - `pnpm install --frozen-lockfile`
    - `pnpm lint`
    - `pnpm typecheck`
    - `pnpm test`
    - `pnpm build`
    - `pnpm --filter @datab/sdk-ts test`
    - `pnpm --filter @datab/portal-web lint`
    - `pnpm --filter @datab/portal-web typecheck`
    - `pnpm --filter @datab/portal-web test:unit`
    - `pnpm --filter @datab/portal-web test:e2e`
    - `pnpm --filter @datab/portal-web build`
  - 后端 / 通用：
    - `cargo fmt --all`
    - `cargo check -p platform-core`
    - `cargo test -p platform-core`
    - `cargo sqlx prepare --workspace`
    - `./scripts/check-query-compile.sh`
  - 真实联调与 smoke：
    - `./scripts/check-keycloak-realm.sh`
    - 真实 password grant 获取 `local-platform-admin / platform_admin` Bearer Token，并验证 token claims 含 `user_id / org_id / realm_access.roles`
    - 直连 `platform-core`：
      - `GET /api/v1/recommendations?placement_code=home_featured&subject_scope=organization&subject_org_id=10000000-0000-0000-0000-000000000103&limit=5`
      - `GET /api/v1/catalog/search?q=工业设备运行指标&entity_scope=all&page=1&page_size=4`
      - `GET /api/v1/catalog/standard-scenarios`
    - 门户生产构建启动：
      - `PLATFORM_CORE_BASE_URL=http://127.0.0.1:8080 pnpm --filter @datab/portal-web exec next start --hostname 127.0.0.1 --port 3101`
    - 浏览器端手工 / Playwright smoke：
      - 桌面视口 `1440x1200`：登录态占位注入真实 Bearer，校验主体条、五条标准链路、`home_featured` 推荐位、搜索预览 `total 2` 与浏览器请求只落 `/api/platform/**`
      - 移动视口 `iPhone 13`：校验首页可正常加载、标准链路/推荐位/搜索入口可见、页面无横向溢出
    - 数据库回查：
      - `select placement_code, page_context, status, jsonb_array_length(metadata->'fixed_samples') as fixed_sample_count from recommend.placement_definition where placement_code='home_featured';`
      - `select placement_code, subject_scope, subject_org_id, status, candidate_source_summary->>'placement_sample' as placement_sample, request_attrs->>'candidate_backend' as candidate_backend, created_at from recommend.recommendation_request where placement_code='home_featured' order by created_at desc limit 3;`
      - `select target_type, request_id, accessor_user_id, created_at from audit.access_audit where accessor_user_id='10000000-0000-0000-0000-000000000353' and target_type in ('recommendation_result','search_catalog') order by created_at desc limit 10;`
- 验证结果：
  - `pnpm install --frozen-lockfile`、`pnpm lint`、`pnpm typecheck`、`pnpm test`、`pnpm build` 全部通过；`portal-web` 单体的 lint / typecheck / unit / e2e / build 也全部通过；`sdk-ts` 单测通过，确认绝对 URL 查询拼接回归已收住。
  - `pnpm test` 期间 `console-web` 的 Playwright WebServer 仍会打印若干 `ECONNREFUSED 127.0.0.1:8094` 代理失败日志，但最终 smoke 断言通过；这是工作区现有控制台默认运行时噪音，不影响本批门户首页交付结果。
  - `cargo check -p platform-core`、`cargo test -p platform-core`、`cargo sqlx prepare --workspace` 与 `./scripts/check-query-compile.sh` 全部通过；本批没有引入新的 Rust / SQLx 回归。
  - `./scripts/check-keycloak-realm.sh` 通过，重新确认本地 realm、`portal-web` password grant 与正式 `user_id / org_id / roles` claims 可用。
  - 真实推荐 / 搜索联调通过：
    - `home_featured` 返回 `5` 条官方样例，顺序为 `工业设备运行指标 API 订阅 -> 工业质量与产线日报文件包交付 -> 供应链协同查询沙箱 -> 零售门店经营分析 API / 报告订阅 -> 商圈/门店选址查询服务`
    - `工业设备运行指标` 搜索返回 `total=2`
  - 桌面浏览器 smoke 通过：主体条真实显示 `Local Platform Admin / platform_admin / 10000000-0000-0000-0000-000000000103 / aal1`；`home_featured` 推荐位、标准场景模板 `5 / live`、搜索预览 `total 2` 都成功渲染；捕获到的浏览器端正式 API 请求仅有：
    - `/api/platform/health/ready`
    - `/api/platform/api/v1/catalog/standard-scenarios`
    - `/api/platform/api/v1/recommendations?...`
    - `/api/platform/api/v1/catalog/search?...`
    - `directPlatformCoreRequestCount = 0`
  - 移动浏览器 smoke 通过：标准链路、推荐位、搜索入口与主体条都可见，桌面 / 移动的 `horizontalFit` 均为 `true`；其中桌面横向溢出问题已通过推荐卡 badge 折行修正收敛。
  - 数据库回查与页面行为对齐：
    - `recommend.placement_definition(home_featured)` 当前 `page_context='home'`、`status='active'`、`fixed_sample_count=5`
    - 最近三条 `recommend.recommendation_request` 均为 `subject_scope='organization'`、`subject_org_id='10000000-0000-0000-0000-000000000103'`、`status='served'`、`placement_sample='5'`、`candidate_backend='postgresql_local_minimal'`
    - `audit.access_audit` 中已出现本批首页 Bearer 读取产生的 `recommendation_result` 与 `search_catalog` 访问记录，请求时间与浏览器 smoke 一致
  - 本批未新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`；业务测试数据未向数据库插入额外临时对象，审计访问记录按 append-only 保留。
- 覆盖的冻结文档条目：
  - `v1-core-开发任务清单.csv / .md`：`WEB-003`
  - `页面说明书-V1-完整版.md`：门户首页章节、登录后 / 未登录推荐差异、标准链路与搜索入口说明
  - `商品推荐与个性化发现设计.md`：首页推荐位 `home_featured` 与首页推荐降级语义
  - `数据交易平台-全集成基线-V1.md`：五条标准链路官方命名与八个标准 SKU 口径
  - `search-rec-cases.md`：首页推荐与搜索 Bearer 边界、本地 `home_featured` 固定样例顺序
  - `packages/openapi/catalog.yaml`、`recommendation.yaml`、`search.yaml`、`iam.yaml` 与 `docs/02-openapi/*.yaml`
- 覆盖的任务清单条目：`WEB-003`
- 未覆盖项：
  - 无。`WEB-003` 要求的门户首页场景导航、推荐位、搜索入口、标准链路快捷入口、真实 API 接入、E2E / 浏览器 smoke、数据库回查与日志留痕均已完成；更完整的搜索结果页、商品详情页、卖方主页业务细节由 `WEB-004 / WEB-005 / WEB-006` 继续展开。
- 新增 TODO / 预留项：
  - 无新增 `TODO(V1-gap)` / `TODO(V2-reserved)` / `TODO(V3-reserved)`。
