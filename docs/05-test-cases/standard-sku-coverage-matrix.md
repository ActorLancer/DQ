# Standard SKU Coverage Matrix

`TEST-023` 的目标，是把 `8` 个标准 `sku_type` 的主路径、异常路径、退款/争议证据和五条标准链路挂点收敛成一个正式、可执行、可回查的矩阵。

## Authority

- 任务源：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 标准链路与 SKU 映射：`docs/全集成文档/数据交易平台-全集成基线-V1.md` `5.3.2 / 5.3.2A`
- SKU 真值与闭环矩阵：`docs/00-context/standard-sku-truth.md`、`docs/00-context/v1-closed-loop-matrix.md`
- 状态机基线：`docs/05-test-cases/order-state-machine.md`
- Billing trigger authority：`db/seeds/031_sku_trigger_matrix.sql`、`fixtures/demo/billing.json`
- Demo order authority：`fixtures/demo/scenarios.json`、`fixtures/demo/orders.json`、`fixtures/demo/sku-coverage-matrix.json`

## Official Entry

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-standard-sku-coverage.sh
```

正式 artifact：

- `target/test-artifacts/standard-sku-coverage/summary.json`
- `target/test-artifacts/standard-sku-coverage/catalog-standard-scenarios-response.json`
- `target/test-artifacts/standard-sku-coverage/billing-detail-*.json`
- `target/test-artifacts/standard-sku-coverage/executed-cargo-tests.txt`

## Evidence Rule

每个 `sku_type` 都必须同时具备三类证据：

1. 主路径：对应 `trade008~015` 中的正式状态机 smoke。
2. 异常路径：同一 smoke 内的非法跳转 / 终态重入保护，或正式 revoke / timeout / disable 入口。
3. 退款或争议路径：
   - `FILE_STD / FILE_SUB / SHARE_RO / RPT_STD` 直接使用各自已有的 refund / dispute / reject / revoke smoke。
   - `API_SUB / API_PPU / QRY_LITE / SBX_STD` 不额外发明第二套 order transition；它们通过正式 `billing basis + trigger bridge + dispute settlement engine` 接入退款/争议链路。`TEST-023` 必须同时验证：
     - demo 订单 `GET /api/v1/billing/{order_id}` 暴露正确的 `refund_entry / dispute_freeze_trigger / resume_settlement_trigger`
     - bridge / dispute recompute smoke 能真实运行

## Matrix

| SKU | 场景挂点 | 主路径 evidence | 异常 / 阻断 evidence | 退款 / 争议 evidence | billing basis order |
| --- | --- | --- | --- | --- | --- |
| `FILE_STD` | `S2` 主挂点 | `trade008_file_std_state_machine_db_smoke` | `trade008_file_std_state_machine_db_smoke` | `trade008_file_std_state_machine_db_smoke` | `34000000-0000-0000-0000-000000000002` |
| `FILE_SUB` | `S2` 补充挂点 | `trade009_file_sub_state_machine_db_smoke` | `trade009_file_sub_state_machine_db_smoke` | `trade009_file_sub_state_machine_db_smoke` | `34000000-0000-0000-0000-000000000102` |
| `SHARE_RO` | `S3` 补充挂点 | `trade012_share_ro_state_machine_db_smoke` | `trade012_share_ro_state_machine_db_smoke` | `bil026_share_ro_billing_db_smoke` | `34000000-0000-0000-0000-000000000103` |
| `API_SUB` | `S1`、`S4` 主挂点 | `trade010_api_sub_state_machine_db_smoke` | `trade010_api_sub_state_machine_db_smoke` | `bil017_api_sku_billing_basis_db_smoke` + `bil024_billing_trigger_bridge_db_smoke` + `bil019_dispute_refund_compensation_recompute_db_smoke` + live billing readback | `34000000-0000-0000-0000-000000000001` |
| `API_PPU` | `S1` 补充挂点 | `trade011_api_ppu_state_machine_db_smoke` | `trade011_api_ppu_state_machine_db_smoke` | `bil017_api_sku_billing_basis_db_smoke` + `bil024_billing_trigger_bridge_db_smoke` + `bil019_dispute_refund_compensation_recompute_db_smoke` + live billing readback | `34000000-0000-0000-0000-000000000101` |
| `QRY_LITE` | `S5` 主挂点 | `trade013_qry_lite_state_machine_db_smoke` | `trade013_qry_lite_state_machine_db_smoke` | `bil024_billing_trigger_bridge_db_smoke` + `bil019_dispute_refund_compensation_recompute_db_smoke` + live billing readback | `34000000-0000-0000-0000-000000000005` |
| `SBX_STD` | `S3` 主挂点 | `trade014_sbx_std_state_machine_db_smoke` | `trade014_sbx_std_state_machine_db_smoke` | `bil024_billing_trigger_bridge_db_smoke` + `bil019_dispute_refund_compensation_recompute_db_smoke` + live billing readback | `34000000-0000-0000-0000-000000000003` |
| `RPT_STD` | `S4`、`S5` 补充挂点 | `trade015_rpt_std_state_machine_db_smoke` | `trade015_rpt_std_state_machine_db_smoke` | `bil025_billing_adjustment_freeze_db_smoke` + live billing readback | `34000000-0000-0000-0000-000000000104` |

## Live Readback Scope

`check-standard-sku-coverage.mjs` 会在 demo seed 完成后，真实查询：

- `GET /api/v1/catalog/standard-scenarios`
- `GET /api/v1/billing/{order_id}`

并对每个 `sku_type` 回查：

- 场景挂点是否仍匹配 `S1~S5`
- demo 订单 `current_state / order_amount`
- `sku_billing_basis.refund_entry`
- `sku_billing_basis.dispute_freeze_trigger`
- `sku_billing_basis.resume_settlement_trigger`
- `API_SUB / API_PPU` 的 `api_billing_basis`

## Boundary

- 该矩阵不替代 `TEST-024` 的编排链路顺序验证；`TEST-024` 继续覆盖 `支付成功 -> 待交付 -> 交付完成 -> 待验收 -> 验收通过/拒收 -> 结算/退款` 的连续 orchestration。
- 该矩阵不把 `billing trigger` 静态字段本身误报为“退款/争议已通过”；它必须和真实 `trade / billing / catalog` smoke、正式 API 读回一起出现。
- 该矩阵不新造第二套 SKU、场景、模板或 topic 真值，统一回到 `fixtures/demo`、`db/seeds/031 / 032` 与正式 API。
