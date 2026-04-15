# 接口清单与 OpenAPI/Schema 冻结表

## 1. 文档定位

本文件用于冻结当前阶段平台接口清单、接口归属、公共请求头、统一响应体、关键对象 schema 和版本边界。

本文件服务于以下工作：

- 前后端联调
- OpenAPI 细化
- 鉴权中间件实现
- 外围进程接口实现
- 回调验签与幂等处理
- 测试用例矩阵拆解

本文件不重复展开业务规则，仅冻结接口面。

阅读入口约束：

- 当前阶段的接口冻结以 `../全集成文档/数据交易平台-全集成基线-V1.md` 为主阅读入口。
- 本文件只冻结 `V1` 正式接口面；`V2/V3` 在本文件中仅保留必要边界，不保留实现级接口正文。

上位文档：

- [数据交易平台-全集成基线-V1.md](../全集成文档/数据交易平台-全集成基线-V1.md)
- [服务清单与服务边界正式版.md](../开发准备/服务清单与服务边界正式版.md)
- [目录与商品接口协议正式版.md](../数据库设计/接口协议/目录与商品接口协议正式版.md)
- [支付域接口协议正式版.md](../数据库设计/接口协议/支付域接口协议正式版.md)
- [一致性与事件接口协议正式版.md](../数据库设计/接口协议/一致性与事件接口协议正式版.md)
- [审计、证据链与回放接口协议正式版.md](../数据库设计/接口协议/审计、证据链与回放接口协议正式版.md)
- [身份与会话接口协议正式版.md](../数据库设计/接口协议/身份与会话接口协议正式版.md)
- [接口权限校验清单.md](../权限设计/接口权限校验清单.md)

## 2. 冻结原则

### 2.1 当前接口风格

当前正式采用：

- `REST + JSON`
- 明确版本前缀
- 同步请求返回业务受理结果
- 异步副作用通过事件和回调推进

当前不采用：

- GraphQL 作为主接口
- gRPC 作为前后端主接口
- 无版本前缀的散装接口

### 2.2 Base Path

- `V1`: `/api/v1`
- `V2`: `/api/v2`
- `V3`: `/api/v3`

### 2.3 接口归属规则

- `portal-web / console-web` 只调用 `platform-core` 对外 API
- 外围进程优先消费 Kafka 事件或调用受控内部接口
- 不允许前端直连 `Kafka / PostgreSQL / OpenSearch / Redis / Fabric`

### 2.4 命名规则

- 资源名使用复数路径
- 状态字段使用对象化语义字段名
- 对外逻辑字段优先，不直接暴露数据库列名

示例：

- `current_state`，不直接暴露无语义 `status`
- `order_amount`，不直接暴露无语义 `amount`
- `metered_quantity`，不直接暴露 `units`
- `delivery_mode / pricing_mode`，不直接暴露持久化层 `delivery_type / price_mode`

## 3. 公共请求与响应

## 3.1 公共请求头

所有用户态和控制台态接口统一支持：

- `Authorization: Bearer <access_token>`
- `X-Request-Id`

建议支持：

- `X-Idempotency-Key`
- `X-Trace-Id`
- `X-Step-Up-Token`

说明：

- `X-Step-Up-Token` 用于退款、赔付、人工打款、审计导出、重放、重建索引、切换别名等高风险动作。

## 3.2 成功响应

```json
{
  "code": "OK",
  "message": "success",
  "request_id": "req_01J...",
  "data": {}
}
```

## 3.3 失败响应

```json
{
  "code": "SKU_TYPE_INVALID",
  "message": "invalid sku_type",
  "request_id": "req_01J...",
  "details": {}
}
```

## 3.4 分页协议

列表接口统一采用：

```json
{
  "code": "OK",
  "message": "success",
  "request_id": "req_01J...",
  "data": {
    "items": [],
    "page": 1,
    "page_size": 20,
    "total": 200,
    "has_more": true
  }
}
```

## 4. 关键对象 Schema 冻结

## 4.1 Product

```json
{
  "product_id": "uuid",
  "product_type": "data_product",
  "product_status": "listed",
  "title": "string",
  "delivery_mode": "file_download",
  "usage_mode": "download_and_internal_use",
  "sku_template_id": "uuid",
  "metadata_profile": {},
  "seller_party_id": "uuid"
}
```

冻结规则：

- `Product` 是目录、审核、搜索、详情展示事实源
- `delivery_mode` 是默认值或聚合展示值
- 不得把 `Product` 当交易快照事实源

## 4.2 ProductSKU

```json
{
  "sku_id": "uuid",
  "product_id": "uuid",
  "sku_type": "API_SUB",
  "sku_code": "api-enterprise-monthly",
  "allowed_rights": ["access", "result_get"],
  "forbidden_rights": [],
  "delivery_mode": "api_access",
  "pricing_mode": "subscription",
  "rights_profile_code": "access_result_get",
  "quota_json": {},
  "sla_json": {}
}
```

冻结规则：

- `sku_type` 是标准 SKU 真值
- `sku_code` 是实现型商业套餐编码
- `ProductSKU` 是下单、合同、授权、交付、验收、账单、结算事实源

## 4.3 OrderSummary

```json
{
  "order_id": "uuid",
  "current_state": "payment_succeeded",
  "payment_status": "succeeded",
  "delivery_status": "in_progress",
  "acceptance_status": "pending",
  "settlement_status": "pending",
  "dispute_status": "none",
  "order_amount": "1200.00",
  "currency_code": "USD"
}
```

## 4.4 LifecycleSnapshot

```json
{
  "authorization_status": "active",
  "delivery_status": "completed",
  "settlement_status": "pending",
  "dispute_status": "none",
  "authority_model": "business_authoritative",
  "proof_commit_policy": "required_async",
  "proof_commit_state": "submitted",
  "external_fact_status": "confirmed",
  "reconcile_status": "matched",
  "status_version": 8
}
```

## 4.5 PaymentIntent

```json
{
  "payment_intent_id": "uuid",
  "order_id": "uuid",
  "payment_amount": "1200.00",
  "currency_code": "USD",
  "payment_status": "requires_action",
  "corridor_status": "active",
  "provider_code": "mock_provider"
}
```

## 4.6 SearchResultItem

```json
{
  "entity_scope": "product",
  "entity_id": "uuid",
  "title": "string",
  "product_type": "data_product",
  "seller_party_id": "uuid",
  "delivery_modes": ["api_access"],
  "pricing_modes": ["subscription"],
  "rank_score": 0.92
}
```

## 5. V1 主接口清单

## 5.1 身份与会话

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/auth/register` | `iam` | 主体/成员注册 |
| `POST /api/v1/auth/login` | `iam` | 本地登录 |
| `POST /api/v1/auth/logout` | `iam` | 登出 |
| `POST /api/v1/auth/mfa/verify` | `iam` | MFA 校验 |
| `GET /api/v1/sessions/current` | `iam` | 当前会话查询 |
| `POST /api/v1/sessions/{id}/revoke` | `iam` | 会话撤销 |

## 5.2 主体、组织、应用

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/orgs/register` | `party` | 组织注册 |
| `POST /api/v1/users/invite` | `party` | 邀请成员 |
| `POST /api/v1/apps` | `party` | 创建应用 |
| `PATCH /api/v1/apps/{id}` | `party` | 更新应用 |
| `GET /api/v1/apps/{id}` | `party` | 查看应用 |

## 5.3 商品与目录

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/products` | `catalog` | 创建商品 |
| `PATCH /api/v1/products/{id}` | `catalog` | 编辑商品 |
| `GET /api/v1/products/{id}` | `catalog` | 查看商品详情 |
| `POST /api/v1/products/{id}/submit` | `listing` | 提交审核 |
| `POST /api/v1/products/{id}/suspend` | `listing` | 下架/冻结商品 |
| `GET /api/v1/catalog/search` | `search` | 搜索商品、服务、卖方 |
| `GET /api/v1/sellers/{orgId}/profile` | `catalog` | 查看卖方主页 |

## 5.4 SKU 与模板绑定

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/products/{id}/skus` | `catalog` | 创建 SKU |
| `PATCH /api/v1/skus/{id}` | `catalog` | 更新 SKU |
| `POST /api/v1/products/{id}/bind-template` | `listing` | 商品级模板族默认绑定 |
| `POST /api/v1/skus/{id}/bind-template` | `listing` | SKU 级最终模板绑定 |
| `GET /api/v1/templates/{id}` | `listing` | 模板查看 |

## 5.5 元信息、数据契约、原始接入

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `PUT /api/v1/products/{id}/metadata-profile` | `contract_meta` | 更新商品元信息档案 |
| `POST /api/v1/assets/{versionId}/field-definitions` | `contract_meta` | 维护字段结构说明 |
| `POST /api/v1/assets/{versionId}/quality-reports` | `contract_meta` | 维护质量报告 |
| `POST /api/v1/assets/{versionId}/processing-jobs` | `contract_meta` | 维护加工任务 |
| `POST /api/v1/skus/{id}/data-contracts` | `contract_meta` | 创建数据契约 |
| `GET /api/v1/skus/{id}/data-contracts/{contractId}` | `contract_meta` | 查看数据契约 |
| `POST /api/v1/assets/{assetId}/raw-ingest-batches` | `catalog` | 创建原始接入批次 |
| `POST /api/v1/raw-ingest-batches/{id}/manifests` | `catalog` | 维护原始对象清单 |
| `POST /api/v1/raw-object-manifests/{id}/detect-format` | `catalog` | 格式识别 |
| `POST /api/v1/raw-object-manifests/{id}/extraction-jobs` | `catalog` | 抽取任务 |
| `POST /api/v1/assets/{versionId}/preview-artifacts` | `catalog` | 预览工件 |
| `POST /api/v1/assets/{versionId}/objects` | `catalog` | 可交付对象维护 |
| `PATCH /api/v1/assets/{assetId}/release-policy` | `catalog` | 版本发布/订阅策略 |

## 5.6 订单、合同、生命周期对象

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/orders` | `order` | 创建订单 |
| `GET /api/v1/orders/{id}` | `order` | 查看订单 |
| `POST /api/v1/orders/{id}/cancel` | `order` | 取消订单 |
| `POST /api/v1/orders/{id}/contract-confirm` | `contract` | 确认合同 |
| `GET /api/v1/orders/{id}/lifecycle-snapshots` | `order` | 查看授权/交付/结算/争议摘要 |

## 5.7 交付与验收

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/orders/{id}/deliver` | `delivery` | 提交交付 |
| `GET /api/v1/orders/{id}/download-ticket` | `delivery` | 获取下载令牌 |
| `POST /api/v1/orders/{id}/share-grants` | `delivery` | 开通只读共享 |
| `GET /api/v1/orders/{id}/share-grants` | `delivery` | 查看共享授权 |
| `POST /api/v1/orders/{id}/subscriptions` | `delivery` | 创建或续订版本订阅 |
| `GET /api/v1/orders/{id}/subscriptions` | `delivery` | 查看版本订阅状态 |
| `POST /api/v1/products/{id}/query-surfaces` | `delivery` | 创建查询面 |
| `POST /api/v1/query-surfaces/{id}/templates` | `delivery` | 发布查询模板 |
| `POST /api/v1/orders/{id}/template-grants` | `delivery` | 开通模板查询授权 |
| `POST /api/v1/orders/{id}/sandbox-workspaces` | `delivery` | 开通查询沙箱 |
| `POST /api/v1/orders/{id}/template-runs` | `delivery` | 执行模板查询 |
| `GET /api/v1/orders/{id}/query-runs` | `delivery` | 查看查询执行记录 |
| `POST /api/v1/orders/{id}/accept` | `order` | 验收通过 |
| `POST /api/v1/orders/{id}/reject` | `order` | 拒收 |
| `GET /api/v1/orders/{id}/usage-log` | `delivery` | 查看使用日志 |

## 5.8 账单、支付、退款、争议

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/orders/{id}/lock` | `billing` | 锁定资金/保证金 |
| `GET /api/v1/billing/{order_id}` | `billing` | 查看账单 |
| `POST /api/v1/refunds` | `billing` | 执行退款 |
| `POST /api/v1/compensations` | `billing` | 执行赔付 |
| `POST /api/v1/cases` | `dispute` | 创建争议 |
| `POST /api/v1/cases/{id}/evidence` | `dispute` | 上传证据 |
| `POST /api/v1/cases/{id}/resolve` | `dispute` | 裁决案件 |

## 5.9 审核、风控、审计、运维、开发者

| 接口 | 归属模块 | 说明 |
|---|---|---|
| `POST /api/v1/review/subjects/{id}` | `provider_ops` | 主体审核 |
| `POST /api/v1/review/products/{id}` | `listing` | 商品审核 |
| `POST /api/v1/review/compliance/{id}` | `provider_ops` | 合规审核 |
| `POST /api/v1/risk/freeze` | `fairness` | 冻结主体/商品/应用 |
| `GET /api/v1/audit/orders/{id}` | `audit` | 订单审计联查 |
| `POST /api/v1/audit/packages/export` | `audit` | 导出证据包 |
| `GET /api/v1/ops/consistency/{refType}/{refId}` | `consistency` | 一致性联查 |
| `POST /api/v1/ops/consistency/reconcile` | `consistency` | 发起一致性修复 |
| `GET /api/v1/ops/search/sync` | `search` | 查看搜索同步状态 |
| `POST /api/v1/ops/search/reindex` | `search` | 发起搜索重建 |
| `POST /api/v1/ops/search/aliases/switch` | `search` | 切换搜索别名 |
| `POST /api/v1/ops/search/cache/invalidate` | `search` | 失效缓存 |
| `PATCH /api/v1/ops/search/ranking-profiles/{id}` | `search` | 更新排序配置 |
| `GET /api/v1/ops/outbox` | `consistency` | 查看 outbox |
| `GET /api/v1/ops/dead-letters` | `consistency` | 查看 dead letter |
| `POST /api/v1/ops/dead-letters/{id}/reprocess` | `consistency` | 重处理 dead letter |
| `GET /api/v1/developer/trace` | `developer` | 调试联查 |

## 6. 外围进程与内部接口

## 6.1 `fabric-adapter`

内部接口建议：

- `POST /internal/fabric/submit-anchor`
- `POST /internal/fabric/submit-authorization-proof`
- `POST /internal/fabric/submit-settlement-proof`

调用方：

- `platform-core.integration`

## 6.2 `fabric-event-listener`

建议不对外暴露公共 API，仅：

- 监听 Fabric
- 投递 Kafka 事件
- 或调用受控内部回写接口

若需内部回写接口，建议：

- `POST /internal/consistency/fabric-callbacks`

## 6.3 `search-indexer`

建议不对前端暴露接口，仅：

- 消费搜索同步事件
- 调用 `OpenSearch`
- 通过 `platform-core` 内部接口回写索引状态

内部回写接口建议：

- `POST /internal/search/index-results`

## 6.4 `data-processing-worker`

内部接口建议：

- `POST /internal/data-processing/jobs`
- `POST /internal/data-processing/job-results`

## 6.5 `notification-worker`

内部接口建议：

- `POST /internal/notifications/send`

## 7. 外部回调冻结

## 7.1 支付回调

路径建议：

- `POST /api/v1/payment/providers/{provider_code}/webhooks`

要求：

- 验签
- 时间戳校验
- 事件唯一键去重
- 幂等处理
- 不得在验签前推进业务状态

## 7.2 未来外部系统回调

预留路径：

- `POST /api/v1/integration/kyc/providers/{provider_code}/webhooks`
- `POST /api/v1/integration/sign/providers/{provider_code}/webhooks`

## 8. 关键校验冻结

## 8.1 SKU 冻结规则

`V1` 仅允许：

- `FILE_STD`
- `FILE_SUB`
- `SHARE_RO`
- `API_SUB`
- `API_PPU`
- `QRY_LITE`
- `SBX_STD`
- `RPT_STD`

补充规则：

- `API_SUB / API_PPU` 默认权利：`access + result_get`
- `FILE_SUB` 默认授权模板：`LICENSE_FILE_USE_V1`
- `SBX_STD` 默认授权模板：`LICENSE_SANDBOX_USE_V1`
- `RPT_STD` 默认授权模板：`LICENSE_RESULT_USE_V1`
- API 模板命名统一按 `API_SUB / API_PPU` 细分

## 8.2 生命周期对象冻结规则

以下对象必须作为独立正式对象出现在接口设计中：

- `Authorization`
- `Delivery`
- `Settlement`
- `Dispute`

不得只通过：

- 订单附属字段
- 临时聚合视图
- 审计事件
- 工单系统

来替代它们。

## 8.3 一致性字段族冻结规则

以下字段在涉及生命周期对象摘要的接口中必须可返回：

- `authority_model`
- `proof_commit_policy`
- `proof_commit_state`
- `external_fact_status`
- `reconcile_status`
- `status_version`

## 9. 当前不进入本文件的内容

当前不在本文件内展开：

- 完整 OpenAPI YAML
- 每个字段的全部校验正则
- 错误码全量字典
- Topic 与事件 payload 明细
- 测试用例矩阵

这些内容将在后续冻结文件中继续细化。

## 10. 一句话结论

当前阶段应把接口面理解为：

**以 `platform-core` 为唯一业务 API 出口，围绕商品、SKU、订单、生命周期对象、支付、审计和一致性定义一套稳定的版本化 REST 接口；外围进程仅通过受控内部接口或事件总线协作，不形成第二套业务 API。**
