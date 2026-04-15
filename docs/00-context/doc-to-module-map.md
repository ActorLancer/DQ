# 文档到模块映射（CTX-017）

## 1. 映射原则

- 专题 PRD 只映射到 `V1-Core` 当前模块边界。
- 交付对象同时给出：模块域、迁移域、OpenAPI 分组、测试域。
- 若涉及 `V2/V3` 扩展，仅保留边界提示，不作为当前实现承诺。

## 2. 映射表

| 专题文档（示例） | 模块域（platform-core） | 迁移域（数据库） | OpenAPI 分组 | 测试域 |
| --- | --- | --- | --- | --- |
| 身份认证、注册登录与会话管理设计 | `iam`, `party`, `access` | `010_identity_and_access` | `auth`, `session`, `party` | `CORE-001`, `AUTHZ-*` |
| 数据产品分类与交易模式详细稿 | `catalog`, `listing`, `contract_meta` | `020_catalog_contract`, `061_data_object_trade_modes` | `products`, `skus`, `contracts-meta` | `CORE-002`, `SKU-*` |
| 支付、资金流与轻结算设计 | `billing`, `order`, `dispute` | `040_billing_support_risk` | `payments`, `billing`, `settlements`, `refunds` | `PAY-*`, `CORE-010` |
| 双层权威模型与链上链下一致性设计 | `consistency`, `audit`, `integration` | `056_dual_authority_consistency` | `ops/consistency`, `ops/outbox` | `ASYNC-*`, `CORE-012` |
| 商品搜索、排序与索引同步设计 | `search`, `integration` | `057_search_sync_architecture` | `search`, `ops/search` | `SEARCH-*` |
| 商品推荐与个性化发现设计 | `recommend` | `058_recommendation_module` | `recommendations` | `RECO-*` |
| 审计、证据链与回放设计 | `audit`, `ops` | `050_audit_search_dev_ops`, `055_audit_hardening` | `audit`, `ops/audit` | `AUDIT-*`, `ASYNC-007` |
| 敏感数据处理与受控交付设计 | `delivery`, `authorization`, `provider_ops` | `066_sensitive_data_controlled_delivery` | `deliveries`, `authorizations`, `query-runs` | `LIFE-*`, `CORE-008` |

## 3. 使用方式

1. 开始某一域任务前，先定位对应专题文档行。
2. 再按“模块域 -> 迁移域 -> OpenAPI -> 测试域”顺序推进实现与验证。
3. 出现多文档冲突时，以 CSV 任务技术参考和 V1 冻结文档裁决。
