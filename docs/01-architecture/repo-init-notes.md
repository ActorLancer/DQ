# 仓库增量初始化说明（BOOT-024）

## 1. 目标

基于当前仓库现状执行增量初始化，不覆盖已可用资产，不做无意义重建。

## 2. 已复用资产

- `apps/`, `packages/`, `infra/`, `scripts/`, `docs/` 既有目录骨架
- `部署脚本/docker-compose.local.yml` 与本地校验脚本
- `docs/开发任务/` 与 `docs/开发准备/` 冻结文档集

## 3. 本批新增/校准内容

- 根目录 `README.md`
- `.gitignore`（补充 `.env.*.example` 放行）
- `.editorconfig`
- `.gitattributes`
- `.env.example` / `.env.local.example` / `.env.staging.example` / `.env.demo.example`

## 4. 禁止覆盖项

- 已存在且被流程引用的任务清单、流程文件、进度日志、审批记录
- 已运行中的本地 compose 与脚本入口
- `docs/00-context/` 已冻结边界文档

## 5. 后续迁移项（由后续任务处理）

- `部署脚本/` 与 `infra/docker/` 路径收敛
- `services/` 与 `workers/` 目录策略落位
- `.github/workflows` 与 CI 流程补齐
