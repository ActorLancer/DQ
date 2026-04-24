# Order E2E Cases（TEST-006）

## Scope

本文件冻结 `TEST-006` 的正式订单端到端验收基线，严格按五条标准链路命名与执行：

- `S1` 工业设备运行指标 API 订阅
- `S2` 工业质量与产线日报文件包交付
- `S3` 供应链协同查询沙箱
- `S4` 零售门店经营分析 API / 报告订阅
- `S5` 商圈/门店选址查询服务

当前任务使用 `fixtures/demo/scenarios.json + orders.json` 中已冻结的正式 demo 商品、SKU 和主订单实例作为唯一真值源。`TEST-006` 不再临时创建一批漂移订单来跑浏览器联调，而是复用 `TEST-002` 导入的 5 条主订单蓝图，确保本地和 CI 可重复执行且不会积累额外业务测试数据。

## Invariants

- 五条链路名称、主 SKU / 补充 SKU、合同 / 验收 / 退款模板必须与 `fixtures/demo/*.json` 和 `数据交易平台-全集成基线-V1.md 5.3.2 / 5.3.2A` 一致。
- 浏览器端只允许访问 `portal-web -> /api/platform -> platform-core`，`restrictedRequests` 必须保持为空。
- 本任务的正式 E2E 必须真实经过：
  - Keycloak / IAM password grant
  - `local-buyer-operator` 门户主体 + `local-tenant-developer` developer trace 主体
  - 门户首页或标准演示页
  - 搜索页
  - 商品详情页
  - 下单页
  - 订单详情页
  - 场景对应交付页
  - 验收页
  - 后端 `order detail / lifecycle / developer trace` 回查
- 宿主机地址边界继续固定为：
  - `platform-core`: `127.0.0.1:8094`
  - Kafka host: `127.0.0.1:9094`

## Matrix

| 用例ID | 标准链路 | 正式商品 / 订单基线 | 门户链路 | 后端回查 | 自动化证据 |
| --- | --- | --- | --- | --- | --- |
| `TEST006-S1` | 工业设备运行指标 API 订阅 | `product_id=20000000-0000-0000-0000-000000000309` / `order_id=34000000-0000-0000-0000-000000000001` | `/demos/S1 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/api -> /delivery/orders/{id}/acceptance` | `GET /api/v1/orders/{id}`、`GET /api/v1/orders/{id}/lifecycle-snapshots`、`GET /api/v1/developer/trace?order_id={id}` | `apps/portal-web/e2e/test006-standard-order-live.spec.ts` |
| `TEST006-S2` | 工业质量与产线日报文件包交付 | `product_id=20000000-0000-0000-0000-000000000310` / `order_id=34000000-0000-0000-0000-000000000002` | `/demos/S2 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/file -> /delivery/orders/{id}/acceptance` | 同上 | 同上 |
| `TEST006-S3` | 供应链协同查询沙箱 | `product_id=20000000-0000-0000-0000-000000000311` / `order_id=34000000-0000-0000-0000-000000000003` | `/demos/S3 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/sandbox -> /delivery/orders/{id}/acceptance` | 同上 | 同上 |
| `TEST006-S4` | 零售门店经营分析 API / 报告订阅 | `product_id=20000000-0000-0000-0000-000000000312` / `order_id=34000000-0000-0000-0000-000000000004` | `/demos/S4 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/api -> /delivery/orders/{id}/acceptance` | 同上 | 同上 |
| `TEST006-S5` | 商圈/门店选址查询服务 | `product_id=20000000-0000-0000-0000-000000000313` / `order_id=34000000-0000-0000-0000-000000000005` | `/demos/S5 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/template-query -> /delivery/orders/{id}/acceptance` | 同上 | 同上 |

## Required Assertions

每条链路至少断言以下事实：

1. 门户页面显示官方场景名、主 SKU、补充 SKU、合同模板、验收模板和退款模板，不出现别名或第二套命名。
2. 搜索页和商品详情页真实返回该链路对应商品，而不是 preview mock 数据。
3. 下单页真实承接当前商品 / SKU / `scenario_code`，并提示 `X-Idempotency-Key`。
4. 订单详情页真实展示 `scenario_code`、`current_state`、`payment_status`、`delivery_status` 和“审计与链路信任边界”。
5. 场景对应交付页真实展示该 SKU 对应的正式入口，并与 `local-buyer-operator` 权限态一致：
   - `S1 / S4`: 展示 `API 开通表单`
   - `S2`: 展示 `文件交付` 正式入口，且显式出现 `主按钮权限不足`
   - `S3`: 展示 `沙箱工作区开通`
   - `S5`: 展示 `模板查询授权`
6. 验收页真实展示“交付结果摘要”和“生命周期摘要”。
7. 后端回查必须命中：
   - `code=OK`
   - `request_id`
   - 正确 `order_id`
   - 正确 `scenario_code`
   - 订单当前 `current_state / payment_status / delivery_status / acceptance_status / settlement_status`
8. 浏览器请求不允许直连 `5432 / 6379 / 7050 / 7051 / 8080 / 8094 / 9000 / 9092 / 9094 / 9200 / 9300 / 18080` 或 `postgres / kafka / opensearch / redis / fabric` 主机名。

## Execution Commands

正式 checker：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh
```

如只想单独重跑门户 live spec：

```bash
WEB_E2E_LIVE=1 \
WEB_E2E_PORTAL_USERNAME=local-buyer-operator \
WEB_E2E_PORTAL_PASSWORD=LocalBuyerOperator123! \
WEB_E2E_TRACE_USERNAME=local-tenant-developer \
WEB_E2E_TRACE_PASSWORD=LocalTenantDeveloper123! \
PLATFORM_CORE_BASE_URL=http://127.0.0.1:8094 \
pnpm --filter @datab/portal-web test:e2e:orders-live
```

## Traceability

- 任务定义：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 场景 authority：`docs/全集成文档/数据交易平台-全集成基线-V1.md`
- 正式 demo 基线：`fixtures/demo/scenarios.json`、`fixtures/demo/orders.json`
- 门户场景引导：`apps/portal-web/src/lib/standard-demo.ts`
- 门户 order live E2E：`apps/portal-web/e2e/test006-standard-order-live.spec.ts`
- 官方 checker：`scripts/check-order-e2e.sh`
