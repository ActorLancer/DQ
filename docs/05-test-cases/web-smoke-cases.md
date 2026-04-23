# Web Smoke Cases

## Scope

本文件冻结 `WEB` 阶段当前已落地页面的最小 smoke 基线，覆盖：

- `portal-web` 门户主链路：首页、标准演示入口、搜索、卖方主页、商品详情、供方侧上架入口、下单、订单详情、交付、验收、账单、争议、门户开发者入口。
- `console-web` 控制台主链路：控制台首页、审核台、审计联查、证据包导出、一致性联查、Outbox / Dead Letter、搜索运维、控制台开发者入口。
- 浏览器受控边界：前端只能访问 `/api/platform/**`，不得直连 `PostgreSQL / Kafka / OpenSearch / Redis / Fabric / MinIO / platform-core`。
- `WEB-018` 已落地的 live E2E：Keycloak / IAM Bearer 登录后，门户与控制台的真实前后端联调链路。

当前范围明确不包含：

- `WEB-022` 通知联查页。通知链路的 worker / topic / DB / 审计验收仍以 [notification-cases.md](./notification-cases.md) 为准，待 `WEB-022` 页面落地后再并入本文件。

## Invariants

- 页面命名、路径和链路顺序必须与《页面说明书》中的“页面间路由关系”和当前 `portal-routes / console-routes` 一致，不得自创别名。
- 错误态必须显示正式错误码，并保留 `request_id` 或等价联调主键；前端不得退化为只显示裸 message。
- 敏感页必须可见当前主体、角色、租户、作用域。
- 写操作页面必须体现 `Idempotency-Key` 要求；高风险动作必须体现 step-up / 审计提示。
- Playwright 浏览器请求中，直连受限系统或 `platform-core` 宿主机地址的次数必须为 `0`。
- 八个标准 SKU 与五条标准链路名称必须保持冻结口径：
  - SKU：`FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`
  - 场景：`S1 工业设备运行指标 API 订阅`、`S2 工业质量与产线日报文件包交付`、`S3 供应链协同查询沙箱`、`S4 零售门店经营分析 API / 报告订阅`、`S5 商圈/门店选址查询服务`

## Matrix

| 用例ID | 范围 | 路由 / 触发 | 预期结果 | 自动化证据 |
| --- | --- | --- | --- | --- |
| `WEB-SMOKE-001` | 门户首页与标准链路入口 | `GET /`，逐个点击 `查看 S1~S5 演示路径` | 首页显示场景导航、推荐位、受控搜索入口和主体条；五条标准链路都能跳到 `/demos/S1~S5`，页面展示官方场景名、`GET /api/v1/catalog/standard-scenarios` 和 `Idempotency-Key` 提示 | `apps/portal-web/e2e/smoke.spec.ts` `portal home links directly to five standard demo paths` |
| `WEB-SMOKE-002` | 门户目录链路状态面 | `/search?preview=forbidden|empty|error`、`/products/:id?preview=forbidden|empty`、`/sellers/:orgId?preview=forbidden|empty|error` | 搜索、商品详情、卖方主页分别覆盖权限态、空态、错态；错误态保留正式错误码如 `SEARCH_BACKEND_UNAVAILABLE`、`CAT_VALIDATION_FAILED` | `apps/portal-web/e2e/smoke.spec.ts` `portal home and scaffold pages are reachable` |
| `WEB-SMOKE-003` | 供方侧上架入口 | `/seller/products?preview=forbidden|empty|error`、`/seller/products/:productId/skus?preview=forbidden` | 上架中心与 SKU 配置页可访问；权限态展示正式权限点 `catalog.product.list / catalog.sku.create`；空态与错态存在 | `apps/portal-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-004` | 交易链路入口 | `/trade/orders/new?preview=forbidden|empty|error`、`/trade/orders/:orderId?preview=forbidden|empty|error` | 下单页显示五条标准链路和官方场景名；订单详情覆盖权限态、空态、`TRD_STATE_CONFLICT` 错态 | `apps/portal-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-005` | 交付中心矩阵 | `/delivery/orders/:orderId/file|api|share|template-query|sandbox|report` 的 `preview` 组合 | 文件交付、API 开通、共享、模板授权、沙箱、报告交付页面都可访问；`FILE_STD / QRY_LITE / RPT_STD` 等官方 SKU 名明确可见；错误态使用 `DELIVERY_STATUS_INVALID` 等正式口径 | `apps/portal-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-006` | 验收、账单、争议 | `/delivery/orders/:orderId/acceptance?preview=forbidden|empty|error`、`/billing?preview=forbidden|empty|error`、`/billing/refunds?preview=forbidden|empty`、`/support/cases/new?preview=forbidden|empty|error` | 验收页、账单页、退款/赔付入口、争议页覆盖权限态、空态、错态；高风险动作提示 step-up / 审计；错误态分别保留 `DELIVERY_STATUS_INVALID`、`BIL_PROVIDER_FAILED`、`DISPUTE_STATUS_INVALID` | `apps/portal-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-007` | 门户开发者入口 | `/developer`、`/developer/apps`、`/developer/trace`、`/developer/assets` | 门户开发者工作台、应用管理、trace 联查、Mock 支付入口可访问；显示主体上下文、`request_id`、`Idempotency-Key` 与 `developer.mock_payment.simulate` | `apps/portal-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-008` | 门户用户态链路 | 写入本地 buyer / Bearer 会话后访问 `/`、`/search`、`/products/:id`、`/trade/orders/new`、`/trade/orders/:orderId`、`/delivery/orders/:orderId/file|acceptance`、`/support/cases/new`、`/developer/trace` | 登录态占位、本地 buyer 会话和 Bearer claims 可正常切换；主体、角色、租户、作用域可见；空态链路可顺序访问，且不触发受限系统直连 | `apps/portal-web/e2e/smoke.spec.ts` `WEB-018 portal user flow covers login, search, product, order, delivery, acceptance and linkage` |
| `WEB-SMOKE-009` | 控制台首页与审核台 | `/`、`/ops/review/subjects?preview=empty`、`/ops/review/products` | 控制台首页显示控制面登录态和最小联调摘要；主体审核台、产品审核台可访问并展示官方审核 API 绑定 | `apps/console-web/e2e/smoke.spec.ts` `console home and scaffold pages are reachable` |
| `WEB-SMOKE-010` | 控制台审计 / 一致性 / 搜索运维 | `/ops/audit/trace`、`/ops/audit/packages?preview=forbidden`、`/ops/consistency`、`/ops/consistency/outbox`、`/ops/search` | 审计联查、证据包导出、一致性联查、Outbox / Dead Letter、搜索运维页面可访问；页面显式展示 `request_id / tx_hash / 链状态 / 投影状态 / dry-run / X-Step-Up-Token` 等关键字段和提示 | `apps/console-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-011` | 控制台开发者入口 | `/developer`、`/developer/apps`、`/developer/trace`、`/developer/assets` | 控制台开发者首页、测试应用、状态联查、测试资产页面可访问；显示 `Idempotency-Key`、`request_id`、`developer.mock_payment.simulate` | `apps/console-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-012` | 控制台控制面链路 | 本地平台管理员会话后访问 `/ops/audit/trace`、`/developer/trace?order_id=...`、`/ops/consistency`、`/ops/search` | 控制台链路可联查 `request_id / tx_hash / PostgreSQL 真值 / Kafka-outbox / OpenSearch / Redis / Fabric` 五类官方信任边界，不触发受限系统直连 | `apps/console-web/e2e/smoke.spec.ts` `WEB-018 console control-plane flow covers login, audit, ops and developer linkage` |
| `WEB-SMOKE-013` | 浏览器受控边界 | 所有 portal / console smoke | `restrictedRequests=[]`；浏览器不得直接访问 `5432 / 6379 / 7050 / 7051 / 8080 / 8094 / 9000 / 9092 / 9094 / 9200 / 9300 / 18080` 或 `postgres / kafka / opensearch / redis / fabric` 主机名 | `apps/portal-web/e2e/smoke.spec.ts`、`apps/console-web/e2e/smoke.spec.ts` |
| `WEB-SMOKE-014` | 门户 live E2E | `WEB_E2E_LIVE=1 playwright test e2e/web018-live.spec.ts` | 通过 Keycloak / IAM Bearer 登录门户，真实联调搜索、商品详情、下单、订单详情、交付、验收和 developer trace；浏览器仍不直连受限系统 | `apps/portal-web/e2e/web018-live.spec.ts` |
| `WEB-SMOKE-015` | 控制台 live E2E | `WEB_E2E_LIVE=1 playwright test e2e/web018-live.spec.ts` | 通过 Keycloak / IAM Bearer 登录控制台，真实联调审计联查、developer trace、一致性联查和搜索运维；浏览器仍不直连受限系统 | `apps/console-web/e2e/web018-live.spec.ts` |

## Manual Smoke Baseline

最小手工 smoke 至少覆盖以下 8 步：

1. 打开门户首页 `/`，确认可见“首页”“标准链路快捷入口”“受控搜索入口”“当前主体 / 角色 / 租户 / 作用域”。
2. 从首页依次进入 `/demos/S1 ~ /demos/S5`，确认五条标准链路官方命名、主 SKU / 补充 SKU 和 `Idempotency-Key` 提示不漂移。
3. 校验门户目录链路的三类状态面：
   - `/search?preview=forbidden|empty|error`
   - `/products/:id?preview=forbidden|empty`
   - `/sellers/:orgId?preview=forbidden|empty|error`
4. 校验交易与交付链路：
   - `/trade/orders/new?preview=forbidden|empty|error`
   - `/trade/orders/:orderId?preview=forbidden|empty|error`
   - `/delivery/orders/:orderId/file|api|template-query|sandbox|acceptance` 的 `preview` 组合
5. 校验账单与争议：
   - `/billing?preview=forbidden|empty|error`
   - `/billing/refunds?preview=forbidden|empty`
   - `/support/cases/new?preview=forbidden|empty|error`
6. 校验门户开发者入口：`/developer`、`/developer/apps`、`/developer/trace`、`/developer/assets`。
7. 校验控制台链路：`/`、`/ops/review/products`、`/ops/audit/trace`、`/ops/consistency`、`/ops/consistency/outbox`、`/ops/search`、`/developer/*`。
8. 如本地具备 Keycloak / platform-core 联调条件，再执行 live smoke：
   - `pnpm --filter @datab/portal-web test:e2e:live`
   - `pnpm --filter @datab/console-web test:e2e:live`

## Execution Commands

最小自动化执行顺序：

```bash
pnpm --filter @datab/portal-web test:e2e
pnpm --filter @datab/console-web test:e2e
pnpm test
pnpm build
```

需要真实 Keycloak / platform-core 联调时再追加：

```bash
WEB_E2E_LIVE=1 pnpm --filter @datab/portal-web test:e2e:live
WEB_E2E_LIVE=1 pnpm --filter @datab/console-web test:e2e:live
```

## Traceability

- 页面链路与覆盖口径：`docs/页面说明书/页面说明书-V1-完整版.md`
- Web 任务顺序与完成定义：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 门户路由与 API 绑定：`apps/portal-web/src/lib/portal-routes.ts`
- 控制台路由与 API 绑定：`apps/console-web/src/lib/console-routes.ts`
- 门户 smoke / live E2E：`apps/portal-web/e2e/smoke.spec.ts`、`apps/portal-web/e2e/web018-live.spec.ts`
- 控制台 smoke / live E2E：`apps/console-web/e2e/smoke.spec.ts`、`apps/console-web/e2e/web018-live.spec.ts`
