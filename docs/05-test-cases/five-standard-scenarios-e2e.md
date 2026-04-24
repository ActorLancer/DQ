# Five Standard Scenarios E2E

`TEST-022` 的正式目标，是把首批五条标准链路写成可顺序执行、可回归复用的 E2E 规格卡片。本文不替代 `TEST-006` 的官方 checker；它为后续 `TEST-023 / 024`、最终 `V1` sign-off 和人工复核提供统一的场景输入、目标状态与验证点。

## Authority

- 任务定义：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 标准链路与 SKU 映射：`docs/全集成文档/数据交易平台-全集成基线-V1.md` `5.3.2 / 5.3.2A`
- Phase 1 验收标准：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md` `15.2`
- 官方 E2E checker：`docs/05-test-cases/order-e2e-cases.md`
- Demo 真值源：`fixtures/demo/scenarios.json`、`fixtures/demo/orders.json`、`fixtures/demo/delivery.json`、`fixtures/demo/billing.json`

## Global Preconditions

按正式口径执行五条链路前，必须先完成：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/smoke-local.sh
./scripts/seed-local-iam-test-identities.sh
./scripts/seed-demo.sh --skip-base-seeds
./scripts/check-demo-seed.sh
./scripts/check-keycloak-realm.sh
```

宿主机边界继续固定为：

- `platform-core`: `http://127.0.0.1:8094`
- Kafka host: `127.0.0.1:9094`

官方自动化入口仍以 `TEST-006` 为准：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-order-e2e.sh
```

## Shared Validation Points

每条链路都必须至少覆盖以下共性检查：

1. 门户页面显示官方场景名、主 SKU、补充 SKU、合同模板、验收模板、退款模板。
2. 搜索页与商品详情页真实命中该场景商品，不是 preview/mock 数据。
3. 下单页真实承接 fixture 中的 `scenario_code`、`request_id`、`idempotency_key` 语义。
4. 订单详情页真实展示 `current_state / payment_status / delivery_status / acceptance_status / settlement_status / dispute_status`。
5. 场景对应交付页展示正确的正式交付入口。
6. 后端 `GET /api/v1/orders/{id}`、`GET /api/v1/orders/{id}/lifecycle-snapshots`、`GET /api/v1/developer/trace?order_id={id}` 回查通过。
7. 浏览器请求不得直连 `Kafka / PostgreSQL / Redis / OpenSearch / Fabric / MinIO`。
8. 审计、交付对象和计费样本必须能回查到 fixture 对应 ID，不允许只看页面文案。

## Scenario Summary

| 场景 | 主 SKU | 补充 SKU | 主订单 | 补充订单 | 主交付对象 | 目标终态 |
| --- | --- | --- | --- | --- | --- | --- |
| `S1` 工业设备运行指标 API 订阅 | `API_SUB` | `API_PPU` | `34000000-0000-0000-0000-000000000001` | `34000000-0000-0000-0000-000000000101` | `api_access` | `active / paid / completed / accepted / pending_cycle / none` |
| `S2` 工业质量与产线日报文件包交付 | `FILE_STD` | `FILE_SUB` | `34000000-0000-0000-0000-000000000002` | `34000000-0000-0000-0000-000000000102` | `encrypted_file_package` | `accepted / paid / completed / accepted / ready_for_settlement / none` |
| `S3` 供应链协同查询沙箱 | `SBX_STD` | `SHARE_RO` | `34000000-0000-0000-0000-000000000003` | `34000000-0000-0000-0000-000000000103` | `sandbox_workspace` | `active / paid / completed / accepted / pending / none` |
| `S4` 零售门店经营分析 API / 报告订阅 | `API_SUB` | `RPT_STD` | `34000000-0000-0000-0000-000000000004` | `34000000-0000-0000-0000-000000000104` | `api_access` | `active / paid / completed / accepted / pending_cycle / none` |
| `S5` 商圈/门店选址查询服务 | `QRY_LITE` | `RPT_STD` | `34000000-0000-0000-0000-000000000005` | `34000000-0000-0000-0000-000000000105` | `template_query_grant + query_result_artifact` | `accepted / paid / completed / accepted / ready_for_settlement / none` |

## Scenario Cards

### `S1` 工业设备运行指标 API 订阅

**输入数据**

- 商品：`product_id=20000000-0000-0000-0000-000000000309`
- 主 SKU：`API_SUB` `sku_id=20000000-0000-0000-0000-000000000409`
- 补充 SKU：`API_PPU` `sku_id=20000000-0000-0000-0000-000000000410`
- 主订单：`order_id=34000000-0000-0000-0000-000000000001`
- 补充订单：`order_id=34000000-0000-0000-0000-000000000101`
- 请求锚点：`request_id=req_test001_s1_api_sub_001`、`idempotency_key=test001-s1-api-sub-001`
- 模板：`CONTRACT_API_SUB_V1 / ACCEPT_API_SUB_V1 / REFUND_API_SUB_V1`
- 交付对象：`41000000-0000-0000-0000-000000000001(api_access)`、`41000000-0000-0000-0000-000000000002(api_metered_access)`
- 计费样本：`42000000-0000-0000-0000-000000000001 / 0002`

**期望状态**

- 主订单：`current_state=active`、`payment_status=paid`、`delivery_status=completed`、`acceptance_status=accepted`、`settlement_status=pending_cycle`、`dispute_status=none`
- 补充订单：`current_state=active`、`settlement_status=metering_open`

**验证点**

1. 门户路径：`/demos/S1 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/api -> /delivery/orders/{id}/acceptance`
2. 交付页必须展示 API 开通表单，不允许回退到文件下载或沙箱入口。
3. 订单详情与 lifecycle snapshot 必须出现 `scenario_code=S1` 与 `API_SUB` 主路径信息。
4. `developer/trace` 必须能回查到 API 访问与授权链路。
5. 补充 SKU `API_PPU` 仅作为加购/计量回查，不得替代 `API_SUB` 主路径。

### `S2` 工业质量与产线日报文件包交付

**输入数据**

- 商品：`product_id=20000000-0000-0000-0000-000000000310`
- 主 SKU：`FILE_STD` `sku_id=20000000-0000-0000-0000-000000000411`
- 补充 SKU：`FILE_SUB` `sku_id=20000000-0000-0000-0000-000000000412`
- 主订单：`order_id=34000000-0000-0000-0000-000000000002`
- 补充订单：`order_id=34000000-0000-0000-0000-000000000102`
- 请求锚点：`request_id=req_test001_s2_file_std_001`、`idempotency_key=test001-s2-file-std-001`
- 模板：`CONTRACT_FILE_V1 / ACCEPT_FILE_V1 / REFUND_FILE_V1`
- 交付对象：`41000000-0000-0000-0000-000000000003(encrypted_file_package)`、`41000000-0000-0000-0000-000000000004(revision_subscription)`
- 计费样本：`42000000-0000-0000-0000-000000000003 / 0004`

**期望状态**

- 主订单：`accepted / paid / completed / accepted / ready_for_settlement / none`
- 补充订单：`active / paid / completed / accepted / cycle_open / none`

**验证点**

1. 门户路径：`/demos/S2 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/file -> /delivery/orders/{id}/acceptance`
2. 交付页必须展示文件交付正式入口，并显式体现按钮权限态或下载票据态。
3. 后端回查必须命中文件票据 / storage object / delivery ticket 相关对象。
4. 主路径必须以一次性交付 `FILE_STD` 为准；`FILE_SUB` 仅作为周期文件订阅补充证据。
5. 验收页必须展示文件交付摘要和正式 acceptance 模板信息。

### `S3` 供应链协同查询沙箱

**输入数据**

- 商品：`product_id=20000000-0000-0000-0000-000000000311`
- 主 SKU：`SBX_STD` `sku_id=20000000-0000-0000-0000-000000000413`
- 补充 SKU：`SHARE_RO` `sku_id=20000000-0000-0000-0000-000000000414`
- 主订单：`order_id=34000000-0000-0000-0000-000000000003`
- 补充订单：`order_id=34000000-0000-0000-0000-000000000103`
- 请求锚点：`request_id=req_test001_s3_sbx_std_001`、`idempotency_key=test001-s3-sbx-std-001`
- 模板：`CONTRACT_SANDBOX_V1 / ACCEPT_SANDBOX_V1 / REFUND_SANDBOX_V1`
- 交付对象：`41000000-0000-0000-0000-000000000005(sandbox_workspace)`、`41000000-0000-0000-0000-000000000006(share_grant)`
- 计费样本：`42000000-0000-0000-0000-000000000005 / 0006`

**期望状态**

- 主订单：`active / paid / completed / accepted / pending / none`
- 补充订单：`active / paid / completed / accepted / pending / none`

**验证点**

1. 门户路径：`/demos/S3 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/sandbox -> /delivery/orders/{id}/acceptance`
2. 交付页必须展示沙箱工作区开通，而不是直接文件或 API 入口。
3. 回查 `sandbox_workspace / sandbox_session / share_grant` 相关对象与 developer trace。
4. 补充 SKU `SHARE_RO` 必须保持独立对象事实，不得并入沙箱主路径。
5. 后续 `TEST-012` 必须继续用该场景验证 share/sandbox revoke。

### `S4` 零售门店经营分析 API / 报告订阅

**输入数据**

- 商品：`product_id=20000000-0000-0000-0000-000000000312`
- 主 SKU：`API_SUB` `sku_id=20000000-0000-0000-0000-000000000415`
- 补充 SKU：`RPT_STD` `sku_id=20000000-0000-0000-0000-000000000416`
- 主订单：`order_id=34000000-0000-0000-0000-000000000004`
- 补充订单：`order_id=34000000-0000-0000-0000-000000000104`
- 请求锚点：`request_id=req_test001_s4_api_sub_001`、`idempotency_key=test001-s4-api-sub-001`
- 模板：`CONTRACT_API_SUB_V1 / ACCEPT_API_SUB_V1 / REFUND_API_SUB_V1`
- 交付对象：`41000000-0000-0000-0000-000000000007(api_access)`、`41000000-0000-0000-0000-000000000008(report_artifact)`
- 计费样本：`42000000-0000-0000-0000-000000000007 / 0008`

**期望状态**

- 主订单：`active / paid / completed / accepted / pending_cycle / none`
- 补充订单：`accepted / paid / completed / accepted / ready_for_settlement / none`

**验证点**

1. 门户路径：`/demos/S4 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/api -> /delivery/orders/{id}/acceptance`
2. 主路径必须保持 `API_SUB`；补充 `RPT_STD` 只作为报告交付和签收证据。
3. 后端回查必须能关联 `api_access` 与 `report_artifact` 两类交付对象。
4. 页面与 API 返回必须展示 `S4` 的正式场景名，而不是 `S1` 的 API 订阅别名。
5. 后续 `TEST-023` 需要把 `RPT_STD` 的异常/退款证据补齐到该场景。

### `S5` 商圈/门店选址查询服务

**输入数据**

- 商品：`product_id=20000000-0000-0000-0000-000000000313`
- 主 SKU：`QRY_LITE` `sku_id=20000000-0000-0000-0000-000000000417`
- 补充 SKU：`RPT_STD` `sku_id=20000000-0000-0000-0000-000000000418`
- 主订单：`order_id=34000000-0000-0000-0000-000000000005`
- 补充订单：`order_id=34000000-0000-0000-0000-000000000105`
- 请求锚点：`request_id=req_test001_s5_qry_lite_001`、`idempotency_key=test001-s5-qry-lite-001`
- 模板：`CONTRACT_QUERY_LITE_V1 / ACCEPT_QUERY_LITE_V1 / REFUND_QUERY_LITE_V1`
- 交付对象：`41000000-0000-0000-0000-000000000009(template_query_grant)`、`41000000-0000-0000-0000-000000000010(query_result_artifact)`、`41000000-0000-0000-0000-000000000011(report_artifact)`
- 计费样本：`42000000-0000-0000-0000-000000000009 / 0010`

**期望状态**

- 主订单：`accepted / paid / completed / accepted / ready_for_settlement / none`
- 补充订单：`delivered / paid / completed / pending / pending_acceptance / none`

**验证点**

1. 门户路径：`/demos/S5 -> /search -> /products/{id} -> /trade/orders/new -> /trade/orders/{id} -> /delivery/orders/{id}/template-query -> /delivery/orders/{id}/acceptance`
2. 主路径必须展示模板查询授权与结果摘要，不得回退到通用文件下载页。
3. 主订单必须能回查 `template_query_grant + query_result_artifact`；补充订单必须能回查 `report_artifact`。
4. `S5` 是 `评分闭环` 的正式落点，后端结果必须继续回 PostgreSQL 与审计放行。
5. 补充 `RPT_STD` 当前处于 `delivered / pending_acceptance`，后续 `TEST-024` 必须用它覆盖“交付完成 -> 待验收”编排链路。

## Execution Notes

- 本文用于顺序执行与回归复核；真正自动化入口仍是 `TEST-006` 的 `check-order-e2e.sh`。
- 若需要补充 scenario-specific live order 运行记录，应把新增 `order_id` 追加到 `TEST-021` 的 `20+ orders` sign-off 记录中。
- 本文与 `order-e2e-cases.md` 冲突时，以本文件的 fixture 输入和目标状态为准；页面路径、浏览器边界与自动化入口继续以 `order-e2e-cases.md` 为准。
