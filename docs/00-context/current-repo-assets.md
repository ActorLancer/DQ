# 当前仓库资产盘点（CTX-022）

## 1. 盘点范围

- 目录骨架
- 应用骨架
- 脚本与配置
- 文档资产

盘点口径：`exists / partial / missing`

## 2. 顶层目录资产

| 资产 | 状态 | 说明 |
| --- | --- | --- |
| `apps/` | exists | 已有 9 个应用目录，名称与冻结服务清单一致。 |
| `packages/` | exists | 已有 `api-contracts / event-contracts / domain-types / test-fixtures`。 |
| `infra/` | partial | 目录存在，但多数子域仅骨架，实质配置文件较少。 |
| `scripts/` | partial | 已有环境检查与迁移校验脚本，但未形成完整统一入口集合。 |
| `docs/` | exists | 冻结文档与任务文档齐全，且已补充 `docs/00-context/`。 |
| `tests/` | partial | 目录存在，未见跨应用测试资产。 |
| `tools/` | partial | 目录存在，未见具体工具实现。 |
| `.github/` | missing | 当前未发现 CI 工作流目录。 |

## 3. 应用资产

| 应用 | 状态 | 说明 |
| --- | --- | --- |
| `apps/platform-core` | partial | 已有 Rust 最小服务（`/healthz`），模块目录未完整展开。 |
| `apps/portal-web` | partial | 仅 `README.md`。 |
| `apps/console-web` | partial | 仅 `README.md`。 |
| `apps/fabric-adapter` | partial | 仅 `README.md`。 |
| `apps/fabric-event-listener` | partial | 仅 `README.md`。 |
| `apps/search-indexer` | partial | 仅 `README.md`。 |
| `apps/data-processing-worker` | partial | 仅 `README.md`。 |
| `apps/notification-worker` | partial | 仅 `README.md`。 |
| `apps/mock-payment-provider` | partial | 仅 `README.md`。 |

## 4. 配置与部署资产

| 资产 | 状态 | 说明 |
| --- | --- | --- |
| `部署脚本/docker-compose.local.yml` | exists | 本地核心中间件编排已存在。 |
| `部署脚本/docker-compose.postgres-test.yml` | exists | 数据库迁移测试编排存在。 |
| `scripts/validate_database_migrations.sh` | exists | 迁移校验脚本存在。 |
| `scripts/check-local-env.sh` | exists | 本地环境可用性检查脚本存在。 |
| `scripts/verify-local-stack.sh` | exists | 本地栈探活脚本存在。 |
| `infra/docker/monitoring/*.yml` | exists | Prometheus/Loki/Tempo 配置存在。 |
| `README.md`（仓库根） | missing | 根目录说明文件缺失。 |

## 5. 结论（供后续任务引用）

1. 当前仓库已具备“单仓多应用 + 文档冻结 + 本地编排”基础事实，可按增量方式推进，不需要重建骨架。
2. 主要缺口在“应用实现深度”“CI 与自动化测试”“infra 子域实质配置补齐”。
3. 后续 BOOT/ENV/CORE 任务应以复用已有目录和脚本为优先策略，避免重复搭建。
