# 本地部署边界（CTX-020）

本文件冻结本地部署职责边界，防止把基础设施编排和业务应用编排混成一个无限膨胀的 compose。

## 边界结论（冻结）

- `docker-compose.local.yml` 只负责：
  - 中间件：PostgreSQL、Redis、Kafka、MinIO、OpenSearch、可观测性组件
  - mock/provider 辅助能力（如需要）
  - Fabric 测试网络相关依赖（通过 profile 控制）
- 业务应用默认通过本机进程启动（`make run-*` / `cargo run` 等），不默认塞进基础设施 compose。
- 若确需容器化业务应用联调，使用独立文件 `docker-compose.apps.local.yml`（或当前阶段的 `*.example.yml` 占位），与基础设施编排解耦。

## 文件职责

| 文件 | 职责 | 非职责 |
| --- | --- | --- |
| `docker-compose.local.yml` | 本地基础设施与可观测组件编排 | 不承载所有业务应用容器 |
| `docker-compose.apps.local.yml` | 可选的应用容器联调编排（独立维护） | 不重复声明基础设施全部组件 |
| `scripts/check-local-stack.sh` | 基础设施探活与就绪检查 | 不替代业务接口联调测试 |

## 运行建议

1. 先启动基础设施：`make up-local`（或 profile 子集）。
2. 再启动业务应用：`platform-core` 等本机进程。
3. 使用检查脚本做健康检查，再执行业务联调或集成测试。

## 约束

- 禁止把 `platform-core`、`search-indexer`、`notification-service`、`fabric-adapter` 全部直接塞进 `docker-compose.local.yml`。
- 新增本地联调需求时，优先扩展 `docker-compose.apps.local.yml`，而不是污染基础设施 compose。
- 所有边界变更必须同步更新 runbook 与实施日志。
