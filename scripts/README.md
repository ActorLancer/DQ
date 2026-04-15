# scripts 目录说明

## 职责

- 提供本地开发、联调、校验、重置等统一脚本入口。

## 边界

- 脚本用于编排和检查，不替代业务模块实现。

## 依赖

- 依赖 `部署脚本/` 的 compose 文件和根 `Makefile` 目标。

## 常用脚本

- `up-local.sh` / `down-local.sh`：本地基础栈启停。
- `check-local-stack.sh`：本地依赖健康检查。
- `prune-local.sh`：安全清理当前仓库本地卷、网络、Fabric 状态（默认 `--dry-run`）。
- `export-local-config.sh`：导出 compose 解析后的只读快照。
- `smoke-local.sh`：执行本地环境 smoke 套件（DB 迁移探测、bucket/topic/realm/Grafana/mock-payment）。

## 禁止事项

- 禁止在脚本中写死环境专属参数。
- 禁止绕过 `Makefile` 私自新增重复入口。
