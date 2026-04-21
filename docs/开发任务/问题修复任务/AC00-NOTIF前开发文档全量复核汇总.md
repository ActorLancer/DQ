# NOTIF 前开发文档全量复核汇总

## 1. 复核范围

本轮复核聚焦 `NOTIF` 阶段进入开发前的执行准备状态，重点覆盖：

- 执行源与阅读版：
  - `docs/开发任务/v1-core-开发任务清单.csv`
  - `docs/开发任务/v1-core-开发任务清单.md`
  - `docs/开发任务/V1-Core-TODO与预留清单.md`
- 上下文与架构索引：
  - `docs/00-context/**`
  - `docs/01-architecture/**`
  - 仓库根 `README.md`
- 运行与验收说明：
  - `docs/04-runbooks/**`
  - `docs/05-test-cases/**`
  - `packages/openapi/**`
- 环境/目录/服务边界冻结文档：
  - `docs/开发准备/**`
- 必要的仓库事实核对：
  - `apps/notification-worker/`
  - `services/README.md`
  - `workers/README.md`
  - `Makefile`
  - `scripts/**`
  - `infra/**`

本轮不是代码实现审计，但为判断文档是否失真，补查了少量目录与代码事实。

## 2. 当前总体判断

`NOTIF` 前的文档口径已经较前几轮明显收敛，以下高风险问题已不再构成当前阻塞：

- `notification-worker / fabric-adapter` 的正式消费入口已统一为专用 topic 单入口，不再存在 `dtp.outbox.domain-events` 双入口主链冲突。
- `A01~A15` 已完成“历史问题起点 / 当前状态 / 剩余未完成项”的治理文档收口，不再把旧问题现状直接冒充当前状态。
- OpenAPI README、测试 README、运行模式、Kafka 地址边界、`check-topic-topology.sh` 与 `smoke-local.sh` 的职责边界已完成收口。

但进入 `NOTIF` 开发前，仍存在 3 组会直接影响开发判断的剩余问题：

1. `NOTIF-001~009` 仍以“部分完成”的状态配合“运行时可验证”的完成定义/验收语句存在，容易误导实现批次。
2. 多份架构/目录索引文档仍把历史占位目录 `apps/search-indexer`、`apps/fabric-adapter` 等写成当前正式运行时落位。
3. 少数 onboarding / 现状盘点文档仍保留过期仓库事实，如旧 compose 主入口、缺失的根 `README` / `.github`、缺失的 topic/bucket init 与 observability 组件。

## 3. 已复核的关键证据

- `v1-core-开发任务清单.csv` / `.md`
  - `NOTIF-001~009` 仍为 `partial`，但完成定义/验收普遍写成“Worker 可消费并发送 mock 通知、模板/幂等/重试可验证、触发事件后可见发送记录/重试结果”。
  - `NOTIF-010 / 012 / TEST-027` 已被降为当前批次仅冻结目标与承接关系，不再伪装成已跑通。
- `docs/04-runbooks/notification-worker.md`
  - 已明确当前批次只冻结 topic、consumer group、V1 渠道边界与后续 OpenAPI / test-case 承接，不代表正式实现已完成。
- `docs/05-test-cases/README.md`
  - 已明确 `notification-cases.md` 尚未落盘，需要在 `NOTIF` 实现批次补齐。
- `apps/notification-worker/README.md`
  - 当前通知进程正式落位目录仍只有说明文件，未见正式实现代码。
- `services/README.md` / `workers/README.md`
  - 已明确当前正式落位：
    - `services/fabric-adapter`
    - `services/fabric-event-listener`
    - `services/mock-payment-provider`
    - `workers/search-indexer`
    - `workers/recommendation-aggregator`
  - `apps/search-indexer` 仅保留历史说明，不再是当前权威入口。
- `docs/01-architecture/service-runtime-map.md`
  - 仍将 `apps/fabric-adapter`、`apps/search-indexer` 写成当前外围独立进程落位。
- `docs/00-context/service-to-module-map.md`
  - 仍将 `search-service` 映射到 `apps/search-indexer`，`fabric-adapter-service` 映射到 `apps/fabric-adapter`。
- `docs/00-context/current-repo-assets.md`
  - 仍写根 `README.md` 缺失、`.github/` 缺失、topic/bucket init 缺失、`Alertmanager` / `otel-collector` 缺失、`Fabric` 启动脚本缺失。
- 仓库事实核对：
  - 根 `README.md` 已存在。
  - `.github/workflows/build.yml`、`lint.yml`、`test.yml` 已存在。
  - `infra/docker/docker-compose.local.yml` 已是正式主入口。
  - `infra/kafka/init-topics.sh`、`infra/minio/init-minio.sh`、`infra/fabric/*.sh` 已存在。

## 4. 问题清单

### [AC00-001]
- 严重级别：`blocker`
- 问题分类：`任务未承接`、`DoD/acceptance 不充分`、`阶段边界漂移`
- 关联阶段：`NOTIF`
- 关联任务ID：`NOTIF-001~009`
- 现象：
  - `NOTIF-001~009` 仍处于 `partial`，但完成定义/验收语句普遍写成“通知 Worker 可消费事件并发送 mock 通知”“模板/幂等/重试可验证”“触发对应事件后能看到发送记录、幂等去重和失败重试结果”。
  - 与此同时，`apps/notification-worker` 当前仍仅为说明文件占位，runbook / test README / TODO 也仍明确通知正式实现、OpenAPI、验收清单尚未闭环。
- 正确口径：
  - 在正式进入 `NOTIF` 代码实现批次前，`NOTIF-001~009` 若继续保持未完成状态，其 DoD/验收只能表达“设计边界已冻结、后续实现应达到的目标”，不能写成当前已经可通过运行时验证。
  - 若确实尚未开始或仅冻结边界，应进一步降为 `limited`，或改写 DoD/验收使其停留在文档/边界级。
- 当前文档/任务现状：
  - `docs/开发任务/v1-core-开发任务清单.csv:338-346`
  - `docs/开发任务/v1-core-开发任务清单.md:2469-2532`
  - `docs/04-runbooks/notification-worker.md:12-20`
  - `docs/05-test-cases/README.md:24-29`
  - `docs/开发任务/V1-Core-TODO与预留清单.md:65`
  - `apps/notification-worker/README.md:1`
- 风险：
  - 后续 `NOTIF` 开发批次会误以为通知 Worker 基础运行能力、幂等/重试、mock-log 发送与审计联查已经具备，导致跳过真正的骨架实现和测试落盘。
  - `NOTIF-001~009` 会继续给出“边界未闭合，但任务像已做过半”的错误信号。
- 建议修复方向：
  - 将 `NOTIF-001~009` 统一改成当前真实状态：
    - 若仅冻结边界：`limited`
    - 若确有部分落盘资产：保留 `partial`，但把 DoD/验收改为“当前仅完成哪些冻结项”
  - 对这 9 条统一追加：
    - `docs/04-runbooks/notification-worker.md`
    - `docs/05-test-cases/README.md`
    - `V1-Core-TODO与预留清单.md`
    作为当前真实边界参考
  - 明确写出：正式运行时发送、幂等、重试、DLQ、审计联查、通知验收矩阵，均以 `NOTIF-010~014` 与后续实现批次为准。
- 是否必须在进入对应阶段前修复：`yes`

### [AC00-002]
- 严重级别：`high`
- 问题分类：`说明文件未同步`、`推荐阅读不足`、`承接不完整`
- 关联阶段：`AUD / SEARCHREC / ENV / cross-stage`
- 关联任务ID：`CTX-019`、`CTX-022`、`BOOT-029~032`、`AUD-013~017`、`SEARCHREC-001~017`
- 现象：
  - 多份架构/目录索引文档仍把历史 `apps/*` 占位目录写成当前正式运行时落位。
  - 当前正式落位已被 `services/README.md` / `workers/README.md` 改写，但这些索引文档尚未同步。
- 正确口径：
  - 当前正式运行时落位应以：
    - `services/` 承载 `fabric-adapter`、`fabric-event-listener`、`mock-payment-provider`
    - `workers/` 承载 `search-indexer`、`recommendation-aggregator`、`outbox-publisher`
    - `apps/notification-worker` 继续承载通知进程
  - `apps/search-indexer`、`apps/fabric-adapter` 等只能作为历史说明，不得继续在架构索引里被写成当前正式进程入口。
- 当前文档/任务现状：
  - `docs/01-architecture/service-runtime-map.md:10-21`
  - `docs/01-architecture/service-worker-package-layout.md:12-31`
  - `docs/00-context/service-to-module-map.md:21-24`
  - `docs/00-context/architecture-style.md:21-31`
  - `docs/00-context/current-repo-assets.md:30-37`
  - `docs/开发准备/仓库拆分与目录结构建议.md:70-78,229-311`
  - 对照真实权威说明：
    - `services/README.md:1-10`
    - `workers/README.md:1-21`
- 风险：
  - 进入 `AUD` / `SEARCHREC` / Fabric 实现批次时，开发者会被带到历史占位目录，重复在错误目录落代码或写 runbook。
  - 任务的 `technical_reference` 若先指向这些索引文档，会削弱当前目录边界的唯一性。
- 建议修复方向：
  - 一次性同步：
    - `service-runtime-map.md`
    - `service-worker-package-layout.md`
    - `service-to-module-map.md`
    - `architecture-style.md`
    - `current-repo-assets.md`
    - `仓库拆分与目录结构建议.md`
  - 统一说明：
    - `apps/*` 中相关目录为历史占位或说明
    - 当前正式实现入口以 `services/README.md`、`workers/README.md` 为准
  - 对 `CTX-019 / CTX-022 / BOOT-029~032` 的 `technical_reference` 追加 `services/README.md`、`workers/README.md`，避免只读旧索引。
- 是否必须在进入对应阶段前修复：`yes`

### [AC00-003]
- 严重级别：`medium`
- 问题分类：`说明文件未同步`、`推荐阅读不足`
- 关联阶段：`CTX / ENV / onboarding`
- 关联任务ID：`CTX-014`、`CTX-022`、`CTX-023`
- 现象：
  - 少数 onboarding / 仓库现状盘点文档仍传播过期事实：
    - 根 `README.md` 仍把本地编排主入口写成 `部署脚本/docker-compose.local.yml`
    - `current-repo-assets.md` 仍写根 `README.md` 缺失、`.github/` 缺失、`topic/bucket init` 缺失、`Alertmanager` / `otel-collector` / Fabric 启动脚本缺失
    - `current-gap-analysis.md` 仍写 `apps/platform-core` 是“最小服务（Rust + /healthz）”
- 正确口径：
  - onboarding / 盘点文档应反映当前仓库事实：
    - 本地主入口：`infra/docker/docker-compose.local.yml` + `Makefile` / `scripts/`
    - 根 `README.md` 与 `.github/workflows/*` 已存在
    - topic / bucket 初始化脚本与 Fabric 脚本已存在
    - `platform-core` 已不再是仅 `/healthz` 的最小骨架
- 当前文档/任务现状：
  - `README.md:28`
  - `docs/00-context/current-repo-assets.md:12-19,28-53`
  - `docs/00-context/current-gap-analysis.md:5-18`
  - 对照真实仓库事实：
    - `README.md:1`
    - `.github/README.md:1`
    - `.github/workflows/build.yml:1`
    - `infra/kafka/init-topics.sh:1`
    - `infra/minio/init-minio.sh:1`
    - `infra/fabric/fabric-up.sh:1`
    - `scripts/README.md:1`
    - `docs/04-runbooks/local-startup.md:1`
- 风险：
  - 新一轮开发前的 onboarding 会先读到旧主入口和旧仓库事实，增加无效排查和路径判断成本。
  - `CTX-014 / 022 / 023` 会继续给出“仓库还缺这些基础资产”的旧印象。
- 建议修复方向：
  - 更新根 `README.md` 的本地开发入口为：
    - `make up-local / make up-mocks / make up-demo`
    - `infra/docker/docker-compose.local.yml`
    - `scripts/README.md`
  - 重写 `current-repo-assets.md` 与 `current-gap-analysis.md` 的现状描述，只保留今天仍然缺失的内容。
  - 复核 `CTX-014 / 022 / 023` 是否仍引用这些旧现状，必要时同步调整 `technical_reference`。
- 是否必须在进入对应阶段前修复：`no`

## 5. 已收缩、当前无问题项

- `notification-worker / fabric-adapter` 的正式消费入口已统一为专用 topic 单入口。
  - 证据：
    - `docs/04-runbooks/notification-worker.md`
    - `docs/04-runbooks/fabric-local.md`
    - `docs/开发准备/事件模型与Topic清单正式版.md`
    - `infra/kafka/topics.v1.json`
    - `scripts/check-topic-topology.sh`
- `A01~A15` 已完成治理文档化，不再直接把历史问题现状当当前状态。
  - 证据：
    - `docs/开发任务/问题修复任务/README.md`
    - `docs/开发任务/问题修复任务/A01~A15`
- `check-topic-topology.sh` 与 `smoke-local.sh` 的职责边界已冻结。
  - 证据：
    - `docs/04-runbooks/kafka-topics.md`
    - `docs/04-runbooks/notification-worker.md`
    - `docs/05-test-cases/README.md`
- 运行模式已统一为 `local / staging / demo`，`mocks` 与 `staging-like local` 只是 `local` 子场景。
  - 证据：
    - `docs/00-context/run-modes.md`
    - `docs/00-context/local-deployment-boundary.md`
    - `docs/04-runbooks/compose-profiles.md`

## 6. 进入 NOTIF 前的优先处理顺序

1. **先修 `AC00-001`**
   - 否则 `NOTIF` 阶段入口本身就带着“像已经做过半”的错误信号。
2. **再修 `AC00-002`**
   - 否则 `SEARCHREC / AUD / Fabric` 后续实现仍会被历史占位目录带偏。
3. **最后处理 `AC00-003`**
   - 主要影响 onboarding 和阅读效率，不直接阻塞 `NOTIF` 阶段实现。

## 7. 当前 readiness 结论

当前项目在 `NOTIF` 前的文档 readiness **已比前几轮显著提升，但还不足以直接无风险进入 `NOTIF` 实现批次**。

最核心的剩余阻塞不是架构设计本身，而是：

- `NOTIF` 执行源仍残留“运行时已可验证”的旧任务状态与验收口径；
- 架构/目录索引文档仍在传播历史占位目录，影响后续真实落位判断。

只要完成 `AC00-001` 与 `AC00-002`，本轮复核范围内的高风险文档冲突就基本清空，之后进入 `NOTIF` 开发会稳很多。
