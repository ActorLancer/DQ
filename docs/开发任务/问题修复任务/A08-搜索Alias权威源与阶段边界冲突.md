# A08 搜索 Alias 权威源与阶段边界冲突

## 1. 任务定位

- 问题编号：`A08`
- 严重级别：`high`
- 关联阶段：`SEARCHREC`
- 关联任务：`AUD-022`、`SEARCHREC-007`、`SEARCHREC-015`、`SEARCHREC-021`
- 处理方式：先统一搜索 alias 的权威源、命名口径和阶段边界，再实现重建/切换/运维能力；不允许继续让 PRD、migration、运行脚本、代码、接口协议各自说一套

## 2. 问题描述

当前搜索 alias 的命名和控制边界互相冲突，至少存在三层漂移：

1. PRD 推荐使用 `product_search_read/write`、`seller_search_read/write`
2. migration 与 `search.index_alias_binding` 也按这套结构化别名建模
3. 运行脚本和代码却硬编码 `catalog_products_v1`、`seller_profiles_v1`、`search_sync_jobs_v1`

阶段边界冲突现已完成人工裁决，并冻结为：

1. `alias switch` 属于当前 `V1` 的最小运维能力
2. `V3` 仅扩展更复杂的切换策略、灰度策略和自动回滚策略

这意味着：

- alias 的结构化权威源和实际运行默认值不一致
- 搜索重建、切换、runbook、OpenSearch 初始化无法共享同一基线
- `SEARCHREC / AUD` 的运维接口、OpenAPI 和实现必须同步到同一个阶段答案

## 3. 正确冻结口径

根据 PRD、migration 和当前阶段任务目标，应先明确并冻结以下原则：

### 3.1 Alias 权威源

- `search.index_alias_binding` 应是 alias 绑定的结构化权威源
- 运行脚本、初始化脚本、应用默认值、ops 接口都应与该权威源一致
- 不应继续让代码和脚本各自硬编码另一套默认别名

### 3.2 Alias 命名口径

当前 alias 命名口径直接收口为：

- `product_search_read`
- `product_search_write`
- `seller_search_read`
- `seller_search_write`
- 运行脚本、初始化脚本、应用默认值、runbook、ops 接口、OpenSearch 初始化都必须使用这套别名
- 不再允许重新打开“是否采用另一套命名”的选择空间

### 3.3 阶段边界

当前执行口径直接收口为：

- `alias switch` 属于当前 `V1` 的最小运维能力
- `V3` 可以扩展更复杂的切换策略、灰度策略和自动回滚策略
- 相关协议、全集成文档和实现说明必须同步到同一口径，不允许残留历史 `V3` 表述

## 4. 已知证据

已核对的典型漂移点包括但不限于：

- [商品搜索、排序与索引同步设计.md](/home/luna/Documents/DataB/docs/原始PRD/商品搜索、排序与索引同步设计.md)
  - 建议 alias 使用 `product_search_read/write`、`seller_search_read/write`
- [057_search_sync_architecture.sql](/home/luna/Documents/DataB/docs/数据库设计/V1/upgrade/057_search_sync_architecture.sql)
  - 已 seed `search.index_alias_binding`，别名与 PRD 口径一致
- [商品搜索、排序与索引同步接口协议正式版.md](/home/luna/Documents/DataB/docs/数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md)
  - 已同步收口为 `V1` 搜索运维接口
- [v1-core-开发任务清单.md](/home/luna/Documents/DataB/docs/开发任务/v1-core-开发任务清单.md)
  - `AUD-022` 明确要求当前阶段实现“别名切换”
- [lib.rs](/home/luna/Documents/DataB/apps/platform-core/src/lib.rs)
  - 运行时代码仍硬编码另一套 alias
- [init-opensearch.sh](/home/luna/Documents/DataB/infra/opensearch/init-opensearch.sh)
  - 初始化脚本仍硬编码另一套 alias
- [opensearch-local.md](/home/luna/Documents/DataB/docs/04-runbooks/opensearch-local.md)
  - runbook 仍沿另一套 alias 命名
- [fixtures/local/opensearch-indices-manifest.json](/home/luna/Documents/DataB/fixtures/local/opensearch-indices-manifest.json)
  - 本地初始化清单也仍沿另一套 alias / physical index 命名

## 5. 任务目标

把搜索 alias 的命名、权威源和阶段边界收敛为单一答案，确保：

1. `search.index_alias_binding` 与运行时默认值一致
2. 初始化脚本、应用代码、runbook、ops 接口使用同一 alias 字典
3. `alias switch` 明确属于当前阶段正式能力
4. 搜索重建、切换、对账、runbook 能围绕同一套 alias 工作

## 6. 强约束

1. 不能只改脚本，不改应用默认值
2. 不能只改应用默认值，不改 migration / PRD / runbook 口径
3. 不能继续保留两套 alias 命名作为默认主路径
4. 不能回避阶段边界冲突，必须明确 `V1` 与 `V3` 口径
5. 不能只把 `search.index_alias_binding` 当表结构摆设，运行时应有明确对应关系

## 7. 建议修复方案

### 7.1 先冻结唯一 alias 字典

建议新增并统一引用一份搜索 alias 配置/字典，至少覆盖：

- product read alias = `product_search_read`
- product write alias = `product_search_write`
- seller read alias = `seller_search_read`
- seller write alias = `seller_search_write`
- sync job / admin 辅助索引的命名方式

该字典应成为：

- migration seed
- OpenSearch init 脚本
- 应用默认配置
- ops 搜索重建/切换逻辑
- runbook

的统一执行基线。

### 7.2 明确 `search.index_alias_binding` 是否为正式 authority

本任务直接收口为：

- `search.index_alias_binding` 就是运行时 alias 绑定的正式 authority
- 运行时代码、初始化脚本、默认配置、ops 接口必须读取或至少严格服从该 authority
- 不再允许把该表继续当作“只是规划型占位”

### 7.3 收口阶段边界

必须统一为唯一答案：

- `V1` 当前阶段支持 alias switch 的最小运维能力
- 接口协议、任务清单、runbook、代码、初始化脚本都要按这个答案收口
- `V3` 只承接更高阶的扩展能力，不再承接“首次提供 alias switch”

### 7.4 清理硬编码 alias

重点清理：

- `infra/opensearch/init-opensearch.sh`
- `apps/platform-core/src/lib.rs`
- `docs/04-runbooks/opensearch-local.md`
- 其他与搜索初始化、ops 接口、重建脚本相关的默认值

要求：

- 不再各自硬编码另一套运行时 alias
- 最终别名命名与冻结口径一致

## 8. 实施范围

至少覆盖以下内容：

### 8.1 文档与 schema

- `docs/原始PRD/商品搜索、排序与索引同步设计.md`
- `docs/数据库设计/V1/upgrade/057_search_sync_architecture.sql`
- `docs/数据库设计/接口协议/商品搜索、排序与索引同步接口协议正式版.md`
- `docs/开发任务/v1-core-开发任务清单.md`

### 8.2 运行时与脚本

- `apps/platform-core/src/lib.rs`
- `infra/opensearch/init-opensearch.sh`
- `docs/04-runbooks/opensearch-local.md`
- 相关搜索初始化/重建/切换脚本

### 8.3 运维与接口

- `AUD-022` 对应的搜索同步状态、重建、别名切换、缓存失效接口
- `SEARCHREC-007` 的搜索同步作业与异常记录
- `SEARCHREC-015` 的一致性测试

## 9. 验收标准

### 9.1 静态收口

以下状态必须成立：

- 搜索 alias 命名只剩一套正式默认值
- `search.index_alias_binding` 与运行时默认值不再冲突
- PRD / migration / runbook / 代码 / 初始化脚本使用同一套 alias 口径
- `alias switch` 的阶段归属只有一个正式答案

### 9.2 动态验证

至少验证：

1. OpenSearch 初始化后别名与结构化权威源一致
2. 搜索重建能写入 write alias
3. 搜索查询读取 read alias
4. alias switch 作为当前阶段最小运维能力可真实执行并留痕

### 9.3 运维可证明性

修复后应能明确回答：

- 当前 product / seller 搜索 read/write alias 是什么
- alias 的结构化权威源是什么
- 当前阶段是否支持 alias switch
- 答案必须唯一为“支持，并作为 V1 最小运维能力提供”
- runbook、ops 页面、初始化脚本是否基于同一口径

若这些问题仍有多套答案，则视为未完成收口。

### 9.4 当前收口结果

截至 `2026-04-20`，本任务已完成首轮命名与 authority 收口，当前冻结口径如下：

- 正式 read/write alias 固定为 `product_search_read`、`product_search_write`、`seller_search_read`、`seller_search_write`
- `search.index_alias_binding` 继续作为 product / seller 搜索 alias 的结构化 authority
- `search_sync_jobs_v1` 被收口为辅助运维索引名，不再伪装成 alias authority 的一部分
- `platform-core` 启动自检已切换为校验 `4` 个 alias 和 `1` 个 sync jobs index
- `infra/opensearch/init-opensearch.sh` 已切换为创建 `product_search_v1_bootstrap`、`seller_search_v1_bootstrap` 并挂接读写双 alias
- 初始化脚本重跑时会先清理历史 `catalog_products_v1` / `seller_profiles_v1` / `search_sync_jobs_v1_000001` 遗留索引，避免旧 alias 与当前索引名继续冲突
- compose、本地 env、runbook、fixture manifest、任务清单已同步到同一套命名

## 10. 输出要求

修复该问题的 Agent 在处理时应输出：

1. 变更文件清单
2. 最终 alias 字典
3. `PRD / migration / code / script / runbook` 映射对齐结果
4. `V1 / V3` 阶段边界结论
5. 初始化、重建、切换相关测试与联调结果

## 11. 一句话结论

`A08` 的核心问题不是“别名叫啥”，而是搜索 alias 的结构化权威源、运行时默认值和阶段边界同时发生冲突；如果不先收口这一点，后续搜索重建、切换和运维接口都会建立在不一致基线上。
