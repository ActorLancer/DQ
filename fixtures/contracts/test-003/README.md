# TEST-003 Contract Baseline

本目录是 `TEST-003` 的正式 contract baseline，只覆盖 API / OpenAPI 相关的冻结契约：

- 统一成功响应 envelope：`code + message + request_id + data`
- 统一失败响应 envelope：`code + message + request_id + details`
- 关键响应字段回收到冻结逻辑名：`current_state / order_amount / metered_quantity`
- 关键错误码在权威文档、OpenAPI 或正式测试中的存在性
- 订单状态机 action enum 与对应禁止错误码的冻结绑定

本目录不替代：

- `./scripts/check-openapi-schema.sh` 的 OpenAPI 基础结构与镜像同步检查
- `./scripts/check-canonical-contracts.sh` 的 canonical topic / topology / smoke 检查（正式归属 `TEST-028`）

本目录由 `./scripts/check-api-contract-baseline.sh` 消费。
