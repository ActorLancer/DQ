# OpenAPI 分域骨架（BOOT-007）

本目录按领域拆分 OpenAPI：

- `packages/openapi/*.yaml` 是当前实现阶段的 OpenAPI 设计参考与版本对象。
- `docs/02-openapi/*.yaml` 只承接实现校验后的归档副本，不作为当前实现期权威源。

- `iam.yaml`
- `catalog.yaml`
- `trade.yaml`
- `billing.yaml`
- `search.yaml`
- `recommendation.yaml`
- `audit.yaml`
- `ops.yaml`

使用 `merge-openapi.sh` 生成聚合输出占位（当前仅做骨架）。
