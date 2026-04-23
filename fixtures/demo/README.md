# Demo Fixtures（TEST-001）

`fixtures/demo/` 是 `TEST` 阶段正式五条标准链路 demo 数据包入口，不再把 `fixtures/local/` 的轻量样例误报为完整演示数据。

目录约定：

- `manifest.json`：数据包元信息、文件清单、校验入口与上游真值源。
- `subjects.json`：演示租户、用户、应用主体。
- `catalog.json`：五条标准链路官方展示商品、10 个展示 SKU、模板映射与 `home_featured` 固定样例。
- `orders.json`：按五条标准链路整理的 10 个交易蓝图（5 条主路径 + 5 条补充 SKU 路径）。
- `delivery.json`：交付对象蓝图与复用的 `DLV-026` fixture 来源。
- `billing.json`：支付/账单样例与 8 个标准 SKU 的 billing trigger matrix 对照。
- `audit.json`：每条链路必须出现的审计动作与 step-up 约束。
- `scenarios.json`：五条标准链路的总览 bundle，串起主体、商品、订单、交付、账单、审计。

使用方式：

```bash
./scripts/check-demo-fixtures.sh
./scripts/seed-demo.sh
./scripts/check-demo-seed.sh
```

说明：

- `catalog.json` 的官方展示商品与 SKU 复用 `db/seeds/033_searchrec_recommendation_samples.sql` 的正式首页/搜索样例。
- `scripts/seed-demo.sh` 会先执行 `db/scripts/seed-up.sh` 的正式 manifest，再按 `orders.json / billing.json / delivery.json` 追加 `TEST-002` 的 demo 订单、支付和交付记录。
- `billing.json` 的 `payment_provider.provider_key` 与正式支付 provider 口径保持一致，使用 `mock_payment`；`mock-payment-provider` 是本地联调服务名，不是数据库中的 `provider_key`。
- `orders.json` 及其下游交付、账单、审计对象是 `TEST-001` 新定义的 demo blueprint，供 `TEST-002` 的 `seed-demo.sh` 与后续 E2E/验收矩阵导入使用。
- 若未来扩展或替换 demo 数据包，必须同时更新 `manifest.json`、校验脚本和本 README。
