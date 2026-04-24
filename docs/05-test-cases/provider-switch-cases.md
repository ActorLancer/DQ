# Provider Switch Cases（TEST-007）

## Scope

本文件冻结 `TEST-007` 的正式 provider 切换验收基线。目标不是证明“仓库里有 mock/real 类型”，而是证明以下切换都通过配置完成，不改业务代码：

- `platform-core`：`PROVIDER_MODE=mock|real`
- `fabric-adapter`：`FABRIC_ADAPTER_PROVIDER_MODE=mock|fabric-test-network`
- `mock-payment-provider` live adapter：`MOCK_PAYMENT_ADAPTER_MODE=stub|live`

## Authority

- `docs/04-runbooks/provider-switch.md`
- `docs/全集成文档/数据交易平台-全集成基线-V1.md`
- `apps/platform-core/crates/provider-kit/src/lib.rs`
- `apps/platform-core/crates/config/src/lib.rs`
- `services/fabric-adapter/internal/config/config.go`
- `services/fabric-adapter/internal/provider/factory.go`

## Required Assertions

1. `provider-kit` 的支付 / 签章 / 链写 provider 同时存在 `mock` 与 `real` 两套入口，切换后 `kind()` 与返回引用显式变化。
2. `mock-payment-provider` live adapter 真实命中三条正式路径：
   - `/mock/payment/charge/success`
   - `/mock/payment/charge/fail`
   - `/mock/payment/charge/timeout`
3. `platform-core` 的 real provider 仍走同一业务路径，不改 handler / router：
   - `POST /api/v1/orders/{id}/contract-confirm`
   - `PROVIDER_MODE=mock` 时返回 `signature_provider_mode=mock`
   - `PROVIDER_MODE=real` 且 `FF_REAL_PROVIDER=true` 时返回 `signature_provider_mode=real`
4. `platform-core` 对 real provider 有正式门控：
   - `PROVIDER_MODE=real` 且未开启 `FF_REAL_PROVIDER` 时 startup self-check 拒绝启动
5. `fabric-adapter` 的 provider factory 真实承接两种模式：
   - `mock`
   - `fabric-test-network`
6. `fabric-test-network` 路径必须留下真实证据：
   - Go provider live smoke 通过
   - Fabric 账本查询命中
   - PostgreSQL `ops.external_fact_receipt` 回查命中

## Execution Matrix

| 面向 | mock 路径 | real / live 路径 | 正式入口 |
| --- | --- | --- | --- |
| 支付 provider | `cargo test -p provider-kit` | `MOCK_PAYMENT_ADAPTER_MODE=live cargo test -p provider-kit live_mock_payment_adapter_hits_three_mock_paths -- --ignored --nocapture` | `./scripts/check-provider-switch.sh` |
| 签章 provider | `TRADE_DB_SMOKE=1 PROVIDER_MODE=mock cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture` | `TRADE_DB_SMOKE=1 PROVIDER_MODE=real FF_REAL_PROVIDER=true cargo test -p platform-core trade026_contract_signing_provider_db_smoke -- --nocapture` | `./scripts/check-provider-switch.sh` |
| 链写 provider | `./scripts/fabric-adapter-test.sh` 中的 mock factory / config coverage | `./scripts/fabric-adapter-live-smoke.sh` | `./scripts/check-provider-switch.sh` |

## Execution Commands

正式 checker：

```bash
ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh
```

CI workflow：

```bash
.github/workflows/provider-switch.yml
```

## Traceability

- 任务定义：`docs/开发任务/v1-core-开发任务清单.csv`、`docs/开发任务/v1-core-开发任务清单.md`
- 运行边界：`docs/04-runbooks/provider-switch.md`
- 平台 provider：`apps/platform-core/crates/provider-kit/src/lib.rs`
- 平台签章切换 smoke：`apps/platform-core/src/modules/order/tests/trade026_contract_signing_provider_db.rs`
- Fabric provider 切换：`services/fabric-adapter/internal/provider/factory.go`
- 官方 checker：`scripts/check-provider-switch.sh`
