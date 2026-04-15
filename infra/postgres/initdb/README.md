# PostgreSQL InitDB（ENV-005）

本目录中的脚本会在 PostgreSQL 首次初始化数据目录时自动执行。

## 目标

- 创建业务 schema
- 启用基础扩展
- 创建本地最小角色与权限

## 说明

- 仅用于 `local` / `demo` 环境。
- 变更后若需重放，请清理 `postgres_data` 卷再重启。
