# Catalog Review Cases (CAT-026)

## 1. 目标与范围

本用例文档用于覆盖 `Catalog/Listing/Review` 在上架审核阶段的关键约束，聚焦以下四类场景：

- 上架规则（草稿提交、审核通过/驳回、状态机）
- 字段缺失（必填字段校验）
- 模板不匹配（SKU 与模板族强约束）
- 风险阻断（risk block 时禁止提交）

不包含 V2/V3 预留能力，不变更接口契约。

## 2. 关联冻结约束

- 领域模型：`docs/领域模型/全量领域模型与对象关系说明.md`（4.2 目录与商品聚合）
- 接口协议：`docs/数据库设计/接口协议/目录与商品接口协议正式版.md`（5. V1 接口）
- 业务流程：`docs/业务流程/业务流程图-V1-完整版.md`（4.2 商品创建、模板绑定与上架流程）

## 3. 前置环境

- 本地核心栈已启动：`make up-local`
- 应用联调统一读取：`infra/docker/.env.local`
- 宿主机直连 Kafka 时一律使用 `127.0.0.1:9094`；`kafka:9092` / 容器内 `localhost:9092` 仅供 compose 网络内部与容器内探测使用
- 服务启动（示例）：

```bash
set -a; source infra/docker/.env.local; set +a
APP_PORT=18080 APP_HOST=127.0.0.1 \
KAFKA_BROKERS=127.0.0.1:9094 KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core
```

## 4. 用例矩阵

| 用例ID | 分类 | 接口 | 输入要点 | 预期结果 | 预期审计 |
| --- | --- | --- | --- | --- | --- |
| CAT-026-001 | 上架规则 | `POST /api/v1/products/{id}/submit` | 商品为 `draft`，且已具备 metadata profile、至少 1 个 SKU、SKU 已绑定模板，且未风险阻断 | `200`，状态变为 `pending_review`，返回 `review_task_id` | `catalog.product.submit` |
| CAT-026-002 | 上架规则 | `POST /api/v1/review/products/{id}` (`approve`) | 商品状态 `pending_review`，审核动作为 `approve` | `200`，状态变为 `listed` | `catalog.review.product` |
| CAT-026-003 | 上架规则 | `POST /api/v1/review/products/{id}` (`reject`) | 商品状态 `pending_review`，审核动作为 `reject` | `200`，状态回退 `draft` | `catalog.review.product` |
| CAT-026-004 | 字段缺失 | `POST /api/v1/products` | 缺少必填字段（如 `asset_id` / `seller_org_id` / `delivery_type`）或为空串 | `400`，`CAT_VALIDATION_FAILED` | 无成功审计事件 |
| CAT-026-005 | 字段缺失 | `POST /api/v1/products/{id}/skus` | 缺少 `sku_code` / `sku_type` / `billing_mode` / `acceptance_mode` / `refund_mode` | `400`，`CAT_VALIDATION_FAILED` | 无成功审计事件 |
| CAT-026-006 | 模板不匹配 | `POST /api/v1/skus/{id}/bind-template` | 模板 `applicable_sku_types` 不包含该 SKU 类型，或命中禁用的旧通用 API 模板名 | `400`，`CAT_VALIDATION_FAILED` | 无成功审计事件 |
| CAT-026-007 | 模板不匹配 | `POST /api/v1/products/{id}/bind-template` | 商品下任一 SKU 与模板族不兼容 | `400`，`CAT_VALIDATION_FAILED` | 无成功审计事件 |
| CAT-026-008 | 风险阻断 | `POST /api/v1/products/{id}/submit` | `catalog.product.metadata.risk_blocked=true` 或 `metadata.risk_flags.block_submit=true` | `409`，`TRD_STATE_CONFLICT`，提示风险阻断 | 无成功审计事件 |

## 5. 关键断言细则

### 5.1 状态机断言

- 仅允许：`draft -> pending_review -> listed`，以及 `pending_review -> draft`（驳回）。
- `submit` 前必须满足完整性校验：metadata profile、SKU、SKU 模板绑定。

### 5.2 模板约束断言

- `sku_type` 与模板 `applicable_sku_types` 必须匹配。
- 禁止旧通用 API 模板名：`CONTRACT_API_V1 / LICENSE_API_USE_V1 / ACCEPT_API_V1 / REFUND_API_V1`。
- `API_SUB` 只能绑定 `API_SUB` 模板族；`API_PPU` 只能绑定 `API_PPU` 模板族。

### 5.3 风险阻断断言

- 当商品元数据标记风险阻断时，`submit` 必须被拒绝。
- 返回应保持统一错误码语义，且不产生成功提交审计事件。

## 6. 建议执行顺序

1. 先跑正向链路：`CAT-026-001/002/003`。
2. 再跑参数与契约校验：`CAT-026-004/005/006/007`。
3. 最后跑风险阻断：`CAT-026-008`。

## 7. 验证记录模板

```text
执行日期：
执行人：
环境：local/core
request_id：
用例ID：
输入摘要：
HTTP 结果：
错误码/状态：
审计验证结果：
结论：pass/fail
```
