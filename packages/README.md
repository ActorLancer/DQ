# packages 目录校准（BOOT-032）

`packages/` 用于共享契约、配置与前端/SDK 复用资产，当前边界如下：

已存在：

- `api-contracts/`
- `event-contracts/`
- `domain-types/`
- `test-fixtures/`

本批新增落位：

- `openapi/`
- `sdk-ts/`
- `ui/`
- `shared-config/`
- `observability-contracts/`

说明：

- 本批只做目录边界收敛，不引入具体实现内容。
