# V1 标准 SKU 真值冻结（CTX-006）

## 1. V1 正式 SKU 清单

`V1-Core` 只承认以下 8 个标准 `sku_type` 作为交易事实源：

1. `FILE_STD`
2. `FILE_SUB`
3. `SHARE_RO`
4. `API_SUB`
5. `API_PPU`
6. `QRY_LITE`
7. `SBX_STD`
8. `RPT_STD`

## 2. 真值规则

- `sku_type` 是标准化交易类型真值，必须稳定可审计。
- `sku_code` 是商业套餐编码，不得反向替代 `sku_type`。
- 订单、合同、授权、交付、验收、计费、结算、争议都应按 `sku_type` 快照，不得用“场景名”替代。

## 3. 禁止事项

- 禁止把 `SHARE_RO / QRY_LITE / RPT_STD` 合并回“文件/API/沙箱大类”。
- 禁止新增未冻结的 `V1` SKU 枚举并直接进入主链路。
- 禁止在未完成审批时将 `V2/V3` SKU 扩展混入 `V1-Core`。
