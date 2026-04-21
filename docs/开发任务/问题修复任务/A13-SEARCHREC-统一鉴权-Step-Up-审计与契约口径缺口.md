# A13 SEARCHREC 统一鉴权 / Step-Up / 审计与契约口径缺口

## 1. 任务定位

- 问题编号：`A13`
- 严重级别：`high`
- 关联阶段：`SEARCHREC`
- 关联任务：`AUD-022`、`SEARCHREC-018`、`SEARCHREC-019`、`SEARCHREC-015`、`SEARCHREC-016`、`SEARCHREC-017`
- 处理方式：当前批次先冻结搜索/推荐的统一鉴权、权限点、`step-up`、审计、错误码口径，并把未来代码改造义务写回任务清单；进入 `SEARCHREC` / `AUD` 代码实现批次后，再修改 handler / service / repo / OpenAPI / 测试

## 1.1 当前批次边界

如果当前批次只处理“口径未收缩 / 实现占位语义已漂移 / 文档冲突”，则：

- 可以先冻结 `Authorization + 正式权限点 + X-Idempotency-Key + 必要 X-Step-Up-Token` 的正式口径
- 可以先修正 runbook 中的旧 `x-role` 示例和占位描述
- 可以先把后续实现义务回写到 `AUD-022`、`SEARCHREC-018`、`SEARCHREC-019`、`SEARCHREC-015`、`SEARCHREC-016`、`SEARCHREC-017`
- 但不能把尚未实现的统一鉴权接入、OpenAPI 细节补全、测试矩阵补全误报为已完成

进入后续代码实现批次后，Agent 必须同步补齐：

- `apps/platform-core/src/modules/search/**` 与 `apps/platform-core/src/modules/recommendation/**` 中对统一鉴权门面的真实接入
- 搜索/推荐运维写接口的正式 `step-up` 判定、审计留痕与错误码映射
- `packages/openapi/search.yaml`、`packages/openapi/recommendation.yaml` 与 `docs/02-openapi/**` 归档
- `docs/05-test-cases/search-rec-cases.md` 中的统一鉴权 / 审计 / 错误码验收项

## 2. 问题描述

当前 `SEARCHREC` 运行时、OpenAPI、测试之间仍没有共享同一套正式契约；本批次虽然已先修正 runbook，但代码与契约层仍混用了两套语义：

1. 冻结协议要求的正式语义：
   - 统一鉴权
   - 正式权限点
   - `Authorization`
   - 写接口 `X-Idempotency-Key`
   - 高风险写接口必要时要求 `X-Step-Up-Token`
   - 写操作审计留痕
   - 搜索域正式错误码 `SEARCH_*`
2. 当前实现与示例里固化的占位语义：
   - `x-role`
   - 只检查 header 是否存在的伪 `step-up`
   - 未显式冻结的审计要求
   - 泛化到 `OPS_*` 的错误码

这意味着：

- 后续开发会继续按错误 header / 错误权限语义实现接口
- OpenAPI 和测试会验证错误口径
- 统一鉴权真正接入时，会在 `SEARCHREC` 面积性返工

## 3. 正确冻结口径

以接口协议、权限设计和运行边界文档为冻结基线，`SEARCHREC` 阶段必须使用以下正式口径。

### 3.1 鉴权与请求头

- 统一鉴权入口为 `Authorization: Bearer <access_token>`
- 请求追踪头为 `X-Request-Id`
- 所有写接口必须携带 `X-Idempotency-Key`
- 高风险运维写接口必须携带 `X-Step-Up-Token`
- `x-role` 只能作为历史占位说明，不再作为任何正式实现、runbook、OpenAPI、测试的默认语义

### 3.2 正式权限点

搜索侧：

- `portal.search.read`
- `ops.search_sync.read`
- `ops.search_reindex.execute`
- `ops.search_alias.manage`
- `ops.search_cache.invalidate`
- `ops.search_ranking.read`
- `ops.search_ranking.manage`

推荐侧：

- `portal.recommendation.read`
- `ops.recommendation.read`
- `ops.recommendation.manage`
- `ops.recommend_rebuild.execute`

### 3.3 审计与 Step-Up

- `POST /api/v1/ops/search/reindex` 需要审计、`step-up`
- `POST /api/v1/ops/search/aliases/switch` 需要审计、`step-up`
- `PATCH /api/v1/ops/search/ranking-profiles/{id}` 需要审计、`step-up`
- `POST /api/v1/ops/search/cache/invalidate` 需要审计
- `PATCH /api/v1/ops/recommendation/placements/{placement_code}` 需要审计、`step-up`
- `PATCH /api/v1/ops/recommendation/ranking-profiles/{id}` 需要审计、`step-up`
- `POST /api/v1/ops/recommendation/rebuild` 需要审计，是否要求 `step-up` 以后续实现批次按权限清单与风险级别落地

### 3.4 错误码

搜索域错误码应收敛为协议中的 `SEARCH_*`，至少包括：

- `SEARCH_QUERY_INVALID`
- `SEARCH_BACKEND_UNAVAILABLE`
- `SEARCH_RESULT_STALE`
- `SEARCH_REINDEX_FORBIDDEN`
- `SEARCH_ALIAS_SWITCH_FORBIDDEN`
- `SEARCH_CACHE_INVALIDATE_FORBIDDEN`

### 3.5 OpenAPI 与测试约束

- OpenAPI 必须描述正式请求头、权限与错误码
- 测试样例必须验证正式请求头、正式权限点、必要的 `step-up` 与审计留痕
- 后续不得继续使用 `x-role` 占位请求来证明“接口已通过”

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [商品搜索、排序与索引同步接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md)
  - 已冻结搜索接口头、运维接口、错误码与 `X-Idempotency-Key`
- [商品推荐与个性化发现接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/商品推荐与个性化发现接口协议正式版.md)
  - 已冻结推荐权限点、行为幂等、运维配置接口与重建接口
- [接口权限校验清单.md](/home/luna/Documents/DataB/docs/权限设计/接口权限校验清单.md)
  - 已冻结 `SEARCHREC` 接口的正式权限点、审计与 `step-up`
- [apps/platform-core/src/modules/search/api/handlers.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/search/api/handlers.rs)
  - 仍存在直接读取占位权限头的实现
- [apps/platform-core/src/modules/recommendation/api/handlers.rs](/home/luna/Documents/DataB/apps/platform-core/src/modules/recommendation/api/handlers.rs)
  - 仍存在直接读取占位权限头的实现
- [docs/05-test-cases/search-rec-cases.md](/home/luna/Documents/DataB/docs/05-test-cases/search-rec-cases.md)
  - 仅覆盖业务链路，尚未声明统一鉴权 / 审计 / 错误码验收义务

## 5. 本批次修复目标

本批次只做以下收口动作：

1. 冻结 `SEARCHREC` 的正式鉴权 / 权限 / `step-up` / 审计 / 错误码口径
2. 清理 runbook 中继续传播旧 `x-role` 语义的内容
3. 把未来代码实现义务写回任务清单与 TODO
4. 明确 OpenAPI / 测试仍需在后续实现批次补齐，当前不能伪造完成态

## 6. 强约束

1. 当前批次不能直接修改搜索/推荐运行时代码来“顺手修掉”
2. 当前批次不能伪造 `packages/openapi/search.yaml` / `recommendation.yaml` 已完成统一鉴权改造
3. 当前批次不能伪造 `docs/05-test-cases/search-rec-cases.md` 已完整覆盖鉴权 / 审计 / 错误码矩阵
4. 进入实现批次后，Agent 不得再把 `x-role` 当作正式契约继续传播

## 7. 后续实现建议

### 7.1 运行时代码

后续应在以下层面切换到正式口径：

- `search` / `recommendation` handler
- 统一鉴权门面接入
- 正式权限点校验
- `step-up` 中间件或门禁逻辑
- 审计写入
- 搜索域错误码映射

### 7.2 OpenAPI

后续应同步补齐：

- `packages/openapi/search.yaml`
- `packages/openapi/recommendation.yaml`
- `docs/02-openapi/search.yaml`
- `docs/02-openapi/recommendation.yaml`

并明确：

- `Authorization`
- `X-Request-Id`
- `X-Idempotency-Key`
- 必要的 `X-Step-Up-Token`
- 正式权限点与错误码示例

### 7.3 测试

后续应补齐：

- Search / Recommendation API 集成测试
- 运维写接口的 `step-up` 与审计断言
- 搜索域 `SEARCH_*` 错误码断言
- 拒绝继续使用 `x-role` 占位请求的负向用例

## 8. 任务与文档联动要求

以下任务在后续代码实现时必须以本文件为额外参考：

- `AUD-022`
- `SEARCHREC-018`
- `SEARCHREC-019`
- `SEARCHREC-015`
- `SEARCHREC-016`
- `SEARCHREC-017`

并要求这些任务在 `technical_reference` 中显式加入本文件，避免后续 Agent 只读旧任务描述而忽略本次冻结口径。

推荐实施顺序冻结为：

1. `AUD-022`
2. `SEARCHREC-018`
3. `SEARCHREC-019`
4. `SEARCHREC-015`
5. `SEARCHREC-016`
6. `SEARCHREC-017`
