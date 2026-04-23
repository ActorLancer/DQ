# Local Fixtures（BOOT-010）

本目录用于本地演示与联调样例：

- `product-sample.json`
- `api-request-sample.json`
- `evidence-package-sample.json`
- `standard-scenarios-manifest.json`
- `standard-scenarios-sample.json`
- `local-smoke-suite-manifest.json`

说明：

- `fixtures/local/` 仍服务于 `ENV-*` 本地 bootstrap、smoke、配置快照与轻量样例。
- `TEST-001` 起，五条标准链路的正式 demo 数据包迁移到 `fixtures/demo/`，后续 `seed-demo.sh`、E2E 与验收矩阵应优先消费 `fixtures/demo/`。

## ENV-041 补充说明

- `standard-scenarios-manifest.json` 对齐首批五条标准链路（S1~S5）与主 SKU。
- `standard-scenarios-sample.json` 提供最小演示数据集，覆盖：
  - 企业主体（卖方/买方）
  - 产品与 8 个 V1 SKU
  - 模板样例
  - 订单、支付、交付样例
  - request_id 可追踪键
- `SEARCHREC-014` 起，五条标准链路官方商品样例统一对齐首页推荐位 `home_featured` 的固定样例配置；fixture 中的场景名必须与推荐首页直达入口保持一致，不再使用泛化英文样例名替代。
