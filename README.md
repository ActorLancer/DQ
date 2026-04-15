# DataB V1-Core Repository

本仓库用于实现“区块链数据交易平台”的 `V1-Core` 最小可信交易闭环，执行以 `docs/开发任务/v1-core-开发任务清单.csv` 为唯一任务源。

## 仓库定位

- 架构：模块化单体 `platform-core` + 外围独立进程
- 阶段：仅 `V1-Core`，`V2/V3` 只做预留边界
- 原则：优先复用现有骨架与脚本，增量推进

## 目录总览

- `apps/`：主应用与外围应用骨架
- `packages/`：共享契约/类型/测试夹具
- `infra/`：基础设施与运维配置
- `scripts/`：本地环境与校验脚本
- `docs/`：冻结文档、任务与执行记录

## 执行入口

1. `docs/开发任务/v1-core-开发任务清单.csv`
2. `docs/开发任务/Agent-开发与半人工审核流程.md`
3. `docs/00-context/`
4. `docs/开发任务/V1-Core-实施进度日志.md`

## 本地开发入口

- 本地编排：`部署脚本/docker-compose.local.yml`
- 环境检查：`scripts/check-local-env.sh`
- 栈验证：`scripts/verify-local-stack.sh`
- 迁移校验：`scripts/validate_database_migrations.sh`

## 约束

- 不允许在未完成审批时进入下一批。
- 不允许将 `V1-gap` 伪装为 `V2/V3-reserved`。
- 不允许无依据修改冻结业务边界与生命周期主对象。
